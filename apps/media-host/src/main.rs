//! ZeroWeb media-host desktop process entry.
//!
//! stdio 拆两半承载 [`RoleIo`]：stdin 读线程阻塞 recv → mpsc（对端断开即落线程），
//! 主线程 `recv_timeout` 等消息——超时即 Idle，内核时钟推进（host 自走时钟，
//! DESIGN §10；`IpcChannel` 的阻塞 recv 无超时面，是拆分的根因）。stdout 写半
//! 承载回写（`io::Stdout` 句柄克隆共享同一全局流，读写分线程无竞争）。

#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

use std::io::{self, Write};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use zero_media_host::{FeedTick, MediaKernel, RoleIo, run_role_feed};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, IpcMessage};

/// 生产 stdio IO 面：读半在线程、写半在主线程（模块文档）。
struct StdioRoleIo {
    writer: PipeTransport<io::Empty, io::Stdout>,
    feed: mpsc::Receiver<IpcMessage>,
}

impl RoleIo for StdioRoleIo {
    fn next_message(&mut self, timeout: Duration) -> FeedTick {
        match self.feed.recv_timeout(timeout) {
            Ok(message) => FeedTick::Message(Box::new(message)),
            Err(RecvTimeoutError::Timeout) => FeedTick::Idle,
            Err(RecvTimeoutError::Disconnected) => FeedTick::Disconnected,
        }
    }

    fn send_reply(&mut self, message: IpcMessage) -> bool {
        match self.writer.send(message) {
            Ok(()) => true,
            Err(error) => {
                tracing::warn!("media-host: response write failed: {error}");
                false
            }
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_writer(io::stderr)
        .init();

    // 读半：stdin 阻塞 recv 专线程（decode 错误/EOF = 对端断开，落线程使
    // mpsc 断开 → 主循环 Disconnected）。写半：stdout 承载回写。
    let (sender, feed) = mpsc::channel::<IpcMessage>();
    let reader = thread::Builder::new()
        .name("media-host-stdin".into())
        .spawn(move || {
            let mut transport = PipeTransport::new(io::stdin(), io::sink());
            loop {
                match transport.recv() {
                    Ok(message) => {
                        if sender.send(message).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        if !zero_protocol::is_disconnected_channel_message(&error.to_string()) {
                            tracing::warn!("media-host: stdin read failed: {error}");
                            let _ = io::stderr().flush();
                        }
                        break;
                    }
                }
            }
        })
        .expect("media-host: spawn stdin reader thread");

    let writer = PipeTransport::new(io::empty(), io::stdout());
    let mut role = StdioRoleIo { writer, feed };
    let start = Instant::now();
    run_role_feed(&mut role, &mut MediaKernel::new(), &mut || {
        start.elapsed().as_millis() as u64
    });
    // 读线程随对端断开/主循环退出自然终结；正常 shutdown 时显式等待，不留悬线程。
    let _ = reader.join();
}

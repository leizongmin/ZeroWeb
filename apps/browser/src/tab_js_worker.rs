//! 标签页专用 JS 线程 — V8 与布局/绘制 worker 分离。

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_browser_shell::TabId;
use zero_script_sandbox::{SandboxConfig, V8Sandbox};

type ScriptFn = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

enum JsWorkerCommand {
    Execute {
        script: String,
        reply: Sender<Result<String, String>>,
    },
    Shutdown,
}

/// 专用 JS worker 句柄（每 Tab 一个）。
pub struct TabJsWorkerHandle {
    cmd_tx: Sender<JsWorkerCommand>,
    join: Option<JoinHandle<()>>,
    executor: ScriptFn,
}

impl TabJsWorkerHandle {
    /// 启动 JS 专用线程。
    pub fn spawn(tab_id: TabId) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let cmd_for_exec = cmd_tx.clone();

        let join = thread::Builder::new()
            .name(format!("tab-js-{}", tab_id.0))
            .spawn(move || js_worker_main(cmd_rx))
            .expect("spawn tab js worker");

        let executor: ScriptFn = Arc::new(move |script: &str| {
            let (reply_tx, reply_rx) = mpsc::channel();
            cmd_for_exec
                .send(JsWorkerCommand::Execute {
                    script: script.to_string(),
                    reply: reply_tx,
                })
                .map_err(|e| e.to_string())?;
            reply_rx
                .recv_timeout(Duration::from_secs(30))
                .map_err(|e| e.to_string())?
        });

        Self {
            cmd_tx,
            join: Some(join),
            executor,
        }
    }

    /// 供 WebView 注入的外部脚本执行器。
    pub fn executor(&self) -> ScriptFn {
        Arc::clone(&self.executor)
    }

    /// 关闭 JS 线程。
    pub fn shutdown(&mut self) {
        let _ = self.cmd_tx.send(JsWorkerCommand::Shutdown);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for TabJsWorkerHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn js_worker_main(cmd_rx: Receiver<JsWorkerCommand>) {
    let js_config = SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(js_config).expect("V8 sandbox init");

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            JsWorkerCommand::Execute { script, reply } => {
                let result = sandbox.execute(&script).map(|r| r.value).map_err(|e| e.to_string());
                let _ = reply.send(result);
            }
            JsWorkerCommand::Shutdown => break,
        }
    }
}

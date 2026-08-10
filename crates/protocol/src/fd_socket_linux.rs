//! Linux Unix socket 辅助通道：经 SCM_RIGHTS 传递 dma-buf fd（RFC 4.3 真纹理零拷贝）。

use std::io::{IoSlice, IoSliceMut};
use std::mem::size_of;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::Duration;

use crate::ProtocolError;

const FD_SOCKET_PREFIX: &str = "zeroweb-fd-";

/// fd 辅助 socket 路径。
pub fn fd_socket_path(name: &str) -> PathBuf {
    PathBuf::from(format!("/dev/shm/{FD_SOCKET_PREFIX}{name}"))
}

/// compositor 侧：bind 并等待单次连接，经 SCM_RIGHTS 发送 `fd`。
pub fn publish_fd(name: &str, fd: RawFd, accept_timeout: Duration) -> Result<(), ProtocolError> {
    let path = fd_socket_path(name);
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
    let listener =
        UnixListener::bind(&path).map_err(|e| ProtocolError::Channel(format!("fd socket bind 失败: {e}")))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| ProtocolError::Channel(format!("fd socket nonblocking 失败: {e}")))?;

    let deadline = std::time::Instant::now() + accept_timeout;
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= deadline {
                    let _ = std::fs::remove_file(&path);
                    return Err(ProtocolError::Channel("fd socket accept 超时".into()));
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(error) => {
                let _ = std::fs::remove_file(&path);
                return Err(ProtocolError::Channel(format!("fd socket accept 失败: {error}")));
            }
        }
    };
    send_fd(&stream, fd)?;
    let _ = std::fs::remove_file(&path);
    Ok(())
}

/// Browser 侧：connect 并 recv fd。
pub fn consume_fd(name: &str, connect_timeout: Duration) -> Result<OwnedFd, ProtocolError> {
    let path = fd_socket_path(name);
    let deadline = std::time::Instant::now() + connect_timeout;
    let stream = loop {
        match UnixStream::connect(&path) {
            Ok(stream) => break stream,
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound
                    || error.kind() == std::io::ErrorKind::ConnectionRefused =>
            {
                if std::time::Instant::now() >= deadline {
                    return Err(ProtocolError::Channel(format!("fd socket connect 超时: {path:?}")));
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(error) => return Err(ProtocolError::Channel(format!("fd socket connect 失败: {error}"))),
        }
    };
    recv_fd(&stream)
}

/// fd socket 名。
pub fn fd_socket_name(surface_id: u64, frame_id: u64) -> String {
    format!("{surface_id}-{frame_id}-fd")
}

fn send_fd(stream: &UnixStream, fd: RawFd) -> Result<(), ProtocolError> {
    let data = [1u8];
    let iov = [IoSlice::new(&data)];
    let fds = [fd];
    sendmsg_fds(stream.as_raw_fd(), &iov, &fds)
}

fn recv_fd(stream: &UnixStream) -> Result<OwnedFd, ProtocolError> {
    let mut data = [0u8; 1];
    let mut iov = [IoSliceMut::new(&mut data)];
    let fd = recvmsg_fd(stream.as_raw_fd(), &mut iov)?;
    // SAFETY: SCM_RIGHTS 返回的新 fd 由本模块取得所有权。
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

fn sendmsg_fds(socket: RawFd, iov: &[IoSlice<'_>], fds: &[RawFd]) -> Result<(), ProtocolError> {
    let fd_size = size_of::<RawFd>();
    let cmsg_len = unsafe { libc::CMSG_SPACE(fd_size as u32) as usize };
    let mut cmsg = vec![0u8; cmsg_len];
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = iov.as_ptr() as *mut libc::iovec;
    msg.msg_iovlen = iov.len() as _;
    msg.msg_control = cmsg.as_mut_ptr() as *mut _;
    msg.msg_controllen = cmsg.len() as _;

    unsafe {
        let cptr = libc::CMSG_FIRSTHDR(&msg);
        if cptr.is_null() {
            return Err(ProtocolError::Channel("CMSG_FIRSTHDR 失败".into()));
        }
        (*cptr).cmsg_level = libc::SOL_SOCKET;
        (*cptr).cmsg_type = libc::SCM_RIGHTS;
        (*cptr).cmsg_len = libc::CMSG_LEN(fd_size as u32) as _;
        std::ptr::copy_nonoverlapping(fds.as_ptr(), libc::CMSG_DATA(cptr) as *mut RawFd, fds.len());
    }

    let sent = unsafe { libc::sendmsg(socket, &msg, 0) };
    if sent < 0 {
        return Err(ProtocolError::Channel(format!(
            "sendmsg 失败: {}",
            std::io::Error::last_os_error()
        )));
    }
    Ok(())
}

fn recvmsg_fd(socket: RawFd, iov: &mut [IoSliceMut<'_>]) -> Result<RawFd, ProtocolError> {
    let fd_size = size_of::<RawFd>();
    let cmsg_len = unsafe { libc::CMSG_SPACE(fd_size as u32) as usize };
    let mut cmsg = vec![0u8; cmsg_len];
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = iov.as_mut_ptr() as *mut libc::iovec;
    msg.msg_iovlen = iov.len() as _;
    msg.msg_control = cmsg.as_mut_ptr() as *mut _;
    msg.msg_controllen = cmsg.len() as _;

    let received = unsafe { libc::recvmsg(socket, &mut msg, 0) };
    if received < 0 {
        return Err(ProtocolError::Channel(format!(
            "recvmsg 失败: {}",
            std::io::Error::last_os_error()
        )));
    }

    unsafe {
        let mut cptr = libc::CMSG_FIRSTHDR(&msg);
        while !cptr.is_null() {
            if (*cptr).cmsg_level == libc::SOL_SOCKET && (*cptr).cmsg_type == libc::SCM_RIGHTS {
                let data = libc::CMSG_DATA(cptr) as *const RawFd;
                return Ok(*data);
            }
            cptr = libc::CMSG_NXTHDR(&msg, cptr);
        }
    }
    Err(ProtocolError::Channel("SCM_RIGHTS 未携带 fd".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn fd_socket_round_trip_memfd() {
        let name = format!("test-{}", std::process::id());
        let memfd = unsafe { libc::memfd_create(c"zeroweb-fd-test".as_ptr(), libc::MFD_CLOEXEC) };
        assert!(memfd >= 0, "memfd_create: {}", std::io::Error::last_os_error());

        let name_clone = name.clone();
        let handle = thread::spawn(move || publish_fd(&name_clone, memfd, Duration::from_secs(2)));

        thread::sleep(Duration::from_millis(20));
        let received = consume_fd(&name, Duration::from_secs(2)).expect("consume fd");
        handle.join().expect("publish join").expect("publish fd");
        assert!(received.as_raw_fd() >= 0);
        unsafe {
            libc::close(memfd);
        }
    }
}

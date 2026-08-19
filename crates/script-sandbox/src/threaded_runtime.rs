use crate::ScriptError;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

const TERMINATE_JOIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub(crate) struct ThreadedRuntimeCore<C, E> {
    command_sender: Sender<C>,
    event_receiver: Receiver<E>,
    thread_handle: Option<JoinHandle<()>>,
    terminate_flag: Arc<AtomicBool>,
    terminated: bool,
}

impl<C, E> ThreadedRuntimeCore<C, E>
where
    C: Send + 'static,
    E: Send + 'static,
{
    pub(crate) fn spawn<F>(thread_name: &str, error_subject: &str, run: F) -> Result<Self, ScriptError>
    where
        F: FnOnce(Receiver<C>, Sender<E>, Arc<AtomicBool>) + Send + 'static,
    {
        let (command_sender, command_receiver) = mpsc::channel();
        let (event_sender, event_receiver) = mpsc::channel();
        let terminate_flag = Arc::new(AtomicBool::new(false));
        let thread_terminate_flag = Arc::clone(&terminate_flag);
        let thread_handle = thread::Builder::new()
            .name(thread_name.to_string())
            .spawn(move || run(command_receiver, event_sender, thread_terminate_flag))
            .map_err(|error| {
                ScriptError::EngineUnavailable(format!("Failed to spawn {error_subject} thread: {error}"))
            })?;

        Ok(Self {
            command_sender,
            event_receiver,
            thread_handle: Some(thread_handle),
            terminate_flag,
            terminated: false,
        })
    }

    pub(crate) fn send(&self, command: C) -> Result<(), ()> {
        self.command_sender.send(command).map_err(|_| ())
    }

    pub(crate) fn try_recv(&self) -> Option<E> {
        self.event_receiver.try_recv().ok()
    }

    pub(crate) fn recv(&self) -> Result<E, ()> {
        self.event_receiver.recv().map_err(|_| ())
    }

    pub(crate) fn recv_timeout(&self, timeout: std::time::Duration) -> Result<E, mpsc::RecvTimeoutError> {
        self.event_receiver.recv_timeout(timeout)
    }

    pub(crate) fn terminate<F>(&mut self, command: C, interrupt: F)
    where
        F: FnOnce(),
    {
        if self.terminated {
            return;
        }
        self.terminate_flag.store(true, Ordering::Release);
        let _ = self.command_sender.send(command);
        interrupt();
        if let Some(handle) = self.thread_handle.take() {
            join_bounded_or_detach(handle);
        }
        self.terminated = true;
    }

    pub(crate) fn is_terminated(&self) -> bool {
        self.terminated
    }
}

fn join_bounded_or_detach(handle: JoinHandle<()>) {
    let start = std::time::Instant::now();
    while start.elapsed() < TERMINATE_JOIN_TIMEOUT {
        if handle.is_finished() {
            let _ = handle.join();
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(handle);
}

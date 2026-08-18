//! Shared image-decoder role entry used by desktop helpers and Android Services.

use zero_protocol::message::{ImageDecodeParams, ImageDecodeResultParams, IpcMessage, IpcMessageKind};
use zero_protocol::{IpcChannel, is_disconnected_channel_message};

#[cfg(test)]
use zero_protocol::ProtocolError;

/// Decodes image bytes with the same implementation used by the in-process fallback.
fn decode(mime: &str, bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    let image = zero_render_foundation::image_cache::decode_image_bytes(bytes)
        .map_err(|error| format!("decode failed ({mime}): {error}"))?;
    Ok((image.width, image.height, image.pixels))
}

/// Runs the isolated image-decoder role until its IPC peer disconnects.
///
/// The transport is supplied by the platform bootstrap: desktop uses stdio while
/// Android Services will provide a socket-backed implementation.
pub fn run_role<C: IpcChannel>(transport: &mut C) {
    tracing::info!("image-decoder: ready for decode requests");

    loop {
        let message: IpcMessage = match transport.recv() {
            Ok(message) => message,
            Err(error) => {
                if is_disconnected_channel_message(&error.to_string()) {
                    tracing::info!("image-decoder: IPC peer disconnected");
                    break;
                }
                tracing::warn!("image-decoder: receive failed: {error}");
                continue;
            }
        };

        match message.kind {
            IpcMessageKind::ImageDecodeRequest(ImageDecodeParams {
                request_id,
                mime,
                bytes,
            }) => {
                let (width, height, rgba, error) = match decode(&mime, &bytes) {
                    Ok((width, height, rgba)) => (width, height, rgba, None),
                    Err(error) => (0, 0, Vec::new(), Some(error)),
                };
                let response = IpcMessage {
                    id: message.id,
                    kind: IpcMessageKind::ImageDecodeResult(ImageDecodeResultParams {
                        request_id,
                        width,
                        height,
                        rgba,
                        error,
                    }),
                };
                if let Err(error) = transport.send(response) {
                    tracing::warn!("image-decoder: response write failed: {error}");
                    break;
                }
            }
            _ => tracing::warn!("image-decoder: ignored unsupported IPC message"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    struct TestChannel {
        inbound: VecDeque<IpcMessage>,
        outbound: Vec<IpcMessage>,
    }

    impl IpcChannel for TestChannel {
        fn send(&mut self, message: IpcMessage) -> Result<(), ProtocolError> {
            self.outbound.push(message);
            Ok(())
        }

        fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
            self.inbound
                .pop_front()
                .ok_or_else(|| ProtocolError::Channel("IPC 通道已关闭".to_string()))
        }

        fn try_recv(&mut self) -> Result<Option<IpcMessage>, ProtocolError> {
            Ok(self.inbound.pop_front())
        }

        fn close(&mut self) {}
    }

    #[test]
    fn role_returns_decode_error_without_crashing() {
        let mut channel = TestChannel {
            inbound: VecDeque::from([IpcMessage {
                id: 9,
                kind: IpcMessageKind::ImageDecodeRequest(ImageDecodeParams {
                    request_id: 7,
                    mime: "image/png".to_string(),
                    bytes: vec![0, 1, 2],
                }),
            }]),
            outbound: Vec::new(),
        };

        run_role(&mut channel);

        assert_eq!(channel.outbound.len(), 1);
        let response = &channel.outbound[0];
        assert_eq!(response.id, 9);
        match &response.kind {
            IpcMessageKind::ImageDecodeResult(result) => {
                assert_eq!(result.request_id, 7);
                assert_eq!(result.width, 0);
                assert_eq!(result.height, 0);
                assert!(result.rgba.is_empty());
                assert!(result.error.is_some());
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }
}

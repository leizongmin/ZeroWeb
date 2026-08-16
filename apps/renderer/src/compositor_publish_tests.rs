//! Renderer compositor 发布与故障回退测试。

use super::*;
use std::io::Write;
use std::sync::{Arc, Mutex};
use zero_engine::{HitTestCache, HitTestCacheSnapshot, HitTestLayoutSnapshot, HitTestNodeSnapshot, node_id_from_u64};
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};

#[derive(Clone)]
struct SharedBuf(Arc<Mutex<Vec<u8>>>);

impl Write for SharedBuf {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn drain_messages(bytes: &[u8]) -> Vec<IpcMessage> {
    let mut transport = PipeTransport::new(std::io::Cursor::new(bytes), std::io::empty());
    let mut messages = Vec::new();
    while let Ok(message) = transport.recv() {
        messages.push(message);
    }
    messages
}

fn sample_frame() -> zero_page_runtime::FrameModel {
    let root_id = node_id_from_u64(1);
    zero_page_runtime::FrameModel {
        viewport: (800, 600),
        document_height: 900.0,
        primitives: RenderPrimitives {
            fills: vec![FillPrimitive {
                rect: Rect::new(0.0, 0.0, 100.0, 100.0),
                color: Color::rgb(255, 0, 0),
            }],
            ..RenderPrimitives::new()
        },
        dirty_rects: vec![(0.0, 0.0, 800.0, 600.0)],
        hit_test: Some(HitTestCache::from_snapshot(HitTestCacheSnapshot {
            doc_root: root_id,
            layout_root: HitTestLayoutSnapshot {
                node_id: Some(root_id),
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 900.0,
                children: Vec::new(),
            },
            nodes: vec![(
                root_id,
                HitTestNodeSnapshot {
                    tag_name: "body".into(),
                    id: None,
                    class_name: None,
                    selector: "body".into(),
                    href: None,
                    src: None,
                },
            )],
            parents: Vec::new(),
        })),
    }
}

#[test]
fn publish_frame_emits_viewpainted_with_primitives() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
    let mut next_msg_id = 1_u64;
    let mut publish_state = FramePublishState::new(41, FramePublishMode::Legacy);
    let frame = sample_frame();
    publish_render_with_layout(
        &mut outbound,
        None,
        &mut next_msg_id,
        &mut publish_state,
        &frame,
        1.0,
        Some("smoke".into()),
        Vec::new(),
        7,
        3,
    )
    .expect("publish");

    let messages = drain_messages(&buf.0.lock().unwrap());
    assert!(
        messages
            .iter()
            .all(|message| !matches!(message.kind, IpcMessageKind::CompositorFrame { .. }))
    );
    let painted = messages
        .iter()
        .find_map(|message| match &message.kind {
            IpcMessageKind::ViewPainted(paint) => Some(paint.as_ref()),
            _ => None,
        })
        .expect("须产出 ViewPainted");
    assert!(!painted.fills.is_empty());
    assert_eq!(painted.navigation_epoch, 7);
    assert_eq!(painted.document_generation, 3);
}

#[test]
fn publish_frame_emits_compositor_sequence_with_full_paint_payload() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
    let mut next_msg_id = 1_u64;
    let mut publish_state = FramePublishState::new(73, FramePublishMode::Compositor);
    let frame = sample_frame();

    for epoch in [11, 12] {
        publish_render_with_layout(
            &mut outbound,
            None,
            &mut next_msg_id,
            &mut publish_state,
            &frame,
            1.0,
            None,
            Vec::new(),
            epoch,
            1,
        )
        .expect("publish compositor frame");
    }

    let messages = drain_messages(&buf.0.lock().unwrap());
    let frames: Vec<_> = messages
        .iter()
        .filter_map(|message| match &message.kind {
            IpcMessageKind::CompositorFrame {
                surface_id,
                navigation_epoch,
                frame_id,
                paint,
            } => Some((*surface_id, *navigation_epoch, *frame_id, paint.as_ref())),
            _ => None,
        })
        .collect();
    assert_eq!(
        frames
            .iter()
            .map(|(surface_id, epoch, frame_id, _)| (*surface_id, *epoch, *frame_id))
            .collect::<Vec<_>>(),
        vec![(73, 11, 1), (73, 12, 2)]
    );
    assert!(frames.iter().all(|(_, _, _, paint)| paint.hit_test.is_some()));
}

#[test]
fn publish_compositor_frame_carries_dirty_rects() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
    let mut next_msg_id = 1_u64;
    let mut publish_state = FramePublishState::new(73, FramePublishMode::Compositor);
    let mut frame = sample_frame();
    frame.dirty_rects = vec![(10.0, 20.0, 30.0, 40.0)];

    publish_render_with_layout(
        &mut outbound,
        None,
        &mut next_msg_id,
        &mut publish_state,
        &frame,
        1.0,
        None,
        Vec::new(),
        11,
        1,
    )
    .expect("publish compositor frame");

    let messages = drain_messages(&buf.0.lock().unwrap());
    let paint = messages
        .iter()
        .find_map(|message| match &message.kind {
            IpcMessageKind::CompositorFrame { paint, .. } => Some(paint.as_ref()),
            _ => None,
        })
        .expect("CompositorFrame");
    assert_eq!(paint.dirty_rects.len(), 1);
    assert!((paint.dirty_rects[0].x - 10.0).abs() < f32::EPSILON);
    assert!((paint.dirty_rects[0].y - 20.0).abs() < f32::EPSILON);
    assert!((paint.dirty_rects[0].width - 30.0).abs() < f32::EPSILON);
    assert!((paint.dirty_rects[0].height - 40.0).abs() < f32::EPSILON);
}

#[test]
fn publish_mode_switch_republishes_legacy_only() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
    let mut next_msg_id = 1_u64;
    let mut publish_state = FramePublishState::new(91, FramePublishMode::Compositor);
    let frame = sample_frame();

    publish_render_with_layout(
        &mut outbound,
        None,
        &mut next_msg_id,
        &mut publish_state,
        &frame,
        1.0,
        None,
        Vec::new(),
        6,
        1,
    )
    .expect("publish compositor frame");
    publish_state.set_mode(FramePublishMode::Legacy);
    for _ in 0..2 {
        publish_render_with_layout(
            &mut outbound,
            None,
            &mut next_msg_id,
            &mut publish_state,
            &frame,
            1.0,
            None,
            Vec::new(),
            6,
            1,
        )
        .expect("publish legacy frame");
    }

    let messages = drain_messages(&buf.0.lock().unwrap());
    assert!(matches!(
        messages[0].kind,
        IpcMessageKind::CompositorFrame { frame_id: 1, .. }
    ));
    assert!(
        messages[1..]
            .iter()
            .all(|message| matches!(message.kind, IpcMessageKind::ViewPainted(_)))
    );
}

#[test]
fn one_input_transaction_publishes_at_most_one_page_frame() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let (_inbound_tx, inbound_rx) = std::sync::mpsc::channel();
    let mut runtime = RendererRuntime::with_io(101, FramePublishMode::Legacy, Box::new(buf.clone()), inbound_rx);
    runtime.stub_network = true;
    runtime.cached_html = "<input id='name' value='a'>".into();
    runtime
        .webview
        .as_mut()
        .expect("webview")
        .load_html(&runtime.cached_html, None);

    runtime
        .run_frame_transaction(|runtime| {
            runtime.publish_webview(None, false)?;
            runtime.invalidate_script_render();
            runtime.publish_webview(None, false)
        })
        .expect("coalesced publish");

    let messages = drain_messages(&buf.0.lock().unwrap());
    let frame_count = messages
        .iter()
        .filter(|message| matches!(message.kind, IpcMessageKind::ViewPainted(_)))
        .count();
    assert_eq!(frame_count, 1);
}

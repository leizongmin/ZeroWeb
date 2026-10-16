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
                reported: None,
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
                    link_text: None,
                },
            )],
            parents: Vec::new(),
            hidden_nodes: Vec::new(),
            pe_none_nodes: Vec::new(),
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
        Vec::new(),
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
            Vec::new(),
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
        Vec::new(),
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
        Vec::new(),
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
            Vec::new(),
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

/// P100：帧内图片总量超过 `MAX_PAINT_IMAGE_BATCH_BYTES` 时拆批先行——批消息
/// FIFO 先于帧、帧本体载荷置空、同步路径回传全部已发出 key。各 payload 25MiB
/// （两个合计 50MiB 超批上限 48MiB，各自 ≤ 上限），每批一个 payload。
#[test]
fn oversize_image_payloads_split_into_batches_before_frame() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
    let mut next_msg_id = 1_u64;
    let mut publish_state = FramePublishState::new(41, FramePublishMode::Legacy);
    let frame = sample_frame();
    let batch_cap = zero_protocol::MAX_PAINT_IMAGE_BATCH_BYTES;
    let payload_bytes = batch_cap * 25 / 48;
    let payloads = vec![
        zero_protocol::IpcImagePayload {
            image_key: 11,
            width: 1,
            height: 1,
            rgba: vec![0u8; payload_bytes],
        },
        zero_protocol::IpcImagePayload {
            image_key: 12,
            width: 1,
            height: 1,
            rgba: vec![0u8; payload_bytes],
        },
    ];

    let sent_now = publish_render_with_layout(
        &mut outbound,
        None,
        &mut next_msg_id,
        &mut publish_state,
        &frame,
        1.0,
        None,
        payloads,
        7,
        1,
        Vec::new(),
    )
    .expect("publish with oversize images");

    let mut messages = drain_messages(&buf.0.lock().unwrap());
    // 批先于帧：取出末尾 ViewPainted 后，其余应为按序两条 ImagePayloadBatch。
    let painted = match messages.pop() {
        Some(message) => match message.kind {
            IpcMessageKind::ViewPainted(paint) => paint,
            other => panic!("末尾应为 ViewPainted，实际 {other:?}"),
        },
        None => panic!("无消息"),
    };
    let batches: Vec<_> = messages
        .iter()
        .filter_map(|message| match &message.kind {
            IpcMessageKind::ImagePayloadBatch {
                surface_id,
                navigation_epoch,
                payloads,
            } => Some((*surface_id, *navigation_epoch, payloads)),
            _ => None,
        })
        .collect();
    assert_eq!(batches.len(), 2);
    // Legacy 发布（ViewPainted）无 surface 归属，批 surface_id 为 None。
    assert!(batches.iter().all(|(surface_id, _, _)| surface_id.is_none()));
    assert!(batches.iter().all(|(_, epoch, _)| *epoch == 7));
    let keys: Vec<_> = batches
        .iter()
        .map(|(_, _, payloads)| payloads.iter().map(|p| p.image_key).collect::<Vec<_>>())
        .collect();
    assert_eq!(keys, vec![vec![11], vec![12]]);
    // 帧本体不携带像素（批已先发），同步回传含全部已发出 key。
    assert!(painted.image_payloads.is_empty());
    assert_eq!(sent_now, vec![11, 12]);
}

/// P100：帧内图片未超批上限时维持现状——载荷随帧携带、不产生批消息。
#[test]
fn small_image_payloads_stay_in_frame() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
    let mut next_msg_id = 1_u64;
    let mut publish_state = FramePublishState::new(41, FramePublishMode::Legacy);
    let frame = sample_frame();
    let payloads = vec![zero_protocol::IpcImagePayload {
        image_key: 21,
        width: 1,
        height: 1,
        rgba: vec![0u8; 16],
    }];

    let sent_now = publish_render_with_layout(
        &mut outbound,
        None,
        &mut next_msg_id,
        &mut publish_state,
        &frame,
        1.0,
        None,
        payloads,
        7,
        1,
        Vec::new(),
    )
    .expect("publish with small images");

    let messages = drain_messages(&buf.0.lock().unwrap());
    assert!(
        messages
            .iter()
            .all(|message| !matches!(message.kind, IpcMessageKind::ImagePayloadBatch { .. }))
    );
    let painted = messages
        .iter()
        .find_map(|message| match &message.kind {
            IpcMessageKind::ViewPainted(paint) => Some(paint.as_ref()),
            _ => None,
        })
        .expect("ViewPainted");
    assert_eq!(painted.image_payloads.len(), 1);
    assert_eq!(painted.image_payloads[0].image_key, 21);
    assert_eq!(sent_now, vec![21]);
}

/// P100：生产默认异步路径（Compositor 模式 + 发布线程）——批经 regular FIFO 先于
/// 帧写出，批 key 由发布线程回传（drain_sent_image_keys），主线程同步回传为空，
/// 主 outbound 直发通道无任何消息。删除发布线程批 key 回传分支时本测试失败。
#[test]
fn compositor_mode_batches_flow_through_publish_thread() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
    let thread_buf = Arc::new(Mutex::new(Vec::new()));
    let writer: Box<dyn Write + Send> = Box::new(SharedBuf(Arc::clone(&thread_buf)));
    let thread = compositor_publish_thread::CompositorPublishThread::spawn(Arc::new(Mutex::new(writer)));
    let mut next_msg_id = 1_u64;
    let mut publish_state = FramePublishState::new(73, FramePublishMode::Compositor);
    let frame = sample_frame();
    let batch_cap = zero_protocol::MAX_PAINT_IMAGE_BATCH_BYTES;
    let payload_bytes = batch_cap * 25 / 48;
    let payloads = vec![
        zero_protocol::IpcImagePayload {
            image_key: 31,
            width: 1,
            height: 1,
            rgba: vec![0u8; payload_bytes],
        },
        zero_protocol::IpcImagePayload {
            image_key: 32,
            width: 1,
            height: 1,
            rgba: vec![0u8; payload_bytes],
        },
    ];

    let sent_now = publish_render_with_layout(
        &mut outbound,
        Some(&thread),
        &mut next_msg_id,
        &mut publish_state,
        &frame,
        1.0,
        None,
        payloads,
        7,
        1,
        Vec::new(),
    )
    .expect("publish with oversize images via publish thread");

    // 纯异步：批全部入队成功，主线程无同步直发 key。
    assert!(sent_now.is_empty());
    // 发布线程按 FIFO 写出两条批 + 一帧并回传批 key。
    // 每批约 26MiB 的序列化走逐字节 serde 路径，秒级耗时属预期（非挂死），
    // 预留 30s 预算；正常数秒内完成。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut sent = Vec::new();
    loop {
        let got = thread.drain_sent_image_keys();
        sent.extend(got);
        if sent.len() >= 2 {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "发布线程必须回传全部批 key");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(thread); // join worker 后再读共享 buffer，避免并发写
    let mut messages = drain_messages(&thread_buf.lock().unwrap());
    assert_eq!(messages.len(), 3);
    let frame_message = messages.pop().expect("帧消息");
    // P100 返修（F1）：compositor 模式批必须带所属 surface（浏览器据此中继
    // 给 compositor 进程），与帧同 surface。
    let batch_surface_ids: Vec<Option<u64>> = messages
        .iter()
        .map(|message| match &message.kind {
            IpcMessageKind::ImagePayloadBatch { surface_id, .. } => *surface_id,
            other => panic!("批先于帧，实际 {other:?}"),
        })
        .collect();
    assert_eq!(batch_surface_ids, vec![Some(73), Some(73)]);
    let batch_keys: Vec<Vec<u64>> = messages
        .iter()
        .map(|message| match &message.kind {
            IpcMessageKind::ImagePayloadBatch { payloads, .. } => payloads.iter().map(|p| p.image_key).collect(),
            other => panic!("批先于帧，实际 {other:?}"),
        })
        .collect();
    assert_eq!(batch_keys, vec![vec![31], vec![32]]);
    let IpcMessageKind::CompositorFrame { paint, .. } = frame_message.kind else {
        panic!("Compositor 模式末尾应为 CompositorFrame");
    };
    assert!(paint.image_payloads.is_empty());
    // 主 outbound 直发通道无任何消息（全部经发布线程）。
    assert!(buf.0.lock().unwrap().is_empty());
}

/// P100：图片总量恰等于批上限时不拆批——阈值是严格大于，边界行为钉死。
#[test]
fn total_image_bytes_exactly_at_cap_stays_in_frame() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
    let mut next_msg_id = 1_u64;
    let mut publish_state = FramePublishState::new(41, FramePublishMode::Legacy);
    let frame = sample_frame();
    let batch_cap = zero_protocol::MAX_PAINT_IMAGE_BATCH_BYTES;
    let half = batch_cap / 2;
    let payloads = vec![
        zero_protocol::IpcImagePayload {
            image_key: 41,
            width: 1,
            height: 1,
            rgba: vec![0u8; half],
        },
        zero_protocol::IpcImagePayload {
            image_key: 42,
            width: 1,
            height: 1,
            rgba: vec![0u8; batch_cap - half],
        },
    ];

    let sent_now = publish_render_with_layout(
        &mut outbound,
        None,
        &mut next_msg_id,
        &mut publish_state,
        &frame,
        1.0,
        None,
        payloads,
        7,
        1,
        Vec::new(),
    )
    .expect("publish at exact cap");

    let messages = drain_messages(&buf.0.lock().unwrap());
    assert!(
        messages
            .iter()
            .all(|message| !matches!(message.kind, IpcMessageKind::ImagePayloadBatch { .. }))
    );
    let painted = messages
        .iter()
        .find_map(|message| match &message.kind {
            IpcMessageKind::ViewPainted(paint) => Some(paint.as_ref()),
            _ => None,
        })
        .expect("ViewPainted");
    // 帧本体携带全部像素，key 随帧同步标记。
    assert_eq!(
        painted.image_payloads.iter().map(|p| p.image_key).collect::<Vec<_>>(),
        vec![41, 42]
    );
    assert_eq!(sent_now, vec![41, 42]);
}

/// P100：总量恰超上限 1 字节即拆批——阈值边界另一侧。
#[test]
fn total_image_bytes_one_over_cap_splits() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
    let mut next_msg_id = 1_u64;
    let mut publish_state = FramePublishState::new(41, FramePublishMode::Legacy);
    let frame = sample_frame();
    let batch_cap = zero_protocol::MAX_PAINT_IMAGE_BATCH_BYTES;
    let half = batch_cap / 2;
    let payloads = vec![
        zero_protocol::IpcImagePayload {
            image_key: 51,
            width: 1,
            height: 1,
            rgba: vec![0u8; half],
        },
        zero_protocol::IpcImagePayload {
            image_key: 52,
            width: 1,
            height: 1,
            rgba: vec![0u8; batch_cap - half + 1],
        },
    ];

    let sent_now = publish_render_with_layout(
        &mut outbound,
        None,
        &mut next_msg_id,
        &mut publish_state,
        &frame,
        1.0,
        None,
        payloads,
        7,
        1,
        Vec::new(),
    )
    .expect("publish one byte over cap");

    let mut messages = drain_messages(&buf.0.lock().unwrap());
    let painted = match messages.pop() {
        Some(message) => match message.kind {
            IpcMessageKind::ViewPainted(paint) => paint,
            other => panic!("末尾应为 ViewPainted，实际 {other:?}"),
        },
        None => panic!("无消息"),
    };
    let batch_keys: Vec<Vec<u64>> = messages
        .iter()
        .filter_map(|message| match &message.kind {
            IpcMessageKind::ImagePayloadBatch { payloads, .. } => Some(payloads.iter().map(|p| p.image_key).collect()),
            _ => None,
        })
        .collect();
    assert_eq!(batch_keys, vec![vec![51], vec![52]]);
    assert!(painted.image_payloads.is_empty());
    assert_eq!(sent_now, vec![51, 52]);
}

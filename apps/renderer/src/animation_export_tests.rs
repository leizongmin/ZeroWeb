//! M2-S1b：合成动画导出状态机与 engine→protocol 换算测试。

use super::*;
use std::sync::{Arc, Mutex};
use zero_css_parser::values::{
    AnimationDirectionValue, AnimationFillModeValue, StepPosition, TimingFunctionValue, TransformFunction,
};
use zero_engine::animation::compositor_export::{
    ExportedAnimationDeclaration, ExportedAnimationKeyframe, ExportedAnimationValue,
};
use zero_protocol::{
    AnimationDirection, AnimationEasing, AnimationFillMode, AnimationPropertyKind, AnimationPropertyValue,
    TransformFunctionValue,
};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{PaintAnimationProperty, PaintAnimationScope, RenderPrimitives};

use super::compositor_publish_tests::{SharedBuf, drain_messages};

/// opacity 双端点声明 fixture（engine 域，秒单位）。
fn opacity_decl(id: u64, from: f64, to: f64) -> ExportedAnimationDeclaration {
    ExportedAnimationDeclaration {
        animation_id: id,
        element_key: 1,
        // M2-S2a：换算测试不涉及查询覆盖，选择器候选空（build 侧跳过）。
        element_selectors: Vec::new(),
        name: "fade".into(),
        duration_s: 1.0,
        delay_s: 0.0,
        iterations: None,
        direction: AnimationDirectionValue::Normal,
        fill: AnimationFillModeValue::None,
        easing: TimingFunctionValue::Linear,
        start_time_s: 0.0,
        keyframes: vec![
            ExportedAnimationKeyframe {
                offset: 0.0,
                value: ExportedAnimationValue::Opacity(from),
                easing: None,
            },
            ExportedAnimationKeyframe {
                offset: 1.0,
                value: ExportedAnimationValue::Opacity(to),
                easing: None,
            },
        ],
    }
}

#[test]
fn declaration_to_ipc_converts_units_and_enums() {
    let mut declaration = opacity_decl(7, 0.3, 1.0);
    declaration.duration_s = 2.5;
    declaration.delay_s = 0.125;
    declaration.direction = AnimationDirectionValue::Alternate;
    declaration.fill = AnimationFillModeValue::Both;
    declaration.easing = TimingFunctionValue::CubicBezier(0.1, 0.2, 0.3, 0.4);
    declaration.start_time_s = 0.25;
    let anchors = HashMap::from([(7u64, 1_000u64)]);
    let ipc = paint_export::animation_declaration_to_ipc(&declaration, &anchors).expect("可换算");
    assert_eq!(ipc.animation_id, 7);
    assert_eq!(ipc.property, AnimationPropertyKind::Opacity);
    assert!(ipc.composable, "导出面声明全可合成");
    assert_eq!(ipc.start_monotonic_ns, 1_000 + 250_000_000, "锚点 + start_time*1e9");
    assert_eq!(ipc.timing.duration_ms, 2500.0, "秒 → 毫秒");
    assert_eq!(ipc.timing.delay_ms, 125.0);
    assert_eq!(ipc.timing.iterations, None);
    assert_eq!(ipc.timing.direction, AnimationDirection::Alternate);
    assert_eq!(ipc.timing.fill, AnimationFillMode::Both);
    assert_eq!(ipc.timing.easing, AnimationEasing::CubicBezier(0.1, 0.2, 0.3, 0.4));
    assert_eq!(ipc.keyframes.len(), 2);
    assert_eq!(ipc.keyframes[0].value, AnimationPropertyValue::Opacity(0.3));
    assert_eq!(ipc.keyframes[1].value, AnimationPropertyValue::Opacity(1.0));
}

#[test]
fn declaration_to_ipc_maps_engine_transform_subset() {
    // RotateZ→Rotate（协议无独立变体，语义同 rotate）、Scale(s, None)→(s, s)、
    // Skew(ax, None)→(ax, 0)、Steps 的 pos 两侧都不参与进度映射（折叠为 n）。
    let declaration = ExportedAnimationDeclaration {
        keyframes: vec![ExportedAnimationKeyframe {
            offset: 0.0,
            value: ExportedAnimationValue::Transform(vec![
                TransformFunction::RotateZ(45.0),
                TransformFunction::Scale(2.0, None),
                TransformFunction::Skew(10.0, None),
            ]),
            easing: Some(TimingFunctionValue::Steps(4, Some(StepPosition::Start))),
        }],
        ..opacity_decl(9, 1.0, 1.0)
    };
    let anchors = HashMap::from([(9u64, 5u64)]);
    let ipc = paint_export::animation_declaration_to_ipc(&declaration, &anchors).expect("可换算");
    assert_eq!(ipc.property, AnimationPropertyKind::Transform);
    assert_eq!(
        ipc.keyframes[0].value,
        AnimationPropertyValue::Transform(vec![
            TransformFunctionValue::Rotate(45.0),
            TransformFunctionValue::Scale(2.0, 2.0),
            TransformFunctionValue::Skew(10.0, 0.0),
        ])
    );
    assert_eq!(ipc.keyframes[0].easing, Some(AnimationEasing::Steps(4)));
}

#[test]
fn declaration_to_ipc_rejects_subset_violation_and_missing_anchor() {
    // Matrix 超导出 2D 子集（分类规则 2 的防御面）→ 整条拒绝。
    let matrix_decl = ExportedAnimationDeclaration {
        keyframes: vec![ExportedAnimationKeyframe {
            offset: 0.0,
            value: ExportedAnimationValue::Transform(vec![TransformFunction::Matrix(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)]),
            easing: None,
        }],
        ..opacity_decl(3, 1.0, 1.0)
    };
    let anchors = HashMap::from([(3u64, 1u64)]);
    assert!(paint_export::animation_declaration_to_ipc(&matrix_decl, &anchors).is_none());
    // 锚点缺失（导出先行于状态推进的调用序错误）→ 拒绝，不发无锚声明。
    let good = opacity_decl(4, 0.0, 1.0);
    assert!(paint_export::animation_declaration_to_ipc(&good, &HashMap::new()).is_none());
}

#[test]
fn snapshot_carries_converted_animation_scopes() {
    // paint_snapshot_from_primitives 的作用域换算出口：engine 域 → IpcAnimationScope
    //（rect 展开 [x,y,w,h]），flag 关恒空由恒空入参覆盖（publish 侧契约）。
    let scopes = vec![PaintAnimationScope {
        animation_id: 11,
        property: PaintAnimationProperty::Transform,
        draw_begin: 2,
        draw_end: 5,
        rect: Rect::new(10.0, 20.0, 30.0, 40.0),
        transform_index: Some(1),
    }];
    let snapshot = paint_export::paint_snapshot_from_primitives(
        800,
        600,
        1.0,
        900.0,
        &RenderPrimitives::new(),
        &[],
        Vec::new(),
        None,
        3,
        1,
        &scopes,
    );
    assert_eq!(snapshot.animation_scopes.len(), 1);
    let ipc = &snapshot.animation_scopes[0];
    assert_eq!(ipc.animation_id, 11);
    assert_eq!(ipc.property, AnimationPropertyKind::Transform);
    assert_eq!(ipc.draw_begin, 2);
    assert_eq!(ipc.draw_end, 5);
    assert_eq!(ipc.rect, [10.0, 20.0, 30.0, 40.0]);
    assert_eq!(ipc.transform_index, Some(1));

    let empty = paint_export::paint_snapshot_from_primitives(
        800,
        600,
        1.0,
        900.0,
        &RenderPrimitives::new(),
        &[],
        Vec::new(),
        None,
        3,
        1,
        &[],
    );
    assert!(empty.animation_scopes.is_empty(), "flag 关恒空");
}

#[test]
fn export_state_advance_sends_first_skips_unchanged_and_bumps_on_change() {
    let mut state = AnimationExportState::new(0);
    let decl = opacity_decl(7, 0.3, 1.0);

    // 首帧必发（compositor 无状态可比较），generation 从 0 → 1。
    let first = state.advance(0, 100, std::slice::from_ref(&decl)).expect("首帧必发");
    assert_eq!(state.generation, 1);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].animation_id, 7);
    assert_eq!(state.anchors[&7], 100, "新 id 记本帧锚点");

    // 同集重发 → None（零消息，声明洪泛闸门）。
    assert!(state.advance(0, 200, std::slice::from_ref(&decl)).is_none());
    assert_eq!(state.generation, 1);
    assert_eq!(state.anchors[&7], 100, "既有 id 锚点稳定（动画时间轴不漂移）");

    // 集合变化（新增声明）→ 重发，generation +1；新 id 记新锚点。
    let second = opacity_decl(8, 0.0, 1.0);
    let changed = state.advance(0, 300, &[decl.clone(), second]).expect("集合变化必发");
    assert_eq!(state.generation, 2);
    assert_eq!(changed.len(), 2);
    assert_eq!(state.anchors[&7], 100);
    assert_eq!(state.anchors[&8], 300);
}

#[test]
fn export_state_advance_prunes_gone_ids_and_reanchors_on_return() {
    let mut state = AnimationExportState::new(0);
    let decl = opacity_decl(7, 0.3, 1.0);
    state.advance(0, 100, std::slice::from_ref(&decl)).expect("首帧");

    // 动画消失 → 集变化发空集（清理），锚点剪除。
    let empty = state.advance(0, 200, &[]).expect("清理发送");
    assert!(empty.is_empty());
    assert!(state.anchors.is_empty(), "消失 id 剪除");
    // 空集稳态 → 零消息。
    assert!(state.advance(0, 300, &[]).is_none());

    // 重现 = 重新开始（新锚点，CSS animation 重新触发语义）。
    let again = state.advance(0, 400, &[decl]).expect("重现必发");
    assert_eq!(again.len(), 1);
    assert_eq!(state.anchors[&7], 400, "重现记新锚点");
}

#[test]
fn export_state_epoch_change_resets_generation_and_anchors() {
    let mut state = AnimationExportState::new(0);
    let decl = opacity_decl(7, 0.3, 1.0);
    state.advance(0, 100, std::slice::from_ref(&decl)).expect("首帧");
    assert_eq!(state.generation, 1);

    // 导航换代：代际回 1、锚点重建（旧代语义不延续）。
    let next = state.advance(5, 900, &[decl]).expect("换代后首帧必发");
    assert_eq!(state.generation, 1, "换代重置代际");
    assert_eq!(state.epoch, 5);
    assert_eq!(state.anchors[&7], 900);
    assert_eq!(next[0].animation_id, 7);
}

#[test]
fn export_state_deactivate_only_after_send() {
    let mut state = AnimationExportState::new(0);
    assert!(!state.deactivate(), "未发送过无需清理");
    let decl = opacity_decl(7, 0.3, 1.0);
    state.advance(0, 100, &[decl]).expect("首帧");
    assert!(state.deactivate(), "发过 → 需发空集清理");
    assert_eq!(state.generation, 2);
    assert!(state.sent.is_none() && state.anchors.is_empty());
    assert!(!state.deactivate(), "稳态零消息");
}

#[test]
fn legacy_mode_publish_never_sends_animation_state() {
    // Legacy 帧无 CompositorAnimationState 消费方；本测试与 ZW_COMPOSITOR_ANIMATIONS
    // 环境值无关（模式门独立于 flag 生效）。
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let (_inbound_tx, inbound_rx) = std::sync::mpsc::channel();
    let mut runtime = RendererRuntime::with_io(101, FramePublishMode::Legacy, Box::new(buf.clone()), inbound_rx);
    runtime.stub_network = true;
    runtime.cached_html = "<div class='a'>x</div>".into();
    runtime
        .webview
        .as_mut()
        .expect("webview")
        .load_html(&runtime.cached_html, None);
    runtime.publish_webview(None, false).expect("publish");
    let messages = drain_messages(&buf.0.lock().unwrap());
    assert!(
        !messages
            .iter()
            .any(|m| matches!(m.kind, IpcMessageKind::CompositorAnimationState(_))),
        "Legacy 模式不发动画状态"
    );
}

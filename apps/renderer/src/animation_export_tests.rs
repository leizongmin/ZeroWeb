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
        paused: false,
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
fn finish_drain_reports_each_animation_exactly_once() {
    // M2-S2b：惰性终结判定——活跃期空、结束后一条 animationend、重复唤醒不重发。
    let mut state = AnimationExportState::new(0);
    let mut decl = opacity_decl(7, 0.0, 1.0);
    decl.iterations = Some(1.0);
    decl.element_selectors = vec![
        "div#box".into(),
        "html:nth-child(1) > body:nth-child(2) > div#box".into(),
    ];
    state.advance(0, 0, std::slice::from_ref(&decl)).expect("首帧");

    // 活跃期（0.5s）→ 无事件。
    assert!(state.drain_animation_events(500_000_000).is_empty());
    // 结束后（1.5s）→ 恰一条；selector 取候选首个（unique 形态）；elapsedTime =
    // duration × iterations（镜像 engine tick 结束分支）。
    let events = state.drain_animation_events(1_500_000_000);
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].kind, zero_engine::AnimationEventKind::End));
    assert_eq!(events[0].selector, "div#box");
    assert_eq!(events[0].name, "fade");
    assert!((events[0].elapsed - 1.0).abs() < 1e-9);
    // 重复采样/重复唤醒 → 幂等零事件（FR-7 恰一次）。
    assert!(state.drain_animation_events(2_500_000_000).is_empty());
    // deactivate 清空 sent 与 reported → 退出导出域零事件。
    assert!(state.deactivate());
    assert!(state.drain_animation_events(3_500_000_000).is_empty());
}

#[test]
fn finish_drain_epoch_change_allows_same_id_to_fire_again() {
    // M2-S2b：导航换代整表清空——同 id 新页动画是**新动画**（合法再派发），
    // 旧代声明随 sent 替换消失，跨代重放被双重阻断（FR-7 不重放）。
    let mut state = AnimationExportState::new(0);
    let mut decl = opacity_decl(7, 0.0, 1.0);
    decl.iterations = Some(1.0);
    decl.element_selectors = vec!["div#box".into()];
    state.advance(0, 0, std::slice::from_ref(&decl)).expect("首帧");
    assert_eq!(state.drain_animation_events(1_500_000_000).len(), 1);

    // 换代（epoch 0→5）：reported/sent/anchors 全重置，同 id 重新导出（新锚点 900）。
    state.advance(5, 900, std::slice::from_ref(&decl)).expect("换代必发");
    assert_eq!(state.anchors[&7], 900);
    // 新动画活跃期（950ms < 1s）→ 无事件（旧 reported 不再压制新动画）。
    assert!(state.drain_animation_events(950_000_000).is_empty());
    assert_eq!(
        state.drain_animation_events(2_000_000_000).len(),
        1,
        "新动画终结合法派发"
    );
}

#[test]
fn cancel_diff_fires_animationcancel_exactly_once() {
    // M2-S3a：同代际导出集差分——infinite 动画（永不自然终结）声明消失 → 恰一条
    // animationcancel，elapsedTime = 取消时刻活跃时长（延迟不计入，CSS Animations
    // §animationcancel）；重复 advance/唤醒不重发（S3 场景 c 恰一次）。
    let mut state = AnimationExportState::new(0);
    let mut decl = opacity_decl(7, 0.0, 1.0);
    decl.iterations = None;
    decl.element_selectors = vec!["div#box".into()];
    state.advance(0, 0, std::slice::from_ref(&decl)).expect("首帧");

    // 活跃期声明仍在 → 无事件；声明消失（advance 发空集）→ 差分记取消事件。
    assert!(state.drain_animation_events(500_000_000).is_empty());
    state.advance(0, 800_000_000, &[]).expect("清理发送");
    let events = state.drain_animation_events(800_000_000);
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].kind, zero_engine::AnimationEventKind::Cancel));
    assert_eq!(events[0].selector, "div#box");
    assert_eq!(events[0].name, "fade");
    assert!((events[0].elapsed - 0.8).abs() < 1e-9, "活跃时长 0.8s");
    // 集已剪除 + cancelled_reported 双重阻断 → 重复评估零重发。
    assert!(state.advance(0, 900_000_000, &[]).is_none(), "稳态零消息");
    assert!(state.drain_animation_events(900_000_000).is_empty());
}

#[test]
fn cancel_diff_reports_end_when_removal_races_natural_finish() {
    // M2-S3a：finish-precedence——有限动画样式移除与自然完成同帧（差分时采样上代
    // 声明已 finished）→ 补发 animationend 而非 cancel（终态优先，elapsed 与 S2b
    // 终结判定同口径 duration × iterations）。
    let mut state = AnimationExportState::new(0);
    let mut decl = opacity_decl(7, 0.0, 1.0);
    decl.iterations = Some(1.0);
    decl.element_selectors = vec!["div#box".into()];
    state.advance(0, 0, std::slice::from_ref(&decl)).expect("首帧");

    state.advance(0, 1_500_000_000, &[]).expect("清理发送");
    let events = state.drain_animation_events(1_500_000_000);
    assert_eq!(events.len(), 1);
    assert!(
        matches!(events[0].kind, zero_engine::AnimationEventKind::End),
        "终态优先于取消"
    );
    assert!((events[0].elapsed - 1.0).abs() < 1e-9);
    // 终态已报 → 后续不二次派发（End 或 Cancel 均不重发）。
    assert!(state.drain_animation_events(2_500_000_000).is_empty());
}

#[test]
fn cancel_diff_not_fired_across_epoch_change() {
    // M2-S3a：导航换代 `Self::new` 整表重置（S2b 同款）——旧代活跃声明随换代消失
    // 不派发 cancel（新页是干净世界，S3 场景 d「不跳旧世代」）。
    let mut state = AnimationExportState::new(0);
    let decl = opacity_decl(7, 0.0, 1.0);
    state.advance(0, 0, std::slice::from_ref(&decl)).expect("首帧");

    state.advance(5, 900, &[]).expect("换代后首帧（新页无该动画）");
    assert!(state.pending_events.is_empty());
    assert!(state.drain_animation_events(1_000_000_000).is_empty());
}

#[test]
fn deactivate_drops_pending_cancel_events() {
    // M2-S3a：域退出不发积压事件——导出面整体消失由 deactivate 空集清理承接，
    // 不是页面级动画取消（RFC §2.1 明示简化）。
    let mut state = AnimationExportState::new(0);
    let mut decl = opacity_decl(7, 0.0, 1.0);
    decl.iterations = None;
    decl.element_selectors = vec!["div#box".into()];
    state.advance(0, 0, std::slice::from_ref(&decl)).expect("首帧");
    state.advance(0, 400_000_000, &[]).expect("清理发送");
    assert!(!state.pending_events.is_empty(), "取消事件已积压");

    assert!(state.deactivate());
    assert!(state.pending_events.is_empty(), "域退出清空积压");
    assert!(state.drain_animation_events(500_000_000).is_empty());
}

#[test]
fn pause_freezes_declaration_and_resume_rebases_anchor() {
    // M2-S3b：play-state 翻转驱动 renderer 暂停态机——冻结相位随声明上行后字节
    // 稳定零 churn（发送闸门），恢复重基锚点使活跃时长续自冻结值（暂停不计入）。
    let mut state = AnimationExportState::new(0);
    let mut decl = opacity_decl(7, 0.0, 1.0);
    decl.iterations = Some(1.0);
    state.advance(0, 0, std::slice::from_ref(&decl)).expect("首帧");

    // 暂停翻转：冻结当刻活跃相位 0.8s（内容变化 → 必发）。
    decl.paused = true;
    let paused = state
        .advance(0, 800_000_000, std::slice::from_ref(&decl))
        .expect("暂停翻转必发");
    assert_eq!(paused[0].frozen_phase_s, Some(0.8));

    // 暂停中墙钟推进：冻结值恒定 → 声明字节稳定，零消息（洪泛闸门）。
    assert!(
        state.advance(0, 2_500_000_000, std::slice::from_ref(&decl)).is_none(),
        "暂停中零 churn"
    );

    // 恢复：锚点重基 = now − delay − frozen（3.0 − 0.8 = 2.2s），冻结清除。
    decl.paused = false;
    let resumed = state
        .advance(0, 3_000_000_000, std::slice::from_ref(&decl))
        .expect("恢复必发");
    assert_eq!(resumed[0].frozen_phase_s, None);
    assert_eq!(state.anchors[&7], 2_200_000_000, "重基 = 3.0s − 0.8s 冻结");
    // 恢复后 0.1s 墙钟 = 活跃 0.1 + 冻结 0.8 = 0.9 → 线性渐变 0.9（采样自冻结相位续走）。
    let sample = zero_protocol::animation::sample_declaration(&resumed[0], 3_100_000_000);
    match sample.value {
        Some(AnimationPropertyValue::Opacity(o)) => assert!((o - 0.9).abs() < 1e-9, "自冻结相位续走"),
        _ => panic!("恢复后活跃期应有采样值"),
    }
}

#[test]
fn paused_declaration_no_finish_and_cancel_uses_frozen_elapsed() {
    // M2-S3b：暂停声明不触发 S2b 终结（冻结相位在活跃窗内 → finished 恒 false，
    // 墙钟越过原定结束也不派 animationend）；暂停中取消的 elapsedTime = 冻结相位
    // 而非墙钟差（暂停不计入活跃时长，CSS Animations §animationcancel）。
    let mut state = AnimationExportState::new(0);
    let mut decl = opacity_decl(7, 0.0, 1.0);
    decl.iterations = Some(1.0);
    decl.element_selectors = vec!["div#box".into()];
    state.advance(0, 0, std::slice::from_ref(&decl)).expect("首帧");

    // 0.1s 处暂停，墙钟推进到 5s（远超原定 1s 结束）→ 无 animationend。
    decl.paused = true;
    state
        .advance(0, 100_000_000, std::slice::from_ref(&decl))
        .expect("暂停翻转必发");
    assert!(state.drain_animation_events(5_000_000_000).is_empty(), "暂停不终结");

    // 暂停中声明消失 → animationcancel，elapsedTime = 冻结相位 0.1s（非墙钟 5s）。
    state.advance(0, 5_500_000_000, &[]).expect("清理发送");
    let events = state.drain_animation_events(5_500_000_000);
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].kind, zero_engine::AnimationEventKind::Cancel));
    assert!((events[0].elapsed - 0.1).abs() < 1e-9, "elapsed = 冻结相位");
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

// ── DC-2 offscreen 声明级门控（后台资源生命周期 M3）────────────────────────

/// opacity 作用域 fixture（几何占位——门控只消费 animation_id）。
fn opacity_scope(id: u64) -> PaintAnimationScope {
    PaintAnimationScope {
        animation_id: id,
        property: PaintAnimationProperty::Opacity,
        draw_begin: 0,
        draw_end: 1,
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        transform_index: None,
    }
}

#[test]
fn visible_declarations_filters_by_surviving_scopes() {
    // 门控过滤语义：匹配保留、不匹配剔除、空 scopes 全剔、同 id 多 scope 不重复、
    // 页面隐藏全剔（M4：hidden tab 无采样面，作用域存活也清空）。
    let declarations = vec![opacity_decl(1, 0.0, 1.0), opacity_decl(2, 1.0, 0.0)];

    let visible = paint_export::visible_declarations(&declarations, &[opacity_scope(2)], false);
    assert_eq!(visible.iter().map(|d| d.animation_id).collect::<Vec<_>>(), vec![2]);

    assert!(
        paint_export::visible_declarations(&declarations, &[], false).is_empty(),
        "全部作用域被剔 → 空集（offscreen 页声明不进 compositor 状态）"
    );

    let duplicated = paint_export::visible_declarations(&declarations, &[opacity_scope(1), opacity_scope(1)], false);
    assert_eq!(duplicated.len(), 1, "同 id 多 scope 声明不重复");
    assert_eq!(duplicated[0].animation_id, 1);

    assert!(
        paint_export::visible_declarations(&declarations, &[opacity_scope(1)], true).is_empty(),
        "page_hidden → 空集（hidden tab 呈现链空转收口）"
    );
}

/// flag 开 + Compositor 模式的 runtime（renderer 测试腿串行，set_var 无竞态；
/// 每测尾恢复 env，不污染后续测试）。
fn compositor_runtime(id: u64, buf: &SharedBuf) -> RendererRuntime {
    unsafe { std::env::set_var("ZW_COMPOSITOR_ANIMATIONS", "1") };
    let (_inbound_tx, inbound_rx) = std::sync::mpsc::channel();
    let mut runtime = RendererRuntime::with_io(id, FramePublishMode::Compositor, Box::new(buf.clone()), inbound_rx);
    runtime.stub_network = true;
    runtime
}

fn compositor_animation_states(buf: &SharedBuf) -> Vec<zero_protocol::CompositorAnimationStateParams> {
    drain_messages(&buf.0.lock().unwrap())
        .into_iter()
        .filter_map(|m| match m.kind {
            IpcMessageKind::CompositorAnimationState(params) => Some(params),
            _ => None,
        })
        .collect()
}

/// 发布线程异步写出（`send_regular` 经 enqueue_regular 入队）——轮询直到 buf
/// 全量快照消息数 ≥ min 或超时；drain 是从头解析的全量快照（非增量消费），
/// 每轮独立取，不做跨轮累计。本进程无其他写者，快照即本 runtime 发送量。
fn wait_compositor_states(buf: &SharedBuf, min: usize) -> Vec<zero_protocol::CompositorAnimationStateParams> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let states = compositor_animation_states(buf);
        if states.len() >= min || std::time::Instant::now() >= deadline {
            return states;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn compositor_state_sends_declarations_with_surviving_scope() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut runtime = compositor_runtime(201, &buf);
    let declarations = vec![opacity_decl(1, 0.0, 1.0)];
    runtime
        .send_compositor_animation_state(declarations, &[opacity_scope(1)], &[])
        .expect("发送");
    let states = wait_compositor_states(&buf, 1);
    assert_eq!(states.len(), 1, "首帧必发（sent 无先前值判变）");
    assert_eq!(states[0].declarations.len(), 1);
    assert_eq!(states[0].declarations[0].animation_id, 1);
    unsafe { std::env::remove_var("ZW_COMPOSITOR_ANIMATIONS") };
}

#[test]
fn compositor_state_sends_empty_set_when_all_scopes_culled() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut runtime = compositor_runtime(202, &buf);
    // 声明存在但全部作用域被视口剔除（offscreen 动画页）→ 空集发送（首帧必发），
    // compositor animations_active 不置位。
    let declarations = vec![opacity_decl(1, 0.0, 1.0)];
    runtime
        .send_compositor_animation_state(declarations, &[], &[])
        .expect("发送");
    let states = wait_compositor_states(&buf, 1);
    assert_eq!(states.len(), 1);
    assert!(states[0].declarations.is_empty(), "无可见采样面 → 空集");
    unsafe { std::env::remove_var("ZW_COMPOSITOR_ANIMATIONS") };
}

#[test]
fn compositor_state_recovers_declarations_when_scope_returns() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut runtime = compositor_runtime(203, &buf);
    let declarations = vec![opacity_decl(1, 0.0, 1.0)];
    // offscreen：空集代际。
    runtime
        .send_compositor_animation_state(declarations.clone(), &[], &[])
        .expect("发送");
    let states = wait_compositor_states(&buf, 1);
    assert_eq!(states.len(), 1);
    assert!(states[0].declarations.is_empty());
    let offscreen_generation = states[0].generation;

    // 稳定空集：集合判等零消息（空转消除的机制核心——不重复发空集）。等待窗口
    // 内无新增即为稳定（失败方向 = 假红，不掩盖回归）。
    runtime
        .send_compositor_animation_state(declarations.clone(), &[], &[])
        .expect("发送");
    std::thread::sleep(std::time::Duration::from_millis(50));
    assert_eq!(wait_compositor_states(&buf, 0).len(), 1, "稳定空集不重发");

    // 恢复沿：元素回视口下一帧作用域重现 → 声明经集合差分恢复，generation 翻转。
    runtime
        .send_compositor_animation_state(declarations, &[opacity_scope(1)], &[])
        .expect("发送");
    let states = wait_compositor_states(&buf, 2);
    assert_eq!(states.len(), 2);
    assert_eq!(states[1].declarations.len(), 1);
    assert_eq!(states[1].declarations[0].animation_id, 1);
    assert!(states[1].generation > offscreen_generation, "恢复代际递增");
    unsafe { std::env::remove_var("ZW_COMPOSITOR_ANIMATIONS") };
}

#[test]
fn compositor_state_clears_declarations_while_page_hidden() {
    // M4 GUI 判别腿缺口收口：tab 隐藏（SetVisibility → page_hidden）后声明清空 →
    // compositor animations_active 回落 → keep-alive 停对不可见 surface 的常态 Get
    // 重光栅；恢复可见首帧集合差分重现。
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let mut runtime = compositor_runtime(204, &buf);
    let declarations = vec![opacity_decl(1, 0.0, 1.0)];
    runtime
        .send_compositor_animation_state(declarations.clone(), &[opacity_scope(1)], &[])
        .expect("发送");
    let states = wait_compositor_states(&buf, 1);
    assert_eq!(states[0].declarations.len(), 1, "可见：声明照发");

    runtime.page_hidden = true;
    runtime
        .send_compositor_animation_state(declarations.clone(), &[opacity_scope(1)], &[])
        .expect("发送");
    let states = wait_compositor_states(&buf, 2);
    assert_eq!(states[1].declarations.len(), 0, "隐藏：声明清空（作用域存活也不发）");
    let hidden_generation = states[1].generation;

    runtime.page_hidden = false;
    runtime
        .send_compositor_animation_state(declarations, &[opacity_scope(1)], &[])
        .expect("发送");
    let states = wait_compositor_states(&buf, 3);
    assert_eq!(states[2].declarations.len(), 1);
    assert!(states[2].generation > hidden_generation, "恢复代际递增");
    unsafe { std::env::remove_var("ZW_COMPOSITOR_ANIMATIONS") };
}

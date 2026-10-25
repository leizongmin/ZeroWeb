//! T12 投机预热暖槽单测（设计卡 §7-1 五场景 + 返修 R1 延迟补位/R3 poll 侧清槽，
//! 与实现同提交）。
//!
//! 真实 renderer 子进程链路（`resolve_renderer_binary`，与 service_worker_owner_tests
//! 同款约束）：多进程锁串行 + 缺二进制即显式失败。renderer 侧「工厂态首文档跳过臂 /
//! 真复位臂」语义由 zero-renderer 既有钉守卫（js_worker.rs:2986/:3044/:3892）；本文件
//! 钉 browser 侧不变量——warm 面命令白名单、认领绑定、失效回退（认领侧 + poll 侧）、
//! T_idle 回收、边沿单 shot、claim 后 D 延迟补位。

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::*;
use zero_protocol::message::IpcMessageKind;
use zero_protocol::{AutomationOperation, AutomationRequest, AutomationResult, AutomationValue};

fn lock_multiprocess_tests() -> std::sync::MutexGuard<'static, ()> {
    crate::tests::MULTIPROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn real_backend() -> ProcessTabBackend {
    let renderer = resolve_renderer_binary().expect("fresh zero-renderer binary is required");
    ProcessTabBackend::with_renderer_bin(renderer)
}

/// 轮询至该 tab 的文档提交（认领 spare 后首文档可达性；20s 有界）。
fn poll_until_committed(
    backend: &mut ProcessTabBackend,
    snapshots: &mut HashMap<TabId, TabSnapshot>,
    tab_id: TabId,
    url: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        backend.poll(snapshots, &mut HashMap::new(), Some(tab_id), true);
        if backend
            .committed_document_urls
            .values()
            .any(|committed| committed == url)
        {
            return;
        }
        assert!(Instant::now() < deadline, "renderer did not commit test page {url}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// 在已绑定 tab 上执行脚本并等回执（认领后文档域状态断言用）。
fn execute_script(
    backend: &mut ProcessTabBackend,
    snapshots: &mut HashMap<TabId, TabSnapshot>,
    tab_id: TabId,
    request_id: u64,
    script: &str,
) -> Result<AutomationValue, String> {
    backend.send_automation_request(
        tab_id,
        request_id,
        AutomationOperation::ExecuteScript {
            script: script.to_string(),
            arguments: Vec::new(),
        },
    )?;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        backend.poll(snapshots, &mut HashMap::new(), Some(tab_id), true);
        if let Some(response) = backend.take_automation_response(tab_id, request_id) {
            return match response.result {
                Ok(AutomationResult::Value(value)) => Ok(value),
                Ok(other) => Err(format!("unexpected automation result: {other:?}")),
                Err(error) => Err(format!("{:?}: {}", error.code, error.message)),
            };
        }
        assert!(Instant::now() < deadline, "automation request {request_id} timed out");
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// 场景 1（设计 §7-1）：认领绑定 virgin spare；warm 面/认领尾段出站命令均为白名单
/// 三命令（无 Execute 族——不置位 document_started，首文档走 T9 工厂态跳过臂），
/// 且认领后首文档可正常提交。
#[test]
fn warm_claim_binds_virgin_spare_with_allowlisted_command_face() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let _ = take_test_renderer_outbound();
    let mut backend = real_backend();
    let tab_id = TabId(901);

    backend.note_warm_tick(false);
    assert!(backend.warm_spare.is_none(), "非聚焦 tick 不得触发预热");
    backend.note_warm_tick(true);
    let Some((spare_rid, _)) = backend.warm_spare else {
        panic!("聚焦边沿应触发预热");
    };

    // warm 面白名单：测试环境 compositor 非 Disconnected → 仅 SetViewport + SetJavascriptEnabled。
    let warm_face = take_test_renderer_outbound();
    assert!(
        matches!(
            warm_face.as_slice(),
            [IpcMessageKind::SetViewport(_), IpcMessageKind::SetJavascriptEnabled(_)]
        ),
        "warm 面必须是白名单三命令（此环境为后两项）: {warm_face:?}"
    );

    // 认领：tab 绑定 spare、清槽、照常尾段。
    backend.ensure_renderer(tab_id, (800, 600));
    assert_eq!(
        backend.tab_to_renderer.get(&tab_id),
        Some(&spare_rid),
        "认领应绑定 spare rid"
    );
    assert!(backend.warm_spare.is_none(), "认领后必须清槽");

    // 认领尾段 + 首文档装载：出站面不得出现 Execute 族（保工厂态跳过臂前提）。
    let mut snapshots = HashMap::new();
    let url = "https://warm-claim.test/page";
    backend.load_html(
        tab_id,
        "<!doctype html><html><body>warm</body></html>",
        None,
        Some(url),
        1,
    );
    let face_after = take_test_renderer_outbound();
    assert!(
        face_after.iter().all(|kind| matches!(
            kind,
            IpcMessageKind::SetViewport(_)
                | IpcMessageKind::SetJavascriptEnabled(_)
                | IpcMessageKind::SetFramePublishMode(_)
                | IpcMessageKind::LoadHtml(_)
        )),
        "认领尾段+首文档不得含 Execute 族命令: {face_after:?}"
    );
    poll_until_committed(&mut backend, &mut snapshots, tab_id, url);
}

/// 场景 2（设计 §7-1/§4 臂 B）：污染 spare（virgin 面注入 automation Execute）→
/// 认领后首文档走真复位臂，语义恒正确——污染残留被 wipe、新文档正常装载。
#[test]
fn polluted_warm_spare_claim_engages_true_reset_and_still_loads() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = real_backend();
    let tab_id = TabId(902);

    backend.note_warm_tick(false);
    backend.note_warm_tick(true);
    let Some((spare_rid, _)) = backend.warm_spare else {
        panic!("聚焦边沿应触发预热");
    };

    // 设计外载荷（臂 B 前提）：直接向 spare 注入 automation Execute（置位 document_started）。
    let pollutant = AutomationRequest {
        operation: AutomationOperation::ExecuteScript {
            script: "globalThis.__warm_pollutant = 'stale'; 'set'".to_string(),
            arguments: Vec::new(),
        },
    };
    let response = backend
        .manager
        .get_renderer(spare_rid)
        .expect("spare handle 应在场")
        .request_automation(77001, pollutant, Duration::from_secs(20), |_, _| Ok(()))
        .expect("污染脚本应执行成功");
    assert!(response.result.is_ok(), "污染脚本执行失败: {response:?}");

    // 认领污染 spare 并装载首文档（真复位臂：优先通道 Reset + 有界等待）。
    backend.ensure_renderer(tab_id, (800, 600));
    assert_eq!(backend.tab_to_renderer.get(&tab_id), Some(&spare_rid));
    let mut snapshots = HashMap::new();
    let url = "https://warm-polluted.test/page";
    backend.load_html(
        tab_id,
        "<!doctype html><html><body>fresh</body></html>",
        None,
        Some(url),
        1,
    );
    poll_until_committed(&mut backend, &mut snapshots, tab_id, url);

    // 真复位已 wipe 污染：新文档全局对象不含 stale 残留。
    let value = execute_script(
        &mut backend,
        &mut snapshots,
        tab_id,
        77002,
        "return String(globalThis.__warm_pollutant)",
    )
    .unwrap();
    assert_eq!(
        value,
        AutomationValue::String("undefined".into()),
        "真复位臂必须 wipe 污染残留"
    );
}

/// 场景 3（设计 §7-1）：崩溃槽失效 → 认领校验失败 → 清槽冷启回退（crash_fallback），
/// 无等待原语，tab 绑定全新 renderer。
#[test]
fn crashed_warm_spare_falls_back_to_cold_spawn() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = real_backend();
    let tab_id = TabId(903);

    backend.note_warm_tick(false);
    backend.note_warm_tick(true);
    let Some((spare_rid, _)) = backend.warm_spare else {
        panic!("聚焦边沿应触发预热");
    };
    // 以子进程退出为准令 spare 失效（活性判定不得依赖 RendererState）：shutdown 后
    // child=None，`is_alive`（exit_reason 为 None 才算活）即 false——与崩溃同观测量。
    let _ = backend.manager.get_renderer(spare_rid).unwrap().shutdown();

    backend.ensure_renderer(tab_id, (800, 600));
    assert!(backend.warm_spare.is_none(), "失效槽必须清空");
    let bound_rid = backend
        .tab_to_renderer
        .get(&tab_id)
        .copied()
        .expect("冷启后必须绑定 renderer");
    assert_ne!(bound_rid, spare_rid, "崩溃 spare 不得被认领，必须冷启");
}

/// 场景 4（设计 §7-1/§3）：T_idle 到期回收（shutdown_renderer 强杀清槽）→ 后续
/// ensure_renderer 冷启。回收时限经构造字段覆写（env 覆写的测试等价面）。
#[test]
fn warm_idle_reap_clears_slot_for_later_cold_spawn() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = real_backend();
    backend.warm_idle_reap = Duration::from_millis(50);
    let tab_id = TabId(904);

    backend.note_warm_tick(false);
    backend.note_warm_tick(true);
    let Some((spare_rid, _)) = backend.warm_spare else {
        panic!("聚焦边沿应触发预热");
    };

    let mut snapshots = HashMap::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        backend.poll(&mut snapshots, &mut HashMap::new(), None, true);
        if backend.warm_spare.is_none() {
            break;
        }
        assert!(Instant::now() < deadline, "T_idle 到期后必须回收 spare");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        backend.manager.get_renderer(spare_rid).is_none(),
        "回收必须 shutdown_renderer（句柄移除，内存归还）"
    );

    backend.ensure_renderer(tab_id, (800, 600));
    let bound_rid = backend.tab_to_renderer.get(&tab_id).copied().expect("回收后必须冷启");
    assert_ne!(bound_rid, spare_rid, "回收后的新 tab 必须走冷启");
}

/// 场景 5（设计 §7-1/§2.1 + 返修 R1）：触发边沿一次性——已有 spare 不重复触发；
/// 认领后进入 D 延迟补位 pending：pending 期边沿被抑制不 spawn，到点由 poll tick
/// 补位入槽，补位后边沿不再重复触发。首个冷启动迭代的首边沿即时 arm 由本测试
/// 开头的边沿触发覆盖。
#[test]
fn focus_edge_triggers_once_per_arm_and_rearms_after_claim() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = real_backend();
    backend.warm_refill_delay = Duration::from_millis(80);
    let tab_id = TabId(905);

    backend.note_warm_tick(false);
    backend.note_warm_tick(true);
    let Some((first_rid, _)) = backend.warm_spare else {
        panic!("首次聚焦边沿应触发预热");
    };

    // 已有 spare：同态/再次边沿转发都不得重复触发。
    backend.note_warm_tick(true);
    backend.note_warm_tick(false);
    backend.note_warm_tick(true);
    let still = backend.warm_spare.map(|(rid, _)| rid);
    assert_eq!(still, Some(first_rid), "已有 spare 不得重复触发");

    // 认领清槽 → 延迟补位 pending；pending 期边沿被抑制，不得立即 spawn。
    backend.ensure_renderer(tab_id, (800, 600));
    assert!(backend.warm_spare.is_none());
    assert!(
        backend.warm_refill_pending_since.is_some(),
        "认领必须置延迟补位 pending"
    );
    backend.note_warm_tick(false);
    backend.note_warm_tick(true);
    backend.note_warm_tick(false);
    backend.note_warm_tick(true);
    assert!(
        backend.warm_spare.is_none() && backend.warm_refill_pending_since.is_some(),
        "pending 期边沿不得触发 spawn"
    );

    // 到点后由 poll tick 补位入槽（全新 spare）并发射清 pending。
    let mut snapshots = HashMap::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    let refill_rid = loop {
        backend.poll(&mut snapshots, &mut HashMap::new(), None, true);
        if let Some((rid, _)) = backend.warm_spare {
            assert_ne!(rid, first_rid, "延迟补位必须是全新 spare");
            break rid;
        }
        assert!(Instant::now() < deadline, "到点后 poll tick 必须补位 spawn");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(backend.warm_refill_pending_since.is_none(), "补位发射必须清 pending");

    // 补位后槽占用：边沿不重复触发。
    backend.note_warm_tick(false);
    backend.note_warm_tick(true);
    assert_eq!(
        backend.warm_spare.map(|(rid, _)| rid),
        Some(refill_rid),
        "补位后边沿不得重复触发"
    );
}

/// 场景 6（返修 R3，TV-F3）：spare 崩溃后 poll tick 清槽（与认领侧场景 3 对偶）——
/// poll 检测槽位 !alive 即清槽，后续 ensure_renderer 走冷启。
#[test]
fn crashed_warm_spare_cleared_by_poll_tick() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = real_backend();
    let tab_id = TabId(906);

    backend.note_warm_tick(false);
    backend.note_warm_tick(true);
    let Some((spare_rid, _)) = backend.warm_spare else {
        panic!("聚焦边沿应触发预热");
    };
    // 以子进程退出为准令 spare 失效（shutdown 后 child=None，`is_alive` 即 false）。
    let _ = backend.manager.get_renderer(spare_rid).unwrap().shutdown();

    // 认领侧回退由场景 3 覆盖；此处验 poll 侧：poll tick 检测槽位失效并清槽。
    let mut snapshots = HashMap::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        backend.poll(&mut snapshots, &mut HashMap::new(), None, true);
        if backend.warm_spare.is_none() {
            break;
        }
        assert!(Instant::now() < deadline, "poll tick 必须清掉崩溃 spare 槽位");
        std::thread::sleep(Duration::from_millis(10));
    }

    // 清槽后无 pending 干扰，ensure_renderer 自然落冷启。
    assert!(backend.warm_refill_pending_since.is_none());
    backend.ensure_renderer(tab_id, (800, 600));
    let bound_rid = backend
        .tab_to_renderer
        .get(&tab_id)
        .copied()
        .expect("清槽后必须冷启绑定");
    assert_ne!(bound_rid, spare_rid, "崩溃 spare 不得被认领");
}

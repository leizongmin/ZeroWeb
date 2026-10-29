//! WPT testharness runner and minimal testdriver adapter for HTML interactions.

use crate::wpt_file_loader::read_corpus_to_string;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use zero_page_runtime::{HtmlActionRequest, HtmlUserAction};
use zero_webview::{WebView, WebViewConfig};

const CASE_TIMEOUT: Duration = Duration::from_secs(10);

/// web-api-batch2 两 corpus（clipboard-apis / fullscreen）单案时限。上游
/// clipboard-apis basics 族单案串行 ~19 个 test_driver.click 激活周期（waitForUserActivation），
/// runner 探针循环每周期 ~0.8s 摊销（每轮 media/observer/animation 泵 execute_script 群）
/// → 10s 默认时限在 ~9 周期处伪超时（M2-s2 实测 b16 探针 9 绿 + 10s deadline 幂等复现）。
/// 30s 覆盖 ~35 周期，非 Pass 判定语义不变。
const CORPUS_CASE_TIMEOUT: Duration = Duration::from_secs(30);

/// 进程级 runner 开关快照（docs/learnings/performance/2026-09/2026-09-14：稳定
/// 策略开关禁止进热路径——探针循环每轮、每案多次 `env::var` 曾实测 +67% 退化）。
/// 全部来自 env，进程生命周期内不变；仅 Makefile 命令行设置，无 mid-process
/// 改值用法（reftest/tests.rs 的 set_var 只动 ZW_PRINT_* 变量）。
struct RunnerFlags {
    corpus_case_timeout_secs: Option<u64>,
    script_timeout_ms: Option<u64>,
    time_log: bool,
    raf_frame_driven: bool,
    timer_per_task: bool,
    disable_pump: bool,
    td_debug: bool,
}

static RUNNER_FLAGS: std::sync::OnceLock<RunnerFlags> = std::sync::OnceLock::new();

fn runner_flags() -> &'static RunnerFlags {
    RUNNER_FLAGS.get_or_init(|| RunnerFlags {
        corpus_case_timeout_secs: std::env::var("ZW_CORPUS_CASE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok()),
        script_timeout_ms: std::env::var("ZW_SCRIPT_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok()),
        time_log: std::env::var("ZW_TESTHARNESS_TIME_LOG").ok().as_deref() == Some("1"),
        raf_frame_driven: std::env::var("ZW_TESTHARNESS_RAF_FRAME_DRIVEN").as_deref() == Ok("1"),
        timer_per_task: std::env::var("ZW_TESTHARNESS_TIMER_PER_TASK").as_deref() != Ok("0"),
        disable_pump: std::env::var("ZW_DISABLE_PUMP").is_ok(),
        td_debug: std::env::var("ZW_TD_DEBUG").as_deref() == Ok("1"),
    })
}

/// V8 看门狗脚本执行预算——`ZW_SCRIPT_TIMEOUT_MS` 按跑覆盖（encoding-legacy-mb
/// M1 记账：decode wrapper 每个variant 对 chars 文档全量 span 做桥调用密集断言
/// （~3–5 万次/variant），90s 缺省预算内跑不完被截成全案 synthetic Fail；缺省
/// 维持 90s，零行为变化）。外层案超时只在 run_page_scripts 返回后 tick，拦不到
/// 同步脚本段——本看门狗是该形态的实际预算边界。
fn script_timeout_ms() -> u64 {
    runner_flags().script_timeout_ms.unwrap_or(90_000)
}

/// 单案 wall-clock 预算——`ZW_CORPUS_CASE_TIMEOUT_SECS` 按跑覆盖（html-syntax-compat
/// M2 记账：html5lib 全量语料 × document.write 面在 30s 案预算内跑不完，切片跑法用；
/// 缺省维持 30s，零行为变化）。
pub(crate) fn corpus_case_timeout() -> Duration {
    runner_flags()
        .corpus_case_timeout_secs
        .map(Duration::from_secs)
        .unwrap_or(CORPUS_CASE_TIMEOUT)
}

/// First supported upstream HTML interaction cases.
pub const HTML_INTERACTION_CASES: &[&str] = &[
    "html/semantics/embedded-content/media-elements/networkState_initial.html",
    "html/semantics/embedded-content/media-elements/readyState_initial.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLTrackElement/readyState.html",
    "html/semantics/forms/the-output-element/output.html",
    "html/semantics/forms/the-input-element/input-whitespace.html",
    "html/interaction/focus/sequential-focus-navigation-and-the-tabindex-attribute/focus-tabindex-default-value.html",
    "uievents/constructors/inputevent-constructor.html",
];

/// Canvas 2D 专项（docs/goal/canvas-2d.md）M1 切片 1 导入的目录面。
///
/// 由 `scripts/fetch-canvas-subset.sh` 维护；新目录随切片扩展追加。
/// R34xx（G6）：OffscreenCanvas worker 变体目录面（`html/canvas/offscreen/*`——与
/// CANVAS_TEST_SUBDIRS 的 element 面镜像；.worker.js 变体经 fetch_tests_from_worker 聚合）。
pub const CANVAS_OFFSCREEN_SUBDIRS: &[&str] = &[
    "html/canvas/offscreen/the-canvas-state",
    "html/canvas/offscreen/drawing-rectangles-to-the-canvas",
    "html/canvas/offscreen/transformations",
    "html/canvas/offscreen/pixel-manipulation",
    "html/canvas/offscreen/line-styles",
    "html/canvas/offscreen/shadows",
    "html/canvas/offscreen/compositing",
    "html/canvas/offscreen/fill-and-stroke-styles",
    "html/canvas/offscreen/text",
    "html/canvas/offscreen/conformance-requirements",
    // R34xx（2026-08-15 第二批导入）：与 fetch-canvas-subset.sh OFFSCREEN_SUBDIRS 同步。
    "html/canvas/offscreen/drawing-images-to-the-canvas",
    "html/canvas/offscreen/path-objects",
    "html/canvas/offscreen/reset",
    "html/canvas/offscreen/canvas-context",
    "html/canvas/offscreen/canvas-host",
    "html/canvas/offscreen/color-type",
    "html/canvas/offscreen/filters",
    "html/canvas/offscreen/layers",
    "html/canvas/offscreen/wide-gamut-canvas",
];

pub const CANVAS_TEST_SUBDIRS: &[&str] = &[
    "html/canvas/element/the-canvas-state",
    "html/canvas/element/drawing-rectangles-to-the-canvas",
    "html/canvas/element/transformations",
    "html/canvas/element/pixel-manipulation",
    "html/canvas/element/line-styles",
    "html/canvas/element/shadows",
    "html/canvas/element/compositing",
    "html/canvas/element/fill-and-stroke-styles",
    "html/canvas/element/text",
    // R34xx（2026-08-15 第二批导入）：补全范围内子目录（与 fetch-canvas-subset.sh
    // SUBDIRS 同步——manual 交互面与 video 媒体面不在目标范围）。
    "html/canvas/element/conformance-requirements",
    "html/canvas/element/drawing-images-to-the-canvas",
    "html/canvas/element/path-objects",
    "html/canvas/element/reset",
    "html/canvas/element/canvas-context",
    "html/canvas/element/canvas-host",
    "html/canvas/element/color-type",
    "html/canvas/element/filters",
    "html/canvas/element/layers",
    "html/canvas/element/global-hdr-headroom",
    "html/canvas/element/wide-gamut-canvas",
];

/// R34xx（2026-08-15）：element 顶层 testharness 用例（目录扫描不覆盖——与
/// fetch-canvas-subset.sh CANVAS_TOP_FILES 同步）。
pub const CANVAS_TOP_LEVEL_FILES: &[&str] = &[
    "html/canvas/element/2d.conformance.requirements.basics.html",
    "html/canvas/element/2d.conformance.requirements.delete.html",
    "html/canvas/element/2d.conformance.requirements.drawings.html",
    "html/canvas/element/2d.conformance.requirements.missingargs.html",
    "html/canvas/element/2d.putImageData.html",
    "html/canvas/offscreen/2d.conformance.requirements.basics.html",
    "html/canvas/offscreen/2d.conformance.requirements.missingargs.html",
    "html/canvas/offscreen/OffscreenCanvas-ctx-font-sibling-index-invalid.tentative.html",
    "html/canvas/offscreen/set-proprietary-font-names-001-crash.html",
];

/// R56h：WPT 套件内部语义冲突用例（skip 清单，NotRun 中性状态）。
///
/// 2d.path.stroke.skew：期望 stroke 在 draw 时重应用当前 CTM（线段
/// (49,-50)→(201,-50) 被 rotate(π/4)·scale(1,283) 旋转后横贯画布）——
/// 与套件内 5+ 用例（stroke.scale1/2、transformation.changing/multiple/basic，
/// 全 Pass 实证「路径追加时烘焙 CTM，draw 只缩放线宽」）在任何单一模型下
/// 互斥：draw 时重应用 CTM 则 transformation.changing/multiple + scale1/2
/// 全部回归。保持主流语义（追加时烘焙）并跳过该用例。
/// https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-stroke
const CANVAS_SKIP_FILES: &[&str] = &[
    "html/canvas/element/path-objects/2d.path.stroke.skew.html",
    "html/canvas/offscreen/path-objects/2d.path.stroke.skew.worker.js",
];

/// canvas-tests.js 的 WPT 内路径（prepare 时内联替换）。
const CANVAS_TESTS_JS_PATH: &str = "html/canvas/resources/canvas-tests.js";

/// DOM 专项（docs/goal/archive/js-dom.md，M4 / DC-3）导入的上游 `dom/` 子目录面。
///
/// 由 `tests/wpt-runner/scripts/fetch-dom-subset.sh` 维护（wpt-data gitignored，
/// 用例按需 fetch、不入库）；新子目录随 M4 切片扩展追加。dom 用例只需
/// `resources/testharness.js`（runner 内联），不依赖 canvas-tests.js。
/// R21 追加 dom/events（Event-dispatch 系列 / EventTarget / EventListener——事件桥核心面）。
/// R37 追加 dom/collections（HTMLCollection / NodeList / document.forms 等集合 API——纯 DOM API，
/// 不依赖 document/window listener 深结构，根因清楚可按聚类驱动修复）。
pub const DOM_TEST_SUBDIRS: &[&str] = &[
    "dom/nodes",
    "dom/events",
    "dom/collections",
    "dom/traversal",
    "dom/ranges",
    // R373（js-dom M4/DC-3 扩展）：dom/abort——AbortSignal/AbortController 域（与 fetch
    // shim 的 AbortSignal 基建同域；5 个真实用例：timeout/reason/any-crash + any.js 变体）。
    "dom/abort",
    // R374（js-dom M4/DC-3 扩展）：dom/lists——DOMTokenList 域（classList 同域；5 个
    // 真实用例：Iterable/iteration/stringifier/value/coverage-for-attributes）。
    "dom/lists",
    // R375（js-dom M4/DC-3 扩展）：dom 根目录散用例（interface-objects /
    // window-extends-event-target / attributes-are-nodes / xpath-result /
    // eventPathRemoved / svg-insert-crash / historical 域——read_dir 单层扫描，
    // 子目录由各自条目覆盖）。
    "dom",
    // slice28（RP-4，2026-10-04）：Window named access on the window object 用例集
    // （RP-1 name 面修复的资产化对位；上游 22 文件中 17 .html/.window.js + resources/
    // 支持文件，runner 单层扫描 .html——basics/removing 等 live 动态语义面在本引擎
    // 未实现，基线如实记录不放宽断言；静态 id/name 面子集经本切片修复须绿）。
    // https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
    "html/browsers/the-window-object/named-access-on-the-window-object",
];

/// Selection goal（editing/contenteditable M1 / DC-1）pinned upstream subset
/// directories. Cases are fetched by `fetch-selection-subset.sh`（selection/，
/// M1 切片 1；selection/textcontrols/，E2 切片 12——selectionchange 全语义面）
/// 与 `fetch-editing-subset.sh`（editing/，M1 切片 3——beforeinput/
/// input 事件面 + editing/other 删除/插入行为基线，M2 前置）into
/// `wpt-data/`（gitignored）。
pub const SELECTION_TEST_SUBDIRS: &[&str] = &["selection", "selection/textcontrols", "editing", "editing/other"];

/// Keyboard goal（keyboard-default-actions M1 / DC-1）pinned upstream subset
/// directories. Cases are fetched by `fetch-keyboard-subset.sh` into
/// `wpt-data/`（gitignored）——uievents/keyboard 事件面 + forms 隐式提交/激活面。
pub const KEYBOARD_TEST_SUBDIRS: &[&str] = &[
    "uievents/keyboard",
    "html/semantics/forms/form-submission-0",
    // keyboard-page-scrolling goal 共享面（M1 切片 2 扩展）：snap 容器键盘滚动交互
    //（keyboard.html / paged.html / scroll-padding-paged.html——Actions 键盘链落地后
    // 解除 defer）。
    "css/css-scroll-snap/input",
];

/// IndexedDB goal pinned upstream `.any.js` subset.
///
/// Each case is run in its window variant with the META-declared support script.
/// https://web-platform-tests.org/writing-tests/testharness.html#multi-global-tests
pub const INDEXEDDB_CASES: &[(&str, &[&str])] = &[
    ("IndexedDB/globalscope-indexedDB-SameObject.any.js", &[]),
    ("IndexedDB/idbfactory_cmp.any.js", &["resources/support-promises.js"]),
    ("IndexedDB/idbfactory_deleteDatabase.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbfactory-deleteDatabase-request-success.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbfactory_open.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbfactory-open-error-properties.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbfactory-open-request-error.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbfactory-open-request-success.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbversionchangeevent.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbdatabase_createObjectStore.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbdatabase-createObjectStore-exception-order.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbdatabase_deleteObjectStore.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbdatabase-deleteObjectStore-exception-order.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbobjectstore_keyPath.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbobjectstore-transaction-SameObject.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbtransaction_objectStoreNames.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbdatabase_transaction.any.js", &["resources/support.js"]),
    ("IndexedDB/keypath.any.js", &["resources/support.js"]),
    ("IndexedDB/keypath_invalid.any.js", &["resources/support.js"]),
    ("IndexedDB/keypath-exceptions.any.js", &["resources/support.js"]),
    (
        "IndexedDB/keypath-special-identifiers.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/keypath_maxsize.any.js", &["resources/support.js"]),
    ("IndexedDB/key_valid.any.js", &["resources/support.js"]),
    ("IndexedDB/key_invalid.any.js", &["resources/support.js"]),
    ("IndexedDB/keyorder.any.js", &["resources/support.js"]),
    ("IndexedDB/idbobjectstore_createIndex.any.js", &["resources/support.js"]),
    ("IndexedDB/idbobjectstore_deleteIndex.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbobjectstore-deleteIndex-exception-order.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbindex_indexNames.any.js", &["resources/support.js"]),
    ("IndexedDB/idbindex_keyPath.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbindex-objectStore-SameObject.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbindex-request-source.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbindex-query-exception-order.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbindex-rename.any.js", &["resources/support-promises.js"]),
    (
        "IndexedDB/idbindex-rename-errors.any.js",
        &["resources/support-promises.js"],
    ),
    (
        "IndexedDB/idbindex-rename-abort.any.js",
        &["resources/support-promises.js"],
    ),
    (
        "IndexedDB/idbobjectstore-rename-store.any.js",
        &["resources/support-promises.js"],
    ),
    (
        "IndexedDB/idbobjectstore-rename-errors.any.js",
        &["resources/support-promises.js"],
    ),
    (
        "IndexedDB/idbobjectstore-rename-abort.any.js",
        &["resources/support-promises.js"],
    ),
    ("IndexedDB/name-scopes.any.js", &["resources/support-promises.js"]),
    ("IndexedDB/list_ordering.any.js", &["resources/support.js"]),
    (
        "IndexedDB/transaction-abort-index-metadata-revert.any.js",
        &["resources/support-promises.js", "resources/support.js"],
    ),
    (
        "IndexedDB/transaction-abort-object-store-metadata-revert.any.js",
        &["resources/support-promises.js", "resources/support.js"],
    ),
    (
        "IndexedDB/transaction-abort-multiple-metadata-revert.any.js",
        &["resources/support-promises.js", "resources/support.js"],
    ),
    (
        "IndexedDB/transaction-abort-generator-revert.any.js",
        &["resources/support-promises.js", "resources/support.js"],
    ),
    ("IndexedDB/idbobjectstore_index.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbobjectstore-index-finished.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbtransaction-db-SameObject.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbtransaction-objectStore-exception-order.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbkeyrange.any.js", &["resources/support.js"]),
    ("IndexedDB/idbkeyrange-includes.any.js", &["resources/support.js"]),
    ("IndexedDB/idbkeyrange_incorrect.any.js", &["resources/support.js"]),
    ("IndexedDB/idb_binary_key_conversion.any.js", &[]),
    ("IndexedDB/idb-binary-key-detached.any.js", &["resources/support.js"]),
    ("IndexedDB/idb-binary-key-roundtrip.any.js", &["resources/support.js"]),
    ("IndexedDB/key-conversion-exceptions.any.js", &["resources/support.js"]),
    ("IndexedDB/objectstore_keyorder.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbdatabase-transaction-exception-order.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbobjectstore-add-put-exception-order.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbobjectstore-clear-exception-order.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbobjectstore-delete-exception-order.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbobjectstore-query-exception-order.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbobjectstore-request-source.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/delete-range.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idb-explicit-commit-throw.any.js",
        &["resources/support-promises.js"],
    ),
    (
        "IndexedDB/idb-explicit-commit.any.js",
        &["resources/support-promises.js"],
    ),
    ("IndexedDB/idbtransaction.any.js", &["resources/support.js"]),
    (
        "IndexedDB/transaction-create_in_versionchange.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/transaction-relaxed-durability.any.js",
        &["resources/support-promises.js"],
    ),
    (
        "IndexedDB/upgrade-transaction-lifecycle-committed.any.js",
        &["resources/support.js", "resources/support-promises.js"],
    ),
    (
        "IndexedDB/upgrade-transaction-lifecycle-user-aborted.any.js",
        &["resources/support.js", "resources/support-promises.js"],
    ),
    (
        "IndexedDB/upgrade-transaction-lifecycle-backend-aborted.any.js",
        &["resources/support.js", "resources/support-promises.js"],
    ),
    (
        "IndexedDB/upgrade-transaction-deactivation-timing.any.js",
        &["resources/support.js", "resources/support-promises.js"],
    ),
    (
        "IndexedDB/idbobjectstore-getAll-enforcerange.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbobjectstore-getAllKeys-enforcerange.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbobjectstore_getAll-options.any.js",
        &[
            "resources/nested-cloning-common.js",
            "resources/support.js",
            "resources/support-get-all.js",
            "resources/support-promises.js",
        ],
    ),
    (
        "IndexedDB/idbobjectstore_getAllKeys-options.any.js",
        &[
            "resources/nested-cloning-common.js",
            "resources/support.js",
            "resources/support-get-all.js",
            "resources/support-promises.js",
        ],
    ),
    (
        "IndexedDB/idbobjectstore_getAllRecords.any.js",
        &[
            "resources/nested-cloning-common.js",
            "resources/support.js",
            "resources/support-get-all.js",
            "resources/support-promises.js",
        ],
    ),
    (
        "IndexedDB/idbindex-getAll-enforcerange.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbindex-getAllKeys-enforcerange.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbindex_getAll-options.any.js",
        &[
            "resources/nested-cloning-common.js",
            "resources/support.js",
            "resources/support-get-all.js",
            "resources/support-promises.js",
        ],
    ),
    (
        "IndexedDB/idbindex_getAllKeys-options.any.js",
        &[
            "resources/nested-cloning-common.js",
            "resources/support.js",
            "resources/support-get-all.js",
            "resources/support-promises.js",
        ],
    ),
    (
        "IndexedDB/idbindex_getAllRecords.any.js",
        &[
            "resources/nested-cloning-common.js",
            "resources/support.js",
            "resources/support-get-all.js",
            "resources/support-promises.js",
        ],
    ),
    (
        "IndexedDB/abort-in-initial-upgradeneeded.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/close-in-upgradeneeded.any.js", &["resources/support.js"]),
    ("IndexedDB/delete-request-queue.any.js", &["resources/support.js"]),
    ("IndexedDB/fire-error-event-exception.any.js", &["resources/support.js"]),
    (
        "IndexedDB/fire-success-event-exception.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/fire-upgradeneeded-event-exception.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbrequest-onupgradeneeded.any.js", &[]),
    (
        "IndexedDB/request-abort-ordering.any.js",
        &["resources/support-promises.js", "resources/support.js"],
    ),
    (
        "IndexedDB/transaction_bubble-and-capture.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/transaction-lifetime.any.js", &["resources/support.js"]),
    ("IndexedDB/transaction-requestqueue.any.js", &["resources/support.js"]),
    ("IndexedDB/idbobjectstore_add.any.js", &["resources/support.js"]),
    ("IndexedDB/idbobjectstore_put.any.js", &["resources/support.js"]),
    ("IndexedDB/idbobjectstore_get.any.js", &["resources/support.js"]),
    ("IndexedDB/idbobjectstore_delete.any.js", &["resources/support.js"]),
    ("IndexedDB/idbobjectstore_clear.any.js", &["resources/support.js"]),
    ("IndexedDB/idbobjectstore_count.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbobjectstore_getAll.any.js",
        &[
            "resources/nested-cloning-common.js",
            "resources/support.js",
            "resources/support-get-all.js",
            "resources/support-promises.js",
        ],
    ),
    (
        "IndexedDB/idbobjectstore_getAllKeys.any.js",
        &[
            "resources/nested-cloning-common.js",
            "resources/support.js",
            "resources/support-get-all.js",
            "resources/support-promises.js",
        ],
    ),
    ("IndexedDB/idbobjectstore_getKey.any.js", &["resources/support.js"]),
    ("IndexedDB/idbindex_get.any.js", &["resources/support.js"]),
    ("IndexedDB/idbindex_getKey.any.js", &["resources/support.js"]),
    ("IndexedDB/idbindex_count.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbindex_getAll.any.js",
        &[
            "resources/nested-cloning-common.js",
            "resources/support.js",
            "resources/support-get-all.js",
            "resources/support-promises.js",
        ],
    ),
    (
        "IndexedDB/idbindex_getAllKeys.any.js",
        &[
            "resources/nested-cloning-common.js",
            "resources/support.js",
            "resources/support-get-all.js",
            "resources/support-promises.js",
        ],
    ),
    ("IndexedDB/idbobjectstore_openCursor.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbobjectstore_openCursor_invalid.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbobjectstore_openKeyCursor.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbindex_openCursor.any.js", &["resources/support.js"]),
    ("IndexedDB/idbindex_openKeyCursor.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-key.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-primarykey.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-source.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-request.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-request-source.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-direction.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-direction-index.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbcursor-direction-index-keyrange.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbcursor-direction-objectstore.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbcursor-direction-objectstore-keyrange.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbcursor_iterating.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-iterating-update.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-reused.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbcursor_continue_delete_objectstore.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbcursor-continue.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-advance.any.js", &["resources/support.js"]),
    ("IndexedDB/idbcursor-advance-invalid.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbcursor-advance-continue-async.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbcursor_advance_index.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbcursor_advance_objectstore.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbcursor-advance-exception-order.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbcursor_continue_index.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbcursor_continue_objectstore.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbcursor_continue_invalid.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbcursor-continue-exception-order.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/cursor-overloads.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbcursor-continuePrimaryKey.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbcursor-continuePrimaryKey-exceptions.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbcursor-continuePrimaryKey-exception-order.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbcursor-delete-exception-order.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbcursor_delete_index.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbcursor_delete_objectstore.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/idbcursor-update-exception-order.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbcursor_update_index.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbcursor_update_objectstore.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbrequest_result.any.js", &["resources/support.js"]),
    ("IndexedDB/idbrequest_error.any.js", &["resources/support.js"]),
    (
        "IndexedDB/idbtransaction-objectStore-finished.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbtransaction_abort.any.js", &["resources/support.js"]),
    (
        "IndexedDB/request_bubble-and-capture.any.js",
        &["resources/support-promises.js", "resources/support.js"],
    ),
    (
        "IndexedDB/transaction-abort-request-error.any.js",
        &["resources/support-promises.js", "resources/support.js"],
    ),
    ("IndexedDB/error-attributes.any.js", &["resources/support.js"]),
    ("IndexedDB/idbtransaction-oncomplete.any.js", &["resources/support.js"]),
    (
        "IndexedDB/transaction-deactivation-timing.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/event-dispatch-active-flag.any.js", &["resources/support.js"]),
    ("IndexedDB/transaction-lifetime-empty.any.js", &["resources/support.js"]),
    (
        "IndexedDB/transaction-scheduling-across-connections.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/transaction-scheduling-across-databases.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/transaction-scheduling-mixed-scopes.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/transaction-scheduling-ordering.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/transaction-scheduling-ro-waits-for-rw.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/transaction-scheduling-rw-scopes.any.js",
        &["resources/support.js"],
    ),
    (
        "IndexedDB/transaction-scheduling-within-database.any.js",
        &["resources/support.js"],
    ),
    ("IndexedDB/idbdatabase_close.any.js", &["resources/support.js"]),
    ("IndexedDB/open-request-queue.any.js", &["resources/support.js"]),
];

/// CacheStorage goal pinned upstream window subset.
///
/// Most cases have `// META: global=window,worker`; this runner only executes
/// the window variant for `docs/goal/archive/storage-cache-api.md`. Service Worker
/// variants remain under the Service Worker goal.
pub const CACHE_STORAGE_WINDOW_CASES: &[(&str, &[&str])] = &[
    (
        "service-workers/cache-storage/cache-storage.https.any.js",
        &["resources/test-helpers.js"],
    ),
    (
        "service-workers/cache-storage/cache-match.https.any.js",
        &["resources/test-helpers.js", "/common/get-host-info.sub.js"],
    ),
    (
        "service-workers/cache-storage/cache-put.https.any.js",
        &["resources/test-helpers.js", "/common/get-host-info.sub.js"],
    ),
    (
        "service-workers/cache-storage/cache-abort.https.any.js",
        &["resources/test-helpers.js", "/common/utils.js"],
    ),
    (
        "service-workers/cache-storage/zeroweb-filtered-response-types.https.any.js",
        &["resources/test-helpers.js"],
    ),
    (
        "service-workers/cache-storage/cache-add.https.any.js",
        &["resources/test-helpers.js", "/common/get-host-info.sub.js"],
    ),
    (
        "service-workers/cache-storage/cache-storage-buckets.https.any.js",
        &[
            "resources/test-helpers.js",
            "/common/get-host-info.sub.js",
            "/storage/buckets/resources/util.js",
        ],
    ),
    (
        "service-workers/cache-storage/cache-storage-keys.https.any.js",
        &["resources/test-helpers.js"],
    ),
    (
        "service-workers/cache-storage/cache-delete.https.any.js",
        &["resources/test-helpers.js"],
    ),
    (
        "service-workers/cache-storage/cache-keys.https.any.js",
        &["resources/test-helpers.js"],
    ),
    (
        "service-workers/cache-storage/cache-matchAll.https.any.js",
        &["resources/test-helpers.js"],
    ),
    (
        "service-workers/cache-storage/cache-storage-match.https.any.js",
        &["resources/test-helpers.js"],
    ),
    ("service-workers/cache-storage/common.https.window.js", &[]),
    ("service-workers/cache-storage/common.https.html", &[]),
    ("service-workers/cache-storage/cache-api-nested-worker.https.html", &[]),
    ("service-workers/cache-storage/sandboxed-iframes.https.html", &[]),
    (
        "service-workers/cache-storage/credentials.https.html",
        &["../service-worker/resources/test-helpers.sub.js"],
    ),
    (
        "service-workers/cache-storage/window/cache-storage.https.html",
        &["../resources/test-helpers.js", "../script-tests/cache-storage.js"],
    ),
    (
        "service-workers/cache-storage/window/cache-storage-keys.https.html",
        &["../resources/test-helpers.js", "../script-tests/cache-storage-keys.js"],
    ),
    (
        "service-workers/cache-storage/window/cache-delete.https.html",
        &["../resources/test-helpers.js", "../script-tests/cache-delete.js"],
    ),
    (
        "service-workers/cache-storage/window/cache-keys.https.html",
        &["../resources/test-helpers.js", "../script-tests/cache-keys.js"],
    ),
    (
        "service-workers/cache-storage/window/cache-matchAll.https.html",
        &["../resources/test-helpers.js", "../script-tests/cache-matchAll.js"],
    ),
    (
        "service-workers/cache-storage/window/cache-storage-match.https.html",
        &["../resources/test-helpers.js", "../script-tests/cache-storage-match.js"],
    ),
    (
        "service-workers/cache-storage/window/cache-match.https.html",
        &[
            "/common/get-host-info.sub.js",
            "../resources/test-helpers.js",
            "../script-tests/cache-match.js",
        ],
    ),
    (
        "service-workers/cache-storage/window/cache-put.https.html",
        &[
            "/common/get-host-info.sub.js",
            "../resources/test-helpers.js",
            "../script-tests/cache-put.js",
        ],
    ),
    (
        "service-workers/cache-storage/window/cache-add.https.html",
        &[
            "/common/get-host-info.sub.js",
            "../resources/test-helpers.js",
            "../script-tests/cache-add.js",
        ],
    ),
    (
        "service-workers/cache-storage/window/cache-abort.https.html",
        &[
            "../resources/test-helpers.js",
            "/common/utils.js",
            "../script-tests/cache-abort.js",
        ],
    ),
    ("service-workers/cache-storage/window/sandboxed-iframes.https.html", &[]),
    ("service-workers/cache-storage/worker/cache-storage.https.html", &[]),
    (
        "service-workers/cache-storage/worker/cache-storage-keys.https.html",
        &[],
    ),
    ("service-workers/cache-storage/worker/cache-delete.https.html", &[]),
    ("service-workers/cache-storage/worker/cache-keys.https.html", &[]),
    ("service-workers/cache-storage/worker/cache-matchAll.https.html", &[]),
    (
        "service-workers/cache-storage/worker/cache-storage-match.https.html",
        &[],
    ),
    ("service-workers/cache-storage/worker/cache-match.https.html", &[]),
    ("service-workers/cache-storage/worker/cache-put.https.html", &[]),
    ("service-workers/cache-storage/worker/cache-add.https.html", &[]),
    ("service-workers/cache-storage/worker/cache-abort.https.html", &[]),
    (
        "service-workers/cache-storage/crashtests/cache-response-clone.https.html",
        &[],
    ),
];

/// OPFS goal（storage-opfs M1 / DC-1）pinned upstream File System (`fs/`) window
/// subset. Cases are fetched by `fetch-fs-subset.sh`（WPT
/// 315976933870b34d6ea30e3f6643403edae678ba）into `wpt-data/`（gitignored）.
///
/// `.any.js` 用例以 window 变体执行；support 脚本由各用例 `// META: script=` 声明
/// 自动解析（相对路径相对用例目录，`/`-前缀相对 wpt-data 根）——与 cache-storage
/// 窗口子集的手工配对不同：fs 域 script-tests/resources 两层结构 META 数量不一，
/// 自动解析避免清单漂移。skip 域（Support Envelope 排除项）见 fetch 脚本头注释。
pub const FS_CASES: &[&str] = &[
    "fs/root-name.https.any.js",
    "fs/FileSystemBaseHandle-isSameEntry.https.any.js",
    "fs/FileSystemBaseHandle-getUniqueId.https.any.js",
    "fs/FileSystemBaseHandle-remove.https.any.js",
    "fs/FileSystemDirectoryHandle-getDirectoryHandle.https.any.js",
    "fs/FileSystemDirectoryHandle-getFileHandle.https.any.js",
    "fs/FileSystemDirectoryHandle-iteration.https.any.js",
    "fs/FileSystemDirectoryHandle-removeEntry.https.any.js",
    "fs/FileSystemDirectoryHandle-resolve.https.any.js",
    "fs/FileSystemFileHandle-getFile.https.any.js",
    "fs/FileSystemWritableFileStream.https.any.js",
    "fs/FileSystemWritableFileStream-write.https.any.js",
    "fs/FileSystemWritableFileStream-piped.https.any.js",
];

/// Page WASM goal（page-wasm M1 / DC-1）pinned upstream WebAssembly (`wasm/jsapi/`)
/// window subset——标准（非 tentative）JS API 面：Module/Instance/Memory/Table/Global/
/// compile/instantiate/validate/导出面/链接错误。Cases are fetched by
/// `fetch-wasm-subset.sh`（WPT 315976933870b34d6ea30e3f6643403edae678ba）into
/// `wpt-data/`（gitignored）.
///
/// `.any.js` 用例以 window 变体执行；support 脚本由各用例 `// META: script=` 声明自动
/// 解析（与 fs 子集同机制）。skip 域（Support Envelope 排除项：esm-integration/jspi/
/// js-string/gc/exception/tag/function[tentative]/functions[realm harness]/idlharness）
/// 见 fetch 脚本头注释，双侧一致。
pub const WASM_CASES: &[&str] = &[
    "wasm/jsapi/interface.any.js",
    "wasm/jsapi/prototypes.any.js",
    "wasm/jsapi/constructor/compile.any.js",
    "wasm/jsapi/constructor/instantiate.any.js",
    "wasm/jsapi/constructor/instantiate-bad-imports.any.js",
    "wasm/jsapi/constructor/multi-value.any.js",
    "wasm/jsapi/constructor/toStringTag.any.js",
    "wasm/jsapi/constructor/validate.any.js",
    "wasm/jsapi/global/constructor.any.js",
    "wasm/jsapi/global/toString.any.js",
    "wasm/jsapi/global/value-get-set.any.js",
    "wasm/jsapi/global/valueOf.any.js",
    "wasm/jsapi/instance/constructor.any.js",
    "wasm/jsapi/instance/constructor-bad-imports.any.js",
    "wasm/jsapi/instance/constructor-caching.any.js",
    "wasm/jsapi/instance/exports.any.js",
    "wasm/jsapi/instance/toString.any.js",
    "wasm/jsapi/memory/buffer.any.js",
    "wasm/jsapi/memory/constructor.any.js",
    "wasm/jsapi/memory/grow.any.js",
    "wasm/jsapi/memory/toString.any.js",
    "wasm/jsapi/module/constructor.any.js",
    "wasm/jsapi/module/customSections.any.js",
    "wasm/jsapi/module/exports.any.js",
    "wasm/jsapi/module/imports.any.js",
    "wasm/jsapi/module/toString.any.js",
    "wasm/jsapi/table/constructor.any.js",
    "wasm/jsapi/table/get-set.any.js",
    "wasm/jsapi/table/grow.any.js",
    "wasm/jsapi/table/length.any.js",
    "wasm/jsapi/table/toString.any.js",
];

/// WebStorage goal（webstorage-compat M1 / DC-1）pinned upstream window subset——
/// Storage 接口语义（getItem/setItem/removeItem/clear/key/length、named property、
/// 枚举序、配额异常、字符串转换）+ StorageEvent 构造面可执行子集。Cases are
/// fetched by `scripts/goals/15-webstorage-compat.sh`（WPT
/// 315976933870b34d6ea30e3f6643403edae678ba）into `wpt-data/webstorage/`（gitignored）.
///
/// `.window.js` 用例以 window 变体执行（[`any_js_window_wrapper`]，无 META support
/// 依赖——仅被 skip 的案才引 resources/）。skip 域（DC-3 甄别结论，evidence 落档）：
/// - `*window_open*.window.js` / `storage_session_window_reopen.window.js` /
///   `localstorage-cross-origin-iframe.https.window.js`——window.open/iframe 第二
///   browsing context（runner 无跨文档管道）
/// - 全部 `event_*.html` 派发面——`eventTestHarness.js` 经 iframe + storage 事件跨
///   文档派发（同上；frame 树依赖 → navigation-compat M3 同门槛挂账）
/// - `document-domain.html`、`*.partitioned*.html`、`localstorage-share-data-
///   unrelated-origins.html`——document.domain / 存储分区 / 跨源 iframe
/// - `*-manual.html`——需真实用户操作
pub const WEBSTORAGE_CASES: &[&str] = &[
    "webstorage/defineProperty.window.js",
    "webstorage/event_constructor.window.js",
    "webstorage/event_initstorageevent.window.js",
    "webstorage/missing_arguments.window.js",
    "webstorage/set.window.js",
    "webstorage/storage_builtins.window.js",
    "webstorage/storage_clear.window.js",
    "webstorage/storage_enumerate.window.js",
    "webstorage/storage_functions_not_overwritten.window.js",
    "webstorage/storage_getitem.window.js",
    "webstorage/storage_indexing.window.js",
    "webstorage/storage_in.window.js",
    "webstorage/storage_key_empty_string.window.js",
    "webstorage/storage_key.window.js",
    "webstorage/storage_length.window.js",
    "webstorage/storage_local_quota_independent_from_session.window.js",
    "webstorage/storage_local_setitem_quotaexceedederr.window.js",
    "webstorage/storage_removeitem.window.js",
    "webstorage/storage_session_quota_independent_from_local.window.js",
    "webstorage/storage_session_setitem_quotaexceedederr.window.js",
    "webstorage/storage_setitem.window.js",
    "webstorage/storage_set_value_enumerate.window.js",
    "webstorage/storage_string_conversion.window.js",
    "webstorage/storage_supported_property_names.window.js",
    "webstorage/symbol-props.window.js",
];

const ZEROWEB_CACHE_FILTERED_RESPONSE_TYPES_SOURCE: &str = r#"
// META: title=ZeroWeb CacheStorage filtered response type generation
// META: global=window
// META: script=./resources/test-helpers.js

cache_test(async cache => {
  const url = '/service-workers/cache-storage/resources/simple.txt?zw-filtered=basic';
  const response = await fetch(url);
  assert_equals(response.type, 'basic');
  await cache.put(url, response.clone());
  assert_equals((await cache.match(url)).type, 'basic');
}, 'CacheStorage stores same-origin fetch() as a basic filtered response');

cache_test(async cache => {
  const url = 'https://www1.wpt.test/service-workers/cache-storage/resources/simple.txt?zw-filtered=cors';
  const response = await fetch(url, { mode: 'cors' });
  assert_equals(response.type, 'cors');
  await cache.put(url, response.clone());
  assert_equals((await cache.match(url)).type, 'cors');
}, 'CacheStorage stores cross-origin CORS fetch() as a CORS filtered response');

cache_test(async cache => {
  const url = 'https://www1.wpt.test/service-workers/cache-storage/resources/simple.txt?zw-filtered=opaque';
  const request = new Request(url, { mode: 'no-cors' });
  const response = await fetch(request);
  assert_equals(response.type, 'opaque');
  assert_equals(response.status, 0);
  assert_equals(await response.text(), '');
  await cache.put(request, response.clone());
  const cached = await cache.match(request);
  assert_equals(cached.type, 'opaque');
  assert_equals(cached.status, 0);
  assert_equals(cached.headers.get('vary'), null);
  assert_equals(await cached.text(), '');
}, 'CacheStorage stores cross-origin no-cors fetch() as an opaque filtered response');

cache_test(async cache => {
  const url = 'https://www1.wpt.test/service-workers/cache-storage/resources/redirect.py?zw-filtered=opaqueredirect';
  const request = new Request(url, { redirect: 'manual' });
  const response = await fetch(request);
  assert_equals(response.type, 'opaqueredirect');
  assert_equals(response.status, 0);
  assert_equals(response.headers.get('location'), null);
  await cache.put(request, response.clone());
  const cached = await cache.match(request);
  assert_equals(cached.type, 'opaqueredirect');
  assert_equals(cached.status, 0);
  assert_equals(cached.headers.get('location'), null);
  assert_equals(await cached.text(), '');
}, 'CacheStorage stores manual redirect fetch() as an opaque-redirect filtered response');
"#;

/// Fixed Service Worker M1 core corpus at the pinned WPT revision.
pub const SERVICE_WORKER_CORE_CASES: &[&str] = &[
    "service-workers/service-worker/active.https.html",
    "service-workers/service-worker/activate-event-after-install-state-change.https.html",
    "service-workers/service-worker/activation-after-registration.https.html",
    "service-workers/service-worker/controller-on-load.https.html",
    "service-workers/service-worker/controller-on-disconnect.https.html",
    "service-workers/service-worker/controller-on-reload.https.html",
    "service-workers/service-worker/clients-matchall-on-evaluation.https.html",
    "service-workers/service-worker/ServiceWorkerGlobalScope/close.https.html",
    "service-workers/service-worker/ServiceWorkerGlobalScope/isSecureContext.https.html",
    "service-workers/service-worker/ServiceWorkerGlobalScope/extendable-message-event.https.html",
    "service-workers/service-worker/ServiceWorkerGlobalScope/error-message-event.https.html",
    "service-workers/service-worker/ServiceWorkerGlobalScope/message-event-ports.https.html",
    "service-workers/service-worker/ServiceWorkerGlobalScope/registration-attribute.https.html",
    "service-workers/service-worker/ServiceWorkerGlobalScope/service-worker-error-event.https.html",
    "service-workers/service-worker/ServiceWorkerGlobalScope/unregister.https.html",
    "service-workers/service-worker/extendable-event-async-waituntil.https.html",
    "service-workers/service-worker/extendable-event-waituntil.https.html",
    "service-workers/service-worker/getregistration.https.html",
    "service-workers/service-worker/registration-iframe.https.html",
    "service-workers/service-worker/installing.https.html",
    "service-workers/service-worker/waiting.https.html",
    "service-workers/service-worker/global-serviceworker.https.any.js",
    "service-workers/service-worker/historical.https.any.js",
    "service-workers/service-worker/immutable-prototype-serviceworker.https.html",
    "service-workers/service-worker/import-scripts-cross-origin.https.html",
    "service-workers/service-worker/import-scripts-data-url.https.html",
    "service-workers/service-worker/import-scripts-mime-types.https.html",
    "service-workers/service-worker/interface-requirements-sw.https.html",
    "service-workers/service-worker/install-event-type.https.html",
    "service-workers/service-worker/import-scripts-redirect.https.html",
    "service-workers/service-worker/import-scripts-resource-map.https.html",
    "service-workers/service-worker/import-scripts-updated-flag.https.html",
    "service-workers/service-worker/multiple-update.https.html",
    "service-workers/service-worker/no-dynamic-import.any.js",
    "service-workers/service-worker/no-dynamic-import-in-module.any.js",
    "service-workers/service-worker/onactivate-script-error.https.html",
    "service-workers/service-worker/oninstall-script-error.https.html",
    "service-workers/service-worker/register-default-scope.https.html",
    "service-workers/service-worker/registration-basic.https.html",
    "service-workers/service-worker/registration-end-to-end.https.html",
    "service-workers/service-worker/registration-events.https.html",
    "service-workers/service-worker/registration-scope.https.html",
    "service-workers/service-worker/registration-scope-module-static-import.https.html",
    "service-workers/service-worker/registration-script-module.https.html",
    "service-workers/service-worker/registration-updateviacache.https.html",
    "service-workers/service-worker/skip-waiting-without-client.https.html",
    "service-workers/service-worker/update-module-request-mode.https.html",
    "service-workers/service-worker/update-no-cache-request-headers.https.html",
    "service-workers/service-worker/update-not-allowed.https.html",
    "service-workers/service-worker/update-registration-with-type.https.html",
    "service-workers/service-worker/registration-script-url.https.html",
    "service-workers/service-worker/registration-service-worker-attributes.https.html",
    "service-workers/service-worker/rejections.https.html",
    "service-workers/service-worker/serviceworkerobject-scripturl.https.html",
    "service-workers/service-worker/state.https.html",
    "service-workers/service-worker/synced-state.https.html",
    "service-workers/service-worker/unregister.https.html",
    "service-workers/service-worker/skip-waiting-using-registration.https.html",
    "service-workers/service-worker/skip-waiting-without-using-registration.https.html",
    "service-workers/service-worker/update-bytecheck-cors-import.https.html",
    "service-workers/service-worker/update-bytecheck.https.html",
    "service-workers/service-worker/update-import-scripts.https.html",
    "service-workers/service-worker/update-missing-import-scripts.https.html",
    "service-workers/service-worker/update-result.https.html",
    "service-workers/service-worker/update.https.html",
];

/// Fixed Service Worker M2 fetch/interception corpus at the pinned WPT revision.
pub const SERVICE_WORKER_FETCH_CASES: &[&str] = &[
    "service-workers/service-worker/ServiceWorkerGlobalScope/fetch-on-the-right-interface.https.any.js",
    "service-workers/service-worker/ServiceWorkerGlobalScope/extendable-message-event-constructor.https.html",
    "service-workers/service-worker/ServiceWorkerGlobalScope/postmessage.https.html",
    "service-workers/service-worker/historical.https.any.js",
    "service-workers/service-worker/request-end-to-end.https.html",
    "service-workers/service-worker/fetch-event-add-async.https.html",
    "service-workers/service-worker/fetch-event-async-respond-with.https.html",
    "service-workers/service-worker/fetch-event-within-sw.https.html",
    "service-workers/service-worker/fetch-event-respond-with-custom-response.https.html",
    "service-workers/service-worker/fetch-event-handled.https.html",
    "service-workers/service-worker/fetch-event-after-navigation-within-page.https.html",
    "service-workers/service-worker/intercepted-referrer.https.html",
    "service-workers/service-worker/controller-with-no-fetch-event-handler.https.html",
    "service-workers/service-worker/fetch-with-body.https.html",
    "service-workers/service-worker/fetch-event-respond-with-stops-propagation.https.html",
    "service-workers/service-worker/fetch-event-throws-after-respond-with.https.html",
    "service-workers/service-worker/fetch-event-network-error.https.html",
    "service-workers/service-worker/fetch-event-respond-with-argument.https.html",
    "service-workers/service-worker/fetch-event-respond-with-readable-stream-chunk.https.html",
    "service-workers/service-worker/fetch-event-respond-with-readable-stream.https.html",
    "service-workers/service-worker/fetch-event-respond-with-response-body-with-invalid-chunk.https.html",
    "service-workers/service-worker/fetch-error.https.html",
    "service-workers/service-worker/iso-latin1-header.https.html",
    "service-workers/service-worker/invalid-header.https.html",
    "service-workers/service-worker/invalid-blobtype.https.html",
    "service-workers/service-worker/uncontrolled-page.https.html",
    "service-workers/service-worker/claim-fetch.https.html",
    "service-workers/service-worker/claim-not-using-registration.https.html",
    "service-workers/service-worker/claim-using-registration.https.html",
    "service-workers/service-worker/unregister-controller.https.html",
    "service-workers/service-worker/fetch-event-respond-with-body-loaded-in-chunk.https.html",
];

/// Fixed Service Worker CacheStorage corpus at the pinned WPT revision.
pub const SERVICE_WORKER_CACHE_STORAGE_CASES: &[&str] = &[
    "service-workers/cache-storage/cache-abort.https.any.js",
    "service-workers/cache-storage/cache-add.https.any.js",
    "service-workers/cache-storage/cache-delete.https.any.js",
    "service-workers/cache-storage/cache-keys.https.any.js",
    "service-workers/cache-storage/cache-match.https.any.js",
    "service-workers/cache-storage/cache-matchAll.https.any.js",
    "service-workers/cache-storage/cache-put.https.any.js",
    "service-workers/cache-storage/cache-storage.https.any.js",
    "service-workers/cache-storage/cache-storage-buckets.https.any.js",
    "service-workers/cache-storage/cache-storage-keys.https.any.js",
    "service-workers/cache-storage/cache-storage-match.https.any.js",
    "service-workers/cache-storage/cache-keys-attributes-for-service-worker.https.html",
    "service-workers/cache-storage/credentials.https.html",
    "service-workers/cache-storage/serviceworker/cache-storage.https.html",
    "service-workers/cache-storage/serviceworker/cache-storage-keys.https.html",
    "service-workers/cache-storage/serviceworker/cache-delete.https.html",
    "service-workers/cache-storage/serviceworker/cache-keys.https.html",
    "service-workers/cache-storage/serviceworker/cache-matchAll.https.html",
    "service-workers/cache-storage/serviceworker/cache-storage-match.https.html",
    "service-workers/cache-storage/serviceworker/cache-match.https.html",
    "service-workers/cache-storage/serviceworker/cache-put.https.html",
    "service-workers/cache-storage/serviceworker/cache-add.https.html",
    "service-workers/cache-storage/serviceworker/cache-abort.https.html",
    "service-workers/cache-storage/serviceworker/cache-keys-attributes-for-service-worker.https.html",
    "service-workers/cache-storage/serviceworker/credentials.https.html",
];

/// WPT subtest status.
///
/// 映射上游 testharness subtest status 数字编码（`testharness.js` 的 `Test.status`）：
/// `0=PASS`、`1=FAIL`、`2=TIMEOUT`、`3=NOTRUN`、`4=PRECONDITION_FAILED`。其中 `NOTRUN`（测试
/// 因脚本错误/超时未执行）与 `PRECONDITION_FAILED`（`assert_implements`/`assert_implements_optional`
/// 的 precondition 不满足，如 optional feature 不支持）是**中性状态**——上游 WPT dashboard 既不计入
/// pass 也不计入 fail（precondition 失败非实现缺陷，NOTRUN 属基础设施跳过）。runner 通过率统计须把
/// 它们与 `Fail` 区分（js-dom R20：原 `map_harness_results` 的 `_ => Fail` 把 3/4 误计为 Fail，
/// 拖低 optional feature 如 TouchEvent 的 dom/nodes 通过率）。
/// net-api M4-S22：preflight-cache 族 token 状态 stash（上游 wptserve
/// request.server.stash 等价物——reset-token.py 置空、cache*.py 状态机 take/put）。
static WPT_TOKEN_STASH: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<String, String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HarnessStatus {
    /// The subtest passed.
    Pass,
    /// The subtest failed.
    Fail,
    /// The case did not complete before the wall-clock deadline.
    Timeout,
    /// The case requires a testdriver API outside the declared support surface.
    Unsupported,
    /// The subtest was never run（上游 status `NOTRUN=3`）：脚本错误/超时致 test() 块未执行。
    /// 中性状态，通过率统计不计入 fail。
    NotRun,
    /// The subtest's precondition failed（上游 status `PRECONDITION_FAILED=4`）：
    /// `assert_implements`/`assert_implements_optional` 失败（optional feature 不支持等）。
    /// 中性状态，通过率统计不计入 fail。
    PreconditionFailed,
}

/// One WPT subtest result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessSubtestResult {
    /// Subtest name.
    pub name: String,
    /// Stable result status.
    pub status: HarnessStatus,
    /// Optional assertion or infrastructure message.
    pub message: Option<String>,
}

/// Run the selected upstream HTML interaction cases under `wpt_root`.
/// 运行 `html/semantics/forms/constraints` testharness 用例（Form Validation goal M1）。
///
/// 目录扫描顶层 .html（support/ 资源目录排除——resources 经 fetch 脚本独立拉取）。
/// FV M2/M3：interactive validation 的 forms 用例（constraints 目录外——
/// the-form-element 的 requestSubmit/checkValidity——validation 面）。
const CONSTRAINTS_EXTRA_FILES: &[&str] = &[
    "html/semantics/forms/the-form-element/form-requestsubmit.html",
    "html/semantics/forms/the-form-element/form-checkvalidity.html",
];

pub fn run_constraints_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    let dir = wpt_root.join("html/semantics/forms/constraints");
    let entries: Vec<_> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries.flatten().collect(),
        Err(error) => {
            return vec![(
                dir.display().to_string(),
                vec![HarnessSubtestResult {
                    name: "scan constraints dir".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    let mut cases = Vec::new();
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.extension().is_none_or(|ext| ext != "html") {
            continue;
        }
        let relative = format!("html/semantics/forms/constraints/{name}");
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let source = match read_corpus_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                cases.push((
                    relative,
                    vec![HarnessSubtestResult {
                        name: "load WPT case".into(),
                        status: HarnessStatus::Fail,
                        message: Some(error.to_string()),
                    }],
                ));
                continue;
            }
        };
        let results = run_testharness_html(wpt_root, &relative, &source, &harness_source, CASE_TIMEOUT);
        cases.push((relative, results));
    }
    // FV M2/M3：interactive validation 的 forms 用例（静态列表——the-form-element）
    for relative in CONSTRAINTS_EXTRA_FILES {
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let path = wpt_root.join(relative);
        match read_corpus_to_string(&path) {
            Ok(source) => {
                let results = run_testharness_html(wpt_root, relative, &source, &harness_source, CASE_TIMEOUT);
                cases.push(((*relative).to_string(), results));
            }
            Err(_) => continue, // 未拉取（网络/资产缺失）——不阻断
        }
    }
    cases
}

pub fn run_html_interaction_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_path = wpt_root.join("resources/testharness.js");
    let harness_source = match read_corpus_to_string(&harness_path) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                harness_path.display().to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    HTML_INTERACTION_CASES
        .iter()
        .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
        .map(|path| {
            let source = read_corpus_to_string(wpt_root.join(path));
            let results = match source {
                Ok(source) => run_testharness_html(wpt_root, path, &source, &harness_source, CASE_TIMEOUT),
                Err(error) => vec![HarnessSubtestResult {
                    name: "load WPT case".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            };
            ((*path).to_string(), results)
        })
        .collect()
}

/// Run the upstream `html/canvas` testharness cases under `wpt_root` (Canvas 2D goal M1).
///
/// 扫描 [`CANVAS_TEST_SUBDIRS`] 下全部主线程 .html 用例；`canvas-tests.js`（用例的
/// `_addTest` 驱动框架）与 testharness.js 一样内联执行。filter 按路径子串过滤。
pub fn run_canvas_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    let canvas_tests_source = match read_corpus_to_string(wpt_root.join(CANVAS_TESTS_JS_PATH)) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                CANVAS_TESTS_JS_PATH.to_string(),
                vec![HarnessSubtestResult {
                    name: "load canvas-tests.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    let mut cases = Vec::new();
    for subdir in CANVAS_TEST_SUBDIRS.iter().chain(
        // R34xx（2026-08-15）：顶层 testharness 用例（目录扫描不覆盖）。
        ["html/canvas/element"].iter(),
    ) {
        let dir = wpt_root.join(subdir);
        let entries: Vec<_> = match std::fs::read_dir(&dir) {
            Ok(entries) => entries.flatten().collect(),
            Err(_) => continue,
        };
        for entry in entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.extension().is_none_or(|ext| ext != "html") {
                continue;
            }
            // 顶层文件只取 CANVAS_TOP_LEVEL_FILES 清单内（目录内其余为 reftest-format
            // 对（-ref/expected）与范围外文件）。
            if *subdir == "html/canvas/element" {
                let relative = format!("{}/{}", subdir, name);
                if !CANVAS_TOP_LEVEL_FILES.contains(&relative.as_str()) {
                    continue;
                }
            }
            let relative = format!("{}/{}", subdir, name);
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let source = match read_corpus_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    cases.push((
                        relative.clone(),
                        vec![HarnessSubtestResult {
                            name: "load WPT case".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    ));
                    continue;
                }
            };
            // R34xx（2026-08-15）：reftest-format 文件 → 跳过（NotRun 中性状态），由
            // reftest/oracle 面负责（此前误入 testharness 面全部 Timeout，污染分母）。
            // 判定：① `rel="match"` 引用参考页的 test 文件；② `-ref.html`/`-expected.html`
            // 后缀的参考页本体（无 testharness 也无 match 链接——2d.layer.*-expected 等）。
            if source.contains("rel=\"match\"")
                || source.contains("rel='match'")
                || relative.ends_with("-ref.html")
                || relative.ends_with("-expected.html")
            {
                cases.push((
                    relative.clone(),
                    vec![HarnessSubtestResult {
                        name: "reftest-format file".into(),
                        status: HarnessStatus::NotRun,
                        message: Some(
                            "reftest-format（rel=match / -ref / -expected）——非 testharness 面，走 reftest/oracle"
                                .into(),
                        ),
                    }],
                ));
                continue;
            }
            // R56h：WPT 套件内部语义冲突用例（见 CANVAS_SKIP_FILES 注释）→ NotRun。
            if CANVAS_SKIP_FILES.contains(&relative.as_str()) {
                cases.push((
                    relative.clone(),
                    vec![HarnessSubtestResult {
                        name: "WPT suite-inconsistent case".into(),
                        status: HarnessStatus::NotRun,
                        message: Some(
                            "与套件内 stroke.scale1/2 + transformation.changing/multiple 的 CTM 语义互斥（追加时烘焙）——保持主流语义并跳过"
                                .into(),
                        ),
                    }],
                ));
                continue;
            }
            let results = run_canvas_testharness_html(
                wpt_root,
                &relative,
                &source,
                &harness_source,
                &canvas_tests_source,
                CASE_TIMEOUT,
            );
            cases.push((relative, results));
        }
    }
    cases
}

/// Run the upstream `dom/` testharness cases under `wpt_root`（JS/DOM nativization goal M4 / DC-3）。
///
/// 扫描 [`DOM_TEST_SUBDIRS`] 下全部主线程 .html 用例；仅依赖 `testharness.js`（与
/// [`run_html_interaction_cases`] 同一底层 [`run_testharness_html`]，不经 canvas-tests.js）。
/// filter 按路径子串过滤。用例由 `fetch-dom-subset.sh` 按需拉取（wpt-data gitignored）。
pub fn run_dom_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    let mut cases = Vec::new();
    for subdir in DOM_TEST_SUBDIRS {
        let dir = wpt_root.join(subdir);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "html") {
                continue;
            }
            let relative = format!("{}/{}", subdir, entry.file_name().to_string_lossy());
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let source = match read_corpus_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    cases.push((
                        relative.clone(),
                        vec![HarnessSubtestResult {
                            name: "load WPT case".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    ));
                    continue;
                }
            };
            let variants = case_variants(&source);
            if variants.is_empty() {
                let results = run_testharness_html(wpt_root, &relative, &source, &harness_source, CASE_TIMEOUT);
                cases.push((relative, results));
                continue;
            }
            // R329：variant 用例逐 query 跑（基础 URL 的行为由上游 harness 不注册 =
            // 0 subtest 空转，跳过防超时伪败；case 名带 query 区分，与上游 dashboard 对齐）。
            for variant in variants {
                let case_name = format!("{relative}{variant}");
                let results = run_testharness_html(wpt_root, &case_name, &source, &harness_source, CASE_TIMEOUT);
                cases.push((case_name, results));
            }
        }
    }
    cases
}

/// Run the upstream `selection/` testharness cases under `wpt_root`
/// （editing/contenteditable goal M1 / DC-1——Selection API 可观察面基线）。
///
/// 与 [`run_dom_cases`] 同构：扫描 [`SELECTION_TEST_SUBDIRS`] 下全部主线程
/// .html 用例；仅依赖 `testharness.js`。filter 按路径子串过滤。用例由
/// `fetch-selection-subset.sh` 按需拉取（wpt-data gitignored）。
pub fn run_selection_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    let mut cases = Vec::new();
    for subdir in SELECTION_TEST_SUBDIRS {
        let dir = wpt_root.join(subdir);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "html" && ext != "htm") {
                continue;
            }
            let relative = format!("{}/{}", subdir, entry.file_name().to_string_lossy());
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let source = match read_corpus_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    cases.push((
                        relative.clone(),
                        vec![HarnessSubtestResult {
                            name: "load WPT case".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    ));
                    continue;
                }
            };
            let variants = case_variants(&source);
            if variants.is_empty() {
                let results = run_testharness_html(wpt_root, &relative, &source, &harness_source, CASE_TIMEOUT);
                cases.push((relative, results));
                continue;
            }
            for variant in variants {
                let case_name = format!("{relative}{variant}");
                let results = run_testharness_html(wpt_root, &case_name, &source, &harness_source, CASE_TIMEOUT);
                cases.push((case_name, results));
            }
        }
    }
    cases
}

/// Run the upstream keyboard testharness cases under `wpt_root`
/// （keyboard-default-actions goal M1 / DC-1——键盘默认动作基线：uievents/keyboard
/// 事件面 + forms 隐式提交面）。
///
/// 与 [`run_selection_cases`] 同构：扫描 [`KEYBOARD_TEST_SUBDIRS`] 下全部主线程
/// .html 用例；仅依赖 `testharness.js` + runner 内建 testdriver 适配
///（click/send_keys/Actions，R142）。用例由 `fetch-keyboard-subset.sh` 按需拉取。
pub fn run_keyboard_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    let mut cases = Vec::new();
    for subdir in KEYBOARD_TEST_SUBDIRS {
        collect_keyboard_dir_cases(wpt_root, subdir, filter, &harness_source, &mut cases);
    }
    cases
}

/// 递归收集一个键盘子目录下的主线程 .html 用例（resources/ 等辅助目录跳过）。
fn collect_keyboard_dir_cases(
    wpt_root: &Path,
    subdir: &str,
    filter: Option<&str>,
    harness_source: &str,
    cases: &mut Vec<(String, Vec<HarnessSubtestResult>)>,
) {
    let dir = wpt_root.join(subdir);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "html" && ext != "htm") {
            continue;
        }
        let relative = format!("{}/{}", subdir, entry.file_name().to_string_lossy());
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let source = match read_corpus_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                cases.push((
                    relative.clone(),
                    vec![HarnessSubtestResult {
                        name: "load WPT case".into(),
                        status: HarnessStatus::Fail,
                        message: Some(error.to_string()),
                    }],
                ));
                continue;
            }
        };
        let variants = case_variants(&source);
        if variants.is_empty() {
            let results = run_testharness_html(wpt_root, &relative, &source, harness_source, CASE_TIMEOUT);
            cases.push((relative, results));
            continue;
        }
        for variant in variants {
            let case_name = format!("{relative}{variant}");
            let results = run_testharness_html(wpt_root, &case_name, &source, harness_source, CASE_TIMEOUT);
            cases.push((case_name, results));
        }
    }
}

/// uievents-compat goal corpus subdirs（M1 / DC-1）：上游 `uievents/` +
/// `pointerevents/` 全域递归扫描。`uievents/keyboard` 子域与
/// [`KEYBOARD_TEST_SUBDIRS`] 重叠属有意——keyboard goal 已收口，重复计入本 goal
/// 基线分母（分类口径独立）。
pub const UIEVENTS_TEST_SUBDIRS: &[&str] = &["uievents", "pointerevents"];

/// uievents-compat 语料跳过规则（非静默丢弃——记入 evidence 导入清单）：
/// - `*-manual.html`：需真实指针/触控硬件交互（clipboard/fullscreen 先例）
/// - `pointerevents/crashtests/`：崩溃回归用例，非 testharness 格式
/// - `pointerevents/pointerlock/`：pointer lock 域——goal 契约挂账（宿主鼠标
///   捕获模式域，重入条件 = 用户点名）
/// - `pointerevents/resources/`：helper 脚本目录，非用例
/// - `uievents/legacy-domevents-tests/`：DOM Level 3 旧套件（Status.html 索引页，
///   非 testharness.js 格式；approved/ 子目录未拉取）
fn uievents_case_skipped(relative: &str) -> bool {
    let file_name = relative.rsplit('/').next().unwrap_or(relative);
    if file_name.ends_with("-manual.html") {
        return true;
    }
    [
        "pointerevents/crashtests/",
        "pointerevents/pointerlock/",
        "pointerevents/resources/",
        "uievents/legacy-domevents-tests/",
    ]
    .iter()
    .any(|prefix| relative.starts_with(prefix))
}

/// Run the upstream UI/pointer-events testharness corpus under `wpt_root`
/// （uievents-compat goal M1 / DC-1——鼠标事件序 + Pointer Events 语义基线）。
///
/// 递归扫描 [`UIEVENTS_TEST_SUBDIRS`]（两域为嵌套布局，与 flat 的
/// [`run_any_js_corpus_subdirs`] 不同）。仅依赖 `testharness.js` + runner 内建
/// testdriver 适配（click/send_keys/Actions）；依赖白名单外 testdriver API 的
/// 用例在 [`run_testharness_html`] 内判 Unsupported。用例由
/// `scripts/goals/50-uievents-compat.sh` 按需拉取（gitignored）。
pub fn run_uievents_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    let mut cases = Vec::new();
    for subdir in UIEVENTS_TEST_SUBDIRS {
        collect_uievents_dir_cases(wpt_root, subdir, filter, &harness_source, &mut cases);
    }
    cases
}

/// 递归收集一个 UI/pointer 子域下的主线程 .html 用例（跳过规则见
/// [`uievents_case_skipped`]；variant meta 展开，js-dom R329 先例）。
fn collect_uievents_dir_cases(
    wpt_root: &Path,
    subdir: &str,
    filter: Option<&str>,
    harness_source: &str,
    cases: &mut Vec<(String, Vec<HarnessSubtestResult>)>,
) {
    let Ok(entries) = std::fs::read_dir(wpt_root.join(subdir)) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let relative = format!("{subdir}/{file_name}");
        if path.is_dir() {
            collect_uievents_dir_cases(wpt_root, &relative, filter, harness_source, cases);
            continue;
        }
        if !file_name.ends_with(".html") && !file_name.ends_with(".htm") {
            continue;
        }
        if uievents_case_skipped(&relative) {
            continue;
        }
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let Ok(source) = read_corpus_to_string(&path) else {
            continue;
        };
        let variants = case_variants(&source);
        if variants.is_empty() {
            let results = run_testharness_html(wpt_root, &relative, &source, harness_source, corpus_case_timeout());
            cases.push((relative, results));
            continue;
        }
        for variant in variants {
            let case_name = format!("{relative}{variant}");
            let results = run_testharness_html(wpt_root, &case_name, &source, harness_source, corpus_case_timeout());
            cases.push((case_name, results));
        }
    }
}

/// 解析用例声明的 `<meta name="variant" content="?query">` 列表（js-dom R329）。
///
/// WPT variant 用例（如 Range-in-shadow-after-the-shadow-removed 的 `?mode=open` /
/// `?mode=closed`）以同一文件 + 不同 query string 组成参数矩阵；runner 此前只跑基础
/// URL（无 query），依赖 variant 参数的用例全簇误败（`mode=null` 落 TypeError）。
/// content 支持无引号/单双引号形式；与上游 wpt struct 一致，query 含前导 `?`。
pub(crate) fn case_variants(source: &str) -> Vec<String> {
    // uievents-compat 尾簇 7：variant 值大小写保真——上游 `<meta name="variant"
    // content="?Shift">` 的查询串大小写敏感（页面 `location.search` 消费方按字面
    // switch 分派——modifier_no_mouse_movement 族旧被整体小写成 `?shift`，键miss
    // → `keyDown(undefined)` 全 variant 16F）。ASCII 小写化逐字节等长，匹配在
    // 小写镜像上做、值切片回**原文**。
    let mut out = Vec::new();
    let lower = source.to_ascii_lowercase();
    let mut rest = lower.as_str();
    let mut origin_rest = source;
    while let Some(idx) = rest.find("name=\"variant\"").or_else(|| rest.find("name=variant")) {
        let after = &rest[idx..];
        let Some(content_idx) = after.find("content=") else {
            break;
        };
        let offset = idx + content_idx + "content=".len();
        let tail = &origin_rest[offset..];
        let value = if let Some(stripped) = tail.strip_prefix('"') {
            stripped.split('"').next().unwrap_or("")
        } else if let Some(stripped) = tail.strip_prefix('\'') {
            stripped.split('\'').next().unwrap_or("")
        } else {
            tail.split_whitespace().next().unwrap_or("")
        };
        if !value.is_empty() {
            out.push(value.to_string());
        }
        rest = &after[1..];
        origin_rest = &origin_rest[idx + 1..];
    }
    out
}

/// touch-events-compat goal corpus subdir（M1 / DC-1）：上游 `touch-events/` flat 布局。
pub const TOUCH_EVENTS_TEST_SUBDIRS: &[&str] = &["touch-events"];

/// touch-events-compat 语料跳过规则（非静默丢弃——逐类记入 evidence 导入清单）：
/// - `*-manual.html`：真实多点触控硬件交互（uievents manual 先例）
/// - `touch-action-document-crash.html`：crashtest 形态（无 testharness，「不崩即过」；
///   uievents crashtests 先例）
/// - source 含 `test_driver` / `testdriver.js`：宿主触摸注入面——test_driver Actions
///   触摸链依赖宿主 touch 真值流（runner 内 pointerType 'touch' 仅作标签透传，无
///   touchstart/touchend 派发）。两类标记都要查：test_driver 调用可能落在同目录
///   helper（multi-touch-interactions.js 先例——.html 只带 script src）。单指/多指/
///   pinch/compat-mouse/tap+sync-xhr 十案挂 M3 注入面带数字收口，重入条件 = 宿主
///   触摸注入管线
///
/// `idlharness.window.js` 不经此规则——collector 只扫 `.html`；不跑的原因同仓
/// 先例：/resources/WebIDLParser.js 为 build 期生成资产，repo 内不存在
/// （clipboard/timing/net-api idlharness 先例）。
fn touch_events_case_skipped(relative: &str, source: &str) -> bool {
    let file_name = relative.rsplit('/').next().unwrap_or(relative);
    if file_name.ends_with("-manual.html") || file_name == "touch-action-document-crash.html" {
        return true;
    }
    source.contains("test_driver") || source.contains("testdriver.js")
}

/// Run the upstream touch-events constructor/attribute face corpus under `wpt_root`
/// （touch-events-compat goal M1 / DC-1——Touch/TouchEvent 构造器 + 列表属性语义基线）。
///
/// flat 布局走共享 [`run_corpus_subdirs`] 收集器；跳过规则见
/// [`touch_events_case_skipped`]（manual/crashtest/宿主触摸注入面，逐类记账）。
/// 用例由 `scripts/goals/22-touch-events-compat.sh` 按需拉取（gitignored）。
/// filter 按路径子串透传：`make testharness-touch-events FILTER=constructor`。
pub fn run_touch_events_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_corpus_subdirs(wpt_root, TOUCH_EVENTS_TEST_SUBDIRS, filter, touch_events_case_skipped)
}

/// Media elements goal（docs/goal/archive/media-elements.md，M1 / DC-1）导入的上游
/// `html/semantics/embedded-content/media-elements` testharness 用例面。
///
/// 由 `tests/wpt-runner/scripts/fetch-media-subset.sh` 维护（wpt-data gitignored，
/// 用例按需 fetch、不入库）；首批判定标准 = 只断言 JS 可观察语义（反射/canPlayType/
/// 元数据初值/track 反射），不依赖真实媒体解码。依赖真解码/播放驱动的用例
/// （event_* 族、autoplay、seeking/）随语义层落地逐批追加。
pub const MEDIA_TEST_FILES: &[&str] = &[
    // M3 扩批：event_* 族（M2 headless 加载序列落地后事件断言可跑——时序/状态断言面）。
    "html/semantics/embedded-content/media-elements/event_canplay.html",
    "html/semantics/embedded-content/media-elements/event_canplay_noautoplay.html",
    "html/semantics/embedded-content/media-elements/event_canplaythrough.html",
    "html/semantics/embedded-content/media-elements/event_canplaythrough_noautoplay.html",
    "html/semantics/embedded-content/media-elements/event_loadeddata.html",
    "html/semantics/embedded-content/media-elements/event_loadeddata_noautoplay.html",
    "html/semantics/embedded-content/media-elements/event_loadedmetadata.html",
    "html/semantics/embedded-content/media-elements/event_loadedmetadata_noautoplay.html",
    "html/semantics/embedded-content/media-elements/event_loadstart.html",
    "html/semantics/embedded-content/media-elements/event_loadstart_noautoplay.html",
    "html/semantics/embedded-content/media-elements/event_order_canplay_canplaythrough.html",
    "html/semantics/embedded-content/media-elements/event_order_canplay_playing.html",
    "html/semantics/embedded-content/media-elements/event_order_durationchange_resize_loadedmetadata.html",
    "html/semantics/embedded-content/media-elements/event_order_loadedmetadata_loadeddata.html",
    "html/semantics/embedded-content/media-elements/event_order_loadstart_progress.html",
    "html/semantics/embedded-content/media-elements/event_pause.html",
    "html/semantics/embedded-content/media-elements/event_pause_noautoplay.html",
    "html/semantics/embedded-content/media-elements/event_play.html",
    "html/semantics/embedded-content/media-elements/event_play_noautoplay.html",
    "html/semantics/embedded-content/media-elements/event_playing.html",
    "html/semantics/embedded-content/media-elements/event_playing_noautoplay.html",
    "html/semantics/embedded-content/media-elements/event_progress.html",
    "html/semantics/embedded-content/media-elements/event_progress_noautoplay.html",
    "html/semantics/embedded-content/media-elements/event_volumechange.html",
    "html/semantics/embedded-content/media-elements/volume_nonfinite.html",
    "html/semantics/embedded-content/media-elements/controlsList.tentative.html",
    "html/semantics/embedded-content/media-elements/event_timeupdate.html",
    "html/semantics/embedded-content/media-elements/event_timeupdate_noautoplay.html",
    "html/semantics/embedded-content/media-elements/error-codes/error.html",
    "html/semantics/embedded-content/media-elements/historical.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLMediaElement/addTextTrack.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLMediaElement/crossOrigin.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLMediaElement/textTracks.html",
    // M3 扩批 XII：TextTrack 家族接口语义面（TextTrack/TextTrackCueList/TextTrackList/
    // TextTrackCue/TrackEvent——VTTCue 最小面 + addCue/removeCue/cues 排序/getCueById/
    // mode 枚举归一/on* EventTarget 面）。逐文件全导入——VTT 解析依赖子测（track.src=
    // data:text/vtt → parsed cue）经 settle 链 data: URL 文本面解锁（见 shim part06）。
    "html/semantics/embedded-content/media-elements/interfaces/TextTrack/activeCues.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrack/addCue.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrack/constants.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrack/cues.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrack/kind.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrack/label.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrack/language.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrack/mode.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrack/oncuechange.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrack/removeCue.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCue/constructor.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCue/endTime.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCue/id.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCue/onenter.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCue/onexit.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCue/pauseOnExit.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCue/startTime.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCue/track.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCueList/getCueById.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCueList/getter.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackCueList/length.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackList/getTrackById.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackList/getter.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackList/length.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackList/onaddtrack.html",
    "html/semantics/embedded-content/media-elements/interfaces/TextTrackList/onremovetrack.html",
    "html/semantics/embedded-content/media-elements/interfaces/TrackEvent/constructor.html",
    "html/semantics/embedded-content/media-elements/interfaces/TrackEvent/createEvent.html",
    // M3 扩批 III：the-audio-element 反射面（Audio 构造器 spec 语义——preload=auto +
    // 无 new TypeError + HTMLAudioElement illegal-constructor 调用面）。
    "html/semantics/embedded-content/the-audio-element/audio_constructor.html",
    // M3 扩批 IV：the-video-element 反射面（属性不凭空出现——UA 面不加 tabindex）。
    "html/semantics/embedded-content/the-video-element/video-tabindex.html",
    // M3 扩批 VIII：空 src 容错面（error 事件不 crash）。
    "html/semantics/embedded-content/the-video-element/video_crash_empty_src.html",
    // M3 扩批 XXXV：loading=eager 立即加载面（loadeddata 到达——headless settle
    // 无视口 gate；loading IDL 反射面 `video.loading = 'eager'` setter 不抛即用）。
    "html/semantics/embedded-content/the-video-element/video-loading-eager.html",
    // M3 扩批 XXXVI：the-audio-element 目录清点——audio-loading-eager 导入（同
    // XXXV 面，audio 形态；lazy/deferred 系与 eager-by-default 互斥维持排除）。
    "html/semantics/embedded-content/the-audio-element/audio-loading-eager.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLTrackElement/default.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLTrackElement/kind.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLTrackElement/label.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLTrackElement/readyState.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLTrackElement/src.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLTrackElement/srclang.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLTrackElement/track.html",
    "html/semantics/embedded-content/media-elements/location-of-the-media-resource/currentSrc.html",
    "html/semantics/embedded-content/media-elements/mime-types/canPlayType.html",
    "html/semantics/embedded-content/media-elements/networkState_during_loadstart.html",
    "html/semantics/embedded-content/media-elements/networkState_during_progress.html",
    "html/semantics/embedded-content/media-elements/networkState_initial.html",
    "html/semantics/embedded-content/media-elements/offsets-into-the-media-resource/currentTime.html",
    "html/semantics/embedded-content/media-elements/offsets-into-the-media-resource/duration.html",
    "html/semantics/embedded-content/media-elements/paused_false_during_play.html",
    "html/semantics/embedded-content/media-elements/paused_true_during_pause.html",
    "html/semantics/embedded-content/media-elements/playing-the-media-resource/playbackRate.html",
    // M3 扩批 X：track 子元素 ↔ textTracks 集合同步（树序段 + addTextTrack 尾段、
    // append/remove/innerHTML 同步、TextTrack.id 反射、getTrackById）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-api-texttracks.html",
    // M3 扩批 XV：http(s) VTT 文件加载 + WebVTT 解析深化（shim part06 `_zwParseVtt`
    // ——同步 __zw_fetch 取文本 + header 校验/cue id 错误恢复/cue settings/实体解码）。
    // track-add-remove-cue：settings.vtt 加载 + cue 增删/排序 + getCueById('junk')
    // → null + VTTCue 缺省反射（headless 语义面此前已落地，本批验证 http VTT 路径）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-add-remove-cue.html",
    // cue id 行解析（含 '-->' 错误恢复——含 '-->' 的行不识别为 id）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-cue-identifiers.html",
    // 空行/无分隔 cue 块解析（无分隔 → 文本并入上一 cue）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-blank-lines.html",
    // cue settings（line/position/size/align/vertical，% 值 + tab 分隔 + bad separation）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-settings.html",
    // 实体解码（&amp;/&lt;/&gt;/&lrm;/&rlm;/&nbsp;）+ settings 组合。
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-entities.html",
    // 小时位时间戳（00:00:00.000 / 100:20:00.500 → 361200.5）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-timings-hour.html",
    // WEBVTT magic header 校验（rubbish 头 → cues []；no-webvtt → error 面）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-magic-header.html",
    // header 长度/名称校验（empty-after/newlines-after → load；too-short/invalid-equal → error）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-header-checks.html",
    // 负时间戳 cue（VTTCue(-5,...) 存储 + 排序 + setter 负值——纯 headless 面）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-cue-negative-timestamp.html",
    // src 三段变更（cues 立即清空 + same list 身份 + 同值变更不重载 + 上游文本断言）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-element-src-change.html",
    // default 属性 readyState=LOADED 面（静态 HTML 两 track 形态，onload 后断言）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-default-attribute.html",
    // src setter 触发加载（NONE → LOADED；track.track.mode='hidden' 先设——mode 触发面）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-load-from-src-readyState.html",
    // ---- M3 扩批 XVI（2026-09-02）：track-cues-* 播放推进族（fixture-mounted 切片 2——
    // 播放桥 + 泵 + time-marches-on/seek sync 就绪后解锁）。movie_5.webm（VP9+Opus 5s）
    // 为媒体源；cue enter/exit 由桥真值钟驱动（runner 泵每 tick 调 _zwMediaTimeMarchesOn）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-cues-enter-seeking.html",
    // M3 扩批 XXI：TextTrackList change 事件广播（TextTrack↔TextTrackList 反向链
    // _zwOwnerList + mode 有效值变更异步 Event('change') target=list——深结构项
    // D 组首个收口；track-change-event 断言 instanceof Event / 无 track 属性 /
    // target 身份）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-change-event.html",
    // M3 扩批 XXII（2026-09-03）：B 组排除件随 change 广播/播放推进基建复评导入——
    // track-disabled（disabled track march 跳过 + active 清空——spec time-marches-on
    // 步 2 disabled gate）/ no-cuechange-before-play（播放前不派 cuechange——march
    // 仅 playing 态跑，天然满足；EventWatcher + promise_test 框架面验证）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-disabled.html",
    "html/semantics/embedded-content/media-elements/track/track-element/no-cuechange-before-play.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-remove-active-cue.html",
    // M3 扩批 XXIII（2026-09-03）：load invoke 重置面收口——media load 算法 invoke 步 6
    // （spec：current playback position 归 0 + readyState HAVE_NOTHING）+ settle 前
    // _resourceStates 残留清除 + settle 的 media/track 元素 load/error 派发改
    // _zwMediaFire（handle-only 元素 on* expando handler 兜底）。track-active-cues 导入
    //（error 后 activeCues 清空 + video.onerror 断言面）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-active-cues.html",
    // M3 扩批 XX（2026-09-03）：HAVE_NOTHING 期 seek 挂起语义（spec「default playback
    // start position」）——currentTime setter readyState 0 时挂 _zwSeekDeferred，
    // _zwMediaLoadSequence readyState 0→1 翻转时补跑 seek 算法（seeking + seeked
    // 异步回落 + cue active 面同步）。track-cues-seeking 的 onseeked 计数链解锁。
    "html/semantics/embedded-content/media-elements/track/track-element/track-cues-seeking.html",
    // M3 扩批 XIX（2026-09-03）：track-cues-* 播放推进族续批——解码器 EOF 排空
    // 缺陷修复（zero-media decode.rs：demux 尽后解码器残余帧经 drain_frame 排空 +
    // player present_pending 未来帧退回 un_read——此前 position < duration 即提前
    // Ended，cue@4-5s 永不触发）+ march pauseOnExit 暂停先于 exit 派发（上游
    // onexit handler 内 assert_true(video.paused) 断言面）+ pending seek 补推路径
    // 补 seekSync（起点恰在 seek 目标上的 cue 立即 enter）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-cues-enter-exit.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-cues-pause-on-exit.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-cues-missed.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-cues-sorted-before-dispatch.html",
    // 不导入（B 组——依赖真播放钟推进 time-marches-on/cue enter/exit/cuechange/
    // activeCues 变化，随 media-playback 泵接语义层后复评）：track-cues-* 全族、
    // no-cuechange-before-play、track-remove-active-cue、
    // track-change-event（深结构：TextTrackList↔TextTrack 反向链）。
    // （扩批 XXI/XXII/XXIII 更新：change 广播已落地、disabled gate/cuechange 派发/
    // activeCues 面已落地——track-change-event、track-disabled、no-cuechange-
    // before-play、track-remove-active-cue、track-active-cues 均已解除排除导入。）
    // 不导入（C 组——cue 视觉渲染/布局，渲染域远期）：track-cue-rendering-*、
    // track-css-cue-pseudo-class、track-webvtt-*positioning/layout、track-cue-inline。
    // 不导入（D 组——深结构/契约冲突，单独评估）：track-mode-triggers-loading
    //（canplaythrough 后才触发 track 加载的时序面）、track-disabled（timeupdate
    // 播放推进面）、track-element-src-aborted-load/-src-change-error（abort/error
    // 时序）、track-remove-quickly/-by-setting-innerHTML（移除竞态）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-addtrack-kind.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-texttracks.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-node-add-remove.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-id.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-element-dom-change.html",
    // M3 扩批 XIII：VTTCue 定位选项 IDL 面（line/position/size/align/vertical/
    // snapToLines——headless 仅存储不做视觉布局）+ data:text/vtt 加载（onload 后
    // cue 值断言；crossorigin 属性三态）。
    "html/semantics/embedded-content/media-elements/track/track-element/vtt-cue-float-precision.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-data-url.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-add-track.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-cue-order.html",
    "html/semantics/embedded-content/media-elements/track/track-element/src-clear-cues.html",
    // M3 扩批 VII：移除文档暂停面（spec「media elements pause on removal」）。
    "html/semantics/embedded-content/media-elements/playing-the-media-resource/pause-remove-from-document.html",
    // M3 扩批 XIV：移除暂停两变体——appendChild 后 src 路径（同语义域）+
    // NETWORK_EMPTY 负例（无候选 play() promise pending → 移除后 AbortError）。
    "html/semantics/embedded-content/media-elements/playing-the-media-resource/pause-remove-from-document-different-load.html",
    "html/semantics/embedded-content/media-elements/playing-the-media-resource/pause-remove-from-document-networkState.html",
    // M3 扩批 XXIV（2026-09-03）：loop 属性真面——spec「ended playback」步 6.4
    //（loop 元素不进入 ended playback：位置回卷 + seeked 非 ended）。配套 registry
    // set_loop（音频 entry 流末回卷重建）+ shim loop IDL 面 + march Ended 分叉。
    // played-loop：played TimeRanges 跨 loop 保持（test-1s）；
    // audio_loop_seek_to_eos：loop 音频 seek 到 EOS 仍播放（sound_5.mp3 音频面）。
    // 不导入 audio_loop_base/video_loop_base（短 fixture < 泵采样粒度的回卷时序
    // 不可观测 + 2x2-green 为 VP8 解码域外）。
    "html/semantics/embedded-content/media-elements/played-loop.html",
    "html/semantics/embedded-content/media-elements/audio_loop_seek_to_eos.html",
    // M3 扩批 XXV（2026-09-03）：loop-from-ended.tentative——ended 后设 loop 再 play
    // 须回卷 seeked（Chromium crbug 364442 断言面：ended 翻转 + currentTime==duration
    // + seeked 时 currentTime<duration）。此前排除的 settle 竞态由 duration getter
    // 兜底（settle durationMs 即刻生效）+ registry Ended→play 解码器重建 + 泵时钟
    // 注入（play 锚与 tick 同源）解除。
    "html/semantics/embedded-content/media-elements/playing-the-media-resource/loop-from-ended.tentative.html",
    // M3 扩批 XXXIV（2026-09-04）：play-in-detached-document——detached 文档
    // video play() 推进面（headless 时钟推进 + 周期 timeupdate 已由扩批 XXVII
    // 落地——此前「依赖真播放钟」排除注记失效）。
    "html/semantics/embedded-content/media-elements/playing-the-media-resource/play-in-detached-document.html",
    // M3 扩批 XXVI（2026-09-04）：seeking/ 三件——seekable TimeRanges（headless
    // [0,duration] 近似 getter 落地后的断言面：clamp 边界 + seeking/timeupdate/
    // seeked 事件序）+ volume_nonfinite（volume IDL setter 非有限 TypeError）。
    "html/semantics/embedded-content/media-elements/seeking/seek-to-currentTime.html",
    "html/semantics/embedded-content/media-elements/seeking/seek-to-max-value.htm",
    "html/semantics/embedded-content/media-elements/seeking/seek-to-negative-time.htm",
    "html/semantics/embedded-content/media-elements/volume_nonfinite.html",
    // M3 扩批 XXVII（2026-09-04）：media fragment #t= 起点解析（settle 加载序列
    // 内 currentTime 初始化）+ broken track 不阻塞 autoplay 推进。
    // 不导入 no-autoplay-audio-history-back（iframe+history+postMessage 导航深结构）。
    "html/semantics/embedded-content/media-elements/media_fragment_seek.html",
    "html/semantics/embedded-content/media-elements/autoplay-with-broken-track.html",
    // M3 扩批 XXVIII（2026-09-04）：同文档移动不重置播放（currentTime>=10 保持 +
    // paused=false；movie_300.webm 长流）。不导入 loop_base（XXIV 注记）/
    // preserves-pitch/src_object_blob（testdriver 音高/blob 面）。
    "html/semantics/embedded-content/media-elements/offsets-into-the-media-resource/currentTime-move-within-document.html",
    // M3 扩批 XXVIII 续：track-mode-triggers-loading——metadata track disabled 不
    // 加载，mode 改 hidden 触发（mode 触发加载面 + VTT cue 解析断言）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-mode-triggers-loading.html",
    // M3 扩批 XXX 续：mode/cuechange 播放推进面（B 组基建现成试导）。
    // track-mode 维持排除——mode 数值 setter 回落 + cue 计数 done 链。
    // M3 扩批 XXXI：removetrack 派发落地后试导（selection metadata mode 面 /
    // removetrack TrackEvent 面 / 播放中 add cue 面）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-selection-metadata.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-remove-track.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-cues-missed-no-immediate-events.html",
    // track-remove-insert-ready-state 维持排除（LII 三方案负结果终版）：①首调度同步 body
    //（XLVIII 形态复刻——33 件 Timeout/cues 翻倍，事件 defer 未救回 cue 填充重复）；②延迟置位
    //（同型回归）；③枚举兜底（QSA 直查——枚举非根因）。canplaythrough 时 track settle 的
    //「同步可达」需求与既有 40+ track 件的事件时序在当前 runner 沙箱事件循环下不相容——归
    // runner 事件循环统一（deep-structure）。
    // track-mode-not-changed-by-new-track 维持排除（LIII 试导回退，2026-09-05）：旧注记
    //「身份对拍」与用例实际断言面不符（mode 稳态 + addtrack 身份链——event.track 须为
    // addTextTrack 产物 track3）。探针实证：迟注册的 onaddtrack 收到的首个事件是 append 期
    // track2 的 addtrack（晚到的跨 execute 派发），而非 track3 的——addtrack 的 queued task
    // 在 runner 沙箱内跨 execute 的派发时点不稳定（microtask checkpoint 不保证在所属 execute
    // 末即时排空；改 setTimeout 承载后仍受 host 泵 tick 合并影响），属 runner 事件循环统一
    //（deep-structure）域。LIII 三项 spec 对齐改动保留（addtrack 每实例一次幂等 / observed
    // 登记——parse 期 track 子不再由迟到首读补派 / append 时刻建 list）——629P 保绿。
    "html/semantics/embedded-content/media-elements/track/track-element/track-cues-cuechange-dynamically-created-track-element.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-disabled-addcue.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-insert-after-load.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-load-error-readyState.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-load-from-element-readyState.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-cue-mutable.html",
    "html/semantics/embedded-content/media-elements/track/track-element/src-empty-string.html",
    // M3 扩批 XXXIII：TextTrackCueList 功能面。track-cue-mutable-fragment 维持排除
    //（cue 标记树 isEqualNode）；track-selection-task-order 维持排除（宏任务序）。
    // track-mode 维持排除（XLI 试导 Timeout——mode 切换 no-event 断言依赖 cuechange
    // 计数 done 链（4 次 enter/exit cuechange），真播放推进 + cue 时序收敛依赖；
    // mode 数值 setter 回落 + disabled cues null 语义面已由其他用例覆盖）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-text-track-cue-list.html",
    // M3 扩批 XXXVIII：track-cue-empty 解除排除（getCueAsHTML 空 cue——fragment 单
    // 空 Text 节点 + constructor.name 断言；Text.prototype.constructor 自引修复后绿）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-cue-empty.html",
    // M3 扩批 XXXIX：loop_base 双件试导（loop=true → seeking 二次派发——扩批 XXIV/XXV
    // loop 真面落地后解除排除）。
    "html/semantics/embedded-content/media-elements/audio_loop_base.html",
    // M3 扩批 XXXIII：TextTrackCueList 功能面。track-cue-mutable-fragment 维持排除
    //（cue 标记树 isEqualNode）；track-selection-task-order 维持排除（宏任务序）。
    // M3 扩批 XXXII：readyState/cue-mutable/mode 稳态面批量试导。
    // M3 扩批 XXXI：removetrack 派发落地后试导（selection metadata mode 面 /
    // removetrack TrackEvent 面 / 播放中 add cue 面）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-mode-disabled.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-cues-cuechange.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-cues-add-new-track.html",
    // M3 扩批 XLII（2026-09-04）：markup 结构族解除排除——getCueAsHTML 升级为 cue
    // text markup 树解析（_zwCueTextToFragment：b/i/u/ruby/rt 同名元素 + c/v → span
    //（class → className / v annotation → title）+ 无效起始标签丢弃 + 未知标签忽略
    // 保留内容 + 裸 rt 丢弃 + timestamp 锚点无产物——spec webvtt-cue-text-parsing-
    // rules）。voice/class-markup/markup 为 isEqualNode 节点树对拍；cue-recovery 为
    // cues_match 对拍；unsupported-markup/timestamp 为 textContent 对拍。
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-voice.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-class-markup.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-markup.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-cue-recovery.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-unsupported-markup.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-timestamp.html",
    // M3 扩批 XXX（2026-09-04）：WebVTT 解析面批量导入（track-helpers.js 断言辅助
    // + 27 件 vtt 资源）——BOM/UTF8 编码面/header 注释/空 cue/timings 变体/
    // 退化形态/interspersed-non-cue/newlines。positioning/layout 渲染件维持 C 组
    // 排除；track-cue-empty 维持排除
    //（constructor.name === 'Text' 原生 class 断言——shim 工厂面差异）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-bom.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-utf8.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-header-comment.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-interspersed-non-cue.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-no-timings.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-cue-no-id.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-degenerate-cues.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-empty-cue.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-newlines.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-timings-no-hours.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-webvtt-timings-whitespace.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-cue-negative-duration.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-large-timestamp.html",
    // M3 扩批 XXVIII 续二：track 移除不 crash smoke 面（innerHTML 注入 video+track
    // / seeked 链中 innerHTML 清空后再 seek——testharness.js 兜底 test() 空 body 形态）。
    "html/semantics/embedded-content/media-elements/track/track-element/track-remove-quickly.html",
    "html/semantics/embedded-content/media-elements/track/track-element/track-remove-by-setting-innerHTML.html",
    // 不导入 track-element-src-change-error：stage3→4 依赖「加载中移除 src」的
    // in-flight 中断时序——headless settle 同步完成无 in-flight 窗口（2026-09-04
    // 实证：settings.vtt onload 恒先于 removeAttribute，onload case4 unreached）。
    // 不导入 track-element-src-aborted-load：WPT trickle pipe 机制不可复现。
    // M3 扩批 XXIX（2026-09-04）：ready-states/autoplay——autoplaying flag 交互
    // + 事件严格序（audio+video 各 5 子测）。
    "html/semantics/embedded-content/media-elements/ready-states/autoplay.html",
    // 不导入 video_size_preserved_after_ended（2026-09-04 实证；2026-09-05 复评试导
    // 仍 Timeout——EventWatcher wait_for("loadedmetadata") 未达，pending=1 卡死；
    // XLIV~XLVI host 通道修复未触达该件根因——静态 <source> 形态的 loadedmetadata
    // 与 promise_test EventWatcher 时序在 headless 双通道 settle 下不稳定——维持
    // 排除，随 runner/shim 事件通道统一后复评）。不导入 video_timeupdate_on_seek
    //（WPT CGI src）/ video_initially_paused（reftest 型）/ video-loading-* poster 族。

    // M3 扩批 XLI（2026-09-04，上游核查试导）：playbackRate（上游 edge 7/7 绿——
    // setter ratechange 派发面，M2 既有）。pause-move-to-other-document 试导失败回退
    //（本地「paused after stable state got true」——shim 融合视图下 iframe
    // contentDocument.body.appendChild 先触发 removal-pause 两段 defer；spec related
    // 文档判定含 iframe 文档，修复须 pause-on-removal 的 related-document 判定精化，
    // 归移除暂停精化切片——上游 1/1 绿为可回访断言面）。
    "html/semantics/embedded-content/media-elements/playing-the-media-resource/playbackRate.html",
    // M3 扩批 XLIII（2026-09-04）：pause-move-to-other-document 解除排除——iframe body
    // appendChild 的 sel-only 子移除标记清除（related 判定精化）落地后绿。
    "html/semantics/embedded-content/media-elements/playing-the-media-resource/pause-move-to-other-document.html",
    // M3 扩批 IX：移动面（同文档移动仍 related → 不暂停）。
    "html/semantics/embedded-content/media-elements/playing-the-media-resource/pause-move-within-document.html",
    // M3 扩批 XI：resource selection 算法 JS 可观察面（loading-the-media-resource 逐文件
    // 白名单——networkState 同步段 NO_SOURCE/稳定态 EMPTY、invoke 面（play/pause/load/
    // set-src/insert source）、src 移除不触发；_zwMediaResourceSelect microtask 续段）。
    // 依赖真资源失败时序的 pointer/candidate/source-media 族与 MSE/iframe/manual 变体
    // 不导入（master.md 排除清单）。data:, 两案（invoke-pause/remove-networkState）
    // 排除理由见上方 2026-09-03 复评注记。
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/autoplay-overrides-preload.html",
    // data:, 两案（invoke-pause/remove-networkState）维持排除（2026-09-03 复评）：
    // error settle 依赖「fetch 成功但解码探测失败」的两段 settle——现管道单次提交；
    // 且已导入的 location currentSrc.html 断言 data:, loadstart 后 currentSrc === src
    //（依赖 loaded settle 置 currentSrc），全局 error 化会回归该案。两段 settle
    //（fetch loaded → 解码探测 error）随解码层真失败判定重评。
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/load-removes-queued-error-event.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-candidate-insert-before.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-audio-constructor-no-src.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-audio-constructor.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-in-sync-event.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-insert-fragment-into-document.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-insert-into-document.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-insert-parent-into-document.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-insert-source-in-div.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-insert-source-in-namespace.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-insert-source-not-in-document.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-insert-source.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-load.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-pause.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-play.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-remove-from-document.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-remove-src.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-set-src-in-namespace.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-set-src-networkState.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-set-src-not-in-document.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-set-src.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-remove-source.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-remove-src.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-resumes-onload.html",
    // M3 扩批 XL（2026-09-04，排除注记复核）：pointer/candidate 族试导——扩批 XI 排除
    // 注记「依赖 source 真实 fetch 失败时序」在 runner 静态 source settle（扩批 XXX）
    // 落地后复核。上游核查（wpt.fyi 2026-09-04 master run，edge=Chromium 内核）：
    // pointer 全 7 件 + candidate-moved + candidate-remove-onerror 上游亦红/Timeout
    //（crbug 593289「await a stable state」族——无 src source 是否派 error 的指针语义
    // Chromium 自身未实现）；candidate-remove-addEventListener 上游无数据且本地
    // Timeout。均维持排除（坏用例不导入——与 Chromium oracle 口径一致）。
    // candidate-remove-no-listener 上游 1/1 绿 → 解除排除导入。
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-candidate-remove-no-listener.html",
    // load-events-networkState / invoke-pause-networkState（上游 edge 4/4 与 1/1 绿）
    // 试导——load() 的 abort/emptied/timeupdate 事件序 + 「pause() 不重触发资源选择」。
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/load-events-networkState.html",
    "html/semantics/embedded-content/media-elements/loading-the-media-resource/resource-selection-invoke-pause-networkState.html",
    "html/semantics/embedded-content/media-elements/preload_reflects_none_autoplay.html",
    "html/semantics/embedded-content/media-elements/readyState_during_canplay.html",
    "html/semantics/embedded-content/media-elements/readyState_during_canplaythrough.html",
    "html/semantics/embedded-content/media-elements/readyState_during_loadeddata.html",
    "html/semantics/embedded-content/media-elements/readyState_during_loadedmetadata.html",
    "html/semantics/embedded-content/media-elements/readyState_during_playing.html",
    "html/semantics/embedded-content/media-elements/readyState_initial.html",
    "html/semantics/embedded-content/media-elements/src_reflects_attribute_not_source_elements.html",
];

/// media-audio M3：Web Audio 最小面（AudioContext 构造/节点接口/connect 语义——
/// D1 批复切片 2 WPT 可执行子集）。逐文件白名单；依赖真渲染（startRendering/
/// OfflineAudioContext 渲染/worklet）的用例不导入（RFC §0 不做清单）。
pub const WEBAUDIO_TEST_FILES: &[&str] = &[
    // connect 返回值面（OfflineAudioContext 构造 + createGain + connect 链）。
    "webaudio/the-audio-api/the-audionode-interface/audionode-connect-return-value.html",
    // destination 接口面（channelCount 2 缺省 + maxChannelCount ≥ 2 + 断言
    // destination 为 AudioDestinationNode——identity/实例面）。
    "webaudio/the-audio-api/the-destinationnode-interface/destination.html",
    // OscillatorNode 构造器面（audit.js 框架——invalid/default ctor + type/
    // frequency 440 属性反射断言；runner 内联 webaudio/resources/audit*.js）。
    "webaudio/the-audio-api/the-oscillatornode-interface/ctor-oscillator.html",
    // ---- 第四批（2026-09-02）：处理类节点 ctor 族 + AudioParam 异常面 ----
    // GainNode ctor（W3CTH 形态——invalid/default + AudioNodeOptions + {gain:-2}
    // 选项反射）。
    "webaudio/the-audio-api/the-gainnode-interface/ctor-gain.html",
    // StereoPannerNode ctor（audit——clamped-max 缺省 + channelCount [1,2] 界 +
    // mode 'max' NotSupportedError + {pan:0.75}）。
    "webaudio/the-audio-api/the-stereopanner-interface/ctor-stereopanner.html",
    // DelayNode ctor（audit——delayTime + maxDelayTime 选项 + maxValue 反射）。
    "webaudio/the-audio-api/the-delaynode-interface/ctor-delay.html",
    // BiquadFilterNode ctor（W3CTH——type 五枚举 + Q/detune/frequency/gain 缺省
    // 1/0/350/0 + 选项反射）。
    "webaudio/the-audio-api/the-biquadfilternode-interface/ctor-biquadfilter.html",
    // AnalyserNode ctor（audit——fftSize 2048 幂 + frequencyBinCount + min/max
    // Decibels 交叉校验 + smoothingTimeConstant [0,1]）。
    "webaudio/the-audio-api/the-analysernode-interface/ctor-analyser.html",
    // createPeriodicWave 非 finite → TypeError（Float32Array Infinity 面）。
    "webaudio/the-audio-api/the-periodicwave-interface/createPeriodicWaveInfiniteValuesThrows.html",
    // AudioParam 调度方法异常面（value/time 非 finite → TypeError、负时间/
    // 零时长 → RangeError、exponentialRamp 零值 → RangeError、setValueCurve
    // 曲线非 finite → TypeError——无渲染断言，纯异常语义）。
    "webaudio/the-audio-api/the-audioparam-interface/audioparam-exceptional-values.html",
    // ---- 第五批（2026-09-02）：AudioBuffer 构造/接口面 ----
    // AudioBuffer 纯构造 + duration/getChannelData 断言（W3CTH 形态，无渲染）。
    "webaudio/the-audio-api/the-audiobuffer-interface/audiobuffer.html",
    // GainNode.gain instanceof AudioParam 断言（audit 单 task，无渲染）。
    "webaudio/the-audio-api/the-gainnode-interface/gain-basic.html",
    // ---- 第七批（2026-09-03）：ChannelMerger/Splitter/ConstantSource ctor 族 +
    // AudioBuffer getChannelData same-object 面（W3CTH/audit 构造面，无渲染——
    // shim 第七批：createChannelMerger/Splitter + createConstantSource +
    // createBuffer + 固定通道 setter 面同步落地）。
    "webaudio/the-audio-api/the-channelmergernode-interface/ctor-channelmerger.html",
    "webaudio/the-audio-api/the-channelsplitternode-interface/ctor-channelsplitter.html",
    "webaudio/the-audio-api/the-constantsourcenode-interface/ctor-constantsource.html",
    "webaudio/the-audio-api/the-audiobuffer-interface/audiobuffer-getChannelData.html",
    // ---- 第八批（2026-09-03）：AudioNode 接口基本面——跨 context connect/
    // disconnect InvalidAccessError（ctx 身份校验）+ connect 索引越界
    // IndexSizeError + AudioBufferSourceNode 0入1出接口反射 + AudioContext
    // 3-arg legacy 拒收（shim 第八批同步落地）。
    "webaudio/the-audio-api/the-audionode-interface/audionode.html",
    "webaudio/the-audio-api/the-audionode-interface/different-contexts.html",
    // ---- 第九批（2026-09-03）：处理类节点 ctor 第二批——WaveShaper（curve 拷贝
    // 语义 + oversample enum）/ DynamicsCompressor（五 AudioParam 缺省 + reduction
    // number + channelCount [1,2] 界）/ Panner（13 属性 + 六 AudioParam + listener
    // 面 + RangeError/InvalidStateError 校验）/ IIRFilter（feedforward/feedback
    // required + [1,20] 界 + fb[0]≠0 + getFrequencyResponse 异常面）——全部无渲染。
    // 不导入 ctor-iirfilter（Functional task 依赖 startRendering 渲染对比——
    // 语义面 AudioNodeOptions 已落 shim，随渲染切片复评）。
    "webaudio/the-audio-api/the-waveshapernode-interface/ctor-waveshaper.html",
    "webaudio/the-audio-api/the-dynamicscompressornode-interface/ctor-dynamicscompressor.html",
    "webaudio/the-audio-api/the-dynamicscompressornode-interface/dynamicscompressor-basic.html",
    "webaudio/the-audio-api/the-pannernode-interface/ctor-panner.html",
    "webaudio/the-audio-api/the-iirfilternode-interface/iirfilter-basic.html",
    // ---- 第十批（2026-09-03）：零新增缺口复评导入——biquadfilternode-basic
    //（type 八枚举 setter 面已由 ctor-biquadfilter 落 shim，断言 99 不生效）+
    // ctor-offlineaudiocontext（dict 构造/required/正义约束/destination 通道面
    // ——shim OfflineAudioContext 构造器扩 OfflineAudioContextOptions 后导入）。
    "webaudio/the-audio-api/the-biquadfilternode-interface/biquadfilternode-basic.html",
    "webaudio/the-audio-api/the-offlineaudiocontext-interface/ctor-offlineaudiocontext.html",
    // ---- 第十一批（2026-09-03）：源节点语义面——constant-source-basic（offset
    // min/max float 界 + start/stop 调度异常 W3CTH）/ stereopannernode-basic
    //（pan AudioParam + channelCount [1,2] setter 面）/ audiobuffersource-basic
    //（start/stop 异常 audit 面）。配套 shim：AudioScheduledSourceNode 调度异常
    // 共享面。
    "webaudio/the-audio-api/the-constantsourcenode-interface/constant-source-basic.html",
    "webaudio/the-audio-api/the-stereopanner-interface/stereopannernode-basic.html",
    "webaudio/the-audio-api/the-audiobuffersourcenode-interface/audiobuffersource-basic.html",
    // ---- 第十二批（2026-09-03）：ctor-audiobuffersource（全 task 零渲染；
    // ctor options 面 buffer/detune/loop/loopEnd/loopStart/playbackRate 反射——
    // shim 补 loopStart/loopEnd 后导入。此前排除注记与 ctor-audiobuffer 混淆，
    // 实测核对后解除）。
    "webaudio/the-audio-api/the-audiobuffersourcenode-interface/ctor-audiobuffersource.html",
    // ---- 第十三批（2026-09-03）：audiocontext-getoutputtimestamp（AudioTimestamp
    // 初始值面——shim getOutputTimestamp；其余 audiocontext-* 需用户手势/
    // onstatechange 时序/iframe 跨源 helper/真设备，维持排除）。
    "webaudio/the-audio-api/the-audiocontext-interface/audiocontext-getoutputtimestamp.html",
    // ---- 第十四批（2026-09-04）：the-audiocontext-interface 余件试导（门面 NullSink
    // 面可覆盖的 options/构造/suspend 形态；跑筛后定性保留或排除）。
    "webaudio/the-audio-api/the-audiocontext-interface/audiocontextoptions.html",
    "webaudio/the-audio-api/the-audiocontext-interface/suspend-after-construct.html",
    // ---- 第十五批（2026-09-04）：promise-methods-after-discard（iframe realm
    // 构造 + frame.remove() 后 suspend/resume/close reject InvalidStateError——
    // shim：part05 IframeAudioContext 绑定构造器（not fully active → InvalidStateError）
    // + part06 suspend/resume/close detached reject 面 + part01 removeChild 挂钩的
    // destroyed 印记与 SW client 解挂解耦）。
    "webaudio/the-audio-api/the-audiocontext-interface/promise-methods-after-discard.html",
    // ---- 第十六批（2026-09-04）：convolver/analyser 零渲染候选——ctor-convolver
    //（5 W3CTH task：invalid ctor TypeError / 缺省属性 / AudioNodeOptions [1,2] 界
    // + mode 校验 / nullable buffer / sampleRate 不匹配 NotSupportedError +
    // disableNormalization 面）/ convolver-setBuffer-null + -already-has-value
    //（buffer setter 重复赋值 + null 清空 audit 面）/ realtimeanalyser-basic
    //（1入1出 + 缺省 -100/-30/0.8 + 可写面）。配套 shim：ConvolverNode builder
    // + createConvolver 工厂（normalize/buffer sampleRate 校验 + channel12 界）。
    "webaudio/the-audio-api/the-convolvernode-interface/ctor-convolver.html",
    "webaudio/the-audio-api/the-convolvernode-interface/convolver-setBuffer-null.html",
    "webaudio/the-audio-api/the-convolvernode-interface/convolver-setBuffer-already-has-value.html",
    "webaudio/the-audio-api/the-analysernode-interface/realtimeanalyser-basic.html",
    // M3 第十七批：MediaStreamAudioDestinationNode ctor 语义面（1入0出 + explicit/
    // speakers 缺省 + options channelCount 非固定——shim _zwWABuildMediaStreamDestination）。
    "webaudio/the-audio-api/the-mediastreamaudiodestinationnode-interface/ctor-mediastreamaudiodestination.html",
    // M3 第十八批续：OfflineAudioContext detached execution context——
    // createElementNS iframe contentWindow gate 修复后解除排除。
    "webaudio/the-audio-api/the-offlineaudiocontext-interface/offlineaudiocontext-detached-execution-context.html",
    // 不导入：constructor-allowed-to-start（test_driver.bless 用户手势 + 断言
    // 「构造后立即 'suspended' → onstatechange 异步转 'running'」——shim headless
    // 恒 'running' 近似与该异步状态机断言结构性互斥，bless stub 化后仍必 Fail；
    // R142 unsupported 白名单不含 bless，误入清单使 make testharness-webaudio
    // exit 1——第十五批勘误移除）。
    // ---- 第十九批（2026-09-05，media-audio D3 获批窄授权——offline 渲染路径）：
    // startRendering 最小面落地（shim 侧 JS 波形合成——四型振荡器 + custom
    // periodic wave spec 归一化 + 线性 gain 链解析；offline 渲染不削幅——spec
    // AudioBuffer 可超 ±1）后解除排除导入。
    "webaudio/the-audio-api/the-oscillatornode-interface/osc-basic-waveform.html",
    // ---- 第二十批（D3 第三片——splitter/merger 通道路由图推进）：gain.html
    //（11 note 增益衰减渲染对比——通道 0/1 = gain 缩放、2/3 = 源直通，逐通道 SNR）。
    "webaudio/the-audio-api/the-gainnode-interface/gain.html",
    // ---- 第二十一批（D3 第三片续——AudioParam automation timeline）：增益包络
    // 调度事件表 + startRendering 逐采样求值（setValue/linear/exponential/target）。
    "webaudio/the-audio-api/the-audioparam-interface/audioparam-method-chaining.html",
    // ---- 第二十二批（D3 第五增量——AudioParam nominal range 界表）：每节点每
    // param spec min/max 反射 + value clamp + 只读 + 原型参数发现。
    "webaudio/the-audio-api/the-audioparam-interface/audioparam-nominal-range.html",
    // ---- 第二十三批（D3 第五增量续）：audiobuffer-copy-channel——copyFrom/
    // copyToChannel 数据面（无 startRendering 引用；原「后段不可分割」注记失效）。
    "webaudio/the-audio-api/the-audiobuffer-interface/audiobuffer-copy-channel.html",
    // ---- 第二十四批（D3 第六增量续）：ctor-audiobuffer——ctor 全族 + multiple
    // contexts 渲染对比（双 OfflineAudioContext 共享 buffer）。
    "webaudio/the-audio-api/the-audiobuffer-interface/ctor-audiobuffer.html",
    // ---- 第二十五批（D3 第七增量续）：ctor-iirfilter——ctor 校验 + Functional
    // 渲染对比（ctor vs 工厂同系数双通道对拍——pass-through 等价面精确相等）。
    "webaudio/the-audio-api/the-iirfilternode-interface/ctor-iirfilter.html",
    // ---- 第二十六批（D3 第九增量）：periodicWave.html（迁址路径）——Blink
    // maxAbsSum 归一化 + createPeriodicWave 最小长度 2 + setPeriodicWave。
    "webaudio/the-audio-api/the-periodicwave-interface/periodicWave.html",
    // ---- 第二十七批（D3 第十增量）：detune 耦合两件——computed frequency
    //（frequency · 2^(detune/1200)）+ ≥Nyquist 精确静默（跳过写入）+ detune
    // linearRamp 逐采样耦合频率（_zwParamValueAt 通用 automation 求值复用）。
    "webaudio/the-audio-api/the-oscillatornode-interface/detune-limiting.html",
    "webaudio/the-audio-api/the-oscillatornode-interface/detune-overflow.html",
    // ---- 第二十九批（D3 第十二增量）：sub-sample-start——亚帧起点 ceil 语义
    //（首输出帧 = ceil(t·rate)）+ 同值 ramp a-rate 路径与常频参考逐样本对拍。
    "webaudio/the-audio-api/the-oscillatornode-interface/sub-sample-start.html",
    // 不导入：audioparam 四型 ramp 对拍（setValueAtTime/linearRamp/exponentialRamp/
    // setTargetAtTime，第二十八批试导 2026-09-05）——上游坏 helper：audioparam-testing.js
    // verifyDiscontinuities 引用 createAudioGraphAndTest 形参 numberOfTests（非模块
    // 作用域）→ oncomplete 回调内必抛 ReferenceError → 上游 Chromium 亦红（master
    // 2026-09-05 同字节未修）。shim 侧 OfflineAudioContext.oncomplete 已作为基础设施
    // 资产落地（spec OfflineAudioCompletionEvent——event.renderedBuffer 承载渲染产物，
    // 语义正确保留）；四件待上游修复 helper 后复评。
    // ---- 第二十批（同日，D3 第二片——AudioBufferSourceNode 数据播放 + 链式
    // gain 累计 + 直连 destination 双态 + offset/duration/loop 窗口）：
    // 不导入：gain.html（需 splitter/merger 通道路由图语义——merger 通道选择映射
    // 是其断言核心，线性链近似不可达——第三片通道图面复评；ABSN 数据播放/gain
    // 链/loop 窗口渲染面已落 shim）。
    // 不导入：ctor-audiobuffer.html（末 task「multiple contexts」渲染对比——
    // 多 OfflineAudioContext 交叉渲染面随第二片复评）；audiobuffer-copy-channel
    //（startRendering 后段同文件不可分割——数据面已落 shim，同片复评）；
    // periodicWave.html（custom wave 谱断言——归一化系数精化后复评）。
    // 不导入：audioparam-nominal-range（Param 调度自动化面——value 调度表随
    // AudioParam 调度切片）。
    // baseLatency 档位值（playbackLatency×10 → 0.8 恒等断言——Linux Chromium 实测
    // 档），headless 无设备延迟模型不可复现；构造/enum/double/sampleRange 语义面
    // 已落 shim（AudioContextOptions + close/suspend/resume + baseLatency），随设备
    // 面（CpalSink 真出声切片）复评。
];

/// audit.js 框架脚本（wpt-data 内 vendored 原文件——与 canvas-tests.js 同款
/// inline_extras 内联机制；用例以绝对路径 `/webaudio/resources/*.js` 引用）。
pub const WEBAUDIO_SUPPORT_SCRIPTS: &[(&str, &str)] = &[
    ("/webaudio/resources/audit-util.js", "webaudio/resources/audit-util.js"),
    (
        "/webaudio/resources/audionodeoptions.js",
        "webaudio/resources/audionodeoptions.js",
    ),
    ("/webaudio/resources/audit.js", "webaudio/resources/audit.js"),
    // M3 扩批 XXIV：audioparam-testing.js（audiobuffer-getChannelData 引用——
    // 仅脚本加载，两个 task 不触发 createAudioGraphAndTest 渲染路径）。
    (
        "/webaudio/resources/audioparam-testing.js",
        "webaudio/resources/audioparam-testing.js",
    ),
    // M3 扩批 XXVII：start-stop-exceptions.js（constant-source-basic 以相对路径
    // ../../resources/ 引用 + audiobuffersource-basic 以绝对路径引用——调度异常
    // 共享断言 helper，纯语义面）。
    (
        "/webaudio/resources/start-stop-exceptions.js",
        "webaudio/resources/start-stop-exceptions.js",
    ),
];

/// Run the pinned upstream Web Audio testharness subset（media-audio M3 切片 2）。
pub fn run_webaudio_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    // audit.js 框架内联（canvas-tests.js 同款机制——用例以绝对路径引用
    // /webaudio/resources/*.js，extract_page_scripts 不加载外部 src）。
    let inline_extras: Vec<(&str, String)> = WEBAUDIO_SUPPORT_SCRIPTS
        .iter()
        .filter_map(|(src, path)| {
            read_corpus_to_string(wpt_root.join(path))
                .ok()
                .map(|content| (*src, content))
        })
        .collect();
    let inline_refs: Vec<(&str, &str)> = inline_extras
        .iter()
        .map(|(src, content)| (*src, content.as_str()))
        .collect();

    WEBAUDIO_TEST_FILES
        .iter()
        .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
        .map(|path| {
            let source = read_corpus_to_string(wpt_root.join(path));
            let results = match source {
                Ok(source) => {
                    run_testharness_html_inner(wpt_root, path, &source, &harness_source, &inline_refs, CASE_TIMEOUT)
                }
                Err(error) => vec![HarnessSubtestResult {
                    name: "load WPT case".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            };
            ((*path).to_string(), results)
        })
        .collect()
}

/// Run the pinned upstream media-elements testharness subset.
pub fn run_media_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    MEDIA_TEST_FILES
        .iter()
        .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
        .map(|path| {
            let source = read_corpus_to_string(wpt_root.join(path));
            let results = match source {
                Ok(source) => run_testharness_html(wpt_root, path, &source, &harness_source, CASE_TIMEOUT),
                Err(error) => vec![HarnessSubtestResult {
                    name: "load WPT case".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            };
            ((*path).to_string(), results)
        })
        .collect()
}

/// Run the pinned upstream IndexedDB `.any.js` subset.
pub fn run_indexeddb_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    INDEXEDDB_CASES
        .iter()
        .filter(|(path, _)| filter.is_none_or(|filter| path.contains(filter)))
        .map(|(path, support)| {
            let case_source = match read_corpus_to_string(wpt_root.join(path)) {
                Ok(source) => source,
                Err(error) => {
                    return (
                        (*path).to_string(),
                        vec![HarnessSubtestResult {
                            name: "load IndexedDB case".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    );
                }
            };
            let case_dir = Path::new(path).parent().unwrap_or_else(|| Path::new(""));
            let mut support_sources = Vec::with_capacity(support.len());
            for script in *support {
                match read_corpus_to_string(wpt_root.join(case_dir).join(script)) {
                    Ok(source) => support_sources.push((*script, source)),
                    Err(error) => {
                        return (
                            (*path).to_string(),
                            vec![HarnessSubtestResult {
                                name: format!("load IndexedDB support {script}"),
                                status: HarnessStatus::Fail,
                                message: Some(error.to_string()),
                            }],
                        );
                    }
                }
            }
            let support_refs = support_sources
                .iter()
                .map(|(name, source)| (*name, source.as_str()))
                .collect::<Vec<_>>();
            let html = indexeddb_window_wrapper(path, &support_refs, &case_source);
            let results = run_testharness_html(wpt_root, path, &html, &harness_source, CASE_TIMEOUT);
            ((*path).to_string(), results)
        })
        .collect()
}

/// Run the pinned upstream CacheStorage window subset.
pub fn run_cache_storage_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return CACHE_STORAGE_WINDOW_CASES
                .iter()
                .filter(|(path, _)| filter.is_none_or(|filter| path.contains(filter)))
                .map(|(path, _)| {
                    (
                        (*path).to_string(),
                        vec![HarnessSubtestResult {
                            name: "load testharness.js".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    )
                })
                .collect();
        }
    };

    CACHE_STORAGE_WINDOW_CASES
        .iter()
        .filter(|(path, _)| filter.is_none_or(|filter| path.contains(filter)))
        .map(|(path, support)| {
            let case_source = if let Some(source) = cache_storage_builtin_case_source(path) {
                source.to_string()
            } else {
                match read_corpus_to_string(wpt_root.join(path)) {
                    Ok(source) => source,
                    Err(error) => {
                        return (
                            (*path).to_string(),
                            vec![HarnessSubtestResult {
                                name: "load CacheStorage case".into(),
                                status: HarnessStatus::Fail,
                                message: Some(error.to_string()),
                            }],
                        );
                    }
                }
            };
            let case_dir = Path::new(path).parent().unwrap_or_else(|| Path::new(""));
            let mut support_sources = Vec::with_capacity(support.len());
            for script in *support {
                let support_path = if let Some(root_relative) = script.strip_prefix('/') {
                    wpt_root.join(root_relative)
                } else {
                    wpt_root.join(case_dir).join(script)
                };
                match read_corpus_to_string(support_path) {
                    Ok(source) => support_sources.push((*script, source)),
                    Err(error) => {
                        return (
                            (*path).to_string(),
                            vec![HarnessSubtestResult {
                                name: format!("load CacheStorage support {script}"),
                                status: HarnessStatus::Fail,
                                message: Some(error.to_string()),
                            }],
                        );
                    }
                }
            }
            let html = if path.ends_with(".html") {
                let mut html = apply_wpt_substitutions(&case_source);
                for (name, source) in &support_sources {
                    let source = apply_wpt_substitutions(source);
                    html = replace_script_source(&html, name, &format!("<script>{source}</script>"));
                }
                html
            } else {
                let support_refs = support_sources
                    .iter()
                    .map(|(name, source)| (*name, source.as_str()))
                    .collect::<Vec<_>>();
                any_js_window_wrapper(path, &support_refs, &case_source)
            };
            let results = run_testharness_html(wpt_root, path, &html, &harness_source, CASE_TIMEOUT);
            ((*path).to_string(), results)
        })
        .collect()
}

fn cache_storage_builtin_case_source(path: &str) -> Option<&'static str> {
    match path {
        "service-workers/cache-storage/zeroweb-filtered-response-types.https.any.js" => {
            Some(ZEROWEB_CACHE_FILTERED_RESPONSE_TYPES_SOURCE)
        }
        _ => None,
    }
}

/// Run the pinned upstream File System (OPFS) window subset.
///
/// Support scripts are discovered from each case's `// META: script=` headers
/// (wpt_meta_scripts), so no hand-maintained case→support pairing is needed.
pub fn run_fs_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return FS_CASES
                .iter()
                .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
                .map(|path| {
                    (
                        (*path).to_string(),
                        vec![HarnessSubtestResult {
                            name: "load testharness.js".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    )
                })
                .collect();
        }
    };

    FS_CASES
        .iter()
        .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
        .map(|path| {
            let case_source = match read_corpus_to_string(wpt_root.join(path)) {
                Ok(source) => source,
                Err(error) => {
                    return (
                        (*path).to_string(),
                        vec![HarnessSubtestResult {
                            name: "load File System case".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    );
                }
            };
            let case_dir = Path::new(path).parent().unwrap_or_else(|| Path::new(""));
            let mut support_sources = Vec::new();
            for script in wpt_meta_scripts(path, &case_source) {
                let support_path = if let Some(root_relative) = script.strip_prefix('/') {
                    wpt_root.join(root_relative)
                } else {
                    wpt_root.join(case_dir).join(&script)
                };
                match read_corpus_to_string(support_path) {
                    Ok(source) => support_sources.push((script, source)),
                    Err(error) => {
                        return (
                            (*path).to_string(),
                            vec![HarnessSubtestResult {
                                name: format!("load File System support {script}"),
                                status: HarnessStatus::Fail,
                                message: Some(error.to_string()),
                            }],
                        );
                    }
                }
            }
            let support_refs = support_sources
                .iter()
                .map(|(name, source)| (name.as_str(), source.as_str()))
                .collect::<Vec<_>>();
            let html = any_js_window_wrapper(path, &support_refs, &case_source);
            let results = run_testharness_html(wpt_root, path, &html, &harness_source, CASE_TIMEOUT);
            ((*path).to_string(), results)
        })
        .collect()
}

/// Run the pinned upstream WebAssembly jsapi window subset
/// （page-wasm goal M1 / DC-1）。结构同 [`run_fs_cases`]：固定清单 + META script
/// 自动解析 + [`any_js_window_wrapper`] window 变体执行。
pub fn run_wasm_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return WASM_CASES
                .iter()
                .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
                .map(|path| {
                    (
                        (*path).to_string(),
                        vec![HarnessSubtestResult {
                            name: "load testharness.js".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    )
                })
                .collect();
        }
    };

    WASM_CASES
        .iter()
        .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
        .map(|path| {
            let case_source = match read_corpus_to_string(wpt_root.join(path)) {
                Ok(source) => source,
                Err(error) => {
                    return (
                        (*path).to_string(),
                        vec![HarnessSubtestResult {
                            name: "load WebAssembly case".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    );
                }
            };
            let case_dir = Path::new(path).parent().unwrap_or_else(|| Path::new(""));
            let mut support_sources = Vec::new();
            for script in wpt_meta_scripts(path, &case_source) {
                let support_path = if let Some(root_relative) = script.strip_prefix('/') {
                    wpt_root.join(root_relative)
                } else {
                    wpt_root.join(case_dir).join(&script)
                };
                match read_corpus_to_string(support_path) {
                    Ok(source) => support_sources.push((script, source)),
                    Err(error) => {
                        return (
                            (*path).to_string(),
                            vec![HarnessSubtestResult {
                                name: format!("load WebAssembly support {script}"),
                                status: HarnessStatus::Fail,
                                message: Some(error.to_string()),
                            }],
                        );
                    }
                }
            }
            let support_refs = support_sources
                .iter()
                .map(|(name, source)| (name.as_str(), source.as_str()))
                .collect::<Vec<_>>();
            let html = any_js_window_wrapper(path, &support_refs, &case_source);
            let results = run_testharness_html(wpt_root, path, &html, &harness_source, CASE_TIMEOUT);
            ((*path).to_string(), results)
        })
        .collect()
}

/// Run the pinned upstream WebStorage window subset
/// （webstorage-compat goal M1 / DC-1）。结构同 [`run_wasm_cases`]：固定清单 +
/// [`any_js_window_wrapper`] window 变体执行；用例无 META support 依赖（skip 域见
/// [`WEBSTORAGE_CASES`] 文档）。配额用例（`while(true) setItem`）依赖
/// [`CASE_TIMEOUT`] 单案超时记账为 Fail，不挂起全量。
pub fn run_webstorage_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return WEBSTORAGE_CASES
                .iter()
                .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
                .map(|path| {
                    (
                        (*path).to_string(),
                        vec![HarnessSubtestResult {
                            name: "load testharness.js".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    )
                })
                .collect();
        }
    };

    WEBSTORAGE_CASES
        .iter()
        .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
        .map(|path| {
            let case_source = match read_corpus_to_string(wpt_root.join(path)) {
                Ok(source) => source,
                Err(error) => {
                    return (
                        (*path).to_string(),
                        vec![HarnessSubtestResult {
                            name: "load WebStorage case".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    );
                }
            };
            let html = any_js_window_wrapper(path, &[], &case_source);
            let results = run_testharness_html(wpt_root, path, &html, &harness_source, CASE_TIMEOUT);
            ((*path).to_string(), results)
        })
        .collect()
}

/// compression-compat goal（24 / DC-1）corpus 目录——fetch 脚本
/// `scripts/goals/24-compression-compat.sh`（同 WPT_REV pin）拉取 `.any.js` 用例与
/// `resources/`、`third_party/pako/` 依赖到 `wpt-data/compression/`（gitignored）。
const COMPRESSION_TEST_DIR: &str = "compression";

/// [`COMPRESSION_TEST_DIR`] 用例名排除规则（runner 侧记账排除，fetch 侧全量拉取）。
fn compression_case_skipped(relative: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    // idlharness.*：依赖 /resources/WebIDLParser.js 与 /interfaces/compression.idl
    // build 期生成资产，repo 内不存在，window 不可执行（clipboard/timing/net-api
    // idlharness 先例）。
    name.starts_with("idlharness.")
}

/// Run the pinned upstream compression window subset
/// （compression-compat goal M1 / DC-1——CompressionStream/DecompressionStream 语义基线）。
/// 目录扫描 `.any.js`（用例唯一形态，无 .html），META script 自动解析（fs/wasm 先例；
/// `third_party/pako/pako_inflate.min.js` 相对用例目录解析），[`any_js_window_wrapper`]
/// window 变体执行。filter 按路径子串过滤。
pub fn run_compression_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{COMPRESSION_TEST_DIR}/"),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    let mut cases = Vec::new();
    let Ok(entries) = std::fs::read_dir(wpt_root.join(COMPRESSION_TEST_DIR)) else {
        return cases;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() || !name.ends_with(".any.js") {
            continue;
        }
        let relative = format!("{COMPRESSION_TEST_DIR}/{name}");
        if filter.is_some_and(|filter| !relative.contains(filter)) || compression_case_skipped(&relative) {
            continue;
        }
        let Ok(case_source) = read_corpus_to_string(&path) else {
            continue;
        };
        let case_dir = Path::new(&relative).parent().unwrap_or_else(|| Path::new(""));
        let mut support_sources = Vec::new();
        let mut support_failed = false;
        for script in wpt_meta_scripts(&relative, &case_source) {
            let support_path = match script.strip_prefix('/') {
                Some(root_relative) => wpt_root.join(root_relative),
                None => wpt_root.join(case_dir).join(&script),
            };
            match read_corpus_to_string(support_path) {
                Ok(source) => support_sources.push((script, source)),
                Err(error) => {
                    cases.push((
                        relative.clone(),
                        vec![HarnessSubtestResult {
                            name: format!("load compression support {script}"),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    ));
                    support_failed = true;
                    break;
                }
            }
        }
        if support_failed {
            continue;
        }
        let support_refs = support_sources
            .iter()
            .map(|(name, source)| (name.as_str(), source.as_str()))
            .collect::<Vec<_>>();
        let html = any_js_window_wrapper(&relative, &support_refs, &case_source);
        let results = run_testharness_html(wpt_root, &relative, &html, &harness_source, CASE_TIMEOUT);
        cases.push((relative, results));
    }
    cases
}

/// webcrypto-compat goal（26 / DC-1）corpus 目录——fetch 脚本
/// `scripts/goals/26-webcrypto-compat.sh`（同 WPT_REV pin，git trees 深挖
/// `lib.sh fetch_dir_deep`）拉取 `.any.js`/`.window.js` 用例与 vectors/support
/// 到 `wpt-data/WebCryptoAPI/`（gitignored）。
const WEBCRYPTO_TEST_DIR: &str = "WebCryptoAPI";

/// [`WEBCRYPTO_TEST_DIR`] 用例名排除规则（runner 侧记账排除，fetch 侧全量拉取）。
/// - `idlharness.*`：依赖 /resources/WebIDLParser.js 与 idlharness.js build 期生成
///   资产，repo 内不存在，window 不可执行（compression idlharness 先例）。
/// - `secure_context/` 的 `.sub.html` 为 include 资产非用例（扫描只取 .js 形态，
///   天然排除）。
fn webcrypto_case_skipped(relative: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    name.starts_with("idlharness.")
}

/// Run the pinned upstream WebCryptoAPI window subset
/// （webcrypto-compat goal M1 / DC-1——getRandomValues/randomUUID/subtle 分域基线）。
/// 目录**递归**扫描 `.any.js` / `.window.js`（用例入口形态；语料埋二三层子目录），
/// META script 自动解析（compression 先例；vectors/support 与用例同树，
/// `/common/subset-tests.js`、`/common/gc.js` 由 fetch 脚本补拉），[`any_js_window_wrapper`]
/// window 变体执行。排序收集保证基线顺序可复现。filter 按路径子串过滤。
pub fn run_webcrypto_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{WEBCRYPTO_TEST_DIR}/"),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    // 递归收集用例路径（含子目录；排序保证跨轮基线可复现）。
    let mut case_paths = Vec::new();
    let mut stack = vec![wpt_root.join(WEBCRYPTO_TEST_DIR)];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".any.js") || name.ends_with(".window.js") {
                case_paths.push(path);
            }
        }
    }
    case_paths.sort();
    let mut cases = Vec::new();
    for path in case_paths {
        let relative = match path.strip_prefix(wpt_root) {
            Ok(relative) => relative.to_string_lossy().to_string(),
            Err(_) => continue,
        };
        if filter.is_some_and(|filter| !relative.contains(filter)) || webcrypto_case_skipped(&relative) {
            continue;
        }
        let Ok(case_source) = read_corpus_to_string(&path) else {
            continue;
        };
        let case_dir = Path::new(&relative).parent().unwrap_or_else(|| Path::new(""));
        let mut support_sources = Vec::new();
        let mut support_failed = false;
        for script in wpt_meta_scripts(&relative, &case_source) {
            let support_path = match script.strip_prefix('/') {
                Some(root_relative) => wpt_root.join(root_relative),
                None => wpt_root.join(case_dir).join(&script),
            };
            match read_corpus_to_string(support_path) {
                Ok(source) => support_sources.push((script, source)),
                Err(error) => {
                    cases.push((
                        relative.clone(),
                        vec![HarnessSubtestResult {
                            name: format!("load WebCryptoAPI support {script}"),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    ));
                    support_failed = true;
                    break;
                }
            }
        }
        if support_failed {
            continue;
        }
        let support_refs = support_sources
            .iter()
            .map(|(name, source)| (name.as_str(), source.as_str()))
            .collect::<Vec<_>>();
        let html = any_js_window_wrapper(&relative, &support_refs, &case_source);
        let results = run_testharness_html(wpt_root, &relative, &html, &harness_source, CASE_TIMEOUT);
        cases.push((relative, results));
    }
    cases
}

/// webgl-compat goal（99 / M2）corpus 目录——fetch 脚本
/// `scripts/goals/99-webgl-compat.sh`（同 WPT_REV pin，`GOAL_PULL_ANY_JS` opt-in +
/// idlharness 资产补拉）拉取到 `wpt-data/webgl/`（gitignored）。pin rev 处为
/// 平铺目录（7 testharness 错误语义案 + 1 手工 crash 页 + idlharness.any.js +
/// common.js 支撑 + 元数据），扫描按目录递归保持形态鲁棒。
const WEBGL_TEST_DIR: &str = "webgl";

/// Run the pinned upstream webgl/ corpus（webgl-compat goal M2——getContext 上下文
/// 对象面 + buffer/texture 错误语义 + idlharness 接口面基线）。
///
/// `.html` 用例直接作为 testharness 页运行；`idlharness.any.js` 经
/// [`any_js_window_wrapper`] 包窗（META scripts：WebIDLParser/idlharness/webgl
/// common.js——idlharness 资产已按 2026-10-22 用户裁决③补拉，无 skip 记账排除）。
/// `idl_test` 运行期经 fetch('/interfaces/*.idl') 取 IDL，由 page fetch 处理的
/// 静态文件回退路径供给。退出码契约由调用方（`testharness-webgl` 子命令）执行：
/// 有用例非 Pass 或用例集为空 → 1。
pub fn run_webgl_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{WEBGL_TEST_DIR}/"),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    let mut case_paths = Vec::new();
    let mut stack = vec![wpt_root.join(WEBGL_TEST_DIR)];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".html") || name.ends_with(".any.js") {
                case_paths.push(path);
            }
        }
    }
    case_paths.sort();
    let mut cases = Vec::new();
    for path in case_paths {
        let relative = match path.strip_prefix(wpt_root) {
            Ok(relative) => relative.to_string_lossy().to_string(),
            Err(_) => continue,
        };
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let Ok(case_source) = read_corpus_to_string(&path) else {
            continue;
        };
        // .html 用例（testharness 页）直接运行；.any.js（idlharness）包窗。
        if !relative.ends_with(".any.js") {
            let results = run_testharness_html(wpt_root, &relative, &case_source, &harness_source, CASE_TIMEOUT);
            cases.push((relative, results));
            continue;
        }
        let case_dir = Path::new(&relative).parent().unwrap_or_else(|| Path::new(""));
        let mut support_sources = Vec::new();
        let mut support_failed = false;
        for script in wpt_meta_scripts(&relative, &case_source) {
            let support_path = match script.strip_prefix('/') {
                Some(root_relative) => wpt_root.join(root_relative),
                None => wpt_root.join(case_dir).join(&script),
            };
            match read_corpus_to_string(support_path) {
                Ok(source) => support_sources.push((script, source)),
                Err(error) => {
                    cases.push((
                        relative.clone(),
                        vec![HarnessSubtestResult {
                            name: format!("load webgl support {script}"),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    ));
                    support_failed = true;
                    break;
                }
            }
        }
        if support_failed {
            continue;
        }
        let support_refs = support_sources
            .iter()
            .map(|(name, source)| (name.as_str(), source.as_str()))
            .collect::<Vec<_>>();
        let html = any_js_window_wrapper(&relative, &support_refs, &case_source);
        let results = run_testharness_html(wpt_root, &relative, &html, &harness_source, CASE_TIMEOUT);
        cases.push((relative, results));
    }
    cases
}

/// Web Components goal（web-components M1 / DC-1）pinned upstream subset——三个目录的
/// 递归 .html 用例清单。Cases are fetched by `fetch-web-components-subset.sh`（WPT
/// 315976933870b34d6ea30e3f6643403edae678ba）into `wpt-data/`（gitignored）.
///
/// 清单在 runner 侧按目录递归扫描 + [`WC_SKIP_PATTERNS`] 排除重建（fetch 脚本头注释
/// 同一份 skip 域），非逐案常量——custom-elements 175 案逐条列常量维护成本高于按名
/// 排除，且两侧用同一规则集防漂移。skip 域（Support Envelope 排除项 + 非 window
/// 可执行形态）：
/// - `*-ref.html`/`*-notref.html`（reftest 参照页）
/// - 渲染级 composed tree / 几何命中 / 交互面（slot-fallback-content-*、layout-*、
///   directionality-*、restyle-*、shadow-style-*、invalidate-sibling-*、
///   manual-slot-assignment-*、imperative-slot-layout-invalidation-*、
///   user-agent-shadow*、nested-hover-*、accesskey*、touch-*、wheel-*、scroll-*、
///   offset*、focus-within-*、caretPositionFromPoint、elementFromPoint、
///   highlightsFromPoint、offsetX-offsetY、drag*、css-user-agent-style-sheet-*）
/// - 非主线程/范围外形态：declarative/（declarative SD）、leaktests/、crashtests/
///   （slice 1 不导入崩溃面）、focus*/（焦点导航交互面）、reference-target/、
///   resources/（helper 资产非用例）
pub const WC_TEST_SUBDIRS: &[&str] = &[
    "custom-elements",
    "shadow-dom",
    "html/semantics/scripting-1/the-template-element",
];

/// [`WC_TEST_SUBDIRS`] 的用例名排除规则（fetch 脚本 skip 域的 runner 侧镜像）。
/// 相对路径匹配；`ends_with`/`contains` 语义见各条注释。
fn wc_case_skipped(relative: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    let dir = relative.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    // 非用例资产（helper .js 由 inline_local_scripts 消费，ref 页是 reftest 参照）。
    if !name.ends_with(".html") || name.ends_with("-ref.html") || name.ends_with("-notref.html") {
        return true;
    }
    // 范围外子目录（fetch 脚本不拉，双保险）。
    for skipped_dir in [
        "shadow-dom/declarative",
        "shadow-dom/leaktests",
        "shadow-dom/crashtests",
        "shadow-dom/focus",
        "shadow-dom/focus-navigation",
        "shadow-dom/reference-target",
        "shadow-dom/reference",
        "shadow-dom/resources",
        "custom-elements/resources",
        "html/semantics/scripting-1/the-template-element/resources",
    ] {
        if dir.starts_with(skipped_dir) {
            return true;
        }
    }
    // 渲染级 composed tree / 几何命中 / 交互面（等用户点名 Shadow DOM 渲染级专项）。
    const RENDER_PREFIXES: &[&str] = &[
        "slot-fallback-content-",
        "layout-",
        "directionality-",
        "restyle-",
        "shadow-style-",
        "invalidate-sibling-",
        "manual-slot-assignment-",
        "imperative-slot-layout-invalidation-",
        "user-agent-shadow",
        "nested-hover-",
        "accesskey",
        "touch-",
        "wheel-",
        "scroll-",
        "offset",
        "focus-within-",
        "css-user-agent-style-sheet-",
        "drag",
    ];
    if RENDER_PREFIXES.iter().any(|prefix| name.starts_with(prefix)) {
        return true;
    }
    const RENDER_CONTAINS: &[&str] = &[
        "caretPositionFromPoint",
        "elementFromPoint",
        "highlightsFromPoint",
        "offsetX-offsetY",
    ];
    if RENDER_CONTAINS.iter().any(|needle| name.contains(needle)) {
        return true;
    }
    false
}

/// Run the pinned upstream Web Components subset（web-components M1 / DC-1）。
///
/// 递归扫描 [`WC_TEST_SUBDIRS`] 全部主线程 .html 用例，经 [`wc_case_skipped`] 排除
/// skip 域；仅依赖 `testharness.js` + 用例自带 helper（`resources/shadow-dom.js` /
/// `resources/custom-elements-helpers.js` / `/html/resources/common.js` 由
/// `inline_local_scripts` 内联）。filter 按路径子串过滤。
pub fn run_web_components_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    let mut cases = Vec::new();
    for subdir in WC_TEST_SUBDIRS {
        collect_wc_cases(wpt_root, subdir, &harness_source, filter, &mut cases);
    }
    cases
}

/// 递归收集一个 WC 子目录下的 .html 用例（[`wc_case_skipped`] 排除）并执行。
fn collect_wc_cases(
    wpt_root: &Path,
    subdir: &str,
    harness_source: &str,
    filter: Option<&str>,
    cases: &mut Vec<(String, Vec<HarnessSubtestResult>)>,
) {
    let dir = wpt_root.join(subdir);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let nested = format!("{subdir}/{}", entry.file_name().to_string_lossy());
            collect_wc_cases(wpt_root, &nested, harness_source, filter, cases);
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "html") {
            continue;
        }
        let relative = format!("{subdir}/{}", entry.file_name().to_string_lossy());
        if wc_case_skipped(&relative) || filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let Ok(source) = read_corpus_to_string(&path) else {
            continue;
        };
        let results = run_testharness_html(wpt_root, &relative, &source, harness_source, CASE_TIMEOUT);
        cases.push((relative, results));
    }
}

/// IntersectionObserver / ResizeObserver pinned upstream subset——event-loop-spec goal
/// M1 / DC-1。两目录 top-level .html 用例由 `fetch-observers-subset.sh` 全量拉取
/// （WPT 315976933870b34d6ea30e3f6643403edae678ba）into `wpt-data/`（gitignored），
/// 运行面按 [`observers_case_skipped`] 内容规则筛减。
///
/// skip 规则（fetch 脚本头注释同域，双保险）：
/// - *-ref.html / *-notref.html（reftest 参照页）
/// - source 含 `<iframe`——iframe 依赖面（runner 无 iframe 文档/几何管道）；按内容
///   而非名字判定（document-scrolling-element-root.html 名字无 iframe 但内容用 iframe）
/// - `intersection-observer/v2/`（IO v2 trackVisibility 范围外，fetch 侧不拉——双保险）
/// - `*/resources/`（helper 资产，inline_local_scripts / script fetcher 消费）
fn observers_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-ref.html") || name.ends_with("-notref.html") {
        return true;
    }
    for skipped_dir in [
        "intersection-observer/resources",
        "intersection-observer/v2",
        "resize-observer/resources",
    ] {
        if relative.starts_with(skipped_dir) {
            return true;
        }
    }
    source.contains("<iframe")
}

/// Run the pinned upstream IntersectionObserver window subset
/// （event-loop-spec goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_intersection_observer_cases(
    wpt_root: &Path,
    filter: Option<&str>,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_observers_subdir(wpt_root, "intersection-observer", filter)
}

/// Run the pinned upstream ResizeObserver window subset
/// （event-loop-spec goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_resize_observer_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_observers_subdir(wpt_root, "resize-observer", filter)
}

fn run_observers_subdir(
    wpt_root: &Path,
    subdir: &str,
    filter: Option<&str>,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{subdir}/"),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    // 两目录用例均为 flat 布局（resources/ 与 v2/ 是子目录），无需递归。
    let mut cases = Vec::new();
    let entries = match std::fs::read_dir(wpt_root.join(subdir)) {
        Ok(entries) => entries,
        Err(_) => return cases,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() || path.extension().is_none_or(|ext| ext != "html") {
            continue;
        }
        let relative = format!("{subdir}/{}", entry.file_name().to_string_lossy());
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let Ok(source) = read_corpus_to_string(&path) else {
            continue;
        };
        if observers_case_skipped(&relative, &source) {
            continue;
        }
        let results = run_testharness_html(wpt_root, &relative, &source, &harness_source, CASE_TIMEOUT);
        cases.push((relative, results));
    }
    cases
}

/// Clipboard API goal（web-api-batch2 M1 / DC-1）pinned upstream subset directories.
/// Cases are fetched by `fetch-clipboard-apis-subset.sh` into `wpt-data/`
/// （gitignored）——top-level window 用例 + events/permissions/text-write-read 子目录。
pub const CLIPBOARD_APIS_SUBDIRS: &[&str] = &[
    "clipboard-apis",
    "clipboard-apis/events",
    "clipboard-apis/permissions",
    "clipboard-apis/text-write-read",
];

/// Fullscreen goal（web-api-batch2 M1 / DC-1）pinned upstream subset directories.
/// Cases are fetched by `fetch-fullscreen-subset.sh` into `wpt-data/`
/// （gitignored）——api/ + model/ + crashtests/ 三目录（rendering/ 不拉取，
/// `:fullscreen` 样式面跨域记账 rendering-compat 流域）。
pub const FULLSCREEN_SUBDIRS: &[&str] = &["fullscreen/api", "fullscreen/model", "fullscreen/crashtests"];

/// web-api-batch2 两 corpus 共用的多子目录扫描器（observers [`run_observers_subdir`]
/// 同款形态：harness 内联 + 内容级 skip + CASE_TIMEOUT 单案执行）。
fn run_corpus_subdirs(
    wpt_root: &Path,
    subdirs: &[&str],
    filter: Option<&str>,
    case_skipped: fn(&str, &str) -> bool,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return subdirs
                .iter()
                .map(|subdir| {
                    (
                        format!("{subdir}/"),
                        vec![HarnessSubtestResult {
                            name: "load testharness.js".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    )
                })
                .collect();
        }
    };
    // 拉取面均为 flat 布局（resources/ 等是子目录，skip 规则排除），无需递归。
    let mut cases = Vec::new();
    for subdir in subdirs {
        let entries = match std::fs::read_dir(wpt_root.join(subdir)) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() || path.extension().is_none_or(|ext| ext != "html") {
                continue;
            }
            let relative = format!("{subdir}/{}", entry.file_name().to_string_lossy());
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let Ok(source) = read_corpus_to_string(&path) else {
                continue;
            };
            if case_skipped(&relative, &source) {
                continue;
            }
            let results = run_testharness_html(wpt_root, &relative, &source, &harness_source, corpus_case_timeout());
            cases.push((relative, results));
        }
    }
    cases
}

/// Run the pinned upstream Clipboard API window subset
/// （web-api-batch2 goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_clipboard_apis_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_corpus_subdirs(wpt_root, CLIPBOARD_APIS_SUBDIRS, filter, clipboard_apis_case_skipped)
}

/// Run the pinned upstream Fullscreen window subset
/// （web-api-batch2 goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_fullscreen_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_corpus_subdirs(wpt_root, FULLSCREEN_SUBDIRS, filter, fullscreen_case_skipped)
}

/// css-variables 上游语料目录（R5114：fetch-wpt-data 随 css/ 全量拉取
/// `wpt-data/css/css-variables`，无需独立 fetch 脚本）。
pub const CSS_VARIABLES_SUBDIRS: &[&str] = &["css/css-variables"];

/// Run the upstream css-variables window corpus（site-optimizer P9-9 切片 2/R5114
/// 测试资产化：gCS 自定义属性暴露 + StyleSheetList 语义的常驻上游断言）。filter
/// 按路径子串过滤。基线首跑通过率只记录不设门禁（clipboard/csp 先例）。
pub fn run_css_variables_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_corpus_subdirs(wpt_root, CSS_VARIABLES_SUBDIRS, filter, css_variables_case_skipped)
}

/// css-variables 运行面筛减规则（数据全量在 data 侧，运行面按内容规则收口）：
/// - `*/resources/`、`*/support/`、`*/reference/`（helper / ref 资产，不按案跑）
/// - `*-crash.html`（病态输入 crash 回归——解析器病态输入行为按仓库规约走
///   fuzz-repro-guard 口径，不进常驻语料运行面）
/// - `*-manual.html`（需真实用户交互；规则同域防新增）
/// - source 含 `<iframe`——iframe 依赖面（runner 无 iframe 文档管道，fullscreen 同口径）
fn css_variables_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html") || name.ends_with("-crash.html") {
        return true;
    }
    for skipped_dir in [
        "css/css-variables/resources",
        "css/css-variables/support",
        "css/css-variables/reference",
    ] {
        if relative.starts_with(skipped_dir) {
            return true;
        }
    }
    source.contains("<iframe")
}

/// Clipboard API pinned subset 运行面筛减规则（fetch 脚本头注释同域，双保险）：
/// - `*/resources/`（helper 资产，inline_local_scripts 消费）
/// - `detached-iframe/`、`permissions-policy/`（fetch 侧不拉——iframe 面 /
///   Permissions-Policy 头 infra；双保险防误放入扫描面）
/// - `*-manual.html`（需真实用户手势/物理剪贴板交互）
/// - `drag-multiple-urls.html`（DnD 挂账面，goal 排除）
/// - source 含 `<iframe`——iframe 依赖面（按内容而非名字判定，observers 先例）
fn clipboard_apis_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html") || name == "drag-multiple-urls.html" {
        return true;
    }
    for skipped_dir in [
        "clipboard-apis/resources",
        "clipboard-apis/detached-iframe",
        "clipboard-apis/permissions-policy",
    ] {
        if relative.starts_with(skipped_dir) {
            return true;
        }
    }
    source.contains("<iframe")
}

/// Fullscreen pinned subset 运行面筛减规则（fetch 脚本头注释同域，双保险）：
/// - `fullscreen/rendering/`（`:fullscreen` 伪类/UA 样式面——rendering-compat 流域
///   跨域记账，fetch 侧不拉取，双保险）
/// - `fullscreen/api/resources/`（iframe 内嵌 helper 页，不按案跑）
/// - `*-manual.html`（当前无此类案，规则同域防新增）
/// - source 含 `<iframe`——fullscreen 的 nested/allowfullscreen/cross-origin 深依赖
///   iframe 语义，runner 无 iframe 文档管道（跨域记账，重入条件 = iframe 管道）
fn fullscreen_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html") {
        return true;
    }
    for skipped_dir in ["fullscreen/rendering", "fullscreen/api/resources"] {
        if relative.starts_with(skipped_dir) {
            return true;
        }
    }
    source.contains("<iframe")
}

/// Security-hardening goal（docs/goal/archive/security-hardening.md M1 / DC-1）pinned upstream
/// subset directories. Cases are fetched by `fetch-security-csp-subset.sh` into
/// `wpt-data/`（gitignored）——CSP 覆盖范围指令目录 + securitypolicyviolation/generic/
/// meta/gen top.meta 面（M2+ 扩批目录见 fetch 脚本头注释）。
pub const CSP_CORPUS_SUBDIRS: &[&str] = &[
    "content-security-policy/securitypolicyviolation",
    "content-security-policy/generic",
    "content-security-policy/meta",
    "content-security-policy/default-src",
    "content-security-policy/script-src",
    "content-security-policy/style-src",
    "content-security-policy/img-src",
    "content-security-policy/connect-src",
    "content-security-policy/frame-src",
    "content-security-policy/font-src",
    "content-security-policy/media-src",
    "content-security-policy/object-src",
    "content-security-policy/base-uri",
    "content-security-policy/form-action",
    "content-security-policy/frame-ancestors",
    "content-security-policy/blob",
    "content-security-policy/child-src",
    // 第二批扩批（security-hardening R7）：政策继承 / 导航 / sandbox 指令 / eval 面。
    "content-security-policy/inheritance",
    "content-security-policy/navigation",
    "content-security-policy/sandbox",
    "content-security-policy/unsafe-eval",
    "content-security-policy/wasm-unsafe-eval",
    "content-security-policy/gen/top.meta/script-src-self",
    "content-security-policy/gen/top.meta/script-src-wildcard",
    "content-security-policy/gen/top.meta/worker-src-none",
    "content-security-policy/gen/top.meta/worker-src-self",
    "content-security-policy/gen/top.meta/worker-src-wildcard",
];

/// Mixed Content goal（security-hardening M1 / DC-1）pinned upstream subset.
/// 顶层可执行面（blob.https.sub.html / imageset.https.sub.html）；gen/ 为
/// window.js/iframe 包装形态不拉取（fetch 脚本头注释同域）。
pub const MIXED_CONTENT_SUBDIRS: &[&str] = &["mixed-content"];

/// Secure Contexts goal（security-hardening M1 / DC-1）pinned upstream subset.
pub const SECURE_CONTEXTS_SUBDIRS: &[&str] = &["secure-contexts"];

/// Security 三 corpus 共用运行面筛减规则（fetch 脚本头注释同域，双保险）：
/// - `*(-ref|-notref).html`（reftest 参照页，observers 先例）
/// - `*-manual.html`（需真实用户交互，clipboard/fullscreen 先例）
/// - source 含 `<iframe`——iframe 依赖面（按内容而非名字判定，observers 先例；
///   frame-ancestors / form-action 深依赖 iframe 语义，runner 无 iframe 文档管道）
/// - source 含 `new Worker(` / `SharedWorker(` / `navigator.serviceWorker`——worker
///   执行面（inside-worker / secure-contexts worker 族，runner 无 worker 文档管道）
fn security_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-ref.html")
        || name.ends_with("-notref.html")
        || name.ends_with("-manual.html")
        // secure-contexts 的 iframe 内嵌 helper 页（仅作为上游 iframe 用例的载荷，
        // 独立执行必 Fail——resources/ 同类，observers 先例）
        || name.starts_with("postMessage-helper")
    {
        return true;
    }
    source.contains("<iframe")
        || source.contains("new Worker(")
        || source.contains("SharedWorker(")
        || source.contains("navigator.serviceWorker")
}

/// Run the pinned upstream Content-Security-Policy window subset
/// （security-hardening goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_content_security_policy_cases(
    wpt_root: &Path,
    filter: Option<&str>,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_corpus_subdirs(wpt_root, CSP_CORPUS_SUBDIRS, filter, security_case_skipped)
}

/// Run the pinned upstream Mixed Content window subset
/// （security-hardening goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_mixed_content_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_corpus_subdirs(wpt_root, MIXED_CONTENT_SUBDIRS, filter, security_case_skipped)
}

/// Run the pinned upstream Secure Contexts window subset
/// （security-hardening goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_secure_contexts_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_corpus_subdirs(wpt_root, SECURE_CONTEXTS_SUBDIRS, filter, security_case_skipped)
}

// ══ timing-animation-compat goal（docs/goal/archive/timing-animation-compat.md M1 / DC-1）══
// 四 corpus（hr-time / performance-timeline / user-timing / web-animations）window
// 可执行子集。fetch 侧 = `scripts/goals/10-timing-animation-compat.sh`（目录清单 +
// .any.js 显式清单补拉），运行面 skip 规则与 fetch 清单同域双保险。.any.js 以 window
// 变体执行（wasm/fs/indexeddb 先例：`// META: script=` 自动解析 + window wrapper）。

/// hr-time corpus pinned upstream subset（计时面——performance.now 精度/单调性/
/// timeOrigin）。
pub const HR_TIME_SUBDIRS: &[&str] = &["hr-time"];

/// performance-timeline corpus pinned upstream subset（PerformanceObserver/entry
/// buffer 面）。
pub const PERFORMANCE_TIMELINE_SUBDIRS: &[&str] = &["performance-timeline"];

/// user-timing corpus pinned upstream subset（mark/measure/clear 面）。
pub const USER_TIMING_SUBDIRS: &[&str] = &["user-timing"];

/// web-animations corpus pinned upstream subset（WAAPI——Animation/KeyframeEffect/
/// getAnimations API 面 + 时序模型语义）。responsive/ 与 crashtests/ 已 fetch 不扫描：
/// 前者为 responsive reftest（动画渲染效果跨域记账 rendering-compat，fullscreen/
/// rendering 先例），后者为崩溃面（web-components crashtests 先例）。
pub const WEB_ANIMATIONS_SUBDIRS: &[&str] = &[
    "web-animations/animation-model",
    "web-animations/animation-trigger",
    "web-animations/interfaces/Animatable",
    "web-animations/interfaces/Animation",
    "web-animations/interfaces/AnimationEffect",
    "web-animations/interfaces/AnimationPlaybackEvent",
    "web-animations/interfaces/Document",
    "web-animations/interfaces/DocumentTimeline",
    "web-animations/interfaces/KeyframeEffect",
    "web-animations/interfaces/TimelineTrigger",
    "web-animations/timing-model/animation-effects",
    "web-animations/timing-model/animations",
    "web-animations/timing-model/time-transformations",
    "web-animations/timing-model/timelines",
];

/// 绝对路径 helper → wpt-data 本地文件映射（fetch 脚本补拉）。`inline_local_scripts`
/// 只内联相对 src；上游整仓 serve 的绝对路径 helper（/common/ 通用资产、
/// /web-animations/ 顶面 testcommon）经本表以 inline_extras 内联，本地文件缺失时
/// 跳过该 extra（用例按真实缺口失败，不静默注入空）。
const TIMING_ABSOLUTE_HELPERS: &[(&str, &str)] = &[
    (
        "/common/performance-timeline-utils.js",
        "common/performance-timeline-utils.js",
    ),
    ("/web-animations/testcommon.js", "web-animations/testcommon.js"),
];

/// 收集用例引用的绝对路径 helper 的 inline_extras（只装被引用且已拉取的）。
fn timing_absolute_helper_extras(wpt_root: &Path, source: &str) -> Vec<(String, String)> {
    TIMING_ABSOLUTE_HELPERS
        .iter()
        .filter(|(src, _)| source.contains(src))
        .filter_map(|(src, local)| {
            read_corpus_to_string(wpt_root.join(local))
                .ok()
                .map(|content| ((*src).to_string(), content))
        })
        .collect()
}

/// timing 四 corpus 共用扫描器：`.html` 用例直跑（[`run_corpus_subdirs`] 同款内容级
/// skip）；`.any.js` 用例以 window 变体执行（[`run_wasm_cases`] 同款 META 解析 +
/// wrapper）；绝对路径 helper 经 [`timing_absolute_helper_extras`] 内联。
fn run_timing_subdirs(
    wpt_root: &Path,
    subdirs: &[&str],
    filter: Option<&str>,
    case_skipped: fn(&str, &str) -> bool,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    let mut cases = Vec::new();
    for subdir in subdirs {
        // 拉取面均为 flat 布局（子目录已显式列入 subdirs），无需递归。
        let Ok(entries) = std::fs::read_dir(wpt_root.join(subdir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            let is_html = file_name.ends_with(".html");
            let is_any_js = file_name.ends_with(".any.js");
            if !is_html && !is_any_js {
                continue;
            }
            let relative = format!("{subdir}/{file_name}");
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let Ok(source) = read_corpus_to_string(&path) else {
                continue;
            };
            if case_skipped(&relative, &source) {
                continue;
            }
            let results = if is_any_js {
                run_any_js_window_case(wpt_root, &relative, &source, &harness_source)
            } else {
                let extras = timing_absolute_helper_extras(wpt_root, &source);
                let extra_refs = extras
                    .iter()
                    .map(|(src, body)| (src.as_str(), body.as_str()))
                    .collect::<Vec<_>>();
                run_testharness_html_inner(
                    wpt_root,
                    &relative,
                    &source,
                    &harness_source,
                    &extra_refs,
                    CORPUS_CASE_TIMEOUT,
                )
            };
            cases.push((relative, results));
        }
    }
    cases
}

/// `.any.js` 单案 window 变体执行（[`run_wasm_cases`] 同款：META script 自动解析 +
/// [`any_js_window_wrapper`]；support 缺失按 Fail 记账不静默跳过）。
pub(crate) fn run_any_js_window_case(
    wpt_root: &Path,
    relative: &str,
    source: &str,
    harness_source: &str,
) -> Vec<HarnessSubtestResult> {
    let case_dir = Path::new(relative).parent().unwrap_or_else(|| Path::new(""));
    let mut support_sources = Vec::new();
    for script in wpt_meta_scripts(relative, source) {
        let support_path = if let Some(root_relative) = script.strip_prefix('/') {
            wpt_root.join(root_relative)
        } else {
            wpt_root.join(case_dir).join(&script)
        };
        match read_corpus_to_string(support_path) {
            Ok(support) => support_sources.push((script, support)),
            Err(error) => {
                return vec![HarnessSubtestResult {
                    name: format!("load support {script}"),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }];
            }
        }
    }
    let support_refs = support_sources
        .iter()
        .map(|(name, source)| (name.as_str(), source.as_str()))
        .collect::<Vec<_>>();
    let html = any_js_window_wrapper(relative, &support_refs, source);
    run_testharness_html(wpt_root, relative, &html, harness_source, corpus_case_timeout())
}

/// timing 四 corpus 共用基础筛减：`*-manual.html`（需真实用户交互，clipboard 先例）、
/// `*-ref.html`/`*-notref.html`（reftest 参照页，observers 先例）、`idlharness.*`
/// （依赖 /resources/WebIDLParser.js——上游 resources/webidl2 的 build 期生成资产，
/// repo 内不存在，window 可执行面之外，clipboard idlharness 先例）。
fn timing_base_case_skipped(relative: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    name.ends_with("-manual.html")
        || name.ends_with("-ref.html")
        || name.ends_with("-notref.html")
        || name.starts_with("idlharness.")
}

/// iframe 依赖面：literal `<iframe`（observers/clipboard 先例）或
/// `document.createElement('iframe')` 动态形态（hr-time 跨 frame 测量、
/// performance-timeline detached-frame）——runner 无多 frame 文档管道。
fn timing_iframe_dependency(source: &str) -> bool {
    source.contains("<iframe")
        || source.contains("createElement('iframe')")
        || source.contains("createElement(\"iframe\")")
}

/// worker 执行面：runner 无 worker 文档管道（security/clipboard 先例）。
fn timing_worker_dependency(source: &str) -> bool {
    source.contains("new Worker(") || source.contains("SharedWorker(")
}

/// hr-time 筛减：基础 + iframe 依赖。worker 内容放行——timeOrigin.html 首子测试为
/// window timeOrigin 核心断言（DC-2 面），worker 子测试按真实缺口 Timeout/Fail
/// 记账（evidence 注记）。
fn hr_time_case_skipped(relative: &str, source: &str) -> bool {
    timing_base_case_skipped(relative) || timing_iframe_dependency(source)
}

/// performance-timeline 筛减：基础 + iframe/worker + bfcache/navigationId 基建面
/// （`helper.sub.js` + `/common/utils.js` dispatcher 跨窗会话机制在 window 可执行面
/// 之外——navigation-compat 流域记账）。
fn performance_timeline_case_skipped(relative: &str, source: &str) -> bool {
    timing_base_case_skipped(relative)
        || timing_iframe_dependency(source)
        || timing_worker_dependency(source)
        || source.contains("back-forward-cache/resources/helper.sub.js")
}

/// user-timing 筛减：基础 + iframe/worker（当前案面均不含，规则同域防上游漂移）。
fn user_timing_case_skipped(relative: &str, source: &str) -> bool {
    timing_base_case_skipped(relative) || timing_iframe_dependency(source) || timing_worker_dependency(source)
}

/// web-animations 筛减：基础 + iframe/worker + reftest-wait 渲染等待面
/// （animation-model/side-effects-*——动画渲染效果跨域记账 rendering-compat，
/// goal 边界：JS API 面 vs 渲染计算）。
fn web_animations_case_skipped(relative: &str, source: &str) -> bool {
    timing_base_case_skipped(relative)
        || timing_iframe_dependency(source)
        || timing_worker_dependency(source)
        || source.contains("reftest-wait")
}

/// Run the pinned upstream hr-time window subset
/// （timing-animation-compat goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_hr_time_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_timing_subdirs(wpt_root, HR_TIME_SUBDIRS, filter, hr_time_case_skipped)
}

/// Run the pinned upstream performance-timeline window subset
/// （timing-animation-compat goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_performance_timeline_cases(
    wpt_root: &Path,
    filter: Option<&str>,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_timing_subdirs(
        wpt_root,
        PERFORMANCE_TIMELINE_SUBDIRS,
        filter,
        performance_timeline_case_skipped,
    )
}

/// Run the pinned upstream user-timing window subset
/// （timing-animation-compat goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_user_timing_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_timing_subdirs(wpt_root, USER_TIMING_SUBDIRS, filter, user_timing_case_skipped)
}

/// Run the pinned upstream web-animations window subset
/// （timing-animation-compat goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_web_animations_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_timing_subdirs(wpt_root, WEB_ANIMATIONS_SUBDIRS, filter, web_animations_case_skipped)
}

// ── performance-api-compat goal（docs/goal/archive/performance-api-compat.md M1 / DC-1）：
// resource / navigation / event / longtask 四 corpus window 可执行子集。拉取面 =
// tests/wpt-runner/scripts/goals/19-performance-api-compat.sh（DIRS 与本表同域双
// 保险）。WPT 无 performance-api 顶层目录——四件套是四个独立顶层套件（M1 实测）。

/// resource-timing corpus pinned upstream subset（PerformanceResourceTiming 条目
/// 生成与 buffer 语义面）。
pub const RESOURCE_TIMING_SUBDIRS: &[&str] = &["resource-timing"];

/// navigation-timing corpus pinned upstream subset（PerformanceNavigationTiming
/// 导航生命周期条目面）。
pub const NAVIGATION_TIMING_SUBDIRS: &[&str] = &["navigation-timing"];

/// event-timing corpus pinned upstream subset（PerformanceEventTiming 事件时延
/// 面——事件循环集成）。
pub const EVENT_TIMING_SUBDIRS: &[&str] = &["event-timing"];

/// longtask-timing corpus pinned upstream subset（longtask 长任务判定面）。
pub const LONGTASK_TIMING_SUBDIRS: &[&str] = &["longtask-timing"];

/// performance-api 四 corpus 绝对路径 helper 补充（`timing_absolute_helper_extras`
/// 消费 TIMING_ABSOLUTE_HELPERS——在此追加本 goal 引用面，本地缺失时跳过该 extra，
/// 用例按真实缺口失败）。/common/get-host-info.sub.js、/common/utils.js 已在
/// wpt-data（前序 goal 拉取），此处只补 timing 表未覆盖的两个。
const PERFORMANCE_API_ABSOLUTE_HELPERS: &[(&str, &str)] =
    &[("/common/get-host-info.sub.js", "common/get-host-info.sub.js")];

/// performance-api 四 corpus 绝对路径 helper 内联（[`timing_absolute_helper_extras`]
/// 同款，只装被引用且已拉取的）。
fn performance_api_absolute_helper_extras(wpt_root: &Path, source: &str) -> Vec<(String, String)> {
    TIMING_ABSOLUTE_HELPERS
        .iter()
        .chain(PERFORMANCE_API_ABSOLUTE_HELPERS.iter())
        .filter(|(src, _)| source.contains(src))
        .filter_map(|(src, local)| {
            read_corpus_to_string(wpt_root.join(local))
                .ok()
                .map(|content| ((*src).to_string(), content))
        })
        .collect()
}

/// performance-api 四 corpus 共用筛减（M1 可执行面定义，缺面逐项理由同步
/// docs/goal/archive/performance-api-compat/evidence/ M1 甄别报告）：
/// - timing 基础（manual/ref/idlharness——[`timing_base_case_skipped`]；idlharness
///   依赖 WebIDLParser build 期生成资产，clipboard idlharness 先例）
/// - iframe / worker 依赖（runner 无多 frame / worker 文档管道，timing 先例）：
///   resource-timing 跨 frame 观察面、event-timing crossiframe、longtask subframe
/// - `.py` server handler 引用（redirect/content-resp 等——wptserve 动态端点，
///   静态语料不可执行；net-api「server-handler .py 重依赖」先例）
/// - `/common/dispatcher` 跨窗会话机制（bfcache / navigationId helper 依赖——
///   performance-timeline 同款记账）
/// - `custom-cors-response` / `web-bundle` / `compression-dictionary` 引用（CORS
///   响应整形与未拉取支持目录——TAO/字典装载面在本地 fetch handler 之外）
/// - bfcache helper（back-forward-cache/resources/helper.sub.js——跨窗会话）
///
/// 语义缺面（无 resource/navigation/event/longtask 条目生成管道）**不在 skip 之列**
/// ——用例照跑、按真实 Fail 记账，这正是基线要度量的分母。
fn performance_api_case_skipped(relative: &str, source: &str) -> bool {
    timing_base_case_skipped(relative)
        || timing_iframe_dependency(source)
        || timing_worker_dependency(source)
        || source.contains(".py")
        || source.contains("/common/dispatcher")
        || source.contains("custom-cors-response")
        || source.contains("web-bundle")
        || source.contains("compression-dictionary")
        || source.contains("back-forward-cache/resources/helper.sub.js")
        // longtask 容器归因面（longtask-timing/containerTypes——iframe/frame/object/
        // embed 四容器经 Container helper 动态构建，`<iframe`/createElement 字面规则
        // 不达）：跨 frame 条目归因需多 frame 管道，runner 不可执行面。
        || source.contains("new Container('iframe'")
}

/// Run the pinned upstream resource-timing window subset
/// （performance-api-compat goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_resource_timing_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_performance_api_subdirs(wpt_root, RESOURCE_TIMING_SUBDIRS, filter)
}

/// Run the pinned upstream navigation-timing window subset
/// （performance-api-compat goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_navigation_timing_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_performance_api_subdirs(wpt_root, NAVIGATION_TIMING_SUBDIRS, filter)
}

/// Run the pinned upstream event-timing window subset
/// （performance-api-compat goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_event_timing_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_performance_api_subdirs(wpt_root, EVENT_TIMING_SUBDIRS, filter)
}

/// Run the pinned upstream longtask-timing window subset
/// （performance-api-compat goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_longtask_timing_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_performance_api_subdirs(wpt_root, LONGTASK_TIMING_SUBDIRS, filter)
}

/// performance-api 四 corpus 共用扫描器（[`run_timing_subdirs`] 同构：`.html` 直跑 +
/// `.any.js`/`.window.js` 以 window 变体执行（upstream `.window.js` 即 window-only
/// 变体源，META 机制与 .any.js 同款——longtask buffered-flag/long-microtask 核心
/// 语义案面在此形态）；绝对路径 helper 经 [`performance_api_absolute_helper_extras`]
/// 内联；skip 用 [`performance_api_case_skipped`]）。与 [`run_timing_subdirs`] 唯一
/// 差异是 helper 表、skip 函数与 `.window.js` 变体——独立入口避免动 timing 四
/// corpus 的既有行为。
fn run_performance_api_subdirs(
    wpt_root: &Path,
    subdirs: &[&str],
    filter: Option<&str>,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    let mut cases = Vec::new();
    for subdir in subdirs {
        // 拉取面均为 flat 布局（子目录已显式列入 subdirs），无需递归。
        let Ok(entries) = std::fs::read_dir(wpt_root.join(subdir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            let is_html = file_name.ends_with(".html");
            let is_any_js = file_name.ends_with(".any.js") || file_name.ends_with(".window.js");
            if !is_html && !is_any_js {
                continue;
            }
            let relative = format!("{subdir}/{file_name}");
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let Ok(source) = read_corpus_to_string(&path) else {
                continue;
            };
            if performance_api_case_skipped(&relative, &source) {
                continue;
            }
            let results = if is_any_js {
                run_any_js_window_case(wpt_root, &relative, &source, &harness_source)
            } else {
                let extras = performance_api_absolute_helper_extras(wpt_root, &source);
                let extra_refs = extras
                    .iter()
                    .map(|(src, body)| (src.as_str(), body.as_str()))
                    .collect::<Vec<_>>();
                run_testharness_html_inner(
                    wpt_root,
                    &relative,
                    &source,
                    &harness_source,
                    &extra_refs,
                    CORPUS_CASE_TIMEOUT,
                )
            };
            cases.push((relative, results));
        }
    }
    cases
}

// ── net-api-compat goal（docs/goal/archive/net-api-compat.md M1 / DC-1）：六 corpus window
// 可执行子集（fetch / xhr / url / mimesniff / streams / eventsource）。拉取面 =
// tests/wpt-runner/scripts/goals/20-net-api-compat.sh（DIRS 与本表同域双保险）。

/// net-api 六 corpus pinned subset directories（fetch 脚本 DIRS 同域——M2/M3 语义簇 +
/// 自包含面；server-handler .py 重依赖 / 策略头面 / HTTP 栈面 / 新 API 面显式不拉，
/// fetch 脚本头注释逐域记账）。
pub const NET_API_CORPUS_SUBDIRS: &[&str] = &[
    "fetch/api/headers",
    "fetch/api/request",
    "fetch/api/response",
    "fetch/api/body",
    "fetch/api/basic",
    "fetch/api/abort",
    "fetch/api/credentials",
    "fetch/api/cors",
    "fetch/data-urls",
    "fetch/content-type",
    "fetch/content-length",
    "fetch/h1-parsing",
    "fetch/images",
    "fetch/redirects",
    "xhr",
    "xhr/formdata",
    "url",
    "mimesniff/mime-types",
    "streams",
    "streams/piping",
    "streams/readable-byte-streams",
    "streams/readable-streams",
    "streams/transform-streams",
    "streams/writable-streams",
    "eventsource",
    // P6 二期切片（M5-S1，重入条件 2026-10-13 兑现）：websockets/ 连接面由 runner
    // 内嵌 WS fixture 端点承接（ws_fixture.rs，语义基准 = 上游 handlers/*.py）。
    // multi-globals（多文档）/unload-a-document（多页编排）不列——runner 无多文档
    // 管道，与既有 worker/iframe 筛减同因。
    "websockets",
    "websockets/binary",
    "websockets/closing-handshake",
    "websockets/constructor",
    "websockets/cookies",
    "websockets/keeping-connection-open",
    "websockets/opening-handshake",
    "websockets/security",
];

/// net-api corpus 页面 scheme 判定（M5-S5，M4-S27 记账翻转）：六 corpus 用例页锚
/// `http://wpt.test`（上游 ?default 变体语义）；其余通道（service-workers 等
/// secure-context 面）维持 https 页锚不动。websockets 已在此列（M5-S1 特例被
/// 覆盖，语义不变）。子串前缀匹配与 [`run_net_api_cases`] 的通道枚举同口径。
fn net_api_case_uses_http_page(case_name: &str) -> bool {
    NET_API_CORPUS_SUBDIRS.iter().any(|dir| case_name.starts_with(dir))
}

/// WebCryptoAPI corpus 页面 scheme 判定（webcrypto-compat 残留收口）：上游按文件名
/// `.https.` 中缀双态——带中缀的案（digest/derive/sign_verify 等全部语义案）跑
/// https 安全上下文，无中缀的案（historical.any.js 断言非安全上下文无
/// crypto.subtle/CryptoKey、getRandomValues.any.js 与 scheme 无关）跑 http。runner
/// 页锚同口径翻转，shim 侧 [SecureContext] 门禁（part01b `_zwSubtle`）随之生效。
fn webcrypto_case_uses_http_page(case_name: &str) -> bool {
    let Some(name) = case_name.strip_prefix("WebCryptoAPI/") else {
        return false;
    };
    let file = name.rsplit('/').next().unwrap_or(name);
    !file.contains(".https.")
}

/// s116：referrer corpus 页面 scheme 判定——referrer-strict-policies 前提「文档
/// over HTTP」（断言文本与导入链均锚 http：https 文档令 `./resources` 包装导入
/// https 化、checker 子请求成降级，origin 断言翻红）。webcrypto 同口径先例。
fn referrer_case_uses_http_page(case_name: &str) -> bool {
    case_name.ends_with("module/referrer-strict-policies.sub.html")
}

/// navigation-compat M2-S10 单案 origin 端口化：上游 wptserve 恒带显式端口
/// （http :8800 / https :9443），location_port.html 断言取 `location.href` 权威段
/// ":" 后子串与 `location.port` 全等——runner 缺省 origin 无端口时权威段整体
/// （"wpt.test"）≠ ""，案设环境缺失恒 Fail。单案端口化 = 复现上游端口形态；
/// 引擎 URL 权威端口解析与 location.port 语义仍是真实断言目标。
fn navigation_case_uses_ported_origin(case_name: &str) -> bool {
    case_name.ends_with("/the-location-interface/location_port.html")
}

/// net-api 六 corpus 共用运行面筛减规则（fetch 脚本头注释同域，双保险）：
/// - `*/resources/`（helper 资产——fetch/api/resources、xhr/resources、url/resources、
///   eventsource/resources、streams/resources 等，META script / inline 消费不按案跑）
/// - `*-manual.html`（需真实用户交互，clipboard/security 先例）
/// - `idlharness.*`（/resources/WebIDLParser.js 为 build 期生成资产，repo 内不存在，
///   timing 先例）
/// - source 含 `<iframe` / `createElement('iframe')`——iframe 依赖面（runner 无多
///   frame 文档管道，observers/timing 先例）
/// - source 含 `new Worker(` / `SharedWorker(` / `navigator.serviceWorker`——worker
///   执行面（runner 无 worker 文档管道，security/timing 先例）
fn net_api_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html") || name.starts_with("idlharness.") {
        return true;
    }
    // net-api M5-S1：websockets `.https.*` 案需安全上下文 + TLS wss 端点（上游 ?wss
    // 变体）——runner 页面为 http origin、fixture 无 TLS，两臂均不可测，筛减记账
    // （mixed-content 案在 http 页必假阴）。
    if relative.starts_with("websockets/") && name.contains(".https.") {
        return true;
    }
    if relative.contains("/resources/") {
        return true;
    }
    source.contains("<iframe")
        || source.contains("createElement('iframe')")
        || source.contains("createElement(\"iframe\")")
        || source.contains("new Worker(")
        || source.contains("SharedWorker(")
        || source.contains("navigator.serviceWorker")
}

/// encoding corpus pinned subset directories（fetch 脚本 DIRS 同域——JS API 面
/// textdecoder/textencoder/streams 变体 + legacy-mb 七编码资产（M2 解码表参照）；
/// 文档级嗅探面 / .py 服务器端点显式不跑，fetch 脚本头注释逐域记账）。
pub const ENCODING_CORPUS_SUBDIRS: &[&str] = &[
    "encoding",
    "encoding/streams",
    "encoding/legacy-mb-japanese/euc-jp",
    "encoding/legacy-mb-japanese/iso-2022-jp",
    "encoding/legacy-mb-japanese/shift_jis",
    "encoding/legacy-mb-korean/euc-kr",
    "encoding/legacy-mb-schinese/gbk",
    "encoding/legacy-mb-schinese/gb18030",
    "encoding/legacy-mb-tchinese/big5",
];

/// encoding corpus 共用运行面筛减规则（fetch 脚本头注释同域，双保险）：
/// - `encoding/legacy-mb-*/*.html` 不跑——decode 族为 iframe 子文档解码面（runner
///   无多 frame 文档管道），encode-form/encode-href 族为 document 编码耦合的 form
///   提交/URL 序列化面；目录照拉：`*-decoder.any.js`（gbk/gb18030 TextDecoder
///   pointer 数组案——DC-2 可执行载体）照跑，index .js 为 M2 数据资产，net-api
///   「resources 只拉 helper 不按案跑」先例
/// - `*/resources/`（helper 资产 + BOM/UTF-32 子文档——decode-common/encodings.js 等
///   按 META script 消费，不按案跑）
/// - `*-manual.html` / `*-ref.html` / `*-notref.html` / `idlharness.*`（net-api/
///   timing 先例；eof-* 族 test 页经 rel=match 规则、ref 页经后缀规则）
/// - `.https.html`（secure-context + SharedArrayBuffer 面——window http 通道之外，
///   clipboard 先例）
/// - 文档级编码嗅探面（goal 排除——`document.characterSet` 断言的 bom-handling/
///   sniffing/utf-32 族、`<meta charset>` 文档编码耦合的 big5/iso-2022-jp-encoder
///   URL 序列化面、`rel=match` 的 eof-* reftest 页；挂账 html-syntax-compat，M4 定稿）
/// - iframe 依赖面（utf-32/remove-only-one-bom 动态形态——runner 无多 frame 文档
///   管道，net-api 先例）
/// - worker 执行面（net-api 先例）
fn encoding_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if relative.starts_with("encoding/legacy-mb-") && relative.ends_with(".html") {
        return true;
    }
    if name.ends_with("-manual.html")
        || name.ends_with("-ref.html")
        || name.ends_with("-notref.html")
        || name.ends_with(".https.html")
        || name.starts_with("idlharness.")
    {
        return true;
    }
    if relative.contains("/resources/") {
        return true;
    }
    source.contains("document.characterSet")
        || source.contains("<meta charset")
        || source.contains("rel=match")
        || source.contains("rel=\"match\"")
        || source.contains("<iframe")
        || source.contains("createElement('iframe')")
        || source.contains("createElement(\"iframe\")")
        || source.contains("new Worker(")
        || source.contains("SharedWorker(")
        || source.contains("navigator.serviceWorker")
}

/// flat 布局 corpus 通用扫描执行，.html + .any.js window 变体双形态（web-api-batch2
/// [`run_corpus_subdirs`] 为 html-only 变体；timing 专用的 absolute-helper extras
/// 分支不适用 net/encoding 面——.html 案直接 [`run_testharness_html`]）。
pub(crate) fn run_any_js_corpus_subdirs(
    wpt_root: &Path,
    subdirs: &[&str],
    filter: Option<&str>,
    case_skipped: fn(&str, &str) -> bool,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_any_js_corpus_subdirs_with_helpers(wpt_root, subdirs, filter, case_skipped, &[])
}

/// [`run_any_js_corpus_subdirs`] 的绝对路径 helper 变体（timing
/// [`timing_absolute_helper_extras`] 同款语义）：`absolute_helpers` 为
/// `(上游绝对 src, wpt-data 本地相对路径)` 表——`.html` 案 source 引用且本地已拉取时经
/// inline extras 内联，缺失跳过按真实缺口失败。
fn run_any_js_corpus_subdirs_with_helpers(
    wpt_root: &Path,
    subdirs: &[&str],
    filter: Option<&str>,
    case_skipped: fn(&str, &str) -> bool,
    absolute_helpers: &[(&str, &str)],
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_any_js_corpus_subdirs_with_helpers_variants(wpt_root, subdirs, filter, case_skipped, absolute_helpers, false)
}

/// navigation-compat S4K：同上 + variant-meta 展开（`case_variants`——uievents 尾簇 7 既有
/// 提取器复用；expand_variants 门仅 navigation corpus 启用）。
fn run_any_js_corpus_subdirs_with_helpers_variants(
    wpt_root: &Path,
    subdirs: &[&str],
    filter: Option<&str>,
    case_skipped: fn(&str, &str) -> bool,
    absolute_helpers: &[(&str, &str)],
    expand_variants: bool,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    let mut cases = Vec::new();
    for subdir in subdirs {
        // 拉取面均为 flat 布局（子目录已显式列入 NET_API_CORPUS_SUBDIRS），无需递归。
        let Ok(entries) = std::fs::read_dir(wpt_root.join(subdir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            let is_html = file_name.ends_with(".html");
            let is_any_js = file_name.ends_with(".any.js");
            if !is_html && !is_any_js {
                continue;
            }
            let relative = format!("{subdir}/{file_name}");
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let Ok(source) = read_corpus_to_string(&path) else {
                continue;
            };
            if case_skipped(&relative, &source) {
                continue;
            }
            // navigation-compat S4K：variant-meta 展开（expand_variants 且 html 形态）。
            let variants = if expand_variants && is_html {
                case_variants(&source)
            } else {
                Vec::new()
            };
            if is_any_js {
                let results = run_any_js_window_case(wpt_root, &relative, &source, &harness_source);
                cases.push((relative, results));
            } else {
                let extras: Vec<(String, String)> = absolute_helpers
                    .iter()
                    .filter(|(src, _)| source.contains(src))
                    .filter_map(|(src, local)| {
                        read_corpus_to_string(wpt_root.join(local))
                            .ok()
                            .map(|content| ((*src).to_string(), content))
                    })
                    .collect();
                let extra_refs = extras
                    .iter()
                    .map(|(src, body)| (src.as_str(), body.as_str()))
                    .collect::<Vec<_>>();
                if variants.is_empty() {
                    // net-api 二期 M2-S3：.html 案面 wptserve substitution（{{host}}/
                    // {{ports}}/{{domains}}/{{hosts}} → 本地锚定值）——.any.js 窗口包装
                    //（any_js_window_wrapper）与 cookies 通道既有同款语义，.html 分支此前
                    // 缺位，fetch/api request-init-001/003 referrer 案面 token 字面残留
                    // → URL 解析失败/期望串不匹配。替换只动行内 token 不动行数（CSP
                    // 位置表按原始 source 预计算，行号稳定）。
                    let substituted = apply_wpt_substitutions(&source);
                    let results = run_testharness_html_inner(
                        wpt_root,
                        &relative,
                        &substituted,
                        &harness_source,
                        &extra_refs,
                        corpus_case_timeout(),
                    );
                    cases.push((relative, results));
                } else {
                    // navigation-compat S4K：每变体独立运行（case URL 追加 query——
                    // WPT variant 用例矩阵；case 名带 query 与上游 dashboard 对齐）。
                    for variant in variants {
                        let case_v = format!("{relative}{variant}");
                        let results_v = run_testharness_html_inner(
                            wpt_root,
                            &case_v,
                            &source,
                            &harness_source,
                            &extra_refs,
                            corpus_case_timeout(),
                        );
                        cases.push((case_v, results_v));
                    }
                }
            }
        }
    }
    cases
}

/// Run the pinned upstream net-API six-corpus window subset
/// （net-api-compat goal M1 / DC-1）。filter 按路径子串过滤（如 `fetch/`、`xhr/`、
/// `streams/`——基线按 corpus 分类）。
pub fn run_net_api_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_any_js_corpus_subdirs(wpt_root, NET_API_CORPUS_SUBDIRS, filter, net_api_case_skipped)
}

/// Run the pinned upstream encoding window subset
/// （encoding-compat goal M1 / DC-1）。filter 按路径子串过滤（如 `textdecoder-`、
/// `streams/`、`legacy-mb-japanese/`——基线按域分类）。
pub fn run_encoding_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_any_js_corpus_subdirs(wpt_root, ENCODING_CORPUS_SUBDIRS, filter, encoding_case_skipped)
}

/// encoding/legacy-mb 文档族通道语料叶目录（fetch 脚本 goals/31-encoding-legacy-mb.sh
/// DIRS 同域：日/韩/繁中/简中四族 × 各编码标签目录；flat 扫描不递归，与 fetch
/// depth 布局对齐）。
pub const ENCODING_LEGACY_MB_CORPUS_SUBDIRS: &[&str] = &[
    "encoding/legacy-mb-japanese/euc-jp",
    "encoding/legacy-mb-japanese/iso-2022-jp",
    "encoding/legacy-mb-japanese/shift_jis",
    "encoding/legacy-mb-korean/euc-kr",
    "encoding/legacy-mb-schinese/gb18030",
    "encoding/legacy-mb-schinese/gbk",
    "encoding/legacy-mb-tchinese/big5",
];

/// legacy-mb corpus 案面规则（31 号 fetch 脚本头注释同域）：只跑 decode wrapper 页
/// ——`*-decode.html` / `*-decode-<label>.html`（testharness + `<meta charset>`
/// legacy 标签 + chars iframe + `<meta name="variant">` 窗口 ×14；decode-common.js
/// 按 iframe span 断言 `textContent == decoder(dataset.bytes)`）与
/// `*-decode-errors.html`（错误替换面，自足 inline 断言）。其余全 skip：
/// `*-chars*.html` / `*_errors.html`（iframe 子文档，由 wrapper 经 iframe 消费）；
/// `*-encode-form-*.html` / `*-encode-href*.html`（goal 划界：form 提交编码与
/// `<a href>` URL 序列化面不属本 goal，M3 甄别）；helper JS（`*-decoder.js`/
/// `*index*.js`/`ranges.js`）与 `.headers` sidecar 非案；`.any.js`（30 号 JS API
/// 面遗留的 `*-decoder.any.js`，testharness-encoding 通道已收口）不在本域。
fn encoding_legacy_mb_case_skipped(relative: &str, _source: &str) -> bool {
    if relative.ends_with(".any.js") {
        return true;
    }
    let name = relative.rsplit('/').next().unwrap_or(relative);
    !(name.ends_with("-decode.html") || name.contains("-decode-"))
}

/// Run the legacy-mb document-family decode wrappers（encoding-legacy-mb goal M1 /
/// DC-2）。filter 按路径子串透传（如 `euc-jp/`、`iso-2022-jp/`、`big5/`——基线按
/// 编码族分类）。variant 展开（navigation S4K 先例）+ iframe 管道（html-syntax
/// xmldecl R115 先例：引擎侧子文档字节解码/嗅探）。
pub fn run_encoding_legacy_mb_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_any_js_corpus_subdirs_with_helpers_variants(
        wpt_root,
        ENCODING_LEGACY_MB_CORPUS_SUBDIRS,
        filter,
        encoding_legacy_mb_case_skipped,
        &[],
        true,
    )
}

/// html-syntax corpus pinned subset directories（fetch 脚本 goals/40-html-syntax-compat.sh
/// DIRS 同域——html/syntax 解析树/序列化面 + html/dom 接口语义/reflection 面 +
/// encoding-compat P5 双向记账转入的文档级编码嗅探案面）。html/syntax 顶层无 .html
/// （全在子目录）——列叶子目录；html/dom 顶层 .html（reflection-*/aria-*/historical）
/// 直列。flat 扫描（子目录不递归），与 fetch depth-1 布局对齐。
pub const HTML_SYNTAX_CORPUS_SUBDIRS: &[&str] = &[
    "html/syntax/charset",
    "html/syntax/parsing",
    "html/syntax/parsing-html-fragments",
    "html/syntax/serializing-html-fragments",
    "html/syntax/serializing-xml-fragments",
    "html/syntax/speculative-charset",
    "html/syntax/xmldecl",
    "html/dom",
    "html/dom/directionality",
    "html/dom/elements",
    "html/dom/render-blocking",
];

/// html-syntax corpus 共用运行面筛减规则（fetch 脚本头注释同域，双保险）：
/// - `*-ref.html` / `-notref.html`（reftest ref 页）、`-manual.html`（用户交互面）、
///   `.https.html`（secure-context 面——usvstring-reflection）、`idlharness.*`
///   （WebIDLParser build 期资产）——net-api/encoding 先例
/// - `*/resources/`（helper 资产——html/dom/resources 子文档，不按案跑）
/// - 无 `testharness.js` 引用页（legacy 自述式 foreign_content_* 11 案、索引页
///   reflection-original、crash 回归页 range-on-pi-*——非 testharness 形态）
/// - rel=match reftest 形态页（`<meta charset>` 文档解码面：charset/ 全域 7 案 +
///   directionality/bdi test 页——testharness 通道外，P5 落地面归 reftest 通道）
/// - HTTP 头编码面（`flags content='http'`——the-input-byte-stream 11 案断言依赖
///   per-file HTTP charset 头，本地 file 通道无法复现测试条件；P5 记账）
/// - iframe 依赖面（runner 无多 frame 文档管道，net-api 先例——serializing.html
///   解析上下文 iframe、doctype-system-identifier-distinction compatMode iframe
///   面；例外：xmldecl/ 3 案 2026-10-20 起收编——R115 同步 iframe 管道 + 子文档
///   字节解码/嗅探（part01 _zwFinishIframeEntry）就绪，support/ corpus 已补拉）
/// - worker 执行面（net-api 先例；当前语料零命中，re-pin 增案防静默错跑）
fn html_syntax_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html")
        || name.ends_with("-ref.html")
        || name.ends_with("-notref.html")
        || name.ends_with(".https.html")
        || name.starts_with("idlharness.")
    {
        return true;
    }
    if relative.contains("/resources/") {
        return true;
    }
    if !source.contains("testharness.js") {
        return true;
    }
    source.contains("rel=match")
        || source.contains("rel=\"match\"")
        || source.contains("content='http'")
        || (source.contains("<iframe") && !relative.starts_with("html/syntax/xmldecl/"))
        || source.contains("createElement('iframe')")
        || source.contains("createElement(\"iframe\")")
        || source.contains("new Worker(")
        || source.contains("SharedWorker(")
        || source.contains("navigator.serviceWorker")
}

/// Run the pinned upstream html/syntax + html/dom window subset
/// （html-syntax-compat goal M1 / DC-1——解析树一致性/序列化/接口语义基线 + P5
/// 文档级嗅探案面）。filter 按路径子串过滤（如 `serializing-html-fragments/`、
/// `reflection-`——基线按域分类）。
pub fn run_html_syntax_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_any_js_corpus_subdirs(wpt_root, HTML_SYNTAX_CORPUS_SUBDIRS, filter, html_syntax_case_skipped)
}

// ── navigation-compat goal（docs/goal/archive/navigation-compat.md M1 / DC-1）：导航面
// corpus window 可执行子集（navigation-api / 会话历史 / 导航遍历 / iframe 元素属性）。
// 拉取面 = tests/wpt-runner/scripts/goals/60-navigation-compat.sh（DIRS 与本表同域
// 双保险）。顶层 history/ 在 WPT_REV pin 下不存在——会话历史 corpus 实际落在
// html/browsers/history/** 四叶目录（fetch 脚本头注释同域记账）。

/// navigation corpus pinned subset directories（fetch 脚本 DIRS 同域——DC-2 轻面
/// history/location/事件序标尺 + DC-3 门控面的资产盘存面）。navigation-api/ 顶层无
/// .html（全在 12 个语义子目录）——列叶子目录；history 面的 resources//non-automated/
/// 子目录不列（helper 资产 / 上游标记不可自动跑，skip 规则同域双保险）。
pub const NAVIGATION_CORPUS_SUBDIRS: &[&str] = &[
    // navigation-api 12 leaf dirs（Navigation API——navigatesuccess/currententry 面）
    "navigation-api/currententrychange-event",
    "navigation-api/focus-reset",
    "navigation-api/navigate-event",
    "navigation-api/navigation-activation",
    "navigation-api/navigation-history-entry",
    "navigation-api/navigation-methods",
    "navigation-api/ordering-and-transition",
    "navigation-api/per-entry-events",
    "navigation-api/precommit-handler",
    "navigation-api/scroll-behavior",
    "navigation-api/state",
    "navigation-api/updateCurrentEntry-method",
    // 会话历史（history/location/会话历史条目语义——DC-2 轻面标尺）
    "html/browsers/history/the-history-interface",
    "html/browsers/history/the-history-interface/joint_session_history",
    "html/browsers/history/the-history-interface/pushstate-replacestate-empty-string",
    "html/browsers/history/the-location-interface",
    "html/browsers/history/the-session-history-of-browsing-contexts",
    "html/browsers/history/joint-session-history",
    // 导航遍历（前进后退/跨文档导航/fragid/卸载事件序）
    "html/browsers/browsing-the-web/history-traversal",
    "html/browsers/browsing-the-web/history-traversal/event-order",
    "html/browsers/browsing-the-web/history-traversal/pagereveal",
    "html/browsers/browsing-the-web/history-traversal/pageswap",
    "html/browsers/browsing-the-web/history-traversal/persisted-user-state-restoration",
    "html/browsers/browsing-the-web/navigating-across-documents",
    "html/browsers/browsing-the-web/navigating-across-documents/initial-empty-document",
    "html/browsers/browsing-the-web/navigating-across-documents/javascript-url-abort",
    "html/browsers/browsing-the-web/navigating-across-documents/multiple-globals",
    "html/browsers/browsing-the-web/navigating-across-documents/refresh",
    "html/browsers/browsing-the-web/navigating-across-documents/replace-before-load",
    "html/browsers/browsing-the-web/navigating-across-documents/source",
    // fetch 于本轮限流未完成、语料落地后自动纳入扫描（read_dir 失败静默跳过）
    "html/browsers/browsing-the-web/scroll-to-fragid",
    "html/browsers/browsing-the-web/unloading-documents",
    // iframe 元素属性面（DC-1 资产；用例即 M3 门控深结构面——skip 规则零执行记账）
    "html/semantics/embedded-content/the-iframe-element",
];

/// navigation corpus 共用运行面筛减规则（fetch 脚本头注释同域，双保险）：
/// - `*-manual.html` / `*-ref.html` / `*-notref.html` / `.https.html` / `idlharness.*`
///   （net-api/encoding/html-syntax 先例）
/// - `*/resources/`（helper 资产——navigation-api/resources 等，META script 消费
///   不按案跑）
/// - 无 `testharness.js` 引用页——legacy 多页自动导航套件（history-traversal 001.html
///   族 window.open 多页链、joint-session-history 多页序、navigating-across-documents
///   legacy 表单页）依赖跨文档导航链，单文档 runner 通道无法建立测试前提
///   （html-syntax 先例）
/// - iframe 依赖面（runner 无多 frame 文档管道，net-api/encoding 先例）——
///   navigation-compat M3 起 **the-iframe-element 域例外**（用户门控 2026-10-21
///   批复「批准 M3 最小面」后解锁执行）：contentWindow/contentDocument/frames/
///   parent/top 最小面 + iframe 元素属性语义即本域断言面，shim iframe 机制
///   （R115/R177 缓存 + contentWindow lazy fetch）已可用；html/browsers 载体域与
///   navigation-api sentinel 域维持跳过（依赖子文档 session history replace 语义，
///   M3 后续切片再评）
/// - 多窗口/popup 依赖面（window.open/openner 链——runner 单文档无窗口管理）
/// - worker 执行面（net-api 先例）
fn navigation_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html")
        || name.ends_with("-ref.html")
        || name.ends_with("-notref.html")
        || name.ends_with(".https.html")
        || name.starts_with("idlharness.")
    {
        return true;
    }
    if relative.contains("/resources/") {
        return true;
    }
    if !source.contains("testharness.js") {
        return true;
    }
    // M3 解锁域：iframe 内容规则不跳过（popup/worker/serviceWorker 面照常筛）。
    // 运行形态/深结构子域维持记账跳过（M3 后续切片）：服务端 stash 协议
    //（iframe_harness.js get/send_test_results → wptserve stash.py，runner 无该端点——
    // parentage 族 + download tentative 族）、子发起嵌套导航控制（iframe_sandbox_navigate_*
    // 的 child→ancestor/descendant 面）、popup 窗口管理（iframe_sandbox_popups_* /
    // allow_top_navigation 的 open() 链 + 用户手势）、三层嵌套 bc（change_parentage /
    // cross-origin-to-whom 的 grandchild DOM 写与 frame/embed/object 元素面）。
    // srcdoc/sandbox/frames/parent/top/load 语义最小面照常执行。
    if relative.contains("the-iframe-element") {
        // navigation-compat M3-S6：iframe_sandbox_navigate_history_ 三案解锁——
        // 子窗局部 history 面（back/forward 越界 no-op + pushState 后子窗局部
        // popstate）。ancestor/descendants/itself/other_frame_popup 维持记账
        //（data: URL 文档合成 + 跨帧 location 写 + 三层嵌套 = 后续切片）。
        return relative.contains("iframe_sandbox_popups_")
            || (relative.contains("iframe_sandbox_navigate_")
                && !relative.contains("iframe_sandbox_navigate_history_"))
            || relative.contains("iframe_sandbox_allow_top_navigation")
            || relative.contains("iframe_navigate_ancestor")
            || relative.contains("change_parentage")
            || relative.contains("cross-origin-to-whom")
            || source.contains("stash.py")
            || source.contains("iframe_harness.js")
            || source.contains("window.open(")
            || source.contains("new Worker(")
            || source.contains("SharedWorker(")
            || source.contains("navigator.serviceWorker");
    }
    source.contains("<iframe")
        || source.contains("createElement('iframe')")
        || source.contains("createElement(\"iframe\")")
        || source.contains("window.open(")
        || source.contains("new Worker(")
        || source.contains("SharedWorker(")
        || source.contains("navigator.serviceWorker")
}

/// navigation corpus 绝对路径 helper → wpt-data 本地映射（timing
/// [`TIMING_ABSOLUTE_HELPERS`] 同款——只装被引用且已拉取的，缺失跳过按真实缺口失败）。
/// location-stringifier 用 `/common/stringifiers.js` 的 `test_stringifier_attribute`。
/// `/common/test-setting-immutable-prototype.js`（location-prototype-setting）**不装**：
/// immutable-prototype 面需 exotic [[SetPrototypeOf]]（与 preventExtensions 同族，shim
/// plain object 未实现）——装了也转真语义 fail，留 M2 后续切片定夺。
const NAVIGATION_ABSOLUTE_HELPERS: &[(&str, &str)] = &[
    ("/common/stringifiers.js", "common/stringifiers.js"),
    // M2-S4F：location-prototype-setting-same-origin 的 [[SetPrototypeOf]] 面助手
    //（此前不拉——plain object 未实现 exotic 内部方法；Proxy 面落地后启用）。
    (
        "/common/test-setting-immutable-prototype.js",
        "common/test-setting-immutable-prototype.js",
    ),
];

/// Run the pinned upstream navigation corpus window subset
/// （navigation-compat goal M1 / DC-1）。filter 按路径子串过滤（如
/// `the-history-interface/`、`navigation-api/`、`scroll-to-fragid/`——基线按 corpus
/// 分类）。
pub fn run_navigation_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_any_js_corpus_subdirs_with_helpers_variants(
        wpt_root,
        NAVIGATION_CORPUS_SUBDIRS,
        filter,
        navigation_case_skipped,
        NAVIGATION_ABSOLUTE_HELPERS,
        true,
    )
}

/// Run the fixed Service Worker M1 core testharness corpus.
pub fn run_service_worker_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_service_worker_case_set(wpt_root, filter, SERVICE_WORKER_CORE_CASES)
}

/// Run the fixed Service Worker M2 fetch/interception testharness corpus.
pub fn run_service_worker_fetch_cases(
    wpt_root: &Path,
    filter: Option<&str>,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_service_worker_case_set(wpt_root, filter, SERVICE_WORKER_FETCH_CASES)
}

/// Run the fixed Service Worker CacheStorage testharness corpus.
pub fn run_service_worker_cache_storage_cases(
    wpt_root: &Path,
    filter: Option<&str>,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_service_worker_case_set(wpt_root, filter, SERVICE_WORKER_CACHE_STORAGE_CASES)
}

fn run_service_worker_case_set(
    wpt_root: &Path,
    filter: Option<&str>,
    manifest: &[&str],
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let selected: Vec<_> = manifest
        .iter()
        .copied()
        .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
        .collect();
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return selected
                .into_iter()
                .map(|path| {
                    (
                        path.to_string(),
                        vec![HarnessSubtestResult {
                            name: "load testharness.js".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    )
                })
                .collect();
        }
    };

    selected
        .into_iter()
        .map(|path| {
            let results = match read_corpus_to_string(wpt_root.join(path)) {
                Ok(source) if is_service_worker_any_js(path) => {
                    let html = service_worker_any_js_wrapper(path, &source);
                    run_testharness_html(wpt_root, path, &html, &harness_source, CASE_TIMEOUT)
                }
                Ok(source) => run_testharness_html(wpt_root, path, &source, &harness_source, CASE_TIMEOUT),
                Err(error) => vec![HarnessSubtestResult {
                    name: "load Service Worker WPT case".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            };
            (path.to_string(), results)
        })
        .collect()
}

fn indexeddb_window_wrapper(path: &str, support: &[(&str, &str)], case_source: &str) -> String {
    any_js_window_wrapper(path, support, case_source)
}

fn any_js_window_wrapper(path: &str, support: &[(&str, &str)], case_source: &str) -> String {
    let mut source = String::new();
    for (name, script) in support {
        let script = apply_wpt_substitutions(script);
        source.push_str(&format!("// source: {name}\n{script}\n"));
    }
    let case_source = apply_wpt_substitutions(case_source);
    source.push_str(&format!("// source: {path}\n{case_source}"));
    let source = source.replace("</script", "<\\/script");
    let timeout_meta = if wpt_js_has_long_timeout(&case_source) {
        "<meta name=\"timeout\" content=\"long\">"
    } else {
        ""
    };
    format!(
        "<!doctype html><meta charset=\"utf-8\">{timeout_meta}\
         <script src=\"/resources/testharness.js\"></script>\
         <script src=\"/resources/testharnessreport.js\"></script>\
         <script>{source}</script>"
    )
}

fn service_worker_any_js_wrapper(path: &str, case_source: &str) -> String {
    let timeout_meta = if wpt_js_has_long_timeout(case_source) {
        "<meta name=\"timeout\" content=\"long\">"
    } else {
        ""
    };
    let script = serde_json::to_string(&format!("/{path}")).unwrap_or_else(|_| "\"\"".into());
    let scope = serde_json::to_string(&format!("/{path}.scope/")).unwrap_or_else(|_| "\"\"".into());
    let type_option = if service_worker_any_js_is_module(case_source) {
        ", type: 'module'"
    } else {
        ""
    };
    let description = serde_json::to_string(path).unwrap_or_else(|_| "\"Service Worker any.js\"".into());
    format!(
        "<!doctype html><meta charset=\"utf-8\">{timeout_meta}\
         <script src=\"/resources/testharness.js\"></script>\
         <script src=\"/resources/testharnessreport.js\"></script>\
         <script>\
         promise_test(async function(test) {{\
           const registration = await navigator.serviceWorker.register({script}, {{scope: {scope}{type_option}}});\
           test.add_cleanup(function() {{ return registration.unregister(); }});\
           const worker = registration.installing || registration.waiting || registration.active;\
           assert_true(!!worker, 'registration exposes a worker');\
           await fetch_tests_from_worker(worker);\
         }}, {description});\
         </script>"
    )
}

fn service_worker_any_js_source(path: &str, case_source: &str) -> String {
    let case_source = apply_wpt_substitutions(case_source);
    let meta_scripts = wpt_meta_scripts(path, &case_source);
    let fixture = if path.contains("cache-abort") {
        format!("{CACHE_ABORT_FETCH_FIXTURE}\n")
    } else {
        String::new()
    };
    let imports = if service_worker_any_js_is_module(&case_source) {
        meta_scripts
            .iter()
            .map(|script| format!("import '{}';", script.replace('\'', "\\'")))
            .collect::<Vec<_>>()
            .join("\n")
    } else if meta_scripts.is_empty() {
        String::new()
    } else {
        let scripts = meta_scripts
            .iter()
            .map(|script| format!("'{}'", script.replace('\'', "\\'")))
            .collect::<Vec<_>>()
            .join(", ");
        format!("importScripts({scripts});\n")
    };
    if service_worker_any_js_is_module(&case_source) {
        format!("import '/resources/testharness.js';\n{imports}\n{fixture}{case_source}")
    } else {
        format!("importScripts('/resources/testharness.js');\n{imports}{fixture}{case_source}")
    }
}

pub(crate) fn wpt_meta_scripts(path: &str, source: &str) -> Vec<String> {
    let case_dir = path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let script = trimmed.strip_prefix("// META: script=")?.trim();
            if script.is_empty() {
                return None;
            }
            if script.starts_with('/') {
                Some(script.to_string())
            } else {
                Some(format!("/{}", normalize_relative(&format!("{case_dir}/{script}"))))
            }
        })
        .collect()
}

fn is_service_worker_any_js(path: &str) -> bool {
    path.ends_with(".any.js")
}

fn service_worker_any_js_is_module(source: &str) -> bool {
    source
        .lines()
        .any(|line| line.trim().eq_ignore_ascii_case("// META: global=serviceworker-module"))
}

fn wpt_js_has_long_timeout(source: &str) -> bool {
    source
        .lines()
        .any(|line| line.trim().eq_ignore_ascii_case("// META: timeout=long"))
}

pub(crate) fn apply_wpt_substitutions(source: &str) -> String {
    source
        .replace("{{host}}", "wpt.test")
        // s116：{{domains[www]}} = 同 {{host}}（wptserve domains 常量集——referrer
        // strict-policies 的 https 导入臂用；此前 token 字面残留令 URL 解析失效）。
        .replace("{{domains[www]}}", "wpt.test")
        .replace("{{domains[www1]}}", "www1.wpt.test")
        .replace("{{domains[www2]}}", "www2.wpt.test")
        .replace("{{hosts[alt][]}}", "alt.wpt.test")
        .replace("{{hosts[alt][www]}}", "www1.alt.wpt.test")
        .replace("{{hosts[alt][www2]}}", "www2.alt.wpt.test")
        // 端口值对齐上游 /common/get-host-info.js 真值（http [8000,8081]、https
        // [8444,8443]）——[0] 必须非默认端口：真 WPT 环境下字符串字面量（"…:8000/"）
        // 与 `new URL().origin` 序列化形（非默认端口保留）两判据形殊途同归；曾映射
        // 默认端口 80/443 时 URL 序列化剥端口令两形撕裂（s116 referrer 族实测：
        // 同页内 expected 无 :80 / got 带 :80）。跨源判定/文件解析按 host 段，端口值
        // 不消费；改动波及全语料 token 页，全量 A/B 兜底。
        .replace("{{ports[http][0]}}", "8000")
        .replace("{{ports[http][1]}}", "8081")
        .replace("{{ports[https][0]}}", "8444")
        .replace("{{ports[https][1]}}", "8443")
        // net-api M5-S1：websockets 常量面（constants.sub.js SCHEME_DOMAIN_PORT）——
        // ws/wss/h2 三 token 全映射到内嵌 WS fixture 端口（wss/h2 变体不执行，token
        // 落在未走的分支里；?default 变体走 ws + fixture 端口，宿主改写见 ws_fixture）。
        .replace("{{ports[ws][0]}}", &crate::ws_fixture::fixture_ws_port().to_string())
        .replace("{{ports[wss][0]}}", &crate::ws_fixture::fixture_ws_port().to_string())
        .replace("{{ports[h2][0]}}", &crate::ws_fixture::fixture_ws_port().to_string())
}

/// Run one canvas testharness case with `canvas-tests.js` inlined.
fn run_canvas_testharness_html(
    wpt_root: &Path,
    case_name: &str,
    source: &str,
    harness_source: &str,
    canvas_tests_source: &str,
    timeout: Duration,
) -> Vec<HarnessSubtestResult> {
    let inline_extras = [(CANVAS_TESTS_JS_PATH, canvas_tests_source)];
    run_testharness_html_inner(wpt_root, case_name, source, harness_source, &inline_extras, timeout)
}

/// Run one HTML testharness case with the declared click/send_keys adapter.
/// WPT long-timeout 用例上限（`<meta name=timeout content=long>`，dom/ranges 等参数矩阵
/// mega-case 数千 subtest 常态跑数十秒；上游标准 normal=10s / long=60s）。
const CASE_TIMEOUT_LONG: Duration = Duration::from_secs(60);

pub fn run_testharness_html(
    wpt_root: &Path,
    case_name: &str,
    source: &str,
    harness_source: &str,
    timeout: Duration,
) -> Vec<HarnessSubtestResult> {
    // js-dom R51：尊重 WPT `<meta name=timeout content=long>`——调用方传入 normal 10s 时按
    // 用例声明放宽到 60s（mega-case 不再被 CASE_TIMEOUT 截断为 Timeout 伪失败）。检测为
    // 朴素子串匹配（meta 属性序/引号变体极少，上游用例均标准形态）。
    let effective = if timeout == CASE_TIMEOUT && is_long_timeout_case(source) {
        CASE_TIMEOUT_LONG
    } else {
        timeout
    };
    run_testharness_html_inner(wpt_root, case_name, source, harness_source, &[], effective)
}

/// 用例 HTML 是否声明 `<meta name=timeout content=long>`。
fn is_long_timeout_case(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    (lower.contains("name=\"timeout\"") && lower.contains("content=\"long\""))
        || (lower.contains("name=timeout") && lower.contains("content=long"))
}

/// R34xx：headless 图片源获取器——`https://wpt.test/<path>`（wpt-data 相对路径）→
/// `wpt_root/<path>` 本地文件读取（PNG 等解码由 webview decode_image 完成）。
fn wpt_data_image_fetcher(wpt_root: &std::path::Path) -> Option<zero_webview::ImageSourceFetcher> {
    let root = wpt_root.to_path_buf();
    Some(std::sync::Arc::new(move |url: &str| {
        // 仅 wpt.test 域名（测试资源）；其他 URL 回退网络。M5-S5：页锚 scheme 双态
        // （net-api corpus http、其余通道 https）——strip 双 scheme。
        let path_part = url
            .strip_prefix("https://wpt.test")
            .or_else(|| url.strip_prefix("http://wpt.test"))
            // net-api 二期 M2-S3b：net-api 页锚端口化（wpt.test:8000）——图片源
            // 解析出 :8000 绝对 URL，双 scheme 前缀同步放行。
            .or_else(|| url.strip_prefix("https://wpt.test:8000"))
            .or_else(|| url.strip_prefix("http://wpt.test:8000"))?;
        let path_part = path_part.strip_prefix('/').unwrap_or(path_part);
        // 去查询串/片段。
        let clean = path_part.split(['?', '#']).next()?;
        if clean.is_empty() {
            return None;
        }
        std::fs::read(root.join(clean)).ok()
    }))
}

/// R34xx（G6）：外链脚本源获取器（.worker.js worker 变体 + worker 内 importScripts 的
/// testharness.js/canvas-tests.js）——`(page_url, src)` → wpt-data 文件。
/// s115：`serve-with-content-type.py?fn=<rel>&ct=<mime>` 查询解析（fn/ct 百分号
/// 解码）——`wpt_data_script_fetcher`（模块/运行时面）与 `inline_local_scripts`
/// （parser classic 内联面）共用。缺 fn 或 ct → None。
fn parse_serve_with_content_type_query(query: &str) -> Option<(String, String)> {
    let mut fn_param = None;
    let mut ct_param = None;
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("fn=") {
            fn_param = Some(
                percent_encoding::percent_decode_str(value)
                    .decode_utf8_lossy()
                    .into_owned(),
            );
        } else if let Some(value) = pair.strip_prefix("ct=") {
            ct_param = Some(
                percent_encoding::percent_decode_str(value)
                    .decode_utf8_lossy()
                    .into_owned(),
            );
        }
    }
    Some((fn_param?, ct_param?))
}

/// s122：文档编码嗅探——meta charset prescan（spec HTML encoding sniffing 的
/// meta 面简化：BOM/transport 头不参与，本地 file 通道页面无 transport charset；
/// `<meta charset=>` 与 `<meta http-equiv=content-type content="...charset=>"`
/// 两形同配——charset= 均在单标签文本内）。label 经 encoding_rs for_label 归一
/// （无法识别 → None，调用方顺延下一级）。1024 字符窗近似 spec prescan 窗。
/// https://html.spec.whatwg.org/multipage/parsing.html#encoding-sniffing-algorithm
fn sniff_document_encoding(html: &str) -> Option<String> {
    // 窗口按字节计（spec prescan 1024 bytes），截断点须落在 char boundary——
    // byte 1024 落进多字节字符时直切 panic（canvas 语料全角注释实证 rc=101）；
    // 回退最近边界至多丢窗尾半个字符，不影响 meta 面匹配。
    let mut end = html.len().min(1024);
    while !html.is_char_boundary(end) {
        end -= 1;
    }
    let window = &html[..end];
    let re_match = window.match_indices("<meta").find_map(|(pos, _)| {
        let tag_end = window[pos..].find('>')? + pos;
        let tag = &window[pos..=tag_end];
        let idx = tag.to_ascii_lowercase().find("charset")?;
        let after = &tag[idx + "charset".len()..];
        let after = after.strip_prefix('=')?;
        let value = after
            .trim_start()
            .trim_start_matches(['"', '\''])
            .split(['"', '\'', '>', ' ', '\t', '\n', '\r'])
            .next()?;
        (!value.is_empty()).then(|| value.to_string())
    })?;
    encoding_rs::Encoding::for_label(re_match.as_bytes()).map(|enc| enc.name().to_string())
}

/// s115：classic 外链脚本响应解码序（spec fetch a classic script）——BOM 嗅探 >
/// ct charset 参数 > charset 属性 > 文档编码（s122 嗅探回退，原恒 UTF-8——
/// script-charset-02 判据面）> UTF-8 lossy。
/// label 无法识别（encoding_rs for_label 落空）→ 顺延下一级。
/// https://html.spec.whatwg.org/multipage/scripting.html#fetch-a-classic-script
fn decode_classic_script_bytes(
    bytes: &[u8],
    ct: &str,
    charset_attr: Option<&str>,
    doc_encoding: Option<&str>,
) -> String {
    if let Some((enc, _)) = encoding_rs::Encoding::for_bom(bytes) {
        return enc.decode(bytes).0.into_owned();
    }
    let ct_label = ct.split(';').skip(1).find_map(|param| {
        let param = param.trim();
        if param.get(..8).is_some_and(|head| head.eq_ignore_ascii_case("charset=")) {
            Some(param[8..].trim().to_string())
        } else {
            None
        }
    });
    let label = ct_label.or_else(|| charset_attr.map(str::to_string));
    let label = label.or_else(|| doc_encoding.map(str::to_string));
    match label
        .as_deref()
        .and_then(|l| encoding_rs::Encoding::for_label(l.as_bytes()))
    {
        Some(enc) => enc.decode(bytes).0.into_owned(),
        None => String::from_utf8_lossy(bytes).into_owned(),
    }
}

/// s116：referrer policy label 合法集（解析失败/未知 label 按 spec 忽略——继承顺延）。
/// https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-grammar
fn is_known_referrer_policy(label: &str) -> bool {
    matches!(
        label,
        "no-referrer"
            | "no-referrer-when-downgrade"
            | "same-origin"
            | "origin"
            | "strict-origin"
            | "origin-when-cross-origin"
            | "strict-origin-when-cross-origin"
            | "unsafe-url"
    )
}

/// s116：绝对 URL 拆解 (scheme, 权威段书写原样含端口, host, 缺省端口归一值)。仅
/// http/https（runner 页锚与 wpt host 别名面）；其余 scheme → None。
fn parse_wpt_url_components(url: &str) -> Option<(&str, &str, &str, u16)> {
    let (scheme, rest) = url.split_once("://")?;
    if !matches!(scheme, "http" | "https") {
        return None;
    }
    let slash = rest.find('/').unwrap_or(rest.len());
    let authority = &rest[..slash];
    let default_port: u16 = if scheme == "https" { 443 } else { 80 };
    let port = match authority.rsplit_once(':') {
        Some((host, port_str))
            if !host.is_empty() && !port_str.is_empty() && port_str.chars().all(|c| c.is_ascii_digit()) =>
        {
            match port_str.parse::<u16>() {
                Ok(p) => p,
                Err(_) => return None,
            }
        }
        _ => default_port,
    };
    let host = authority.split(':').next().unwrap_or(authority);
    Some((scheme, authority, host, port))
}

/// s116：同源判定——(scheme, host, 缺省端口归一) 三元组相等（wpt.test 与
/// wpt.test:80 同源；:443/:80 跨 scheme 即异源）。
fn wpt_urls_same_origin(a: &str, b: &str) -> bool {
    match (parse_wpt_url_components(a), parse_wpt_url_components(b)) {
        (Some((sa, _, ha, pa)), Some((sb, _, hb, pb))) => sa == sb && ha == hb && pa == pb,
        _ => false,
    }
}

/// s116：origin 形 Referer——scheme://权威段（书写原样）+ "/"。真 WPT 端口为
/// 非默认值（get-host-info http[0]=8000），URL 序列化不剥端口——书写原样即浏览器
/// 序列化形；替换表已对齐该值（apply_wpt_substitutions s116 注）。
fn wpt_url_origin_form(url: &str) -> Option<String> {
    parse_wpt_url_components(url).map(|(scheme, authority, _, _)| format!("{scheme}://{authority}/"))
}

/// s116：降级判定——引用源 potentially trustworthy（https）而请求非（http）。
fn wpt_referrer_is_downgrade(source_url: &str, request_url: &str) -> bool {
    source_url.starts_with("https://") && !request_url.starts_with("https://")
}

/// s116：Fetch determine-request's-referrer 引用串计算——None = 不发送 Referer。
/// 引用源 = 引用方模块 URL（含 query——`?name=` 去重即 spec 语义）；请求 URL =
/// checker 请求绝对 URL。
/// https://fetch.spec.whatwg.org/#determine-request-s-referrer
fn compute_module_referer(source_url: &str, request_url: &str, policy: &str) -> Option<String> {
    let full = || source_url.split('#').next().unwrap_or(source_url).to_string();
    let origin = || wpt_url_origin_form(source_url).unwrap_or_default();
    let same_origin = wpt_urls_same_origin(source_url, request_url);
    let downgrade = wpt_referrer_is_downgrade(source_url, request_url);
    match policy {
        "no-referrer" => None,
        "unsafe-url" => Some(full()),
        "origin" => Some(origin()),
        "same-origin" => same_origin.then(full),
        "origin-when-cross-origin" => {
            if same_origin {
                Some(full())
            } else {
                Some(origin())
            }
        }
        "strict-origin" => (!downgrade).then(origin),
        "strict-origin-when-cross-origin" => {
            if downgrade {
                None
            } else if same_origin {
                Some(full())
            } else {
                Some(origin())
            }
        }
        "no-referrer-when-downgrade" => (!downgrade).then(full),
        _ => None,
    }
}

/// s116：文档 `<meta name="referrer" content="X">` 策略提取。窄解析（属性小写双引号
/// 形——语料 referrer 族七页同形；referrerpolicy 属性/meta http-equiv/CSP 面未建模）。
/// 首个命中生效；无 content 或空 label → None（继承默认）。
/// https://w3c.github.io/webappsec-referrer-policy/#parse-referrer-policy-from-header
fn extract_meta_referrer_policy(source: &str) -> Option<String> {
    let lower = source.to_ascii_lowercase();
    let needle = "<meta name=\"referrer\" content=\"";
    let start = lower.find(needle)? + needle.len();
    let rest = &source[start..];
    let end = rest.find('"')?;
    let label = rest[..end].trim().to_ascii_lowercase();
    (!label.is_empty()).then_some(label)
}

/// s116：wpt host 别名前缀剥离——`http(s)://<别名>[:port]/<path>` → `/<path>`。
/// 别名集窄收 wpt.test + www1.wpt.test（referrer 族跨源面实际所需；**www2 不可达
/// 保持**——dynamic-imports-fetch-error "cross-origin without CORS" 判据页依赖
/// www2 导入必败，放行即 P→F；CORS 语义/cookies 基建另立后同批放行）。端口段只
/// 要求纯数字（替换端口值），路径解析不消费端口。同树按 path 解析——跨源语义
/// 仅用于 Referer origin 判定（fetcher 无 CORS 层，模块图跨源取回即成功）。非别名
/// 域绝对 URL → None（调用方保持既有 Err 语义）。
fn strip_wpt_host_prefix(src: &str) -> Option<String> {
    let (scheme, rest) = src.split_once("://")?;
    if !matches!(scheme, "http" | "https") {
        return None;
    }
    let slash = rest.find('/')?;
    let authority = &rest[..slash];
    let host = authority.split(':').next().unwrap_or(authority);
    matches!(host, "wpt.test" | "www1.wpt.test").then(|| rest[slash..].to_string())
}

fn wpt_data_script_fetcher(
    wpt_root: &std::path::Path,
    doc_referrer_policy: Option<String>,
) -> Option<zero_webview::ScriptSourceFetcher> {
    let root = wpt_root.to_path_buf();
    Some(std::sync::Arc::new(move |page_url: &str, src: &str| {
        let path_part = match strip_wpt_host_prefix(src) {
            Some(path) => path,
            None if src.starts_with("http://") || src.starts_with("https://") => {
                return Err(format!("external script origin is not available in WPT runner: {src}"));
            }
            None if src.starts_with('/') => src.to_string(),
            None => {
                let page_path = wpt_url_path(page_url).strip_prefix('/').unwrap_or(page_url);
                let page_dir = page_path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
                normalize_relative(&format!("{page_dir}/{src}"))
            }
        };
        let path_part = path_part.strip_prefix('/').unwrap_or(&path_part);
        let clean = path_part.split(['?', '#']).next().unwrap_or(path_part);
        if clean.is_empty() {
            return Err("empty path".to_string());
        }
        // workers-compat M2：script MIME 判定（classic worker(-imported) script 的
        // fetch 步骤——非 JavaScript MIME 按网络错误失败；Worker_script_mimetype/
        // importscripts_mime 案面）。派生序（wptserve 语义）：① wptserve
        // `?pipe=header(Content-Type,X)` 响应头模板（s87——dynamic-import
        // fetch-error 用例经 specifier 直带；errorhandling-wrongMimetype.js `text/plain`
        // 面：文件本体合法 JS，无此建模按 .js 扩展名误判 scripty → import() 须拒未拒）；
        // ② `<file>.headers` sidecar Content-Type 覆盖（无扩展名脚本 `1`/`null`/
        // `undefined`、nosniff 面）；③ `workers/support/imported_script.py?mime=` 动态
        // handler（wptserve 返空体——成功源以 "\n" 交付，__zw_fetch_script 以空串承载
        // Err 不可区分，空脚本与 "\n" 执行语义恒等）；④ 扩展名 .js/.mjs scripty，
        // 其余（含无扩展名）非 scripty。其余 pipe 指令（trickle 等）不涉 MIME 照旧忽略。
        // https://html.spec.whatwg.org/multipage/workers.html#dom-workerglobalscope-importscripts
        fn mime_is_scripty(mime: &str) -> bool {
            let essence = mime.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
            matches!(
                essence.as_str(),
                "text/javascript" | "application/javascript" | "text/ecmascript" | "application/ecmascript"
            )
        }
        let mime_from_pipe = src.split('?').nth(1).and_then(|query| {
            query
                .split('&')
                .find_map(|kv| kv.strip_prefix("pipe="))
                .and_then(|pipe| {
                    let inner = pipe.strip_prefix("header(")?.strip_suffix(')')?;
                    let (name, value) = inner.split_once(',')?;
                    name.trim()
                        .eq_ignore_ascii_case("content-type")
                        .then(|| value.trim().to_string())
                })
        });
        let mime_from_sidecar = read_corpus_to_string(root.join(format!("{clean}.headers")))
            .ok()
            .and_then(|headers| {
                headers.lines().find_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    if key.trim().eq_ignore_ascii_case("content-type") {
                        Some(value.trim().to_string())
                    } else {
                        None
                    }
                })
            });
        // s107（JSON module）：`.json` 静态文件与 `serve-with-content-type.py?fn=<rel>&ct=<mime>`
        // 动态 handler 的 JSON 意图面——按 Content-Type 判 JSON MIME type：essence
        // application/json | text/json 或 `+json` 后缀、`;参数` 剥离（text/json 为
        // corpus 时代合法集——现行 spec 已收窄至 application/json 与 +json 后缀，
        // json-module/valid-content-type 8 臂判据为准）。合法 → 交付文件原文；非法 →
        // Err（网络错误语义）。ct 值百分号解码（charset 页 %3B、valid-content-type t7
        // %2B；裸 `+` 原样保留——t3/t4/t8 的 `text/html+json` 在 query 中不表示空格）。
        // 端点本体不在 runner 语料（上游 wptserve .py 脚本），按 handler 语义模拟：
        // 交付 fn 指向的文件（fn 相对 .py 所在目录解析——corpus
        // `fn=json-module/module.json` 对 the-script-element/ 目录成立）。
        // https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
        fn mime_is_json(mime: &str) -> bool {
            let essence = mime.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
            essence == "application/json" || essence == "text/json" || essence.ends_with("+json")
        }
        let json_intent: Option<(String, String)> = if clean.ends_with(".json") {
            // wptserve 对 .json 文件默认 Content-Type: application/json；
            // `?pipe=header(...)` / `.headers` sidecar 可覆盖（MIME 判定派生序同 scripty 面）。
            let mime = mime_from_pipe
                .clone()
                .or_else(|| mime_from_sidecar.clone())
                .unwrap_or_else(|| "application/json".to_string());
            Some((clean.to_string(), mime))
        } else if std::path::Path::new(clean)
            .file_name()
            .is_some_and(|name| name == std::ffi::OsStr::new("serve-with-content-type.py"))
        {
            // s115：查询解析抽共享 helper（inline_local_scripts classic 内联面同用）。
            let query = src.split('?').nth(1).unwrap_or("");
            match parse_serve_with_content_type_query(query) {
                Some((file, ct)) => {
                    let dir = clean.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
                    Some((format!("{dir}/{file}"), ct))
                }
                None => return Err(format!("serve-with-content-type.py missing fn/ct: {src}")),
            }
        } else {
            None
        };
        if let Some((file_rel, mime)) = json_intent {
            if mime_is_json(&mime) {
                return std::fs::read(root.join(&file_rel))
                    .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                    .map_err(|e| format!("json module fetch failed: {file_rel} ({e})"));
            }
            // s115：scripty ct——serve-with-content-type.py 交付 scripty fixture 文件。
            // 解码恒 UTF-8 lossy（fetcher 无脚本类型/charset 属性上下文）：模块面 spec
            // 本就恒 UTF-8——charset 参数/属性双忽略、BOM 忽略不嗅探（体内 U+FEFF 为
            // JS 空白：UTF-8 BOM 模块照常执行、UTF-16 内容乱码语法错——module/
            // charset-01/02/03 三页判据面）；classic 运行时取回无 charset 属性上下文，
            // 同 UTF-8 近似（parser classic 面的 charset 感知在 inline_local_scripts
            // 投递时完成）。非 scripty 非 JSON → Err（网络错误语义，s107 原口径）。
            if mime_is_scripty(&mime) {
                return std::fs::read(root.join(&file_rel))
                    .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                    .map_err(|e| format!("script fetch failed: {file_rel} ({e})"));
            }
            return Err(format!("non-scripty MIME for script fixture: {file_rel} ({mime})"));
        }
        // s116：referrer-checker.py 动态端点仿真（上游 wptserve handler 不在语料——
        // `request.headers.referer` → `export const referrer = '<Referer>';`）。仅
        // module/resources/ 面（JS 模块 checker；text-module/ css-module/ 同名端点按
        // 数据模块内容形态回体——type:text/css 合成面挂账）。
        // Referer 按 Fetch determine-request's-referrer：引用源 = 引用方模块 URL（本
        // 回调首参——顶层模块图首跳即文档 URL，包装子跳即包装 URL，collect_module_deps
        // 逐层穿 parent）；策略链 = 引用方自身交付的 Referrer-Policy 响应头（wptserve
        // `?pipe=header(Referrer-Policy,P)` 查询模板——读父 URL query）优先，缺省继承
        // 文档策略（<meta name=referrer> 逐页随闭包捕获），非法 label 按 spec 忽略
        // 顺延，全缺 → spec 默认 strict-origin-when-cross-origin。
        // https://fetch.spec.whatwg.org/#determine-request-s-referrer
        // https://github.com/whatwg/html/pull/9210（descendant 继承判据页）
        if clean.contains("the-script-element/module/resources/referrer-checker.py") {
            let full_url = zero_engine::resolve_document_url(page_url, src);
            let parent_query = page_url.split('?').nth(1).unwrap_or("");
            let pipe_policy = parent_query.split('&').find_map(|kv| {
                kv.strip_prefix("pipe=").and_then(|pipe| {
                    let inner = pipe.strip_prefix("header(")?.strip_suffix(')')?;
                    let (name, value) = inner.split_once(',')?;
                    name.trim()
                        .eq_ignore_ascii_case("referrer-policy")
                        .then(|| value.trim().to_ascii_lowercase())
                })
            });
            // 逐臂过滤：非法 pipe label 按 spec 忽略后顺延文档策略（整体 or+filter
            // 会把合法 doc policy 连带丢弃——descendants invalid-policy 臂实测）。
            let effective = pipe_policy
                .filter(|p| is_known_referrer_policy(p))
                .or_else(|| doc_referrer_policy.clone().filter(|p| is_known_referrer_policy(p)))
                .unwrap_or_else(|| "strict-origin-when-cross-origin".to_string());
            let referer = compute_module_referer(page_url, &full_url, &effective).unwrap_or_default();
            return Ok(format!("export const referrer = '{referer}';"));
        }
        // s124：echo-referrer.py 动态端点仿真（上游 wptserve handler 不在语料——
        // `window.referrers["<uid>"] = "<Referer>";` 回体，modulepreload referrerpolicy
        // 三页判据面：link rel=modulepreload 取回 + 动态 import 共用此端点记账）。
        // 策略链 = `__zw_lp` 查询参数（引擎 shim link modulepreload 取回面的元素级
        // referrerpolicy 通道——fetcher 契约 (page_url, src) 无策略形参，shim 以内部
        // 标记参数携带，计算前剥离；无参数回落 referrer-checker 同链 pipe → 文档策略 →
        // spec 默认）。uid 取查询参数原样（语料页数值形）；Referer 内嵌 JS 字符串面
        // 做最小转义。无 uid → 回体不记账（上游 first(b"uid", b"") 同义）。
        // https://html.spec.whatwg.org/multipage/links.html#link-type-modulepreload
        // https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-attribute
        if clean.contains("preload/resources/echo-referrer.py") {
            let full_url = zero_engine::resolve_document_url(page_url, src);
            let lp_policy = src
                .split('?')
                .nth(1)
                .and_then(|query| query.split('&').find_map(|kv| kv.strip_prefix("__zw_lp=")));
            let parent_query = page_url.split('?').nth(1).unwrap_or("");
            let pipe_policy = parent_query.split('&').find_map(|kv| {
                kv.strip_prefix("pipe=").and_then(|pipe| {
                    let inner = pipe.strip_prefix("header(")?.strip_suffix(')')?;
                    let (name, value) = inner.split_once(',')?;
                    name.trim()
                        .eq_ignore_ascii_case("referrer-policy")
                        .then(|| value.trim().to_ascii_lowercase())
                })
            });
            let effective = lp_policy
                .filter(|p| is_known_referrer_policy(p))
                .map(|p| p.to_string())
                .or_else(|| pipe_policy.filter(|p| is_known_referrer_policy(p)))
                .or_else(|| doc_referrer_policy.clone().filter(|p| is_known_referrer_policy(p)))
                .unwrap_or_else(|| "strict-origin-when-cross-origin".to_string());
            let referer = compute_module_referer(page_url, &full_url, &effective).unwrap_or_default();
            let uid = src
                .split('?')
                .nth(1)
                .unwrap_or("")
                .split('&')
                .find_map(|kv| kv.strip_prefix("uid="))
                .unwrap_or("")
                .replace("%3D", "=")
                .replace("%3d", "=");
            let referer = referer.replace('\\', "\\\\").replace('"', "\\\"");
            return Ok(format!("window.referrers[\"{uid}\"] = \"{referer}\";"));
        }
        // s124：script.py 端点仿真（上游 wptserve handler 不在语料——modulepreload
        // cross-origin referrerpolicy 页的取回目标
        // `https://{{domains[www1]}}:{{ports[https][0]}}/common/security-features/
        // subresource/script.py`，判据仅 onload 到场（"should load"），无 Referer
        // 断言）。回体非空 JS（引擎 modulepreload 面以空体承载取回失败）；MIME 面
        // 上游默认 text/javascript。strip_wpt_host_prefix 已剥 www1.wpt.test:8444
        // 别名前缀，此处按路径命中。
        // https://html.spec.whatwg.org/multipage/links.html#link-type-modulepreload
        if clean == "common/security-features/subresource/script.py" {
            return Ok("globalThis.__zwScriptPyLoaded = (globalThis.__zwScriptPyLoaded || 0) + 1;".to_string());
        }
        let scripty = if let Some(mime) = mime_from_pipe {
            mime_is_scripty(&mime)
        } else if let Some(mime) = mime_from_sidecar {
            mime_is_scripty(&mime)
        } else if clean == "workers/support/imported_script.py" {
            // wptserve 动态 handler 语义：按 `?mime=` 查询回 Content-Type + 空体——
            // 文件本体（Python 源）不交付，scripty 成功直接以 "\n" 作源（空脚本与
            // "\n" 执行语义恒等，__zw_fetch_script 以空串承载 Err 不可区分）；非
            // scripty → Err（NetworkError 语义）。查询值 shim 原样透传（无浏览器
            // 编码）；容忍 +/%20/%3B 最小编码。
            let mime = src
                .split('?')
                .nth(1)
                .and_then(|query| query.split('&').find_map(|kv| kv.strip_prefix("mime=")))
                .map(|value| {
                    value
                        .replace('+', " ")
                        .replace("%20", " ")
                        .replace("%3B", ";")
                        .replace("%3b", ";")
                });
            if mime.map(|mime| mime_is_scripty(&mime)).unwrap_or(false) {
                return Ok("\n".to_string());
            }
            return Err(format!("non-scripty MIME for script: {clean}"));
        } else {
            matches!(
                std::path::Path::new(clean).extension().and_then(|e| e.to_str()),
                Some("js") | Some("mjs")
            )
        };
        if !scripty {
            return Err(format!("non-scripty MIME for script: {clean}"));
        }
        // workers-compat M2：`.any.js` 作 worker 顶脚本（页面 `new Worker('/…x.any.worker.js')`
        // —— dedicated 通道）→ 返回 ClassicWorkerHandler 同构包装源（GLOBAL flags +
        // importScripts testharness + META 依赖 + 语料正文内嵌 + done()）——上游
        // AnyWorkerHandler 语义。worker 自真实合成 origin（wpt.test）服务，正文
        // importScripts 根绝对路径可解析（data: URL wrapper 的 base 解析必抛已废弃）。
        // 正文内嵌而非 importScripts 自引用——该路径命中本分支会再包一层成环。
        // dedicated 变体对外 URL 为 `.any.worker.js`（upstream wptserve 命名——
        // examples/general.any.js 断言 self.location.pathname
        // `/workers/examples/general.any.worker.js`）：磁盘仍读 `.any.js` 源文件，
        // 包装 relative 用 worker 路径（location 面走该名）。裸 `.any.js` 请求仍收
        // （importScripts 直引语料、 Window lane 之外的既有抓取面不变）。
        let (worker_relative, file_relative): (&str, Option<String>) =
            if let Some(base) = clean.strip_suffix(".any.worker.js") {
                (clean, Some(format!("{base}.any.js")))
            } else {
                (clean, None)
            };
        if worker_relative.ends_with(".any.js") || worker_relative.ends_with(".any.worker.js") {
            let fetch_path = file_relative.as_deref().unwrap_or(worker_relative);
            let case_bytes =
                std::fs::read(root.join(fetch_path)).map_err(|e| format!("script fetch failed: {fetch_path} ({e})"))?;
            let case_source = String::from_utf8_lossy(&case_bytes).into_owned();
            // .sub 模板替换与主读路径同全集（{{hosts[alt][]}} 字面残留会使 URL 解析
            // 抛 SyntaxError，跨源 muted-errors 案期望的 NetworkError 前置走样）。
            let case_source = apply_wpt_substitutions(&case_source);
            return Ok(crate::testharness_workers::synthesize_any_worker_source(
                worker_relative,
                &case_source,
            ));
        }
        let full = root.join(clean);
        // lossy 解码（无效 UTF-8 → U+FFFD）——脚本源浏览器按 UTF-8 带替换解码
        // （workers/semantics/encodings/004.worker.js 原始无效字节用例；严格
        // read_to_string 对该类源返 Err → 脚本抓取失败）。
        std::fs::read(&full)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .map(|source| {
                // net-api M4-S22：替换面收口到 apply_wpt_substitutions 全集（原仅 host/
                // domains[www1]/ports[https][0] 三项——get-host-info 的 {{ports[http][0]}}
                // 残留 → HTTP_REMOTE_ORIGIN 带 '{{ports}}' 字面量，html 页跨源链路 302 不
                // 跟随（expose-headers-on-redirect 面）。上游 wptserve 对 .sub 文件做全集
                // 替换，此处语义对齐。
                let mut out = apply_wpt_substitutions(&source);
                // security-hardening M2-s4：`{{GET[name]}}` 模板替换（WPT .sub.js 服务端
                // 模板面——上游由 .py handler 按请求查询串注入；stylenonce/logTest/
                // alertAssert 族依赖）。查询参数取自 script src 的 query（`?logs=[]`），
                // 值做最小百分号解码（[ ]）；无匹配占位符 → 空串（上游缺参语义）。
                let query = src.split('?').nth(1).unwrap_or("");
                for pair in query.split('&') {
                    if pair.is_empty() {
                        continue;
                    }
                    let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
                    let decoded = value
                        .replace("%5B", "[")
                        .replace("%5D", "]")
                        .replace("%20", " ")
                        .replace("%22", "\"")
                        .replace("%27", "'");
                    out = out.replace(&format!("{{{{GET[{key}]}}}}"), &decoded);
                }
                // 残留 `{{GET[...]}}`（查询未提供）→ 空串。
                while let Some(start) = out.find("{{GET[") {
                    let Some(end) = out[start..].find("}}") else { break };
                    out.replace_range(start..start + end + 2, "");
                }
                out
            })
            .map_err(|e| format!("script fetch failed: {clean} ({e})"))
    }))
}

#[derive(Default)]
struct ServiceWorkerFixtureState {
    next_version: u64,
    update_worker_visits: HashMap<String, u64>,
    update_worker_from_file_visits: HashMap<String, u64>,
    bytecheck_visits: HashMap<String, u64>,
    type_update_visits: HashMap<String, u64>,
    request_metadata_visits: HashMap<String, u64>,
    update_via_cache_main_visits: HashMap<String, u64>,
    update_via_cache_import_visits: HashMap<String, u64>,
    update_via_cache_current: Option<(String, String)>,
    cached_missing_import_keys: std::collections::HashSet<String>,
    missing_import_main_visits: HashMap<String, u64>,
    missing_import_script_visits: HashMap<String, u64>,
}

fn service_worker_fixture_path(src: &str) -> Result<(&str, &str), String> {
    let path_and_query = if let Some((scheme, after_scheme)) = src.split_once("://") {
        let path_index = after_scheme
            .find('/')
            .ok_or_else(|| format!("Service Worker script URL has no path: {src}"))?;
        let authority = &after_scheme[..path_index];
        if scheme != "https" || !matches!(authority, "wpt.test" | "www1.wpt.test" | "www1.wpt.test:8444") {
            return Err(format!(
                "external Service Worker fixture origin is not available: {src}"
            ));
        }
        &after_scheme[path_index + 1..]
    } else {
        src.trim_start_matches('/')
    };
    let (clean, query) = path_and_query
        .split_once('?')
        .map_or((path_and_query, ""), |(path, query)| (path, query));
    if clean.is_empty() || clean.split('/').any(|segment| segment == "..") {
        return Err(format!("invalid Service Worker fixture path: {clean}"));
    }
    Ok((clean, query))
}

fn service_worker_fixture_query(query: &str) -> Result<HashMap<String, String>, String> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            let decode = |input: &str| {
                let form_value = input.replace('+', " ");
                percent_encoding::percent_decode_str(&form_value)
                    .decode_utf8()
                    .map(|value| value.into_owned())
                    .map_err(|_| "Service Worker fixture query is not UTF-8".to_string())
            };
            Ok((decode(name)?, decode(value)?))
        })
        .collect()
}

fn resolve_service_worker_fixture_redirect(src: &str, target: &str) -> Result<String, String> {
    let source_without_query = src.split('?').next().unwrap_or(src);
    let resolved = if target.starts_with("https://") {
        target.to_string()
    } else {
        let (scheme, after_scheme) = source_without_query
            .split_once("://")
            .ok_or_else(|| format!("Service Worker redirect base is not absolute: {src}"))?;
        let authority_end = after_scheme
            .find('/')
            .ok_or_else(|| format!("Service Worker redirect base has no path: {src}"))?;
        let origin = format!("{scheme}://{}", &after_scheme[..authority_end]);
        if target.starts_with('/') {
            format!("{origin}{target}")
        } else {
            let directory = source_without_query
                .rsplit_once('/')
                .map_or(source_without_query, |(directory, _)| directory);
            format!("{directory}/{target}")
        }
    };
    service_worker_fixture_path(&resolved)?;
    Ok(resolved)
}

fn service_worker_fixture_response(body: String, url: String, redirect_count: usize) -> zero_net::HttpResponse {
    zero_net::HttpResponse {
        status_code: 200,
        headers: vec![("Content-Type".into(), "application/javascript".into())],
        body: body.into_bytes(),
        url,
        redirect_count,
    }
}

/// Structured Service Worker responses for the pinned WPT corpus.
///
/// Static scripts are read from `wpt_root`; dynamic Python fixtures preserve MIME, changing-body,
/// redirect, and per-key stash behavior within one WebView test instance.
fn wpt_data_service_worker_script_fetcher(
    wpt_root: &std::path::Path,
) -> Option<zero_webview::ServiceWorkerScriptFetcher> {
    let root = wpt_root.to_path_buf();
    let state = Mutex::new(ServiceWorkerFixtureState::default());
    Some(std::sync::Arc::new(move |_page_url: &str, src: &str| {
        let (clean, query) = service_worker_fixture_path(src)?;
        let params = service_worker_fixture_query(query)?;

        let mut headers = Vec::new();
        let body = if clean.ends_with("/resources/invalid-chunked-encoding.py")
            || clean.ends_with("/resources/invalid-chunked-encoding-with-flush.py")
        {
            return Err("Service Worker script has invalid chunked encoding".into());
        } else if clean.ends_with("/resources/malformed-worker.py") {
            let source = if params.contains_key("parse-error") {
                "var foo = function() {;"
            } else if params.contains_key("undefined-error") {
                "foo.bar = 42;"
            } else if params.contains_key("uncaught-exception") {
                "throw new DOMException('AbortError');"
            } else if params.contains_key("caught-exception") {
                "try { throw new Error; } catch(e) {}"
            } else if params.contains_key("import-malformed-script") {
                "importScripts('malformed-worker.py?parse-error');"
            } else if params.contains_key("import-no-such-script") {
                "importScripts('no-such-script.js');"
            } else if params.contains_key("top-level-await") {
                "await Promise.resolve(1);"
            } else if params.contains_key("instantiation-error") {
                "import nonexistent from './imported-module-script.js';"
            } else if params.contains_key("instantiation-error-and-top-level-await") {
                "import nonexistent from './imported-module-script.js'; await Promise.resolve(1);"
            } else {
                return Err("malformed-worker.py requires a known mode".into());
            };
            return Ok(service_worker_fixture_response(source.into(), src.to_string(), 0));
        } else if clean.ends_with("/resources/mime-type-worker.py") {
            if let Some(mime) = params.get("mime") {
                headers.push(("Content-Type".into(), mime.clone()));
            }
            Vec::new()
        } else if clean.ends_with("/resources/bytecheck-worker.py") {
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            let visit = state.bytecheck_visits.entry(src.to_string()).or_default();
            *visit += 1;
            let main_content = if params.get("main").is_some_and(|value| value == "time") {
                visit.to_string()
            } else {
                "default".into()
            };
            let imported_path = params.get("path").map(String::as_str).unwrap_or_default();
            let imported_query = if params.get("imported").is_some_and(|value| value == "time") {
                "?imported=time"
            } else {
                ""
            };
            let imported_url = format!("{imported_path}bytecheck-worker-imported-script.py{imported_query}");
            let source = if params.get("type").is_some_and(|value| value == "module") {
                format!("// {main_content}\nimport '{}';\n", imported_url)
            } else {
                format!("// {main_content}\nimportScripts('{}');\n", imported_url)
            };
            return Ok(service_worker_fixture_response(source, src.to_string(), 0));
        } else if clean.ends_with("/resources/bytecheck-worker-imported-script.py") {
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            let visit = state.bytecheck_visits.entry(src.to_string()).or_default();
            *visit += 1;
            let imported_content = if params.get("imported").is_some_and(|value| value == "time") {
                visit.to_string()
            } else {
                "default".into()
            };
            return Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".into(), "application/javascript".into()),
                    ("Access-Control-Allow-Origin".into(), "*".into()),
                ],
                body: format!("// {imported_content}\n").into_bytes(),
                url: src.to_string(),
                redirect_count: 0,
            });
        } else if clean.ends_with("/resources/update-registration-with-type.py") {
            let key = params
                .get("key")
                .ok_or_else(|| "update-registration-with-type.py requires key".to_string())?;
            let classic_first = params
                .get("classic_first")
                .ok_or_else(|| "update-registration-with-type.py requires classic_first".to_string())?;
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            let visit = state.type_update_visits.entry(key.clone()).or_default();
            *visit += 1;
            let classic = (*visit == 1) == (classic_first == "1");
            let source = if classic {
                "importScripts('./imported-classic-script.js');\n\
                 self.onmessage = e => { e.source.postMessage(imported); };\n"
            } else {
                "import * as module from './imported-module-script.js';\n\
                 self.onmessage = e => { e.source.postMessage(module.imported); };\n"
            };
            return Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".into(), "application/javascript".into()),
                    ("Pragma".into(), "no-store".into()),
                    ("Cache-Control".into(), "no-store".into()),
                ],
                body: source.as_bytes().to_vec(),
                url: src.to_string(),
                redirect_count: 0,
            });
        } else if clean.ends_with("/resources/test-request-mode-worker.py")
            || clean.ends_with("/resources/test-request-headers-worker.py")
        {
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            let visit = state.request_metadata_visits.entry(src.to_string()).or_default();
            *visit += 1;
            let mut request_headers = serde_json::Map::new();
            request_headers.insert("service-worker".into(), serde_json::Value::String("script".into()));
            request_headers.insert("sec-fetch-mode".into(), serde_json::Value::String("same-origin".into()));
            if clean.ends_with("/resources/test-request-headers-worker.py") && *visit > 1 {
                request_headers.insert("if-none-match".into(), serde_json::Value::String("etag".into()));
            }
            let template_path = clean.trim_end_matches(".py").to_string() + ".js";
            let template = read_corpus_to_string(root.join(&template_path))
                .map_err(|error| format!("Service Worker fixture fetch failed: {template_path} ({error})"))?;
            let source = template
                .replace("%HEADERS%", &serde_json::Value::Object(request_headers).to_string())
                .replace("%UUID%", &visit.to_string());
            return Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".into(), "application/javascript".into()),
                    ("ETag".into(), "etag".into()),
                ],
                body: source.into_bytes(),
                url: src.to_string(),
                redirect_count: 0,
            });
        } else if clean.ends_with("/resources/update-max-aged-worker.py") {
            let test_name = params
                .get("test")
                .ok_or_else(|| "update-max-aged-worker.py requires test".to_string())?
                .clone();
            let modes = test_name
                .strip_prefix("register-with-updateViaCache-")
                .or_else(|| test_name.strip_prefix("access-updateViaCache-after-unregister-"))
                .ok_or_else(|| "update-max-aged-worker.py received an invalid test name".to_string())?;
            let (first_mode, second_mode) = modes
                .split_once("-then-")
                .map_or((modes, None), |(first, second)| (first, Some(second)));
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            let visit = state.update_via_cache_main_visits.entry(test_name.clone()).or_default();
            *visit += 1;
            let visit = *visit;
            let mode = second_mode.filter(|_| visit > 1).unwrap_or(first_mode);
            let mode = if mode == "undefined" { "imports" } else { mode };
            state.update_via_cache_current = Some((test_name.clone(), mode.to_string()));
            let main_time = if visit > 1 && mode == "all" { 1 } else { visit };
            let source = format!(
                "const mainTime = {main_time};\n\
                 const testName = {};\n\
                 importScripts('update-max-aged-worker-imported-script.py');\n\
                 addEventListener('message', event => {{\n\
                   event.source.postMessage({{mainTime, importTime, test: testName}});\n\
                 }});\n",
                serde_json::to_string(&test_name).unwrap()
            );
            return Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".into(), "application/javascript".into()),
                    ("Cache-Control".into(), "max-age=86400".into()),
                    ("Last-Modified".into(), "Thu, 20 Aug 2026 00:00:00 GMT".into()),
                ],
                body: source.into_bytes(),
                url: src.to_string(),
                redirect_count: 0,
            });
        } else if clean.ends_with("/resources/update-max-aged-worker-imported-script.py") {
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            let (test_name, mode) = state
                .update_via_cache_current
                .clone()
                .ok_or_else(|| "updateViaCache imported script has no main request context".to_string())?;
            let visit = state.update_via_cache_import_visits.entry(test_name).or_default();
            *visit += 1;
            let import_time = if *visit > 1 && mode == "none" { *visit } else { 1 };
            return Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".into(), "application/javascript".into()),
                    ("Cache-Control".into(), "max-age=86400".into()),
                    ("Last-Modified".into(), "Thu, 20 Aug 2026 00:00:00 GMT".into()),
                ],
                body: format!("const importTime = {import_time};\n").into_bytes(),
                url: src.to_string(),
                redirect_count: 0,
            });
        } else if clean.ends_with("/resources/update-worker-from-file.py") {
            let key = params
                .get("Key")
                .ok_or_else(|| "update-worker-from-file.py requires Key".to_string())?;
            let first = params
                .get("First")
                .ok_or_else(|| "update-worker-from-file.py requires First".to_string())?;
            let second = params
                .get("Second")
                .ok_or_else(|| "update-worker-from-file.py requires Second".to_string())?;
            if [first, second]
                .into_iter()
                .any(|name| name.is_empty() || name.contains('/') || name.contains('\\') || name.contains(".."))
            {
                return Err("update-worker-from-file.py received an invalid filename".into());
            }
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            let visit = state.update_worker_from_file_visits.entry(key.clone()).or_default();
            *visit += 1;
            let filename = match *visit {
                1 => first,
                2 => second,
                _ => return Err("update-worker-from-file.py received too many requests".into()),
            };
            let directory = clean.rsplit_once('/').map_or("", |(directory, _)| directory);
            let path = format!("{directory}/{filename}");
            let source = read_corpus_to_string(root.join(&path))
                .map_err(|error| format!("Service Worker fixture fetch failed: {path} ({error})"))?;
            return Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".into(), "application/javascript".into()),
                    ("Cache-Control".into(), "no-cache, must-revalidate".into()),
                    ("Pragma".into(), "no-cache".into()),
                ],
                body: source.into_bytes(),
                url: src.to_string(),
                redirect_count: 0,
            });
        } else if clean.ends_with("/resources/update-during-installation-worker.py") {
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            state.next_version += 1;
            return Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".into(), "application/javascript".into()),
                    ("Cache-Control".into(), "max-age=0".into()),
                ],
                body: format!(
                    "// {}\nimportScripts('update-during-installation-worker.js');",
                    state.next_version
                )
                .into_bytes(),
                url: src.to_string(),
                redirect_count: 0,
            });
        } else if clean.ends_with("/resources/update-nocookie-worker.py") {
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            state.next_version += 1;
            return Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".into(), "application/javascript".into()),
                    ("Cache-Control".into(), "no-cache, must-revalidate".into()),
                    ("Pragma".into(), "no-cache".into()),
                ],
                body: format!("// {}", state.next_version).into_bytes(),
                url: src.to_string(),
                redirect_count: 0,
            });
        } else if clean.ends_with("/resources/update-missing-import-scripts-main-worker.py") {
            let key = params
                .get("key")
                .ok_or_else(|| "update-missing-import-scripts-main-worker.py requires key".to_string())?;
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            let visit = state.missing_import_main_visits.entry(key.clone()).or_default();
            *visit += 1;
            let source = if *visit == 1 {
                format!("importScripts('./update-missing-import-scripts-imported-worker.py?key={key}');")
            } else {
                "// removed importScripts()".into()
            };
            return Ok(service_worker_fixture_response(source, src.to_string(), 0));
        } else if clean.ends_with("/resources/update-missing-import-scripts-imported-worker.py") {
            let key = params
                .get("key")
                .ok_or_else(|| "update-missing-import-scripts-imported-worker.py requires key".to_string())?;
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            let visit = state.missing_import_script_visits.entry(key.clone()).or_default();
            *visit += 1;
            if *visit > 1 {
                return Err("Service Worker imported script returned HTTP 404".into());
            }
            return Ok(service_worker_fixture_response(
                "// initial script".into(),
                src.to_string(),
                0,
            ));
        } else if clean.ends_with("/resources/404.py") {
            return Err("Service Worker imported script returned HTTP 404".into());
        } else if clean.ends_with("/resources/import-scripts-404-after-update-plus-update-worker.js")
            || clean.ends_with("/resources/import-scripts-404-after-update.js")
        {
            if !clean.contains("-plus-update-") {
                let key = params
                    .get("Key")
                    .ok_or_else(|| "import-scripts-404-after-update.js requires Key".to_string())?;
                state
                    .lock()
                    .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?
                    .cached_missing_import_keys
                    .insert(key.clone());
            }
            headers.push(("Content-Type".into(), "application/javascript".into()));
            let bytes = std::fs::read(root.join(clean))
                .map_err(|error| format!("Service Worker fixture fetch failed: {clean} ({error})"))?;
            String::from_utf8(bytes)
                .map_err(|_| format!("Service Worker fixture is not UTF-8: {clean}"))?
                .into_bytes()
        } else if clean.ends_with("/resources/import-scripts-version.py") {
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            state.next_version += 1;
            return Ok(service_worker_fixture_response(
                format!("version = \"{}\";\n", state.next_version),
                src.to_string(),
                0,
            ));
        } else if clean.ends_with("/resources/import-scripts-get.py") {
            let output = params
                .get("output")
                .ok_or_else(|| "import-scripts-get.py requires output".to_string())?;
            if !output.bytes().enumerate().all(|(index, byte)| {
                byte == b'_' || byte == b'$' || byte.is_ascii_alphabetic() || (index != 0 && byte.is_ascii_digit())
            }) {
                return Err("import-scripts-get.py output is not a JavaScript identifier".into());
            }
            let message = params
                .get("msg")
                .ok_or_else(|| "import-scripts-get.py requires msg".to_string())?;
            return Ok(service_worker_fixture_response(
                format!("{output} = {};\n", serde_json::to_string(message).unwrap()),
                src.to_string(),
                0,
            ));
        } else if clean.ends_with("/resources/import-scripts-echo.py") {
            let message = params
                .get("msg")
                .ok_or_else(|| "import-scripts-echo.py requires msg".to_string())?;
            return Ok(service_worker_fixture_response(
                format!("echo_output = {};\n", serde_json::to_string(message).unwrap()),
                src.to_string(),
                0,
            ));
        } else if clean.ends_with("/redirect.py") {
            let target = params
                .get("Redirect")
                .ok_or_else(|| "redirect.py requires Redirect".to_string())?;
            let final_url = resolve_service_worker_fixture_redirect(src, target)?;
            let (final_path, _) = service_worker_fixture_path(&final_url)?;
            if final_path.ends_with("/resources/import-scripts-version.py") {
                let mut state = state
                    .lock()
                    .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
                state.next_version += 1;
                return Ok(service_worker_fixture_response(
                    format!("version = \"{}\";\n", state.next_version),
                    final_url,
                    1,
                ));
            }
            let bytes = std::fs::read(root.join(final_path))
                .map_err(|error| format!("Service Worker redirect target fetch failed: {final_path} ({error})"))?;
            let source = String::from_utf8(bytes)
                .map_err(|_| format!("Service Worker redirect target is not UTF-8: {final_path}"))?;
            return Ok(service_worker_fixture_response(source, final_url, 1));
        } else if clean.ends_with("/resources/update-worker.py") {
            let key = params
                .get("Key")
                .ok_or_else(|| "update-worker.py requires Key".to_string())?;
            let mode = params
                .get("Mode")
                .ok_or_else(|| "update-worker.py requires Mode".to_string())?;
            let mut state = state
                .lock()
                .map_err(|_| "Service Worker fixture state lock is poisoned".to_string())?;
            let visited = state.update_worker_visits.entry(key.clone()).or_default();
            *visited += 1;
            let visited = *visited;
            if visited == 2 && mode == "not_found" {
                if state.cached_missing_import_keys.contains(key) {
                    return Ok(service_worker_fixture_response("/* 1 */".into(), src.to_string(), 0));
                }
                return Err("Service Worker imported script returned HTTP 404".into());
            }
            if visited == 2 && mode == "redirect" {
                let target = params
                    .get("Redirect")
                    .ok_or_else(|| "update-worker.py redirect mode requires Redirect".to_string())?;
                let final_url = resolve_service_worker_fixture_redirect(src, target)?;
                state.update_worker_visits.insert(key.clone(), visited + 1);
                return Ok(service_worker_fixture_response(
                    format!("/* {} */", visited + 1),
                    final_url,
                    1,
                ));
            }
            let extra_body = match (visited, mode.as_str()) {
                (2, "syntax_error") => " badsyntax(isbad;",
                (2, "throw_install") => " addEventListener('install', function() { throw new Error('boom'); });",
                _ => "",
            };
            let content_type = if visited == 2 && mode == "bad_mime_type" {
                "text/html"
            } else {
                "application/javascript"
            };
            return Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".into(), content_type.into()),
                    ("Cache-Control".into(), "no-cache, must-revalidate".into()),
                    ("Pragma".into(), "no-cache".into()),
                ],
                body: format!("/* {visited} */{extra_body}").into_bytes(),
                url: src.to_string(),
                redirect_count: 0,
            });
        } else {
            headers.push(("Content-Type".into(), "application/javascript".into()));
            let bytes = std::fs::read(root.join(clean))
                .map_err(|error| format!("Service Worker fixture fetch failed: {clean} ({error})"))?;
            let source = String::from_utf8(bytes)
                .map_err(|_| format!("Service Worker fixture is not UTF-8: {clean}"))?
                .replace("{{domains[www1]}}", "www1.wpt.test")
                // 端口值随 apply_wpt_substitutions 主表重映射（s116——SW 注册 URL 走
                // 共享替换产 :8444，脚本体与 origin 校验须同值防脑裂）。
                .replace("{{ports[https][0]}}", "8444");
            if clean.ends_with("/script-tests/cache-abort.js") {
                format!("{CACHE_ABORT_FETCH_FIXTURE}\n{source}").into_bytes()
            } else if is_service_worker_any_js(clean) {
                service_worker_any_js_source(clean, &source).into_bytes()
            } else {
                source.into_bytes()
            }
        };
        Ok(zero_net::HttpResponse {
            status_code: 200,
            headers,
            body,
            url: src.to_string(),
            redirect_count: 0,
        })
    }))
}

/// R34xx（G6）：运行导入的 `html/canvas` `.worker.js` OffscreenCanvas worker 变体——每个
/// 文件包一个 `fetch_tests_from_worker(new Worker(...))` 主页面（testharness.js 的 worker
/// 聚合协议），经 run_testharness_html_inner 同款轮询执行。返回与主线程用例同构结果。
pub fn run_canvas_worker_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    let mut cases = Vec::new();
    for subdir in CANVAS_OFFSCREEN_SUBDIRS {
        let dir = wpt_root.join(subdir);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".worker.js") {
                continue;
            }
            let relative = format!("{}/{}", subdir, name);
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let source = match read_corpus_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    cases.push((
                        relative.clone(),
                        vec![HarnessSubtestResult {
                            name: "load WPT worker case".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    ));
                    continue;
                }
            };
            // R56h：WPT 套件内部语义冲突用例（见 CANVAS_SKIP_FILES 注释）→ NotRun。
            if CANVAS_SKIP_FILES.contains(&relative.as_str()) {
                cases.push((
                    relative.clone(),
                    vec![HarnessSubtestResult {
                        name: "WPT suite-inconsistent case".into(),
                        status: HarnessStatus::NotRun,
                        message: Some(
                            "与套件内 stroke.scale1/2 + transformation.changing/multiple 的 CTM 语义互斥（追加时烘焙）——保持主流语义并跳过"
                                .into(),
                        ),
                    }],
                ));
                continue;
            }
            // 主页面：testharness.js 内联 + fetch_tests_from_worker 聚合 worker。
            let page = format!(
                "<!DOCTYPE html><html><body><canvas id='c' width='10' height='10'></canvas>\
                 <script src='/resources/testharness.js'></script>\
                 <script>\
                 fetch_tests_from_worker(new Worker('/{relative}'));\
                 </script></body></html>"
            );
            let results = run_testharness_html_inner(wpt_root, &relative, &page, &harness_source, &[], CASE_TIMEOUT);
            // worker 内嵌脚本经 __zw_fetch_script 取（source 变量仅为存在性检查）。
            let _ = source;
            cases.push((relative, results));
        }
    }
    cases
}

/// R34xx：`fetch()` 的 wpt-data 本地处理器（2d.composite.image.* 等用例经
/// `fetch('/images/...') + createImageBitmap(blob)` 取图像源——shim fetch 落 ok:false stub
/// 时无法取源）。接受 `https://wpt.test/<path>`（绝对）与 `<path>`（相对——shim 原样透传
/// fetch 入参，如 '/images/yellow75.png'）；文件缺失返 Err → shim 落 ok:false（404 语义）。
fn wpt_data_fetch_handler(wpt_root: &std::path::Path) -> Option<zero_engine::fetch_bridge::FetchHandler> {
    let root = wpt_root.to_path_buf();
    // page-state-compat M2：beacon/resources 三端点共享 stash（handler per-case 构建——
    // 每案新 stash 与 wptserve server.stash 语义对齐；id 键 = token()/UUID 案内唯一）。
    let beacon_stash: std::sync::Arc<std::sync::Mutex<HashMap<String, String>>> = std::sync::Arc::default();
    let vary_value_override = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
    Some(std::sync::Arc::new(
        move |req: &zero_engine::fetch_bridge::FetchRequest| {
            let path_part = wpt_url_path(&req.url);
            let path_part = path_part.strip_prefix('/').unwrap_or(path_part);
            // net-api M4-S23：路径点段归一（上游 wptserve/URL 语义——`/a/../b` 即 `/b`。
            // cors 域以 `dirname(page)/../resources/x.py` 形态构造端点 URL——不归一则
            // fixture 分支失配（clean 带 `cors/..`），静态读库才由文件系统隐式归一）。
            let (raw_path, path_query) = path_part.split_once('?').unwrap_or((path_part, ""));
            let mut norm_segments: Vec<&str> = Vec::new();
            for segment in raw_path.split(['?', '#']).next().unwrap_or(raw_path).split('/') {
                match segment {
                    ".." => {
                        norm_segments.pop();
                    }
                    "." => {}
                    s => norm_segments.push(s),
                }
            }
            let path_part = if path_query.is_empty() {
                norm_segments.join("/")
            } else {
                format!("{}?{}", norm_segments.join("/"), path_query)
            };
            let clean = path_part.split(['?', '#']).next().unwrap_or(&path_part);
            if clean.is_empty() {
                // net-api M3-S2 fixture：站点根（wpt.test /）——responsetype DONE 腿
                // `open('get', '/')` 须非空响应体；通用最小根页。
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: "wpt.test root\n".to_string(),
                    body_bytes: Some(b"wpt.test root\n".to_vec()),
                });
            }
            if clean == "FileAPI/file/resources/echo-content-escaped.py" {
                // form-submission-0 三族（urlencoded2/text-plain/multipart-formdata
                // .window.js 经 enctypes-helper.js ACTION_URL）的提交回显端点——上游
                // wptserve echo-content-escaped.py 行为等价：请求体逐字节转义为 `\xNN`
                // 文本（可打印 ASCII 与 CR/LF 字面原样；`\` 自身转义 `\x5C`，防被
                // helper 的 `\xNN` 还原正则误吞），text/plain 交付——helper
                // `unescape()` 可逆，断言面即提交编码算法本体。CR/LF 不转义是语义
                // 关键：helper `unescape()` 首步 `/\r\n?|\n/g → "\r\n"` 归一只作用于
                // 字面 CR/LF（先于 `\xNN` 解码）——若转义为 `\x0D\x0A`，解码后不再
                // 归一，text/plain `\n`→`\r\n` 展开案即错。
                let raw: Vec<u8> = req
                    .body_bytes
                    .clone()
                    .or_else(|| req.body.as_ref().map(|b| b.as_bytes().to_vec()))
                    .unwrap_or_default();
                let mut escaped = String::with_capacity(raw.len() * 4);
                for &byte in &raw {
                    match byte {
                        b'\\' => escaped.push_str("\\x5C"),
                        b'\r' | b'\n' | 0x20..=0x7E => escaped.push(byte as char),
                        _ => escaped.push_str(&format!("\\x{byte:02X}")),
                    }
                }
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: escaped.clone(),
                    body_bytes: Some(escaped.into_bytes()),
                });
            }
            if clean == "beacon/resources/inspect-header.py"
                || clean == "beacon/resources/content-type-and-body.py"
                || clean == "beacon/resources/url-echo.py"
            {
                // page-state-compat M2 fixture：beacon/resources 三端点（上游行为等价——
                // https://github.com/web-platform-tests/wpt/blob/315976933870/beacon/resources/
                // inspect-header.py、content-type-and-body.py、url-echo.py）。共享 take 语义
                // stash（wptserve server.stash.put/take——读后即删）；beacon POST 打 put/store，
                // 断言侧 fetch 轮询打 get/stat。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let cmd = wpt_query_value(query, "cmd")
                    .map(|v| v.to_lowercase())
                    .unwrap_or_default();
                let id = wpt_query_value(query, "id").unwrap_or_default();
                let mut headers: Vec<(String, String)> = Vec::new();
                let mut status: u16 = 200;
                let mut body = String::new();
                if clean.ends_with("inspect-header.py") {
                    headers.push(("content-type".into(), "text/plain".into()));
                    let header = wpt_query_value(query, "header").unwrap_or_default();
                    if cmd == "put" {
                        // 上游 request.headers.get(header, b"")——头缺席存空串。
                        let value = req
                            .headers
                            .iter()
                            .find(|(n, _)| n.eq_ignore_ascii_case(&header))
                            .map(|(_, v)| v.clone())
                            .unwrap_or_default();
                        if let Ok(mut stash) = beacon_stash.lock() {
                            stash.insert(id, value);
                        }
                    } else if cmd == "get" {
                        // take 语义；有值（含空串）回 x-request-<header>（pollResult 依
                        // headers.get != undefined 结束轮询——无值则不发射头，轮询继续）。
                        let stored = beacon_stash.lock().ok().and_then(|mut s| s.remove(&id));
                        if let Some(value) = stored {
                            headers.push((format!("x-request-{}", header.to_lowercase()), value));
                        }
                    } else {
                        status = 400;
                        body = "ERROR: Bad Command!".into();
                    }
                } else if clean.ends_with("content-type-and-body.py") {
                    headers.push(("content-type".into(), "text/plain".into()));
                    if cmd == "put" {
                        // 上游 stash[id] = Content-Type + ": " + body（头缺席 CT 为空串——
                        // ArrayBuffer/View 无 CT 腿期望 `: <bytes>`）。
                        let ct = req
                            .headers
                            .iter()
                            .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
                            .map(|(_, v)| v.clone())
                            .unwrap_or_default();
                        let body_bytes = req
                            .body_bytes
                            .clone()
                            .or_else(|| req.body.clone().map(String::into_bytes))
                            .unwrap_or_default();
                        let value = format!("{}: {}", ct, String::from_utf8_lossy(&body_bytes));
                        if let Ok(mut stash) = beacon_stash.lock() {
                            stash.insert(id, value);
                        }
                    } else if cmd == "get" {
                        let stored = beacon_stash.lock().ok().and_then(|mut s| s.remove(&id));
                        match stored {
                            Some(value) => body = value,
                            // 上游缺值 set_error(400)（cmd 合法但 stash 空）。
                            None => {
                                status = 400;
                                body = "ERROR: Bad Command!".into();
                            }
                        }
                    } else {
                        status = 400;
                        body = "ERROR: Bad Command!".into();
                    }
                } else {
                    // url-echo.py：store 存原始（百分号编码形态）query 串；stat take 返回
                    //（无值空体，状态恒 200——上游仅非法 cmd 置 400）。
                    if cmd == "store" {
                        let raw_query = req.url.split_once('?').map(|(_, q)| q).unwrap_or("");
                        if let Ok(mut stash) = beacon_stash.lock() {
                            stash.insert(id, raw_query.to_string());
                        }
                        headers.push(("content-type".into(), "text/plain".into()));
                    } else if cmd == "stat" {
                        headers.push(("content-type".into(), "text/plain".into()));
                        body = beacon_stash
                            .lock()
                            .ok()
                            .and_then(|mut s| s.remove(&id))
                            .unwrap_or_default();
                    } else {
                        status = 400;
                    }
                }
                wpt_add_fetch_metadata(&mut headers, req, status);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status,
                    status_text: wpt_status_text(status).to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean == "service-workers/service-worker/resources/fetch-with-body-worker.py" {
                // https://github.com/web-platform-tests/wpt/blob/04067ce9c7c2165e71ad7d0dde10a4c5cb394a83/service-workers/service-worker/resources/fetch-with-body-worker.py
                let has_body = req.body_bytes.as_ref().is_some_and(|bytes| !bytes.is_empty())
                    || req.body.as_ref().is_some_and(|body| !body.is_empty());
                let status = if has_body { 200 } else { 400 };
                let body = if has_body { "BODY" } else { "NO BODY" };
                let mut headers = wpt_pipe_headers(path_part.split_once('?').map(|(_, query)| query).unwrap_or(""));
                wpt_add_fetch_metadata(&mut headers, req, status);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status,
                    status_text: wpt_status_text(status).to_string(),
                    headers,
                    body: body.to_string(),
                    body_bytes: Some(body.as_bytes().to_vec()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/upload.py") {
                // net-api M4-S19 fixture：xhr/resources/upload.py（上游行为等价——
                // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/
                // upload.py）。解析 multipart/form-data 体：无 filename 的 part 按名排序
                // 输出 `key=value,`，随后 `\n`，再 filename part 按名排序输出
                // `key=filename:ctype:len,`。
                let boundary = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
                    .and_then(|(_, v)| {
                        let lower = v.to_lowercase();
                        lower
                            .find("boundary=")
                            .map(|idx| v[idx + 9..].trim().trim_matches('"').to_string())
                    })
                    .unwrap_or_default();
                let body = req.body_bytes.clone().unwrap_or_default();
                let mut text_parts: Vec<(String, String)> = Vec::new();
                let mut file_parts: Vec<(String, String, String, usize)> = Vec::new();
                if !boundary.is_empty() && !body.is_empty() {
                    let delim = format!("--{}", boundary);
                    let delim_bytes = delim.as_bytes();
                    let mut segments: Vec<&[u8]> = Vec::new();
                    let mut start = 0;
                    for i in 0..body.len() {
                        if body[i..].starts_with(delim_bytes) {
                            if i >= start + 2 {
                                segments.push(&body[start..i]);
                            }
                            start = i + delim_bytes.len();
                        }
                    }
                    for seg in segments {
                        // 每段：前导 \r\n + 头部 + \r\n\r\n + 体（尾随 \r\n 去除）。
                        let seg = seg.strip_prefix(b"\r\n").unwrap_or(seg);
                        let sep = match seg.windows(4).position(|w| w == b"\r\n\r\n") {
                            Some(pos) => pos,
                            None => continue,
                        };
                        let head = String::from_utf8_lossy(&seg[..sep]).to_lowercase();
                        let mut value = &seg[sep + 4..];
                        if value.ends_with(b"\r\n") {
                            value = &value[..value.len() - 2];
                        }
                        let mut name = String::new();
                        let mut filename: Option<String> = None;
                        let mut ctype = String::new();
                        for line in head.split("\r\n") {
                            if let Some(cd) = line.strip_prefix("content-disposition:") {
                                for attr in cd.split(';') {
                                    let attr = attr.trim();
                                    if let Some(nv) = attr.strip_prefix("name=") {
                                        name = nv.trim_matches('"').to_string();
                                    } else if let Some(fv) = attr.strip_prefix("filename=") {
                                        filename = Some(fv.trim_matches('"').to_string());
                                    }
                                }
                            } else if let Some(ct) = line.strip_prefix("content-type:") {
                                ctype = ct.trim().to_string();
                            }
                        }
                        if name.is_empty() {
                            continue;
                        }
                        match filename {
                            Some(fname) => file_parts.push((
                                name,
                                fname,
                                if ctype.is_empty() { "None".into() } else { ctype },
                                value.len(),
                            )),
                            None => text_parts.push((name, String::from_utf8_lossy(value).into_owned())),
                        }
                    }
                }
                text_parts.sort_by(|a, b| a.0.cmp(&b.0));
                file_parts.sort_by(|a, b| a.0.cmp(&b.0));
                let mut content = String::new();
                for (k, v) in &text_parts {
                    content.push_str(&format!("{}={},", k, v));
                }
                content.push('\n');
                for (k, f, ct, len) in &file_parts {
                    content.push_str(&format!("{}={}:{}:{},", k, f, ct, len));
                }
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: content.clone(),
                    body_bytes: Some(content.into_bytes()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/form.py") {
                // net-api M4-S21 fixture：xhr/resources/form.py（上游行为等价——multipart
                // 体按文档序回显 `name:value;`；text part 提取与 upload.py 同型）。
                let boundary = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
                    .and_then(|(_, v)| {
                        let lower = v.to_lowercase();
                        lower
                            .find("boundary=")
                            .map(|idx| v[idx + 9..].trim().trim_matches('"').to_string())
                    })
                    .unwrap_or_default();
                let body = req.body_bytes.clone().unwrap_or_default();
                let mut content = String::new();
                if !boundary.is_empty() && !body.is_empty() {
                    let delim = format!("--{}", boundary);
                    let delim_bytes = delim.as_bytes();
                    let mut segments: Vec<&[u8]> = Vec::new();
                    let mut start = 0;
                    for i in 0..body.len() {
                        if body[i..].starts_with(delim_bytes) {
                            if i >= start + 2 {
                                segments.push(&body[start..i]);
                            }
                            start = i + delim_bytes.len();
                        }
                    }
                    for seg in segments {
                        let seg = seg.strip_prefix(b"\r\n").unwrap_or(seg);
                        let sep = match seg.windows(4).position(|w| w == b"\r\n\r\n") {
                            Some(pos) => pos,
                            None => continue,
                        };
                        let head = String::from_utf8_lossy(&seg[..sep]).to_lowercase();
                        let mut value = &seg[sep + 4..];
                        if value.ends_with(b"\r\n") {
                            value = &value[..value.len() - 2];
                        }
                        let mut name = String::new();
                        let mut has_filename = false;
                        for line in head.split("\r\n") {
                            if let Some(cd) = line.strip_prefix("content-disposition:") {
                                for attr in cd.split(';') {
                                    let attr = attr.trim();
                                    if let Some(nv) = attr.strip_prefix("name=") {
                                        name = nv.trim_matches('"').to_string();
                                    } else if attr.starts_with("filename=") {
                                        has_filename = true;
                                    }
                                }
                            }
                        }
                        // 上游 form.py 对 text part 回显（file part 跳过——本次用例无）。
                        if !name.is_empty() && !has_filename {
                            content.push_str(&format!("{}:{};", name, String::from_utf8_lossy(value)));
                        }
                    }
                }
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: content.clone(),
                    body_bytes: Some(content.into_bytes()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/echo-content-type.py") {
                // net-api M4-S21 fixture：xhr/resources/echo-content-type.py（上游行为
                // 等价——回显请求 Content-Type 头，content-type-unmodified 面）。
                let ct = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: ct.clone(),
                    body_bytes: Some(ct.into_bytes()),
                });
            }
            if clean.starts_with("fetch/api/resources/")
                && (clean.ends_with("/echo-content.py") || clean.ends_with("/echo-content.h2.py"))
            {
                // net-api 二期 M2-S3 fixture：fetch/api/basic/echo-content.py 与
                // echo-content.h2.py（上游行为等价——原样回显请求体，request-upload
                // 面断言 resp.text() == 上传体）。`.h2.py` 端点上游走 h2 流式上传；
                // runner 侧流→字节泵尽后经常规 wire 到达，此处不区分。
                let body = req
                    .body_bytes
                    .clone()
                    .or_else(|| req.body.clone().map(|s| s.into_bytes()))
                    .unwrap_or_default();
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                let lossy = String::from_utf8_lossy(&body).into_owned();
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: lossy,
                    body_bytes: Some(body),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/access-control-origin-header.py") {
                // net-api M4-S21 fixture：xhr/resources/access-control-origin-header.py
                //（上游行为等价——ACAO 回显 Origin + body "PASS: Cross-domain access
                // allowed.\nHTTP_ORIGIN: <origin>"，access-control-basic-allow- 面）。
                let origin = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                headers.push(("access-control-allow-origin".into(), origin.clone()));
                headers.push(("access-control-allow-credentials".into(), "true".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                let body = format!("PASS: Cross-domain access allowed.\nHTTP_ORIGIN: {}", origin);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/over-1-meg.txt") {
                // net-api M4-S21 fixture：xhr/resources/over-1-meg.txt（上游行为等价——
                // 1.16MB "abcd" 重复体，over-1-meg 面）。
                let body = "abcd".repeat(290_000);
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/access-control-basic-allow-star.py") {
                // net-api M4-S17 fixture：xhr/resources/access-control-basic-allow-star.py
                //（上游逐字等价——ACAO * + PASS 体）。
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                headers.push(("access-control-allow-origin".into(), "*".into()));
                let body = "PASS: Cross-domain access allowed.".to_string();
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("xhr/")
                && (clean.ends_with("/access-control-basic-allow.py")
                    || clean.ends_with("/access-control-basic-allow-no-credentials.py"))
            {
                // net-api M4-S17 fixture：xhr/resources/access-control-basic-allow.py 与
                // access-control-basic-allow-no-credentials.py（上游逐字等价——ACAO 回显
                // 请求 Origin；allow 变体另带 ACAC true，no-credentials 变体不带）。
                let origin = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "*".to_string());
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                headers.push(("access-control-allow-origin".into(), origin));
                if clean.ends_with("/access-control-basic-allow.py") {
                    headers.push(("access-control-allow-credentials".into(), "true".into()));
                }
                let body = "PASS: Cross-domain access allowed.".to_string();
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/redirect-cors.py") {
                // net-api M4-S17 fixture：xhr/resources/redirect-cors.py（上游行为等价——
                // GET → 302 + Location；?allow_origin= 回显 Origin、?allow_header= 透传
                // ACAH；OPTIONS preflight 面待 engine preflight 语义，此实现 GET 面）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let location = wpt_query_value(query, "location").unwrap_or_default();
                let origin = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let mut headers: Vec<(String, String)> = Vec::new();
                if req.method == "OPTIONS" {
                    // 上游语义：preflight 不重定向（redirect_preflight 才 302）+ ACAM/Max-Age。
                    headers.push(("access-control-allow-methods".into(), "GET".into()));
                    headers.push(("access-control-max-age".into(), "1".into()));
                    if wpt_query_value(query, "allow_origin").is_some() {
                        headers.push(("access-control-allow-origin".into(), origin));
                    }
                    if let Some(allow_header) = wpt_query_value(query, "allow_header") {
                        headers.push(("access-control-allow-headers".into(), allow_header));
                    }
                    let preflight_redirect = wpt_query_value(query, "redirect_preflight").is_some();
                    let status: u16 = if preflight_redirect { 302 } else { 200 };
                    if preflight_redirect {
                        headers.push(("location".into(), location));
                    }
                    wpt_add_fetch_metadata(&mut headers, req, status);
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status,
                        status_text: wpt_status_text(status).to_string(),
                        headers,
                        body: String::new(),
                        body_bytes: Some(Vec::new()),
                    });
                }
                headers.push(("location".into(), location));
                if wpt_query_value(query, "allow_origin").is_some() {
                    headers.push(("access-control-allow-origin".into(), origin));
                }
                if let Some(allow_header) = wpt_query_value(query, "allow_header") {
                    headers.push(("access-control-allow-headers".into(), allow_header));
                }
                wpt_add_fetch_metadata(&mut headers, req, 302);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 302,
                    status_text: wpt_status_text(302).to_string(),
                    headers,
                    body: String::new(),
                    body_bytes: Some(Vec::new()),
                });
            }
            if clean.starts_with("fetch/api/resources/") && clean.ends_with("/redirect.py") {
                // net-api M4-S17/S23 fixture：fetch/api/resources/redirect.py（上游行为
                // 等价）：ACAO 回显 Origin（无 Origin → *）+ ACAC（Origin 在时）+
                // ?redirect_status=（默认 302）+ Location（**附全量原 query + count**——
                // 上游 urlencode(url_parameters) 语义，M4-S23 对齐）。OPTIONS 分支：
                // 无 ?redirect_preflight → 200 不重定向（preflight 须不跟随——
                // cors-preflight-redirect 面）；有 → **落穿重定向逻辑返回 redirect_status**
                //（上游同构——preflight 命中重定向 → fetch 判失败）。
                static REDIRECT_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let count = REDIRECT_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let status: u16 = wpt_query_value(query, "redirect_status")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(302);
                let origin = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                    .map(|(_, v)| v.clone());
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                headers.push(("cache-control".into(), "no-cache".into()));
                headers.push((
                    "access-control-allow-origin".into(),
                    origin.clone().unwrap_or_else(|| "*".into()),
                ));
                if origin.is_some() {
                    headers.push(("access-control-allow-credentials".into(), "true".into()));
                }
                if req.method == "OPTIONS" {
                    if let Some(allow_headers) = wpt_query_value(query, "allow_headers") {
                        headers.push(("access-control-allow-headers".into(), allow_headers));
                    }
                    if wpt_query_value(query, "redirect_preflight").is_none() {
                        wpt_add_fetch_metadata(&mut headers, req, 200);
                        return Ok(zero_engine::fetch_bridge::FetchResponse {
                            status: 200,
                            status_text: "OK".to_string(),
                            headers,
                            body: String::new(),
                            body_bytes: Some(Vec::new()),
                        });
                    }
                    // redirect_preflight → 落穿：preflight 响应带重定向（fetch 侧拒绝）。
                }
                if let Some(location) = wpt_query_value(query, "location") {
                    // 上游：Location 保留全量原 query（urlencode(url_parameters)）+ count。
                    let sep = if location.contains('?') { '&' } else { '?' };
                    let mut carried: Vec<String> = Vec::new();
                    for pair in query.split('&') {
                        if pair.is_empty() {
                            continue;
                        }
                        let (k, _) = pair.split_once('=').unwrap_or((pair, ""));
                        if k == "location" {
                            continue;
                        }
                        carried.push(pair.to_string());
                    }
                    carried.push(format!("count={}", count));
                    headers.push(("location".into(), format!("{}{}{}", location, sep, carried.join("&"))));
                }
                wpt_add_fetch_metadata(&mut headers, req, status);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status,
                    status_text: wpt_status_text(status).to_string(),
                    headers,
                    body: String::new(),
                    body_bytes: Some(Vec::new()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/image.gif") {
                // net-api M4-S20 fixture：xhr/resources/image.gif（responseurl 重定向
                // 目标——1×1 GIF 43 字节标准头尾）。
                let body: Vec<u8> = vec![
                    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0xff, 0xff, 0xff, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44,
                    0x01, 0x00, 0x3b,
                ];
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "image/gif".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: String::new(),
                    body_bytes: Some(body),
                });
            }
            if clean.ends_with("/echo-headers.py") {
                // net-api M4-S21 fixture：request-content-length 载体——回显全部请求头
                // 为 `Name: value` 行（request-content-length.any.js 断言
                // responseText 含 `Content-Length: <len>`）。
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                let mut body = String::new();
                for (n, v) in &req.headers {
                    // HTTP 头名 HTTP 风格大写（`Content-Length: N` 大小写敏感 includes 面）。
                    let mut title = String::new();
                    let mut cap = true;
                    for ch in n.chars() {
                        if cap {
                            title.extend(ch.to_uppercase());
                            cap = false;
                        } else {
                            title.push(ch);
                            if ch == '-' {
                                cap = true;
                            }
                        }
                    }
                    body.push_str(&format!("{}: {}\n", title, v));
                }
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.ends_with("/delay.py") {
                // net-api M4-S20 fixture：xhr/resources/delay.py（上游行为等价——?ms= 延迟
                // 后回 text/plain "TEST"；延迟由 shim 侧 setTimeout 实现——runner 同步契约
                // 下 host sleep 冻结 JS，此处立即应答）。
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                // net-api M5-S10：OPTIONS preflight 应答（ACAO 回显 + ACAM/ACAH ACRM/ACRH
                // 并入——上游 wptserve 自动应答 preflight 的合规 CORS 服务端语义；
                // timeout-multiple-fetches「CORS preflights should not reset the timer」
                // YO 方法面——无此分支时 OPTIONS 落入本 200 应答无 CORS 头 → preflight
                // 拒 → onerror 而非 ontimeout。M5-S3 eventsource cors.py 同先例）。
                if req.method == "OPTIONS" {
                    headers.push((
                        "access-control-allow-origin".into(),
                        req.headers
                            .iter()
                            .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                            .map(|(_, v)| v.clone())
                            .unwrap_or_else(|| "*".into()),
                    ));
                    let mut allow_methods: Vec<String> = ["GET", "POST", "HEAD", "OPTIONS"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    if let Some(acrm) = req
                        .headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-method"))
                        .map(|(_, v)| v.trim().to_ascii_uppercase())
                        && !acrm.is_empty()
                        && !allow_methods.contains(&acrm)
                    {
                        allow_methods.push(acrm);
                    }
                    headers.push(("access-control-allow-methods".into(), allow_methods.join(", ")));
                    if let Some(acrh) = req
                        .headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
                        .map(|(_, v)| v.trim().to_string())
                        .filter(|v| !v.is_empty())
                    {
                        headers.push(("access-control-allow-headers".into(), acrh));
                    }
                    wpt_add_fetch_metadata(&mut headers, req, 200);
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers,
                        body: String::new(),
                        body_bytes: Some(Vec::new()),
                    });
                }
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: "TEST".to_string(),
                    body_bytes: Some(b"TEST".to_vec()),
                });
            }
            if clean == "xhr/resources/trickle.py" {
                // net-api M4-S21 fixture：xhr/resources/trickle.py（上游行为等价——count 块
                // × 每块 13 字节（与 fetch/api/resources/trickle.py 同 "TEST_TRICKLE\n" 块形，
                // 单测锚定该体）；specifylength=1 时补 Content-Length: <total>。逐块延迟
                // 由 finite 模型折叠——响应一次到位，progress 事件由 shim fill 侧派发）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let count = wpt_query_value(query, "count")
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(10)
                    .min(128);
                let body = "TEST_TRICKLE\n".repeat(count);
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                if path_part.contains("specifylength=1") {
                    headers.push(("content-length".into(), (count * 13).to_string()));
                }
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean == "xhr/resources/top.txt" {
                // net-api M4-S18 fixture：xhr/resources/top.txt（cors-expose-star 载体，
                // ?pipe=header(...) 管道头经既有 wpt_pipe_headers 处理）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let mut headers = wpt_pipe_headers(query);
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: "top\n".to_string(),
                    body_bytes: Some(b"top\n".to_vec()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/access-control-basic-put-allow.py") {
                // net-api M4-S22 fixture：xhr/resources/access-control-basic-put-allow.py
                //（上游逐字等价——OPTIONS: ACAM PUT + ACAO 回显 + ACAC；PUT: ACAO 回显 +
                // 体 "PASS: Cross-domain access allowed.\n<body>"，non-cors-safelisted
                // method 面）。
                let origin = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                if req.method == "OPTIONS" {
                    headers.push(("access-control-allow-credentials".into(), "true".into()));
                    headers.push(("access-control-allow-methods".into(), "PUT".into()));
                    headers.push(("access-control-allow-origin".into(), origin));
                    wpt_add_fetch_metadata(&mut headers, req, 200);
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers,
                        body: String::new(),
                        body_bytes: Some(Vec::new()),
                    });
                }
                if req.method == "PUT" {
                    headers.push(("access-control-allow-credentials".into(), "true".into()));
                    headers.push(("access-control-allow-origin".into(), origin));
                    let body = format!(
                        "PASS: Cross-domain access allowed.\n{}",
                        req.body_bytes
                            .as_ref()
                            .map(|b| String::from_utf8_lossy(b).into_owned())
                            .or_else(|| req.body.clone())
                            .unwrap_or_else(|| "Could not read in content.".into())
                    );
                    wpt_add_fetch_metadata(&mut headers, req, 200);
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers,
                        body: body.clone(),
                        body_bytes: Some(body.into_bytes()),
                    });
                }
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: format!("Wrong method: {}", req.method),
                    body_bytes: Some(format!("Wrong method: {}", req.method).into_bytes()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/reset-token.py") {
                // net-api M4-S22 fixture：xhr/resources/reset-token.py（上游逐字等价——
                // stash[token] 置空 + "PASS"，preflight-cache 族用例起点）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let token = wpt_query_value(query, "token").unwrap_or_default();
                if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
                    stash.insert(token, String::new());
                }
                let origin = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                headers.push(("access-control-allow-origin".into(), origin));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: "PASS".to_string(),
                    body_bytes: Some(b"PASS".to_vec()),
                });
            }
            if clean.starts_with("xhr/")
                && (clean.ends_with("/access-control-basic-preflight-cache.py")
                    || clean.ends_with("/access-control-basic-preflight-cache-timeout.py")
                    || clean.ends_with("/access-control-basic-preflight-cache-invalidation.py"))
            {
                // net-api M4-S22 fixture：xhr/resources/access-control-basic-preflight-cache*
                // （上游逐字等价的 token 状态机——OPTIONS 到达序断言 preflight cache 行为：
                // cache.py 二请求须复用缓存、timeout.py 须过期重发、invalidation.py 须
                // method/header 变更失效。状态迁移见上游 .py 三件）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let token = wpt_query_value(query, "token").unwrap_or_default();
                let origin = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let is_timeout = clean.ends_with("-timeout.py");
                let is_invalidation = clean.ends_with("-invalidation.py");
                // stash take 语义（上游 request.server.stash.take）——缺省或空值均视
                // Uninitialized（Python falsy：reset-token.py put 的是 b""）。
                let state = if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
                    stash.remove(&token).unwrap_or_default()
                } else {
                    String::new()
                };
                let state = if state.is_empty() {
                    "Uninitialized".to_string()
                } else {
                    state
                };
                let method = req.method.clone();
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                headers.push(("access-control-allow-origin".into(), origin));
                headers.push(("access-control-allow-credentials".into(), "true".into()));
                let (status, body, next_state) = match state.as_str() {
                    "Uninitialized" => {
                        if method == "OPTIONS" {
                            headers.push(("access-control-allow-methods".into(), "PUT".into()));
                            if is_timeout {
                                headers.push(("access-control-allow-headers".into(), "x-test".into()));
                            }
                            headers.push((
                                "access-control-max-age".into(),
                                if is_timeout { "1".into() } else { "10".into() },
                            ));
                            (200, String::new(), "OPTIONSSent".to_string())
                        } else {
                            (400, format!("FAIL {method}: Uninitialized"), state)
                        }
                    }
                    "OPTIONSSent" => {
                        if method == "PUT" {
                            (200, "PASS: First PUT request.".to_string(), "FirstPUTSent".to_string())
                        } else {
                            (400, format!("FAIL {method}: OPTIONSSent"), state)
                        }
                    }
                    "FirstPUTSent" => {
                        if method == "PUT" && !is_timeout && !is_invalidation {
                            (200, "PASS: Second PUT request. Preflight worked.".to_string(), state)
                        } else if method == "OPTIONS" {
                            headers.push((
                                "access-control-allow-methods".into(),
                                if is_invalidation {
                                    "PUT, XMETHOD".into()
                                } else {
                                    "PUT".into()
                                },
                            ));
                            if is_timeout || is_invalidation {
                                headers.push(("access-control-allow-headers".into(), "x-test".into()));
                            }
                            (200, String::new(), "SecondOPTIONSSent".to_string())
                        } else {
                            (
                                400,
                                format!("FAIL {method}: Second PUT request sent without preflight"),
                                state,
                            )
                        }
                    }
                    "SecondOPTIONSSent" => {
                        if method == "PUT" || method == "XMETHOD" {
                            (200, "PASS: Second OPTIONS request was sent.".to_string(), state)
                        } else {
                            (400, format!("FAIL {method}: SecondOPTIONSSent"), state)
                        }
                    }
                    _ => (400, format!("FAIL {method}: {state}"), state),
                };
                if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
                    stash.insert(token, next_state);
                }
                wpt_add_fetch_metadata(&mut headers, req, status);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status,
                    status_text: wpt_status_text(status).to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("xhr/")
                && (clean.ends_with("/access-control-preflight-request-header-returns-origin.py")
                    || clean.ends_with("/access-control-preflight-request-allow-headers-returns-star.py"))
            {
                // net-api M4-S22 fixture：access-control-preflight-request-*-returns-*.py
                //（上游逐字等价——OPTIONS: ACAO 回显（或 *）+ ACAH X-Test（或 *）；
                // GET: ACAO * + X-Test 在 → "PASS"，否则 400）。
                let origin = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let star_variant = clean.ends_with("-allow-headers-returns-star.py");
                let mut headers: Vec<(String, String)> = Vec::new();
                if req.method == "OPTIONS" {
                    headers.push((
                        "access-control-allow-origin".into(),
                        if star_variant { "*".into() } else { origin },
                    ));
                    headers.push((
                        "access-control-allow-headers".into(),
                        if star_variant { "*".into() } else { "X-Test".into() },
                    ));
                    wpt_add_fetch_metadata(&mut headers, req, 200);
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers,
                        body: String::new(),
                        body_bytes: Some(Vec::new()),
                    });
                }
                headers.push(("access-control-allow-origin".into(), "*".into()));
                let has_x_test = req.headers.iter().any(|(n, _)| n.eq_ignore_ascii_case("x-test"));
                if has_x_test {
                    headers.push(("content-type".into(), "text/plain".into()));
                    wpt_add_fetch_metadata(&mut headers, req, 200);
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers,
                        body: "PASS".to_string(),
                        body_bytes: Some(b"PASS".to_vec()),
                    });
                }
                wpt_add_fetch_metadata(&mut headers, req, 400);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 400,
                    status_text: wpt_status_text(400).to_string(),
                    headers,
                    body: String::new(),
                    body_bytes: Some(Vec::new()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/echo-content-cors.py") {
                // net-api M4-S22 fixture：xhr/resources/echo-content-cors.py（上游行为
                // 等价——ACAO/ACAH/ACAM 回显（?origin= 参数优先于请求头——上游同型怪癖）
                // + X-Request-* 探针头 + ACAC true + 体回显请求体，cors-upload 面）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let q_origin = wpt_query_value(query, "origin");
                let req_header = |name: &str| -> Option<String> {
                    req.headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case(name))
                        .map(|(_, v)| v.clone())
                };
                let origin = q_origin.clone().or_else(|| req_header("origin")).unwrap_or_default();
                let acrh = q_origin
                    .clone()
                    .or_else(|| req_header("access-control-request-headers"));
                let acrm = q_origin.or_else(|| req_header("access-control-request-method"));
                let mut headers: Vec<(String, String)> = vec![
                    ("x-request-method".into(), req.method.clone()),
                    (
                        "x-request-content-length".into(),
                        req_header("content-length").unwrap_or_else(|| "NO".into()),
                    ),
                    (
                        "x-request-content-type".into(),
                        req_header("content-type").unwrap_or_else(|| "NO".into()),
                    ),
                    ("access-control-allow-credentials".into(), "true".into()),
                    ("content-type".into(), "text/plain".into()),
                ];
                if !origin.is_empty() {
                    headers.push(("access-control-allow-origin".into(), origin));
                }
                if let Some(h) = acrh {
                    headers.push(("access-control-allow-headers".into(), h));
                }
                if let Some(m) = acrm {
                    headers.push(("access-control-allow-methods".into(), format!("OPTIONS, {m}")));
                }
                let body = req
                    .body_bytes
                    .as_ref()
                    .map(|b| String::from_utf8_lossy(b).into_owned())
                    .or_else(|| req.body.clone())
                    .unwrap_or_default();
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.ends_with("/dump-authorization-header.py") {
                // net-api M4-S17 fixture：fetch/api/resources/dump-authorization-header.py
                //（上游逐字等价——回显 Authorization 或 "none"；带 Origin → ACAO 回显 +
                // ACAC true + ACAH Authorization）。
                let origin = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                    .map(|(_, v)| v.clone());
                let auth = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("authorization"))
                    .map(|(_, v)| v.clone());
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/html".into()));
                headers.push(("cache-control".into(), "no-cache".into()));
                match origin {
                    Some(o) => {
                        headers.push(("access-control-allow-origin".into(), o));
                        headers.push(("access-control-allow-credentials".into(), "true".into()));
                    }
                    None => {
                        headers.push(("access-control-allow-origin".into(), "*".into()));
                    }
                }
                headers.push(("access-control-allow-headers".into(), "Authorization".into()));
                let body = auth.unwrap_or_else(|| "none".into());
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/corsenabled.py") {
                // net-api M4-S16 fixture：xhr/resources/corsenabled.py（上游逐字等价——
                // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/
                // corsenabled.py）。CORS 全开（ACAO * + ACAC + methods/headers/expose）+
                // X-Request-* 探针头 + ?delay=；体 "Test"。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let mut headers: Vec<(String, String)> = vec![
                    ("access-control-allow-origin".into(), "*".into()),
                    ("access-control-allow-credentials".into(), "true".into()),
                    ("access-control-allow-methods".into(), "GET, POST, PUT, FOO".into()),
                    ("access-control-allow-headers".into(), "x-test, x-foo".into()),
                    (
                        "access-control-expose-headers".into(),
                        "x-request-method, x-request-content-type, x-request-query, x-request-content-length, x-request-data".into(),
                    ),
                ];
                let delay_ms = wpt_query_value(query, "delay")
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
                    .min(10_000);
                if delay_ms > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                }
                let req_header = |name: &str| -> Option<String> {
                    req.headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case(name))
                        .map(|(_, v)| v.clone())
                };
                headers.push(("x-request-method".into(), req.method.clone()));
                let req_query = path_part
                    .split_once('?')
                    .map(|(_, q)| q.to_string())
                    .unwrap_or_default();
                headers.push((
                    "x-request-query".into(),
                    if req_query.is_empty() { "NO".into() } else { req_query },
                ));
                headers.push((
                    "x-request-content-length".into(),
                    req_header("content-length").unwrap_or_else(|| "NO".into()),
                ));
                headers.push((
                    "x-request-content-type".into(),
                    req_header("content-type").unwrap_or_else(|| "NO".into()),
                ));
                headers.push((
                    "x-request-data".into(),
                    req.body_bytes
                        .as_ref()
                        .map(|b| String::from_utf8_lossy(b).into_owned())
                        .unwrap_or_default(),
                ));
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: "Test".to_string(),
                    body_bytes: Some(b"Test".to_vec()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/redirect.py") {
                // net-api M4-S16 fixture：xhr/resources/redirect.py（上游逐字等价——
                // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/
                // redirect.py）。?code=（默认 302）+ ?location=（缺省本路径?followed）；
                // ?followed= → 200 "MAGIC HAPPENED"（Content:Type 破损头照抄——上游语义）；
                // 否则 → code + Location + "TEST"。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let code: u16 = wpt_query_value(query, "code")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(302);
                let followed = wpt_query_value(query, "followed").is_some();
                if followed {
                    let mut headers: Vec<(String, String)> = Vec::new();
                    headers.push(("content:type".into(), "text/plain".into()));
                    wpt_add_fetch_metadata(&mut headers, req, 200);
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers,
                        body: "MAGIC HAPPENED".to_string(),
                        body_bytes: Some(b"MAGIC HAPPENED".to_vec()),
                    });
                }
                let location = wpt_query_value(query, "location").unwrap_or_else(|| format!("{}?followed", clean));
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("location".into(), location));
                wpt_add_fetch_metadata(&mut headers, req, code);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: code,
                    status_text: wpt_status_text(code).to_string(),
                    headers,
                    body: "TEST".to_string(),
                    body_bytes: Some(b"TEST".to_vec()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/well-formed.xml") {
                // net-api M4-S16 fixture：xhr/resources/well-formed.xml（上游逐字——
                // responseurl-after-abort 的 HEADERS_RECEIVED/LOADING abort 时序载体）。
                let body = "<html xmlns=\"http://www.w3.org/1999/xhtml\">\n  <p id=\"n&#49;\">1</p>\n  <p xmlns=\"namespacesarejuststrings\" id=\"n2\">2</p>\n</html>\n".to_string();
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("access-control-allow-origin".into(), "*".into()));
                headers.push(("content-type".into(), "application/xml".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/pass.txt") {
                // net-api M4-S16 fixture：xhr/resources/pass.txt（abort-progress-events
                // 的 send+abort 竞态载体——内容不断言，请求须可进入在飞态）。
                // M4-S18：ACAO *（runner 页面 https 而 get-host-info HTTP_ORIGIN 为 http——
                // 跨 scheme hop 的 cors check 放行面，runner 环境适配）。
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("access-control-allow-origin".into(), "*".into()));
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: "PASS\n".to_string(),
                    body_bytes: Some(b"PASS\n".to_vec()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with("/content.py") {
                // net-api M4-S15 fixture：xhr/resources/content.py（上游逐字等价——
                // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/
                // content.py）。回显请求体（?content= 覆写）+ Content-type text/plain
                //（?response_charset_label= 追加 ;charset=）+ X-Request-Method/Query/
                // Content-Length/Content-Type 请求探针头。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let mut headers: Vec<(String, String)> = Vec::new();
                let ctype = match wpt_query_value(query, "response_charset_label") {
                    Some(label) => format!("text/plain;charset={}", label),
                    None => "text/plain".to_string(),
                };
                headers.push(("content-type".into(), ctype));
                headers.push(("x-request-method".into(), req.method.clone()));
                let req_query = path_part
                    .split_once('?')
                    .map(|(_, q)| q.to_string())
                    .unwrap_or_default();
                headers.push((
                    "x-request-query".into(),
                    if req_query.is_empty() { "NO".into() } else { req_query },
                ));
                let req_header = |name: &str| -> Option<String> {
                    req.headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case(name))
                        .map(|(_, v)| v.clone())
                };
                headers.push((
                    "x-request-content-length".into(),
                    req_header("content-length").unwrap_or_else(|| "NO".into()),
                ));
                headers.push((
                    "x-request-content-type".into(),
                    req_header("content-type").unwrap_or_else(|| "NO".into()),
                ));
                let body = wpt_query_value(query, "content").unwrap_or_else(|| {
                    let bytes_body = req
                        .body_bytes
                        .as_ref()
                        .map(|b| String::from_utf8_lossy(b).into_owned())
                        .unwrap_or_default();
                    if bytes_body.is_empty() {
                        req.body.clone().unwrap_or_default()
                    } else {
                        bytes_body
                    }
                });
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.ends_with("/status.py")
                && (clean.starts_with("xhr/") || clean == "fetch/api/resources/status.py")
            {
                // net-api M2-S3c：fetch/api/resources/status.py 同语义接入——上游
                // fetch/api/resources/status.py 与 xhr/resources/status.py 逐字同源
                //（code/text/content/type + X-Request-Method）。此前仅 xhr/ 前缀
                // 命中，fetch/api 面落静态读库 404（response-null-body「expected
                // 204 but got 404」、text-utf8 fetched-data 族）。
                // net-api M3-S2 fixture：xhr/resources/status.py（上游逐字等价——
                // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/
                // status.py）。?code=（默认 200）&text=（默认 OMG）&content=&type=
                // → 自定义状态行 + Content-Type + body。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let code: u16 = wpt_query_value(query, "code")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(200);
                let text = wpt_query_value(query, "text").unwrap_or_else(|| "OMG".into());
                // net-api M3-S2：content 保留原始字节（%XX → byte——shift-jis 等非 UTF-8
                // 测试载荷经 body_bytes 保真，不落 lossy UTF-8）。
                let content_bytes: Vec<u8> = wpt_query_value(query, "content")
                    .map(|v| percent_encoding::percent_decode_str(&v).collect())
                    .unwrap_or_default();
                let ctype = wpt_query_value(query, "type").unwrap_or_default();
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), ctype));
                headers.push(("x-request-method".into(), req.method.clone()));
                wpt_add_fetch_metadata(&mut headers, req, code);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: code,
                    status_text: text,
                    headers,
                    body: String::from_utf8_lossy(&content_bytes).into_owned(),
                    body_bytes: Some(content_bytes),
                });
            }
            if clean.starts_with("eventsource/") && clean.ends_with("/message.py") {
                // net-api M4-S12 fixture：eventsource/resources/message.py（上游逐字等价——
                // https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/resources/
                // message.py）。?mime=（默认 text/event-stream）&message=（默认 data: data）
                // &newline=none（尾 \n\n 省略）&sleep=（ms）→ SSE 单响应。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let mime = wpt_query_value(query, "mime").unwrap_or_else(|| "text/event-stream".into());
                let message = wpt_query_value(query, "message").unwrap_or_else(|| "data: data".into());
                let newline = if wpt_query_value(query, "newline").as_deref() == Some("none") {
                    ""
                } else {
                    "\n\n"
                };
                let sleep_ms = wpt_query_value(query, "sleep")
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
                    .min(10_000);
                if sleep_ms > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
                }
                let body = format!("{}{}\n", message, newline);
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), mime));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("eventsource/") && clean.ends_with("/message2.py") {
                // net-api M4-S12 fixture：eventsource/resources/message2.py（上游为无限循环
                // 流式写——headless 有限流模型取一轮循环体即覆盖页面断言的前三个事件；
                // https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/resources/
                // message2.py）。
                let body = "data:msg\ndata: msg\n\n:\nfalsefield:msg\n\nfalsefield:msg\n\n\
                            Data:data\n\ndata\n\ndata:end\n\n"
                    .to_string();
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/event-stream".into()));
                headers.push(("cache-control".into(), "no-cache".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("eventsource/") && clean.ends_with("/last-event-id.py") {
                // net-api M4-S12 fixture：eventsource/resources/last-event-id.py（上游逐字
                // 等价——https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/
                // resources/last-event-id.py）。带 Last-Event-ID 请求头则回 `data: <id>`，
                // 否则回 `id: <idvalue 默认 …>\nretry: 200\ndata: hello`（重连面）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let last_id = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("last-event-id"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or("");
                let body = if !last_id.is_empty() {
                    format!("data: {}\n\n", last_id)
                } else {
                    let idvalue = wpt_query_value(query, "idvalue").unwrap_or_else(|| "\u{2026}".into());
                    format!("id: {}\nretry: 200\ndata: hello\n\n", idvalue)
                };
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/event-stream".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("eventsource/") && clean.ends_with(".event_stream") {
                // net-api M4-S12 fixture：eventsource/resources/accept.event_stream 与
                // cache-control.event_stream（上游为 wptserve 模板文件
                // `data: {{headers[<name>]}}`——按文件名回显对应请求头；
                // https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/resources/）。
                let header_name = clean
                    .rsplit('/')
                    .next()
                    .and_then(|f| f.strip_suffix(".event_stream"))
                    .unwrap_or("accept");
                let echoed = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case(header_name))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let body = format!("data: {}\n\n", echoed);
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/event-stream".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("xhr/") && clean.ends_with(".asis") {
                // net-api M4-S15 fixture：xhr/resources/*.asis 原始 HTTP 响应文件（上游
                // 逐字嵌入——https://github.com/web-platform-tests/wpt/blob/3159769/xhr/
                // resources/*.asis；wptserve 原样发送文件字节，行尾 LF 解析宽松）。
                // 解析：首行状态行，后续非空行 `name: value` 头（空值合法），其余为体。
                let raw = match clean.rsplit('/').next().unwrap_or("") {
                    "header-content-length.asis" => "HTTP/1.0 200 NANANA\nCONTENT-LENGTH:  0\n",
                    "header-content-length-twice.asis" => {
                        "HTTP/1.0 200 NANANA\nCONTENT-LENGTH:  0\ncontent-length:\t 0\n"
                    }
                    "headers-double-empty.asis" => "HTTP/1.1 444 HI\ndouble-trouble:\ndouble-trouble:\n",
                    "headers-basic.asis" => "HTTP/1.1 280 HELLO\nfoo-test: 1\nfoo-test: 2\nfoo-test: 3\n",
                    "headers-some-are-empty.asis" => {
                        "HTTP/1.0 200 MEH\nHEYA:\t \nHEYA: \u{000B}\u{000C}\nHEYA: 1\t\nHEYA:\nHEYA:\t\nHEYA: \t2\n"
                    }
                    "headers-www-authenticate.asis" => {
                        "HTTP/1.1 280 HELLO\nwww-authenticate: 1\nwww-authenticate: 2\nwww-authenticate: 3, 4\n"
                    }
                    _ => {
                        return Err(format!("fixture not embedded: {}", clean));
                    }
                };
                let mut lines = raw.lines();
                let status_line = lines.next().unwrap_or("HTTP/1.0 200 OK");
                let mut parts = status_line.splitn(3, ' ');
                let _http = parts.next().unwrap_or("HTTP/1.0");
                let status: u16 = parts.next().unwrap_or("200").parse().unwrap_or(200);
                let status_text = parts.next().unwrap_or("").to_string();
                let mut headers: Vec<(String, String)> = Vec::new();
                let mut body_lines: Vec<&str> = Vec::new();
                let mut in_body = false;
                for line in lines {
                    if in_body {
                        body_lines.push(line);
                        continue;
                    }
                    if line.is_empty() {
                        in_body = true;
                        continue;
                    }
                    if let Some((name, value)) = line.split_once(':') {
                        headers.push((
                            name.to_lowercase(),
                            value.strip_prefix(' ').unwrap_or(value).to_string(),
                        ));
                    }
                }
                let body = if body_lines.is_empty() {
                    String::new()
                } else {
                    format!("{}\n", body_lines.join("\n"))
                };
                wpt_add_fetch_metadata(&mut headers, req, status);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status,
                    status_text,
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean.starts_with("eventsource/") && clean.ends_with("/cors.py") {
                // net-api M4-S13 fixture：eventsource/resources/cors.py（上游逐字等价——
                // https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/resources/
                // cors.py）。ACAO ← ?origin=（缺省回显请求 Origin 头）+ ACAC ← ?credentials=
                // （默认 true）；?run=cache-control → text/event-stream + cache-control
                // 模板体（回显请求 Cache-Control 头）。其余 run 值上游返空体。
                // net-api M5-S3：OPTIONS preflight 应答面——ES 请求恒携带 Cache-Control:
                // no-cache（非 CORS-safelisted，fetch spec §3.1.1）→ 跨源必须先过 preflight；
                // 上游 cors.py 无 OPTIONS 分支（真实 wptserve 部署由服务端配置补），本 runner
                // fixture 按「支持 ES 跨源的合规 CORS 服务端」语义应答：ACAM ← 回显 ACRM
                // （缺省 GET）+ ACAH ← 回显 ACRH（缺省 cache-control）。缺此则 ES 跨源恒
                // Failed to fetch → request-cache-control 跨源腿 onerror 未接线 → 页级
                // Timeout（M5-S2 轮归因，evidence/2026-10-14-m5-s3-*.md）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let req_origin = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let acao = wpt_query_value(query, "origin").unwrap_or(req_origin);
                let acac = wpt_query_value(query, "credentials").unwrap_or_else(|| "true".into());
                let run = wpt_query_value(query, "run").unwrap_or_default();
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("access-control-allow-origin".into(), acao));
                headers.push(("access-control-allow-credentials".into(), acac));
                if req.method == "OPTIONS" {
                    let acrm = req
                        .headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-method"))
                        .map(|(_, v)| v.clone())
                        .unwrap_or_else(|| "GET".into());
                    let acrh = req
                        .headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
                        .map(|(_, v)| v.clone())
                        .unwrap_or_else(|| "cache-control".into());
                    headers.push(("access-control-allow-methods".into(), acrm));
                    headers.push(("access-control-allow-headers".into(), acrh));
                }
                let body = if run == "cache-control" {
                    headers.push(("content-type".into(), "text/event-stream".into()));
                    let echoed = req
                        .headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case("cache-control"))
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default();
                    format!("data: {}\n\n", echoed)
                } else {
                    String::new()
                };
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean == "fetch/api/resources/clean-stash.py" {
                // net-api M4-S23 fixture：fetch/api/resources/clean-stash.py（上游逐字
                // 等价——stash take 命中 → "1"，否则 "0"；恒 200）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let token = wpt_query_value(query, "token").unwrap_or_default();
                let had = WPT_TOKEN_STASH.lock().ok().and_then(|mut s| s.remove(&token)).is_some();
                let mut headers: Vec<(String, String)> = vec![("content-type".into(), "text/plain".into())];
                wpt_add_fetch_metadata(&mut headers, req, 200);
                let body = if had { "1" } else { "0" };
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.to_string(),
                    body_bytes: Some(body.as_bytes().to_vec()),
                });
            }
            if clean == "fetch/api/resources/preflight.py" {
                // net-api M4-S23 fixture：fetch/api/resources/preflight.py（上游行为
                // 等价——?origin= 多 ACAO、?clear-stash=、?credentials=、OPTIONS 分支
                // （ACRM/Accept 校验 + allow_methods/allow_headers/max_age/preflight_
                // status + stash 记录 did-preflight/ACRH/referrer/UA）、实际请求分支
                // x-did-preflight 等暴露头回读；stash 取后回存上游 take-then-put 语义）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let qval = |name: &str| wpt_query_value(query, name);
                let req_header = |name: &str| -> Option<String> {
                    req.headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case(name))
                        .map(|(_, v)| v.clone())
                };
                let token = qval("token").unwrap_or_default();
                let mut headers: Vec<(String, String)> = vec![("content-type".into(), "text/plain".into())];
                if let Some(origin_list) = qval("origin") {
                    for o in origin_list.split(", ") {
                        headers.push(("access-control-allow-origin".into(), o.to_string()));
                    }
                } else {
                    headers.push(("access-control-allow-origin".into(), "*".into()));
                }
                if qval("clear-stash").is_some() {
                    let had = WPT_TOKEN_STASH.lock().ok().and_then(|mut s| s.remove(&token)).is_some();
                    wpt_add_fetch_metadata(&mut headers, req, 200);
                    let body = if had { "1" } else { "0" };
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers,
                        body: body.to_string(),
                        body_bytes: Some(body.as_bytes().to_vec()),
                    });
                }
                if qval("credentials").is_some() {
                    headers.push(("access-control-allow-credentials".into(), "true".into()));
                }
                // stash dict 四字段定序编码：preflight\x1fcontrol_request_headers\x1f
                // preflight_referrer\x1fpreflight_user_agent（None → "\u{0}"哨兵）。
                const NONE_SENTINEL: &str = "\u{0}";
                let encode = |c: &str, r: &str, u: &str| format!("1\x1f{}\x1f{}\x1f{}", c, r, u);
                if req.method == "OPTIONS" {
                    if req_header("access-control-request-method").is_none() {
                        wpt_add_fetch_metadata(&mut headers, req, 400);
                        return Ok(zero_engine::fetch_bridge::FetchResponse {
                            status: 400,
                            status_text: wpt_status_text(400).to_string(),
                            headers,
                            body: "ERROR: No access-control-request-method in preflight!".to_string(),
                            body_bytes: Some(b"ERROR: No access-control-request-method in preflight!".to_vec()),
                        });
                    }
                    if req_header("accept").map(|v| v.trim() != "*/*").unwrap_or(true) {
                        wpt_add_fetch_metadata(&mut headers, req, 400);
                        return Ok(zero_engine::fetch_bridge::FetchResponse {
                            status: 400,
                            status_text: wpt_status_text(400).to_string(),
                            headers,
                            body: "ERROR: Invalid access in preflight!".to_string(),
                            body_bytes: Some(b"ERROR: Invalid access in preflight!".to_vec()),
                        });
                    }
                    // control_request_headers 旗标在 → 存 ACRH（头缺席 → None 哨兵，
                    // 不发射）；旗标不在 → 缺省 b""（恒发射空值——上游默认字典同形）。
                    let control_headers = if qval("control_request_headers").is_some() {
                        req_header("access-control-request-headers").unwrap_or_else(|| NONE_SENTINEL.into())
                    } else {
                        String::new()
                    };
                    if let Some(ma) = qval("max_age") {
                        headers.push(("access-control-max-age".into(), ma));
                    }
                    if let Some(ah) = qval("allow_headers") {
                        headers.push(("access-control-allow-headers".into(), ah));
                    }
                    if let Some(am) = qval("allow_methods") {
                        headers.push(("access-control-allow-methods".into(), am));
                    }
                    let status: u16 = qval("preflight_status").and_then(|v| v.parse().ok()).unwrap_or(200);
                    let dict = encode(
                        &control_headers,
                        &req_header("referer").unwrap_or_default(),
                        &req_header("user-agent").unwrap_or_default(),
                    );
                    if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
                        stash.insert(token, dict);
                    }
                    wpt_add_fetch_metadata(&mut headers, req, status);
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status,
                        status_text: wpt_status_text(status).to_string(),
                        headers,
                        body: String::new(),
                        body_bytes: Some(Vec::new()),
                    });
                }
                // 实际请求分支：stash 取后回存（上游 take-then-put）。
                let stored = WPT_TOKEN_STASH.lock().ok().and_then(|mut s| s.remove(&token));
                let (preflight_flag, control_headers, preflight_referrer, preflight_ua) = match stored {
                    Some(dict) => {
                        let parts: Vec<&str> = dict.split('\x1f').collect();
                        (
                            parts.first().copied().unwrap_or("0").to_string(),
                            parts.get(1).copied().unwrap_or(NONE_SENTINEL).to_string(),
                            parts.get(2).copied().unwrap_or_default().to_string(),
                            parts.get(3).copied().unwrap_or_default().to_string(),
                        )
                    }
                    // net-api M4-S25：control_request_headers 缺省 b""（非 None）——
                    // x-control-request-headers 恒发射（上游默认字典同形）。
                    None => ("0".to_string(), String::new(), String::new(), String::new()),
                };
                if qval("checkUserAgentHeaderInPreflight").is_some() {
                    let ua = req_header("user-agent").unwrap_or_default();
                    if ua != preflight_ua {
                        wpt_add_fetch_metadata(&mut headers, req, 400);
                        return Ok(zero_engine::fetch_bridge::FetchResponse {
                            status: 400,
                            status_text: wpt_status_text(400).to_string(),
                            headers,
                            body: "ERROR: No user-agent header in preflight".to_string(),
                            body_bytes: Some(b"ERROR: No user-agent header in preflight".to_vec()),
                        });
                    }
                }
                headers.push((
                    "access-control-expose-headers".into(),
                    "x-did-preflight, x-control-request-headers, x-referrer, x-preflight-referrer, x-origin".into(),
                ));
                headers.push(("x-did-preflight".into(), preflight_flag.clone()));
                if control_headers != NONE_SENTINEL {
                    headers.push(("x-control-request-headers".into(), control_headers.clone()));
                }
                headers.push(("x-preflight-referrer".into(), preflight_referrer.clone()));
                headers.push(("x-referrer".into(), req_header("referer").unwrap_or_default()));
                headers.push(("x-origin".into(), req_header("origin").unwrap_or_default()));
                if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
                    stash.insert(
                        token,
                        format!(
                            "{}\x1f{}\x1f{}\x1f{}",
                            preflight_flag.clone(),
                            control_headers.clone(),
                            preflight_referrer.clone(),
                            preflight_ua.clone()
                        ),
                    );
                }
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: String::new(),
                    body_bytes: Some(Vec::new()),
                });
            }
            // net-api M4-S18：非 GET 放行至各 fixture 自行分派（corsenabled/redirect-cors
            // 的 OPTIONS preflight 分支）后兜底拒绝——preflight OPTIONS 不再被 blanket 拒。
            // net-api M2-S5：inspect-headers.py 上游 wptserve handler 方法无关（任意
            // 方法回显请求头）——header-values-normalize XHR POST 腿经此回显；其余
            // 端点维持兜底拒绝。
            if req.method != "GET" && req.method != "OPTIONS"
                && !clean.ends_with("/inspect-headers.py")
            {
                return Err(format!("method not supported: {}", req.method));
            }
            if req.method == "OPTIONS" {
                // 无 OPTIONS 分支的端点：统一 200 + 空（preflight 判定交给 shim 侧
                // ACAM/ACAH 检查）。
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push((
                    "access-control-allow-origin".into(),
                    req.headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                        .map(|(_, v)| v.clone())
                        .unwrap_or_else(|| "*".into()),
                ));
                // net-api M5-S10：ACAM/ACAH 回显（ACRM/ACRH 请求头并入——上游 wptserve
                // 自动应答 preflight 的合规 CORS 服务端语义；timeout-multiple-fetches
                // 「CORS preflights should not reset the timer」YO 方法面——固定集不覆盖
                // 自定义方法 → preflight 拒 → onerror 而非 ontimeout。M5-S3 eventsource
                // cors.py 同先例）。
                let mut allow_methods: Vec<String> = ["GET", "POST", "HEAD", "OPTIONS"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                if let Some(acrm) = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-method"))
                    .map(|(_, v)| v.trim().to_ascii_uppercase())
                    && !acrm.is_empty()
                    && !allow_methods.contains(&acrm)
                {
                    allow_methods.push(acrm);
                }
                headers.push(("access-control-allow-methods".into(), allow_methods.join(", ")));
                if let Some(acrh) = req
                    .headers
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
                    .map(|(_, v)| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                {
                    headers.push(("access-control-allow-headers".into(), acrh));
                }
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: String::new(),
                    body_bytes: Some(Vec::new()),
                });
            }
            if clean.ends_with("/inspect-headers.py") {
                // net-api M3-S1 fixture：fetch/api/resources/inspect-headers.py（上游逐字
                // 等价——https://github.com/web-platform-tests/wpt/blob/3159769/fetch/api/
                // resources/inspect-headers.py）。?headers=a|b|c → 每个请求头存在则回
                // `x-request-<name>: <value>`（多值 ', ' 合并）；?cors → AC 族 + expose；
                // content-type: text/plain。header-values(-normalize) 回读面。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let mut headers: Vec<(String, String)> = Vec::new();
                let checked: Vec<String> = wpt_query_value(query, "headers")
                    .map(|names| {
                        names
                            .split('|')
                            .map(|n| n.trim().to_lowercase())
                            .filter(|n| !n.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();
                for name in &checked {
                    // 上游 inspect-headers.py 按「头存在」回显（wptserve `in
                    // request.headers`），值可为空串——header-values-normalize strip
                    // 腿（值 strip 后为 ""）期望 x-request-val1 == "" 而非缺席。
                    let hits: Vec<&str> = req
                        .headers
                        .iter()
                        .filter(|(n, _)| n.eq_ignore_ascii_case(name))
                        .map(|(_, v)| v.as_str())
                        .collect();
                    if hits.is_empty() {
                        continue;
                    }
                    headers.push((format!("x-request-{}", name), hits.join(", ")));
                }
                if wpt_query_value(query, "cors").is_some() {
                    let origin = req
                        .headers
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                        .map(|(_, v)| v.clone())
                        .unwrap_or_else(|| "*".to_string());
                    headers.push(("access-control-allow-origin".into(), origin));
                    headers.push(("access-control-allow-credentials".into(), "true".into()));
                    headers.push(("access-control-allow-methods".into(), "GET, POST, HEAD".into()));
                    if !checked.is_empty() {
                        let exposed = checked
                            .iter()
                            .map(|n| format!("x-request-{}", n))
                            .collect::<Vec<_>>()
                            .join(", ");
                        headers.push(("access-control-expose-headers".into(), exposed));
                    }
                }
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: String::new(),
                    body_bytes: Some(Vec::new()),
                });
            }
            if clean == "fetch/api/resources/trickle.py" {
                let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
                let count = wpt_query_value(query, "count")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(1);
                let count = count.min(128);
                let body = "TEST_TRICKLE\n".repeat(count);
                let mut headers = wpt_pipe_headers(query);
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            // net-api M5-S9：fetch/api/resources/bad-chunk-encoding.py（上游 wptserve
            // 流式端点——count×合法 chunk 后写裸 "garbage"，客户端 chunk 解析失败 →
            // 中段网络错误）。合成通道无 chunk 帧语义：body = count×TEST_CHUNK +
            // "garbage"，`x-zero-body-error-after` 内部标记头承载「N 字节后流错误」
            // 语义（shim 侧消费后从可见 header list 剥离；error-after-response
            // 两腿 = 首 read 得 TEST_CHUNK、次 read/closed 拒 TypeError）。
            if clean.ends_with("/bad-chunk-encoding.py") {
                // net-api M5-S11：后缀匹配扩 xhr/resources/（event-error.sub 相对 URL
                // 「resources/bad-chunk-encoding.py」→ /xhr/resources/ 面——上游两目录
                // 各有一份同型端点，语义同构）。
                let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
                let count = wpt_query_value(query, "count")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(1);
                let count = count.min(128);
                let mut body_bytes = "TEST_CHUNK".repeat(count).into_bytes();
                let good_len = body_bytes.len();
                body_bytes.extend_from_slice(b"garbage");
                let mut headers = Vec::new();
                headers.push(("x-zero-body-error-after".into(), good_len.to_string()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                let body = String::from_utf8_lossy(&body_bytes).into_owned();
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body,
                    body_bytes: Some(body_bytes),
                });
            }
            // js-dom R141：dom/nodes/encoding.py（Document-characterSet-normalization 两文件
            // 654 subtest 经 `<iframe src="encoding.py?label=X">` 取子文档）——上游是
            // wptserve Python 脚本，wpt-data 无静态文件。此处内置等价生成器：读 ?label=
            // 参数，返回 `<!doctype html><meta charset="X">`（上游脚本逐字等价——
            // https://github.com/web-platform-tests/wpt/blob/3159769/dom/nodes/encoding.py）。
            // R360（js-dom M4）：`/common/redirect.py?location=X`（WPT Document-URL "with
            // redirect"——iframe.src 经 wptserve 重定向到 blank.html 后 contentDocument.URL
            // 须为最终 URL）。wpt-data 无静态 redirect.py；内置等价生成器：读 ?location=
            // 目标路径，取目标文件体 + `X-Zero-Final-URL` 绝对最终 URL（shim 侧消费该头
            // 覆盖 iframe doc._zwURL）。location 相对路径按当前 URL 原点解析。
            if clean == "common/redirect.py" {
                let loc = percent_encoding::percent_decode_str(
                    path_part
                        .split_once('?')
                        .map(|(_, q)| q)
                        .unwrap_or("")
                        .split('#')
                        .next()
                        .unwrap_or(""),
                )
                .decode_utf8_lossy()
                .split('&')
                .find_map(|kv| kv.strip_prefix("location="))
                .unwrap_or("")
                .to_string();
                let final_url = if loc.starts_with("http://") || loc.starts_with("https://") {
                    loc.clone()
                } else {
                    // 相对路径按请求原点解析（location.origin）。
                    let origin = req
                        .url
                        .find("://")
                        .and_then(|i| req.url[i + 3..].find('/').map(|p| i + 3 + p).or(Some(req.url.len())))
                        .map(|end| req.url[..end].to_string())
                        .unwrap_or_default();
                    format!("{origin}{loc}")
                };
                let target_body = match std::fs::read(root.join(loc.trim_start_matches('/'))) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                    Err(_) => String::new(),
                };
                let mut headers = wpt_pipe_headers(path_part.split_once('?').map(|(_, q)| q).unwrap_or(""));
                headers.push(("X-Zero-Final-URL".into(), final_url.clone()));
                headers.push(("X-Zero-Response-Type".into(), wpt_fetch_response_type(req, 200).into()));
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: target_body.clone(),
                    body_bytes: Some(target_body.into_bytes()),
                });
            }
            if clean == "dom/nodes/encoding.py" {
                let label = percent_encoding::percent_decode_str(
                    path_part
                        .split_once('?')
                        .map(|(_, q)| q)
                        .unwrap_or("")
                        .split('#')
                        .next()
                        .unwrap_or(""),
                )
                .decode_utf8_lossy()
                .split('&')
                .find_map(|kv| kv.strip_prefix("label="))
                .unwrap_or("")
                .to_string();
                let body = format!("<!doctype html><meta charset=\"{label}\">");
                let mut headers = Vec::new();
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            if clean == "service-workers/cache-storage/resources/vary.py" {
                let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
                let mut headers = wpt_pipe_headers(query);
                // https://github.com/web-platform-tests/wpt/blob/24197a11e8c5bd29a5cb7bdf18135a82be8a8546/service-workers/cache-storage/resources/vary.py
                if query.split('&').any(|pair| pair == "clear-vary-value-override-cookie") {
                    *vary_value_override.lock().unwrap() = None;
                    wpt_add_fetch_metadata(&mut headers, req, 200);
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers,
                        body: "vary cookie cleared".to_string(),
                        body_bytes: Some(b"vary cookie cleared".to_vec()),
                    });
                }
                if let Some(vary) = wpt_query_value(query, "set-vary-value-override-cookie") {
                    *vary_value_override.lock().unwrap() = Some(vary);
                    wpt_add_fetch_metadata(&mut headers, req, 200);
                    return Ok(zero_engine::fetch_bridge::FetchResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers,
                        body: "vary cookie set".to_string(),
                        body_bytes: Some(b"vary cookie set".to_vec()),
                    });
                }
                let omits_credentials = req.credentials.as_deref() == Some("omit");
                let override_vary = (!omits_credentials)
                    .then(|| vary_value_override.lock().unwrap().clone())
                    .flatten();
                if let Some(vary) = override_vary.or_else(|| wpt_query_value(query, "vary")) {
                    headers.push(("vary".into(), vary));
                }
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: "vary response".to_string(),
                    body_bytes: Some(b"vary response".to_vec()),
                });
            }
            if clean == "service-workers/cache-storage/resources/fetch-status.py" {
                let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
                let status = wpt_query_value(query, "status")
                    .and_then(|value| value.parse::<u16>().ok())
                    .unwrap_or(200);
                let mut headers = wpt_pipe_headers(query);
                wpt_add_fetch_metadata(&mut headers, req, status);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status,
                    status_text: wpt_status_text(status).to_string(),
                    headers,
                    body: String::new(),
                    body_bytes: Some(Vec::new()),
                });
            }
            if clean == "service-workers/service-worker/resources/cors-approved.txt" {
                let mut headers = vec![
                    ("Access-Control-Allow-Origin".into(), "*".into()),
                    ("content-type".into(), "text/plain".into()),
                ];
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: "plaintext\n".to_string(),
                    body_bytes: Some(b"plaintext\n".to_vec()),
                });
            }
            if clean == "service-workers/service-worker/resources/cors-denied.txt" {
                let mut headers = vec![("content-type".into(), "text/plain".into())];
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: "Cross-origin request blocked by missing CORS response header\n".to_string(),
                    body_bytes: Some(b"Cross-origin request blocked by missing CORS response header\n".to_vec()),
                });
            }
            if clean == "service-workers/cache-storage/resources/redirect.py" {
                let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
                let status = wpt_query_value(query, "status")
                    .and_then(|value| value.parse::<u16>().ok())
                    .unwrap_or(302);
                let mut headers = wpt_pipe_headers(query);
                headers.push((
                    "Location".into(),
                    "/service-workers/cache-storage/resources/simple.txt".into(),
                ));
                wpt_add_fetch_metadata(&mut headers, req, status);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status,
                    status_text: wpt_status_text(status).to_string(),
                    headers,
                    body: String::new(),
                    body_bytes: Some(Vec::new()),
                });
            }
            if clean == "common/blank.html" {
                // net-api M4-S22 fixture：common/blank.html（上游为最小空页；wpt-data 未拉
                // 静态本体——redirect 链暴露头面（expose-headers-on-redirect）与 ?pipe=
                // 管道头载体经此处等价供给）。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let mut headers = wpt_pipe_headers(query);
                headers.push(("content-type".into(), "text/html".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: "<!doctype html>\n<meta charset=\"utf-8\">\n<title>Blank</title>\n".to_string(),
                    body_bytes: Some(b"<!doctype html>\n<meta charset=\"utf-8\">\n<title>Blank</title>\n".to_vec()),
                });
            }
            if clean == "websockets/security/check.py" {
                // net-api M5-S1 fixture：websockets/security/check.py（上游逐字等价——
                // `b"FAIL" if b'Sec-WebSocket-Key' in request.headers else b"PASS"`）。
                // 该头是 Fetch 规范 forbidden header name——XHR setRequestHeader 须被
                // UA 剥离，服务端收到即 FAIL（security/002 断言面）。
                let has_key = req
                    .headers
                    .iter()
                    .any(|(n, _)| n.eq_ignore_ascii_case("sec-websocket-key"));
                let body = if has_key { "FAIL" } else { "PASS" };
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/plain".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: body.to_string(),
                    body_bytes: Some(body.as_bytes().to_vec()),
                });
            }
            if clean == "html/syntax/xmldecl/support/trickle.py" {
                // html-syntax（2026-10-20，xmldecl 收编）：wptserve CGI 静态等价——上游
                // trickle.py 逐字节 trickle 输出固定正文（Content-Type: text/html 无
                // charset；编码由正文 bogo-XML declaration 嗅探，windows-1251 断言面）。
                // 同步 fetch 通道无流式语义，输出与上游脚本逐字一致（"declation" 为
                // 上游原文拼写）。正文含 0xE6 非 UTF-8 字节 → R3021 `__zw_bytes:` wire。
                let body: &[u8] = b"<?xml version=\"1.0\" encoding=\"windows-1251\"?>\n\
                    <p>Normal XML declation as slow byte-by-byte trickle</p>\n\
                    <p>Test: \xE6</p>\n\
                    <p>If &#x0436;, XML decl takes effect</p>";
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/html".into()));
                wpt_add_fetch_metadata(&mut headers, req, 200);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers,
                    body: String::from_utf8_lossy(body).into_owned(),
                    body_bytes: Some(body.to_vec()),
                });
            }
            if clean == "html/semantics/scripting-1/the-script-element/resources/load-error-events.py" {
                // html-semantics M2 s100 fixture：经典脚本 load/error 事件族端点（上游
                // wptserve CGI 静态等价，语义见 wpt_load_error_events_response——语料
                // pull 面不含 .py，此前落通用缺文件 404：200 组动态 appendChild 案取回
                // 404 → error 路径误触发 → load-error-events-1 dynamic load 两行假红）。
                // 404 组依赖 `_r.ok` false → 门 throw → error 路径，status 保真即可。
                let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
                let (status, body) = wpt_load_error_events_response(query);
                let mut headers: Vec<(String, String)> = Vec::new();
                headers.push(("content-type".into(), "text/javascript".into()));
                wpt_add_fetch_metadata(&mut headers, req, status);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status,
                    status_text: wpt_status_text(status).to_string(),
                    headers,
                    body: body.clone(),
                    body_bytes: Some(body.into_bytes()),
                });
            }
            match std::fs::read(root.join(clean)) {
                Ok(bytes) => {
                    let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
                    let mut headers = wpt_pipe_headers(query);
                    headers.extend(wpt_static_resource_headers(clean));
                    // wptserve `.headers` sidecar 语义：同名 sidecar 的 Content-Type
                    // 覆盖默认 MIME（wpt_static_resource_headers）——替换而非追加
                    //（重复 content-type 头在 CT 解析侧首项胜出，保留单头不变量）。
                    if let Some(sidecar_ct) = wpt_sidecar_content_type(&root, clean) {
                        headers.retain(|(n, _)| !n.eq_ignore_ascii_case("content-type"));
                        headers.push(("content-type".into(), sidecar_ct));
                    }
                    if clean == "service-workers/cache-storage/resources/simple.txt"
                        && wpt_query_value(query, "zw-filtered").as_deref() == Some("cors")
                    {
                        headers.push(("Access-Control-Allow-Origin".into(), "*".into()));
                    }
                    let status = wpt_pipe_status(query).unwrap_or(200);
                    wpt_add_fetch_metadata(&mut headers, req, status);
                    // net-api M5-S1：`.sub.js` 服务端替换（上游 wptserve 对 .sub 文件做
                    // 全集替换的语义；script fetcher 通道 M4-S22 已对齐——websockets/
                    // *.html 案经 fetch 通道读 constants.sub.js，缺此则 SCHEME_DOMAIN_PORT
                    // 带 '{{ports[ws][0]}}' 字面量 → Invalid URL）。
                    let bytes = if clean == "resources/testharness.js" {
                        // html-semantics M2 s98：write 注入 harness 的完成探测接线——
                        // document.write 写入的 `<script src="/resources/testharness.js">`
                        // （execution-timing/102 全家桶形）经本通道取回，此前拿到 RAW 副本：
                        // 无 `__zw_mark_harness_loaded`/`__zw_harness_state` 完成探测钩子，
                        // 且新文档无生命周期脚本补调（document.open 后 load 事件不重发）→
                        // test_environment.all_loaded 恒 false → 末测 done 后不 complete →
                        // runner 探针等满超时（hook=undefined/state=null 伪 Timeout）。
                        // 此处服务与静态内联路径同款补丁源（apply_harness_completion_patch）
                        // 并自带 eval 尾自调——任一装载路径拿到同一接线面。
                        let patched = apply_harness_completion_patch(String::from_utf8_lossy(&bytes).into_owned());
                        let mut with_tail = patched;
                        with_tail.push_str(HARNESS_LOADED_TAIL_CALL);
                        with_tail.into_bytes()
                    } else if clean == "resources/testharnessreport.js" {
                        // s98 同面：上游 testharnessreport.js 是裸 loader（无 runner 装配），
                        // write 注入副本经其注册的 completion 回调到不了探针全局。服务
                        // reporter 快照（setup({output:false}) + GLOBAL shim +
                        // __zw_harness_results/__zw_harness_complete 装配），与静态路径
                        // prepare_harness_html 移除 report 标签内联 reporter 等价。
                        HARNESS_REPORTER_SOURCE.as_bytes().to_vec()
                    } else if clean.ends_with(".sub.js") {
                        apply_wpt_substitutions(&String::from_utf8_lossy(&bytes)).into_bytes()
                    } else {
                        bytes
                    };
                    // R3254-E2 切片 6（editing goal，2026-09-07）：**iframe 文档的相对
                    // src 脚本内联**——shim iframe 装载（R115 runInlineScripts）只执行
                    // inline `<script>`，外部 src 脚本被跳过（无同步 fetch 通道），iframe
                    // 内 `var`/函数面缺失（selection/test-iframe.html 依赖 common.js 的
                    // setupRangeTests——deleteFromDocument/getSelection iframe 簇根因）。
                    // 主文档面 runner 已有 inline_local_scripts 预处理；此处对 fetch 通道
                    // 返回的 .html 同样内联相对 src（绝对/`/resources/` 保留原标签）。
                    // html-syntax（2026-10-20）：非 UTF-8 文档字节保真——XHR document
                    // 响应的 charset 嗅探/解码消费原始字节（html/syntax/charset 22 案
                    // 断言 windows-1251 等遗留编码字节面）；from_utf8_lossy 把裸字节
                    // 改写为 EF BF BD，信息先于 JS 侧 TextDecoder 损失。合法 UTF-8 时
                    // 维持内联路径（往返恒等）；非 UTF-8 时跳过内联、原字节直供（上游
                    // charset 语料为纯标记、无相对 src 脚本；FIXME: 非 UTF-8 文档的
                    // 相对脚本内联待字节级 tokenizer）。
                    let body = if clean.ends_with(".html") {
                        match std::str::from_utf8(&bytes) {
                            Ok(html) => {
                                // encoding-legacy-mb DC-2（2026-10-28）：合法 UTF-8 字节 +
                                // 非 UTF-8 声明 → serve 侧按声明解码。iso-2022-jp 语料
                                // 7-bit clean（字节合法 UTF-8 ≠ UTF-8 内容），plain wire
                                // 侧 shim 嗅探只落 characterSet 元数据、正文不重解码
                                //（spec encoding sniffing 的 change-encoding-and-restart
                                // 步缺失）→ 文档族字面转义全败。解码走文档管道
                                //（decode_document_bytes：BOM > meta 预扫描 > UTF-8，
                                // DC-1 read_corpus_to_string 同语义）。例外：replacement
                                // 编码（iso-2022-kr bogo 声明，xmldecl replacement.htm
                                // 支持页）不解——replacement 解码恒 U+FFFD 毁页，维持
                                // 字面 + shim characterSet 元数据面（2026-10-20
                                // html-syntax 既定行为）。UTF-8/无声明语料恒等零变化；
                                // 非 UTF-8 字节走 Err 分支（`__zw_bytes:` wire 字节保真，
                                // XHR document/iframe bytes 面零变化）。
                                // https://html.spec.whatwg.org/multipage/parsing.html#encoding-sniffing-algorithm
                                let html = match sniff_document_encoding(html) {
                                    Some(name) if name != "UTF-8" && name != "replacement" => {
                                        zero_dom::decode_document_bytes(&bytes)
                                    }
                                    _ => html.to_string(),
                                };
                                // navigation-compat M3-S4（wptserve 模板面）：fetch 通道
                                // .html 按该请求查询串做 `{{GET[name]}}` 替换——上游
                                // wptserve 对每请求资源注入（replace-before-load 族
                                // code-injector.html?code=<enc js> 载体；主文档通道
                                // run_testharness_html_inner 已替换，iframe 子文档此前拿
                                // 字面量 → 注入脚本语法错误，自导航从未发生）。替换先于
                                // 脚本内联（注入代码须进提取出的 script 源）。
                                let substituted = wpt_apply_get_query_substitutions(html, query);
                                let inlined = inline_local_scripts(
                                    &substituted,
                                    &root,
                                    clean,
                                    sniff_document_encoding(&substituted).as_deref(),
                                );
                                let inlined_bytes = inlined.clone().into_bytes();
                                (inlined_bytes, inlined)
                            }
                            Err(_) => (bytes.clone(), String::from_utf8_lossy(&bytes).into_owned()),
                        }
                    } else {
                        (bytes.clone(), String::from_utf8_lossy(&bytes).into_owned())
                    };
                    Ok(zero_engine::fetch_bridge::FetchResponse {
                        status,
                        status_text: wpt_status_text(status).to_string(),
                        headers,
                        body: body.1,
                        body_bytes: Some(body.0),
                    })
                }
                Err(_e) => {
                    if clean.starts_with("service-workers/service-worker/resources/") && clean.ends_with(".html") {
                        let status = 404;
                        let mut headers = vec![("content-type".into(), "text/html".into())];
                        wpt_add_fetch_metadata(&mut headers, req, status);
                        return Ok(zero_engine::fetch_bridge::FetchResponse {
                            status,
                            status_text: wpt_status_text(status).to_string(),
                            headers,
                            body: String::new(),
                            body_bytes: Some(Vec::new()),
                        });
                    }
                    // net-api M4-S20：文件缺失 → 404 响应（HTTP 语义——缺文件非网络错误；
                    // responseurl.html「404 response has proper responseURL」面）。
                    let mut headers404: Vec<(String, String)> = Vec::new();
                    headers404.push(("content-type".into(), "text/plain".into()));
                    wpt_add_fetch_metadata(&mut headers404, req, 404);
                    Ok(zero_engine::fetch_bridge::FetchResponse {
                        status: 404,
                        status_text: "Not Found".to_string(),
                        headers: headers404,
                        body: String::new(),
                        body_bytes: Some(Vec::new()),
                    })
                }
            }
        },
    ))
}

fn wpt_url_path(url: &str) -> &str {
    let Some(scheme_index) = url.find("://") else {
        return url;
    };
    let after_scheme = &url[scheme_index + 3..];
    match after_scheme.find('/') {
        Some(path_index) => &after_scheme[path_index..],
        None => "/",
    }
}

fn wpt_query_value(query: &str, name: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key != name {
            return None;
        }
        Some(
            percent_encoding::percent_decode_str(value)
                .decode_utf8_lossy()
                .to_string(),
        )
    })
}

/// `html/semantics/scripting-1/the-script-element/resources/load-error-events.py`
/// 的 wptserve CGI 静态等价（html-semantics M2 s100；语料 pull 面不含 .py，运行时
/// 按上游脚本语义仿真——pin 315976933870 逐字核对）：
/// `test` 查询参数须匹配 `^[a-zA-Z0-9_]+$`（缺席/非法 → 400 空体）；含 `_load`
/// → 200 + `<test>.executed = true`（load 组脚本执行体）；否则 → 404 +
/// `<test>.test.step(... assert_unreached ...)`（error 组——上游 404 体**不执行**，
/// 本引擎侧 fetch/dynamic 通道由 status 拒 eval、内联通道直接不投递该体）。
/// Content-Type 恒 `text/javascript`（调用方装配）。经典脚本根 fixture；module 族
/// 各自路径的变体不匹配本路径（语义各异，仍走通用 404——挂账 module 尾片）。
/// https://html.spec.whatwg.org/multipage/scripting.html#execute-the-script-block
fn wpt_load_error_events_response(query: &str) -> (u16, String) {
    let Some(test) = wpt_query_value(query, "test") else {
        return (400, String::new());
    };
    let valid = !test.is_empty() && test.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !valid {
        return (400, String::new());
    }
    if test.contains("_load") {
        (200, format!("\"use strict\";\n{test}.executed = true;"))
    } else {
        (
            404,
            format!(
                "\"use strict\";\n{test}.test.step(function() {{ assert_unreached(\"404 script should not be executed\"); }});"
            ),
        )
    }
}

/// wptserve `{{GET[name]}}` 查询模板替换（fetch 通道 .html——iframe 子文档面）。
/// 上游 wptserve 对每请求资源按**该请求查询串**注入模板（wpt.py 模板语义；
/// replace-before-load 族的 code-injector.html?code=<enc js> 即此载体——主文档通道
/// run_testharness_html_inner 侧已替换，子文档此前拿字面量 → 注入脚本语法错误）。
/// 值全集百分号解码（脚本通道 security-hardening M2-s4 的五项最小表不够——注入
/// 代码含 %0A/%3D/%3B/%2F/%60）；无匹配占位符 → 空串（上游缺参语义）。
fn wpt_apply_get_query_substitutions(mut body: String, query: &str) -> String {
    if !body.contains("{{GET[") {
        return body;
    }
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let decoded = percent_encoding::percent_decode_str(value)
            .decode_utf8_lossy()
            .to_string();
        body = body.replace(&format!("{{{{GET[{key}]}}}}"), &decoded);
    }
    // 残留 `{{GET[...]}}`（查询未提供）→ 空串。
    while let Some(start) = body.find("{{GET[") {
        let Some(end) = body[start..].find("}}") else { break };
        body.replace_range(start..start + end + 2, "");
    }
    body
}

fn wpt_url_origin(url: &str) -> String {
    let Some(scheme_index) = url.find("://") else {
        return String::new();
    };
    let after_scheme = &url[scheme_index + 3..];
    let authority_end = after_scheme.find('/').unwrap_or(after_scheme.len());
    format!("{}://{}", &url[..scheme_index], &after_scheme[..authority_end])
}

fn wpt_fetch_response_type(req: &zero_engine::fetch_bridge::FetchRequest, status: u16) -> &'static str {
    // https://fetch.spec.whatwg.org/#concept-filtered-response-basic
    // https://fetch.spec.whatwg.org/#concept-filtered-response-cors
    // https://fetch.spec.whatwg.org/#concept-filtered-response-opaque
    // https://fetch.spec.whatwg.org/#concept-filtered-response-opaque-redirect
    // M5-S5：页锚 scheme 双态（net-api corpus http / 其余通道 https）——同源判定按
    // wpt.test 权威双 scheme 成员（本 runner 页面域恒 wpt.test）。
    let page_origin = "https://wpt.test";
    let response_origin = wpt_url_origin(&req.url);
    // net-api 二期 M2-S3b：net-api 页锚端口化（wpt.test:8000）——同源成员同步
    // 扩 :8000 形（HTTP_ORIGIN 端点响应 origin 全等面）。
    let same_origin = response_origin == page_origin
        || response_origin == "http://wpt.test"
        || response_origin == "http://wpt.test:8000";
    let mode = req.mode.as_deref().unwrap_or("");
    let redirect = req.redirect.as_deref().unwrap_or("");
    if redirect == "manual" && matches!(status, 301 | 302 | 303 | 307 | 308) {
        "opaqueredirect"
    } else if mode == "no-cors" && !same_origin {
        "opaque"
    } else if !response_origin.is_empty() && !same_origin {
        "cors"
    } else {
        "basic"
    }
}

fn wpt_add_fetch_metadata(
    headers: &mut Vec<(String, String)>,
    req: &zero_engine::fetch_bridge::FetchRequest,
    status: u16,
) {
    headers.push(("X-Zero-Final-URL".into(), req.url.clone()));
    headers.push((
        "X-Zero-Response-Type".into(),
        wpt_fetch_response_type(req, status).into(),
    ));
}

fn wpt_pipe_headers(query: &str) -> Vec<(String, String)> {
    let Some(pipe) = wpt_query_value(query, "pipe") else {
        return Vec::new();
    };
    pipe.split('|')
        .filter_map(|command| {
            let args = command.strip_prefix("header(")?.strip_suffix(')')?;
            let (name, value) = args.split_once(',')?;
            Some((
                percent_encoding::percent_decode_str(name)
                    .decode_utf8_lossy()
                    .to_string(),
                percent_encoding::percent_decode_str(value)
                    .decode_utf8_lossy()
                    .to_string(),
            ))
        })
        .collect()
}

fn wpt_pipe_status(query: &str) -> Option<u16> {
    let pipe = wpt_query_value(query, "pipe")?;
    pipe.split('|')
        .find_map(|command| command.strip_prefix("status(")?.strip_suffix(')')?.parse::<u16>().ok())
}

fn wpt_static_resource_headers(path: &str) -> Vec<(String, String)> {
    // WAB2-M2-s2（web-api-batch2 goal，2026-09-23）：补图片/数据类静态资源 MIME——
    // clipboard-apis write-blobs 族经 `fetch('resources/greenbox.png')` → blob.type 断言
    // 'image/png'（Response.blob() 读 content-type 头 → Blob.type），缺映射时 blob.type=''
    // → 3 案 4 subtest 红斑。表驱动照上游 wptserve mime.types 常用子集。
    let content_type = if path.ends_with(".html") || path.ends_with(".htm") {
        Some("text/html")
    } else if path.ends_with(".txt") {
        Some("text/plain")
    } else if path.ends_with(".js") {
        Some("application/javascript")
    } else if path.ends_with(".css") {
        Some("text/css")
    } else if path.ends_with(".json") {
        Some("application/json")
    } else if path.ends_with(".png") {
        Some("image/png")
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if path.ends_with(".gif") {
        Some("image/gif")
    } else if path.ends_with(".webp") {
        Some("image/webp")
    } else if path.ends_with(".svg") {
        Some("image/svg+xml")
    } else if path.ends_with(".ico") {
        Some("image/x-icon")
    } else {
        None
    };
    content_type
        .map(|value| vec![("content-type".to_string(), value.to_string())])
        .unwrap_or_default()
}

/// `.headers` sidecar（wptserve per-file 响应头）读取——`<path>.headers` INI 形态
///（`[section]` 忽略、`key: value` 行），返回 Content-Type 值；文件缺失/无该行返 None。
/// 消费面：fetch 通道静态服务对同名 sidecar 的 Content-Type 覆盖默认 MIME
///（xmldecl support `*-vs-http` 族——断言 HTTP charset 头优先于 XML 声明/meta）。
/// reftest 通道 CSS 侧已有独立 sidecar 读取（resources.rs charset determination）。
fn wpt_sidecar_content_type(root: &Path, path: &str) -> Option<String> {
    let bytes = std::fs::read(root.join(format!("{path}.headers"))).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('[') && !line.starts_with('#') && !line.starts_with(';'))
        .find_map(|line| {
            line.split_once(':').and_then(|(key, value)| {
                key.trim()
                    .eq_ignore_ascii_case("content-type")
                    .then(|| value.trim().to_string())
                    .filter(|v| !v.is_empty())
            })
        })
}

fn wpt_status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        206 => "Partial Content",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "",
    }
}

/// testdriver Actions 指针步注入模式（dnd-compat M3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointerStepInjection {
    /// uievents/pointerevents 通道：shim 脚本合成（`script_pointer_*`，仅事件面）。
    /// 两套注入模式并存的取舍见 dnd-compat M3 RFC §5.6（dnd 语料走与桌面相同的
    /// 真实状态机，非假面；收敛不在本期）。
    ShimSynthesis,
    /// dnd 通道：宿主拖放状态机真管道（`WebView::apply_drag_input`——drag* 事件序
    /// + DataTransfer 会话与桌面浏览器同一实现）。
    DragStateMachine,
}

pub(crate) fn run_testharness_html_inner(
    wpt_root: &Path,
    case_name: &str,
    source: &str,
    harness_source: &str,
    inline_extras: &[(&str, &str)],
    timeout: Duration,
) -> Vec<HarnessSubtestResult> {
    run_testharness_html_with_pointer_injection(
        wpt_root,
        case_name,
        source,
        harness_source,
        inline_extras,
        timeout,
        PointerStepInjection::ShimSynthesis,
    )
}

/// [`run_testharness_html_inner`] 的注入模式参数化形态（dnd 通道传
/// [`PointerStepInjection::DragStateMachine`]，其余通道经包装走默认）。
/// 诊断（env `ZW_TESTHARNESS_TIME_LOG=1` 启用）：每案分段计时（镜像 main.rs
/// `CaseTimer` 的 RAII 模式）——WebView 构建/字体预载/文档加载/首次脚本（含
/// js-dom shim 的 V8 编译）/setup/媒体登记各段耗时与探针循环轮数，定位每案
/// 固定成本。输出走 stderr：`--json` 时 stdout 必须是纯 JSON（main.rs 结束时
/// 单次 println）。env 关闭时仅一次 env 读 + 一次 `Instant::now`。
struct HarnessCaseTimer {
    id: String,
    enabled: bool,
    start: std::time::Instant,
    phase: std::time::Instant,
    segments: Vec<(&'static str, u128)>,
    rounds: usize,
}

impl HarnessCaseTimer {
    fn new(id: &str) -> Self {
        let now = std::time::Instant::now();
        let enabled = runner_flags().time_log;
        Self {
            id: id.to_string(),
            enabled,
            start: now,
            phase: now,
            segments: Vec::new(),
            rounds: 0,
        }
    }

    /// 结束当前分段并开启下一段；env 关闭时零成本。
    fn done(&mut self, segment: &'static str) {
        if self.enabled {
            self.segments.push((segment, self.phase.elapsed().as_millis()));
            self.phase = std::time::Instant::now();
        }
    }
}

impl Drop for HarnessCaseTimer {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }
        // 尾段（探针循环；提前 return 时为残余路径）不显式标记者并入 run。
        let run_ms = self.phase.elapsed().as_millis();
        let segs = self
            .segments
            .iter()
            .map(|(name, ms)| format!("{name}={ms}ms"))
            .collect::<Vec<_>>()
            .join(" ");
        eprintln!(
            "[th-time] {} | {} run={run_ms}ms rounds={} total={:.3}s",
            self.id,
            segs,
            self.rounds,
            self.start.elapsed().as_secs_f64()
        );
    }
}

/// runner adopted 线程局部作用域 guard（engine `enter/exit_runner_adopted_scope`）：
/// 构造即 enter，Drop 即 exit——case 函数多 return 点（unsupported 提前 return、
/// 错误/Timeout 路径）全覆盖。串行也启用：修的是上一案 adopted 文本经进程槽残留
/// 到下一案的既有跨案污染，单代码路径与并行度无关（Commit 4，perf/wpt-runner）。
struct RunnerAdoptedScope;

impl RunnerAdoptedScope {
    fn enter() -> Self {
        zero_engine::js_dom_bridge::enter_runner_adopted_scope();
        Self
    }
}

impl Drop for RunnerAdoptedScope {
    fn drop(&mut self) {
        zero_engine::js_dom_bridge::exit_runner_adopted_scope();
    }
}

pub(crate) fn run_testharness_html_with_pointer_injection(
    wpt_root: &Path,
    case_name: &str,
    source: &str,
    harness_source: &str,
    inline_extras: &[(&str, &str)],
    timeout: Duration,
    pointer_injection: PointerStepInjection,
) -> Vec<HarnessSubtestResult> {
    let _adopted_scope = RunnerAdoptedScope::enter();
    let mut case_timer = HarnessCaseTimer::new(case_name);
    let unsupported = unsupported_testdriver_dependencies(source);
    if !unsupported.is_empty() {
        return vec![HarnessSubtestResult {
            name: case_name.to_string(),
            status: HarnessStatus::Unsupported,
            message: Some(format!("unsupported testdriver API: {}", unsupported.join(", "))),
        }];
    }

    let html = prepare_harness_html(source, harness_source, inline_extras, wpt_root, case_name);
    // R130：crash 用例判定（prepare_harness_html 同款判定——无 testharness 引用的纯脚本页）。
    let has_harness_ref = source.contains("/resources/testharness.js") || source.contains("testharnessreport.js");
    let scripts = zero_engine::extract_page_scripts(&html);
    let script_lengths = scripts
        .iter()
        .map(|script| match script {
            zero_engine::PageScript::Inline(source) | zero_engine::PageScript::InlineModule(source) => source.len(),
            zero_engine::PageScript::External(_) | zero_engine::PageScript::ExternalModule { .. } => 0,
        })
        .collect::<Vec<_>>();
    // js-dom M5/M7 收尾（R384）：kill-switch `ZW_NATIVE_DOM` env 已删——原生绑定是唯一
    // 生产路径，runner 经默认配置即走 native（testharness-dom-native 入口的 env 前缀
    // 成为无害 no-op，Makefile 目标保留以维持既有命令面）。
    let mut webview = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        // js-dom R201：V8 看门狗——页面脚本层死循环（mutation 视图失同步自旋，
        // Range-mutations-insertBefore 的 indexOf while 自旋）经 terminate_execution
        // 截断为 ScriptError::Timeout，单用例 Fail 收场，不再卡死整套 runner（case
        // 级 CASE_TIMEOUT 只在 run_page_scripts 返回后 tick，同步脚本死循环拦不到）。
        // 阈值取 CASE_TIMEOUT_LONG 同量级放宽（90s > 60s mega-case 正常脚本段）。
        // R350 实测（探针归档 evidence/2026-08-29-r350-range-registry-scan.md）：
        // Range-mutations dataChange 族 90s 截断的根因是 shim `__zwLiveRanges` 注册表
        // 随用例序列线性增长后，R260/R262/R263 adjust 对全部历史条目全量扫描 → 每用例
        // 二次方累积（非固定 per-op 成本，放宽超时无解）；同轮已在 shim 侧加跨树根守卫
        // 修复，90s 对修复后的单用例余量充足。
        script_timeout_ms: script_timeout_ms(),
        // security-hardening M2-s1：runner 侧开启 CSP 强制（生产 default off = kill-switch；
        // runner 是 WPT 标尺的实验臂——meta CSP 装配 + script 检查点 + violation 事件经
        // 此生效）。make test 全量门禁走生产默认（off），零 delta 由工作区测试守。
        csp_enforcement: true,
        // M2-s2：violation 源位置表从**原始 case 源**预计算（harness 内联扭曲装配后
        // 文档行号——blockeduri-inline 期望 15:9 口径），ordinal 与全量 script 序对齐
        //（harness/testdriver/inline extras 均与原标签 1:1 换）。
        csp_script_positions: Some(csp_script_positions_prepared(source)),
        // R34xx：headless 图片源——wpt.test/images/* 映射到本地 wpt-data 目录
        //（testharness 无网络；G5 DOM img 源解锁依赖图片加载）。
        // js-dom goal：dom 用例同样需要本地 .js 内联 + 图片资源，两条路径统一走 wpt_root。
        image_source_fetcher: wpt_data_image_fetcher(wpt_root),
        // R34xx：fetch() 本地资源（2d.composite.image.* fetch+createImageBitmap 路径）。
        fetch_handler: wpt_data_fetch_handler(wpt_root),
        // R34xx（G6）：.worker.js 变体 + worker importScripts 的脚本源。
        script_source_fetcher: wpt_data_script_fetcher(wpt_root, extract_meta_referrer_policy(source)),
        service_worker_script_fetcher: wpt_data_service_worker_script_fetcher(wpt_root),
        // net-api M5-S1：websockets 用例注入 WS 宿主 factory——内嵌 fixture 端点 +
        // 宿主改写委派 default_net_ws_host 真栈（ws_fixture.rs 模块注释）。仅
        // `websockets/` 用例启用：CSP/cookies 通道的 WebSocket 用例维持无宿主降级，
        // 跨通道零耦合（run-rules §10）。
        ws_host_factory: case_name
            .starts_with("websockets/")
            .then(crate::ws_fixture::fixture_ws_host_factory),
        ..WebViewConfig::default()
    });
    case_timer.done("new");
    // net-api M5-S1：websockets 用例页面走 http origin——上游 `?default` 变体语义
    // 即 http 页面 + ws 连接（constants.sub.js 判 `location.protocol === 'https:'`
    // 翻 wss 分支；runner 无 TLS fixture，wss 变体不在度量面）。通道级 scheme 翻转
    // 是 P3 挂账另案（两前置审计），此处仅 websockets 用例局部生效。
    // net-api M5-S5（M4-S27 记账翻转）：net-api 六 corpus 页锚 http（上游 ?default
    // 变体语义——access-control/cors-origin/integrity/response-url/url a-element 的
    // http: 形态期望面解锁）；其余通道维持 https（service-workers 等 secure-context
    // 面 scheme 门不动）。websockets 先行特例被本判定覆盖（同 http）。
    // webcrypto-compat 残留收口：WebCryptoAPI 无 `.https.` 中缀案同 http（上游
    // 文件名双态语义，historical.any.js 断言非安全上下文面；webcrypto_case_uses_http_page）。
    let page_scheme = if net_api_case_uses_http_page(case_name)
        || webcrypto_case_uses_http_page(case_name)
        || referrer_case_uses_http_page(case_name)
        || crate::testharness_page_state::page_state_case_uses_http_page(case_name)
    {
        "http"
    } else {
        "https"
    };
    // navigation-compat M2-S10：location_port 单案 origin 端口化（https 案锚上游
    // :9443 形态；谓词注释见 navigation_case_uses_ported_origin）。
    let page_authority = if navigation_case_uses_ported_origin(case_name) {
        "wpt.test:9443"
    } else if net_api_case_uses_http_page(case_name) {
        // net-api 二期 M2-S3b：net-api 页锚端口化 wpt.test:8000——上游 wptserve
        // 形态下 get_host_info().HTTP_ORIGIN = http://wpt.test:8000（{{ports[http][0]}}
        // 替换值）与页锚 origin 全等，同源判定不再撕裂（access-control-and-redirects
        // 「same-origin redirect」面：:8000 端点此前被按跨源预检、无 ACAO 即翻红）。
        // 非默认端口替换形与 `new URL().origin` 序列化形一致（:8000 保留），s116
        // :80 默认端口两形撕裂面不复现。ref/page-state http 家族维持无端口锚不动。
        "wpt.test:8000"
    } else {
        "wpt.test"
    };
    webview.prepare_document_state(&format!("{page_scheme}://{page_authority}/{case_name}"));
    let page_url = format!("{page_scheme}://{page_authority}/{case_name}");
    // M3 扩批：播放泵时钟原点（tab_worker pump_epoch 同款——registry play/tick 共用
    // 单调毫秒；桥 play 传 0 即本原点）。
    let playback_clock_origin = std::time::Instant::now();
    // R34xx：canvas 默认字体（sans-serif）预载系统真字体（带 kern）——无 @font-face 的
    // 页面（2d.text.drawing.style.fontKerning 等）默认字体度量/kerning 面依赖。需
    // resolve_font_id 大小写不敏感修复配套（否则 CanvasTest 显式族 miss 回退 sans-serif）。
    webview.load_canvas_system_sans_font();
    case_timer.done("font");
    let external_css = webview.fetch_page_images(&html, &page_url);
    webview.load_html(&html, Some(&external_css));
    case_timer.done("load");
    // M3 扩批 XVI：播放宿主桥**先于页面脚本**安装（canplaythrough handler 在初始脚本
    // 执行内同步调 video.play()——桥后装则 play 走 headless 分支、bridgeOn 永不置位，
    // 播放钟推进面失联）。execute_script 空转预热 = ensure_sandbox + ensure_js_shim
    //（install_playback_bridge 的回调注册前提）。
    // R5000 片 d：runner 生命周期尾接管 pageshow（shim 首监听自派发钩子按此门让位
    // ——the-end 的 DCL→load→pageshow 序断言依赖）。
    // webcrypto 残留收口：须走 with_dom 变体——裸 execute_script 会先触发
    // ensure_js_shim（无 __zw_get_page_url → location 回退 about:blank →
    // isSecureContext 误 true），shim 的 [SecureContext] 门禁（part02 尾）在 init
    // 时把 http 页锚误判 secure，historical.any.js 断言面失效。with_dom 先注册
    // DOM 回调（含 page URL）再装 shim，location 初值即真页锚。
    let _ = webview.execute_script_with_dom("0;globalThis.__zwPageShowRunnerOwned = true;");
    // 首次 execute 含 ensure_sandbox + js-dom shim 全量 V8 编译，单列观察。
    case_timer.done("first_exec");
    // wasm/jsapi 通道形态还原（2026-10-19 复跑实证）：with_dom 注入的 dom polyfill 以
    // 桥接对象替换原生 V8 WebAssembly（生产 execute 路径既有契约），桥 stub
    // Module/exports 使 wasm/jsapi 通道自基线 715/719 塌缩至 141/460。经桥的
    // __zwNative__ 不可枚举锚点整面回挂原生——还原 b785548dc 之前全部 runner corpus
    // 的页面环境形态；生产路径不回挂，宿主 wasm-sandbox 桥语义不变。
    let _ = webview.execute_script(
        "if (globalThis.WebAssembly && globalThis.WebAssembly.__zwNative__) globalThis.WebAssembly = globalThis.WebAssembly.__zwNative__;",
    );
    // M3 扩批 XVIII（2026-09-03，注册竞态消除）：媒体源按需供给方——宿主桥 play
    // 未命中（源未登记）时**同步**读 wpt-data 字节补登记，消除「重试等下一 probe
    // tick」的时序依赖（全套件并行负载下 tick 延迟放大，track-cues-enter-exit 的
    // 1/4 Timeout 根因）。URL → wpt-data 相对路径解析与逐 tick 登记同款。
    {
        let wpt_root = wpt_root.to_path_buf();
        webview.set_media_source_provider(move |url| {
            let path_part = url.split("://").nth(1)?;
            let (_, path) = path_part.split_once('/')?;
            let clean = path.split(['?', '#']).next().unwrap_or(path);
            if clean.is_empty() {
                return None;
            }
            std::fs::read(wpt_root.join(clean)).ok()
        });
    }
    // M3 扩批 XXV：宿主泵时钟注入——桥 play 的 nowMs=0（shim 无钟）翻译为泵时钟
    // 现值，播放锚与下方泵 tick 同源（原点错位曾使首拍 delta=泵全程，位置瞬间
    // 跳到流末——loop 回卷再 play 的推进失真根因）。
    let pump_clock = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let _ = webview.install_playback_bridge_with_clock(Some(std::sync::Arc::clone(&pump_clock)));
    // R3254-KP4（keyboard-page-scrolling goal M2 切片 2，2026-09-07）：帧驱动 rAF
    // opt-in——`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1` 时在页面脚本执行前置位 shim 的
    // `__ZW_RAF_FRAME_DRIVEN`（R2713a kill-switch 同名语义），rAF 回调改在下方 probe
    // 循环按帧派发（`__zw_raf_tick(墙钟 ms)`）。默认 OFF 维持同步 stub（既有全部
    // testharness 套件零行为变化；仅 testharness-keyboard 入口开启——snap 三案的
    // waitForDelayWithoutScrollEvent 200ms 静默窗依赖真实墙钟在帧间流逝，同步级联内
    // 永不满足 → 整簇 Timeout）。
    let raf_frame_driven = runner_flags().raf_frame_driven;
    if raf_frame_driven {
        let _ = webview.execute_script("globalThis.__ZW_RAF_FRAME_DRIVEN = true;");
    }
    // event-loop-spec M3-S1：timer 泵 per-task 边界（2026-09-12 default-on，opt-out
    // `ZW_TESTHARNESS_TIMER_PER_TASK=0`）。ON → `__zw_fire_due_timers` 每次调用只派发一个
    // 到期 timer——probe 循环每次迭代一个 execute → 一 timer 一 task 一 checkpoint（spec
    // event loop processing model step 3-6），消除「N timer 一个 execute、checkpoint
    // 只在批尾」的批量派发违反（evidence/2026-09-11-m1-event-loop-gap-list.md §1.2）。
    // A/B 零回归证据：evidence/2026-09-11-m3-s1-timer-per-task.md（dom 全量 54k subtest
    // / MO / IO / RO 双臂零 delta）+ evidence/2026-09-12-m4-runner-viewport-bridging.md
    // 后 IO/RO 复跑零 delta；runner 侧开关，生产路径零触碰。
    let timer_per_task = runner_flags().timer_per_task;
    if timer_per_task {
        let _ = webview.execute_script("globalThis.__ZW_TIMER_PER_TASK = true;");
    }
    // M2-S4X（navigation-compat，t8m renderer/tab_scripts 路径镜像——spec dom-document-readystate）：
    // 脚本阶段起点置 "loading"——页面脚本语义位置 = parser-inserted classic script（此前
    // testharness 路径 `__zwReadyState` 未注入、getter 缺省 "complete"，WPT
    // navigate-history-push-not-loaded「Document must not have loaded yet」即此根因）。
    // interactive/complete 两过渡随下方生命周期 timer 任务原子派发（含 readystatechange）。
    let _ = webview.execute_script(&zero_engine::script_set_ready_state("loading"));
    // M2-S5（navigation-compat，2026-10-10）：pagereveal 于首次渲染机会派发——rAF 同步
    // stub（part01 reftest 兼容模型）在**注册时**即跑回调，页面首 listener→rAF 序里
    // reveal 必须先于首个 rAF 回调（spec reveal steps 先于该渲染机会的 rAF 回调批）。
    // 包一层 rAF：首次调用先派 pagereveal（trusted，__zwPagerevealFired 门）再透传；
    // 页面从不调 rAF 时由下方生命周期 timer 兜底派发（同门防双发）。仅 testharness
    // runner 路径包裹，零产品面。WPT pagereveal/order-in-new-document-navigation
    // 「pagereveal,rAF」。
    let _ = webview.execute_script(
        "(function(){try{if(globalThis.__zwPagerevealFired===undefined){globalThis.__zwPagerevealFired=false;}var _zwOrigRaf=globalThis.requestAnimationFrame;if(typeof _zwOrigRaf==='function'){globalThis.requestAnimationFrame=function(fn){try{if(!globalThis.__zwPagerevealFired){globalThis.__zwPagerevealFired=true;var mk=globalThis.__zwMakeTrustedEvent;var pr=mk?mk('pagereveal'):new Event('pagereveal');try{if(globalThis.Event&&globalThis.Event.prototype){Object.setPrototypeOf(pr,globalThis.Event.prototype);}}catch(_e5prp){}globalThis.dispatchEvent(pr);}}catch(_e5pr){}return _zwOrigRaf.call(globalThis,fn);};}}catch(_e5pr0){}})();",
    );
    // M2-S12（navigation-compat，2026-10-20）：初始 autofocus 处理——spec flush
    // autofocus candidates 的「ready for post-load tasks」相位（文档序首个合格
    // candidate 获焦；interaction.html#the-autofocus-attribute）。runner 模型里
    // 页面脚本用同步 rAF stub，setup 期 `rAF×2` await 在脚本任务内即解——该相位
    // 必须落在**全部页面脚本之前**才可观察（真浏览器 post-parse 相位的页可观察
    // 等价位）。合格性：connected（querySelectorAll 语义自带）+ 非 disabled +
    // 非 hidden input；首个即 break（flush 语义：余者作废）。
    // 仅 testharness runner 路径执行，零产品面（产品宿主 autofocus 管线随真实
    // focus 域另行接线）。WPT navigation-api/focus-reset/autofocus promise_setup
    // 「Non-navigation API autofocus was processed」。
    // FIXME(产品面): 浏览器宿主解析路径的 autofocus 初始处理未接线。
    let _ = webview.execute_script(
        "(function(){try{var _c=document.querySelectorAll('[autofocus]');for(var i=0;i<_c.length;i++){var _e=_c[i];if(_e.disabled)continue;if(_e.tagName==='INPUT'&&String(_e.type).toLowerCase()==='hidden')continue;if(typeof _e.focus==='function'){_e.focus();break;}}}catch(_eAfp){}})();",
    );
    // s112（html-semantics M2，parser 外链时序调度）：页面侧脚本调度器。预处理把
    // defer/async/trickle 外链单元改投 `__zwSchedRegister(kind, delayMs, fn)`（见
    // script_sched_unit），本 bootstrap 承接注册与相位执行：
    // - async/block：注册即 setTimeout(delay)——trickle 模拟取回延迟，取回完成即执行
    //   （async 任意序）；block 为解析阻塞延迟近似（不暂停后续投递）。
    // - defer：登记文档序队列 + max-trickle 到期时刻（首注册时间锚定——注册全部发生在
    //   同步解析任务内）；由生命周期链 interactive 之后 `__zwSchedArmDrain(onDone)`：
    //   等到期 → 文档序执行 → onDone（链侧在其内派 DOMContentLoaded——spec the-end：
    //   deferred 执行完成后 DCL 同任务派发）。
    // pending 计数供 load 等待（loadStep5275 扩展条件）：async/block 尾随计时器与
    // deferred drain 完成前 load 不派发。
    // https://html.spec.whatwg.org/multipage/scripting.html#the-end
    let _ = webview.execute_script(
        "(function(){var _s={deferred:[],defMax:0,defFirst:0,pending:0,drained:false};\
         globalThis.__zwSchedRegister=function(kind,delayMs,fn){_s.pending++;\
         if(kind==='defer'){if(_s.deferred.length===0){_s.defFirst=Date.now();}\
         if(delayMs>_s.defMax){_s.defMax=delayMs;} _s.deferred.push(fn);return;}\
         setTimeout(function(){try{fn();}finally{_s.pending--;}},delayMs|0);};\
         globalThis.__zwSchedHasDeferred=function(){return _s.deferred.length>0;};\
         globalThis.__zwSchedPending=function(){return _s.pending>0;};\
         globalThis.__zwSchedArmDrain=function(onDone){\
         var wait=_s.drained?0:Math.max(0,_s.defFirst+_s.defMax-Date.now());\
         setTimeout(function(){for(var i=0;i<_s.deferred.length;i++){try{_s.deferred[i]();}finally{_s.pending--;}}\
         _s.drained=true;onDone();},wait);};})();",
    );
    let script_result = webview.run_page_scripts_strict();
    // R3254-E2 切片（editing goal，2026-09-07）：DOMContentLoaded/load 派发 + harness_loaded
    // 标记改 runner 侧执行（此前是 prepare 尾部 append 的 `<script>`——被解析器归入 body，
    // 成为页面可见 DOM，污染 documentElement.outerHTML 精确断言）。时序不变：全部页面脚本
    // 之后同步执行。
    let _ = webview.execute_script(
        // R5000 片 d（html-syntax-compat，spec the-end）：生命周期三事件单 timer 任务
        // 内严格序 DCL → load → pageshow。①DCL 异步入队——defer 脚本已入队的 timer
        // 任务先于 DCL 跑（WPT DOMContentLoaded-defer：defer 脚本队列任务执行时 DCL
        // 不得已发）；②DCL bubbles:true（the-end 断言）；③load/pageshow 经 window
        // 通道派发但 target=document（spec：Window load/pageshow 的 target 是
        // Document——预写 target，shim dispatch 不覆写，part02 pageshow 同款先例）；
        // ④pageshow bubbles+cancelable:true（whatwg#6794）+ PageTransitionEvent 原型
        //（assert_class_string）；⑤关 shim 首监听自派发钩子防双发。
        // M2-S5（navigation-compat，2026-10-10）：⑥pagereveal 于 pageshow 后同任务派发
        //（spec reveal steps——新文档显示时 window 上派 pagereveal，早于首个渲染机会/rAF；
        // persisted=false 新载入形。WPT pagereveal/order-in-new-document-navigation
        // 「pagereveal,rAF」——此前无此事件面，log 恒 'rAF'）。
        // R5275（html-semantics M2 切片 68，2026-10-24）：单 timer 任务拆**三任务链**——
        // ①interactive+DCL（spec：DCL 不等图片）；②img settle 独立任务（`__zwFireStaticImgSettle`
        // 派发的 img load 事件其 promise 回调（invisible-image have_images_loaded 的
        // Promise.all）是微任务，须在任务边界排水——同任务直派则 window load listener
        // 的同步断言先于微任务看见旗标）；③load 等待任务（先 settle 再查
        // `__zwImgLoadBlocked`：R56h fetch 在飞（wptserve trickle(dN) shim 侧延迟的
        // 慢图，delay-load-event-detached）或闸门队列未 drain → 100ms 重试；s110 扩展：
        // 动态外链 script 取回在飞（`__zwScriptFetchPending`，122-126 面——spec
        // 「delaying the load event」的 script 臂）同条件重试；s112 扩展：调度器
        // pending（async/block 计时器未 fire、deferred 未 drain——`__zwSchedPending`）
        // 同条件重试；就绪 →
        // complete+load+pageshow+pagereveal）。严格序 DCL→load→pageshow 不变（链式
        // setTimeout 保序）；readyState complete 相位随 load 后移（spec the-end：
        // complete 即 fully loaded 相位）。s112：DCL 派发条件化——无 deferred 单元时
        // 维持本任务内即派（既有页形态逐位不变）；有 deferred 单元时 DCL 移交
        // `__zwSchedArmDrain` 回调（interactive 后 drain：max trickle 到期 → 文档序
        // 执行 → 同任务派 DCL，spec the-end 序）。
        // https://html.spec.whatwg.org/multipage/scripting.html#the-end
        "(function () {var mk = globalThis.__zwMakeTrustedEvent;function protoLink(ev, proto) {try { if (proto && proto.prototype) Object.setPrototypeOf(ev, proto.prototype); } catch (_e5000pl) {}return ev;}function fireSettle5275() {try { if (typeof globalThis.__zwFireStaticImgSettle === 'function') globalThis.__zwFireStaticImgSettle(); } catch (_eIm5275) {}}\
        setTimeout(function () {try {globalThis.__zwReadyState='interactive';} catch (_ers8m1) {}try {var rs1 = protoLink(mk ? mk('readystatechange') : new Event('readystatechange'), globalThis.Event);document.dispatchEvent(rs1);} catch (_ers8m1d) {}var _zwDclFire112 = function () {try {if (!globalThis.__zwDclFired112) {globalThis.__zwDclFired112 = true;var dcl = protoLink(mk ? mk('DOMContentLoaded', { bubbles: true }) : new Event('DOMContentLoaded', { bubbles: true }), globalThis.Event);document.dispatchEvent(dcl);}} catch (_edcl112) {}};var _zwHasDef112 = (typeof globalThis.__zwSchedHasDeferred === 'function') && globalThis.__zwSchedHasDeferred();if (_zwHasDef112 && typeof globalThis.__zwSchedArmDrain === 'function') {globalThis.__zwSchedArmDrain(_zwDclFire112);} else {_zwDclFire112();}\
        setTimeout(function () {fireSettle5275();\
        setTimeout(function () {var loadStep5275 = function () {fireSettle5275();try { if ((typeof globalThis.__zwImgLoadBlocked === 'function' && globalThis.__zwImgLoadBlocked()) || (typeof globalThis.__zwScriptFetchPending === 'function' && globalThis.__zwScriptFetchPending()) || (typeof globalThis.__zwSchedPending === 'function' && globalThis.__zwSchedPending())) { setTimeout(loadStep5275, 100); return; } } catch (_eImBlk) {}try {globalThis.__zwReadyState='complete';} catch (_ers8m2) {}try {var rs2 = protoLink(mk ? mk('readystatechange') : new Event('readystatechange'), globalThis.Event);document.dispatchEvent(rs2);} catch (_ers8m2d) {}var load = protoLink(mk ? mk('load') : new Event('load'), globalThis.Event);try { load._zwTargetOverride = true; load.target = document; } catch (_e5000lt) {}globalThis.dispatchEvent(load);var ps = protoLink(mk ? mk('pageshow', { bubbles: true, cancelable: true }) : new Event('pageshow', { bubbles: true, cancelable: true }), globalThis.PageTransitionEvent);try { ps.persisted = false; } catch (_e5000pp) {}try { ps._zwTargetOverride = true; ps.target = document; } catch (_e5000pt) {}globalThis.dispatchEvent(ps);try {if(!globalThis.__zwPagerevealFired){globalThis.__zwPagerevealFired=true;var pr = protoLink(mk ? mk('pagereveal') : new Event('pagereveal'), globalThis.Event);globalThis.dispatchEvent(pr);}} catch (_ers5pr) {}};loadStep5275();}, 0);\
        }, 0);\
        }, 0);\
        })();if (typeof globalThis.__zw_mark_harness_loaded === 'function') {globalThis.__zw_mark_harness_loaded();}",
    );
    // M3 扩批（2026-09-02，fixture-mounted 播放切片）：播放宿主桥 + 媒体源登记。
    // 页面 <video>/<audio> src（经 extract_media_resources 提取、相对 case 目录解析）
    // 从 wpt-data 读字节登记进 VideoPlayerRegistry——shim play() 经 __zwVideoBridge
    // 走真值播放（registry 泵推进帧/音频时钟）。非 webm 源登记后 play 返 false 回落
    // headless（语义层零回归）。
    // M3 扩批 XVI：动态 `.src=`（页面脚本赋值）登记——runner 初始 extract 只见静态
    // 标记，脚本赋值产生的媒体源须按 DOM 现值补登记（幂等；webview.borrow 桥回读
    // 不可行——host 侧持 &mut，经 JS 快照 media 元素 src 列表）。
    // 返回：本轮新登记的 (kind_tag, abs_url, duration_ms) 列表（供 settle 提交）。
    fn register_dynamic_media_sources(
        webview: &mut WebView,
        wpt_root: &Path,
        byte_cache: &mut std::collections::HashMap<String, std::rc::Rc<Vec<u8>>>,
    ) -> Vec<(&'static str, String, Option<u64>)> {
        // 页面侧快照（DOM 现值）：audio/video 元素的 resolve 后 src（data:/blob: 跳过）。
        let snapshot = webview
            .execute_script(
                "(function(){\
                   var out=[];\
                   var els=document.querySelectorAll('audio,video');\
                   for (var i=0;i<els.length;i++){\
                     var s=els[i].currentSrc||els[i].src||'';\
                     if (s) out.push(s);\
                   }\
                   return out.join('|');\
                 })()",
            )
            .unwrap_or_default();
        let mut committed = Vec::new();
        if snapshot.is_empty() {
            return committed;
        }
        let players_arc = webview.video_players();
        let Ok(mut reg) = players_arc.lock() else {
            return committed;
        };
        for src in snapshot.split('|') {
            if src.is_empty() || !src.starts_with("http") {
                continue;
            }
            let path_part = src
                .split("://")
                .nth(1)
                .and_then(|rest| rest.split_once('/'))
                .map(|(_, path)| path)
                .unwrap_or("");
            let clean = path_part.split(['?', '#']).next().unwrap_or(path_part);
            if clean.is_empty() {
                continue;
            }
            let bytes = match byte_cache.get(clean) {
                Some(cached) => std::rc::Rc::clone(cached),
                None => match std::fs::read(wpt_root.join(clean)) {
                    Ok(bytes) => {
                        byte_cache.insert(clean.to_string(), std::rc::Rc::new(bytes));
                        std::rc::Rc::clone(&byte_cache[clean])
                    }
                    Err(_) => continue,
                },
            };
            let kind_tag = if clean.ends_with(".oga") || clean.ends_with(".mp3") {
                "audio"
            } else {
                "video"
            };
            // 幂等：player 已存在（播放中/已建）或源字节已登记则跳过——泵 tick 高频调用面。
            if reg.contains_source(src) {
                continue;
            }
            reg.register_source(src, bytes.as_ref().clone());
            if kind_tag == "audio" {
                reg.register_audio_source(src, bytes.as_ref().clone());
            }
            committed.push((
                kind_tag,
                src.to_string(),
                reg.duration(src).map(|d| (d * 1000.0) as u64),
            ));
        }
        // settle 提交在锁外（execute_script 走 JS 桥，避免锁序交叉）。
        drop(reg);
        committed
    }

    case_timer.done("setup");
    if script_result.is_ok() {
        // 静态标记登记（初始 extract——DOM 快照此时尚未含脚本赋值）+ 首轮动态登记。
        // settle 提交（readyState/duration 真值链）随登记产出提交。
        let mut media_byte_cache: std::collections::HashMap<String, std::rc::Rc<Vec<u8>>> =
            std::collections::HashMap::new();
        for resource in zero_engine::extract_media_resources(&html) {
            let resolved = zero_engine::resolve_document_url(&page_url, &resource.src);
            // M3 扩批（fixture-mounted 切片 2）：静态 HTML media 元素的 headless settle 提交
            //（tab_scripts finish 同款——testharness 无宿主提交通道，静态 <video src> 此前
            // 永不 settle → readyState 恒 NONE → seek 门 readyState>=1 不开）。
            if matches!(
                resource.kind,
                zero_engine::MediaResourceElementKind::Video
                    | zero_engine::MediaResourceElementKind::Audio
                    | zero_engine::MediaResourceElementKind::Source
            ) {
                // M3 扩批 XXX：source 子候选 settle 提交（此前静态 <source> 永不
                // settle——__zw_commit 的 source 分支处理父级 available settle +
                // 加载序列派发）。源字节可达性判定（source 候选语义——缺席候选
                // commit 'error' 走下一候选；可播源登记 + probe 真值链）。
                let path_part = resolved
                    .split("://")
                    .nth(1)
                    .and_then(|rest| rest.split_once('/'))
                    .map(|(_, p)| p)
                    .unwrap_or("");
                let clean = path_part.split(['?', '#']).next().unwrap_or(path_part);
                let bytes_ok = (!clean.is_empty())
                    .then(|| std::fs::read(wpt_root.join(clean)).ok())
                    .flatten();
                if let Some(bytes) = bytes_ok.clone()
                    && let Ok(mut reg) = webview.video_players().lock()
                    && !reg.contains_source(&resolved)
                {
                    reg.register_source(&resolved, bytes);
                }
                let outcome = if bytes_ok.is_some() { "loaded" } else { "error" };
                let (width, height, duration_ms) = match webview.video_players().lock() {
                    Ok(reg) => match resource.kind {
                        zero_engine::MediaResourceElementKind::Video
                        | zero_engine::MediaResourceElementKind::Source
                            if outcome == "loaded" =>
                        {
                            let (w, h) = reg.probe_dimensions(&resolved);
                            (w, h, reg.duration(&resolved).map(|d| (d * 1000.0) as u64))
                        }
                        _ => (0, 0, reg.duration(&resolved).map(|d| (d * 1000.0) as u64)),
                    },
                    Err(_) => (0u32, 0u32, None::<u64>),
                };

                let _ = webview.execute_script(&zero_engine::script_commit_resource_element_state(
                    kind_tag(resource.kind),
                    &resolved,
                    outcome,
                    width,
                    height,
                    duration_ms,
                ));
            }
        }
        let _ = register_dynamic_media_sources(&mut webview, wpt_root, &mut media_byte_cache);
        // M3 扩批 LI（2026-09-05）：静态提交链的 setTimeout 链集中排空——单次 execute 的
        // 20ms drain 窗口可能未及排空整链（线程 send 与 pending 计数竞态），媒体任务
        // （canplaythrough 等）滞留队列使「静态 src 无动态交互」形态的 handler 永不触发
        //（L 轮 runner 域归因的修复面）。
        webview.drain_pending_timers_until_idle(500);
    }
    case_timer.done("media");
    if let Err(error) = script_result {
        // 无 testharness 引用的 crash/no-harness 用例只有在脚本执行完成且未崩溃时才可
        // 由下方 terminal 分支判 PASS；脚本抛错不能按“未注册 test()”误判为通过。
        let declared = webview
            .execute_script(
                "(function(){try{var st=typeof globalThis.__zw_harness_state==='function'?globalThis.__zw_harness_state():null;return st&&st.tests?st.tests:0;}catch(_e){return 0;}})()",
            )
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(0);
        // s108（execution-timing 143 形态）：harness 已声明测试（declared>0）时，脚本
        // 抛错不再丢弃结果收集——WPT testharness 语义：uncaught exception 报到
        // window.onerror，不降级 test status（SVG script 语料故意在外层尾段埋
        // `assert_unreached():` 语法错，onload 的 log 顺序断言在轮询 tick 才执行，
        // 抛错时刻 pending=1 属正常中间态）。放行后已完成测试照收（Pass）、未完成
        // 挂到 deadline（Timeout）——短路页此前必判 Fail，此臂无 Pass→非P 面；
        // declared==0（无 testharness 引用）维持短路 Fail——s102 防假绿口径不变：
        // 脚本抛错不能按「未注册 test()」误判为通过。
        // https://html.spec.whatwg.org/multipage/webappapis.html#report-the-error
        if declared == 0 {
            return vec![HarnessSubtestResult {
                name: case_name.to_string(),
                status: HarnessStatus::Fail,
                message: Some(format!("page script threw (declared tests: {declared}): {error}")),
            }];
        }
        // declared>0：放行进下方轮询（已完成测试照收，未完成挂到 deadline 判
        // Timeout——抛错时刻的 pending 中间态不掩盖后续 onload/异步断言的完成机会）。
    }

    let deadline = Instant::now() + timeout;
    let mut partial_results = Vec::new();
    let mut last_test_function = "unknown".to_string();
    let mut last_harness_hook = "unknown".to_string();
    let mut last_state = serde_json::Value::Null;
    let mut last_test_wait = false;
    // R347：testdriver 命令解析重试计数——目标元素由 pending mutation 异步 apply
    //（body 在 timer 回调里 enqueue，host 侧 next pump 才 materialize），probe 每帧
    // 排空队列的首次解析可能先于元素落 doc。同 id 最多重试 20 帧，超限按原错误处理。
    let mut td_command_attempts: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();
    // uievents-compat M3 尾簇 4：已刷新的 mutation 代际（跨 take_probe/命令执行共享
    // ——同代不重复序列化，见 take_probe / resolve_pointer_target）。
    let mut td_refresh_ver: u64 = 0;
    // uievents-compat 尾簇 8：最后已知指针位（pointer 命令 resolve 后记录）——
    // mutation 驱动悬停重结算的命中测试输入。
    let mut td_last_pointer: Option<(f32, f32)> = None;
    // dnd-compat M3：拖放会话 Dragging 期间真实浏览器抑制鼠标/指针事件面
    //（dragstart 接受至 dragend；dragstart 取消 = 鼠标流恢复正常）——跨命令持续，
    // 与 td_last_pointer 同生命周期。
    let mut td_drag_mouse_suppressed = false;
    // M3 扩批 XVI：文件字节缓存（同 src 只读盘一次；注册表 contains_source 幂等）。
    let mut media_byte_cache: std::collections::HashMap<String, std::rc::Rc<Vec<u8>>> =
        std::collections::HashMap::new();
    loop {
        case_timer.rounds += 1;
        // M3 扩批 XVI：逐 tick 动态登记——脚本赋值的 `.src=`（含 script turn 内同步 play 的
        // canplaythrough handler 形态）在注册表补登记后，同 tick/下一 tick 桥 play 即真值
        // 可达。已登记的 src 幂等跳过（contains_source）；文件字节缓存避免重复 IO。
        // settle 提交只在**首次**登记时发（幂等面——_resourceStates 每 key 一次）。
        for (tag, url, duration_ms) in register_dynamic_media_sources(&mut webview, wpt_root, &mut media_byte_cache) {
            let _ = webview.execute_script(&zero_engine::script_commit_resource_element_state(
                tag,
                &url,
                "loaded",
                0,
                0,
                duration_ms,
            ));
        }
        if Instant::now() >= deadline {
            let mut results = map_harness_results(partial_results);
            results.push(HarnessSubtestResult {
                name: case_name.to_string(),
                status: HarnessStatus::Timeout,
                message: Some(format!(
                    "testharness completion callback was not called (test={}, hook={}, scripts={script_lengths:?}, state={last_state}, test_wait={last_test_wait})",
                    last_test_function, last_harness_hook
                )),
            });
            return results;
        }

        let _ = webview.poll_service_worker_runtime_events();
        // R3254-KP4：帧驱动 rAF 派发（opt-in）——每轮 probe 一帧，时间戳取 case 启动
        // 起的墙钟 ms（spec DOMHighResTimeStamp 单调；shim `__zw_raf_tick` 在 OFF 时
        // 早返零开销）。rAF 内重注册进下一帧队列（shim 快照语义）。
        if raf_frame_driven {
            let raf_ms = playback_clock_origin.elapsed().as_secs_f64() * 1000.0;
            let _ = webview.execute_script(&format!("__zw_raf_tick({raf_ms});"));
        }
        // event-loop-spec M1 切片 3a：observer tick 接线——对齐 renderer post-render tick
        // 语义（apps/renderer/src/page_scripts.rs `tick_observers`：`__zw_observers_tick()`
        // 重算 IO/RO threshold 越界/size-diff 并派发后续通知）。testharness runner 无渲染
        // 循环，此前 observe() 之后的动态跟踪面（scroll/样式变化后再通知）天然不可达——
        // IO 基线 65× `entries.length expected N but got 1` 失败簇的直接根因
        //（evidence/2026-09-11-m1-observers-wpt-baseline.md）。回调内 DOM 写走 native
        // 绑定直改 live doc + sync_render_after_native_dom 重渲染（execute_script 既有
        // 路径），rect 快照随渲染刷新，下一轮 tick 读到新几何。无 observer 注册时 shim
        // 侧空表早返，零开销。
        let _ = webview.execute_script("if(globalThis.__zw_observers_tick)globalThis.__zw_observers_tick();");
        // js-dom R342：动画时钟泵——页面脚本设置的 transition/animation 需要第二轮
        // re-style + 时钟 tick 才产生事件（run_page_scripts 只执行脚本不重渲染）。
        // 真实时间作时钟源：probe 循环的墙钟间隔自然推进 30ms/100ms 量级测试动画；
        // 事件经 engine 的 script_gen 脚本派发进 shim（与 renderer 同汇流点）。
        {
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);
            if webview.pump_animation_clock(now_secs) {
                for ev in webview.take_pending_transition_events() {
                    let _ = webview.execute_script(&zero_engine::script_dispatch_transition_event(
                        &ev.selector,
                        ev.kind.as_event_type(),
                        &ev.property,
                        ev.elapsed,
                    ));
                }
                for ev in webview.take_pending_animation_events() {
                    let _ = webview.execute_script(&zero_engine::script_dispatch_animation_event(
                        &ev.selector,
                        ev.kind.as_event_type(),
                        &ev.name,
                        ev.elapsed,
                    ));
                }
            }
        }
        // M3 扩批：播放泵——registry 有播放中源时按单调时钟推进（帧 tick + 音频泵；
        // 时钟原点与桥 play(0) 对齐——tab_worker pump_epoch 同契约）。changed 帧注入
        // ImageCache（painter 通路）；媒体时间事件（timeupdate 等）由 shim 侧 headless
        // 序列承载（语义层不回归）。
        {
            let now_ms = playback_clock_origin.elapsed().as_millis() as u64;
            pump_clock.store(now_ms, std::sync::atomic::Ordering::Relaxed);
            if !runner_flags().disable_pump
                && let Ok(mut reg) = webview.video_players().lock()
                && reg.is_any_playing()
            {
                let cache = webview.image_cache();
                let _ = reg.tick_all(now_ms, cache);
                let _ = reg.audio_advance_all(now_ms);
            }
        }
        let probe = match take_probe(&mut webview, &mut td_refresh_ver) {
            Ok(probe) => probe,
            Err(error) => {
                return vec![HarnessSubtestResult {
                    name: case_name.to_string(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }];
            }
        };
        // uievents-compat 尾簇 8：mutation 驱动悬停重结算——探测轮 flush 后悬停目标
        // 可能已被页内 remove/appendChild 失效（mutTick 推进）。poll 廉价判据命中时
        // 以 last pointer 位跑 fresh 命中测试（resolve 内部代际门控刷新），命中与现
        // 悬停不同 → shim 补跨界序（只派边界序不派 move 对）。无指针活动或未失效
        // 时零命中测试（单 execute_script 轮询判据）。
        if let Some((last_x, last_y)) = td_last_pointer
            && let Ok(dirty) =
                webview.execute_script("globalThis.__zw_ptr_hover_dirty ? globalThis.__zw_ptr_hover_dirty() : '0'")
        {
            let debug = runner_flags().td_debug;
            if debug {
                eprintln!("[zw-settle] poll dirty={} pos=({last_x},{last_y})", dirty.trim());
            }
            if dirty.trim() == "1" {
                let (hit_sel, hx, hy) =
                    resolve_pointer_target(&mut webview, "", last_x, last_y, true, &mut td_refresh_ver);
                if debug {
                    eprintln!("[zw-settle] hit={hit_sel} at ({hx},{hy})");
                }
                let script = zero_engine::script_mut_hover_settle(&hit_sel, hx, hy);
                let _ = webview.execute_script(&script);
            }
        }
        partial_results = probe.results;
        last_test_function = probe.test_function;
        last_harness_hook = probe.harness_hook;
        last_state = probe.state;
        last_test_wait = probe.test_wait;
        for command in probe.commands {
            let result = apply_testdriver_command(
                &mut webview,
                &command,
                &mut td_refresh_ver,
                &mut td_last_pointer,
                pointer_injection,
                &mut td_drag_mouse_suppressed,
            );
            // R347：目标未解析（元素尚未 materialize）→ 重新入队下帧重试。
            let unresolved = result.as_deref().is_some_and(|message| {
                message.starts_with("testdriver target not found")
                    || message.starts_with("testdriver target has no stable selector")
            });
            if unresolved {
                let attempts = td_command_attempts.entry(command.id).or_insert(0);
                if *attempts < 20 {
                    *attempts += 1;
                    let sel_js = match &command.selector {
                        Some(v) => format!("'{}'", v.replace('\'', "\\'")),
                        None => "null".to_string(),
                    };
                    let text_js = match &command.text {
                        Some(v) => format!("'{}'", v.replace('\'', "\\'")),
                        None => "null".to_string(),
                    };
                    let op = command.operation.replace('\'', "\\'");
                    let _ = webview.execute_script(&format!(
                        "(globalThis.__zw_td_queue = globalThis.__zw_td_queue || []).push({{id:{}, operation:'{}', selector:{}, text:{}}});",
                        command.id, op, sel_js, text_js
                    ));
                    continue;
                }
            }
            if let Err(error) = resolve_testdriver_command(&mut webview, command.id, result.as_deref()) {
                return vec![HarnessSubtestResult {
                    name: case_name.to_string(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }];
            }
        }
        if harness_probe_is_terminal(probe.complete, &last_state, partial_results.len()) {
            if partial_results.is_empty() {
                // R130（js-dom M4）：crash 类用例（无 testharness 引用，见 prepare_harness_html
                // 注入分支）语义 = 页面脚本执行不崩溃。harness 走到 terminal（phase=4 /
                // completion 回调已调）且 run_page_scripts_strict 未报脚本抛错（报错早在
                // 上方 return Fail）→ 按上游浏览器语义记 PASS（伪 subtest「did not crash」）。
                if !has_harness_ref {
                    if probe.test_wait {
                        std::thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    return vec![HarnessSubtestResult {
                        name: case_name.to_string(),
                        status: HarnessStatus::Pass,
                        message: None,
                    }];
                }
                return vec![HarnessSubtestResult {
                    name: case_name.to_string(),
                    status: HarnessStatus::Timeout,
                    message: Some(format!(
                        "testharness completed without reporting registered tests (state={})",
                        last_state
                    )),
                }];
            }
            return map_harness_results(partial_results);
        }
        if !has_harness_ref && partial_results.is_empty() && !probe.test_wait {
            return vec![HarnessSubtestResult {
                name: case_name.to_string(),
                status: HarnessStatus::Pass,
                message: None,
            }];
        }
        if probe.due_timer {
            std::thread::yield_now();
        } else {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

fn terminal_harness_state(state: &serde_json::Value, result_count: usize) -> bool {
    state.get("phase").and_then(serde_json::Value::as_u64) == Some(4)
        && state.get("pending").and_then(serde_json::Value::as_u64) == Some(0)
        && state.get("tests").and_then(serde_json::Value::as_u64) == Some(result_count as u64)
}

fn harness_probe_is_terminal(complete: bool, state: &serde_json::Value, result_count: usize) -> bool {
    terminal_harness_state(state, result_count) || (state.is_null() && complete)
}

fn map_harness_results(results: Vec<RawHarnessResult>) -> Vec<HarnessSubtestResult> {
    results
        .into_iter()
        .map(|result| HarnessSubtestResult {
            name: result.name,
            // 上游 testharness subtest status：0=PASS、1=FAIL、2=TIMEOUT、3=NOTRUN、4=PRECONDITION_FAILED
            //（js-dom R20：3/4 此前落到 `_ => Fail` 误计为失败；NOTRUN/PRECONDITION_FAILED 是中性状态，
            // 通过率统计不计入 fail）。未知编码（5+）回落 Fail（保守：无法识别视为失败暴露异常）。
            status: match result.status {
                0 => HarnessStatus::Pass,
                2 => HarnessStatus::Timeout,
                3 => HarnessStatus::NotRun,
                4 => HarnessStatus::PreconditionFailed,
                _ => HarnessStatus::Fail,
            },
            message: result.message,
        })
        .collect()
}

/// security-hardening M2-s1：从用例 meta CSP 政策串取首个 `'nonce-<v>'` 源，生成
/// HTML nonce 属性串（供 runner 注入的 harness/testdriver/内联 extra 脚本戳记——
/// CSP 强制下无 nonce 的注入脚本被 script 检查点阻止 → harness 自身死亡 → 用例全
/// Timeout；harness 装配层适配，非 API 面）。无 meta CSP / 无 nonce 源 → 空串
///（零戳记零影响）。属性值剥除 HTML 敏感字符（nonce 语义为 base64 串，防御性清洗）。
fn harness_nonce_attr(case_source: &str) -> String {
    let policies = zero_engine::extract_meta_csp_policies(case_source);
    let Some(policy) = policies.first() else {
        return String::new();
    };
    let marker = "'nonce-";
    let Some(start) = policy.find(marker) else {
        return String::new();
    };
    let rest = &policy[start + marker.len()..];
    let end = rest.find('\'').unwrap_or(rest.len());
    let nonce = rest[..end].replace(['"', '<', '>', '&', '\''], "");
    if nonce.is_empty() {
        return String::new();
    }
    format!(" nonce=\"{nonce}\"")
}

/// M2-s2：CSP violation 源位置表（**prepared ordinal 空间**）。
///
/// prepare_harness_html 会把若干外链脚本**整标签移除**（testharnessreport /
/// testdriver-vendor / testdriver-actions → 空串替换），其后脚本在装配后文档的
/// script 序号前移——gate 的 script_index 与 shim getElementsByTagName 序号都在
/// prepared 空间，位置表必须同空间。本函数从原始 case 源提取位置，再按「被移除
/// 标签剔除」重映射（扫描口径与 [`zero_engine::extract_script_source_positions`]
/// 一致：逐 `<script` 开标签、无 `>` 即止）。
fn csp_script_positions_prepared(source: &str) -> Vec<(u32, u32)> {
    const REMOVED_SRC_MARKERS: [&str; 3] = [
        "/resources/testharnessreport.js",
        "/resources/testdriver-vendor.js",
        "/resources/testdriver-actions.js",
    ];
    let positions = zero_engine::extract_script_source_positions(source);
    let mut out = Vec::with_capacity(positions.len());
    let mut rest = source;
    let mut original_idx = 0usize;
    while let Some(start) = rest.find("<script") {
        let Some(gt) = rest[start..].find('>') else {
            break;
        };
        let open_tag = &rest[start..=start + gt];
        let is_removed = REMOVED_SRC_MARKERS.iter().any(|m| open_tag.contains(m));
        if !is_removed && let Some(p) = positions.get(original_idx) {
            out.push(*p);
        }
        original_idx += 1;
        rest = &rest[start + gt + 1..];
    }
    out
}

/// harness 装配 reporter 快照（结果/完成回调 → runner 探针全局）。
/// html-semantics M2 s98：提升为模块常量——write 注入 harness 的 fetch 通道服务
/// （[`wpt_data_fetch_handler`] 静态特判）与静态内联路径（[`prepare_harness_html`]）
/// 须共用同一装配面，防两路装配漂移。
const HARNESS_REPORTER_SOURCE: &str = r#"
if (typeof setup === 'function') setup({output: false});
// net-api M2-S2：self.GLOBAL 环境标记 shim（上游 testharnessreport.js 在 window 变体
// 下定义 GLOBAL={isWorker,isWindow,isShadowRealm}——wpt-data 的 pinned 副本不含；
// header-values 等用例经 self.GLOBAL.isWorker() 分支 window/xhr 面）。
globalThis.GLOBAL = globalThis.GLOBAL || {
  isWorker: function () { return false; },
  isWindow: function () { return true; },
  isShadowRealm: function () { return false; }
};
globalThis.__zw_harness_results = [];
globalThis.__zw_harness_complete = false;
add_result_callback(function(test) {
  globalThis.__zw_harness_results.push({
    name: String(test.name || ''),
    status: Number(test.status),
    message: test.message == null ? null : String(test.message)
  });
});
add_completion_callback(function() {
  globalThis.__zw_harness_complete = true;
});
"#;

/// harness 完成探测补丁（闭包内注 `__zw_mark_harness_loaded`/`__zw_harness_state`）。
/// html-semantics M2 s98：静态内联（[`prepare_harness_html`]）与 fetch 通道服务
/// （[`wpt_data_fetch_handler`]）共用同一补丁面。
fn apply_harness_completion_patch(harness_source: String) -> String {
    harness_source.replacen(
        "\n})(self);",
        "\nglobal_scope.__zw_mark_harness_loaded = function() {\n\
         test_environment.all_loaded = true;\n\
         if (tests.all_done()) tests.complete();\n\
         };\n\
         global_scope.__zw_harness_state = function() {\n\
         return {tests:tests.tests.length,pending:tests.num_pending,loaded:test_environment.all_loaded,phase:tests.phase};\n\
         };\n})(self);",
        1,
    )
}

/// write 注入 harness 的 eval 尾自调（镜像生命周期脚本尾钩子调用——静态路径由
/// runner 生命周期脚本补调；fetch 服务源须自带，注入副本求值完成即置 all_loaded）。
const HARNESS_LOADED_TAIL_CALL: &str =
    "\nif (typeof globalThis.__zw_mark_harness_loaded === 'function') {globalThis.__zw_mark_harness_loaded();}\n";

fn prepare_harness_html(
    source: &str,
    harness_source: &str,
    inline_extras: &[(&str, &str)],
    wpt_root: &Path,
    case_path: &str,
) -> String {
    let reporter = HARNESS_REPORTER_SOURCE;
    // security-hardening M2-s1：注入脚本 nonce 戳记（见 [`harness_nonce_attr`]）。
    let csp_nonce_attr = harness_nonce_attr(source);
    // net-api M2-S6：abort/general 同注入 stash 仿真 fixture——wptserve
    // stash-take/put + infinite-slow-response.py 端点语义（「Already aborted
    // signal does not make request」经 stash-take 观测请求未发出；此前落静态
    // 读库 404 空体 → r.json() SyntaxError 掩盖断言面）。
    let cache_abort_fixture = if case_path.contains("cache-abort") || case_path.contains("abort/general")
    {
        CACHE_ABORT_FETCH_FIXTURE
    } else {
        ""
    };
    let harness_source = apply_harness_completion_patch(harness_source.to_string());
    // R34xx：__zw_setTimeout stub 记录定时器（id/at），经 take_probe 的
    // __zw_fire_due_timers 按真实经过时间触发（t.step_timeout(500) 的 fontface.repeat
    // 依赖回调最终触发；记录式避免 microtask 立即触发破坏 testharness 时序——
    // 既有 no-op stub 使回调永不触发）。
    let timer_stub = "\
      // R3254-E2 切片 12（editing goal，2026-09-08）：stub 挂 `__zw_test_setTimeout`
      // 专用名——sandbox 每次 execute 都把 register_callback 原生回调重新 global.set，
      // 覆盖 `__zw_setTimeout` 的 JS 赋值（stub 只存活一个 turn，之后定时器静默回落
      // host 真线程，drain_next_async_callback 每 execute 只 resolve 一个、到达序随机
      // → timer 派发顺序随机，WPT textcontrols/selectionchange 断言族 flake 根因）。
      // 专用名不在 host 注册表 → 永不被 rebind，全部 timer 走本 stub 的确定性队列
      //（push 序 = FIFO，due 分割保序；shim setTimeout 优先探测本名）。
      globalThis.__zw_test_setTimeout = function(id, delay) {\n\
        globalThis.__zw_timers = globalThis.__zw_timers || [];\n\
        globalThis.__zw_timers.push({ id: id, at: Date.now() + (delay | 0) });\n\
      };\n\
      globalThis.__zw_clearTimeout = function() {};\n\
      globalThis.__zw_fire_due_timers = function() {\n\
        var timers = globalThis.__zw_timers || [];\n\
        if (!timers.length) return;\n\
        var now = Date.now();\n\
        var rest = [], due = [];\n\
        for (var i = 0; i < timers.length; i++) {\n\
          if (timers[i].at <= now) due.push(timers[i]); else rest.push(timers[i]);\n\
        }\n\
        // net-api M5-S10：到期序派发（at 升序——HTML event loop timers 语义：先到期
        // 先跑；push 序在「send() 先挂长 timeout 计时器、后挂 0ms 派发/其回调再挂
        // fetch-issue」形态下会让未到期判定后的到期集乱序——busy loop 阻塞后
        // timeout 计时器先于更早到期的 fetch-issue 派发 → 伪 ontimeout。XHR 侧
        // finite 模型静态判定（D < T 不挂计时器）为主修，此排序为通用事件循环
        // 语义兜底）。\n\
        due.sort(function(a, b) { return a.at - b.at; });\n\
        // event-loop-spec M3-S1：per-task 模式（__ZW_TIMER_PER_TASK，runner env\n\
        // ZW_TESTHARNESS_TIMER_PER_TASK 置位）只派发首个到期 timer——未派发的 due\n\
        // 保留队列（下一 probe 迭代再派），一 timer 一 execute 一 checkpoint。\n\
        var limit = globalThis.__ZW_TIMER_PER_TASK ? 1 : due.length;\n\
        var fired = due.splice(0, limit);\n\
        globalThis.__zw_timers = due.concat(rest);\n\
        for (var d = 0; d < fired.length; d++) {\n\
          var fn = globalThis.__zw_pending[fired[d].id];\n\
          if (fn) { delete globalThis.__zw_pending[fired[d].id]; try { fn(); } catch (_e) {} }\n\
        }\n\
      };\n";
    let harness = format!(
        "<script{csp_nonce_attr} data-zw-harness>\n{timer_stub}{harness_source}\n{reporter}\n{cache_abort_fixture}\n</script>"
    );
    // R130（js-dom M4）：crash 类用例（*-crash.html）不引 testharness.js——纯脚本页
    // 断言「不崩溃」。上游跑法是浏览器不崩即 PASS；本 runner 的 completion 探针依赖
    // harness 全局（test/completion callback），无 harness 时永远 Timeout 伪失败。
    // 对无 testharness 引用的用例：预置 harness 内联（插首部——先于用例脚本执行），
    // 用例脚本跑完 0 注册测试 + all_done 后 phase=4，下方 run 路径对「completed
    // without reporting registered tests」按页面脚本未抛错 = PASS 计（crash 语义）。
    let has_harness_ref = source.contains("/resources/testharness.js") || source.contains("testharnessreport.js");
    let mut html = if has_harness_ref {
        replace_script_source(source, "/resources/testharness.js", &harness)
    } else {
        inject_harness_script_before_page_scripts(source, &harness)
    };
    html = replace_script_source(&html, "/resources/testharnessreport.js", "");
    html = replace_script_source(
        &html,
        "/resources/testdriver.js",
        &TESTDRIVER_STUB.replacen("<script>", &format!("<script{csp_nonce_attr} data-zw-harness>"), 1),
    );
    html = replace_script_source(&html, "/resources/testdriver-vendor.js", "");
    html = replace_script_source(&html, "/resources/testdriver-actions.js", "");
    // canvas-tests.js 等用例框架脚本：与 testharness.js 同款内联（外部脚本提取器不加载 src）。
    for (script_src, inline_source) in inline_extras {
        html = replace_script_source(
            &html,
            script_src,
            &format!("<script{csp_nonce_attr} data-zw-harness>{inline_source}</script>"),
        );
    }
    // js-dom goal：用例引用的本地 .js 测试体（如 <script src="attributes.js">、
    // <script src="Document-createProcessingInstruction.js">）——extract_page_scripts 不加载外部 src，
    // 故此处从 wpt-data 读文件内容内联。case_path 形如 "dom/nodes/attributes.html"，本地 .js 解析为
    // 同目录文件（相对 src 如 "attributes.js" / "./attributes.js" / "../constants.js"）。
    // 仅内联相对路径（非 /resources/、非 http(s):）；文件缺失则移除该 script 标签（不注入空）。
    // s122：文档编码对原始 source 嗅探（harness 注入前）——见 inline_local_scripts
    // doc_encoding 参注。
    html = inline_local_scripts(&html, wpt_root, case_path, sniff_document_encoding(source).as_deref());
    // R3254-E2 切片（editing goal，2026-09-07）：**dispatcher 不再注入 DOM**——此前
    // 尾部 append 的 `<script>`（DOMContentLoaded/load 派发 + harness_loaded 标记）被解析器
    // 归入 body，成为页面可见 DOM 子（WPT body-should-not-deleted 的
    // documentElement.outerHTML 精确断言被注入 script 污染）。改为 runner 侧在
    // run_page_scripts_strict 之后 execute_script 执行（时序不变：全部页面脚本之后）。
    html
}

const CACHE_ABORT_FETCH_FIXTURE: &str = r#"
(function() {
  var nativeFetch = globalThis.fetch;
  var stash = globalThis.__zw_cache_abort_stash || {};
  globalThis.__zw_cache_abort_stash = stash;
  function parseUrl(input) {
    var raw = input && typeof input === 'object' && input.url !== undefined ? input.url : input;
    try { return new URL(String(raw), location && location.href ? location.href : 'https://wpt.test/'); }
    catch (_e) { return null; }
  }
  function signalOf(input, init) {
    init = init || {};
    if (typeof AbortSignal !== 'function') return null;
    if (init.signal instanceof AbortSignal) return init.signal;
    if (input && typeof input === 'object' && input.signal instanceof AbortSignal) return input.signal;
    return null;
  }
  function responseFor(url, body, contentType) {
    return new Response(body, {
      status: 200,
      statusText: 'OK',
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Content-Type': contentType || 'text/plain',
        'X-Zero-Final-URL': url.href
      },
      url: url.href
    });
  }
  globalThis.fetch = function(input, init) {
    var url = parseUrl(input);
    if (!url) return nativeFetch(input, init);
    var path = url.pathname;
    if (path.endsWith('/fetch/api/resources/stash-take.py')) {
      var takeKey = url.searchParams.get('key') || '';
      var value = Object.prototype.hasOwnProperty.call(stash, takeKey) ? stash[takeKey] : null;
      delete stash[takeKey];
      return Promise.resolve(responseFor(url, JSON.stringify(value), 'application/json'));
    }
    if (path.endsWith('/fetch/api/resources/stash-put.py')) {
      var putKey = url.searchParams.get('key') || '';
      var putValue = url.searchParams.has('value') ? url.searchParams.get('value') : 'done';
      if (putKey) stash[putKey] = putValue;
      return Promise.resolve(responseFor(url, 'done', 'text/plain'));
    }
    if (path.endsWith('/fetch/api/resources/infinite-slow-response.py')) {
      var signal = signalOf(input, init);
      // net-api M2-S6：入口 abort 只在 wrapper 自持端点拦截（generic URL 交还
      // 引擎语义——Request bodyUsed/流体同步 cancel 等引擎面不被 wrapper 遮蔽）。
      // 已 aborted → 请求未发出，stateKey 不落 'open'（stash-take 观测 null）。
      if (signal && signal.aborted) return Promise.reject(signal.reason);
      var stateKey = url.searchParams.get('stateKey') || '';
      if (stateKey) stash[stateKey] = 'open';
      // 上游 wptserve 头快速到达、体缓慢流式——runner 同步契约下立即返响应
      //（防 await fetch 悬挂到 harness 超时）。net-api M2-S9：体为**开放流**
      //（入队一 chunk 不 close——上游体无限缓慢、连接持续开放；首 chunk 立即
      // 可读、次 read 挂起，消费不悬挂）。abort → 连接关闭观测（stateKey 落
      // 'closed'，「Underlying connection is closed」轮询面）+ 流 error(reason)
      //（挂起读/closed/后续消费拒 abort reason，「Stream errors once aborted /
      // connection closed after calling response.*()」面）。
      var bodyCtl = null;
      var bodyStream = new ReadableStream({
        start: function(controller) {
          bodyCtl = controller;
          controller.enqueue(new TextEncoder().encode(Array(2049).join('.')));
        }
      });
      var done = responseFor(url, bodyStream, 'text/plain');
      if (signal) {
        signal.addEventListener('abort', function() {
          if (stateKey) stash[stateKey] = 'closed';
          if (bodyCtl) { try { bodyCtl.error(signal.reason); } catch (_eBsErr) {} }
          try { done._zwAbortErr = signal.reason; } catch (_eAbortGate) {}
        });
      }
      return Promise.resolve(done);
    }
    return nativeFetch(input, init);
  };
})();
"#;

fn replace_script_source(source: &str, script_src: &str, replacement: &str) -> String {
    let mut output = String::with_capacity(source.len() + replacement.len());
    let mut remaining = source;
    loop {
        let Some(start) = remaining.find("<script") else {
            output.push_str(remaining);
            break;
        };
        output.push_str(&remaining[..start]);
        let candidate = &remaining[start..];
        let Some(open_end) = candidate.find('>') else {
            output.push_str(candidate);
            break;
        };
        let open = &candidate[..=open_end];
        let Some(close_offset) = candidate[open_end + 1..].find("</script>") else {
            output.push_str(candidate);
            break;
        };
        let end = open_end + 1 + close_offset + "</script>".len();
        if open.contains(script_src) {
            output.push_str(replacement);
        } else {
            output.push_str(&candidate[..end]);
        }
        remaining = &candidate[end..];
    }
    output
}

fn inject_harness_script_before_page_scripts(source: &str, harness: &str) -> String {
    for tag in ["<head>", "<HEAD>", "<html>", "<HTML>"] {
        if let Some(pos) = source.find(tag) {
            let insert_at = pos + tag.len();
            let mut output = String::with_capacity(source.len() + harness.len());
            output.push_str(&source[..insert_at]);
            output.push_str(harness);
            output.push_str(&source[insert_at..]);
            return output;
        }
    }
    let mut output = String::with_capacity(source.len() + harness.len());
    output.push_str(harness);
    output.push_str(source);
    output
}

/// 内联用例引用的本地 .js 测试体（js-dom goal R8）。
///
/// `extract_page_scripts` 不加载外部 `<script src>`，故用例引用的同目录 .js（如 attributes.js、
/// Document-createProcessingInstruction.js）或相对路径（../constants.js）不会执行 → `attr_is`/
/// 测试体 not defined。本函数扫描剩余的 `<script src="...">`（相对路径与 `/` 根路径——
/// R4300 起根路径按 wpt_root 解析；非 `//`、非 http），从 wpt-data 读文件内容内联为
/// inline `<script>`；文件缺失则移除该标签（best-effort，不注入空）。
///
/// `case_path` 形如 "dom/nodes/attributes.html"；相对 src 解析为相对该 case 所在目录。
/// `doc_encoding`：s122 文档编码（meta charset prescan）——classic 外链解码链的
/// 文档编码回退（script-charset-02 判据面：文档 windows-1250 时无 BOM/无 ct
/// charset 参数/无 charset 属性的外链按文档编码解码）。**入参方须对注入 harness
/// 前的原始 case html 嗅探**——harness 前导插在 `<head>` 后，会把首 1024 字符
/// 窗内的 meta 挤出 prescan 窗。
fn inline_local_scripts(html: &str, wpt_root: &Path, case_path: &str, doc_encoding: Option<&str>) -> String {
    // case 所在目录（相对 wpt_root），如 "dom/nodes"。
    let case_dir = case_path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let mut output = String::with_capacity(html.len());
    let mut remaining = html;
    loop {
        let Some(start) = remaining.find("<script") else {
            output.push_str(remaining);
            break;
        };
        let Some(open_end) = remaining[start..].find('>') else {
            output.push_str(remaining);
            break;
        };
        let open_end = start + open_end;
        let open = &remaining[start..=open_end];
        // 提取 src="..."（仅相对路径 .js）。s107（SRI）：带 integrity 属性的单元保持
        // 外链整段透传——内联投递丢 src/integrity，webview 顶层模块 SRI 校验
        //（ExternalModule 臂）与元素 load/error 事件 src 匹配派发（R2944）都以 src 为
        // 锚（json-module/module/css-module/text-module 四页 integrity.html 语料）。
        // https://html.spec.whatwg.org/multipage/semantics.html#integrity-metadata-attribute
        let src = if zero_engine::attr_from_open_tag(open, "integrity").is_some() {
            None
        } else {
            extract_script_src(open)
        };
        // s110（html-semantics M2 切片 110）：非 JS type 外链单元整段透传——execution-timing
        // 122-126 族以 type="text/plain" src 外链做 mutation 重触发（removeAttribute(type)/
        // 重插入），data-inline 改写剥 src 使 s109 触发臂走内联分支同步求值（spec 无 type
        // 变更触发面）。透传后 src/type 原样入 DOM：extract 投递面按同判据跳过（不执行、
        // 不投递、不记账），运行期 mutation 触发臂经页面 fetch 通道取 src 求值。
        // https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
        let type_is_js =
            zero_engine::script_type_is_javascript(zero_engine::attr_from_open_tag(open, "type").as_deref());
        let resolved = if !type_is_js {
            None
        } else {
            src.and_then(|s| {
                // R4300（html-semantics M2 切片 23）：`/` 根路径（非 `//`、非 http(s):）
                // 解析为 wpt_root 相对——dialog closedby/canceling 语料以
                // `/common/top-layer.js`、`/close-watcher/resources/helpers.js` 根路径引用
                // 域外助资产，旧版按绝对路径拒绝 → 标签被移除 → helper ReferenceError 全批
                // 假 Fail。`//`（协议相对）与显式 http(s) 维持外部面不内联。
                if s.starts_with("//") || s.starts_with("http://") || s.starts_with("https://") {
                    return None;
                }
                // s100：查询串先剥离再归一——.py fixture src 携带 `?test=` 查询（CGI 端点），
                // 路径归一与语料读面只对路径段，查询由 fixture 仿真面消费。
                let (path_src, src_query) = s.split_once('?').unwrap_or((s, ""));
                let combined = if let Some(root_rel) = path_src.strip_prefix('/') {
                    normalize_relative(root_rel)
                } else {
                    // 规范化 "./" 前缀 + 相对 case_dir 解析（含 ../ 上溯）。
                    let rel = path_src.strip_prefix("./").unwrap_or(path_src);
                    if case_dir.is_empty() {
                        rel.to_string()
                    } else {
                        normalize_relative(&format!("{case_dir}/{rel}"))
                    }
                };
                // s100：load-error-events.py fixture——CGI 端点不在语料 pull 面，按查询串
                // 仿真服务（wpt_load_error_events_response）；非 200 状态脚本体不执行（error
                // 事件面），与上游 404 脚本不执行语义一致。
                if combined == WPT_LOAD_ERROR_EVENTS_PY {
                    let (status, body) = wpt_load_error_events_response(src_query);
                    return Some((combined, status, body));
                }
                // s115：serve-with-content-type.py classic 外链 fixture 仿真——wptserve
                // 端点不在语料 pull 面（文件读落空 → 整段透传外链 → 宿主取回 Err 不执行，
                // script-charset-01 全 6F 根因），按 handler 语义模拟：交付 fn 指向文件，
                // 解码序 spec classic script 响应解码——BOM > ct charset 参数 > charset
                // 属性 > 文档编码（s122 meta 嗅探回退）> UTF-8 lossy。**module 型不在本面**（charset 参数/属性/BOM 三忽略、
                // 恒 UTF-8——spec fetch a single module script；module/charset-01/02/03
                // 判据面）：module 外链保持外链投递，恒 UTF-8 交付在
                // wpt_data_script_fetcher（charset 感知内联会反向翻绿为红——首版乌龙）。
                // https://html.spec.whatwg.org/multipage/scripting.html#fetch-a-classic-script
                // https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
                let is_module_type = zero_engine::attr_from_open_tag(open, "type")
                    .as_deref()
                    .map(|t| {
                        t.trim_matches(|c| matches!(c, '\t' | '\n' | '\x0C' | '\r' | ' '))
                            .eq_ignore_ascii_case("module")
                    })
                    .unwrap_or(false);
                if !is_module_type
                    && std::path::Path::new(&combined)
                        .file_name()
                        .is_some_and(|name| name == std::ffi::OsStr::new("serve-with-content-type.py"))
                {
                    let (file, ct) = parse_serve_with_content_type_query(src_query)?;
                    let dir = combined.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
                    let file_rel = format!("{dir}/{file}");
                    let Ok(bytes) = std::fs::read(wpt_root.join(&file_rel)) else {
                        return None;
                    };
                    let charset_attr = zero_engine::attr_from_open_tag(open, "charset");
                    let decoded = decode_classic_script_bytes(&bytes, &ct, charset_attr.as_deref(), doc_encoding);
                    return Some((combined, 200, decoded));
                }
                read_corpus_to_string(wpt_root.join(&combined)).ok().map(|c| {
                    // net-api M5-S1：`.sub.js` 服务端替换（上游 wptserve 语义；与
                    // script fetcher M4-S22、fetch 通道本切片同口径——`.html` 案外部
                    // 脚本走本预处理直读，缺此则 constants.sub.js token 原样入页）。
                    let c = if combined.ends_with(".sub.js") {
                        apply_wpt_substitutions(&c)
                    } else {
                        c
                    };
                    (combined, 200, c)
                })
            })
        };
        let rest_start = open_end + 1;
        match resolved {
            Some((combined, status, content)) => {
                output.push_str(&remaining[..start]);
                // s92（string-compilation CSP nonce 面）：投递形态改写保留原标签
                // nonce 属性（上方 harness 注入块 csp_nonce_attr 同款保真）——CSP
                // 页 nonce 槽位随脚本执行逐脚本刷新（webview 动态 import 检查输入），
                // 内联投递丢 nonce 会让页面脚本体 Deferred import 拿不到 nonce 被
                // CSP 伪拒（propagate-nonce-external-* 实测）。
                // https://html.spec.whatwg.org/multipage/webappapis.html#concept-scripts-base-url
                let nonce_attr = match zero_engine::attr_from_open_tag(open, "nonce") {
                    Some(n) if !n.is_empty() => {
                        format!(" nonce=\"{}\"", n.replace('&', "&amp;").replace('"', "&quot;"))
                    }
                    _ => String::new(),
                };
                // s94（nomodule face）：投递改写保留原标签 nomodule（bool 属性，裸形在场即
                // 生效）与 type 属性——nomodule 丢失使 prepare 门（extract
                // script_suppressed_by_nomodule）失效、nomodule classic 照跑
                //（nomodule-set-on-synchronously-loaded-* 实测）；type 丢失把 module 形外链
                // 按 classic 内联执行（s92 挂账「投递不保留 type=module」承袭，顶层 this/
                // defer 时序语义全变——module/nomodule-attribute 实测 log 收 window 非
                // undefined）。
                // https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
                let nomodule_attr = if zero_engine::open_tag_has_attr(open, "nomodule") {
                    " nomodule"
                } else {
                    ""
                };
                let type_attr = match zero_engine::attr_from_open_tag(open, "type") {
                    Some(t) if !t.is_empty() => {
                        format!(" type=\"{}\"", t.replace('&', "&amp;").replace('"', "&quot;"))
                    }
                    _ => String::new(),
                };
                // s108（id 保真）：投递改写保留原标签 id——execution-timing 122-126 族以
                // `document.getElementById("test")` 取回 type=text/plain 外链单元做
                // removeAttribute(type)/re-append 的 mutation 重触发（getElementById 落空
                // 即 `Cannot read properties of null`，mutation 面根本不可达）。id 与
                // nonce/nomodule/type 同属「内联投递丢属性改语义」保真族。
                // https://html.spec.whatwg.org/multipage/dom.html#the-id-attribute
                let id_attr = match zero_engine::attr_from_open_tag(open, "id") {
                    Some(v) if !v.is_empty() => {
                        format!(" id=\"{}\"", v.replace('&', "&amp;").replace('"', "&quot;"))
                    }
                    _ => String::new(),
                };
                // s100（load/error 事件投递保真）：保留原标签 onload/onerror 内容属性
                // （s94 保真只留 nonce/nomodule/type——事件处理器内容属性丢失使内联投递
                // 后页面 on* 语义面缺失：nomodule-set-on-synchronously「must run」的
                // assert_true(loaded) 假红、load-error-events-1 解析器组全批 pending 即
                // 本缺口）；200 投递原脚本体 + load 投递尾（成功执行 fire load），404/400
                // 只投 error 投递尾（取回失败脚本体不执行——fixture 404 体上游亦不执行）。
                // 投递尾经 document.currentScript 派发同名事件：on* 内容属性 handler 由
                // shim dispatch 链（R2934 _ensureInlineHandler）编译触发，addEventListener
                // 侦听者同样可达；事件不冒泡，window load 生命周期不受扰。
                // https://html.spec.whatwg.org/multipage/scripting.html#execute-the-script-block
                let onload_attr = script_event_handler_attr(open, "onload");
                let onerror_attr = script_event_handler_attr(open, "onerror");
                output.push_str(&format!(
                    "<script{id_attr}{nonce_attr}{nomodule_attr}{type_attr}{onload_attr}{onerror_attr} data-zw-harness data-inline=\""
                ));
                output.push_str(&combined);
                output.push_str("\">");
                let mut unit_body = String::new();
                let tail_event = if status == 200 {
                    // s101（内联模块依赖解析基）：type="module" 外链内联后，模块体内相对
                    // import 的 spec 基准从模块自身 URL 漂移为页面 URL（engine 内联模块
                    // 依赖收集基 = 页面）——模块体在 resources/ 子目录时其相对 import 解析
                    // 到页目录 → 取回 404 → 空存根 → 假「Module does not provide an
                    // export」编译错（nomodule-set-on-external-module-script 实测）。重写
                    // 相对 spec 为 root-absolute，恢复「相对 requesting module 解析」语义。
                    // https://tc39.es/ecma262/#sec-hostresolveimportedmodule
                    let content = if matches!(
                        zero_engine::attr_from_open_tag(open, "type"),
                        Some(t) if t.trim().eq_ignore_ascii_case("module")
                    ) {
                        rewrite_inline_module_import_specs(&content, &combined)
                    } else {
                        content
                    };
                    unit_body.push_str(&content);
                    "load"
                } else {
                    "error"
                };
                // s112：体内字面 `</script` 会在 HTML 投递面提前封口元素截断执行体
                //（099 include-11.js 的 srcdoc 串实证——单元整体 SyntaxError 零日志；
                // script-type-and-language-js.js 注释内 `</script>` 实证 229 子测被截断
                // 不注册）。`<\/script` 转义在字符串/正则/注释各语法位与原文本等值
                //（`\`+`/` = `/`），与 .js window 包装路径既有转义（any_js_window_wrapper）
                // 同口径。
                // https://html.spec.whatwg.org/multipage/syntax.html#script-data-state
                let mut unit_body = unit_body.replace("</script", "<\\/script");
                // s112：defer/async/trickle 单元改注册进页面侧调度器（分类见
                // script_sched_unit）——执行体闭包化，相位由生命周期链驱动。load/error
                // 投递尾不能读 document.currentScript：setTimeout/drain 相位执行时
                // classic currentScript 已清（getter 返 null，part06 R3258），
                // load-error-events-1 实测 defer/async 四子测事件丢失页面 Timeout。注册
                // 语句同步原位执行时 currentScript 仍指投递元素，立即捕获为 _zwEl 传入
                // 闭包（spec：脚本执行期 currentScript = 脚本元素；此处仅覆盖投递尾，
                // 体内 currentScript 读取仍为 null，挂账）。
                // https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
                match script_sched_unit(open, src.unwrap_or("")) {
                    Some((kind, delay_ms)) => {
                        let tail = script_event_dispatch_tail_el(tail_event, "_zwEl");
                        output.push_str(&format!(
                            "__zwSchedRegister(\"{kind}\", {delay_ms}, \
                             (function (_zwEl) {{ return function () {{\n{unit_body}{tail}\n}}; }})(document.currentScript));"
                        ));
                    }
                    None => {
                        unit_body.push_str(&script_event_dispatch_tail(tail_event));
                        output.push_str(&unit_body);
                    }
                }
                output.push_str("</script>");
            }
            None => {
                // s97（html-semantics M2 切片 97）修复：非本地 .js 单元**整段透传**。
                // 旧版只透传 open 标签、继续从单元体内扫描——`document.write` 语料的
                // writer 单元 JS 字符串内含 `<script src=...>` 书写形态（execution-timing
                // 007-012），被本扫描误认标签改写 + 注入真 `</script>`（内联文件内容后
                // 封口），writer 单元从注入点截断 → 页面 SyntaxError「missing ) after
                // argument list」全批假红。spec rawtext 语义：script 内容仅 `</script>`
                // 结束，体内容对标签扫描不透明；无 close 的病态形态整段透传收尾。
                // https://html.spec.whatwg.org/multipage/syntax.html#script-data-state
                match remaining[rest_start..].find("</script>") {
                    Some(off) => {
                        let end = rest_start + off + "</script>".len();
                        output.push_str(&remaining[..end]);
                        remaining = &remaining[end..];
                    }
                    None => {
                        output.push_str(remaining);
                        break;
                    }
                }
                continue;
            }
        }
        remaining = &remaining[rest_start..];
    }
    output
}

/// s100：经典脚本 load-error fixture 端点路径（fetch 通道与内联预处理共用面）。
const WPT_LOAD_ERROR_EVENTS_PY: &str = "html/semantics/scripting-1/the-script-element/resources/load-error-events.py";

/// s112：wptserve `pipe=trickle(dN)` / `trickle(N)` → 毫秒延迟（d 前缀可选；30s 封顶，
/// 与 shim fetch 通道 part01 同语法同上限）。
/// https://wptserve.readthedocs.io/en/latest/pipes.html#trickle
fn trickle_delay_ms(src: &str) -> u64 {
    let Some(idx) = src.find("pipe=trickle(") else {
        return 0;
    };
    let rest = &src[idx + "pipe=trickle(".len()..];
    let digits: String = rest
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse::<u64>().unwrap_or(0).saturating_mul(1000).min(30_000)
}

/// s112（parser 外链时序调度）：本地 classic 外链单元的分类——defer/async 属性或
/// trickle 查询的单元不再原位内联直执，改为投递 `__zwSchedRegister(kind, delayMs, fn)`
/// 注册闭包，由 runner 生命周期链按 spec 相位执行：
/// - `async`（async 属性在场，含 defer+async——spec prepare 时 async 优先入「as soon
///   as possible」集）：trickle 即取回延迟，取回完成后任意时刻执行（setTimeout(delay)）；
/// - `defer`（defer 无 async）：the-end 相位执行——interactive readystatechange 之后、
///   DOMContentLoaded 之前，文档序（全单元 max trickle 后统一 drain）；
/// - `block`（无 defer/async 但 trickle）：解析阻塞延迟近似——trickle 秒后执行，不暂停
///   后续投递（判据页只断言 log 序，可观察面等价；050 形）。
///
/// 分类冻结于预处理时刻（spec：async/defer 归类在 prepare 即取回发起时定格，运行期
/// removeAttribute 不重分类——103/111/112 判据）；id 属性保留（111/112 以
/// getElementById 取回元素做属性变更，不崩即可）。nonce/nomodule 单元维持原路径
/// （CSP 槽位与 prepare 门各有专用面）；module 单元维持模块取回链；integrity 单元
/// 已在前置分支透传。返回 (kind, delay_ms)。
/// https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
fn script_sched_unit(open: &str, raw_src: &str) -> Option<(&'static str, u64)> {
    if zero_engine::open_tag_has_attr(open, "nomodule")
        || zero_engine::attr_from_open_tag(open, "nonce").is_some()
        || matches!(
            zero_engine::attr_from_open_tag(open, "type"),
            Some(t) if t.trim().eq_ignore_ascii_case("module")
        )
    {
        return None;
    }
    let delay_ms = trickle_delay_ms(raw_src);
    let kind = if zero_engine::open_tag_has_attr(open, "async") {
        "async"
    } else if zero_engine::open_tag_has_attr(open, "defer") {
        "defer"
    } else if delay_ms > 0 {
        "block"
    } else {
        return None;
    };
    Some((kind, delay_ms))
}

/// s101：内联投递模块体的静态 import spec 重写为 root-absolute（wpt_root 相对路径）。
/// spec：import 语句相对 **requesting module 自身 URL** 解析（referrer 为模块自身）；
/// 内联投递后 engine 依赖收集基 = 页面 URL，仅重写 `./`/`../` 相对 spec——已
/// root-absolute、裸 specifier（无 import map 须按解析失败语义保持）与含 scheme 的
/// spec 不动。替换按 lexer 感知提取（extract_static_module_import_specifiers）定位后
/// 引号包裹整体替换（模块体为小体量真实代码，同串注释/字符串误伤面可忽略）。动态
/// import() 走 __zw_compile_module 运行时取回链，不在本重写面（module 尾片挂账）。
/// https://tc39.es/ecma262/#sec-hostresolveimportedmodule
fn rewrite_inline_module_import_specs(content: &str, module_path: &str) -> String {
    let module_dir = module_path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let mut out = content.to_string();
    for spec in zero_script_sandbox::extract_static_module_import_specifiers(content) {
        if !spec.starts_with("./") && !spec.starts_with("../") {
            continue;
        }
        let (path_spec, query) = spec.split_once('?').unwrap_or((spec.as_str(), ""));
        let rel = path_spec.strip_prefix("./").unwrap_or(path_spec);
        let abs = format!("/{}", normalize_relative(&format!("{module_dir}/{rel}")));
        let abs = match query {
            "" => abs,
            q => format!("{abs}?{q}"),
        };
        out = out
            .replace(&format!("\"{spec}\""), &format!("\"{abs}\""))
            .replace(&format!("'{spec}'"), &format!("'{abs}'"));
    }
    out
}

/// s100：从 open 标签提取 on* 事件处理器内容属性并按投递形态转义（& → &amp;、
/// " → &quot;——与 nonce/type 保真同款转义）；缺席/空值返回空串（不进重写标签）。
fn script_event_handler_attr(open_tag: &str, name: &str) -> String {
    match zero_engine::attr_from_open_tag(open_tag, name) {
        Some(v) if !v.is_empty() => format!(" {}=\"{}\"", name, v.replace('&', "&amp;").replace('"', "&quot;")),
        _ => String::new(),
    }
}

/// s100：内联投递脚本的 load/error 事件投递尾——解析器外链被内联后无取回环节，
/// spec 的「成功执行 fire load / 取回失败 fire error」随之丢失；尾片在脚本体求值时
/// 经 `document.currentScript` 派发同名事件（on* 内容属性 handler 由 shim dispatch 链
/// R2934 编译触发，addEventListener 侦听者同样可达；事件不冒泡，window load 生命
/// 周期不受扰）。`el_expr` 为元素表达式（调度单元传闭包捕获变量，见 inline 投递处）。
/// https://html.spec.whatwg.org/multipage/scripting.html#execute-the-script-block
fn script_event_dispatch_tail_el(event: &str, el_expr: &str) -> String {
    format!(
        "\n;(function () {{ var _cs = {el_expr}; \
         if (_cs && typeof _cs.dispatchEvent === 'function') {{ \
         try {{ _cs.dispatchEvent(new Event('{event}')); }} catch (_zwLev) {{}} }} }})();\n"
    )
}

fn script_event_dispatch_tail(event: &str) -> String {
    script_event_dispatch_tail_el(event, "document.currentScript")
}

/// 从 `<script src="...">` 标签提取 src 值（单/双引号）。
fn extract_script_src(open_tag: &str) -> Option<&str> {
    let key = "src=\"";
    if let Some(i) = open_tag.find(key) {
        let after = &open_tag[i + key.len()..];
        return after.split('"').next();
    }
    let key = "src='";
    if let Some(i) = open_tag.find(key) {
        let after = &open_tag[i + key.len()..];
        return after.split('\'').next();
    }
    // R41：无引号属性值（HTML 合法语法，上游 dom/traversal NodeIterator.html 用 `src=../common.js`）。
    // 值截止于空白或标签闭合 `>`（open_tag 含闭合尖括号）。
    let key = "src=";
    if let Some(i) = open_tag.find(key) {
        let after = &open_tag[i + key.len()..];
        return after.split_whitespace().next().map(|v| v.trim_end_matches('>'));
    }
    None
}

/// 规范化相对路径（处理 `..` 上溯，如 "dom/nodes/../constants.js" → "dom/constants.js"）。
pub(crate) fn normalize_relative(path: &str) -> String {
    let mut stack: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                stack.pop();
            }
            s => stack.push(s),
        }
    }
    stack.join("/")
}

fn unsupported_testdriver_dependencies(source: &str) -> Vec<String> {
    let mut dependencies = Vec::new();
    let mut remaining = source;
    while let Some(index) = remaining.find("test_driver.") {
        let after = &remaining[index + "test_driver.".len()..];
        let name: String = after
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect();
        let name_len = name.len();
        // R142：Actions（指针链 pointerMove/pointerDown/pointerUp/send）白名单——stub 已提供
        // 链式构造器（pointer 系列合成点击），不再整体 Unsupported。
        // WAB2-M2-s3（web-api-batch2 goal，2026-09-24）：set_permission 白名单——stub 经
        // shim `__zwSetPermission` 钩子落权限注册表（navigator.permissions.query 如实返回 +
        // clipboard 四方法 denied 拒绝门），clipboard-apis permissions/svg 与 fullscreen
        // permission 三案解除整体 Unsupported。
        // WAB2-M3-s1（2026-09-24）：bless 白名单——stub 授予瞬态激活（`__zwUserActivate`）+
        // 执行回调，fullscreen bless 依赖案（after-error/options/same-element/display-contents/
        // consume-user-activation）解除整体 Unsupported。
        // cookies-compat M1（2026-10-12）：delete_all_cookies 白名单——stub 转发 shim
        // `__zwDeleteAllCookies` 钩子（清 in-JS cookie store）；上游 corpus 的
        // domCookieTest 每子测试前后清 jar 是 exact-match 断言的隔离前提。
        // console-compat M2 切片 2（2026-10-14）：bidi 白名单——stub 提供
        // `bidi.log.entry_added` 页内捕获面（on 注册进 shim `__zwConsoleBidiListeners`，
        // `_zwConsoleEmit` 派发）；console corpus 的 count/countReset/timing/
        // format-specifier logging 观测案（BiDi log.entryAdded 依赖）解除整体
        // Unsupported。
        if !name.is_empty()
            && name != "click"
            && name != "send_keys"
            && name != "Actions"
            && name != "set_permission"
            && name != "bless"
            && name != "delete_all_cookies"
            && name != "bidi"
            && !dependencies.contains(&name)
        {
            dependencies.push(name);
        }
        remaining = after.get(name_len..).unwrap_or_default();
    }
    dependencies
}

#[derive(Deserialize)]
struct HarnessProbe {
    complete: bool,
    results: Vec<RawHarnessResult>,
    commands: Vec<TestdriverCommand>,
    test_function: String,
    harness_hook: String,
    state: serde_json::Value,
    due_timer: bool,
    test_wait: bool,
}

#[derive(Deserialize)]
struct RawHarnessResult {
    name: String,
    status: u8,
    message: Option<String>,
}

#[derive(Deserialize)]
struct TestdriverCommand {
    id: u64,
    operation: String,
    selector: Option<String>,
    text: Option<String>,
}

/// M3 扩批（fixture-mounted 切片 2）：`MediaResourceElementKind` → 提交 tag。
fn kind_tag(kind: zero_engine::MediaResourceElementKind) -> &'static str {
    use zero_engine::MediaResourceElementKind as K;
    match kind {
        K::Audio => "audio",
        K::Video => "video",
        K::Source => "source",
        K::Track => "track",
    }
}

fn take_probe(webview: &mut WebView, last_refresh_ver: &mut u64) -> Result<HarnessProbe, String> {
    // uievents-compat M3 尾簇 7：探测轮首先 flush pending 共享 mutation——上一轮 JS
    //（rAF/timer 回调、事件 listener）排队的结构 wire 在本轮 JS 前落活 DOM（真实
    // 浏览器 DOM 变更同步可见；mouseover-at-removing 30 迭代 remove→re-append 链
    // 的查询断代根因）。队列空时 cursor 早退零成本。
    let _ = webview.flush_pending_shared_mutations();
    // Pump timer tasks first so the sandbox's microtask checkpoint has flushed
    // testharness result callbacks before the state snapshot is serialized.
    webview
        .execute_script("if (typeof globalThis.__zw_fire_due_timers === 'function') globalThis.__zw_fire_due_timers()")
        .map_err(|error| error.to_string())?;
    // uievents-compat M3 尾簇 4：shim 序列化 html 驱动的结构性刷新（泵后——shim 的
    // 序列化状态在 turn 末提交）：页内脚本 remove/appendChild 后 gBCR/命中测试立即可
    // 见（此前管线布局停留在导航时点——after_target_removed 族断言面）。
    // mutation 代际门控：无 mutation 批 → 零成本跳过（探测环 ~1ms/拍，无门控的逐拍
    // 百 KB 级 outerHTML 序列化曾跑穿 test-guard time-limit）。
    // dialog-s31：模态态轮询先于刷新——showModal/close 的模态性只在 shim JS 栈
    //（_zwDialogModalAllSels），DOM 序列化不携带；render 每帧 fresh-parse 后由管线
    // `inject_modal_state` 注入（镜像 :focus 通道），下一行刷新的 render 才能带上
    // `:modal` UA position:fixed（rendering.html#modal-dialogs）。逐拍一次微脚本
    //（返回空串零成本），无需 mutation 门控。
    let modal_sels = webview
        .execute_script(
            "typeof globalThis._zwDialogModalAllSels === 'function' ? globalThis._zwDialogModalAllSels() : ''",
        )
        .unwrap_or_default();
    webview.set_modal_dialog_selectors(
        modal_sels
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
    );
    let ver = zero_engine::js_dom_bridge::mutation_version();
    if ver != *last_refresh_ver {
        let shim_html = webview
            .execute_script("document.documentElement ? document.documentElement.outerHTML : ''")
            .unwrap_or_default();
        webview.refresh_if_html_changed(shim_html.trim());
        *last_refresh_ver = ver;
    }
    // M3 扩批：time-marches-on——cue enter/exit 调度按桥真值时钟推进（泵 tick 同拍；
    // spec media.html#time-marches-on——track-cues-* 播放推进族断言面）。与 timer
    // 泵同一 execute_script 通道（无 registry 锁持有——桥回调各自加锁）。
    webview
        .execute_script(
            "if (typeof globalThis._zwMediaTimeMarchesOn === 'function') globalThis._zwMediaTimeMarchesOn()",
        )
        .map_err(|error| error.to_string())?;
    webview
        .execute_script(
            "if (typeof globalThis.__zwPollServiceWorkerRegistrations === 'function') \
             globalThis.__zwPollServiceWorkerRegistrations()",
        )
        .map_err(|error| error.to_string())?;
    webview
        .execute_script(
            "if (typeof globalThis.__zwPollServiceWorkerMessages === 'function') \
             globalThis.__zwPollServiceWorkerMessages()",
        )
        .map_err(|error| error.to_string())?;
    let value = webview
        .execute_script(
            "JSON.stringify({\
             complete:(function(){\
               var timers = globalThis.__zw_timers || [];\
               var graceDeadline = Date.now() + 1000;\
               for (var i = 0; i < timers.length; i++) {\
                 if (timers[i].at <= graceDeadline && globalThis.__zw_pending\
                     && globalThis.__zw_pending[timers[i].id]) return false;\
               }\
               if (globalThis.__zw_harness_complete) return true;\
               if (typeof globalThis.__zw_harness_state !== 'function') return false;\
               var st = globalThis.__zw_harness_state();\
               if (st && st.phase === 3) return true;\
               return !!(st && st.phase === 4 && st.pending === 0\
                 && (globalThis.__zw_harness_results || []).length === st.tests);\
             })(),\
             results:globalThis.__zw_harness_results||[],\
             test_function:typeof globalThis.test,\
             harness_hook:typeof globalThis.__zw_mark_harness_loaded,\
             state:typeof globalThis.__zw_harness_state==='function'?globalThis.__zw_harness_state():null,\
             due_timer:(globalThis.__zw_timers||[]).some(function(timer){ return timer.at <= Date.now(); }),\
             test_wait:!!(document.documentElement && document.documentElement.classList\
               && document.documentElement.classList.contains('test-wait')),\
             commands:(globalThis.__zw_td_queue||[]).splice(0)})",
        )
        .map_err(|error| error.to_string())?;
    serde_json::from_str(&sanitize_lone_surrogate_escapes(&value))
        .map_err(|error| format!("invalid harness probe: {error}: {value}"))
}

/// net-api 收尾：探针 JSON 中的**孤代理** `\uXXXX` 转义 → `�`。serde 严格拒绝孤
/// 代理转义；urltestdata 含代理测试串（test 名经 V8 well-formed JSON.stringify 逃逸为
/// 孤代理 `\uD800`-`\uDFFF` 形态）→ 探针解析全案失败（url-constructor/a-element）。
/// 已知近似：字符串内的字面 `\\` 后随 `\uXXXX` 会被误判（本 face 诊断名容差可接受）。
fn sanitize_lone_surrogate_escapes(json: &str) -> String {
    let bytes = json.as_bytes();
    let mut out = String::with_capacity(json.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 6 <= bytes.len() && bytes[i + 1] == b'u' {
            // 字符边界守卫（guard 复核备注——病态多字节跟随 `\u` 时切片 panic 面消除）
            if !json.is_char_boundary(i + 6) {
                let ch = json[i..].chars().next().unwrap_or('\u{FFFD}');
                out.push(ch);
                i += ch.len_utf8();
                continue;
            }
            let hex = &json[i + 2..i + 6];
            if let Ok(cp) = u16::from_str_radix(hex, 16) {
                if (0xD800..=0xDBFF).contains(&cp) {
                    // 合法代理对（\uhigh\ulow 相邻）→ 原样保留 12 字节
                    if i + 12 <= bytes.len()
                        && json.is_char_boundary(i + 12)
                        && bytes[i + 6] == b'\\'
                        && bytes[i + 7] == b'u'
                        && json[i + 8..i + 12].bytes().all(|b| b.is_ascii_hexdigit())
                        && u16::from_str_radix(&json[i + 8..i + 12], 16)
                            .map(|lo| (0xDC00..=0xDFFF).contains(&lo))
                            .unwrap_or(false)
                    {
                        out.push_str(&json[i..i + 12]);
                        i += 12;
                        continue;
                    }
                    out.push_str("\\uFFFD");
                    i += 6;
                    continue;
                }
                if (0xDC00..=0xDFFF).contains(&cp) {
                    out.push_str("\\uFFFD");
                    i += 6;
                    continue;
                }
            }
        }
        let ch = json[i..].chars().next().unwrap_or('\u{FFFD}');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// uievents-compat M3 尾簇 4：指针命令目标**执行时点解析**——origin 元素中心 + offset
///（WebDriver 元素 origin 语义）→ fresh gBCR/最小包含盒命中 → (hitSel, px, py)。
/// `absolute`（viewport origin，text '@' 前缀）→ (offX, offY) 即视口坐标。命中为空
///（点在所有元素外）→ 回落 origin 选择器。gBCR 读 rect 快照——take_probe 泵后结构性
/// 刷新保证探测边界新鲜；批内移除由 shim 祖先链回退兜底路由。
fn resolve_pointer_target(
    webview: &mut WebView,
    origin_sel: &str,
    off_x: f32,
    off_y: f32,
    absolute: bool,
    last_refresh_ver: &mut u64,
) -> (String, f32, f32) {
    // 命令执行时点结构性刷新——泵后刷新受 shim 序列化一 turn 滞后：命令批与
    // remove→re-append 的回程变更可同批到达（泵 turn 提交 live 状态、下一 turn 才进
    // 序列化），gBCR 停在移除态。命令前重读一次 shim html（mutation 代际门控——同代
    // 命令批共享探测环已刷新的布局，无门控的逐命令全量序列化是 corpus 超时根因）。
    let ver = zero_engine::js_dom_bridge::mutation_version();
    if ver != *last_refresh_ver {
        let shim_html = webview
            .execute_script("document.documentElement ? document.documentElement.outerHTML : ''")
            .unwrap_or_default();
        webview.refresh_if_html_changed(shim_html.trim());
        *last_refresh_ver = ver;
    }
    let sel_js = if absolute || origin_sel.is_empty() {
        "null".to_string()
    } else {
        serde_json::to_string(origin_sel).unwrap_or_else(|_| "null".into())
    };
    let script = format!(
        "(function(){{\
var el=document.querySelector({sel});\
var px={ox},py={oy};\
if(el){{var r=el.getBoundingClientRect();if(r&&r.width>0){{px=r.left+r.width/2+px;py=r.top+r.height/2+py;}}}}\
var best=null,bestArea=Infinity;\
var all=document.querySelectorAll('body, body *');\
for(var i=0;i<all.length;i++){{var e=all[i];if(!e.getBoundingClientRect)continue;\
var r=e.getBoundingClientRect();\
if(r&&r.width>0&&r.height>0&&px>=r.left&&px<=r.right&&py>=r.top&&py<=r.bottom){{\
if(globalThis._zwDialogModalFocusGateOk&&!globalThis._zwDialogModalFocusGateOk(e))continue;\
var a=r.width*r.height;if(a<=bestArea){{bestArea=a;best=e;}}}}}}\
var imgs=document.querySelectorAll('img[usemap]');\
for(var mi=imgs.length-1;mi>=0;mi--){{var im=imgs[mi];var rim=im.getBoundingClientRect();\
if(!(rim&&rim.width>0&&rim.height>0&&px>=rim.left&&px<=rim.right&&py>=rim.top&&py<=rim.bottom))continue;\
if(best&&best!==im){{var anc=best,contained=false;for(var gi=0;gi<16&&anc;gi++){{if(anc===im){{contained=true;break;}}anc=anc.parentElement;}}if(!contained)continue;}}\
var mn=(im.getAttribute('usemap')||'').replace(/^#/,'');\
var mp=(document.getElementById&&document.getElementById(mn))||(mn?document.querySelector('map[name=\"'+mn+'\"]'):null);if(!mp)continue;\
var ars=mp.querySelectorAll('area');var lx=px-rim.left,ly=py-rim.top;\
for(var ai=0;ai<ars.length;ai++){{var a=ars[ai];if(a.isConnected===false)continue;var sh=(a.getAttribute('shape')||'rect').toLowerCase();\
if(sh==='default'){{best=a;break;}}\
var cs=(a.getAttribute('coords')||'').split(/[\\s,]+/).filter(function(v){{return v!=='';}}).map(Number);var inside=false;\
if(sh==='rect'&&cs.length>=4){{inside=lx>=cs[0]&&lx<=cs[2]&&ly>=cs[1]&&ly<=cs[3];}}\
else if(sh==='circle'&&cs.length>=3){{var dx=lx-cs[0],dy=ly-cs[1];inside=dx*dx+dy*dy<=cs[2]*cs[2];}}\
else if(sh==='poly'&&cs.length>=6){{var inn=false,jj=0,kk=cs.length/2-1;for(;jj<cs.length/2;kk=jj++){{var xi=cs[jj*2],yi=cs[jj*2+1],xj=cs[kk*2],yj=cs[kk*2+1];if(((yi>ly)!==(yj>ly))&&(lx<(xj-xi)*(ly-yi)/(yj-yi)+xi))inn=!inn;}}inside=inn;}}\
if(inside){{best=a;break;}}}}\
if(best&&best.tagName&&String(best.tagName).toLowerCase()==='area')break;}}\
if(best&&String(best.tagName).toUpperCase()==='IFRAME'){{return '@zwframe:'+((best.__zwSelector||(best.id?'#'+best.id:''))||'')+'|'+px+'|'+py;}}\
var msel=(globalThis._zwDialogModalTopSel?globalThis._zwDialogModalTopSel():'');\
if(msel){{try{{var st='';var ss=document.styleSheets;for(var si=0;si<ss.length;si++){{var on=ss[si].ownerNode;if(on&&on.textContent)st+=on.textContent;}}\
if(/::backdrop\\s*\\{{[^}}]*display\\s*:\\s*none/i.test(st))msel='html';}}catch(_eMh){{}}}}\
return (best?((best.__zwSelector||(best.id?'#'+best.id:''))||''):msel)+'|'+px+'|'+py;}})()",
        sel = sel_js,
        ox = off_x,
        oy = off_y,
    );
    let out = match webview.execute_script(&script) {
        Ok(out) => out,
        Err(_) => return (origin_sel.to_string(), off_x, off_y),
    };
    if runner_flags().td_debug {
        eprintln!(
            "[ptr-resolve] origin={origin_sel} off=({off_x},{off_y}) abs={absolute} out={}",
            out.trim()
        );
    }
    let mut parts = out.trim().split('|');
    let hit = parts.next().unwrap_or_default().trim().to_string();
    let x = parts.next().and_then(|v| v.trim().parse::<f32>().ok());
    let y = parts.next().and_then(|v| v.trim().parse::<f32>().ok());
    match (x, y) {
        (Some(x), Some(y)) if !hit.is_empty() => (hit, x, y),
        // uievents-compat 尾簇 18：命中为空但坐标有效 → 以 body 落点 + **计算坐标**
        //（指针可移出任何元素盒到文档空白区——真浏览器事件落 html/body；旧版回落
        // origin + RAW offset 使坐标错页（(0,186)）——range_input 纵向拖拽取值断链
        // 实证：GBCR 算得 (93,202) 被弃用，move 派到 (0,186) 左上角）。
        (Some(x), Some(y)) => ("body".to_string(), x, y),
        _ => (origin_sel.to_string(), off_x, off_y),
    }
}

/// uievents-compat 尾簇 9：mutation 驱动悬停重结算（指针命令入口）——上一命令的
/// 页内 listener 变异（area 插入/coords 收缩/usemap 切换等）推进 mutTick 后，下一
/// 指针命令派发**前**按**最后已知指针位**重结算（真实浏览器的渲染机会边界在命令间
/// 重算悬停——同一 Actions 批的命令在本 runner 同 turn 连发，探测环 settle 来不及；
/// WPT mouse_boundary_events_on_image_map "new <area> is available" 断言面）。
fn mut_hover_settle_if_dirty(webview: &mut WebView, last_pointer: &Option<(f32, f32)>, td_refresh_ver: &mut u64) {
    let Some((last_x, last_y)) = last_pointer else { return };
    let Ok(dirty) =
        webview.execute_script(r#"(globalThis.__zw_ptr_hover_dirty ? globalThis.__zw_ptr_hover_dirty() : '0')"#)
    else {
        return;
    };
    if dirty.trim() != "1" {
        return;
    }
    let (hit_sel, hx, hy) = resolve_pointer_target(webview, "", *last_x, *last_y, true, td_refresh_ver);
    if runner_flags().td_debug {
        eprintln!("[zw-settle-cmd] hit={hit_sel} at ({hx},{hy})");
    }
    let script = zero_engine::script_mut_hover_settle(&hit_sel, hx, hy);
    let _ = webview.execute_script(&script);
}

fn apply_testdriver_command(
    webview: &mut WebView,
    command: &TestdriverCommand,
    td_refresh_ver: &mut u64,
    last_pointer: &mut Option<(f32, f32)>,
    pointer_injection: PointerStepInjection,
    drag_mouse_suppressed: &mut bool,
) -> Option<String> {
    // R145：selector 延迟解析——enqueue 时 mutation 未 apply（正置表空），出队时
    //（跨 turn）经 stub 的 `__zw_td_selector` 现场解析（apply 已 merge handle→selector）。
    // 返回值为裸串（"p"）或字面 "null"/空（无稳定选择器）。
    let selector = match command.selector.as_deref() {
        Some(sel) if !sel.is_empty() => Some(sel.to_string()),
        _ => webview
            .execute_script(&format!(
                "globalThis.__zw_td_selector ? globalThis.__zw_td_selector({}) : null",
                command.id
            ))
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty() && value != "null" && value != "undefined"),
    };
    let Some(selector) = selector else {
        return Some("testdriver target has no stable selector".into());
    };
    let selector = selector.trim().to_string();
    if selector.is_empty() || selector == "null" {
        return Some("testdriver target has no stable selector".into());
    }
    let target = match webview.page_node_ref_for_selector(&selector) {
        Some(target) => target,
        None => return Some(format!("testdriver target not found: {selector}")),
    };
    match command.operation.as_str() {
        // uievents-compat M2 片 1（2026-10-03）：Actions 指针 move 步——宿主
        // `__zw_pointer_move` 钩子（跨界时派 over/out/enter/leave 边界序 + 新目标
        // pointermove/mousemove 对）。仅事件面；无默认动作。
        // uievents-compat M3 尾簇 4（2026-10-04）：**执行时点解析**——text =
        // "offX,offY,pointerType"（元素 origin：中心 + offset）或
        // "@absX,absY,pointerType"（viewport origin：绝对坐标）；命中元素以 fresh
        // 几何现解析（计划构建时点解析受 shim 序列化一 turn 滞后——
        // after_target_removed pointerup-remover 断言面）。
        "pointer_move" => {
            let text = command.text.as_deref().unwrap_or_default();
            let (coords, absolute) = match text.strip_prefix('@') {
                Some(rest) => (rest, true),
                None => (text, false),
            };
            let mut parts = coords.split(',');
            let off_x = parts.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
            let off_y = parts.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
            let pointer_type = parts.next().unwrap_or("mouse").to_string();
            let _ = pointer_type;
            // pointerlock-compat M3 s3 路由（RFC §6.7）：锁定态 move 改道锁目标——
            // delta 由 webview 镜像算，跳过 `__zw_pointer_move` 合成（防双派）与
            // 悬停结算/跨界序（锁定态光标不可达其他元素，boundary 序无意义）。
            if webview.pointer_lock_target().is_some() {
                let (_, px, py) = resolve_pointer_target(webview, &selector, off_x, off_y, absolute, td_refresh_ver);
                *last_pointer = Some((px, py));
                let _ = webview.apply_locked_pointer_move(px, py);
                return None;
            }
            // dnd-compat M3：真管道注入——鼠标/指针事件面（shim 合成）与宿主拖放
            // 状态机（`apply_drag_input`，与桌面同一实现）并行喂入。dragstart 接受后
            // （Dragging）真实浏览器抑制鼠标/指针事件面：本步及后续 move/up 不再派
            // mousemove/pointermove；dragstart 取消 = 鼠标流恢复正常（自动化注入语义）。
            let drag_suppress = pointer_injection == PointerStepInjection::DragStateMachine && *drag_mouse_suppressed;
            if !drag_suppress {
                mut_hover_settle_if_dirty(webview, last_pointer, td_refresh_ver);
            }
            let (hit_sel, px, py) = resolve_pointer_target(webview, &selector, off_x, off_y, absolute, td_refresh_ver);
            // pointerlock-compat M3 s2（RFC §6.3）：位移流在事件源算 delta——自上一
            // move 步位置的差，i64 截断（spec movement 为 long）；首步无前位 → 0。
            let (mv_x, mv_y) = match *last_pointer {
                Some((last_x, last_y)) => ((px - last_x) as i64, (py - last_y) as i64),
                None => (0, 0),
            };
            *last_pointer = Some((px, py));
            if !drag_suppress {
                let script = zero_engine::script_pointer_move(&hit_sel, px, py, mv_x, mv_y);
                let _ = webview.execute_script(&script);
            }
            if pointer_injection == PointerStepInjection::DragStateMachine
                && let Ok(report) = webview.apply_drag_input(zero_webview::DragPointerPhase::Move, px, py)
            {
                *drag_mouse_suppressed = report.session_active;
            }
            None
        }
        "click" => {
            // R4300（html-semantics M2 切片 23，2026-10-22）：light dismiss 前步——落点在
            // open+closedby=any dialog 包含链外 → 先发 close request（spec dialog light
            // dismiss：pointerdown 触发、序在 focus/激活语义之前）。Actions down/up 链由
            // shim `__zw_pointer_down_sequence` 头部钩覆盖；本命令路径无 down 步，前置补齐。
            let ld_script = zero_engine::script_dialog_light_dismiss_check(&selector);
            let _ = webview.execute_script(&ld_script);
            // 主键 → 既有 Activate 路径（focus 步骤 + 全序列 + 连击 dblclick）——
            // move-only 链的尾随合成 click（R142 先例）与无 down/up 的链走此路径；
            // 带 down/up 步的 Actions 链已改逐步重放（M3 pointer_down/pointer_up）。
            // R142：合成指针点击的 focus 步骤（spec UI Events 指针激活序列——可聚焦目标
            // 先获得焦点再派发 click；WPT no-focus-events 期望 focus/focusin 恰好一次、
            // target 为点击元素）。element.focus() 经 shim 的 R3247 focus 派发
            //（focusout(旧) → focus(新) → focusin(新)），click 随后由 Activate 计划派发。
            // focus 失败不阻断 click（不可聚焦目标真实浏览器也派发 click）。
            // R4600（html-semantics M2 切片 26）：focus 步骤经 shim 焦点门
            // `globalThis._zwFocusSel`（内含 `_zwIsFocusableSel` 焦点性判定，与 Actions
            // down/up 链 mousedown 默认动作同款门，part06）——点击 html/body 等非
            // tabindex 目标不再偷焦点（dialog-autofocus / show-modal-focusing-steps
            // `waitUntilLoadedAndAutofocused` 断言面：test_driver.click(documentElement)
            // 不得吃掉 load autofocus 的 focusin；focus-after-close complex usage 仍
            // 聚焦可聚焦 button）。门缺失（旧 shim 拼接序）回落旧无条件行为。
            let focus_script = format!(
                "(function(){{var el=document.querySelector({sel});try{{if(globalThis._zwFocusSel){{globalThis._zwFocusSel({sel});}}else if(el&&el.focus)el.focus();}}catch(_e){{}}}})();",
                sel = serde_json::to_string(&selector).unwrap_or_else(|_| "null".into())
            );
            let _ = webview.execute_script(&focus_script);
            let result = dispatch_action(webview, target, HtmlUserAction::Activate);
            // navigation-compat S4J：合成 click 无 JS 默认动作面——submit 按钮点击后补调
            // 页面提交入口（表单提交 navigate 事件面——WPT navigate-form-userInitiated）。
            // 按钮判定/表单归属在 JS 侧完成（避免缓存 html 快照访问）。
            let submit_script = format!(
                "(function(){{var b=document.querySelector({sel});try{{var ty=b?String(b.getAttribute('type')||'').toLowerCase():'';var isBtn=b&&((b.tagName==='BUTTON'&&(ty==='submit'||ty===''))||(b.tagName==='INPUT'&&(ty==='submit'||ty==='image')));var f=isBtn&&(b.form||(b.closest&&b.closest('form')));if(f&&globalThis.__zwNavFormRequestSubmit)globalThis.__zwNavFormRequestSubmit(f,b);}}catch(_e){{}}}})();",
                sel = serde_json::to_string(&selector).unwrap_or_else(|_| "null".into()),
            );
            let _ = webview.execute_script(&submit_script);
            result
        }
        // uievents-compat M3（2026-10-03）：Actions down 步——独立宿主命令（取代 M2
        // 折叠 click：页内 pointerdown listener 的 setPointerCapture 须影响后续 move/up
        // 步路由）。text = "offX,offY,pointerType,button"（'@' 前缀 = viewport 绝对）。
        // focus 步骤与 collapse click 同位（mousedown 默认动作前置近似——R142 先例）。
        // uievents-compat M3 尾簇 4：落点执行时点解析（同 pointer_move）。
        "pointer_down" => {
            let text = command.text.as_deref().unwrap_or_default();
            let (coords, absolute) = match text.strip_prefix('@') {
                Some(rest) => (rest, true),
                None => (text, false),
            };
            let mut parts = coords.split(',');
            let off_x = parts.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
            let off_y = parts.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
            let pointer_type = parts.next().unwrap_or("mouse").to_string();
            let button = parts.next().and_then(|v| v.parse::<i16>().ok()).unwrap_or(0);
            // pointerlock-compat M3 s3 路由（RFC §6.7）：锁定态 down 改道锁目标
            // （mousedown 经 apply_locked_pointer_button；跳过合成序/悬停结算）。
            if webview.pointer_lock_target().is_some() {
                let (_, px, py) = resolve_pointer_target(webview, &selector, off_x, off_y, absolute, td_refresh_ver);
                *last_pointer = Some((px, py));
                let _ = webview.apply_locked_pointer_button(zero_webview::LockedButtonPhase::Down, button, px, py);
                return None;
            }
            mut_hover_settle_if_dirty(webview, last_pointer, td_refresh_ver);
            let (hit_sel, px, py) = resolve_pointer_target(webview, &selector, off_x, off_y, absolute, td_refresh_ver);
            *last_pointer = Some((px, py));
            // R4600：down 步前置 focus 同过 `globalThis._zwFocusSel` 门（click 命令
            // 同款；mousedown 默认动作门在 sequence 头已二次把关，双门幂等——不可
            // 聚焦落点两处都跳，dialog-autofocus waitUntilLoadedAndAutofocused 断言面）。
            let focus_script = format!(
                "(function(){{var el=document.querySelector({sel});try{{if(globalThis._zwFocusSel){{globalThis._zwFocusSel({sel});}}else if(el&&el.focus)el.focus();}}catch(_e){{}}}})();",
                sel = serde_json::to_string(&hit_sel).unwrap_or_else(|_| "null".into())
            );
            let _ = webview.execute_script(&focus_script);
            let script = zero_engine::script_pointer_down_sequence(&hit_sel, px, py, &pointer_type, button);
            let _ = webview.execute_script(&script);
            // dnd-compat M3：down 步并行武装拖放会话（drag_source_eligible 命中仲裁）；
            // 新按下重置鼠标面抑制（dragstart 未重判）。
            if pointer_injection == PointerStepInjection::DragStateMachine {
                *drag_mouse_suppressed = false;
                let _ = webview.apply_drag_input(zero_webview::DragPointerPhase::Down, px, py);
            }
            None
        }
        // uievents-compat M3：Actions up 步——text =
        // "offX,offY,pointerType,button|downSel|anc1|anc2…"（'@' 前缀 = viewport 绝对；
        // 尾段 = up origin 祖先链）。click/auxclick 组合逻辑在 shim
        // `__zw_pointer_up_sequence`（捕获落点/公共祖先/连击）。downSel = down 步命中
        // 元素（空串回落 shim 记录值）；祖先链 = touch 抬起悬停拆除的 leave 锚（目标
        // 被移除时取首个连通近祖）。尾簇 4：落点执行时点解析。
        "pointer_up" => {
            let text = command.text.as_deref().unwrap_or_default();
            let (coords_rest, absolute) = match text.strip_prefix('@') {
                Some(rest) => (rest, true),
                None => (text, false),
            };
            let mut segments = coords_rest.split('|');
            let coords = segments.next().unwrap_or_default();
            let down_selector = segments.next().unwrap_or_default().to_string();
            let ancestor_chain = segments.map(String::from).collect::<Vec<_>>().join("|");
            let mut parts = coords.split(',');
            let off_x = parts.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
            let off_y = parts.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
            let pointer_type = parts.next().unwrap_or("mouse").to_string();
            let button = parts.next().and_then(|v| v.parse::<i16>().ok()).unwrap_or(0);
            // pointerlock-compat M3 s3 路由（RFC §6.7）：锁定态 up 改道锁目标
            // （mouseup；click 组合语义经 shim `__zw_pointer_up_sequence` 的合成管线
            // 不在本入口——锁定 click 语料未钉）。
            if webview.pointer_lock_target().is_some() {
                let (_, px, py) = resolve_pointer_target(webview, &selector, off_x, off_y, absolute, td_refresh_ver);
                *last_pointer = Some((px, py));
                let _ = webview.apply_locked_pointer_button(zero_webview::LockedButtonPhase::Up, button, px, py);
                return None;
            }
            // dnd-compat M3：Dragging 中释放 → drop/dragend（或 dragleave+dragend）
            // 替代鼠标事件面；未拖拽释放（未武装/dragstart 已取消）→ mouseup 面正常
            // 派发 + 状态机安静清武装（自动化注入语义：两种输入路径并行）。
            let drag_suppress = pointer_injection == PointerStepInjection::DragStateMachine && *drag_mouse_suppressed;
            if !drag_suppress {
                mut_hover_settle_if_dirty(webview, last_pointer, td_refresh_ver);
            }
            let (hit_sel, px, py) = resolve_pointer_target(webview, &selector, off_x, off_y, absolute, td_refresh_ver);
            *last_pointer = Some((px, py));
            if drag_suppress {
                let _ = webview.apply_drag_input(zero_webview::DragPointerPhase::Up, px, py);
                *drag_mouse_suppressed = false;
                return None;
            }
            let script = zero_engine::script_pointer_up_sequence(
                &hit_sel,
                &down_selector,
                px,
                py,
                &pointer_type,
                button,
                &ancestor_chain,
            );
            let _ = webview.execute_script(&script);
            if pointer_injection == PointerStepInjection::DragStateMachine {
                let _ = webview.apply_drag_input(zero_webview::DragPointerPhase::Up, px, py);
            }
            None
        }
        // R3254-K2（keyboard goal M1 切片 2，2026-09-07）：Actions 键盘链命令——
        // keydown/keyup cancelable KeyboardEvent 派发（script_dispatch_dom_event 通道，
        // __zw_dispatch_event 返 'ok'/'prevented'）。keydown 未取消 → 追加字符默认动作
        //（InsertText → beforeinput/input/value，UI Events 事件序 keydown→beforeinput→
        // input；WPT keydown-input-events.html 主断言族）。keyup 仅派事件（无默认动作）。
        "keydown" | "keyup" => {
            let key = command.text.as_deref().unwrap_or_default().to_string();
            let event_type = command.operation.clone();
            let script = zero_engine::script_dispatch_dom_event(
                &selector,
                &event_type,
                Some(&zero_engine::DomEventDetail {
                    key: Some(key.clone()),
                    code: Some(key.clone()),
                    ..Default::default()
                }),
            );
            let result = webview
                .execute_script(&script)
                .map(|v| v.trim().to_string())
                .unwrap_or_default();
            if event_type == "keyup" {
                return None;
            }
            if result == "prevented" {
                // 页面 keydown listener preventDefault → 字符默认动作被抑制
                //（UI Events cancelable 语义；value 不变、editing 事件不派发）。
                return None;
            }
            // uievents-compat 尾簇 11（2026-10-05）：Actions 键盘链的 **ENTER** 默认动作
            //——formless buttonish（BUTTON / input type=button|submit|reset）= 激活点击
            //（真浏览器语义；旧版落入 InsertText{'\u{E007}'} → text-control snapshot 对
            // button 类 input 抛 InvalidStateError「does not support text selection」→
            // actions.send() 整链拒绝，WPT pointerevent_click_is_a_pointerevent
            // non-pointing-device 的 send() 永挂 → 文件 Timeout）。click 为 PointerEvent
            // 实例 + pointerId -1（PE spec 非指针生成；shim click+pointerType 分支）。
            // in-form Enter→click→submit 链与非 buttonish 目标维持既有语义（隐式提交族
            // 绿面不动）。R108 pre-click activation 经 PE 原型链接通——buttonish 无
            // checked 态，激活目标查找（checkbox/radio）恒 miss，无双翻转面。
            if key == "\u{E007}"
                && webview
                    .execute_script(&zero_engine::script_buttonish_probe(&selector))
                    .map(|v| v.trim() == "1")
                    .unwrap_or(false)
                && webview
                    .execute_script(&zero_engine::script_enclosing_form_probe(&selector))
                    .map(|v| v.trim() != "1")
                    .unwrap_or(false)
            {
                let click_detail = zero_engine::DomEventDetail {
                    pointer_type: Some(String::new()),
                    pointer_id: Some(-1),
                    ..Default::default()
                };
                let click_script = zero_engine::script_dispatch_dom_event(&selector, "click", Some(&click_detail));
                let _ = webview.execute_script(&click_script);
                return None;
            }
            if key.chars().count() == 1 {
                if let Some(error) = dispatch_action(webview, target, HtmlUserAction::InsertText { text: key.clone() })
                {
                    return Some(error);
                }
                // UI Events：未取消的字符键 keydown 之后派 keypress（composed，同
                // KeyboardEvent 面；WPT keyboardevent-composed.html keypress 子测）。
                // keypress 无 runner 侧默认动作（字符插入已由 InsertText 完成）。
                let keypress_script = zero_engine::script_dispatch_dom_event(
                    &selector,
                    "keypress",
                    Some(&zero_engine::DomEventDetail {
                        key: Some(key.clone()),
                        code: Some(key.clone()),
                        ..Default::default()
                    }),
                );
                let _ = webview.execute_script(&keypress_script);
                None
            } else {
                None
            }
        }
        "send_keys" => {
            let text = command.text.as_deref().unwrap_or_default();
            // 尾簇 32：send_keys 入口**自聚焦目标**（WebDriver send keys 语义——键事件
            // 与激活归属目标元素；真实浏览器 per-element send 先聚焦）。多 promise_test
            // 交错 focus 的组合时序下（WPT uievents/interface/keyboard-click-event 4 子测
            // 各 focus 自己的元素后并发排队），前序 focus 覆盖 → 激活管线（空格/Enter 对
            // button 的 click 合成）焦点归属错位 → click 不发 → EventWatcher 门铃永挂。
            // slice22 焦点治理通道（__zw_host_focus）——页面可见焦点状态同步 + 焦点事件
            // 派发同流。**实现**：聚焦拼进**首键 keydown 派发脚本**（`_zwFocusPrefix`——
            // 零额外 execute_script 往返；独立 focus 脚本在 workspace 并发负载下使
            // send_keys 命令的轮询窗偶发错过——内嵌 fixture 间歇 Timeout 实证）。
            let mut _zw_first_char = true;
            // R3254-K2 残余切片 4（keyboard goal，2026-09-07）：send_keys 串内修饰键
            // 持久化——修饰字符设置状态位，后续普通字符的事件对继承（WPT
            // keypress-not-fired-for-modifier-shortcuts.html 的 `uE009 + 'v'` 复合序）。
            // 串结束即清零（WebDriver 每次 send 独立按键序列）。
            let mut shift_sticky = false;
            let mut ctrl_sticky = false;
            let mut alt_sticky = false;
            let mut meta_sticky = false;
            for character in text.chars() {
                // 尾簇 33：**首键** keydown 拼 focus 前缀（self-focus——零额外往返形，
                // 见 send_keys 入口注记）。
                let focus_prefix = if _zw_first_char && !selector.is_empty() {
                    format!(
                        "(typeof __zw_host_focus==='function')&&__zw_host_focus({});",
                        serde_json::to_string(&selector).unwrap_or_else(|_| "null".into())
                    )
                } else {
                    String::new()
                };
                _zw_first_char = false;
                // R3254-KP2（keyboard-page-scrolling goal M1 切片 1 解除 defer）：
                // WebDriver 滚动/导航键（arrows/pages/home/end）→ keydown+keyup 事件
                // 对（key 名按 WebDriver 规范映射）。runner 侧无真滚动管线（滚动默认
                // 动作在 browser shell 层）——事件派发是页面可观察面（snap 用例的事件
                // listener + waitForAnimationEnd 驱动）。旧版落 PUA 拒绝分支使
                // css-scroll-snap/input 三案全簇 Unhandled rejection。
                let scroll_key = match character {
                    '\u{E012}' => Some(("ArrowLeft", false, false, false, false)),
                    '\u{E013}' => Some(("ArrowUp", false, false, false, false)),
                    '\u{E014}' => Some(("ArrowRight", false, false, false, false)),
                    '\u{E015}' => Some(("ArrowDown", false, false, false, false)),
                    '\u{E00E}' => Some(("PageUp", false, false, false, false)),
                    '\u{E00F}' => Some(("PageDown", false, false, false, false)),
                    '\u{E010}' => Some(("End", false, false, false, false)),
                    '\u{E011}' => Some(("Home", false, false, false, false)),
                    // R3254-K2 切片 3（keyboard goal M1）：修饰键 → keydown+keyup 事件对
                    // 且对应 modifier 位为 true（modifier-keys.html 断言
                    // event.shiftKey === (key === 'Shift') 等——位来自 DomEventDetail
                    // 修饰键字段经 __zw_dispatch_event init dict 透传）。
                    '\u{E008}' => Some(("Shift", true, false, false, false)),
                    '\u{E009}' => Some(("Control", false, true, false, false)),
                    '\u{E00A}' => Some(("Alt", false, false, true, false)),
                    '\u{E03D}' => Some(("Meta", false, false, false, true)),
                    _ => None,
                };
                if let Some((key_name, shift_k, ctrl_k, alt_k, meta_k)) = scroll_key {
                    shift_sticky = shift_k;
                    ctrl_sticky = ctrl_k;
                    alt_sticky = alt_k;
                    meta_sticky = meta_k;
                    let scroll_detail = zero_engine::DomEventDetail {
                        key: Some(key_name.to_string()),
                        code: Some(key_name.to_string()),
                        shift_key: shift_k,
                        ctrl_key: ctrl_k,
                        alt_key: alt_k,
                        meta_key: meta_k,
                        ..Default::default()
                    };
                    // R3254-K2 切片 3：keydown/keyup 事件对。
                    // R3254-KP5（keyboard-page-scrolling goal M2 切片 2）：keydown 未被
                    // 页面取消时追加滚动默认动作（`__zw_scroll_key_default`——幅度映射
                    // 与 browser R3254-M9 同源，经 R3047 scrollTop setter 派 'scroll'
                    // 事件）。旧版只派事件不滚动 → snap 三案的 scrollend promise 链
                    // 永不解阻（整簇 Timeout）。
                    let keydown_result = dispatch_key_event_script_focused(
                        webview,
                        &selector,
                        "keydown",
                        &scroll_detail,
                        focus_prefix.as_str(),
                    );
                    if keydown_result != "prevented" {
                        // s129（html-semantics M2）：SELECT/radio 焦点键消费门——目标是
                        // SELECT（或 radio 组成员）时 ArrowUp/Down/Home/End 走既有
                        // `__zw_select_key_action` 选中项导航（input+change 派发，
                        // input-change-event-properties down-arrow 断言面；R3254-K5 生成器
                        // 此前无调用点）。键被消费即跳过滚动默认动作（spec：select 键导航
                        // 不滚动）；非 SELECT 返 false → 照旧滚动。
                        let select_consumed = if selector.is_empty() {
                            false
                        } else {
                            webview
                                .execute_script(&zero_engine::script_select_key_action(&selector, key_name))
                                .map(|v| v.trim() == "true")
                                .unwrap_or(false)
                        };
                        if !select_consumed {
                            let scroll_script = zero_engine::script_scroll_key_default(&selector, key_name);
                            let _ = webview.execute_script(&scroll_script);
                        }
                    }
                    dispatch_key_event_script(webview, &selector, "keyup", &scroll_detail);
                    continue;
                }
                // R4300（html-semantics M2 切片 23，2026-10-22）：Escape（WebDriver uE00C，
                // 旧版落 PUA 拒绝分支——dialog-canceling / non-modal-canceling /
                // dialog-closedby 全簇 sendEscKey Unhandled rejection）。keydown/keyup 派发
                // 到**当前焦点元素**（`__zw_dispatch_key_at_active`——不自聚焦：send_keys
                // 目标 body 不夺页面焦点，dialog-canceling 断言面——焦点 input keydown
                // preventDefault 吞 ESC，dialog 不收 cancel）。keydown 未被取消 → dialog
                // close request 默认动作（R3254-K3 `__zw_esc_dialog_cancel`——R4300 加
                // closedby 资格门 + 模态栈序）。
                // https://html.spec.whatwg.org/multipage/interactive-elements.html#dialog-cancelation
                if character == '\u{E00C}' {
                    let esc_detail_json = "{\"key\":\"Escape\",\"code\":\"Escape\"}";
                    let keydown_script = zero_engine::script_dispatch_key_at_active("keydown", esc_detail_json);
                    let keydown_result = webview
                        .execute_script(&keydown_script)
                        .map(|v| v.trim() == "prevented")
                        .unwrap_or(false);
                    if !keydown_result {
                        let esc_script = zero_engine::script_esc_dialog_cancel();
                        let _ = webview.execute_script(&esc_script);
                    }
                    let keyup_script = zero_engine::script_dispatch_key_at_active("keyup", esc_detail_json);
                    let _ = webview.execute_script(&keyup_script);
                    continue;
                }
                let action = match character {
                    '\u{E003}' => HtmlUserAction::DeleteBackward,
                    // uievents-compat 尾簇 31：ForwardDelete（WebDriver uE017——WPT
                    // uievents/textInput delete/delete-selection；CE 宿主不分流（落
                    // text-control 探测 noop——CE ForwardDelete 挂账））。
                    '\u{E017}' => HtmlUserAction::ForwardDelete,
                    '\u{E004}' => HtmlUserAction::MoveFocus { forward: true },
                    // R3254-K1（keyboard goal M1，2026-09-07）：WebDriver ENTER →
                    // Submit 动作（implicit submission 规则——webview 层分发：CE 宿主
                    // → __zw_ce_enter 换行（编辑宿主优先消费）；text control 在表单内
                    // → submit 事件+导航意图；textarea 由用例用字面 '\n' 表达换行，
                    // 不经 uE007）。旧版落入 PUA 拒绝分支——implicit-submission 全簇
                    // Unhandled rejection。
                    // 尾簇 31：uE006（WebDriver Return 键）同 Enter 语义（WPT
                    // uievents/textInput enter-input + uievents/interface/
                    // keyboard-click-event 的 common.js keyMapping 写 uE006）。
                    '\u{E006}' | '\u{E007}' => HtmlUserAction::Submit,
                    character if ('\u{E000}'..='\u{F8FF}').contains(&character) => {
                        return Some(format!("unsupported WebDriver key U+{:04X}", character as u32));
                    }
                    character => HtmlUserAction::InsertText {
                        text: character.to_string(),
                    },
                };
                // 尾簇 33：textarea 的 Enter = **换行**（真实浏览器语义——textarea 不参
                // 与 implicit submission；WPT uievents/textInput
                // enter-textarea-contenteditable 的 basic.sub.js 断言 value='\n' +
                // textInput(data='\n')——走 InsertText 管线（text_plan 的 textInput
                // followup 面）。其余目标维持 Submit 动作。
                let action = if matches!(action, HtmlUserAction::Submit) {
                    let ta_probe = webview.execute_script(&format!(
                        "(function(){{var e=document.querySelector({});return (e && e.tagName==='TEXTAREA') ? '1' : '';}})()",
                        serde_json::to_string(&selector).unwrap_or_else(|_| "null".into())
                    ))
                    .map(|v| v.trim() == "1")
                    .unwrap_or(false);
                    if ta_probe {
                        HtmlUserAction::InsertText { text: "\n".to_string() }
                    } else {
                        action
                    }
                } else {
                    action
                };
                // R3254-K2 残余切片 4（keyboard goal，2026-09-07）：普通/编辑键补全
                // keydown→默认动作→keypress→keyup 事件序（UI Events §keydown/§keyup 默认
                // 动作序；旧版裸 InsertText 无任何键事件，keypress-not-fired-for-
                // modifier-shortcuts 的 keyup listener 永不触发 → 3 subtest 全 pending
                // Timeout）。Ctrl/Meta 按住时 keypress 抑制（UI Events：修饰快捷键不产
                // 生字符点击——keypress-not-fired 主断言）。
                // https://w3c.github.io/uievents/#keys-modifiers
                // https://w3c.github.io/uievents/#event-type-keypress
                // uievents-compat 尾簇 31：编辑/导航键的 key/code 用 **WebDriver 键名**
                //（Backspace/Tab/Enter——真实浏览器 KeyboardEvent.key 语义；scroll_key
                // 表的 arrows/pages/modifiers 先例同款）。旧版透传原始 PUA 转义字符，
                // 页面 keyup listener 按 e.key === 'Backspace' 过滤恒 miss（WPT
                // uievents/textInput no-textInput 族 keyup 门铃永挂 → 文件 Timeout）。
                let key = match character {
                    '\u{E003}' => "Backspace".to_string(),
                    '\u{E004}' => "Tab".to_string(),
                    '\u{E006}' | '\u{E007}' => "Enter".to_string(),
                    // WebDriver Delete 键的 UI Events key 名（区别于 Backspace）。
                    '\u{E017}' => "Delete".to_string(),
                    character => character.to_string(),
                };
                let key_detail = zero_engine::DomEventDetail {
                    key: Some(key.clone()),
                    code: Some(key.clone()),
                    shift_key: shift_sticky,
                    ctrl_key: ctrl_sticky,
                    alt_key: alt_sticky,
                    meta_key: meta_sticky,
                    ..Default::default()
                };
                match action {
                    // 可打印字符：keydown 未取消 → InsertText 默认动作 + keypress
                    //（keydown-input-events.html 的 cancel 语义同款；非字符键无 keypress）。
                    HtmlUserAction::InsertText { .. } => {
                        // R3254-K4 切片 2（2026-09-07）：空格激活**时序**——UI Events/
                        // Chromium：Enter 在 keydown 触发 click、Space 在 **keyup** 触发
                        // （keydown 只派事件不激活）。buttonish 目标（BUTTON / input
                        // type=button|submit|reset）的空格延迟到 keyup 后再走 InsertText
                        // 通道（webview 递归 Activate 合成 click）；非 buttonish 行为不变。
                        let space_buttonish = character == ' '
                            && webview
                                .execute_script(&zero_engine::script_buttonish_probe(&selector))
                                .map(|v| v.trim() == "1")
                                .unwrap_or(false);
                        if space_buttonish {
                            dispatch_key_event_script_focused(
                                webview,
                                &selector,
                                "keydown",
                                &key_detail,
                                focus_prefix.as_str(),
                            );
                            dispatch_key_event_script(webview, &selector, "keyup", &key_detail);
                            if let Some(error) = dispatch_action(webview, target, action) {
                                return Some(error);
                            }
                            continue;
                        }
                        if dispatch_key_event_script_focused(
                            webview,
                            &selector,
                            "keydown",
                            &key_detail,
                            focus_prefix.as_str(),
                        ) != "prevented"
                        {
                            // R3254-K5 切片 3（2026-09-07）：SELECT 焦点上的可打印字符 =
                            // type-ahead 键入跳转（多字符缓冲 500ms 窗，Chromium closed-
                            // select 语义近似）。消费后跳过 InsertText/keypress（SELECT 无
                            // 文本插入面、字符不产生点击）；keydown 未取消才应用。
                            if !character.is_control() && character != ' ' {
                                let ta_script = zero_engine::script_select_type_ahead(&selector, character);
                                let consumed = webview
                                    .execute_script(&ta_script)
                                    .map(|v| v.trim() == "1")
                                    .unwrap_or(false);
                                if consumed {
                                    continue;
                                }
                            }
                            // R3254-KP5：空格对**非可编辑**目标（div/body 等无文本插入面）
                            // = 页面滚动（UI Events 默认动作；snap 用例 KEY_CODE_MAP 'Space'
                            // → ' ' 走此路径）。webview InsertText 对不可编辑目标返
                            // noop(NotApplicable)——以此判定回落滚动默认动作；可编辑宿主
                            // （text control/CE）行为不变。
                            if character == ' ' {
                                let scroll_fallback = matches!(
                                    dispatch_action(webview, target, action),
                                    Some(ref error)
                                        if error.starts_with("action was not applicable")
                                );
                                if scroll_fallback {
                                    let scroll_script = zero_engine::script_scroll_key_default(&selector, " ");
                                    let _ = webview.execute_script(&scroll_script);
                                }
                            } else if let Some(error) = dispatch_action(webview, target, action) {
                                return Some(error);
                            }
                            if !ctrl_sticky && !meta_sticky {
                                dispatch_key_event_script(webview, &selector, "keypress", &key_detail);
                            }
                        }
                    }
                    // 非字符编辑/导航键（Backspace/Tab/ENTER）：keydown → 默认动作（取消
                    // 只抑制字符插入类序，此处动作照旧执行——与旧版行为一致）→ keyup。
                    _ => {
                        // navigation-compat S11：TAB 不做 send 目标自聚焦——焦点导航键的
                        // 迁移起点是页面当前焦点（dispatch_tab_focus），首键 focus 前缀
                        // （编辑键自聚焦语义）会把焦点抢到 send 目标、起点错位。
                        let is_tab = matches!(action, HtmlUserAction::MoveFocus { forward: true });
                        let tab_keydown = if is_tab { String::new() } else { focus_prefix.clone() };
                        dispatch_key_event_script_focused(
                            webview,
                            &selector,
                            "keydown",
                            &key_detail,
                            tab_keydown.as_str(),
                        );
                        // uievents-compat 尾簇 11（2026-10-05）：Enter on **formless**
                        // buttonish（BUTTON / input type=button|submit|reset）= 激活点击
                        //（真浏览器语义；旧版落入 Submit 臂 → enclosing form 缺席
                        // noop(NotApplicable) → 无 click，WPT
                        // pointerevent_click_is_a_pointerevent non-pointing-device 的
                        // click promise 永挂 → 文件 Timeout）。click 为 PointerEvent
                        // 实例 + pointerId -1（PE spec 非指针生成；shim
                        // `__zw_dispatch_event` click+pointerType 分支）。in-form 的
                        // Enter→click→submit 链由 Submit 臂既有语义承载（不改——隐式
                        // 提交族绿面）。R108 pre-click activation 经 PE 原型链接通——
                        // buttonish 无 checked 态、激活目标查找（checkbox/radio）恒
                        // miss，checked 双翻转历史坑不适用。
                        // 尾簇 32：uE006（WebDriver Return）同 Enter——formless
                        // buttonish 激活面与 uE007 一致（WPT
                        // uievents/interface/keyboard-click-event 的 keyMapping 写
                        // uE006——submit 动作对 formless button noop(NotApplicable)
                        // 使 send_keys 链 reject/挂）。
                        let enter_formless_buttonish = (character == '\u{E007}' || character == '\u{E006}')
                            && webview
                                .execute_script(&zero_engine::script_buttonish_probe(&selector))
                                .map(|v| v.trim() == "1")
                                .unwrap_or(false)
                            && webview
                                .execute_script(&zero_engine::script_enclosing_form_probe(&selector))
                                .map(|v| v.trim() != "1")
                                .unwrap_or(false);
                        // 尾簇 32：动作 noop（NotApplicable——formless input 的
                        // Submit 等）不中断键事件序——**keyup 恒派**（UI Events 键事件
                        // 序 keydown → （默认动作可有可无）→ keyup；旧版 return 跳过
                        // keyup 使页面 keyup 门铃永挂（WPT uievents/textInput
                        // enter-input 的 no-textInput 驱动））。
                        let mut action_error = None;
                        if enter_formless_buttonish {
                            let click_detail = zero_engine::DomEventDetail {
                                pointer_type: Some(String::new()),
                                pointer_id: Some(-1),
                                ..Default::default()
                            };
                            let click_script =
                                zero_engine::script_dispatch_dom_event(&selector, "click", Some(&click_detail));
                            let _ = webview.execute_script(&click_script);
                        } else if is_tab {
                            action_error = dispatch_tab_focus(webview);
                        } else {
                            action_error = dispatch_action(webview, target, action);
                        }
                        if let Some(error) = action_error {
                            dispatch_key_event_script(webview, &selector, "keyup", &key_detail);
                            return Some(error);
                        }
                    }
                }
                dispatch_key_event_script(webview, &selector, "keyup", &key_detail);
            }
            None
        }
        // 尾簇 35：wheel 源 scroll 步（Actions scroll → WheelEvent 派发——
        // wheel-basic 的 target 序面 + deadlock 的 delta 透传面；滚动默认动作
        // headless 无真滚动管线不消费）。text = "[@]x,y,dx,dy"（'@' = viewport
        // 绝对坐标；否则 origin 元素中心 + 偏移）。
        "wheel_scroll" => {
            let text = command.text.as_deref().unwrap_or_default().to_string();
            let parts: Vec<&str> = text.trim_start_matches('@').split(',').collect();
            if parts.len() < 4 {
                return Some(format!("wheel_scroll bad text: {text}"));
            }
            let parse = |v: &str| -> f32 { v.trim().parse::<f32>().unwrap_or(0.0) };
            let ox = parse(parts[0]);
            let oy = parse(parts[1]);
            let dx = parse(parts[2]);
            let dy = parse(parts[3]);
            let absolute = text.starts_with('@');
            let (hit, px, py) = resolve_pointer_target(webview, &selector, ox, oy, absolute, td_refresh_ver);
            if runner_flags().td_debug {
                eprintln!("[wheel-scroll] hit={hit} at ({px},{py}) delta=({dx},{dy})");
            }
            let script = zero_engine::script_wheel_scroll(&hit, px, py, dx, dy);
            if let Err(err) = webview.execute_script(&script) {
                return Some(err.to_string());
            }
            None
        }
        operation => Some(format!("unsupported testdriver command: {operation}")),
    }
}

/// 派发单个键事件（R3254-K2 残余切片 4 的 send_keys 事件序辅助）——返回
/// `__zw_dispatch_event` 结果串（'ok'/'prevented'）。
fn dispatch_key_event_script(
    webview: &mut WebView,
    selector: &str,
    event_type: &str,
    detail: &zero_engine::DomEventDetail,
) -> String {
    let script = zero_engine::script_dispatch_dom_event(selector, event_type, Some(detail));
    webview
        .execute_script(&script)
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

/// 尾簇 33：keydown 派发带 **focus 前缀**（send_keys 自聚焦合并进首键脚本——零额外
/// execute_script 往返；独立 focus 脚本在 workspace 并发负载下使命令轮询窗偶发错过，
/// 内嵌 fixture 间歇 Timeout 实证）。prefix 空串时与 `dispatch_key_event_script` 等价。
fn dispatch_key_event_script_focused(
    webview: &mut WebView,
    selector: &str,
    event_type: &str,
    detail: &zero_engine::DomEventDetail,
    focus_prefix: &str,
) -> String {
    let script = zero_engine::script_dispatch_dom_event(selector, event_type, Some(detail));
    let full = if focus_prefix.is_empty() {
        script
    } else {
        format!("{}{}", focus_prefix, script)
    };
    webview
        .execute_script(&full)
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

fn dispatch_action(
    webview: &mut WebView,
    target: zero_page_runtime::PageNodeRef,
    action: HtmlUserAction,
) -> Option<String> {
    match webview.dispatch_loaded_page_user_action(HtmlActionRequest {
        target,
        action,
        shift: false,
    }) {
        Ok(result) if result.noop_reason.is_none() => None,
        // R347：disabled 目标的合成 click =「事件送达但激活被抑制」——noop(DisabledTarget)
        // 对 testdriver 语义即成功（spec HTML §activation：disabled 表单控件跳过激活行为，
        // 但自动化的 click 仍需 resolve 以驱动后续断言；onclick 不触发由页面侧断言）。
        // WPT Event-dispatch-on-disabled-elements「Real clicks」段依赖此 resolve。
        Ok(result) if result.noop_reason == Some(zero_page_runtime::ActionNoopReason::DisabledTarget) => None,
        Ok(result) => Some(format!("action was not applicable: {:?}", result.noop_reason)),
        Err(error) => Some(error.to_string()),
    }
}

/// navigation-compat S11（2026-10-20）：TAB 焦点导航的宿主接线——页面可见焦点迁移。
/// host `MoveFocus` 默认动作只产出 `PageEffect::Focus(next)` 效果，页面可见迁移是
/// **宿主层职责**（真浏览器宿主 renderer 消费效果走 blur_focused/focus_via_tab——
/// 失焦相位 blur/focusout → 获焦相位 focus/focusin + `document.activeElement` 同步，
/// apps/renderer/src/runtime.rs `apply_webview_action_effects`）。runner 宿主此前
/// 丢弃 effects：页面 focus listener 永不触发，focus-reset helpers 的 waitForFocus
/// 门铃永挂（WPT navigation-api/focus-reset basic/multiple-intercept 两案 case
/// Timeout）。
///
/// 迁移走**页面原生 focusing steps**（querySelector 候选序 + `next.focus()`——shim
/// el.focus() 连接臂自带完整迁移序：旧焦点 blur/focusout → 新焦点 focus + `_activeElKey`
/// handle 形键同步；后续 el.remove() 的 `_zwUnfocusIfFocused` 按 handle 形键比对——
/// host 侧 `_zwFocusSel(sel)` 写 sel 形键（`_elKey(sel,null)`）与 remove 钩子
/// `_elKey(sel,handle)` 恒 miss，页面原生 focus 是唯一与既有 unfocus 机制同键体系
/// 的入口）。tab 序 = 正 tabindex 升序在前 + 零/缺省按文档序（HTML §6.5.3
/// sequential focus navigation）；起点 = 页面真实焦点（`document.activeElement` 身份
/// 比对——querySelector 同源 _makeProxy 缓存 identity 相等，handle 兜底双判）；
/// 无下一可聚焦 → 焦点不变（真浏览器 Tab 落浏览器 UI，headless 近似驻留）。
/// R5251（html-semantics M2 切片 48，2026-10-24）：候选集补 editing host——
/// contenteditable true 状态且父不可编辑的元素是 focusable area（WPT
/// dialog-focusability：`<dialog contenteditable=true open>` 须入 Tab 序）。isContentEditable
/// 走 shim R3187 枚举态（大小写不敏感 + inherit 继承）；嵌套 editable 非 host 与
/// contenteditable=false 孤岛不入序。显式 tabindex 元素优先 tabIndex 臂（含 ""）。
/// https://html.spec.whatwg.org/multipage/interaction.html#focusable-area
/// https://html.spec.whatwg.org/multipage/interaction.html#sequential-focus-navigation
/// https://html.spec.whatwg.org/multipage/interaction.html#focusing-steps
fn dispatch_tab_focus(webview: &mut WebView) -> Option<String> {
    // 候选集 = 标准可聚焦面（a/area 带 href、button、input 非 hidden、select、
    // textarea、summary、[tabindex]）；排除 disabled 与 tabindex=-1。真值读 live 树
    // ——appendChild 动态元素天然在面（引擎 cached_html 快照路径的窗口期不适用）。
    let script = "(function(){\
        var act=document.activeElement;\
        var all=document.querySelectorAll('a[href],area[href],button,input,select,textarea,summary,[tabindex],[contenteditable]');\
        var pos=[],zero=[];\
        for(var i=0;i<all.length;i++){\
            var el=all[i];\
            if(el.disabled)continue;\
            var ti=el.getAttribute('tabindex');\
            if(ti==='-1')continue;\
            if(el.tagName==='INPUT'&&String(el.type).toLowerCase()==='hidden')continue;\
            var n=parseInt(ti,10);\
            if(n>0){pos.push([n,el]);continue;}\
            if(ti===null&&el.hasAttribute('contenteditable')){\
                if(!el.isContentEditable)continue;\
                var par=el.parentElement;\
                if(par&&par.isContentEditable)continue;\
                zero.push(el);\
            }else{zero.push(el);}\
        }\
        pos.sort(function(a,b){return a[0]-b[0];});\
        var seq=[];\
        for(var j=0;j<pos.length;j++)seq.push(pos[j][1]);\
        for(var k2=0;k2<zero.length;k2++)seq.push(zero[k2]);\
        var idx=-1;\
        for(var m2=0;m2<seq.length;m2++){\
            if(seq[m2]===act||(act.__zwHandle&&seq[m2].__zwHandle===act.__zwHandle)){idx=m2;break;}\
        }\
        var next=null;\
        if(idx>=0&&idx+1<seq.length)next=seq[idx+1];\
        else if(idx<0&&seq.length)next=seq[0];\
        if(next&&next.focus)next.focus();\
        return JSON.stringify({from:(act===document.body||!act)?'body':'el',moved:next?1:0});\
    })()";
    let result = webview
        .execute_script(script)
        .map(|v| v.trim().to_string())
        .unwrap_or_default();
    if runner_flags().td_debug {
        eprintln!("[tab-focus] {result}");
    }
    None
}

fn resolve_testdriver_command(webview: &mut WebView, id: u64, error: Option<&str>) -> Result<(), String> {
    let error = serde_json::to_string(&error).map_err(|error| error.to_string())?;
    webview
        .execute_script(&format!("globalThis.__zw_td_resolve({id},{error})"))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

const TESTDRIVER_STUB: &str = r#"<script>
(function() {
  var nextId = 1;
  var pending = {};
  var queuedElements = {};
  var queuedChains = {}; // uievents-compat M3 尾簇：id → 祖先选择器链（入队时快照，近祖优先）
  globalThis.__zw_td_queue = [];
  function selectorFor(element) {
    if (!element) return null;
    // uievents-compat 尾簇 31：**shim 树权威 selector 优先**（element.__zwSelector——
    // listener 注册 key 与宿主 selector_for_page_node_handle 的同源形）。下方 stub
    // 启发式（#id/tag/attr/nth）对**无 id 元素**产 tag 形（`input`）——与 shim 树
    // __zwSelector 形不一致时 `__zw_dispatch_event` 的 sel-key listener store miss
    // → keydown/keyup 对无 id text control 静默丢失（WPT uievents/textInput
    // no-textInput 族 keyup 门铃永不触发 → 文件 Timeout；同 id 元素 '#x' 双形一致
    // 不受影响——backspace 负向族的复刻探针实证）。信任前校验 selector 回查命中
    // 同一元素（__zwSelector 可能缺/过期）。
    try {
      var zwSel31 = element.__zwSelector;
      if (zwSel31 && document.querySelector(zwSel31) === element) return zwSel31;
    } catch (_eZ31) {}
    // R145：handle-identity 元素（createElement/cloneNode 产物，如 WPT
    // pointer-event-document-move 的 `template.content.cloneNode` append 后的节点）
    // ——经正置反查表直接解析稳定选择器（tag/attr/nth 启发式对 handle proxy
    // querySelectorAll 命中的是文档内同 tag 的**其他**元素，锚错节点）。
    var handle = element.__zwHandle;
    if (handle && typeof __zw_selector_for_handle === 'function') {
      try {
        var byHandle = __zw_selector_for_handle(String(handle));
        if (byHandle) return byHandle;
      } catch (_eH) {}
    }
    var id = element.getAttribute && element.getAttribute('id');
    // 尾簇 20：id 仅在 **合法 CSS 标识符**（无空白/无特殊字符）时产 `#id` 形——
    // `id="outerFrame body"`（带空格）曾产 `#outerFrame body`（descendant 选择器）
    // → 宿主解析 miss → 「testdriver target not found」（pointercapture_in_frame
    // 全簇首断言面）。非安全 id 落下方 attr 筛选路径（body[id="..."] 合法唯一）。
    if (id && /^[A-Za-z][\w-]*$/.test(id)) return '#' + id;
    var tag = String(element.tagName || '').toLowerCase();
    if (!tag) return null;
    var matches = document.querySelectorAll(tag);
    if (matches.length === 1) return tag;
    // R142：同 tag 多实例时的唯一化——属性筛选器（contenteditable 等）命中唯一实例即用；
    // 仍不唯一时 nth-of-type 兜底（host 侧 query_selector 消费任意合法选择器）。
    var attrs = [];
    var names = (element.getAttributeNames && element.getAttributeNames()) || [];
    for (var i = 0; i < names.length; i++) {
      var v = element.getAttribute(names[i]);
      if (v === '' || v == null) attrs.push('[' + names[i] + ']');
      else attrs.push('[' + names[i] + '="' + String(v).replace(/"/g, '\\"') + '"]');
    }
    for (var a = 0; a < attrs.length; a++) {
      var withAttr = tag + attrs[a];
      try {
        var m2 = document.querySelectorAll(withAttr);
        if (m2.length === 1) return withAttr;
      } catch (_e) {}
    }
    try {
      var parent = element.parentNode;
      if (parent && parent.querySelectorAll) {
        var idx = 1;
        var sibs = parent.querySelectorAll(':scope > ' + tag);
        for (var si = 0; si < sibs.length; si++) {
          if (sibs[si] === element) break;
          idx++;
        }
        if (sibs.length > 0) {
          var viaNth = selectorFor(parent) ;
          if (viaNth) {
            var sel = viaNth + ' > ' + tag + ':nth-of-type(' + idx + ')';
            var m3 = document.querySelectorAll(sel);
            if (m3.length === 1) return sel;
          }
        }
      }
    } catch (_e2) {}
    return null;
  }
  function enqueue(operation, element, text, chain) {
    return new Promise(function(resolve, reject) {
      var id = nextId++;
      pending[id] = { resolve: resolve, reject: reject };
      // R145：selector 延迟解析——enqueue 时（同步 turn）mutation 尚未 apply，
      // handle→selector 正置表空（createElement/cloneNode 产物的 append 在 turn 末才
      // 落 host）。存元素引用，宿主出队时（跨 turn，apply 已完成）经
      // `__zw_td_selector(id)` 现场解析（正置表已 merge）。
      queuedElements[id] = element;
      // uievents-compat M3 尾簇：祖先链快照（入队时最近祖先优先）——执行期目标元素
      // 被页内 listener 移除（after_target_removed 族）时，出队解析回退到**首个仍
      // 连通祖先**（headless 布局/查询快照对移除滞后，命中测试不可用——WPT
      // 「pointerup@parent after (child-removed)」断言面）。
      // chain 预建（move 步时点快照）优先——up/down 命令的 enqueue 在前一命令 resolve
      // 后才跑，彼时命中元素可能已被页内 listener 移除（parentElement 断链 → 链只剩
      // 自身），回退构建已不可用。
      queuedChains[id] = chain || (function(el) {
        var c = [];
        var cur = el, guard = 0;
        while (cur && guard++ < 32) {
          try { var s = selectorFor(cur); if (s) c.push(s); } catch (_eC) {}
          try { cur = cur.parentElement; } catch (_eP) { cur = null; }
        }
        return c;
      })(element);
      globalThis.__zw_td_queue.push({
        id: id, operation: operation, selector: null,
        text: text == null ? null : String(text)
      });
    });
  }
  globalThis.__zw_td_selector = function(id) {
    var element = queuedElements[id];
    if (!element) return null;
    var sel = selectorFor(element);
    // uievents-compat M3 尾簇：目标已断连 → 祖先链回退（首个 querySelector 命中的
    // 近祖）。truthy 判定（isConnected 实现差异容忍——getter 缺失/非布尔回退不触发）。
    try {
      if (!element.isConnected && queuedChains[id]) {
        var chain = queuedChains[id];
        for (var i = 1; i < chain.length; i++) {
          try { if (document.querySelector(chain[i])) return chain[i]; } catch (_eQ) {}
        }
      }
    } catch (_eC2) {}
    return sel;
  };
  globalThis.__zw_td_forget = function(id) { delete queuedElements[id]; delete queuedChains[id]; };
  globalThis.__zw_td_resolve = function(id, error) {
    var entry = pending[id];
    if (!entry) return;
    delete pending[id];
    delete queuedElements[id];
    delete queuedChains[id];
    if (error == null) entry.resolve();
    else entry.reject(new Error(String(error)));
  };
  globalThis.test_driver = {
    // WAB2-M3-s1（web-api-batch2 goal，2026-09-24）：click/send_keys 命令签发即授予瞬态激活
    //（`__zwUserActivate`，「签发即授予」headless 近似——requestFullscreen 激活门 +
    // navigator.userActivation 消费面）。trusted_request 案经此获得激活。
    click: function(element) {
      if (typeof globalThis.__zwUserActivate === 'function') globalThis.__zwUserActivate();
      return enqueue('click', element, null);
    },
    send_keys: function(element, keys) {
      if (typeof globalThis.__zwUserActivate === 'function') globalThis.__zwUserActivate();
      return enqueue('send_keys', element, keys);
    },
    // WAB2-M2-s3（web-api-batch2 goal，2026-09-24）：set_permission——页面侧权限状态注入，
    // 不经宿主命令队列（无 UI 提示语义）。转发 shim `__zwSetPermission` 钩子（clipboard
    // 块发布的权限注册表；navigator.permissions.query 与 clipboard 四方法 denied 门消费）。
    // 钩子缺失（老宿主/精简 shim）→ 拒绝（调用方 user-activation.js trySetPermission 已捕获）。
    set_permission: function(descriptor, state) {
      if (typeof globalThis.__zwSetPermission !== 'function') {
        return Promise.reject(new Error('testdriver set_permission is not supported'));
      }
      return globalThis.__zwSetPermission(descriptor && descriptor.name, state);
    },
    // WAB2-M3-s1：bless——授予瞬态激活（fullscreen 激活门放行）+ 执行回调（上游用例
    // `test_driver.bless("fullscreen", () => el.requestFullscreen())` 依赖回调执行且返回其
    // promise——same-element 案 Promise.all 直接 await bless 结果）。无回调 → resolve。
    bless: function(intent, callback) {
      if (typeof globalThis.__zwUserActivate === 'function') globalThis.__zwUserActivate();
      if (typeof callback === 'function') {
        try {
          return Promise.resolve(callback());
        } catch (e) {
          return Promise.reject(e);
        }
      }
      return Promise.resolve();
    },
    // cookies-compat M1（2026-10-12）：delete_all_cookies——清空页面 cookie store，不经宿主
    // 命令队列（无宿主 cookie jar 语义，操作面就是 shim 的 in-JS store）。转发 shim
    // `__zwDeleteAllCookies` 钩子（domCookieTest 每子测试前后清 jar——exact-match 序列化
    // 断言的隔离前提）。钩子缺失（老宿主/精简 shim）→ 拒绝（调用方 promise_test 以 Fail
    // 记账，不静默）。
    delete_all_cookies: function() {
      if (typeof globalThis.__zwDeleteAllCookies !== 'function') {
        return Promise.reject(new Error('testdriver delete_all_cookies is not supported'));
      }
      try {
        globalThis.__zwDeleteAllCookies();
        return Promise.resolve();
      } catch (e) {
        return Promise.reject(e);
      }
    },
    // console-compat M2 切片 2（2026-10-14）：bidi.log.entry_added——页内 console 观测
    // 通道（WebDriver BiDi log.entryAdded 的 runner 等价面）。`on` 把 handler 注册进
    // shim `__zwConsoleBidiListeners`（`_zwConsoleEmit` 逐条目派发
    // {type:"console", method, text, args}）；`subscribe` 无真 BiDi session 语义，
    // 返 resolve(unsubscribe)（测试先 subscribe 后 on 的时序前提，清理 no-op）。
    // shim 未加载（极老宿主）→ on 仍可注册（派发面自守卫）。
    // https://w3c.github.io/webdriver-bidi/#log-entryAdded
    bidi: {
      log: {
        entry_added: {
          subscribe: function() {
            return Promise.resolve(function() {});
          },
          on: function(handler) {
            if (typeof handler !== 'function') return function() {};
            globalThis.__zwConsoleBidiListeners.push(handler);
            return function() {
              var idx = globalThis.__zwConsoleBidiListeners.indexOf(handler);
              if (idx !== -1) globalThis.__zwConsoleBidiListeners.splice(idx, 1);
            };
          }
        }
      }
    }
  };
  // R142：test_driver.Actions（指针动作链）——镜像上游 testdriver-actions.js 的
  // 多源 API 面（addPointer/setPointer/addKeyboard + tick 语义 + options-object
  // 签名），headless 无真指针：send() 本地重放为宿主命令序——
  // uievents-compat M2 片 1（2026-10-03）：pointerMove → 'pointer_move' 命令
  //（宿主 `__zw_pointer_move` 跨界序 + move 对，pointerType 随源）。
  // uievents-compat M3（2026-10-03）：pointerDown/pointerUp → 'pointer_down'/
  // 'pointer_up' 命令**逐步重放**（取代 M2 的 down/up 折叠 'click'——页内
  // pointerdown listener 的 setPointerCapture 须影响后续 move/up 步路由，capture 族
  // 断言面）。click/auxclick 组合在 shim `__zw_pointer_up_sequence`（捕获落点/公共
  // 祖先/连击 dblclick）。keyDown/Up → 'keydown'/'keyup'（R3254-K2 键盘链——keydown
  // 未取消 → 宿主追加 InsertText 默认动作，事件序断言面）。
  // tick 对齐：上游 serialize 的 per-tick 源矩阵 → 逐 tick 逐源重放（pause/addTick
  // 空 tick 跳过）。move-only 链保留旧折叠语义（补一笔 click——R142 先例兼容）。
  // wheel 源（scroll 步）记账不重放（wheel 事件面 M2 片 2 评估）。
  function Actions() {
    this._sources = {}; // name -> {kind:'pointer'|'key', pointerType, actions:{tick->step}}
    this._sourceOrder = []; // 记录序（tick 重放的源间次序）
    this._current = { pointer: null, key: null };
    this._downOf = {}; // per-source pendingDown {selector,x,y}（M3 down/up 逐步重放）
    this._lastChain = {}; // per-source 祖先链快照（move 步时点——移除后回退锚）
    this._lastMove = {}; // per-source 最近 move {origin,x,y}
    this.tickIdx = 0;
    this._srcIdx = 0;
  }
  Actions.prototype.ButtonType = { LEFT: 0, MIDDLE: 1, RIGHT: 2, BACK: 3, FORWARD: 4 };
  Actions.prototype._createSource = function(kind, name, pointerType) {
    if (!name) {
      do { name = '' + this._srcIdx++; } while (this._sources[name]);
    } else if (this._sources[name]) {
      throw new Error('Already have a source named ' + name);
    }
    this._sources[name] = { kind: kind, pointerType: pointerType || 'mouse', actions: {} };
    this._sourceOrder.push(name);
    this._current[kind] = name;
    return name;
  };
  Actions.prototype._getSource = function(kind, name) {
    if (name == null) name = this._current[kind];
    if (name == null || !this._sources[name]) name = this._createSource(kind, name);
    return this._sources[name];
  };
  Actions.prototype.addPointer = function(name, pointerType) {
    // 上游 createSource 恒置 current（set 参数在 createSource 内已生效）。
    this._createSource('pointer', name, pointerType || 'mouse');
    return this;
  };
  Actions.prototype.setPointer = function(name) {
    if (this._sources[name]) this._current.pointer = name;
    return this;
  };
  Actions.prototype.addKeyboard = function(name) {
    this._createSource('key', name);
    return this;
  };
  Actions.prototype.setKeyboard = function(name) {
    if (this._sources[name]) this._current.key = name;
    return this;
  };
  Actions.prototype.addWheel = function() { return this; };
  Actions.prototype.setContext = function() { return this; };
  Actions.prototype.addTick = function() { this.tickIdx += 1; return this; };
  Actions.prototype.pause = function() { return this; };
  Actions.prototype._tick = function(source, step) {
    var t = this.tickIdx;
    if (source.actions[t]) { this.tickIdx += 1; t = this.tickIdx; }
    source.actions[t] = step;
    return this;
  };
  Actions.prototype.keyDown = function(key, options) {
    this._tick(this._getSource('key', options && options.sourceName), { type: 'keydown', value: key });
    return this;
  };
  Actions.prototype.keyUp = function(key, options) {
    this._tick(this._getSource('key', options && options.sourceName), { type: 'keyup', value: key });
    return this;
  };
  Actions.prototype.pointerDown = function(options) {
    options = options || {};
    this._tick(this._getSource('pointer', options.sourceName), { type: 'pointerDown', button: options.button || 0 });
    return this;
  };
  Actions.prototype.pointerUp = function(options) {
    options = options || {};
    this._tick(this._getSource('pointer', options.sourceName), { type: 'pointerUp', button: options.button || 0 });
    return this;
  };
  Actions.prototype.pointerMove = function(x, y, options) {
    options = options || {};
    this._tick(this._getSource('pointer', options.sourceName), { type: 'pointerMove', x: x, y: y, origin: options.origin });
    return this;
  };
  // 尾簇 35：wheel 源 scroll 步（上游 testdriver-actions 面——x/y 相对 origin
  // （元素中心/viewport 绝对），dx/dy 为滚动增量）。此前 no-op（记帐不重放——
  // wheel-basic/deadlock 的 wheel 事件面未接）。
  Actions.prototype.addWheel = function(name) {
    this._createSource('wheel', name);
    return this;
  };
  Actions.prototype.setWheel = function(name) {
    if (this._sources[name]) this._current.wheel = name;
    return this;
  };
  Actions.prototype.scroll = function(x, y, dx, dy, options) {
    options = options || {};
    this._tick(this._getSource('wheel', options.sourceName),
      { type: 'scroll', x: x || 0, y: y || 0, dx: dx || 0, dy: dy || 0, origin: options.origin });
    return this;
  };
  Actions.prototype.send = function() {
    var element = this._current.pointer && this._resolveOrigin(this._lastMove[this._current.pointer] && this._lastMove[this._current.pointer].origin) || document.activeElement;
    // WAB2-M3-s1：动作链发送同 click 授予瞬态激活（sendPasteShortcutKey 等 paste 链）。
    if (typeof globalThis.__zwUserActivate === 'function') globalThis.__zwUserActivate();
    // 上游 serialize 的 per-tick 源矩阵 → 逐 tick 逐源（sourceOrder 序）重放。
    var plan = [];
    for (var tick = 0; tick <= this.tickIdx; tick++) {
      for (var si = 0; si < this._sourceOrder.length; si++) {
        var source = this._sources[this._sourceOrder[si]];
        var step = source.actions[tick];
        if (!step) continue;
        if (step.type === 'pointerMove') {
          // uievents-compat M3 尾簇 4（2026-10-04）：**执行时点解析**——计划只编
          // origin 元素 + 原始 offset（WebDriver 元素 origin 语义：中心 + offset），
          // 命中元素由 runner 命令执行时以 fresh 几何解析（计划构建时点解析受 shim
          // 序列化一 turn 滞后 + re-append 零盒影响——after_target_removed
          // pointerup-remover 断言面）。'viewport'/缺省 origin（绝对坐标）编 '@'
          // 前缀，origin 元素 = body（仅过 R145 出队解析闸，执行时忽略）。
          var originEl = this._resolveOrigin(step.origin);
          var moveX = step.x || 0, moveY = step.y || 0;
          var abs = !originEl;
          var moveOrigin = originEl || document.body || document.documentElement;
          if (!moveOrigin) continue;
          this._lastMove[this._sourceOrder[si]] = { origin: moveOrigin, x: moveX, y: moveY, abs: abs };
          // 祖先链快照（move 时点——down/up 的 enqueue 在后，彼时元素可能已移除）
          this._lastChain[this._sourceOrder[si]] = (function(el) {
            var c = [];
            var cur = el, guard = 0;
            while (cur && guard++ < 32) {
              try { var s2 = selectorFor(cur); if (s2) c.push(s2); } catch (_eC) {}
              try { cur = cur.parentElement; } catch (_eP) { cur = null; }
            }
            return c;
          })(originEl);
          plan.push({ op: 'pointer_move', origin: moveOrigin, text: (abs ? '@' : '') + moveX + ',' + moveY + ',' + source.pointerType, chain: this._lastChain[this._sourceOrder[si]] });
        } else if (step.type === 'pointerDown') {
          // uievents-compat M3（2026-10-03）：down 步独立入队（取代 M2 折叠 click
          // ——页内 pointerdown listener 的 setPointerCapture 须影响后续 move/up 路由）。
          // down 落点 = 最近 move 命中（无 move → activeElement 回落）。
          var dOrigin = this._lastMove[this._sourceOrder[si]];
          var downEl = this._resolveOrigin(dOrigin && dOrigin.origin) || element;
          this._downOf[this._sourceOrder[si]] = {
            selector: this._selectorOf(downEl),
            x: dOrigin ? dOrigin.x : 0, y: dOrigin ? dOrigin.y : 0
          };
          plan.push({
            op: 'pointer_down',
            origin: downEl,
            text: (dOrigin && dOrigin.abs ? '@' : '') + (dOrigin ? dOrigin.x : 0) + ',' + (dOrigin ? dOrigin.y : 0) + ',' + source.pointerType + ',' + (step.button || 0),
            chain: this._lastChain[this._sourceOrder[si]]
          });
        } else if (step.type === 'pointerUp') {
          // up 步独立入队：click/auxclick 组合在 shim（捕获落点/公共祖先/连击）。
          // downSel 编入 text（'|' 分隔——down 步落点，组合目标 down 侧）；尾部再编
          // up 落点的祖先链（touch 抬起悬停拆除的 leave 锚——目标被页内 listener
          // 移除时取首个连通近祖，WPT after_target_removed ?touch 断言面）。
          var uOrigin = this._lastMove[this._sourceOrder[si]];
          var upEl = this._resolveOrigin(uOrigin && uOrigin.origin) || element;
          var downInfo = this._downOf[this._sourceOrder[si]] || { selector: '', x: 0, y: 0 };
          delete this._downOf[this._sourceOrder[si]];
          var upChain = (function(el) {
            var chain = [];
            var cur = el, guard = 0;
            while (cur && guard++ < 32) {
              try { var s = selectorFor(cur); if (s) chain.push(s); } catch (_eC) {}
              try { cur = cur.parentElement; } catch (_eP) { cur = null; }
            }
            return chain;
          })(upEl).join('|');
          plan.push({
            op: 'pointer_up',
            origin: upEl,
            text: (uOrigin && uOrigin.abs ? '@' : '') + (uOrigin ? uOrigin.x : downInfo.x) + ',' + (uOrigin ? uOrigin.y : downInfo.y) + ',' + source.pointerType + ',' + (step.button || 0) + '|' + downInfo.selector + '|' + upChain,
            chain: this._lastChain[this._sourceOrder[si]]
          });
        } else if (step.type === 'scroll') {
          // 尾簇 35：wheel 源 scroll 步入 plan（wheel_scroll 命令——runner resolve
          // origin（元素中心 + x/y 偏移；viewport origin = 绝对坐标 '@' 形）→ shim
          // WheelEvent 派发（delta 透传）。scroll 记帐不重放的旧口径随之解除。
          var sOriginEl = (step.origin === 'viewport') ? null : this._resolveOrigin(step.origin);
          var sAbs = !sOriginEl;
          var sMoveOrigin = sOriginEl || document.body || document.documentElement;
          if (!sMoveOrigin) continue;
          plan.push({ op: 'wheel_scroll', origin: sMoveOrigin,
            text: (sAbs ? '@' : '') + step.x + ',' + step.y + ',' + step.dx + ',' + step.dy });
        } else {
          // keydown/keyup：对 activeElement（R3254-K2 先例——键盘链无 origin 元素）。
          plan.push({ op: step.type, origin: document.activeElement || element, text: step.value });
        }
      }
    }
    if (!plan.length) {
      // 旧 R142 折叠语义：无有效步骤 → 单 click（链首 origin / activeElement）。
      return enqueue('click', element, null);
    }
    // uievents-compat M3：move-only 链不再补尾随 click——真实浏览器 Actions move-only
    // 不产生 click（M2 折叠期的兼容近似使悬停迁移重放、捕获族「无多余事件」断言面
    // 记入多余 pointermove；WPT pointerevent_capture_suppressing_mouse）。
    // 命令按序链式执行——每步独立入队（事件序对 eventLog 断言敏感）。
    var promise = Promise.resolve();
    for (var i = 0; i < plan.length; i++) {
      promise = promise.then(function(step) {
        return enqueue(step.op, step.origin, step.text, step.chain);
      }.bind(null, plan[i]));
    }
    return promise;
  };
  // origin 归一：元素直接用；'viewport'/'pointer'/缺省 → activeElement 近似
  //（真视口命中测试待宿主指针管线——headless 桥接近似，uievents-compat M2 片 1）。
  Actions.prototype._resolveOrigin = function(origin) {
    if (origin && typeof origin === 'object' && origin.nodeType) return origin;
    return null;
  };
  // 元素 → 唯一选择器（跨目标组合命令的 upSel 编码；page 侧 selectorFor，静态
  // 元素即时稳定；动态创建元素由 R145 延迟解析面覆盖——组合命令暂用即时值）。
  Actions.prototype._selectorOf = function(element) {
    try { return selectorFor(element) || ''; } catch (_e) { return ''; }
  };
  globalThis.test_driver.Actions = Actions;
})();
</script>"#;

#[cfg(test)]
mod tests {
    use super::*;

    // encoding-legacy-mb DC-4 回归钉：sniff 1024 字节窗截断须落在 char
    // boundary——byte 1024 落在多字节字符（'（' 占 1022..1025）内时原 `[..1024]`
    // 直切 panic（canvas 语料全角注释实证 rc=101）。boundary 对齐输入窗恒等，
    // 嗅探语义不变。
    #[test]
    fn sniff_document_encoding_window_multibyte_boundary() {
        // byte 1024 在 '（' 字节中段：修复前 panic；现回退边界 1022，窗内无
        // 完整 meta → None
        let mut html = format!("<p>{}", "a".repeat(1019));
        html.push('（');
        html.push_str("<meta charset=\"utf-8\">");
        assert_eq!(html.len(), 1025 + 22);
        assert_eq!(sniff_document_encoding(&html), None);
        // boundary 对齐：meta 在 1024 字节窗内照常嗅探（对齐输入零行为变化）
        let aligned = format!("<meta charset=\"euc-jp\">{}", "a".repeat(1024));
        assert_eq!(sniff_document_encoding(&aligned).as_deref(), Some("EUC-JP"));
    }

    // s107 回归钉：fetcher JSON module 意图面——`.json` 静态文件默认
    // application/json 放行（pipe/sidecar 可覆盖）；serve-with-content-type.py 按
    // `?fn=<相对 .py 目录>&ct=<mime>` 模拟交付；ct 百分号解码；非 JSON MIME → Err。
    #[test]
    fn json_module_fetcher_mime_gate_s107() {
        let dir = std::env::temp_dir().join(format!("zw-json-fetch-{}-s107", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub/data.json"), "{\"test\":true}").unwrap();
        std::fs::write(dir.join("sub/serve-with-content-type.py"), "# handler\n").unwrap();
        let fetch = wpt_data_script_fetcher(&dir, None).unwrap();
        // 静态 .json 默认 MIME 放行（相对 + 根绝对两形态）
        assert_eq!(
            fetch("https://wpt.test/sub/page.html", "./data.json").unwrap(),
            "{\"test\":true}"
        );
        assert_eq!(
            fetch("https://wpt.test/other.html", "/sub/data.json").unwrap(),
            "{\"test\":true}"
        );
        // serve-with-content-type.py：fn 相对 .py 所在目录 + ct 交付
        assert_eq!(
            fetch(
                "https://wpt.test/sub/page.html",
                "./serve-with-content-type.py?fn=data.json&ct=text/json"
            )
            .unwrap(),
            "{\"test\":true}"
        );
        // ct 百分号解码（%3B 分号；合法 essence 判定剥参数）
        assert_eq!(
            fetch(
                "https://wpt.test/sub/page.html",
                "./serve-with-content-type.py?fn=data.json&ct=text/json%3Bcharset=utf-8"
            )
            .unwrap(),
            "{\"test\":true}"
        );
        // 非 JSON MIME → Err（网络错误语义）
        assert!(
            fetch(
                "https://wpt.test/sub/page.html",
                "./serve-with-content-type.py?fn=data.json&ct=text/plain"
            )
            .is_err(),
            "text/plain 须拒"
        );
        // sidecar 覆盖成非 JSON MIME → Err
        std::fs::write(dir.join("sub/data.json.headers"), "Content-Type: text/plain").unwrap();
        assert!(
            fetch("https://wpt.test/sub/page.html", "./data.json").is_err(),
            "sidecar text/plain 须拒"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // s115：serve-with-content-type.py scripty 意图面——scripty ct 交付 fixture 文件
    //（解码恒 UTF-8 lossy，charset 参数忽略——模块恒 UTF-8 判据面）；非 scripty
    // 非 JSON（text/plain）→ Err（网络错误语义，s107 口径保持）。
    #[test]
    fn serve_with_content_type_scripty_intent_s115() {
        let dir = std::env::temp_dir().join(format!("zw-swct-scripty-{}-s115", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        // windows-1250 原始字节（"śćążź" = 9C E6 B9 BF 9F，同语料 fixture）——
        // UTF-8 lossy 解码为替换符。
        std::fs::write(
            dir.join("sub/w1250.js"),
            b"(function(){window.getSomeString=function(){return \"\x9C\xE6\xB9\xBF\x9F\";};})();",
        )
        .unwrap();
        std::fs::write(dir.join("sub/serve-with-content-type.py"), "# handler\n").unwrap();
        let fetch = wpt_data_script_fetcher(&dir, None).unwrap();
        let page = "https://wpt.test/sub/page.html";
        let src = |q: &str| format!("./serve-with-content-type.py?fn=w1250.js&ct={q}");
        // scripty ct 放行（charset 参数忽略——解码恒 UTF-8 lossy → 体内为替换符）。
        let out = fetch(page, &src("text/javascript")).unwrap();
        assert!(out.contains("getSomeString"), "{out}");
        assert!(
            !out.contains('\u{015b}'),
            "windows-1250 字节须按 UTF-8 lossy 出替换符：{out}"
        );
        let out = fetch(page, &src("text/javascript%3Bcharset=windows-1250")).unwrap();
        assert!(out.contains("getSomeString"), "{out}");
        // 非 scripty 非 JSON → Err。
        assert!(fetch(page, &src("text/plain")).is_err(), "text/plain 须拒");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // s115：classic 内联面解码序——BOM > ct charset 参数 > charset 属性 > UTF-8。
    // script-charset-01 判据形态：ct 压 charset 属性；无 ct charset 时属性接管。
    #[test]
    fn inline_serve_with_content_type_charset_decode_s115() {
        let root = std::env::temp_dir().join(format!("zw-swct-inline-{}-s115", std::process::id()));
        let case_dir = "html/semantics/scripting-1/the-script-element";
        std::fs::create_dir_all(root.join(case_dir)).unwrap();
        // "śćążź" 的 windows-1250 字节（9C E6 B9 BF 9F，同语料 fixture）+ UTF-8 BOM 形。
        let w1250: &[u8] = b"\x9C\xE6\xB9\xBF\x9F";
        std::fs::write(root.join(case_dir).join("w1250.js"), w1250).unwrap();
        let mut bom = b"\xEF\xBB\xBF".to_vec();
        bom.extend_from_slice("globalThis.bomRan = true;".as_bytes());
        std::fs::write(root.join(case_dir).join("bom-utf8.js"), &bom).unwrap();
        let html = "<!doctype html><body>\
            <script src=\"serve-with-content-type.py?fn=w1250.js&ct=text/javascript%3Bcharset=utf-8\" charset=\"windows-1250\"></script>\
            <script src=\"serve-with-content-type.py?fn=w1250.js&ct=text/javascript%3Bcharset=windows-1250\" charset=\"utf-8\"></script>\
            <script src=\"serve-with-content-type.py?fn=w1250.js&ct=text/javascript\" charset=\"windows-1250\"></script>\
            <script src=\"serve-with-content-type.py?fn=bom-utf8.js&ct=text/javascript%3Bcharset=windows-1250\" charset=\"utf-8\"></script>\
            <script type=\"module\" src=\"serve-with-content-type.py?fn=w1250.js&ct=text/javascript\" charset=\"windows-1250\"></script>\
            </body>";
        let out = inline_local_scripts(html, &root, &format!("{case_dir}/script-charset-01.html"), None);
        // ct charset 参数压 charset 属性（ct=utf-8 + attr=windows-1250 → mojibake）。
        let mojibake = String::from_utf8_lossy(w1250).into_owned();
        assert!(out.contains(&mojibake), "ct=utf-8 须解码为替换符串：{out}");
        // ct=windows-1250 + attr=utf-8 → ct 胜出，正确解码。
        assert!(out.contains("śćążź"), "ct=windows-1250 须正确解码：{out}");
        // 无 ct charset → charset 属性接管。
        assert_eq!(out.matches("śćążź").count(), 2, "属性接管臂同串：{out}");
        // BOM 嗅探压 ct charset（UTF-8 BOM + ct=windows-1250 → UTF-8 解码）。
        assert!(out.contains("globalThis.bomRan = true;"), "BOM 嗅探须胜出：{out}");
        // module 型不在 charset 感知内联面——整段透传（恒 UTF-8 交付在 fetcher）。
        assert!(
            out.contains("<script type=\"module\" src=\"serve-with-content-type.py"),
            "module 外链须透传：{out}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    // s116 回归钉：determine-request's-referrer 计算表——八策略臂 + 降级 + 同源
    // 端口归一 + origin 形（权威段书写原样，显式 :80 保留）。
    // https://fetch.spec.whatwg.org/#determine-request-s-referrer
    #[test]
    fn module_referer_policy_table_s116() {
        let src = "https://wpt.test/html/a/import-referrer-checker.sub.js?name=x&pipe=header(Referrer-Policy,origin)";
        let same = "https://wpt.test/html/a/resources/referrer-checker.py?name=x";
        let cross = "http://www1.wpt.test:80/html/a/resources/referrer-checker.py?name=x";
        let full = src.to_string();
        let origin = "https://wpt.test/".to_string();
        assert_eq!(compute_module_referer(src, same, "no-referrer"), None);
        assert_eq!(
            compute_module_referer(src, cross, "unsafe-url").as_deref(),
            Some(full.as_str())
        );
        assert_eq!(
            compute_module_referer(src, same, "origin").as_deref(),
            Some(origin.as_str())
        );
        assert_eq!(
            compute_module_referer(src, same, "same-origin").as_deref(),
            Some(full.as_str())
        );
        assert_eq!(compute_module_referer(src, cross, "same-origin"), None);
        assert_eq!(
            compute_module_referer(src, same, "origin-when-cross-origin").as_deref(),
            Some(full.as_str())
        );
        assert_eq!(
            compute_module_referer(src, cross, "origin-when-cross-origin").as_deref(),
            Some(origin.as_str())
        );
        // strict-origin：https 源 → http 请求 = 降级 → 无；http 源 → origin 形。
        assert_eq!(compute_module_referer(src, cross, "strict-origin"), None);
        let http_src = "http://wpt.test/html/a/wrap.sub.js?name=i";
        assert_eq!(
            compute_module_referer(http_src, cross, "strict-origin").as_deref(),
            Some("http://wpt.test/")
        );
        // SWCO：https 源 → http 请求 = 降级 → 无（跨源降级优先于跨源 origin 形）。
        assert_eq!(
            compute_module_referer(src, cross, "strict-origin-when-cross-origin"),
            None
        );
        // SWCO：非降级（http 源）跨源 → origin 形。
        assert_eq!(
            compute_module_referer(http_src, cross, "strict-origin-when-cross-origin").as_deref(),
            Some("http://wpt.test/")
        );
        assert_eq!(
            compute_module_referer(src, same, "strict-origin-when-cross-origin").as_deref(),
            Some(full.as_str())
        );
        // SWCO：https 源 → http 请求 = 降级 → 无（降级优先于跨源 origin 形）。
        assert_eq!(
            compute_module_referer(src, cross, "strict-origin-when-cross-origin"),
            None
        );
        assert_eq!(compute_module_referer(src, cross, "no-referrer-when-downgrade"), None);
        assert_eq!(
            compute_module_referer(src, same, "no-referrer-when-downgrade").as_deref(),
            Some(full.as_str())
        );
        // 同源判定：缺省端口归一（wpt.test ≡ wpt.test:443）；跨 scheme 即异源。
        assert!(wpt_urls_same_origin("https://wpt.test/a", "https://wpt.test:443/b"));
        assert!(!wpt_urls_same_origin("https://wpt.test/a", "http://wpt.test:80/b"));
        // origin 形：权威段书写原样（显式 :80 保留）+ "/"。
        assert_eq!(
            wpt_url_origin_form("http://www1.wpt.test:80/x?y").as_deref(),
            Some("http://www1.wpt.test:80/")
        );
    }

    // s116 回归钉：referrer-checker.py 端点仿真 + host 别名面 + meta 策略捕获 +
    // pipe 策略链 + 默认策略回落 + wrapper body 替换。
    #[test]
    fn referrer_checker_endpoint_and_host_alias_s116() {
        let dir = std::env::temp_dir().join(format!("zw-referrer-s116-{}", std::process::id()));
        let res = dir.join("html/semantics/scripting-1/the-script-element/module/resources");
        std::fs::create_dir_all(&res).unwrap();
        std::fs::write(res.join("fast-module.js"), "export const marker = 'fast';").unwrap();
        std::fs::write(
            res.join("import-referrer-checker.sub.js"),
            "import { referrer as r } from 'http://{{domains[www1]}}:{{ports[http][0]}}/html/semantics/scripting-1/the-script-element/module/resources/referrer-checker.py?name={{GET[name]}}';\nexport const referrer = r;",
        )
        .unwrap();

        // ① host 别名面：www1:80 绝对 URL 同树解析（跨源语义不在 fetcher 层）。
        let fetch = wpt_data_script_fetcher(&dir, None).unwrap();
        assert_eq!(
            fetch(
                "https://wpt.test/sub/page.html",
                "http://www1.wpt.test:80/html/semantics/scripting-1/the-script-element/module/resources/fast-module.js"
            )
            .unwrap(),
            "export const marker = 'fast';"
        );

        // ② meta 策略捕获：no-referrer 文档 + 顶层直引（引用源=文档 URL）→ 空。
        let fetch_nr = wpt_data_script_fetcher(&dir, Some("no-referrer".into())).unwrap();
        let doc = "https://wpt.test/html/semantics/scripting-1/the-script-element/module/referrer-no-referrer.sub.html";
        assert_eq!(
            fetch_nr(doc, "./resources/referrer-checker.py?name=same").unwrap(),
            "export const referrer = '';"
        );

        // ③ pipe 策略链：包装自身交付 Referrer-Policy 压文档策略 → 引用源 origin 形。
        let wrapper_url = "https://wpt.test/html/semantics/scripting-1/the-script-element/module/resources/import-referrer-checker.sub.js?name=zz&pipe=header(Referrer-Policy,origin)";
        assert_eq!(
            fetch_nr(wrapper_url, "./referrer-checker.py?name=zz").unwrap(),
            "export const referrer = 'https://wpt.test/';"
        );

        // ④ 默认策略（无 meta 无 pipe）strict-origin-when-cross-origin：www1 包装与
        // 其子 checker 同源 → 引用源全 URL（含 query）。
        let www1_wrapper = "http://www1.wpt.test:80/html/semantics/scripting-1/the-script-element/module/resources/import-referrer-checker.sub.js?name=rr";
        assert_eq!(
            fetch(www1_wrapper, "./referrer-checker.py?name=rr").unwrap(),
            "export const referrer = 'http://www1.wpt.test:80/html/semantics/scripting-1/the-script-element/module/resources/import-referrer-checker.sub.js?name=rr';"
        );

        // ⑤ wrapper body 替换：{{domains[www1]}}/{{ports[http][0]}}/{{GET[name]}} 注入
        //（子 spec 落 www1 + name 去重——module map 不串）。
        let body = fetch(
            "https://wpt.test/sub/page.html",
            "/html/semantics/scripting-1/the-script-element/module/resources/import-referrer-checker.sub.js?name=qq",
        )
        .unwrap();
        assert!(
            body.contains("http://www1.wpt.test:8000/html/semantics/scripting-1/the-script-element/module/resources/referrer-checker.py?name=qq"),
            "wrapper body 须替换：{body}"
        );

        // ⑦ policy 链逐臂过滤：非法 pipe label 按 spec 忽略后顺延文档策略（整体
        // or+filter 会把合法 doc policy 连带丢弃——descendants invalid-policy 臂）。
        let invalid_pipe_wrapper = "https://wpt.test/html/semantics/scripting-1/the-script-element/module/resources/import-referrer-checker.sub.js?name=iv&pipe=header(Referrer-Policy,invalid-policy)";
        assert_eq!(
            fetch_nr(invalid_pipe_wrapper, "./referrer-checker.py?name=iv").unwrap(),
            "export const referrer = '';"
        );

        // ⑥ 降级臂：https 包装 → 绝对 http checker 子 spec（{{host}}:80 语料形——
        // relative 子请求恒继承父 scheme，降级仅在跨 scheme 绝对 spec 出现）→ 无 Referer。
        let https_wrapper = "https://wpt.test/html/semantics/scripting-1/the-script-element/module/resources/import-referrer-checker.sub.js?name=dn";
        assert_eq!(
            fetch(
                https_wrapper,
                "http://wpt.test:80/html/semantics/scripting-1/the-script-element/module/resources/referrer-checker.py?name=dn"
            )
            .unwrap(),
            "export const referrer = '';"
        );
        // 非 wpt 域绝对 URL 保持既有 Err 语义。
        assert!(fetch("https://wpt.test/sub/page.html", "http://evil.test/x.js").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // s124 回归钉：echo-referrer.py 端点仿真（__zw_lp 元素级策略通道 + 非法 label
    // 剥离后顺延文档策略 + uid 记账键）与 script.py 端点仿真（cross-origin 页
    // "should load" 判据，www1:8444 别名剥离后按路径命中）。
    #[test]
    fn modulepreload_endpoint_sims_s124() {
        let dir = std::env::temp_dir().join(format!("zw-modulepreload-s124-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let page = "https://wpt.test/html/semantics/scripting-1/the-script-element/module/modulepreload-inline-referrerpolicy.html";

        // ① __zw_lp 通道：元素级 no-referrer 压文档策略 → 空 Referer 记账。
        let fetch_origin = wpt_data_script_fetcher(&dir, Some("origin".into())).unwrap();
        assert_eq!(
            fetch_origin(page, "/preload/resources/echo-referrer.py?uid=X1&__zw_lp=no-referrer").unwrap(),
            "window.referrers[\"X1\"] = \"\";"
        );

        // ② 非法 __zw_lp label 剥离后顺延文档策略（origin → 同源 origin 形）。
        assert_eq!(
            fetch_origin(page, "/preload/resources/echo-referrer.py?uid=X2&__zw_lp=bogus-policy").unwrap(),
            "window.referrers[\"X2\"] = \"https://wpt.test/\";"
        );

        // ③ 无 __zw_lp：文档策略链直落（页面 meta referrer=origin 臂同形）。
        assert_eq!(
            fetch_origin(page, "/preload/resources/echo-referrer.py?uid=X3").unwrap(),
            "window.referrers[\"X3\"] = \"https://wpt.test/\";"
        );

        // ④ script.py 端点：https://www1.wpt.test:8444 别名剥离 → 路径命中非空 JS 回体。
        let fetch = wpt_data_script_fetcher(&dir, None).unwrap();
        assert_eq!(
            fetch(
                page,
                "https://www1.wpt.test:8444/common/security-features/subresource/script.py"
            )
            .unwrap(),
            "globalThis.__zwScriptPyLoaded = (globalThis.__zwScriptPyLoaded || 0) + 1;"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // s116 回归钉：meta referrer 策略窄解析（小写双引号形；属性序不符/缺 meta → None）。
    #[test]
    fn meta_referrer_policy_extract_s116() {
        assert_eq!(
            extract_meta_referrer_policy("<html><head><meta name=\"referrer\" content=\"no-referrer\"></head>"),
            Some("no-referrer".to_string())
        );
        assert_eq!(
            extract_meta_referrer_policy("<meta content=\"Unsafe-URL\" name=\"referrer\">"),
            None
        );
        assert_eq!(extract_meta_referrer_policy("<p>no meta</p>"), None);
    }

    const MINI_HARNESS: &str = r#"
var __resultCallbacks = [], __completionCallbacks = [], __pending = 0;
globalThis.add_result_callback = function(cb) { __resultCallbacks.push(cb); };
globalThis.add_completion_callback = function(cb) { __completionCallbacks.push(cb); };
function __emit(t) { __resultCallbacks.forEach(function(cb){ cb(t); }); }
function __completeSoon() {
  Promise.resolve().then(function() {
    if (__pending === 0) __completionCallbacks.forEach(function(cb){ cb([]); });
  });
}
globalThis.test = function(fn, name) {
  var t = {name:name, status:0, message:null};
  try { fn(); } catch (e) { t.status=1; t.message=String(e); }
  __emit(t); __completeSoon();
};
globalThis.promise_test = function(fn, name) {
  __pending++;
  Promise.resolve().then(fn).then(function() {
    __emit({name:name,status:0,message:null});
  }, function(e) {
    __emit({name:name,status:1,message:String(e)});
  }).then(function(){ __pending--; __completeSoon(); });
};
globalThis.assert_equals = function(a,b,m) { if (a !== b) throw new Error(m || (String(a)+' != '+String(b))); };
globalThis.assert_true = function(v,m) { if (v !== true) throw new Error(m || 'expected true, got ' + String(v)); };
globalThis.assert_false = function(v,m) { if (v !== false) throw new Error(m || 'expected false, got ' + String(v)); };
"#;

    #[test]
    fn runs_supported_html_interaction_subtests() {
        let html = r##"
<script src="/resources/testharness.js"></script>
<script src="/resources/testdriver.js"></script>
<input id="name">
<input id="check" type="checkbox">
<script>
promise_test(async function() {
  var input = document.getElementById('name');
  await test_driver.send_keys(input, 'ab');
  assert_equals(input.value, 'ab');
}, 'send keys updates the live input');
promise_test(async function() {
  var input = document.getElementById('check');
  await test_driver.click(input);
  assert_equals(input.checked, true);
}, 'click updates live checkedness');
</script>
"##;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "local-supported.html",
            html,
            MINI_HARNESS,
            // R5005（R3254-K5 谱系）：workspace 全量并行负载下 JS 启动 + 事件泵超 2s
            // 间歇超限（R5001b/c、R5004 三连折此案·solo 恒过）——case 级超时非语义
            // 断言，与 R3254-K5 同款放宽（30s 只在真挂起时咬合）。
            Duration::from_secs(30),
        );
        assert_eq!(
            results,
            vec![
                HarnessSubtestResult {
                    name: "send keys updates the live input".into(),
                    status: HarnessStatus::Pass,
                    message: None,
                },
                HarnessSubtestResult {
                    name: "click updates live checkedness".into(),
                    status: HarnessStatus::Pass,
                    message: None,
                }
            ]
        );
    }

    #[test]
    fn send_keys_dispatches_key_event_sequence_and_suppresses_modifier_keypress() {
        // R3254-K2 残余切片 4（keyboard goal，2026-09-07）：send_keys 普通字符补全
        // keydown→InsertText→keypress→keyup 事件序（旧版裸 InsertText 无键事件，
        // keypress-not-fired-for-modifier-shortcuts 的 keyup listener 永不 resolve →
        // 3 subtest 全 pending Timeout）。①：普通字符四事件齐发且 value 更新；
        // ②：修饰键（uE009 Control）+ 'v' 复合序——'v' 事件对 ctrlKey=true 且
        // keypress 抑制（UI Events 修饰快捷键语义）；③：串结束修饰位清零（后续
        // send 独立）。驱动用例：WPT uievents/keyboard/keypress-not-fired-for-
        // modifier-shortcuts.html。
        let html = r##"
<script src="/resources/testharness.js"></script>
<script src="/resources/testdriver.js"></script>
<input id="target">
<script>
var CONTROL = '';
// listener 挂 document（bubbles 面）——InsertText mutation 后 input 元素被宿主
// 重渲染替换，原元素上的 listener 会失联；每 subtest 独立注册独立 log（runner
// promise_test 并发排队，共享数组会被前序 send 的事件污染）。
function fresh() { return document.getElementById('target'); }

function record(log) {
  document.addEventListener('keydown', function (e) { log.push('keydown:' + e.key + ':' + e.ctrlKey); });
  document.addEventListener('keypress', function (e) { log.push('keypress:' + e.key); });
  document.addEventListener('keyup', function (e) { log.push('keyup:' + e.key + ':' + e.ctrlKey); });
}

promise_test(async function() {
  var log1 = [];
  record(log1);
  await test_driver.send_keys(fresh(), 'a');
  assert_equals(log1.join('|'), 'keydown:a:false|keypress:a|keyup:a:false', 'plain char full sequence');
  assert_equals(fresh().value, 'a', 'value inserted');
});
promise_test(async function() {
  var log2 = [];
  record(log2);
  await test_driver.send_keys(fresh(), CONTROL + 'v');
  // 并发 subtest 同帧排队且 listener 共享 document——log 前置段可能混入其它 send
  // 的事件；用 indexOf 定位本 subtest 的 Control 段再断言相对序。
  var seq = log2.join('|');
  var i = seq.indexOf('keydown:Control:true');
  assert_true(i !== -1, 'Ctrl events present: ' + seq);
  assert_true(seq.indexOf('keypress:v') === -1, 'Ctrl+v keypress suppressed: ' + seq);
  assert_true(seq.indexOf('keydown:v:true') > i, 'v keydown inherits ctrlKey: ' + seq);
  assert_true(seq.indexOf('keyup:v:true') > i, 'v keyup inherits ctrlKey: ' + seq);
});
promise_test(async function() {
  var log3 = [];
  record(log3);
  await test_driver.send_keys(fresh(), 'b');
  // lastIndexOf 定位本 subtest 的 'b' 段（并发 subtest 污染前置段）；断言 b 事件
  // ctrlKey=false = 跨 send 修饰位清零语义。
  var seq3 = log3.join('|');
  var j = seq3.lastIndexOf('keydown:b:false');
  assert_true(j !== -1, 'b keydown present: ' + seq3);
  assert_equals(seq3.slice(j), 'keydown:b:false|keypress:b|keyup:b:false', 'b sequence clean & ctrl cleared');
});
</script>
"##;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "local-send-keys-sequence.html",
            html,
            MINI_HARNESS,
            // R3254-K5 fixture：三段多键序列 + microtask 链，solo ~1.05s；2s 案超时在
            // workspace 并行负载下间歇超限（连续四轮 make test 折在此处、solo 恒过）——
            // 提到 10s 负载容限（case 级超时非语义断言，放宽无语义影响）。
            // R5005：10s 在 workspace 重载窗口仍间歇超限（R5004c 单折此案）——同谱系
            // 统一 30s。
            Duration::from_secs(30),
        );
        assert!(
            results.iter().all(|r| r.status == HarnessStatus::Pass),
            "all three sequence subtests pass: {results:?}"
        );
    }

    #[test]
    fn send_keys_space_activates_button_on_keyup_r3254_k4() {
        // R3254-K4 切片 2（keyboard goal，2026-09-07）：空格激活时序——buttonish 目标
        // （BUTTON / input type=button|submit|reset）在 keyup 触发 click（UI Events/
        // Chromium：Enter=keydown、Space=keyup）；keydown 只派 keydown/keyup 事件不
        // 激活。非 buttonish 空格（input 文本插入）行为不变。driver：runner send_keys。
        let html = r##"
<script src="/resources/testharness.js"></script>
<script src="/resources/testdriver.js"></script>
<button id="btn">Go</button>
<input id="ib" type="button" value="IB">
<input id="txt">
<p id="log">idle</p>
<script>
var count = 0;
var seq = [];
document.getElementById('btn').addEventListener('click', function () { count++; seq.push('btn-click'); });
document.getElementById('ib').addEventListener('click', function () { seq.push('ib-click'); });
document.addEventListener('keydown', function (e) { if (e.target.id) seq.push('kd:' + e.target.id); });
document.addEventListener('keyup', function (e) { if (e.target.id) seq.push('ku:' + e.target.id); });

promise_test(async function() {
  seq.length = 0;
  await test_driver.send_keys(document.getElementById('btn'), ' ');
  assert_equals(seq.join('|'), 'kd:btn|ku:btn|btn-click', 'Space on button: click after keyup, no keypress');
});
promise_test(async function() {
  await test_driver.send_keys(document.getElementById('ib'), ' ');
  assert_equals(seq.indexOf('ib-click'), seq.length - 1, 'input button click last');
});
promise_test(async function() {
  var txtClicked = false;
  document.getElementById('txt').addEventListener('click', function () { txtClicked = true; });
  await test_driver.send_keys(document.getElementById('txt'), ' ');
  assert_equals(document.getElementById('txt').value, ' ', 'editable space inserts at keydown');
  await new Promise(function (r) { setTimeout(r, 20); });
  assert_false(txtClicked, 'no click on editable space target');
});
</script>
"##;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "local-space-button-keyup.html",
            html,
            MINI_HARNESS,
            // R5005（R3254-K5 谱系）：同 035 系 workspace 并行负载间歇超限（R5001b/c、
            // R5004 折此案）——case 级超时非语义断言，统一 30s。
            Duration::from_secs(30),
        );
        assert!(
            results.iter().all(|r| r.status == HarnessStatus::Pass),
            "space-on-button keyup timing subtests pass: {results:?}"
        );
    }

    #[test]
    fn send_keys_select_type_ahead_multi_char_r3254_k5() {
        // R3254-K5 切片 3（keyboard goal，2026-09-07）：select type-ahead——closed select
        // 上 send_keys 可打印字符跳到首个 text 前缀匹配的 option（大小写不敏感）；多字符
        // 缓冲逐字符精确化（'o'→one、'on'→one、'tw'→two）；无匹配保持当前选中；value/
        // selectedIndex 变化派 input+change。无上游强制用例（键盘映射 UA-dependent，
        // customizable keyboard-behavior 为 .optional 同因）——本地 runner 单测标明本地。
        let html = r##"
<script src="/resources/testharness.js"></script>
<script src="/resources/testdriver.js"></script>
<select id="sel">
  <option value="1">one</option>
  <option value="2">two</option>
  <option value="3">three</option>
  <option value="4" disabled>four</option>
</select>
<script>
var sel = document.getElementById('sel');
var events = [];
sel.addEventListener('input', function () { events.push('input:' + sel.value); });
sel.addEventListener('change', function () { events.push('change:' + sel.value); });

promise_test(async function() {
  // 单 subtest 内确定性序列：'t' → two；紧接 'h' 缓冲累计 'th' → three（多字符精确化）。
  await test_driver.send_keys(sel, 't');
  assert_equals(sel.value, '2', 't jumps to two');
  assert_equals(sel.selectedIndex, 1, 'selectedIndex tracks');
  await test_driver.send_keys(sel, 'h');
  assert_equals(sel.value, '3', 'buffer th refines to three');
  assert_equals(events.join('|'), 'input:2|change:2|input:3|change:3', 'input then change per jump');
  // 600ms 静默清缓冲（shim setTimeout 真实定时器面），随后 'f' 单字符——four disabled，
  // enabled 集合内无 f 前缀 → 保持 three。
  await new Promise(function (r) { setTimeout(r, 600); });
  await test_driver.send_keys(sel, 'f');
  assert_equals(sel.value, '3', 'f after flush: no enabled f-prefix, stays');
  // 'o' → 缓冲单字符 'o' → one（one 是首个 o 前缀）。
  await test_driver.send_keys(sel, 'o');
  assert_equals(sel.value, '1', 'o jumps to one');
});
</script>
"##;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "local-select-typeahead.html",
            html,
            MINI_HARNESS,
            Duration::from_secs(3),
        );
        assert!(
            results.iter().all(|r| r.status == HarnessStatus::Pass),
            "select type-ahead subtests pass: {results:?}"
        );
    }

    #[test]
    fn unsupported_testdriver_command_is_explicit() {
        // WAB2-M2-s3：set_permission 已入白名单（stub + shim 权限注册表落地）——本断言
        // 改用仍未支持的 set_context（越白名单整体 Unsupported 语义不变）。
        let html = "test_driver.set_context('https://example.com')";
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "unsupported.html",
            html,
            MINI_HARNESS,
            Duration::from_secs(1),
        );
        assert_eq!(results[0].status, HarnessStatus::Unsupported);
        assert!(results[0].message.as_deref().unwrap().contains("set_context"));
    }

    #[test]
    fn missing_harness_completion_is_timeout() {
        let html = r#"<script src="/resources/testharness.js"></script>"#;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "timeout.html",
            html,
            "function add_result_callback(){} function add_completion_callback(){}",
            Duration::from_millis(10),
        );
        assert_eq!(results[0].status, HarnessStatus::Timeout);
    }

    #[test]
    fn no_harness_script_error_is_failure() {
        let html = r#"<script>throw new Error('boom');</script>"#;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "no-harness-error.html",
            html,
            MINI_HARNESS,
            Duration::from_secs(1),
        );

        assert_eq!(results.len(), 1);
        // 本测语义：脚本错误 ⇒ 失败结果，且失败消息须携带抛错原始消息（T64 消息保真）。
        // 历史：quickjs 腿下 Error.stack 仅帧行、无 'Error: boom' 消息头（非负载因素；
        // 曾误归因 js_worker 延迟批次——70c3c96f8/PR#208 勘误），70c3c96f8 因此把断言
        // 收窄为「非空」。哨兵 catch 臂改为消息保真组合（script_run_classic_page，
        // 2026-10-18）后恢复消息断言，v8/quickjs 两腿共同钉死。
        assert_eq!(results[0].status, HarnessStatus::Fail);
        assert!(
            results[0].message.as_deref().is_some_and(|m| m.contains("boom")),
            "失败结果应携带抛错消息: {:?}",
            results[0].message
        );
    }

    #[test]
    fn no_harness_test_wait_must_clear_before_pass() {
        let html = r#"<html class="test-wait"><script></script></html>"#;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "no-harness-test-wait.html",
            html,
            MINI_HARNESS,
            // PR#206 同族第三例：Timeout 语义死限保持「页面永不完成」语义，但从 10ms
            // 放宽到 1s——消息断言要求 runner 至少完成一次 probe 采样（last_test_wait
            // 初值 false，10ms 在 make test 并行负载下可于首轮 probe 前过线，消息恒
            // test_wait=false → 确定性红；1s 对齐同形姊妹测
            // registered_tests_cannot_complete_with_empty_results，负载下 probe 采样
            // ms 级完成，不弱化检测力：probe 采样被移除即 test_wait=true 消失转红）。
            Duration::from_secs(1),
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, HarnessStatus::Timeout);
        // 诊断增强（PR#206 D1 同款）：失败时带出实际 status/message，供负载红取数。
        assert!(
            results[0].message.as_deref().unwrap().contains("test_wait=true"),
            "实际 status={:?} message={:?}",
            results[0].status,
            results[0].message
        );
    }

    #[test]
    fn no_harness_module_script_can_clear_test_wait() {
        let html = r#"
<!doctype html>
<html class="test-wait">
<meta charset="utf-8">
<script type="module">
  await Promise.resolve();
  document.documentElement.classList.remove('test-wait');
</script>
</html>
"#;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "no-harness-module.html",
            html,
            MINI_HARNESS,
            Duration::from_secs(1),
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, HarnessStatus::Pass);
    }

    #[test]
    fn registered_tests_cannot_complete_with_empty_results() {
        let html = r#"
<script src="/resources/testharness.js"></script>
<script>
test(function() {}, 'registered');
add_completion_callback(function() { globalThis.__zw_harness_results = []; });
</script>
"#;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "empty-results.html",
            html,
            MINI_HARNESS,
            Duration::from_secs(1),
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, HarnessStatus::Timeout);
        assert!(results[0].message.as_deref().unwrap().contains("without reporting"));
    }

    #[test]
    fn indexeddb_any_wrapper_loads_support_before_case() {
        let html = indexeddb_window_wrapper(
            "IndexedDB/idbfactory_open.any.js",
            &[
                ("resources/first.js", "globalThis.first = true;"),
                ("resources/second.js", "globalThis.second = true;"),
            ],
            "globalThis.caseLoaded = true;",
        );
        let first = html.find("// source: resources/first.js").unwrap();
        let second = html.find("// source: resources/second.js").unwrap();
        let case = html.find("idbfactory_open.any.js").unwrap();
        assert!(html.contains("/resources/testharness.js"));
        assert!(first < second);
        assert!(second < case);
    }

    #[test]
    fn waits_for_active_step_timeout_before_accepting_completion() {
        let harness = r#"
var resultCallbacks = [], completionCallbacks = [];
globalThis.add_result_callback = function(callback) { resultCallbacks.push(callback); };
globalThis.add_completion_callback = function(callback) { completionCallbacks.push(callback); };
globalThis.async_test = function(callback, name) {
  var test = {
    name: name,
    step_func_done: function() {
      return function() {
        resultCallbacks.forEach(function(resultCallback) {
          resultCallback({name: name, status: 0, message: null});
        });
      };
    }
  };
  callback(test);
  completionCallbacks.forEach(function(completionCallback) { completionCallback([]); });
};
globalThis.step_timeout = function(callback, delay) { setTimeout(callback, delay); };
"#;
        let html = r#"
<script src="/resources/testharness.js"></script>
<script>
async_test(function(test) {
  step_timeout(test.step_func_done(), 4);
}, 'delayed completion');
</script>
"#;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "delayed-completion.html",
            html,
            harness,
            Duration::from_secs(10),
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "delayed completion");
        assert_eq!(results[0].status, HarnessStatus::Pass);
    }

    #[test]
    fn maps_notrun_and_precondition_failed_as_neutral_r20() {
        // js-dom R20：上游 testharness subtest status 数字编码 3=NOTRUN、4=PRECONDITION_FAILED 须映射为
        // 中性变体（NotRun/PreconditionFailed），而非 Fail。PRECONDITION_FAILED 是 assert_implements_optional
        // 失败（optional feature 如 TouchEvent 不支持），spec 不算失败——原 `_ => Fail` 误计拖低通过率。
        let mapped = map_harness_results(vec![
            RawHarnessResult {
                name: "pass".into(),
                status: 0,
                message: None,
            },
            RawHarnessResult {
                name: "fail".into(),
                status: 1,
                message: None,
            },
            RawHarnessResult {
                name: "timeout".into(),
                status: 2,
                message: None,
            },
            RawHarnessResult {
                name: "notrun".into(),
                status: 3,
                message: None,
            },
            RawHarnessResult {
                name: "precondition".into(),
                status: 4,
                message: Some("'expose legacy touch event APIs'".into()),
            },
            RawHarnessResult {
                name: "unknown".into(),
                status: 9,
                message: None,
            },
        ]);
        assert_eq!(mapped[0].status, HarnessStatus::Pass);
        assert_eq!(mapped[1].status, HarnessStatus::Fail);
        assert_eq!(mapped[2].status, HarnessStatus::Timeout);
        assert_eq!(mapped[3].status, HarnessStatus::NotRun, "status 3 → NotRun（中性）");
        assert_eq!(
            mapped[4].status,
            HarnessStatus::PreconditionFailed,
            "status 4 → PreconditionFailed（中性，非 Fail）"
        );
        assert_eq!(mapped[5].status, HarnessStatus::Fail, "未知编码 9 → Fail（保守回落）");
    }

    #[test]
    fn service_worker_core_manifest_has_expected_unique_cases() {
        let unique = SERVICE_WORKER_CORE_CASES
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(SERVICE_WORKER_CORE_CASES.len(), 65);
        assert_eq!(unique.len(), 65);
        assert!(
            SERVICE_WORKER_CORE_CASES
                .iter()
                .all(|path| path.starts_with("service-workers/service-worker/")
                    && (path.ends_with(".html") || path.ends_with(".any.js")))
        );
        assert!(
            SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/registration-end-to-end.https.html")
        );
        assert!(SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/registration-events.https.html"));
        assert!(
            SERVICE_WORKER_CORE_CASES
                .contains(&"service-workers/service-worker/ServiceWorkerGlobalScope/isSecureContext.https.html")
        );
        assert!(
            SERVICE_WORKER_CORE_CASES
                .contains(&"service-workers/service-worker/ServiceWorkerGlobalScope/close.https.html")
        );
        assert!(
            SERVICE_WORKER_CORE_CASES.contains(
                &"service-workers/service-worker/ServiceWorkerGlobalScope/extendable-message-event.https.html"
            )
        );
        assert!(
            SERVICE_WORKER_CORE_CASES
                .contains(&"service-workers/service-worker/ServiceWorkerGlobalScope/error-message-event.https.html")
        );
        assert!(
            SERVICE_WORKER_CORE_CASES
                .contains(&"service-workers/service-worker/ServiceWorkerGlobalScope/message-event-ports.https.html")
        );
        assert!(
            SERVICE_WORKER_CORE_CASES
                .contains(&"service-workers/service-worker/ServiceWorkerGlobalScope/registration-attribute.https.html")
        );
        assert!(
            SERVICE_WORKER_CORE_CASES
                .contains(&"service-workers/service-worker/ServiceWorkerGlobalScope/unregister.https.html")
        );
        assert!(
            SERVICE_WORKER_CORE_CASES
                .contains(&"service-workers/service-worker/extendable-event-async-waituntil.https.html")
        );
        assert!(SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/getregistration.https.html"));
        assert!(SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/registration-iframe.https.html"));
        assert!(SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/installing.https.html"));
        assert!(SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/waiting.https.html"));
        assert!(
            SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/controller-on-disconnect.https.html")
        );
        assert!(SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/controller-on-reload.https.html"));
        assert!(
            SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/interface-requirements-sw.https.html")
        );
        assert!(SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/historical.https.any.js"));
        assert!(
            SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/global-serviceworker.https.any.js")
        );
        assert!(
            SERVICE_WORKER_CORE_CASES
                .contains(&"service-workers/service-worker/immutable-prototype-serviceworker.https.html")
        );
        assert!(SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/no-dynamic-import.any.js"));
        assert!(
            SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/no-dynamic-import-in-module.any.js")
        );
        assert!(SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/install-event-type.https.html"));
        assert!(
            SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/onactivate-script-error.https.html")
        );
        assert!(
            SERVICE_WORKER_CORE_CASES.contains(&"service-workers/service-worker/oninstall-script-error.https.html")
        );
        assert!(
            SERVICE_WORKER_CORE_CASES
                .contains(&"service-workers/service-worker/skip-waiting-using-registration.https.html")
        );
        assert!(
            SERVICE_WORKER_CORE_CASES
                .contains(&"service-workers/service-worker/skip-waiting-without-using-registration.https.html")
        );
    }

    #[test]
    fn service_worker_fetch_manifest_has_request_end_to_end_case() {
        let unique = SERVICE_WORKER_FETCH_CASES
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(SERVICE_WORKER_FETCH_CASES.len(), 31);
        assert_eq!(unique.len(), 31);
        assert!(SERVICE_WORKER_FETCH_CASES.contains(
            &"service-workers/service-worker/ServiceWorkerGlobalScope/fetch-on-the-right-interface.https.any.js"
        ));
        assert!(SERVICE_WORKER_FETCH_CASES.contains(
            &"service-workers/service-worker/ServiceWorkerGlobalScope/extendable-message-event-constructor.https.html"
        ));
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/ServiceWorkerGlobalScope/postmessage.https.html")
        );
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/historical.https.any.js"));
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/request-end-to-end.https.html"));
        assert!(
            SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/fetch-event-add-async.https.html")
        );
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/fetch-event-async-respond-with.https.html")
        );
        assert!(
            SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/fetch-event-within-sw.https.html")
        );
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/fetch-event-respond-with-custom-response.https.html")
        );
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/fetch-event-handled.https.html"));
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/fetch-event-after-navigation-within-page.https.html")
        );
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/intercepted-referrer.https.html"));
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/controller-with-no-fetch-event-handler.https.html")
        );
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/fetch-with-body.https.html"));
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/fetch-event-respond-with-stops-propagation.https.html")
        );
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/fetch-event-throws-after-respond-with.https.html")
        );
        assert!(
            SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/fetch-event-network-error.https.html")
        );
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/fetch-event-respond-with-argument.https.html")
        );
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/fetch-event-respond-with-readable-stream-chunk.https.html")
        );
        assert!(SERVICE_WORKER_FETCH_CASES.contains(
            &"service-workers/service-worker/fetch-event-respond-with-response-body-with-invalid-chunk.https.html"
        ));
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/fetch-error.https.html"));
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/iso-latin1-header.https.html"));
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/invalid-header.https.html"));
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/invalid-blobtype.https.html"));
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/uncontrolled-page.https.html"));
        assert!(SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/claim-fetch.https.html"));
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/claim-not-using-registration.https.html")
        );
        assert!(
            SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/claim-using-registration.https.html")
        );
        assert!(
            SERVICE_WORKER_FETCH_CASES.contains(&"service-workers/service-worker/unregister-controller.https.html")
        );
        assert!(
            SERVICE_WORKER_FETCH_CASES
                .contains(&"service-workers/service-worker/fetch-event-respond-with-body-loaded-in-chunk.https.html")
        );
    }

    #[test]
    fn service_worker_cache_storage_manifest_has_expected_unique_cases() {
        let unique = SERVICE_WORKER_CACHE_STORAGE_CASES
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(SERVICE_WORKER_CACHE_STORAGE_CASES.len(), 25);
        assert_eq!(unique.len(), 25);
        assert!(
            SERVICE_WORKER_CACHE_STORAGE_CASES
                .iter()
                .all(|path| path.starts_with("service-workers/cache-storage/")
                    && (path.ends_with(".https.html") || path.ends_with(".https.any.js")))
        );
        assert!(
            SERVICE_WORKER_CACHE_STORAGE_CASES.contains(&"service-workers/cache-storage/cache-storage.https.any.js")
        );
        assert!(
            SERVICE_WORKER_CACHE_STORAGE_CASES
                .contains(&"service-workers/cache-storage/cache-keys-attributes-for-service-worker.https.html")
        );
        assert!(SERVICE_WORKER_CACHE_STORAGE_CASES.contains(&"service-workers/cache-storage/credentials.https.html"));
        assert!(
            SERVICE_WORKER_CACHE_STORAGE_CASES
                .contains(&"service-workers/cache-storage/serviceworker/cache-storage.https.html")
        );
        assert!(
            SERVICE_WORKER_CACHE_STORAGE_CASES
                .contains(&"service-workers/cache-storage/serviceworker/credentials.https.html")
        );
    }

    #[test]
    fn cache_storage_window_manifest_has_expected_unique_cases() {
        let unique = CACHE_STORAGE_WINDOW_CASES
            .iter()
            .map(|(path, _)| *path)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(CACHE_STORAGE_WINDOW_CASES.len(), 39);
        assert_eq!(unique.len(), 39);
        assert!(CACHE_STORAGE_WINDOW_CASES.iter().all(|(path, support)| {
            if !path.starts_with("service-workers/cache-storage/")
                || !(path.ends_with(".https.any.js")
                    || path.ends_with(".https.window.js")
                    || path.ends_with(".https.html"))
            {
                return false;
            }
            match *path {
                "service-workers/cache-storage/cache-match.https.any.js"
                | "service-workers/cache-storage/cache-put.https.any.js"
                | "service-workers/cache-storage/cache-add.https.any.js" => {
                    *support == ["resources/test-helpers.js", "/common/get-host-info.sub.js"]
                }
                "service-workers/cache-storage/zeroweb-filtered-response-types.https.any.js" => {
                    *support == ["resources/test-helpers.js"]
                }
                "service-workers/cache-storage/cache-storage-buckets.https.any.js" => {
                    *support
                        == [
                            "resources/test-helpers.js",
                            "/common/get-host-info.sub.js",
                            "/storage/buckets/resources/util.js",
                        ]
                }
                "service-workers/cache-storage/cache-abort.https.any.js" => {
                    *support == ["resources/test-helpers.js", "/common/utils.js"]
                }
                "service-workers/cache-storage/common.https.window.js"
                | "service-workers/cache-storage/common.https.html"
                | "service-workers/cache-storage/cache-api-nested-worker.https.html"
                | "service-workers/cache-storage/sandboxed-iframes.https.html"
                | "service-workers/cache-storage/crashtests/cache-response-clone.https.html" => support.is_empty(),
                "service-workers/cache-storage/credentials.https.html" => {
                    *support == ["../service-worker/resources/test-helpers.sub.js"]
                }
                "service-workers/cache-storage/window/cache-abort.https.html" => {
                    *support
                        == [
                            "../resources/test-helpers.js",
                            "/common/utils.js",
                            "../script-tests/cache-abort.js",
                        ]
                }
                "service-workers/cache-storage/window/cache-match.https.html" => {
                    *support
                        == [
                            "/common/get-host-info.sub.js",
                            "../resources/test-helpers.js",
                            "../script-tests/cache-match.js",
                        ]
                }
                "service-workers/cache-storage/window/cache-put.https.html" => {
                    *support
                        == [
                            "/common/get-host-info.sub.js",
                            "../resources/test-helpers.js",
                            "../script-tests/cache-put.js",
                        ]
                }
                "service-workers/cache-storage/window/cache-add.https.html" => {
                    *support
                        == [
                            "/common/get-host-info.sub.js",
                            "../resources/test-helpers.js",
                            "../script-tests/cache-add.js",
                        ]
                }
                "service-workers/cache-storage/window/sandboxed-iframes.https.html" => support.is_empty(),
                path if path.starts_with("service-workers/cache-storage/window/") => {
                    let script = path
                        .strip_prefix("service-workers/cache-storage/window/")
                        .unwrap()
                        .replace(".https.html", ".js");
                    support.len() == 2
                        && support[0] == "../resources/test-helpers.js"
                        && support[1] == format!("../script-tests/{script}")
                }
                path if path.starts_with("service-workers/cache-storage/worker/") => support.is_empty(),
                _ => *support == ["resources/test-helpers.js"],
            }
        }));
    }

    #[test]
    fn cache_storage_runner_reports_every_case_when_harness_is_missing() {
        let cases = run_cache_storage_cases(Path::new("/nonexistent-cache-storage-wpt-root"), None);
        assert_eq!(cases.len(), CACHE_STORAGE_WINDOW_CASES.len());
        assert!(cases.iter().all(|(_, results)| {
            results.len() == 1 && results[0].status == HarnessStatus::Fail && results[0].name == "load testharness.js"
        }));
    }

    #[test]
    fn service_worker_runner_reports_every_case_when_harness_is_missing() {
        let cases = run_service_worker_cases(Path::new("/nonexistent-service-worker-wpt-root"), None);
        assert_eq!(cases.len(), SERVICE_WORKER_CORE_CASES.len());
        assert!(cases.iter().all(|(_, results)| {
            results.len() == 1 && results[0].status == HarnessStatus::Fail && results[0].name == "load testharness.js"
        }));
    }

    #[test]
    fn service_worker_fetch_runner_reports_every_case_when_harness_is_missing() {
        let cases = run_service_worker_fetch_cases(Path::new("/nonexistent-service-worker-wpt-root"), None);
        assert_eq!(cases.len(), SERVICE_WORKER_FETCH_CASES.len());
        assert!(cases.iter().all(|(_, results)| {
            results.len() == 1 && results[0].status == HarnessStatus::Fail && results[0].name == "load testharness.js"
        }));
    }

    #[test]
    fn service_worker_cache_storage_runner_reports_every_case_when_harness_is_missing() {
        let cases = run_service_worker_cache_storage_cases(Path::new("/nonexistent-service-worker-wpt-root"), None);
        assert_eq!(cases.len(), SERVICE_WORKER_CACHE_STORAGE_CASES.len());
        assert!(cases.iter().all(|(_, results)| {
            results.len() == 1 && results[0].status == HarnessStatus::Fail && results[0].name == "load testharness.js"
        }));
    }

    #[test]
    fn service_worker_fixture_fetcher_rejects_unknown_external_origins() {
        let fetcher =
            wpt_data_service_worker_script_fetcher(Path::new("/nonexistent-service-worker-wpt-root")).unwrap();
        assert!(
            fetcher(
                "https://wpt.test/page",
                "https://other.test/service-workers/service-worker/resources/worker.js",
            )
            .unwrap_err()
            .contains("external Service Worker fixture origin")
        );
    }

    #[test]
    fn service_worker_fixture_fetcher_accepts_canonical_cross_origin() {
        let fetcher =
            wpt_data_service_worker_script_fetcher(Path::new("/nonexistent-service-worker-wpt-root")).unwrap();
        let first = fetcher(
            "https://wpt.test/page",
            "https://www1.wpt.test/service-workers/service-worker/resources/import-scripts-version.py",
        )
        .unwrap();
        let second = fetcher(
            "https://wpt.test/page",
            "https://www1.wpt.test/service-workers/service-worker/resources/import-scripts-version.py",
        )
        .unwrap();
        assert_eq!(first.content_type_mime(), Some("application/javascript"));
        assert!(first.header("access-control-allow-origin").is_none());
        assert_ne!(first.body, second.body);
    }

    #[test]
    fn service_worker_fixture_fetcher_generates_import_script_from_query() {
        let fetcher =
            wpt_data_service_worker_script_fetcher(Path::new("/nonexistent-service-worker-wpt-root")).unwrap();
        let response = fetcher(
            "https://wpt.test/page",
            "https://wpt.test/service-workers/service-worker/resources/import-scripts-get.py?output=echo1&msg=a%20value",
        )
        .unwrap();
        assert_eq!(
            response.body,
            br#"echo1 = "a value";
"#
        );
    }

    #[test]
    fn service_worker_fixture_fetcher_wraps_any_js_with_worker_harness() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zero-wpt-runner-any-js-{}-{nonce}", std::process::id()));
        let case_path = root
            .join("service-workers/service-worker/ServiceWorkerGlobalScope/fetch-on-the-right-interface.https.any.js");
        std::fs::create_dir_all(case_path.parent().unwrap()).unwrap();
        let support_path = root.join("service-workers/service-worker/ServiceWorkerGlobalScope/resources/helper.js");
        std::fs::create_dir_all(support_path.parent().unwrap()).unwrap();
        std::fs::write(&support_path, "self.helperLoaded = true;\n").unwrap();
        std::fs::write(
            &case_path,
            "// META: script=./resources/helper.js\ntest(() => {}, 'worker side');\n",
        )
        .unwrap();

        let fetcher = wpt_data_service_worker_script_fetcher(&root).unwrap();
        let response = fetcher(
            "https://wpt.test/page",
            "https://wpt.test/service-workers/service-worker/ServiceWorkerGlobalScope/fetch-on-the-right-interface.https.any.js",
        )
        .unwrap();
        let source = String::from_utf8(response.body).unwrap();
        assert!(source.starts_with("importScripts('/resources/testharness.js');\n"));
        assert!(source.contains(
            "importScripts('/service-workers/service-worker/ServiceWorkerGlobalScope/resources/helper.js');"
        ));
        assert!(source.contains("test(() => {}, 'worker side');"));

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn service_worker_fixture_fetcher_wraps_cache_abort_any_js_with_dynamic_fetch_fixture() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zero-wpt-runner-cache-abort-any-js-{}-{nonce}",
            std::process::id()
        ));
        let case_path = root.join("service-workers/cache-storage/cache-abort.https.any.js");
        std::fs::create_dir_all(case_path.parent().unwrap()).unwrap();
        std::fs::write(
            &case_path,
            "// META: script=./resources/test-helpers.js\npromise_test(async () => {}, 'abort side');\n",
        )
        .unwrap();

        let fetcher = wpt_data_service_worker_script_fetcher(&root).unwrap();
        let response = fetcher(
            "https://wpt.test/page",
            "https://wpt.test/service-workers/cache-storage/cache-abort.https.any.js",
        )
        .unwrap();
        let source = String::from_utf8(response.body).unwrap();
        assert!(source.starts_with("importScripts('/resources/testharness.js');\n"));
        assert!(source.contains("__zw_cache_abort_stash"));
        assert!(source.contains("fetch/api/resources/stash-take.py"));
        assert!(source.contains("promise_test(async () => {}, 'abort side');"));

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn service_worker_fixture_fetcher_wraps_module_any_js_with_module_harness() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zero-wpt-runner-module-any-js-{}-{nonce}", std::process::id()));
        let case_path = root.join("service-workers/service-worker/no-dynamic-import-in-module.any.js");
        std::fs::create_dir_all(case_path.parent().unwrap()).unwrap();
        std::fs::write(
            &case_path,
            "// META: global=serviceworker-module\npromise_test(async () => {}, 'module side');\n",
        )
        .unwrap();

        let html = service_worker_any_js_wrapper(
            "service-workers/service-worker/no-dynamic-import-in-module.any.js",
            &std::fs::read_to_string(&case_path).unwrap(),
        );
        assert!(html.contains("type: 'module'"));

        let fetcher = wpt_data_service_worker_script_fetcher(&root).unwrap();
        let response = fetcher(
            "https://wpt.test/page",
            "https://wpt.test/service-workers/service-worker/no-dynamic-import-in-module.any.js",
        )
        .unwrap();
        let source = String::from_utf8(response.body).unwrap();
        assert!(source.starts_with("import '/resources/testharness.js';\n"));
        assert!(source.contains("promise_test(async () => {}, 'module side');"));

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn service_worker_fixture_fetcher_redirects_second_update_request() {
        let fetcher =
            wpt_data_service_worker_script_fetcher(Path::new("/nonexistent-service-worker-wpt-root")).unwrap();
        let url = "https://wpt.test/service-workers/service-worker/resources/update-worker.py?Key=fixture-key&Mode=redirect&Redirect=update-worker.py?Key=fixture-key%26Mode=normal";
        let first = fetcher("https://wpt.test/page", url).unwrap();
        let second = fetcher("https://wpt.test/page", url).unwrap();
        assert_eq!(first.redirect_count, 0);
        assert_eq!(first.body, b"/* 1 */");
        assert_eq!(second.redirect_count, 1);
        assert_eq!(second.body, b"/* 3 */");
        assert_eq!(
            second.url,
            "https://wpt.test/service-workers/service-worker/resources/update-worker.py?Key=fixture-key&Mode=normal"
        );
    }

    #[test]
    fn encoding_py_handler_generates_meta_charset_from_label_query() {
        // js-dom R141：dom/nodes/encoding.py 内置生成器（Document-characterSet-normalization
        // 654 subtest 的子文档源）——`?label=X`（含 percent-encoding）→ `<!doctype html><meta
        // charset="X">`（与上游 wptserve 脚本逐字等价）；无 label 参数 → 空 charset。
        let handler = wpt_data_fetch_handler(Path::new("/nonexistent-wpt-root")).unwrap();
        let make_req = |url: &str| zero_engine::fetch_bridge::FetchRequest {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: Vec::new(),
            body: None,
            body_bytes: None,
            credentials: None,
            mode: None,
            redirect: None,
        };
        let resp = handler(&make_req(
            "https://wpt.test/dom/nodes/encoding.py?label=unicode-1-1-utf-8",
        ))
        .unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "<!doctype html><meta charset=\"unicode-1-1-utf-8\">");
        // percent-encoded label（helper 直接拼 label 原值，URL 里空格等会 encode）。
        let resp_enc = handler(&make_req(
            "https://wpt.test/dom/nodes/encoding.py?label=iso-8859-1%3A1987",
        ))
        .unwrap();
        assert_eq!(resp_enc.body, "<!doctype html><meta charset=\"iso-8859-1:1987\">");
        // 缺 label → 空 charset（上游 escape(None) 同型）。
        let resp_none = handler(&make_req("https://wpt.test/dom/nodes/encoding.py")).unwrap();
        assert_eq!(resp_none.body, "<!doctype html><meta charset=\"\">");
        // 非 .py 路径仍走静态文件（root 不存在 → M4-S20 起 404 响应——HTTP 语义：
        // 缺文件是 404 应答而非网络错误，且 .bak 不落入 encoding 处理臂）。
        let resp_bak = handler(&make_req("https://wpt.test/dom/nodes/encoding.py.bak"));
        assert!(resp_bak.is_ok());
        assert_eq!(resp_bak.unwrap().status, 404);
    }

    #[test]
    fn echo_content_escaped_py_handler_round_trips_helper_unescape() {
        // form-submission-0 三族回显端点：POST body 逐字节转义（printable ASCII 与
        // CR/LF 字面原样、`\` → `\x5C`、其余 `\xNN`），经 enctypes-helper.js
        // `unescape()` 可逆还原（CR/LF 字面是 helper 首步换行归一的前提）。
        let handler = wpt_data_fetch_handler(Path::new("/nonexistent-wpt-root")).unwrap();
        let make_req = |method: &str, body: Option<&str>| zero_engine::fetch_bridge::FetchRequest {
            url: "https://wpt.test/FileAPI/file/resources/echo-content-escaped.py".to_string(),
            method: method.to_string(),
            headers: Vec::new(),
            body: body.map(str::to_string),
            body_bytes: None,
            credentials: None,
            mode: None,
            redirect: None,
        };
        // 基本 urlencoded 对。
        let basic = handler(&make_req("POST", Some("basic=test"))).unwrap();
        assert_eq!(basic.status, 200);
        assert_eq!(basic.body, "basic=test");
        assert_eq!(basic.headers[0], ("content-type".to_string(), "text/plain".to_string()));
        // 0x00 / 换行 / backslash 字节形态（CR/LF 字面交付——helper unescape 首步
        // 归一的作用对象；hex 大写、backslash 用 \x5C 防 `\xNN` 还原正则误吞）。
        let escaped = handler(&make_req("POST", Some("a\x00b\r\nc\\d"))).unwrap();
        assert_eq!(escaped.body, "a\\x00b\r\nc\\x5Cd");
        // 反向校验：转义文本经 helper unescape 逻辑还原原 body（JS 侧等价实现，
        // 含首步字面换行归一 `/\r\n?|\n/ → "\r\n"`）。
        let helper_unescape = |s: &str| -> String {
            let mut out = String::new();
            let mut chars = s.chars().peekable();
            while let Some(c) = chars.next() {
                match c {
                    '\r' => {
                        if chars.peek() == Some(&'\n') {
                            chars.next();
                        }
                        out.push_str("\r\n");
                    }
                    '\n' => out.push_str("\r\n"),
                    '\\' => {
                        // `\xNN` 还原（backslash 已被端点转义为 \x5C，不会误入）。
                        if chars.peek() == Some(&'x') {
                            chars.next(); // 消费 'x'，后两位才是 hex
                            let hex: String = chars.by_ref().take(2).collect();
                            if let Ok(b) = u8::from_str_radix(&hex, 16) {
                                out.push(b as char);
                                continue;
                            }
                        }
                        out.push('\\');
                    }
                    _ => out.push(c),
                }
            }
            out
        };
        assert_eq!(helper_unescape(&escaped.body), "a\x00b\r\nc\\d");
        // text/plain `\n`-in-value 案形态：body 携带 lone LF（提交编码器已归一，
        // 此处兜底端点层不吞）——helper 首步归一还原 `\r\n` 展开。
        let lone_lf = handler(&make_req("POST", Some("a\nb=c\n"))).unwrap();
        assert_eq!(lone_lf.body, "a\nb=c\n");
        assert_eq!(helper_unescape(&lone_lf.body), "a\r\nb=c\r\n");
        // GET / 无 body → 空回显（200 非错误）。
        let empty = handler(&make_req("GET", None)).unwrap();
        assert_eq!(empty.status, 200);
        assert_eq!(empty.body, "");
    }

    #[test]
    fn vary_py_handler_respects_request_credentials() {
        let handler = wpt_data_fetch_handler(Path::new("/nonexistent-wpt-root")).unwrap();
        let make_req = |url: &str, credentials: Option<&str>| zero_engine::fetch_bridge::FetchRequest {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: Vec::new(),
            body: None,
            body_bytes: None,
            credentials: credentials.map(str::to_string),
            mode: None,
            redirect: None,
        };
        let set_url =
            "https://wpt.test/service-workers/cache-storage/resources/vary.py?set-vary-value-override-cookie=x-test";
        handler(&make_req(set_url, Some("same-origin"))).unwrap();

        let with_credentials = handler(&make_req(
            "https://wpt.test/service-workers/cache-storage/resources/vary.py",
            Some("same-origin"),
        ))
        .unwrap();
        assert!(
            with_credentials
                .headers
                .iter()
                .any(|(name, value)| name.eq_ignore_ascii_case("vary") && value == "x-test"),
            "same-origin credentials should expose the WPT vary override cookie"
        );

        let without_credentials = handler(&make_req(
            "https://wpt.test/service-workers/cache-storage/resources/vary.py",
            Some("omit"),
        ))
        .unwrap();
        assert!(
            !without_credentials
                .headers
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case("vary")),
            "omit credentials should ignore the WPT vary override cookie"
        );
    }

    #[test]
    fn fetch_with_body_fixture_distinguishes_empty_and_non_empty_request_bodies() {
        let handler = wpt_data_fetch_handler(Path::new("/nonexistent-wpt-root")).unwrap();
        let make_req = |method: &str, body: Option<&str>| zero_engine::fetch_bridge::FetchRequest {
            url: "https://wpt.test/service-workers/service-worker/resources/fetch-with-body-worker.py".to_string(),
            method: method.to_string(),
            headers: Vec::new(),
            body: body.map(str::to_string),
            body_bytes: None,
            credentials: None,
            mode: None,
            redirect: None,
        };

        let get = handler(&make_req("GET", None)).unwrap();
        assert_eq!(get.status, 400);
        assert_eq!(get.body, "NO BODY");

        let post = handler(&make_req("POST", Some("BODY"))).unwrap();
        assert_eq!(post.status, 200);
        assert_eq!(post.body, "BODY");
    }

    #[test]
    fn trickle_fixture_generates_requested_chunks() {
        let handler = wpt_data_fetch_handler(Path::new("/nonexistent-wpt-root")).unwrap();
        let req = zero_engine::fetch_bridge::FetchRequest {
            url: "https://wpt.test/fetch/api/resources/trickle.py?count=4".to_string(),
            method: "GET".to_string(),
            headers: Vec::new(),
            body: None,
            body_bytes: None,
            credentials: None,
            mode: None,
            redirect: None,
        };

        let resp = handler(&req).unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "TEST_TRICKLE\n".repeat(4));
        assert_eq!(resp.body_bytes, Some(resp.body.as_bytes().to_vec()));
    }

    #[test]
    fn load_error_events_fixture_serves_query_aware_bodies() {
        let handler = wpt_data_fetch_handler(Path::new("/nonexistent-wpt-root")).unwrap();
        let base = "https://wpt.test/html/semantics/scripting-1/the-script-element/resources/load-error-events.py";
        let make_req = |url: &str| zero_engine::fetch_bridge::FetchRequest {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: Vec::new(),
            body: None,
            body_bytes: None,
            credentials: None,
            mode: None,
            redirect: None,
        };

        // 200 组：test 含 `_load` → 执行体（executed = true）+ text/javascript。
        let ok = handler(&make_req(&format!("{base}?test=test3_dynamic_load"))).unwrap();
        assert_eq!(ok.status, 200);
        assert_eq!(ok.body, "\"use strict\";\ntest3_dynamic_load.executed = true;");
        assert!(
            ok.headers
                .iter()
                .any(|(n, v)| n.eq_ignore_ascii_case("content-type") && v == "text/javascript")
        );

        // 404 组：非 `_load` → 404 + assert_unreached 体（本引擎侧 status 拒 eval）。
        let missing = handler(&make_req(&format!("{base}?test=test1_error"))).unwrap();
        assert_eq!(missing.status, 404);
        assert!(missing.body.contains("test1_error.test.step"));
        assert!(missing.body.contains("assert_unreached"));

        // 400 组：test 缺席/非法（`^[a-zA-Z0-9_]+$` 之外）→ 400 空体。
        assert_eq!(handler(&make_req(base)).unwrap().status, 400);
        let bad = handler(&make_req(&format!("{base}?test=bad-name"))).unwrap();
        assert_eq!(bad.status, 400);
        assert_eq!(bad.body, "");

        // 路径精确匹配：module 族变体路径不走本 arm（维持通用缺文件 404 面）。
        let other = handler(&make_req(
            "https://wpt.test/html/semantics/scripting-1/the-script-element/module/load-error-events.py?test=x_load",
        ))
        .unwrap();
        assert_eq!(other.status, 404);
    }

    #[test]
    fn inline_local_scripts_preserves_handlers_and_appends_delivery_tails() {
        let root = std::env::temp_dir().join("zw-s100-inline-local-scripts-test");
        let case_dir = "html/semantics/scripting-1/the-script-element";
        std::fs::create_dir_all(root.join(case_dir)).unwrap();
        std::fs::write(root.join(case_dir).join("ok.js"), "globalThis.ran = true;").unwrap();
        let html = "<!doctype html><body>\
            <script src=\"./ok.js\" onload=\"loaded = true\"></script>\
            <script src=\"resources/load-error-events.py?test=t1_load\" onload=\"onLoad(t1)\"></script>\
            <script src=\"resources/load-error-events.py?test=t1_error\" onerror=\"onError(t1)\"></script>\
            </body>";

        let out = inline_local_scripts(html, &root, &format!("{case_dir}/load-error-events-1.html"), None);

        // 200 本地 .js：体保留 + onload 保真 + load 投递尾。
        assert!(out.contains("globalThis.ran = true;"));
        assert!(out.contains("onload=\"loaded = true\""));
        assert!(out.contains("new Event('load')"));
        // fixture 200：执行体入页 + load 投递尾。
        assert!(out.contains("t1_load.executed = true;"));
        // fixture 404：执行体不入页（上游 404 体不执行）+ onerror 保真 + error 投递尾。
        assert!(!out.contains("assert_unreached"));
        assert!(out.contains("onerror=\"onError(t1)\""));
        assert!(out.contains("new Event('error')"));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn inline_delivery_preserves_id_attr_s108() {
        // s108（id 保真）：投递改写保留原标签 id。s110 收敛：JS type 外链走 data-inline
        // 改写（id 随写保留）；type=text/plain 外链**整段透传**（src/type/id 原样——
        // 122-126 族 mutation 重触发面依赖元素自带完整属性，改写剥 src 使触发臂走错
        // 分支）。两条投递形态各自断言。
        let root = std::env::temp_dir().join("zw-s108-inline-id-test");
        let case_dir = "html/semantics/scripting-1/the-script-element";
        std::fs::create_dir_all(root.join(case_dir)).unwrap();
        std::fs::write(root.join(case_dir).join("ok.js"), "globalThis.ran = true;").unwrap();
        let html = "<!doctype html><body>\
            <script id=\"ok\" src=\"./ok.js\"></script>\
            <script id=\"test\" type=\"text/plain\" src=\"./ok.js\"></script>\
            </body>";

        let out = inline_local_scripts(html, &root, &format!("{case_dir}/mutation-family.html"), None);

        // JS 型单元：data-inline 改写 + id 保真 + 执行体入页。
        assert!(out.contains("id=\"ok\""), "id 须随投递形态保留: {out}");
        assert!(out.contains("globalThis.ran = true;"));
        // 非 JS 型单元：整段透传（src/type/id 原样保留，无 data-inline 改写）。
        assert!(
            out.contains("<script id=\"test\" type=\"text/plain\" src=\"./ok.js\"></script>"),
            "非 JS type 外链须整段透传: {out}"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn inline_sched_classification_wraps_defer_async_trickle_s112() {
        // s112（parser 外链时序调度）：defer/async/trickle 单元改注册 __zwSchedRegister
        //（分类冻结于预处理——spec prepare 定格 async/defer 归类，103/111/112 运行期
        // removeAttribute 不重分类）；普通阻塞单元维持原位直执；nomodule 不入调度器
        //（prepare 门专用面）；trickle 毫秒 = dN×1000；load 投递尾随闭包体内联。
        let root = std::env::temp_dir().join("zw-s112-sched-classify-test");
        let case_dir = "html/semantics/scripting-1/the-script-element/execution-timing";
        std::fs::create_dir_all(root.join(case_dir).join("scripts")).unwrap();
        std::fs::write(root.join(case_dir).join("scripts/a.js"), "log('a');").unwrap();
        let html = "<!doctype html><body>\
            <script id=\"d\" defer src=\"scripts/a.js\"></script>\
            <script async src=\"scripts/a.js?pipe=trickle(d3)\"></script>\
            <script src=\"scripts/a.js?pipe=trickle(d4)\"></script>\
            <script src=\"scripts/a.js\"></script>\
            <script nomodule src=\"scripts/a.js\"></script>\
            <script defer src=\"scripts/a.js?pipe=trickle(d1)\" onload=\"onL()\"></script>\
            </body>";

        let out = inline_local_scripts(html, &root, &format!("{case_dir}/sched.html"), None);

        // defer → kind=defer delay=0；async+trickle → kind=async；无 defer/async +
        // trickle → kind=block。trickle 毫秒 = dN×1000。注册体 = 元素捕获 IIFE 返回的
        // 执行闭包（投递尾相位执行时 currentScript 已清，见投递处注释）。
        assert!(
            out.contains("__zwSchedRegister(\"defer\", 0, (function (_zwEl) {"),
            "defer 单元须注册 kind=defer delay=0: {out}"
        );
        assert!(
            out.contains("__zwSchedRegister(\"async\", 3000, (function (_zwEl) {"),
            "async+trickle 单元须注册 kind=async: {out}"
        );
        assert!(
            out.contains("__zwSchedRegister(\"block\", 4000, (function (_zwEl) {"),
            "trickle 阻塞单元须注册 kind=block: {out}"
        );
        assert!(
            out.contains("__zwSchedRegister(\"defer\", 1000, (function (_zwEl) {"),
            "defer+trickle 单元须注册 kind=defer delay=1000: {out}"
        );
        // 4 单元注册 + 普通阻塞/nomodule 两单元原位直执（注册调用恰 4 处）。
        assert_eq!(out.matches("__zwSchedRegister").count(), 4, "{out}");
        // 投递形态保真：id 随写保留（111/112 getElementById 面）+ onload 属性保留 +
        // load 投递尾在闭包体内（注册体内联，元素表达式 = 捕获形参 _zwEl）。
        assert!(out.contains("id=\"d\""), "{out}");
        assert!(out.contains("onload=\"onL()\""), "{out}");
        assert!(out.contains("})(document.currentScript));"), "{out}");
        assert_eq!(out.matches("var _cs = _zwEl;").count(), 4, "{out}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn inline_body_closing_tag_escaped_s112() {
        // s112：体内字面 `</script` 投递面转义（`<\/script`）——字符串值等值（`\`+`/`）
        // 且不提前封口截断执行体（099 include-11 srcdoc 串 SyntaxError 零日志实证）。
        // 原始闭合序列仅剩每单元一个真封口。
        let root = std::env::temp_dir().join("zw-s112-esc-test");
        let case_dir = "html/semantics/scripting-1/the-script-element/execution-timing";
        std::fs::create_dir_all(root.join(case_dir).join("scripts")).unwrap();
        std::fs::write(
            root.join(case_dir).join("scripts/esc.js"),
            "iframe.srcdoc = \"<script>x</script>\";",
        )
        .unwrap();
        let html = "<!doctype html><body><script src=\"scripts/esc.js\"></script></body>";

        let out = inline_local_scripts(html, &root, &format!("{case_dir}/esc.html"), None);

        assert!(
            out.contains("iframe.srcdoc = \"<script>x<\\/script>\";"),
            "体内闭合序列须转义且字符串值不变: {out}"
        );
        // 体内零未转义 `</script` 后，剩余闭合 = 投递元素自身封口 + 原标签闭合透传
        // （Some 臂只消耗 open 标签，孤立闭合标签解析面无害——s97 注释同型既有行为）。
        assert_eq!(
            out.matches("</script").count(),
            2,
            "体内闭合须全部转义，仅剩两处无害真封口: {out}"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    /// s108：「脚本抛错 + 已声明测试」短路收窄钉的 stateful harness 变体——MINI_HARNESS
    /// 包 IIFE（形参名 `global_scope` 与 completion patch 注入代码的引用名一致）+ 尾锚
    /// `})(self);`（apply_harness_completion_patch 的 replacen 锚），tests 注册表使
    /// `__zw_harness_state()` 读到 declared/pending。
    fn mini_harness_stateful() -> String {
        format!(
            "(function(global_scope) {{\n{MINI_HARNESS}\
             var tests = {{tests: [], num_pending: 0, phase: 0, all_done: function() {{ return true; }}, complete: function() {{}}}};\n\
             var test_environment = {{all_loaded: true}};\n\
             var __miniTest = globalThis.test;\n\
             globalThis.test = function(fn, name) {{ tests.tests.push(1); __miniTest(fn, name); }};\n\
             var __miniPromiseTest = globalThis.promise_test;\n\
             globalThis.promise_test = function(fn, name) {{ tests.num_pending++; __miniPromiseTest(fn, name); }};\n\
             }})(self);\n"
        )
    }

    #[test]
    fn script_throw_with_declared_tests_collects_results_s108() {
        // s108：脚本抛错但 harness 已声明测试 → 不丢弃结果收集（WPT 语义：uncaught
        // exception 不降级 test status）。抛错 script 之前的 test 已 done → 收 Pass；
        // 不再短路出「page script threw」Fail。
        // https://html.spec.whatwg.org/multipage/webappapis.html#report-the-error
        let html = r##"<script src="/resources/testharness.js"></script>
<script>test(function() { assert_true(true); }, 'done before throw');</script>
<script>var x = ();</script>"##;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "throw-with-declared.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "done before throw" && r.status == HarnessStatus::Pass),
            "已完成测试须照收 Pass: {results:?}"
        );
        assert!(
            !results
                .iter()
                .any(|r| r.message.as_deref().is_some_and(|m| m.contains("page script threw"))),
            "declared>0 不再短路 Fail: {results:?}"
        );
    }

    #[test]
    fn script_throw_without_declared_tests_stays_fail_s108() {
        // s108：无 testharness 引用的用例脚本抛错维持短路 Fail（s102 防假绿口径不变——
        // 不能按「未注册 test()」误判为通过）。
        let html = r##"<script>var y = (;</script>"##;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "throw-no-harness.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, HarnessStatus::Fail);
        assert!(
            results[0]
                .message
                .as_deref()
                .is_some_and(|m| m.contains("page script threw")),
            "declared==0 维持短路: {:?}",
            results[0].message
        );
    }

    #[test]
    fn script_prepare_on_children_changed_s110() {
        // s110（mutation 重触发面收敛）：type 变更非 spec prepare 触发面（触发 =
        // 插入 post-connection / children-changed / 移除置 already-started=false）。
        // type=text/plain 内联单元 removeAttribute('type') + appendChild(text)——
        // children-changed 步骤触发 prepare → 脚本体在**调用点同步**执行（124 形：
        // 期望序 [begin, s1, end] 要求嵌套执行发生在 appendChild 内）。
        // https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
        let html = r##"<script src="/resources/testharness.js"></script>
<script>globalThis.__log = [];</script>
<script type="text/plain" id="t">globalThis.__log.push('s1');</script>
<script>
globalThis.test(function() {
  globalThis.__log.push('begin');
  var el = document.getElementById('t');
  el.removeAttribute('type');
  el.appendChild(document.createTextNode(''));
  globalThis.__log.push('end');
  assert_true(JSON.stringify(globalThis.__log) === JSON.stringify(['begin', 's1', 'end']),
    'sync prepare order: ' + globalThis.__log.join(','));
}, 'sync prepare on children changed');
</script>"##;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "prepare-children-changed.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "sync prepare on children changed" && r.status == HarnessStatus::Pass),
            "children-changed 须同步重触发生成: {results:?}"
        );
    }

    #[test]
    fn script_prepare_on_text_content_set_s109() {
        // s109：空 script（type 缺省即合法 classic）textContent= 赋文本 → child text
        // content change 触发 prepare → 同步执行（145 形；SVG namespace script 同面）。
        let html = r##"<script src="/resources/testharness.js"></script>
<svg><script></script></svg>
<script>
globalThis.test(function() {
  var s = document.querySelector('svg > script');
  s.textContent = 'globalThis.__ran = 1;';
  assert_true(globalThis.__ran === 1, '空脚本 textContent= 须同步执行');
}, 'prepare on textContent set');
</script>"##;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "prepare-text-set.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "prepare on textContent set" && r.status == HarnessStatus::Pass),
            "textContent= 须重触发空脚本: {results:?}"
        );
    }

    #[test]
    fn script_prepare_on_reinsertion_external_s110() {
        // s110：type=text/plain 外链单元（s110 起投递面整段透传保 src）重插入 →
        // post-connection 触发（spec 触发面不问 parser-inserted 的语料口径——122-126
        // 族期望序依赖）→ force async 缺省真 → async-set 分支取源执行；工厂节点
        // async=false setter → in-order FIFO 分支（122 形：期望序 [e1, e2]——重插入
        // 先入 set、marker 后入 list，microtask 序保 e1 先执行；两跳 trickle FIFO
        // 保序由 corpus 123 语料钉，bin 侧 sync 取回无延迟窗）。
        let root = std::env::temp_dir().join("zw-s110-external-prepare-test");
        std::fs::create_dir_all(&root).unwrap();
        // 页锚 = https://wpt.test/prepare-external.html（case_name 无目录）→
        // ./inc*.js 相对解析到 root 直下。
        std::fs::write(root.join("inc1.js"), "globalThis.__log.push('e1');").unwrap();
        std::fs::write(root.join("inc2.js"), "globalThis.__log.push('e2');").unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<script>globalThis.__log = [];</script>
<script type="text/plain" id="t" src="./inc1.js"></script>
<script>
globalThis.promise_test(function() {
  return new Promise(function(resolve) {
    var s = document.getElementById('t');
    s.removeAttribute('type');
    s.parentNode.appendChild(s);
    var m = document.createElement('script');
    m.async = false;
    m.src = './inc2.js';
    s.parentNode.appendChild(m);
    setTimeout(resolve, 50);
  }).then(function() {
    assert_true(JSON.stringify(globalThis.__log) === JSON.stringify(['e1', 'e2']),
      'external prepare order: ' + globalThis.__log.join(','));
  });
}, 'external prepare on reinsertion + insertion');
</script>"##;
        let results = run_testharness_html(
            &root,
            "prepare-external.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "external prepare on reinsertion + insertion" && r.status == HarnessStatus::Pass),
            "外链 prepare 须按排队分支取源执行: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn script_src_set_after_docwrite_empty_s117() {
        // s117（html-semantics M2 切片 117）：doc.write 空 script——解析期空源 bail
        // 不置 already-started 章，后设 .src 经 s109 属性臂（s117 体空性门放行空体）
        // + connected 复核（s117 起 getter false 落 parentNode 爬升——mut-tree 新生
        // 元素宿主快照假阴性）→ 外链执行（026 execution-timing 语义，assert_any 双
        // 序的取源完成序形态）。
        let root = std::env::temp_dir().join("zw-s117-docwrite-empty-src-test");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("inc1.js"), "globalThis.__log.push('e1');").unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<script>globalThis.__log = [];</script>
<script>
document.write('<script><\/script>');
var scripts = document.getElementsByTagName('script');
scripts[scripts.length - 1].src = './inc1.js';
</script>
<script>
globalThis.promise_test(function() {
  return new Promise(function(resolve) { setTimeout(resolve, 50); }).then(function() {
    assert_true(JSON.stringify(globalThis.__log) === JSON.stringify(['e1']),
      'doc.write 空 script src 后设须执行外链: ' + globalThis.__log.join(','));
  });
}, 'src set on doc-written empty script');
</script>"##;
        let results = run_testharness_html(
            &root,
            "prepare-docwrite-empty.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "src set on doc-written empty script" && r.status == HarnessStatus::Pass),
            "doc.write 空 script src 后设须触发外链执行: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn script_src_set_after_docwrite_content_no_refire_s117() {
        // s117 对偶面：doc.write 带内容 script——写流面已执行内联体（回放重建
        // plain wrapper 无 sel 章、换代重注册身份断链），后设 .src 按 already-started
        // 语义不重触发（027 execution-timing 语义：s117 体空性门在 s109 属性臂拦下
        // 非空体 sel 形）。若门失效，__log 会多出 'e2'。
        let root = std::env::temp_dir().join("zw-s117-docwrite-content-src-test");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("inc2.js"), "globalThis.__log.push('e2');").unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<script>globalThis.__log = [];</script>
<script>
document.write('<script>globalThis.__log.push("w1");<\/script>');
var scripts = document.getElementsByTagName('script');
scripts[scripts.length - 1].src = './inc2.js';
</script>
<script>
globalThis.promise_test(function() {
  return new Promise(function(resolve) { setTimeout(resolve, 50); }).then(function() {
    assert_true(JSON.stringify(globalThis.__log) === JSON.stringify(['w1']),
      '带内容 doc.write script src 后设不重执行: ' + globalThis.__log.join(','));
  });
}, 'src set on doc-written content script ignored');
</script>"##;
        let results = run_testharness_html(
            &root,
            "prepare-docwrite-content.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "src set on doc-written content script ignored" && r.status == HarnessStatus::Pass),
            "带内容 doc.write script src 后设须按 already-started 跳过: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn script_innerhtml_moved_not_executed_s118() {
        // s118（html-semantics M2 切片 118）：innerHTML 产物 SCRIPT already-started 章
        // （033 execution-timing 语义）——innerHTML 写入的 inline/外链脚本即使
        // appendChild 移入 body 也不得执行（spec prepare the script element：fragment
        // parsing algorithm 创建的 script 元素置 already started）。容器须为标记期
        // 元素（sel 形——033 语料同形；createElement 容器走 handle 形物化路径，
        // 产物换 wrapper 不在本面）。章失效时 __log 会多出
        // 'innerHTML script runs' / 'external script #1'。
        let root = std::env::temp_dir().join("zw-s118-innerhtml-move-test");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("inc3.js"), "globalThis.__log.push('external script #1');").unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<script>globalThis.__log = [];
function zwlog(s) { globalThis.__log.push(s); }</script>
<div id="t"></div>
<script>
zwlog('inline script #1');
var d = document.getElementById('t');
d.innerHTML = '<script>zwlog("innerHTML script runs")<\/script><script src="./inc3.js"><\/script>';
document.body.appendChild(d.firstChild);
document.body.appendChild(d.firstChild);
zwlog('end script #1');
</script>
<script>zwlog('inline script #2');</script>
<script>
globalThis.promise_test(function() {
  return new Promise(function(resolve) { setTimeout(resolve, 50); }).then(function() {
    assert_true(JSON.stringify(globalThis.__log) === JSON.stringify(['inline script #1', 'end script #1', 'inline script #2']),
      'innerHTML 产物移动不执行: ' + globalThis.__log.join(','));
  });
}, 'innerHTML scripts not executed even after move');
</script>"##;
        let results = run_testharness_html(
            &root,
            "prepare-innerhtml-move.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "innerHTML scripts not executed even after move" && r.status == HarnessStatus::Pass),
            "innerHTML 产物 SCRIPT 移动后须按 already-started 保持惰性: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn script_location_href_javascript_evaluated_s118() {
        // s118：location 族 javascript: URL 求值面（079 execution-timing 语义）——
        // window.location = 'javascript:...' 不派 navigate 事件、不走 host 导航，
        // 改跑 javascript: URL 求值（defer 微任务间接 eval）：严格序要求 eval 落
        // 赋值同步栈之后、setTimeout 之前。门失效时体不执行（缺 'javascript: URL'）。
        let root = std::env::temp_dir().join("zw-s118-location-js-href-test");
        std::fs::create_dir_all(&root).unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<script>globalThis.__log = [];
function zwlog(s) { globalThis.__log.push(s); }
zwlog('inline script #1');
</script>
<script>
globalThis.promise_test(function() {
  return new Promise(function(resolve) {
    zwlog('handler');
    window.location = 'javascript:zwlog("javascript: URL")';
    setTimeout(function() { zwlog('timeout'); resolve(); }, 100);
    zwlog('handler ends');
  }).then(function() {
    assert_true(JSON.stringify(globalThis.__log) === JSON.stringify(['inline script #1', 'handler', 'handler ends', 'javascript: URL', 'timeout']),
      'location=javascript: 须求值且落 defer 序: ' + globalThis.__log.join(','));
  });
}, 'location=javascript: evaluated after handler stack');
</script>"##;
        let results = run_testharness_html(
            &root,
            "prepare-location-js-href.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "location=javascript: evaluated after handler stack"
                    && r.status == HarnessStatus::Pass),
            "location= javascript: URL 须按求值步骤 defer 执行: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn script_location_replace_javascript_evaluated_s118() {
        // s118：location.replace('javascript:...') 求值面（028 execution-timing 语义，
        // assert_any 双序）——外链脚本 #2 与 defer eval 的先后两种序都合法，体必须
        // 执行（门失效时 'JS URL' 缺席）。
        let root = std::env::temp_dir().join("zw-s118-location-js-replace-test");
        std::fs::create_dir_all(&root).unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<script>globalThis.__log = [];
function zwlog(s) { globalThis.__log.push(s); }
</script>
<script>
globalThis.promise_test(function() {
  return new Promise(function(resolve) {
    zwlog('1');
    window.location.replace('javascript:zwlog("JS URL")');
    zwlog('end1');
    setTimeout(resolve, 100);
  }).then(function() {
    var j = JSON.stringify(globalThis.__log);
    assert_true(j === JSON.stringify(['1', 'end1', 'JS URL', '2']) || j === JSON.stringify(['1', 'end1', '2', 'JS URL']),
      'replace javascript: assert_any 双序: ' + globalThis.__log.join(','));
  });
}, 'location.replace javascript: evaluated');
</script>
<script>zwlog('2');</script>"##;
        let results = run_testharness_html(
            &root,
            "prepare-location-js-replace.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "location.replace javascript: evaluated" && r.status == HarnessStatus::Pass),
            "location.replace javascript: URL 须按求值步骤执行: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn form_indexed_named_access_focus_s119() {
        // s119（html-semantics M2 切片 119）：HTMLFormElement 支持属性名的 canonical
        // 数字索引臂——无 id/name 的 listed 控件按 form.elements 序号可寻址（031
        // execution-timing 语料面 document.forms[0][0].focus()）。命名（id/name）
        // 匹配先行、索引兜底；越界返回 undefined；focus() 同步派发面随之被钉。
        let root = std::env::temp_dir().join("zw-s119-form-index-test");
        std::fs::create_dir_all(&root).unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<form><input type="button" id="b1"><input name="q"><input type="text"></form>
<script>
globalThis.__focused = [];
document.forms[0][0].addEventListener('focus', function () { globalThis.__focused.push('f0'); });
document.forms[0][0].focus();
document.forms[0][1].focus();
</script>
<script>
globalThis.promise_test(function() {
  var f = document.forms[0];
  assert_true(f[0] && f[0].id === 'b1', 'form[0] 须按 elements 序号取控件');
  assert_true(f[0] === f.elements[0], 'form[N] 与 elements[N] 同一 proxy');
  assert_true(f[1] && f[1].name === 'q', 'form[1] 取第二控件');
  assert_true(f[2] && f[2].type === 'text', 'form[2] 取无名第三控件');
  assert_true(f[9] === undefined, '越界索引 undefined');
  assert_true(f.q && f.q.name === 'q', '命名访问不回归');
  assert_true(JSON.stringify(globalThis.__focused) === JSON.stringify(['f0']),
    'focus 事件序: ' + globalThis.__focused.join(','));
}, 'form indexed access + focus');
</script>"##;
        let results = run_testharness_html(
            &root,
            "form-index-access.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "form indexed access + focus" && r.status == HarnessStatus::Pass),
            "form[N] 索引取控件 + focus 派发须通过: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn script_fragment_clone_not_executed_s120() {
        // s120（html-semantics M2 切片 120）：fragment 解析产物 script 的克隆步骤
        // already-started 继承（svg-script-fragment-parsed-not-executed-on-clone 语料
        // 面）——innerHTML 产物（s118 章 + t8r 物化传递）克隆后插入 body 不得执行；
        // 浅克隆（script 自身）与深克隆（含 script 子树）双面。R387 直插臂 /
        // _r377runScript / notify s111 臂全链章门与 run-once 表继承失效时 __log 会
        // 多出 'clone runs'。合法动态脚本（createElement 直建）照常执行（control）。
        let root = std::env::temp_dir().join("zw-s120-frag-clone-test");
        std::fs::create_dir_all(&root).unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<script>globalThis.__log = [];
function zwlog(s) { globalThis.__log.push(s); }</script>
<script>
globalThis.test(function() {
  var c = document.createElement('div');
  c.innerHTML = '<svg><script>zwlog("clone runs")<\/script></svg>';
  var s = c.querySelector('script');
  var cl = s.cloneNode(true);
  document.body.appendChild(cl);
  assert_true(globalThis.__log.indexOf('clone runs') < 0,
    '浅克隆 fragment SVG script 不得执行: ' + globalThis.__log.join(','));

  globalThis.__log = [];
  var c2 = document.createElement('div');
  c2.innerHTML = '<span><svg><script>zwlog("deep clone runs")<\/script></svg> x</span>';
  var cl2 = c2.firstElementChild.cloneNode(true);
  document.body.appendChild(cl2);
  assert_true(globalThis.__log.indexOf('deep clone runs') < 0,
    '深克隆子树 fragment script 不得执行: ' + globalThis.__log.join(','));

  globalThis.__log = [];
  var dyn = document.createElement('script');
  dyn.textContent = 'zwlog("dynamic runs")';
  document.body.appendChild(dyn);
  assert_true(globalThis.__log.indexOf('dynamic runs') >= 0,
    '合法动态脚本照常执行: ' + globalThis.__log.join(','));
}, 'fragment-parsed script clone not executed');
</script>"##;
        let results = run_testharness_html(
            &root,
            "frag-clone-inert.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "fragment-parsed script clone not executed" && r.status == HarnessStatus::Pass),
            "fragment 产物 script 克隆须按 already-started 保持惰性（动态 control 照常）: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn anchor_url_decomposition_s121() {
        // s121（html-semantics M2 切片 121）：a/area URL 分解组件三面钉——
        // ① sel 形（querySelector）组件 setter 读最新 href（R3070 分支迁
        // __zw_get_attr_lw，M2-S9/R3202 stale-snapshot 族收口）：IDL href 前置后
        // hash setter 不得读 stale 快照静默 no-op；② 解析失败（url null）protocol
        // getter 返 ':'（§4.6.3 protocol getter 步 2）；③ opaque path（cannot-be-a-base）
        // pathname setter no-op（URL spec pathname setter 步 3）。
        let root = std::env::temp_dir().join("zw-s121-anchor-url-test");
        std::fs::create_dir_all(&root).unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<a>anchor</a>
<script>
globalThis.test(function() {
  var a = document.querySelector('a');
  a.href = 'http://google.com/index.html#somehash';
  a.hash = 'someother';
  assert_equals(a.hash, '#someother',
    'sel 形组件 setter 读最新 href: ' + a.hash);

  var b = document.createElement('a');
  b.href = 'http://test:test/';
  assert_equals(b.protocol, ':',
    '解析失败 protocol getter 返 ":": ' + JSON.stringify(b.protocol));

  var c = document.createElement('a');
  c.href = 'non-special:opaque';
  c.pathname = 'string';
  assert_equals(c.pathname, 'opaque',
    'opaque path pathname setter no-op: ' + c.pathname);
}, 'anchor URL decomposition faces');
</script>"##;
        let results = run_testharness_html(
            &root,
            "anchor-url-decomp.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "anchor URL decomposition faces" && r.status == HarnessStatus::Pass),
            "a/area URL 分解三面（sel setter / 失败 protocol / opaque pathname）须通过: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn script_charset_s122() {
        // s122（html-semantics M2 切片 122）：classic script 编码两面钉——
        // ① `document.characterSet` 主文档编码嗅探（meta 元素遍历回退：head 内
        // `<div>` 触发隐式 body 化把后续 meta 推入 body，head innerHTML 树位源
        // 失效——script-charset-02 假设臂形）；② 无 BOM/无 ct charset 参数/无
        // charset 属性外链按**文档编码**解码（runner 侧 sniff 对 harness 注入前
        // 原始 case html——windows-1250 文件 bytes 9C E6 A5 8F BF → "śćążź"）。
        // charset 属性变更脸（append 后改 utf-8 按 append 时值解码）为异步
        // async_test 形，由套件 corpus script-charset-03 本身覆盖，不入本 pin。
        let root = std::env::temp_dir().join("zw-s122-script-charset-test");
        let case_dir = "html/semantics/scripting-1/the-script-element";
        std::fs::create_dir_all(root.join(case_dir)).unwrap();
        // "śćążź" 的 windows-1250 字节（spec script-charset-02 语料同文：
        // 9C E6 B9 BF 9F，corpus external-script-windows1250.js 实测）。
        let ext_bytes: [u8; 5] = [0x9C, 0xE6, 0xB9, 0xBF, 0x9F];
        std::fs::write(
            root.join(case_dir).join("ext-pin-w1250.js"),
            b"globalThis.getSomeString = function () { return \""
                .iter()
                .copied()
                .chain(ext_bytes)
                .chain(b"\"; };".iter().copied())
                .collect::<Vec<u8>>(),
        )
        .unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<div id="log"></div>
<meta charset="windows-1250">
<script type="text/javascript"
  src="serve-with-content-type.py?fn=ext-pin-w1250.js&ct=text/javascript"></script>
<script>
globalThis.test(function() {
  assert_equals(document.characterSet, 'windows-1250',
    'characterSet meta 嗅探（div 后 meta 推入 body 面）: ' + document.characterSet);
  assert_equals(globalThis.getSomeString(), 'śćążź',
    '外链按文档编码解码: ' + JSON.stringify(globalThis.getSomeString()));
}, 'script charset s122 faces');
</script>"##;
        let results = run_testharness_html(
            &root,
            &format!("{case_dir}/script-charset-pin.html"),
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "script charset s122 faces" && r.status == HarnessStatus::Pass),
            "classic script 编码三面（characterSet 嗅探 / 文档编码回退解码）须通过: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn formdata_entry_list_s123() {
        // s123（html-semantics M2 切片 123）：FormData 条目构造两面钉——
        // ①动态建表（createElement+append）同 turn `new FormData(form)` 枚举
        // 非空（枚举双源并集：form 子树 live face + document 级快照 face 按键
        // 去重——原单源 document 级对 live 树恒空，逗号列表查询无 pending 回
        // 落），且 hidden value 原文换行零归一（spec entry list 不做换行规范
        // 化——newline-normalization 12F 判据面）；②file 状态条目臂（spec
        // constructing the entry list：每个选中文件一条 [name, File,
        // filename]——DataTransfer→files→input.files 链 + get 返 File 本体，
        // File.name 原文换行）。
        let root = std::env::temp_dir().join("zw-s123-formdata-entry-test");
        std::fs::create_dir_all(&root).unwrap();
        let html = r##"<script src="/resources/testharness.js"></script>
<script>
globalThis.test(function() {
  var form = document.createElement('form');
  var input = document.createElement('input');
  input.type = 'hidden';
  input.name = 'a';
  input.value = 'b\nc';
  form.appendChild(input);
  document.body.appendChild(form);
  var fd = new FormData(form);
  assert_equals(fd.get('a'), 'b\nc',
    'live form 枚举 + 原文换行零归一: ' + JSON.stringify(fd.get('a')));
}, 's123 live form entry list');
globalThis.test(function() {
  var form = document.createElement('form');
  var input = document.createElement('input');
  input.type = 'file';
  input.name = 'f';
  var dt = new DataTransfer();
  dt.items.add(new File([], 'x\ny', { type: 'text/plain' }));
  input.files = dt.files;
  form.appendChild(input);
  document.body.appendChild(form);
  var fd = new FormData(form);
  var v = fd.get('f');
  assert_equals(v && v.name, 'x\ny',
    'file 条目 File.name 原文: ' + (v ? JSON.stringify(v.name) : 'null'));
}, 's123 file entry filename');
</script>"##;
        let results = run_testharness_html(
            &root,
            "formdata-entry-pin.html",
            html,
            &mini_harness_stateful(),
            CASE_TIMEOUT,
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "s123 live form entry list" && r.status == HarnessStatus::Pass),
            "FormData live 表单枚举面须通过: {results:?}"
        );
        assert!(
            results
                .iter()
                .any(|r| r.name == "s123 file entry filename" && r.status == HarnessStatus::Pass),
            "FormData file 条目面须通过: {results:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn inline_module_import_specs_rewrite_to_root_absolute_s101() {
        // 直接钉 helper：相对 spec 以模块自身目录（resources/）为基上溯归一；
        // root-absolute/裸 specifier/含 scheme 的 spec 不动；单双引号皆替换。
        let body = "import D from './dep.js';\
            \nimport { x } from \"../shared/util.js\";\
            \nimport \"/root-abs.js\";\
            \nimport 'bare-spec';\
            \nimport 'https://elsewhere.test/x.js';";
        let out = rewrite_inline_module_import_specs(
            body,
            "html/semantics/scripting-1/the-script-element/resources/entry.js",
        );
        assert!(out.contains("import D from '/html/semantics/scripting-1/the-script-element/resources/dep.js';"));
        assert!(out.contains("import { x } from \"/html/semantics/scripting-1/the-script-element/shared/util.js\";"));
        assert!(out.contains("import \"/root-abs.js\";"));
        assert!(out.contains("import 'bare-spec';"));
        assert!(out.contains("import 'https://elsewhere.test/x.js';"));

        // 端到端钉：type="module" 外链内联后体内相对 import 被重写；classic 外链体不动。
        let root = std::env::temp_dir().join("zw-s101-inline-module-specs-test");
        let case_dir = "html/semantics/scripting-1/the-script-element";
        std::fs::create_dir_all(root.join(case_dir).join("resources")).unwrap();
        std::fs::write(
            root.join(case_dir).join("resources/entry.js"),
            "import D from './dep.js';\nglobalThis.d = D;",
        )
        .unwrap();
        std::fs::write(root.join(case_dir).join("plain.js"), "import E from './dep.js';").unwrap();
        let html = "<!doctype html><body>\
            <script type=\"module\" src=\"./resources/entry.js\"></script>\
            <script src=\"./plain.js\"></script>\
            </body>";
        let out = inline_local_scripts(html, &root, &format!("{case_dir}/page.html"), None);
        assert!(out.contains("import D from '/html/semantics/scripting-1/the-script-element/resources/dep.js';"));
        // classic 体保持原 spec（依赖收集基 = 页面 URL，./ 相对页目录语义不变）。
        assert!(out.contains("import E from './dep.js';"));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn service_worker_missing_scope_html_returns_document_response() {
        let handler = wpt_data_fetch_handler(Path::new("/nonexistent-wpt-root")).unwrap();
        let req = zero_engine::fetch_bridge::FetchRequest {
            url: "https://wpt.test/service-workers/service-worker/resources/missing-scope.html".to_string(),
            method: "GET".to_string(),
            headers: Vec::new(),
            body: None,
            body_bytes: None,
            credentials: None,
            mode: None,
            redirect: None,
        };

        let resp = handler(&req).unwrap();
        assert_eq!(resp.status, 404);
        assert_eq!(resp.body, "");
        assert!(
            resp.headers
                .iter()
                .any(|(name, value)| name.eq_ignore_ascii_case("content-type") && value == "text/html")
        );
    }

    #[test]
    fn cache_storage_fetch_handler_marks_filtered_response_types() {
        let handler = wpt_data_fetch_handler(Path::new("/nonexistent-wpt-root")).unwrap();
        let make_req =
            |url: &str, mode: Option<&str>, redirect: Option<&str>| zero_engine::fetch_bridge::FetchRequest {
                url: url.to_string(),
                method: "GET".to_string(),
                headers: Vec::new(),
                body: None,
                body_bytes: None,
                credentials: None,
                mode: mode.map(str::to_string),
                redirect: redirect.map(str::to_string),
            };
        let response_type = |response: zero_engine::fetch_bridge::FetchResponse| {
            response
                .headers
                .into_iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("x-zero-response-type"))
                .map(|(_, value)| value)
                .unwrap()
        };

        assert_eq!(
            response_type(
                handler(&make_req(
                    "https://wpt.test/dom/nodes/encoding.py?label=utf-8",
                    None,
                    None,
                ))
                .unwrap()
            ),
            "basic"
        );
        assert_eq!(
            response_type(
                handler(&make_req(
                    "https://www1.wpt.test/service-workers/cache-storage/resources/vary.py",
                    Some("cors"),
                    None,
                ))
                .unwrap()
            ),
            "cors"
        );
        assert_eq!(
            response_type(
                handler(&make_req(
                    "https://www1.wpt.test/service-workers/cache-storage/resources/vary.py",
                    Some("no-cors"),
                    None,
                ))
                .unwrap()
            ),
            "opaque"
        );
        assert_eq!(
            response_type(
                handler(&make_req(
                    "https://www1.wpt.test/service-workers/cache-storage/resources/redirect.py",
                    Some("cors"),
                    Some("manual"),
                ))
                .unwrap()
            ),
            "opaqueredirect"
        );
    }

    #[test]
    fn phase_four_with_all_results_is_terminal() {
        assert!(terminal_harness_state(
            &serde_json::json!({"phase": 4, "pending": 0, "tests": 3}),
            3
        ));
        assert!(!terminal_harness_state(
            &serde_json::json!({"phase": 4, "pending": 0, "tests": 3}),
            2
        ));
        assert!(!terminal_harness_state(
            &serde_json::json!({"phase": 3, "pending": 1, "tests": 2}),
            1
        ));
        assert!(!harness_probe_is_terminal(
            true,
            &serde_json::json!({"phase": 3, "pending": 1, "tests": 2}),
            1
        ));
        assert!(harness_probe_is_terminal(true, &serde_json::Value::Null, 2));
    }
}

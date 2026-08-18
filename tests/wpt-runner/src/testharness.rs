//! WPT testharness runner and minimal testdriver adapter for HTML interactions.

use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use zero_page_runtime::{HtmlActionRequest, HtmlUserAction};
use zero_webview::{WebView, WebViewConfig};

const CASE_TIMEOUT: Duration = Duration::from_secs(10);

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

/// DOM 专项（docs/goal/js-dom.md，M4 / DC-3）导入的上游 `dom/` 子目录面。
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

/// WPT subtest status.
///
/// 映射上游 testharness subtest status 数字编码（`testharness.js` 的 `Test.status`）：
/// `0=PASS`、`1=FAIL`、`2=TIMEOUT`、`3=NOTRUN`、`4=PRECONDITION_FAILED`。其中 `NOTRUN`（测试
/// 因脚本错误/超时未执行）与 `PRECONDITION_FAILED`（`assert_implements`/`assert_implements_optional`
/// 的 precondition 不满足，如 optional feature 不支持）是**中性状态**——上游 WPT dashboard 既不计入
/// pass 也不计入 fail（precondition 失败非实现缺陷，NOTRUN 属基础设施跳过）。runner 通过率统计须把
/// 它们与 `Fail` 区分（js-dom R20：原 `map_harness_results` 的 `_ => Fail` 把 3/4 误计为 Fail，
/// 拖低 optional feature 如 TouchEvent 的 dom/nodes 通过率）。
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
    let harness_source = match std::fs::read_to_string(wpt_root.join("resources/testharness.js")) {
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
        let source = match std::fs::read_to_string(&path) {
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
        match std::fs::read_to_string(&path) {
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
    let harness_source = match std::fs::read_to_string(&harness_path) {
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
            let source = std::fs::read_to_string(wpt_root.join(path));
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
    let harness_source = match std::fs::read_to_string(wpt_root.join("resources/testharness.js")) {
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
    let canvas_tests_source = match std::fs::read_to_string(wpt_root.join(CANVAS_TESTS_JS_PATH)) {
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
            let source = match std::fs::read_to_string(&path) {
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
    let harness_source = match std::fs::read_to_string(wpt_root.join("resources/testharness.js")) {
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
            let source = match std::fs::read_to_string(&path) {
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
            let results = run_testharness_html(wpt_root, &relative, &source, &harness_source, CASE_TIMEOUT);
            cases.push((relative, results));
        }
    }
    cases
}

/// Run the pinned upstream IndexedDB `.any.js` subset.
pub fn run_indexeddb_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match std::fs::read_to_string(wpt_root.join("resources/testharness.js")) {
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
            let case_source = match std::fs::read_to_string(wpt_root.join(path)) {
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
                match std::fs::read_to_string(wpt_root.join(case_dir).join(script)) {
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

fn indexeddb_window_wrapper(path: &str, support: &[(&str, &str)], case_source: &str) -> String {
    let mut source = String::new();
    for (name, script) in support {
        source.push_str(&format!("// source: {name}\n{script}\n"));
    }
    source.push_str(&format!("// source: {path}\n{case_source}"));
    let source = source.replace("</script", "<\\/script");
    format!(
        "<!doctype html><meta charset=\"utf-8\">\
         <script src=\"/resources/testharness.js\"></script>\
         <script src=\"/resources/testharnessreport.js\"></script>\
         <script>{source}</script>"
    )
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
        // 仅 wpt.test 域名（测试资源）；其他 URL 回退网络。
        let path_part = url.strip_prefix("https://wpt.test")?;
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
fn wpt_data_script_fetcher(wpt_root: &std::path::Path) -> Option<zero_webview::ScriptSourceFetcher> {
    let root = wpt_root.to_path_buf();
    Some(std::sync::Arc::new(move |_page_url: &str, src: &str| {
        let path_part = src.strip_prefix('/').unwrap_or(src);
        let clean = path_part.split(['?', '#']).next().unwrap_or(path_part);
        if clean.is_empty() {
            return Err("empty path".to_string());
        }
        let full = root.join(clean);
        std::fs::read_to_string(&full).map_err(|e| format!("script fetch failed: {clean} ({e})"))
    }))
}

/// R34xx（G6）：运行导入的 `html/canvas` `.worker.js` OffscreenCanvas worker 变体——每个
/// 文件包一个 `fetch_tests_from_worker(new Worker(...))` 主页面（testharness.js 的 worker
/// 聚合协议），经 run_testharness_html_inner 同款轮询执行。返回与主线程用例同构结果。
pub fn run_canvas_worker_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match std::fs::read_to_string(wpt_root.join("resources/testharness.js")) {
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
            let source = match std::fs::read_to_string(&path) {
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
    Some(std::sync::Arc::new(
        move |req: &zero_engine::fetch_bridge::FetchRequest| {
            if req.method != "GET" {
                return Err(format!("method not supported: {}", req.method));
            }
            let path_part = req.url.strip_prefix("https://wpt.test").unwrap_or(&req.url);
            let path_part = path_part.strip_prefix('/').unwrap_or(path_part);
            let clean = path_part.split(['?', '#']).next().unwrap_or(path_part);
            if clean.is_empty() {
                return Err("empty path".to_string());
            }
            match std::fs::read(root.join(clean)) {
                Ok(bytes) => Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: Vec::new(),
                    body: String::from_utf8_lossy(&bytes).into_owned(),
                    body_bytes: Some(bytes),
                }),
                Err(e) => Err(format!("not found: {clean} ({e})")),
            }
        },
    ))
}

fn run_testharness_html_inner(
    wpt_root: &Path,
    case_name: &str,
    source: &str,
    harness_source: &str,
    inline_extras: &[(&str, &str)],
    timeout: Duration,
) -> Vec<HarnessSubtestResult> {
    let unsupported = unsupported_testdriver_dependencies(source);
    if !unsupported.is_empty() {
        return vec![HarnessSubtestResult {
            name: case_name.to_string(),
            status: HarnessStatus::Unsupported,
            message: Some(format!("unsupported testdriver API: {}", unsupported.join(", "))),
        }];
    }

    let html = prepare_harness_html(source, harness_source, inline_extras, wpt_root, case_name);
    let scripts = zero_engine::extract_page_scripts(&html);
    let script_lengths = scripts
        .iter()
        .map(|script| match script {
            zero_engine::PageScript::Inline(source) | zero_engine::PageScript::InlineModule(source) => source.len(),
            zero_engine::PageScript::External(_) | zero_engine::PageScript::ExternalModule(_) => 0,
        })
        .collect::<Vec<_>>();
    // js-dom goal DC-3「native 路径对照」：env `ZW_NATIVE_DOM=1` 时 runner 走原生绑定路径
    //（WebViewConfig.native_dom=true），而非默认 polyfill 字符串桥。用于建立 native 通过率
    // 基线，让 R2/R3/R4 native 修复（classList/createElement/node mutation DOMException）的基线
    // 价值可见。env 进程级（testharness 一次跑一个路径，无混跑）。
    let native_dom = std::env::var("ZW_NATIVE_DOM").as_deref() == Ok("1");
    let mut webview = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        native_dom,
        // R34xx：headless 图片源——wpt.test/images/* 映射到本地 wpt-data 目录
        //（testharness 无网络；G5 DOM img 源解锁依赖图片加载）。
        // js-dom goal：dom 用例同样需要本地 .js 内联 + 图片资源，两条路径统一走 wpt_root。
        image_source_fetcher: wpt_data_image_fetcher(wpt_root),
        // R34xx：fetch() 本地资源（2d.composite.image.* fetch+createImageBitmap 路径）。
        fetch_handler: wpt_data_fetch_handler(wpt_root),
        // R34xx（G6）：.worker.js 变体 + worker importScripts 的脚本源。
        script_source_fetcher: wpt_data_script_fetcher(wpt_root),
        ..WebViewConfig::default()
    });
    webview.prepare_document_state(&format!("https://wpt.test/{case_name}"));
    let page_url = format!("https://wpt.test/{case_name}");
    // R34xx：canvas 默认字体（sans-serif）预载系统真字体（带 kern）——无 @font-face 的
    // 页面（2d.text.drawing.style.fontKerning 等）默认字体度量/kerning 面依赖。需
    // resolve_font_id 大小写不敏感修复配套（否则 CanvasTest 显式族 miss 回退 sans-serif）。
    webview.load_canvas_system_sans_font();
    let external_css = webview.fetch_page_images(&html, &page_url);
    webview.load_html(&html, Some(&external_css));
    if let Err(error) = webview.run_page_scripts_strict() {
        return vec![HarnessSubtestResult {
            name: case_name.to_string(),
            status: HarnessStatus::Fail,
            message: Some(format!("page script threw: {error}")),
        }];
    }

    let deadline = Instant::now() + timeout;
    let mut partial_results = Vec::new();
    let mut last_test_function = "unknown".to_string();
    let mut last_harness_hook = "unknown".to_string();
    let mut last_state = serde_json::Value::Null;
    loop {
        if Instant::now() >= deadline {
            let mut results = map_harness_results(partial_results);
            results.push(HarnessSubtestResult {
                name: case_name.to_string(),
                status: HarnessStatus::Timeout,
                message: Some(format!(
                    "testharness completion callback was not called (test={}, hook={}, scripts={script_lengths:?}, state={last_state})",
                    last_test_function, last_harness_hook
                )),
            });
            return results;
        }

        let probe = match take_probe(&mut webview) {
            Ok(probe) => probe,
            Err(error) => {
                return vec![HarnessSubtestResult {
                    name: case_name.to_string(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }];
            }
        };
        partial_results = probe.results;
        last_test_function = probe.test_function;
        last_harness_hook = probe.harness_hook;
        last_state = probe.state;
        for command in probe.commands {
            let result = apply_testdriver_command(&mut webview, &command);
            if let Err(error) = resolve_testdriver_command(&mut webview, command.id, result.as_deref()) {
                return vec![HarnessSubtestResult {
                    name: case_name.to_string(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }];
            }
        }
        if probe.complete {
            if partial_results.is_empty() {
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
        if probe.due_timer {
            std::thread::yield_now();
        } else {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
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

fn prepare_harness_html(
    source: &str,
    harness_source: &str,
    inline_extras: &[(&str, &str)],
    wpt_root: &Path,
    case_path: &str,
) -> String {
    let reporter = r#"
if (typeof setup === 'function') setup({output: false});
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
    let harness_source = harness_source.replacen(
        "\n})(self);",
        "\nglobal_scope.__zw_mark_harness_loaded = function() {\n\
         test_environment.all_loaded = true;\n\
         if (tests.all_done()) tests.complete();\n\
         };\n\
         global_scope.__zw_harness_state = function() {\n\
         return {tests:tests.tests.length,pending:tests.num_pending,loaded:test_environment.all_loaded,phase:tests.phase};\n\
         };\n})(self);",
        1,
    );
    // R34xx：__zw_setTimeout stub 记录定时器（id/at），经 take_probe 的
    // __zw_fire_due_timers 按真实经过时间触发（t.step_timeout(500) 的 fontface.repeat
    // 依赖回调最终触发；记录式避免 microtask 立即触发破坏 testharness 时序——
    // 既有 no-op stub 使回调永不触发）。
    let timer_stub = "\
      globalThis.__zw_setTimeout = function(id, delay) {\n\
        globalThis.__zw_timers = globalThis.__zw_timers || [];\n\
        globalThis.__zw_timers.push({ id: id, at: Date.now() + (delay | 0) });\n\
      };\n\
      globalThis.__zw_clearTimeout = function() {};\n\
      globalThis.__zw_fire_due_timers = function() {\n\
        var timers = globalThis.__zw_timers || [];\n\
        if (!timers.length) return;\n\
        var now = Date.now();\n\
        var due = null, rest = [];\n\
        for (var i = 0; i < timers.length; i++) {\n\
          if (due === null && timers[i].at <= now) due = timers[i]; else rest.push(timers[i]);\n\
        }\n\
        globalThis.__zw_timers = rest;\n\
        if (due === null) return;\n\
        var fn = globalThis.__zw_pending[due.id];\n\
        if (fn) { delete globalThis.__zw_pending[due.id]; try { fn(); } catch (_e) {} }\n\
      };\n";
    let harness = format!("<script>\n{timer_stub}{harness_source}\n{reporter}\n</script>");
    let mut html = replace_script_source(source, "/resources/testharness.js", &harness);
    html = replace_script_source(&html, "/resources/testharnessreport.js", "");
    html = replace_script_source(&html, "/resources/testdriver.js", TESTDRIVER_STUB);
    html = replace_script_source(&html, "/resources/testdriver-vendor.js", "");
    html = replace_script_source(&html, "/resources/testdriver-actions.js", "");
    // canvas-tests.js 等用例框架脚本：与 testharness.js 同款内联（外部脚本提取器不加载 src）。
    for (script_src, inline_source) in inline_extras {
        html = replace_script_source(&html, script_src, &format!("<script>{inline_source}</script>"));
    }
    // js-dom goal：用例引用的本地 .js 测试体（如 <script src="attributes.js">、
    // <script src="Document-createProcessingInstruction.js">）——extract_page_scripts 不加载外部 src，
    // 故此处从 wpt-data 读文件内容内联。case_path 形如 "dom/nodes/attributes.html"，本地 .js 解析为
    // 同目录文件（相对 src 如 "attributes.js" / "./attributes.js" / "../constants.js"）。
    // 仅内联相对路径（非 /resources/、非 http(s):）；文件缺失则移除该 script 标签（不注入空）。
    html = inline_local_scripts(&html, wpt_root, case_path);
    html.push_str(
        "<script>\
         document.dispatchEvent(new Event('DOMContentLoaded'));\
         globalThis.dispatchEvent(new Event('load'));\
         if (typeof globalThis.__zw_mark_harness_loaded === 'function') {\
           globalThis.__zw_mark_harness_loaded();\
         }\
         </script>",
    );
    html
}

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

/// 内联用例引用的本地 .js 测试体（js-dom goal R8）。
///
/// `extract_page_scripts` 不加载外部 `<script src>`，故用例引用的同目录 .js（如 attributes.js、
/// Document-createProcessingInstruction.js）或相对路径（../constants.js）不会执行 → `attr_is`/
/// 测试体 not defined。本函数扫描剩余的 `<script src="...">`（相对路径，非 /resources/、非 http），
/// 从 wpt-data 读文件内容内联为 inline `<script>`；文件缺失则移除该标签（best-effort，不注入空）。
///
/// `case_path` 形如 "dom/nodes/attributes.html"；相对 src 解析为相对该 case 所在目录。
fn inline_local_scripts(html: &str, wpt_root: &Path, case_path: &str) -> String {
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
        // 提取 src="..."（仅相对路径 .js）。
        let src = extract_script_src(open);
        let resolved = src.and_then(|s| {
            // 仅相对路径（不以 / 开头、非 http(s):、非 // ）。
            if s.starts_with('/') || s.starts_with("http://") || s.starts_with("https://") || s.starts_with("//") {
                return None;
            }
            // 规范化 "./" 前缀 + 相对 case_dir 解析（含 ../ 上溯）。
            let rel = s.strip_prefix("./").unwrap_or(s);
            let combined = if case_dir.is_empty() {
                rel.to_string()
            } else {
                normalize_relative(&format!("{case_dir}/{rel}"))
            };
            std::fs::read_to_string(wpt_root.join(&combined))
                .ok()
                .map(|c| (combined, c))
        });
        let rest_start = open_end + 1;
        match resolved {
            Some((combined, content)) => {
                output.push_str(&remaining[..start]);
                output.push_str("<script data-inline=\"");
                output.push_str(&combined);
                output.push_str("\">");
                output.push_str(&content);
                output.push_str("</script>");
            }
            None => {
                // 非本地 .js（/resources/、http、或文件缺失）：保留原标签（extract_page_scripts 处理）。
                output.push_str(&remaining[..rest_start]);
            }
        }
        remaining = &remaining[rest_start..];
    }
    output
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
fn normalize_relative(path: &str) -> String {
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
        if !name.is_empty() && name != "click" && name != "send_keys" && !dependencies.contains(&name) {
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

fn take_probe(webview: &mut WebView) -> Result<HarnessProbe, String> {
    // Pump timer tasks first so the sandbox's microtask checkpoint has flushed
    // testharness result callbacks before the state snapshot is serialized.
    webview
        .execute_script("if (typeof globalThis.__zw_fire_due_timers === 'function') globalThis.__zw_fire_due_timers()")
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
               return st && st.phase === 3;\
             })(),\
             results:globalThis.__zw_harness_results||[],\
             test_function:typeof globalThis.test,\
             harness_hook:typeof globalThis.__zw_mark_harness_loaded,\
             state:typeof globalThis.__zw_harness_state==='function'?globalThis.__zw_harness_state():null,\
             due_timer:(globalThis.__zw_timers||[]).some(function(timer){ return timer.at <= Date.now(); }),\
             commands:(globalThis.__zw_td_queue||[]).splice(0)})",
        )
        .map_err(|error| error.to_string())?;
    serde_json::from_str(&value).map_err(|error| format!("invalid harness probe: {error}: {value}"))
}

fn apply_testdriver_command(webview: &mut WebView, command: &TestdriverCommand) -> Option<String> {
    let Some(selector) = command.selector.as_deref() else {
        return Some("testdriver target has no stable selector".into());
    };
    let target = match webview.page_node_ref_for_selector(selector) {
        Some(target) => target,
        None => return Some(format!("testdriver target not found: {selector}")),
    };
    match command.operation.as_str() {
        "click" => dispatch_action(webview, target, HtmlUserAction::Activate),
        "send_keys" => {
            let text = command.text.as_deref().unwrap_or_default();
            for character in text.chars() {
                let action = match character {
                    '\u{E003}' => HtmlUserAction::DeleteBackward,
                    '\u{E004}' => HtmlUserAction::MoveFocus { forward: true },
                    character if ('\u{E000}'..='\u{F8FF}').contains(&character) => {
                        return Some(format!("unsupported WebDriver key U+{:04X}", character as u32));
                    }
                    character => HtmlUserAction::InsertText {
                        text: character.to_string(),
                    },
                };
                if let Some(error) = dispatch_action(webview, target, action) {
                    return Some(error);
                }
            }
            None
        }
        operation => Some(format!("unsupported testdriver command: {operation}")),
    }
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
        Ok(result) => Some(format!("action was not applicable: {:?}", result.noop_reason)),
        Err(error) => Some(error.to_string()),
    }
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
  globalThis.__zw_td_queue = [];
  function selectorFor(element) {
    if (!element) return null;
    var id = element.getAttribute && element.getAttribute('id');
    if (id) return '#' + id;
    var tag = String(element.tagName || '').toLowerCase();
    if (!tag) return null;
    var matches = document.querySelectorAll(tag);
    return matches.length === 1 ? tag : null;
  }
  function enqueue(operation, element, text) {
    return new Promise(function(resolve, reject) {
      var id = nextId++;
      pending[id] = { resolve: resolve, reject: reject };
      globalThis.__zw_td_queue.push({
        id: id, operation: operation, selector: selectorFor(element),
        text: text == null ? null : String(text)
      });
    });
  }
  globalThis.__zw_td_resolve = function(id, error) {
    var entry = pending[id];
    if (!entry) return;
    delete pending[id];
    if (error == null) entry.resolve();
    else entry.reject(new Error(String(error)));
  };
  globalThis.test_driver = {
    click: function(element) { return enqueue('click', element, null); },
    send_keys: function(element, keys) { return enqueue('send_keys', element, keys); }
  };
})();
</script>"#;

#[cfg(test)]
mod tests {
    use super::*;

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
            Duration::from_secs(2),
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
    fn unsupported_testdriver_command_is_explicit() {
        let html = "test_driver.set_permission({name:'clipboard-read'}, 'granted')";
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "unsupported.html",
            html,
            MINI_HARNESS,
            Duration::from_secs(1),
        );
        assert_eq!(results[0].status, HarnessStatus::Unsupported);
        assert!(results[0].message.as_deref().unwrap().contains("set_permission"));
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
            Duration::from_secs(1),
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
}

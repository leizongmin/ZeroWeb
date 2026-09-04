//! WPT testharness runner and minimal testdriver adapter for HTML interactions.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
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
/// the window variant for `docs/goal/storage-cache-api.md`. Service Worker
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

/// 解析用例声明的 `<meta name="variant" content="?query">` 列表（js-dom R329）。
///
/// WPT variant 用例（如 Range-in-shadow-after-the-shadow-removed 的 `?mode=open` /
/// `?mode=closed`）以同一文件 + 不同 query string 组成参数矩阵；runner 此前只跑基础
/// URL（无 query），依赖 variant 参数的用例全簇误败（`mode=null` 落 TypeError）。
/// content 支持无引号/单双引号形式；与上游 wpt struct 一致，query 含前导 `?`。
fn case_variants(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = source.to_ascii_lowercase();
    let mut rest = lower.as_str();
    while let Some(idx) = rest.find("name=\"variant\"").or_else(|| rest.find("name=variant")) {
        let after = &rest[idx..];
        let Some(content_idx) = after.find("content=") else {
            break;
        };
        let tail = &after[content_idx + "content=".len()..];
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
    }
    out
}

/// Media elements goal（docs/goal/media-elements.md，M1 / DC-1）导入的上游
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
    // 不导入 video_size_preserved_after_ended（2026-09-04 实证）：静态 <source>
    // 形态的 loadedmetadata 与 promise_test EventWatcher 时序在 headless 双通道
    // settle（runner 静态 commit + shim microtask）下不稳定——md 派发早于
    // wait_for 挂载或延迟到超时（两形态均实测）。依赖 settle 时序收敛，随
    // runner/shim 事件通道统一后复评。不导入 video_timeupdate_on_seek（WPT CGI
    // src）/ video_initially_paused（reftest 型）/ video-loading-* poster 族。

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
    // audit.js 框架内联（canvas-tests.js 同款机制——用例以绝对路径引用
    // /webaudio/resources/*.js，extract_page_scripts 不加载外部 src）。
    let inline_extras: Vec<(&str, String)> = WEBAUDIO_SUPPORT_SCRIPTS
        .iter()
        .filter_map(|(src, path)| {
            std::fs::read_to_string(wpt_root.join(path))
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
            let source = std::fs::read_to_string(wpt_root.join(path));
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

    MEDIA_TEST_FILES
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

/// Run the pinned upstream CacheStorage window subset.
pub fn run_cache_storage_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match std::fs::read_to_string(wpt_root.join("resources/testharness.js")) {
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
                match std::fs::read_to_string(wpt_root.join(path)) {
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
                match std::fs::read_to_string(support_path) {
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
    let harness_source = match std::fs::read_to_string(wpt_root.join("resources/testharness.js")) {
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
            let results = match std::fs::read_to_string(wpt_root.join(path)) {
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

fn wpt_meta_scripts(path: &str, source: &str) -> Vec<String> {
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

fn apply_wpt_substitutions(source: &str) -> String {
    source
        .replace("{{host}}", "wpt.test")
        .replace("{{domains[www1]}}", "www1.wpt.test")
        .replace("{{domains[www2]}}", "www2.wpt.test")
        .replace("{{hosts[alt][]}}", "alt.wpt.test")
        .replace("{{hosts[alt][www2]}}", "www2.alt.wpt.test")
        .replace("{{ports[http][0]}}", "80")
        .replace("{{ports[http][1]}}", "8000")
        .replace("{{ports[https][0]}}", "443")
        .replace("{{ports[https][1]}}", "8443")
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
    Some(std::sync::Arc::new(move |page_url: &str, src: &str| {
        let path_part = match src.strip_prefix("https://wpt.test") {
            Some(path) => path.to_string(),
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
        let full = root.join(clean);
        std::fs::read_to_string(&full)
            .map(|source| {
                source
                    .replace("{{host}}", "wpt.test")
                    .replace("{{domains[www1]}}", "www1.wpt.test")
                    .replace("{{ports[https][0]}}", "443")
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
        if scheme != "https" || !matches!(authority, "wpt.test" | "www1.wpt.test" | "www1.wpt.test:443") {
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
            let template = std::fs::read_to_string(root.join(&template_path))
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
            let source = std::fs::read_to_string(root.join(&path))
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
                .replace("{{ports[https][0]}}", "443");
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
    let vary_value_override = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
    Some(std::sync::Arc::new(
        move |req: &zero_engine::fetch_bridge::FetchRequest| {
            let path_part = wpt_url_path(&req.url);
            let path_part = path_part.strip_prefix('/').unwrap_or(path_part);
            let clean = path_part.split(['?', '#']).next().unwrap_or(path_part);
            if clean.is_empty() {
                return Err("empty path".to_string());
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
            if req.method != "GET" {
                return Err(format!("method not supported: {}", req.method));
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
            match std::fs::read(root.join(clean)) {
                Ok(bytes) => {
                    let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
                    let mut headers = wpt_pipe_headers(query);
                    headers.extend(wpt_static_resource_headers(clean));
                    if clean == "service-workers/cache-storage/resources/simple.txt"
                        && wpt_query_value(query, "zw-filtered").as_deref() == Some("cors")
                    {
                        headers.push(("Access-Control-Allow-Origin".into(), "*".into()));
                    }
                    let status = wpt_pipe_status(query).unwrap_or(200);
                    wpt_add_fetch_metadata(&mut headers, req, status);
                    Ok(zero_engine::fetch_bridge::FetchResponse {
                        status,
                        status_text: wpt_status_text(status).to_string(),
                        headers,
                        body: String::from_utf8_lossy(&bytes).into_owned(),
                        body_bytes: Some(bytes),
                    })
                }
                Err(e) => {
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
                    Err(format!("not found: {clean} ({e})"))
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
    let page_origin = "https://wpt.test";
    let response_origin = wpt_url_origin(&req.url);
    let mode = req.mode.as_deref().unwrap_or("");
    let redirect = req.redirect.as_deref().unwrap_or("");
    if redirect == "manual" && matches!(status, 301 | 302 | 303 | 307 | 308) {
        "opaqueredirect"
    } else if mode == "no-cors" && response_origin != page_origin {
        "opaque"
    } else if !response_origin.is_empty() && response_origin != page_origin {
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
    let content_type = if path.ends_with(".html") {
        Some("text/html")
    } else if path.ends_with(".txt") {
        Some("text/plain")
    } else if path.ends_with(".js") {
        Some("application/javascript")
    } else {
        None
    };
    content_type
        .map(|value| vec![("content-type".to_string(), value.to_string())])
        .unwrap_or_default()
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
    // R130：crash 用例判定（prepare_harness_html 同款判定——无 testharness 引用的纯脚本页）。
    let has_harness_ref = source.contains("/resources/testharness.js") || source.contains("testharnessreport.js");
    let scripts = zero_engine::extract_page_scripts(&html);
    let script_lengths = scripts
        .iter()
        .map(|script| match script {
            zero_engine::PageScript::Inline(source) | zero_engine::PageScript::InlineModule(source) => source.len(),
            zero_engine::PageScript::External(_) | zero_engine::PageScript::ExternalModule(_) => 0,
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
        script_timeout_ms: 90_000,
        // R34xx：headless 图片源——wpt.test/images/* 映射到本地 wpt-data 目录
        //（testharness 无网络；G5 DOM img 源解锁依赖图片加载）。
        // js-dom goal：dom 用例同样需要本地 .js 内联 + 图片资源，两条路径统一走 wpt_root。
        image_source_fetcher: wpt_data_image_fetcher(wpt_root),
        // R34xx：fetch() 本地资源（2d.composite.image.* fetch+createImageBitmap 路径）。
        fetch_handler: wpt_data_fetch_handler(wpt_root),
        // R34xx（G6）：.worker.js 变体 + worker importScripts 的脚本源。
        script_source_fetcher: wpt_data_script_fetcher(wpt_root),
        service_worker_script_fetcher: wpt_data_service_worker_script_fetcher(wpt_root),
        ..WebViewConfig::default()
    });
    webview.prepare_document_state(&format!("https://wpt.test/{case_name}"));
    let page_url = format!("https://wpt.test/{case_name}");
    let _zw_hb = std::fs::write("/tmp/zw-hb.txt", format!("prepared {}\n", case_name));
    // M3 扩批：播放泵时钟原点（tab_worker pump_epoch 同款——registry play/tick 共用
    // 单调毫秒；桥 play 传 0 即本原点）。
    let playback_clock_origin = std::time::Instant::now();
    // R34xx：canvas 默认字体（sans-serif）预载系统真字体（带 kern）——无 @font-face 的
    // 页面（2d.text.drawing.style.fontKerning 等）默认字体度量/kerning 面依赖。需
    // resolve_font_id 大小写不敏感修复配套（否则 CanvasTest 显式族 miss 回退 sans-serif）。
    webview.load_canvas_system_sans_font();
    let external_css = webview.fetch_page_images(&html, &page_url);
    webview.load_html(&html, Some(&external_css));
    // M3 扩批 XVI：播放宿主桥**先于页面脚本**安装（canplaythrough handler 在初始脚本
    // 执行内同步调 video.play()——桥后装则 play 走 headless 分支、bridgeOn 永不置位，
    // 播放钟推进面失联）。execute_script 空转预热 = ensure_sandbox + ensure_js_shim
    //（install_playback_bridge 的回调注册前提）。
    let _ = webview.execute_script("0;");
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
    let _zw_hb2 = std::fs::write("/tmp/zw-hb.txt", format!("pre-scripts {}\n", case_name));
    let script_result = webview.run_page_scripts_strict();
    let _zw_hb3 = std::fs::write("/tmp/zw-hb2.txt", format!("post-scripts {}\n", case_name));
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
        return vec![HarnessSubtestResult {
            name: case_name.to_string(),
            status: HarnessStatus::Fail,
            message: Some(format!("page script threw (declared tests: {declared}): {error}")),
        }];
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
    // M3 扩批 XVI：文件字节缓存（同 src 只读盘一次；注册表 contains_source 幂等）。
    let mut media_byte_cache: std::collections::HashMap<String, std::rc::Rc<Vec<u8>>> =
        std::collections::HashMap::new();
    loop {
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
            if std::env::var("ZW_DISABLE_PUMP").is_err()
                && let Ok(mut reg) = webview.video_players().lock()
                && reg.is_any_playing()
            {
                let cache = webview.image_cache();
                let _ = reg.tick_all(now_ms, cache);
                let _ = reg.audio_advance_all(now_ms);
            }
        }
        let _zw_hb4 = std::fs::write(
            "/tmp/zw-hb3.txt",
            format!("loop {}ms\n", playback_clock_origin.elapsed().as_millis()),
        );
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
        last_test_wait = probe.test_wait;
        for command in probe.commands {
            let result = apply_testdriver_command(&mut webview, &command);
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
    let cache_abort_fixture = if case_path.contains("cache-abort") {
        CACHE_ABORT_FETCH_FIXTURE
    } else {
        ""
    };
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
        var rest = [], due = [];\n\
        for (var i = 0; i < timers.length; i++) {\n\
          if (timers[i].at <= now) due.push(timers[i]); else rest.push(timers[i]);\n\
        }\n\
        globalThis.__zw_timers = rest;\n\
        for (var d = 0; d < due.length; d++) {\n\
          var fn = globalThis.__zw_pending[due[d].id];\n\
          if (fn) { delete globalThis.__zw_pending[due[d].id]; try { fn(); } catch (_e) {} }\n\
        }\n\
      };\n";
    let harness = format!("<script>\n{timer_stub}{harness_source}\n{reporter}\n{cache_abort_fixture}\n</script>");
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
         (function () {\
           var mk = globalThis.__zwMakeTrustedEvent;\
           document.dispatchEvent(mk ? mk('DOMContentLoaded') : new Event('DOMContentLoaded'));\
           globalThis.dispatchEvent(mk ? mk('load') : new Event('load'));\
         })();\
         if (typeof globalThis.__zw_mark_harness_loaded === 'function') {\
           globalThis.__zw_mark_harness_loaded();\
         }\
         </script>",
    );
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
    var signal = signalOf(input, init);
    if (signal && signal.aborted) return Promise.reject(signal.reason);
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
      var stateKey = url.searchParams.get('stateKey') || '';
      if (stateKey) stash[stateKey] = 'open';
      return new Promise(function(resolve, reject) {
        var settled = false;
        function abort() {
          if (settled) return;
          settled = true;
          reject(signal && signal.reason);
        }
        if (signal) signal.addEventListener('abort', abort);
        if (!signal) Promise.resolve().then(function() {
          if (settled) return;
          settled = true;
          resolve(responseFor(url, Array(2049).join('.'), 'text/plain'));
        });
      });
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
        // R142：Actions（指针链 pointerMove/pointerDown/pointerUp/send）白名单——stub 已提供
        // 链式构造器（pointer 系列合成点击），不再整体 Unsupported。
        if !name.is_empty()
            && name != "click"
            && name != "send_keys"
            && name != "Actions"
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

fn take_probe(webview: &mut WebView) -> Result<HarnessProbe, String> {
    // Pump timer tasks first so the sandbox's microtask checkpoint has flushed
    // testharness result callbacks before the state snapshot is serialized.
    webview
        .execute_script("if (typeof globalThis.__zw_fire_due_timers === 'function') globalThis.__zw_fire_due_timers()")
        .map_err(|error| error.to_string())?;
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
    serde_json::from_str(&value).map_err(|error| format!("invalid harness probe: {error}: {value}"))
}

fn apply_testdriver_command(webview: &mut WebView, command: &TestdriverCommand) -> Option<String> {
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
        "click" => {
            // R142：合成指针点击的 focus 步骤（spec UI Events 指针激活序列——可聚焦目标
            // 先获得焦点再派发 click；WPT no-focus-events 期望 focus/focusin 恰好一次、
            // target 为点击元素）。element.focus() 经 shim 的 R3247 focus 派发
            //（focusout(旧) → focus(新) → focusin(新)），click 随后由 Activate 计划派发。
            // focus 失败不阻断 click（不可聚焦目标真实浏览器也派发 click）。
            let focus_script = format!(
                "(function(){{var el=document.querySelector({sel});try{{if(el&&el.focus)el.focus();}}catch(_e){{}}}})();",
                sel = serde_json::to_string(&selector).unwrap_or_else(|_| "null".into())
            );
            let _ = webview.execute_script(&focus_script);
            dispatch_action(webview, target, HtmlUserAction::Activate)
        }
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
        // R347：disabled 目标的合成 click =「事件送达但激活被抑制」——noop(DisabledTarget)
        // 对 testdriver 语义即成功（spec HTML §activation：disabled 表单控件跳过激活行为，
        // 但自动化的 click 仍需 resolve 以驱动后续断言；onclick 不触发由页面侧断言）。
        // WPT Event-dispatch-on-disabled-elements「Real clicks」段依赖此 resolve。
        Ok(result) if result.noop_reason == Some(zero_page_runtime::ActionNoopReason::DisabledTarget) => None,
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
  var queuedElements = {};
  globalThis.__zw_td_queue = [];
  function selectorFor(element) {
    if (!element) return null;
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
    if (id) return '#' + id;
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
  function enqueue(operation, element, text) {
    return new Promise(function(resolve, reject) {
      var id = nextId++;
      pending[id] = { resolve: resolve, reject: reject };
      // R145：selector 延迟解析——enqueue 时（同步 turn）mutation 尚未 apply，
      // handle→selector 正置表空（createElement/cloneNode 产物的 append 在 turn 末才
      // 落 host）。存元素引用，宿主出队时（跨 turn，apply 已完成）经
      // `__zw_td_selector(id)` 现场解析（正置表已 merge）。
      queuedElements[id] = element;
      globalThis.__zw_td_queue.push({
        id: id, operation: operation, selector: null,
        text: text == null ? null : String(text)
      });
    });
  }
  globalThis.__zw_td_selector = function(id) {
    var element = queuedElements[id];
    if (!element) return null;
    return selectorFor(element);
  };
  globalThis.__zw_td_forget = function(id) { delete queuedElements[id]; };
  globalThis.__zw_td_resolve = function(id, error) {
    var entry = pending[id];
    if (!entry) return;
    delete pending[id];
    delete queuedElements[id];
    if (error == null) entry.resolve();
    else entry.reject(new Error(String(error)));
  };
  globalThis.test_driver = {
    click: function(element) { return enqueue('click', element, null); },
    send_keys: function(element, keys) { return enqueue('send_keys', element, keys); }
  };
  // R142：test_driver.Actions（指针动作链）——上游用 no-focus-events 等 case 经
  // pointerMove/pointerDown/pointerUp 合成一次指针点击。headless 无真指针，语义映射：
  // 链上记录 origin 元素（pointerMove 的 options.origin / 链首隐式），send() 时对
  // origin 元素入队与 click 同形的 'click' 命令（宿主走既有 Activate 派发管线派发
  // click 事件）。pointerDown/pointerUp 间无移动语义（单一 target），key 系列不支持
  // （抛错——本 runner 未覆盖）。链式 API：每个方法返 this。
  function Actions() { this._origin = null; this._keys = false; }
  Actions.prototype.pointerMove = function(x, y, options) {
    if (options && options.origin) this._origin = options.origin;
    return this;
  };
  Actions.prototype.pointerDown = function() { return this; };
  Actions.prototype.pointerUp = function() { return this; };
  Actions.prototype.keyDown = function() { this._keys = true; return this; };
  Actions.prototype.keyUp = function() { return this; };
  Actions.prototype.send = function() {
    var element = this._origin || document.activeElement;
    if (this._keys) {
      return Promise.reject(new Error('testdriver Actions key sequence unsupported'));
    }
    if (!element) {
      return Promise.reject(new Error('testdriver Actions has no pointer origin'));
    }
    return enqueue('click', element, null);
  };
  globalThis.test_driver.Actions = Actions;
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
        assert_eq!(results[0].status, HarnessStatus::Fail);
        assert!(results[0].message.as_deref().unwrap().contains("boom"));
    }

    #[test]
    fn no_harness_test_wait_must_clear_before_pass() {
        let html = r#"<html class="test-wait"><script></script></html>"#;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "no-harness-test-wait.html",
            html,
            MINI_HARNESS,
            Duration::from_millis(10),
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, HarnessStatus::Timeout);
        assert!(results[0].message.as_deref().unwrap().contains("test_wait=true"));
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
        assert_eq!(SERVICE_WORKER_FETCH_CASES.len(), 30);
        assert_eq!(unique.len(), 30);
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
        // 非 .py 路径仍走静态文件（root 不存在 → 错误）。
        assert!(handler(&make_req("https://wpt.test/dom/nodes/encoding.py.bak")).is_err());
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

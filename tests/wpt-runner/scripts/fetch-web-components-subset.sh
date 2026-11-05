#!/usr/bin/env bash
# Fetch the pinned Web Components testharness subset used by the web-components goal
# (docs/goal/web-components.md, M1 / DC-1): custom-elements/ + shadow-dom/ +
# html/semantics/scripting-1/the-template-element/ window-executable main-thread cases.
#
# 与 fetch-dom-subset.sh 同构（同 WPT_REV pin + raw 拉单文件，显式清单）。
# wpt-data 整体 gitignored，用例按需 fetch、不入库。
#
# 用例级 skip 域（Support Envelope / DC-1 排除项，非静默丢弃——evidence 基线报告同记；
# 2026-11-06 shadow-dom-expansion M0 差集甄别后收窄，逐目录结论见
# docs/goal/shadow-dom-expansion/evidence/）：
# - *-ref.html / *-notref.html：reftest 参照页（渲染面，非可执行用例）
# - shadow-dom/ 渲染级 composed tree 用例（slot-fallback-content-00{1..8} /
#   layout-* / directionality-* / restyle-* / shadow-style-* / invalidate-sibling-* /
#   manual-slot-assignment-* / imperative-slot-layout-invalidation-* / user-agent-shadow* /
#   nested-hover-* / accesskey / touch-event-* / wheel-event-* / scroll-* /
#   offsetParent-* / offsetTop-* / focus-within-* / 命中测试四面
#   caretPositionFromPoint / elementFromPoint / highlightsFromPoint / offsetX-offsetY /
#   dragenter-dragleave related-target）：依赖 shadow 树进样式/布局/绘制或几何命中
#   ——已立项专项 goal 45 号 shadow-dom-rendering，不进本通道
# - shadow-dom/focus*/（焦点导航跨 shadow 边界——交互+渲染面）、crashtests/（纯崩溃
#   面，无子测试分母）、leaktests/（leak harness，GC 可观测性环境受限）、
#   reference/（空支撑页）
# - declarative/【M0 重审转正】parser/template 层（s168/s173 已落）入通道；
#   渲染级消费仍归 45 号。tentative/shadowrootadoptedstylesheets/ 同批入通道，
#   失败按 JS 语义域如实记账（M2 分档）
# - reference-target/【M0 重审转正】tentative 特性 JS 语义可修档入通道
#   （referenceTarget 属性未实现，M3 归因）
# - untriaged/【M0 重审转正】v0 遗留面按 content-level 甄别只入 testharness
#   用例（54 案）；reftest 对页不入
# - *.window.js（多全局包装形态——runner 无 .window.js wrapper，后续切片评估）、
#   *.xhtml/*.svg（XML/SVG parser 形态，slice 1 排除）
# - custom-elements/reactions/ 全导入（59 用例——per-API CEReactions 断言，
#   template 真实化 + CE 收口的 driving 面）
# - the-template-element 的 css-user-agent-style-sheet-*（reftest/渲染断言，渲染域）
# - the-template-element 的 resources/ iframe fixture 全量拉取（definitions/ 与
#   template-element/ 大量 testInIFrame 变体依赖）

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  if ! curl --fail --location --silent --show-error \
    --retry 4 --retry-all-errors --retry-delay 2 \
    --connect-timeout 10 --max-time 60 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"; then
    rm -f "${temporary}"
    echo "fetch failed: ${relative}" >&2
    return 1
  fi
  test -e "${temporary}"
  mv "${temporary}" "${target}"
}

SHARED=(
  "resources/testharness.js"
  "resources/testharnessreport.js"
  "common/utils.js"
  "common/get-host-info.sub.js"
  "html/resources/common.js"
)

CE_CASES=(
  "custom-elements/CustomElementRegistry-constructor-and-callbacks-are-held-strongly.html"
  "custom-elements/CustomElementRegistry-getName.html"
  "custom-elements/CustomElementRegistry.html"
  "custom-elements/Document-createElement-customized-builtins.html"
  "custom-elements/Document-createElement.html"
  "custom-elements/Document-createElementNS-customized-builtins.html"
  "custom-elements/Document-createElementNS-prefix-timing.html"
  "custom-elements/Document-createElementNS.html"
  "custom-elements/ElementInternals-accessibility.html"
  "custom-elements/ElementInternals-accessibility.tentative.html"
  "custom-elements/ElementInternals-role.html"
  "custom-elements/HTMLElement-attachInternals.html"
  "custom-elements/HTMLElement-constructor-customized-builtins.html"
  "custom-elements/HTMLElement-constructor.html"
  "custom-elements/adopted-callback.html"
  "custom-elements/append-children-to-new-parent-cycle.html"
  "custom-elements/attribute-changed-callback.html"
  "custom-elements/builtin-coverage.html"
  "custom-elements/connected-callbacks-html-fragment-parsing.html"
  "custom-elements/connected-callbacks-template.html"
  "custom-elements/connected-callbacks.html"
  "custom-elements/cross-realm-callback-report-exception.html"
  "custom-elements/custom-element-reaction-queue.html"
  "custom-elements/customized-built-in-constructor-exceptions.html"
  "custom-elements/disconnected-callbacks.html"
  "custom-elements/element-internals-aria-element-reflection.html"
  "custom-elements/element-internals-behaviors.tentative.html"
  "custom-elements/element-internals-shadowroot.html"
  "custom-elements/enqueue-custom-element-callback-reactions-inside-another-callback.html"
  "custom-elements/historical.html"
  "custom-elements/microtasks-and-constructors.html"
  "custom-elements/overwritten-customElements-global.html"
  "custom-elements/perform-microtask-checkpoint-before-construction.html"
  "custom-elements/prevent-extensions-crash.html"
  "custom-elements/pseudo-class-defined-customized-builtins.html"
  "custom-elements/pseudo-class-defined-print.html"
  "custom-elements/pseudo-class-defined.html"
  "custom-elements/range-and-constructors.html"
  "custom-elements/reaction-timing.html"
  "custom-elements/throw-on-dynamic-markup-insertion-counter-construct.html"
  "custom-elements/throw-on-dynamic-markup-insertion-counter-reactions.html"
  "custom-elements/upgrading.html"
  "custom-elements/when-defined-reentry-crash.html"
  "custom-elements/form-associated/ElementInternals-NotSupportedError.html"
  "custom-elements/form-associated/ElementInternals-behavior-accessibility.tentative.html"
  "custom-elements/form-associated/ElementInternals-form.html"
  "custom-elements/form-associated/ElementInternals-labels.html"
  "custom-elements/form-associated/ElementInternals-reportValidity-bubble.html"
  "custom-elements/form-associated/ElementInternals-reportValidity-delegatesFocus.html"
  "custom-elements/form-associated/ElementInternals-setFormValue-nullish-value.html"
  "custom-elements/form-associated/ElementInternals-setFormValue.html"
  "custom-elements/form-associated/ElementInternals-submit-behavior-dialog.tentative.html"
  "custom-elements/form-associated/ElementInternals-submit-behavior.tentative.html"
  "custom-elements/form-associated/ElementInternals-target-element-is-held-strongly.html"
  "custom-elements/form-associated/ElementInternals-validation.html"
  "custom-elements/form-associated/disabled-delegatesFocus.html"
  "custom-elements/form-associated/fieldset-elements.html"
  "custom-elements/form-associated/focusability.html"
  "custom-elements/form-associated/form-associated-callback.html"
  "custom-elements/form-associated/form-disabled-callback.html"
  "custom-elements/form-associated/form-elements-namedItem.html"
  "custom-elements/form-associated/form-reset-callback.html"
  "custom-elements/form-associated/label-delegatesFocus.html"
  "custom-elements/htmlconstructor/newtarget-customized-builtins.html"
  "custom-elements/htmlconstructor/newtarget.html"
  "custom-elements/parser/parser-constructs-custom-element-in-document-write.html"
  "custom-elements/parser/parser-constructs-custom-element-synchronously.html"
  "custom-elements/parser/parser-constructs-custom-elements-with-is.html"
  "custom-elements/parser/parser-constructs-custom-elements.html"
  "custom-elements/parser/parser-custom-element-in-foreign-content.html"
  "custom-elements/parser/parser-fallsback-to-unknown-element.html"
  "custom-elements/parser/parser-sets-attributes-and-children.html"
  "custom-elements/parser/parser-uses-constructed-element.html"
  "custom-elements/parser/parser-uses-registry-of-owner-document.html"
  "custom-elements/parser/serializing-html-fragments-customized-builtins.html"
  "custom-elements/reactions/customized-builtins/HTMLAreaElement.html"
  "custom-elements/reactions/customized-builtins/HTMLBaseElement.html"
  "custom-elements/reactions/customized-builtins/HTMLButtonElement.html"
  "custom-elements/reactions/customized-builtins/HTMLCanvasElement.html"
  "custom-elements/reactions/customized-builtins/HTMLDataElement.html"
  "custom-elements/reactions/customized-builtins/HTMLDetailsElement.html"
  "custom-elements/reactions/customized-builtins/HTMLEmbedElement.html"
  "custom-elements/reactions/customized-builtins/HTMLFieldSetElement.html"
  "custom-elements/reactions/customized-builtins/HTMLImageElement.html"
  "custom-elements/reactions/customized-builtins/HTMLInputElement.html"
  "custom-elements/reactions/customized-builtins/HTMLLIElement.html"
  "custom-elements/reactions/customized-builtins/HTMLLabelElement.html"
  "custom-elements/reactions/customized-builtins/HTMLMapElement.html"
  "custom-elements/reactions/customized-builtins/HTMLMediaElement.html"
  "custom-elements/reactions/customized-builtins/HTMLMetaElement.html"
  "custom-elements/reactions/customized-builtins/HTMLMeterElement.html"
  "custom-elements/reactions/customized-builtins/HTMLModElement.html"
  "custom-elements/reactions/customized-builtins/HTMLOListElement.html"
  "custom-elements/reactions/customized-builtins/HTMLOptGroupElement.html"
  "custom-elements/reactions/customized-builtins/HTMLParamElement.html"
  "custom-elements/reactions/customized-builtins/HTMLProgressElement.html"
  "custom-elements/reactions/customized-builtins/HTMLQuoteElement.html"
  "custom-elements/reactions/customized-builtins/HTMLSlotElement.html"
  "custom-elements/reactions/customized-builtins/HTMLSourceElement.html"
  "custom-elements/reactions/customized-builtins/HTMLStyleElement.html"
  "custom-elements/reactions/customized-builtins/HTMLTableCellElement.html"
  "custom-elements/reactions/customized-builtins/HTMLTableColElement.html"
  "custom-elements/reactions/customized-builtins/HTMLTimeElement.html"
  "custom-elements/reactions/Animation.html"
  "custom-elements/reactions/AriaMixin-element-attributes.html"
  "custom-elements/reactions/AriaMixin-string-attributes.html"
  "custom-elements/reactions/AriaMixin-string-attributes.tentative.html"
  "custom-elements/reactions/Attr.html"
  "custom-elements/reactions/CSSStyleDeclaration.html"
  "custom-elements/reactions/ChildNode.html"
  "custom-elements/reactions/DOMStringMap.html"
  "custom-elements/reactions/DOMTokenList.html"
  "custom-elements/reactions/Document.html"
  "custom-elements/reactions/Element.html"
  "custom-elements/reactions/ElementContentEditable.html"
  "custom-elements/reactions/HTMLAnchorElement.html"
  "custom-elements/reactions/HTMLElement.html"
  "custom-elements/reactions/HTMLOptionElement.html"
  "custom-elements/reactions/HTMLOptionsCollection.html"
  "custom-elements/reactions/HTMLOutputElement.html"
  "custom-elements/reactions/HTMLSelectElement.html"
  "custom-elements/reactions/HTMLTableElement.html"
  "custom-elements/reactions/HTMLTableRowElement.html"
  "custom-elements/reactions/HTMLTableSectionElement.html"
  "custom-elements/reactions/HTMLTitleElement.html"
  "custom-elements/reactions/NamedNodeMap.html"
  "custom-elements/reactions/Node.html"
  "custom-elements/reactions/ParentNode.html"
  "custom-elements/reactions/Range.html"
  "custom-elements/reactions/Selection.html"
  "custom-elements/reactions/ShadowRoot.html"
  "custom-elements/reactions/with-exceptions.html"
  "custom-elements/registries/Construct.html"
  "custom-elements/registries/CustomElementRegistry-define.html"
  "custom-elements/registries/CustomElementRegistry-initialize.html"
  "custom-elements/registries/CustomElementRegistry-multi-register.html"
  "custom-elements/registries/CustomElementRegistry-upgrade.html"
  "custom-elements/registries/Document-createElement.html"
  "custom-elements/registries/Document-createElementNS.html"
  "custom-elements/registries/Document-customElementRegistry.html"
  "custom-elements/registries/Document-importNode.html"
  "custom-elements/registries/Element-customElementRegistry-exceptions.html"
  "custom-elements/registries/Element-customElementRegistry.html"
  "custom-elements/registries/Element-innerHTML.html"
  "custom-elements/registries/ShadowRoot-init-customElementRegistry.html"
  "custom-elements/registries/ShadowRoot-init-declarative.html"
  "custom-elements/registries/ShadowRoot-innerHTML.html"
  "custom-elements/registries/constructor-reentry-with-different-definition.html"
  "custom-elements/registries/define-customized-builtins.html"
  "custom-elements/registries/define.html"
  "custom-elements/registries/element-mutation-null-registry-removal.html"
  "custom-elements/registries/element-mutation.html"
  "custom-elements/registries/per-document.html"
  "custom-elements/registries/scoped-custom-element-registry-customelementregistry-attribute.html"
  "custom-elements/registries/scoped-registry-append.html"
  "custom-elements/registries/scoped-registry-define-upgrade-criteria.html"
  "custom-elements/registries/scoped-registry-define-upgrade-order.html"
  "custom-elements/registries/scoped-registry-effective-global-registry.html"
  "custom-elements/registries/scoped-registry-initialize-upgrades.html"
  "custom-elements/registries/scoped-registry-initialize.html"
  "custom-elements/registries/scoped-registry-registry-define-get-etc.html"
  "custom-elements/registries/upgrade.html"
  "custom-elements/registries/valid-custom-element-names.html"
  "custom-elements/state/ElementInternals-states.html"
  "custom-elements/state/state-css-selector-nth-of.html"
  "custom-elements/state/state-css-selector-shadow-dom.html"
  "custom-elements/state/state-css-selector.html"
  "custom-elements/state/state-pseudo-class.html"
  "custom-elements/upgrading/Document-importNode-customized-builtins.html"
  "custom-elements/upgrading/Document-importNode.html"
  "custom-elements/upgrading/Node-cloneNode-customized-builtins.html"
  "custom-elements/upgrading/Node-cloneNode.html"
  "custom-elements/upgrading/upgrade-custom-element-error-event.html"
  "custom-elements/upgrading/upgrading-enqueue-reactions.html"
  "custom-elements/upgrading/upgrading-parser-created-element.html"
)

CE_HELPERS=(
  "custom-elements/resources/custom-elements-helpers.js"
  "custom-elements/resources/empty-html-document.html"
  "custom-elements/resources/my-custom-element-html-document.html"
  "custom-elements/resources/navigation-destination.html"
  "custom-elements/reactions/resources/reactions.js"
)

SD_CASES=(
  "shadow-dom/Document-prototype-adoptNode.html"
  "shadow-dom/Document-prototype-currentScript.html"
  "shadow-dom/Document-prototype-importNode.html"
  "shadow-dom/Element-interface-attachShadow-custom-element.html"
  "shadow-dom/Element-interface-attachShadow.html"
  "shadow-dom/Element-interface-shadowRoot-attribute.html"
  "shadow-dom/Extensions-to-Event-Interface.html"
  "shadow-dom/HTMLSlotElement-interface.html"
  "shadow-dom/Node-prototype-cloneNode.html"
  "shadow-dom/Range-prototype-insertNode.html"
  "shadow-dom/ShadowRoot-interface.html"
  "shadow-dom/Slottable-mixin.html"
  "shadow-dom/assign-slottables-after-removing-shadow-tree-from-document.html"
  "shadow-dom/attach-shadow-non-html-namespace.html"
  "shadow-dom/attachShadow-with-ShadowRoot.html"
  "shadow-dom/build-deep-detached-shadow-then-append-text.html"
  "shadow-dom/capturing-and-bubbling-event-listeners-across-shadow-trees.html"
  "shadow-dom/cross-shadow-boundary-selection-remove-splittext-crash.html"
  "shadow-dom/cross-shadow-boundary-selection-splittext-crash.html"
  "shadow-dom/event-composed-path-after-dom-mutation.html"
  "shadow-dom/event-composed-path-with-related-target.html"
  "shadow-dom/event-composed-path.html"
  "shadow-dom/event-composed.html"
  "shadow-dom/event-dispatch-order.tentative.html"
  "shadow-dom/event-inside-shadow-tree.html"
  "shadow-dom/event-inside-slotted-node.html"
  "shadow-dom/event-on-pseudo-element-crash.html"
  "shadow-dom/event-post-dispatch-no-listeners.html"
  "shadow-dom/event-post-dispatch.html"
  "shadow-dom/event-with-related-target.html"
  "shadow-dom/execcommand-insertList-in-shadow.html"
  "shadow-dom/form-control-form-attribute.html"
  "shadow-dom/getElementById-dynamic-001.html"
  "shadow-dom/getElementById-dynamic-002.html"
  "shadow-dom/historical.html"
  "shadow-dom/imperative-slot-api-crash.html"
  "shadow-dom/imperative-slot-api-cross-shadow-root.html"
  "shadow-dom/imperative-slot-api-disconnected.html"
  "shadow-dom/imperative-slot-api-slotchange.html"
  "shadow-dom/imperative-slot-api.html"
  "shadow-dom/imperative-slot-assign-not-slotable-crash.html"
  "shadow-dom/imperative-slot-fallback-clear.html"
  "shadow-dom/imperative-slot-initial-fallback.html"
  "shadow-dom/inserting-fragment-under-shadow-host.html"
  "shadow-dom/invalidate-shadow-dom-crash.html"
  "shadow-dom/nested-slot-remove-crash.html"
  "shadow-dom/shadow-host-child-restyle-crash.html"
  "shadow-dom/shadow-root-clonable.html"
  "shadow-dom/slot-dir-attach-child-crash.html"
  "shadow-dom/slot-dir-attach-child-details-crash.html"
  "shadow-dom/slot-dir-attach-child-meter-crash.html"
  "shadow-dom/slot-dir-attach-child-progress-crash.html"
  "shadow-dom/slot-dir-attach-child-textarea-crash.html"
  "shadow-dom/slot-reconciliation-at-node-removal.html"
  "shadow-dom/slot-with-slottable-slot-in-display-none-subtree-crash.html"
  "shadow-dom/slotchange-customelements.html"
  "shadow-dom/slotchange-event.html"
  "shadow-dom/slotchange.html"
  "shadow-dom/slots-fallback-in-document.html"
  "shadow-dom/slots-fallback.html"
  "shadow-dom/slots-outside-shadow-dom.html"
  "shadow-dom/slots.html"
)

SD_HELPERS=(
  "shadow-dom/resources/Document-prototype-currentScript-helper.js"
  "shadow-dom/resources/event-path-test-helpers.js"
  "shadow-dom/resources/shadow-dom-utils.js"
  "shadow-dom/resources/shadow-dom.js"
)

# ---- shadow-dom-expansion 扩面批（docs/goal/shadow-dom-expansion.md M0 差集
# 甄别 + M1 通道扩面，2026-11-06；pin 不变，清单按上游 pin 树 git trees API
# 全量甄别后逐档准入，与 runner 侧 wc_case_skipped 两侧同源）：
# - declarative/ parser/template 层：s168（materialize_declarative_shadow_roots）
#   / s173（html5ever shadowrootmode 绕道）已落，M0 重审把「declarative 范围外」
#   排除条款收窄为「渲染级消费归 45 号」——本层 JS 语义面转正入通道。
# - declarative/tentative/shadowrootadoptedstylesheets/：constructable
#   stylesheets shim 已有，module script 样式加载面 M2 基线分档（JS 语义域，
#   失败如实入账不预排除）。
# - reference-target/（tentative 特性）：referenceTarget 属性未实现——JS 语义
#   可修档入池，M2 基线 + M3 归因（就地修或具名挂账）。
# - untriaged/：v0 时代遗留面，只入 content-level 甄别出的 testharness 用例
#   （reftest 对页与 resources/ 不入通道）。
SD_DECL_CASES=(
  "shadow-dom/declarative/declarative-after-attachshadow.html"
  "shadow-dom/declarative/declarative-parser-interaction.html"
  "shadow-dom/declarative/declarative-shadow-dom-attachment.html"
  "shadow-dom/declarative/declarative-shadow-dom-available-to-element-internals.html"
  "shadow-dom/declarative/declarative-shadow-dom-basic.html"
  "shadow-dom/declarative/declarative-shadow-dom-opt-in.html"
  "shadow-dom/declarative/declarative-shadow-dom-repeats-2.html"
  "shadow-dom/declarative/declarative-shadow-dom-repeats-slot-assignment.html"
  "shadow-dom/declarative/declarative-shadow-dom-repeats.html"
  "shadow-dom/declarative/declarative-shadow-dom-serialization.html"
  "shadow-dom/declarative/declarative-shadow-dom-slot-assignment-serialization.html"
  "shadow-dom/declarative/declarative-shadow-dom-slot-assignment.html"
  "shadow-dom/declarative/declarative-shadow-dom-write-to-iframe.html"
  "shadow-dom/declarative/declarative-with-disabled-shadow.html"
  "shadow-dom/declarative/gethtml-ordering.html"
  "shadow-dom/declarative/gethtml.html"
  "shadow-dom/declarative/innerhtml-before-closing-tag.html"
  "shadow-dom/declarative/innerhtml-on-ordinary-template.html"
  "shadow-dom/declarative/move-template-before-closing-tag.html"
  "shadow-dom/declarative/script-access.html"
)

SD_DECL_HELPERS=(
  "shadow-dom/declarative/support/declarative-child-frame.html"
)

SD_DECL_TENTATIVE_CASES=(
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-disconnect-iframe.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-disconnect.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-duplicate-bare-specifier.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-duplicate-importmap.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-duplicate.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-failure.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-mutation-clear.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-mutation-insertrule.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-mutation-reinsert-same.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-mutation-reinsert-swap.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-mutation-remove-middle.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-mutation-replacesync.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-ordering.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch-shared.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-async-fetch.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-basic.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-clonenode-async-fetch.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-clonenode.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-document-parse-crash.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-fetched-module.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-idl-absent-and-empty.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-idl-feature-detection.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-idl-reflection.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-invalid-specifier.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-modulepreload-async.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-modulepreload-basic.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-modulepreload-failure.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-modulepreload-identity.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-modulepreload-nested.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-modulepreload-ordering.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-modulepreload-shared.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-multiple.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-nested-async.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-nested-sync.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-placeholder-identity.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-same-document.html"
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/shadowrootadoptedstylesheets-serialization.html"
)

SD_DECL_TENTATIVE_HELPERS=(
  "shadow-dom/declarative/tentative/shadowrootadoptedstylesheets/support/helpers.js"
)

SD_REFTARGET_CASES=(
  "shadow-dom/reference-target/tentative/aria-labelledby.html"
  "shadow-dom/reference-target/tentative/commandfor.html"
  "shadow-dom/reference-target/tentative/dom-mutation.html"
  "shadow-dom/reference-target/tentative/event-path.html"
  "shadow-dom/reference-target/tentative/form.html"
  "shadow-dom/reference-target/tentative/gethtml-serialization.html"
  "shadow-dom/reference-target/tentative/interestfor.tentative.html"
  "shadow-dom/reference-target/tentative/label-descendant.html"
  "shadow-dom/reference-target/tentative/label-for.html"
  "shadow-dom/reference-target/tentative/popovertarget.html"
  "shadow-dom/reference-target/tentative/property-reflection-idl-setters.html"
  "shadow-dom/reference-target/tentative/property-reflection-imperative-setup.html"
  "shadow-dom/reference-target/tentative/property-reflection.html"
  "shadow-dom/reference-target/tentative/reference-target-basics.html"
  "shadow-dom/reference-target/tentative/shadowrootreferencetarget-idl-reflection.html"
)

SD_REFTARGET_HELPERS=(
  "shadow-dom/reference-target/tentative/resources/property-reflection-helper.js"
  "wai-aria/scripts/aria-utils.js"
)

# 顶层差集 content-level 甄别新增的 JS 语义候选（其余 50 个渲染级/ref 页归
# 45 号 shadow-dom-rendering，不入本通道）。
SD_TOP_DELTA_CASES=(
  "shadow-dom/input-element-list.html"
  "shadow-dom/input-type-radio.html"
)

# untriaged/ v0 遗留面 content-level 甄别出的 54 个 testharness 用例（reftest
# 对页 5+5 不入通道；resources/ 两页是 iframe fixture，拉取不扫描）。
SD_UNTRIAGED_CASES=(
  "shadow-dom/untriaged/elements-and-dom-objects/extensions-to-element-interface/attributes/test-006.html"
  "shadow-dom/untriaged/elements-and-dom-objects/extensions-to-element-interface/methods/test-001.html"
  "shadow-dom/untriaged/elements-and-dom-objects/extensions-to-element-interface/methods/test-002.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-attributes/activeElement-confirm-return-null.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-attributes/test-007.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-attributes/test-009.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-attributes/test-010.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-attributes/test-011.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-attributes/test-012.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-attributes/test-013.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-methods/test-001.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-methods/test-004.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-methods/test-006.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-methods/test-007.html"
  "shadow-dom/untriaged/elements-and-dom-objects/shadowroot-object/shadowroot-methods/test-010.html"
  "shadow-dom/untriaged/events/event-dispatch/test-002.html"
  "shadow-dom/untriaged/events/event-dispatch/test-003.html"
  "shadow-dom/untriaged/events/event-retargeting/test-001.html"
  "shadow-dom/untriaged/events/event-retargeting/test-003.html"
  "shadow-dom/untriaged/events/retargeting-focus-events/test-001.html"
  "shadow-dom/untriaged/events/retargeting-focus-events/test-002.html"
  "shadow-dom/untriaged/events/retargeting-focus-events/test-003.html"
  "shadow-dom/untriaged/events/retargeting-relatedtarget/test-001.html"
  "shadow-dom/untriaged/events/retargeting-relatedtarget/test-002.html"
  "shadow-dom/untriaged/events/retargeting-relatedtarget/test-003.html"
  "shadow-dom/untriaged/events/test-001.html"
  "shadow-dom/untriaged/html-elements-in-shadow-trees/html-forms/test-001.html"
  "shadow-dom/untriaged/html-elements-in-shadow-trees/html-forms/test-002.html"
  "shadow-dom/untriaged/html-elements-in-shadow-trees/html-forms/test-003.html"
  "shadow-dom/untriaged/html-elements-in-shadow-trees/inert-html-elements/test-001.html"
  "shadow-dom/untriaged/html-elements-in-shadow-trees/inert-html-elements/test-002.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/dom-tree-accessors-001.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/dom-tree-accessors-002.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/ownerdocument-001.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/ownerdocument-002.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/selectors-api-001.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/selectors-api-002.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/shadow-root-001.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/test-005.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/test-007.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/test-009.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/test-011.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/window-named-properties-001.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/window-named-properties-002.html"
  "shadow-dom/untriaged/shadow-trees/upper-boundary-encapsulation/window-named-properties-003.html"
  "shadow-dom/untriaged/styles/test-001.html"
  "shadow-dom/untriaged/styles/test-003.html"
  "shadow-dom/untriaged/styles/test-005.html"
  "shadow-dom/untriaged/styles/test-008.html"
  "shadow-dom/untriaged/user-interaction/active-element/test-001.html"
  "shadow-dom/untriaged/user-interaction/active-element/test-002.html"
  "shadow-dom/untriaged/user-interaction/editing/inheritance-of-content-editable-001.html"
  "shadow-dom/untriaged/user-interaction/ranges-and-selections/test-001.html"
  "shadow-dom/untriaged/user-interaction/ranges-and-selections/test-002.html"
)

SD_UNTRIAGED_HELPERS=(
  "shadow-dom/untriaged/resources/blank.html"
  "shadow-dom/untriaged/resources/bobs_page.html"
)

TPL_CASES=(
  "html/semantics/scripting-1/the-template-element/reflect-html-for.tentative.html"
  "html/semantics/scripting-1/the-template-element/template-construction-in-inactive-document-crash.html"
  "html/semantics/scripting-1/the-template-element/template-table-crash.html"
  "html/semantics/scripting-1/the-template-element/additions-to-parsing-xhtml-documents/node-document.html"
  "html/semantics/scripting-1/the-template-element/additions-to-parsing-xhtml-documents/template-child-nodes.html"
  "html/semantics/scripting-1/the-template-element/additions-to-serializing-xhtml-documents/outerhtml.html"
  "html/semantics/scripting-1/the-template-element/additions-to-the-steps-to-clone-a-node/template-clone-children.html"
  "html/semantics/scripting-1/the-template-element/additions-to-the-steps-to-clone-a-node/templates-copy-document-owner.html"
  "html/semantics/scripting-1/the-template-element/definitions/template-contents-owner-document-type.html"
  "html/semantics/scripting-1/the-template-element/definitions/template-contents-owner-test-001.html"
  "html/semantics/scripting-1/the-template-element/definitions/template-contents-owner-test-002.html"
  "html/semantics/scripting-1/the-template-element/definitions/template-contents.html"
  "html/semantics/scripting-1/the-template-element/innerhtml-on-templates/innerhtml.html"
  "html/semantics/scripting-1/the-template-element/serializing-html-templates/outerhtml.html"
  "html/semantics/scripting-1/the-template-element/template-element/content-attribute.html"
  "html/semantics/scripting-1/the-template-element/template-element/node-document-changes.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-as-a-descendant.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-construction-in-inactive-document-crash.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-content-hierarcy.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-content-in-inactive-document-crash.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-content-move-to-inactive-document-crash.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-content-node-document.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-content.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-descendant-body.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-descendant-frameset.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-descendant-head.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-element-clone-into-inactive-document-crash.html"
  "html/semantics/scripting-1/the-template-element/template-element/template-set-inner-html-in-inactive-document-crash.html"
)

TPL_HELPERS=(
  "html/semantics/scripting-1/the-template-element/resources/end-template-tag-in-body.html"
  "html/semantics/scripting-1/the-template-element/resources/end-template-tag-in-head.html"
  "html/semantics/scripting-1/the-template-element/resources/frameset-end-tag.html"
  "html/semantics/scripting-1/the-template-element/resources/head-template-contents-div-no-end-tag.html"
  "html/semantics/scripting-1/the-template-element/resources/head-template-contents-table-no-end-tag.html"
  "html/semantics/scripting-1/the-template-element/resources/html-start-tag.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents-attribute.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents-body.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents-div-no-end-tag.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents-empty.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents-frameset.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents-head.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents-html.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents-nested.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents-table-no-end-tag.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents-text.html"
  "html/semantics/scripting-1/the-template-element/resources/template-contents.html"
  "html/semantics/scripting-1/the-template-element/resources/template-descendant-body.html"
  "html/semantics/scripting-1/the-template-element/resources/template-descendant-frameset.html"
  "html/semantics/scripting-1/the-template-element/resources/template-descendant-head.html"
  "html/semantics/scripting-1/the-template-element/resources/two-templates.html"
)

for rel in "${SHARED[@]}" "${CE_CASES[@]}" "${CE_HELPERS[@]}" "${SD_CASES[@]}" "${SD_HELPERS[@]}" \
  "${SD_DECL_CASES[@]}" "${SD_DECL_HELPERS[@]}" "${SD_DECL_TENTATIVE_CASES[@]}" "${SD_DECL_TENTATIVE_HELPERS[@]}" \
  "${SD_REFTARGET_CASES[@]}" "${SD_REFTARGET_HELPERS[@]}" \
  "${SD_TOP_DELTA_CASES[@]}" "${SD_UNTRIAGED_CASES[@]}" "${SD_UNTRIAGED_HELPERS[@]}" \
  "${TPL_CASES[@]}" "${TPL_HELPERS[@]}"; do
  fetch_raw "$rel"
done

echo "web-components subset ready: ${#CE_CASES[@]}+${#SD_CASES[@]}+${#SD_DECL_CASES[@]}+${#SD_DECL_TENTATIVE_CASES[@]}+${#SD_REFTARGET_CASES[@]}+${#SD_TOP_DELTA_CASES[@]}+${#SD_UNTRIAGED_CASES[@]}+${#TPL_CASES[@]} cases + ${#CE_HELPERS[@]}+${#SD_HELPERS[@]}+${#SD_DECL_HELPERS[@]}+${#SD_DECL_TENTATIVE_HELPERS[@]}+${#SD_REFTARGET_HELPERS[@]}+${#SD_UNTRIAGED_HELPERS[@]}+${#TPL_HELPERS[@]} helpers @ ${WPT_REV:0:12}"

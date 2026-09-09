# Web Components 三目录 WPT 基线（web-components M1 切片 1 / DC-1）

- 日期：2026-09-10
- WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`（与 indexeddb/fs/selection 导入同版 pin）
- 执行通道：`make testharness-web-components`（test-guard 包裹；wpt-runner `testharness-web-components` 模式）
- 用例：265（custom-elements 175 + shadow-dom 62 + the-template-element 28；window 主线程 .html 面）
- Subtests：4730（Pass 437 / Fail 4284 / Unsupported 3 / Timeout 4 / PreconditionFailed 2，**通过率 9%**）

**这是立项时点现有实现的真水平标定**：Custom Elements（quickjs/v8 双路径既有 registry + 三回调）、
template（parser 占位 + shim content 视图）、slot（Rust 孤立数据结构、engine 零接线）、Shadow DOM
JS 面基础（attachShadow/查询/retarget 基础）。M1-M3 修复的对照起点。

## 分目录

| 目录 | Cases | Subtests | Pass | 通过率 |
|---|---:|---:|---:|---:|
| custom-elements | 175 | 3787 | 340 | 9% |
| shadow-dom | 62 | 647 | 84 | 13% |
| the-template-element（html/semantics/scripting-1） | 28 | 296 | 13 | 4% |
| **合计** | **265** | **4730** | **437** | **9%** |

## 分用例（subtests ≥ 1；全 265 行见 JSON）

| 用例 | Subtests | Pass | Fail/其他 |
|---|---:|---:|---:|
| `custom-elements/CustomElementRegistry-constructor-and-callbacks-are-held-strongly.html` | 5 | 1 | 4 |
| `custom-elements/CustomElementRegistry-getName.html` | 4 | 2 | 2 |
| `custom-elements/CustomElementRegistry.html` | 46 | 13 | 33 |
| `custom-elements/Document-createElement-customized-builtins.html` | 4 | 1 | 3 |
| `custom-elements/Document-createElement.html` | 36 | 9 | 27 |
| `custom-elements/Document-createElementNS-customized-builtins.html` | 3 | 1 | 2 |
| `custom-elements/Document-createElementNS-prefix-timing.html` | 3 | 0 | 3 |
| `custom-elements/Document-createElementNS.html` | 4 | 1 | 3 |
| `custom-elements/ElementInternals-accessibility.html` | 50 | 0 | 50 |
| `custom-elements/ElementInternals-accessibility.tentative.html` | 3 | 0 | 3 |
| `custom-elements/ElementInternals-role.html` | 1 | 0 | 1 |
| `custom-elements/HTMLElement-attachInternals.html` | 4 | 0 | 4 |
| `custom-elements/HTMLElement-constructor-customized-builtins.html` | 2 | 0 | 2 |
| `custom-elements/HTMLElement-constructor.html` | 12 | 1 | 11 |
| `custom-elements/adopted-callback.html` | 71 | 1 | 70 |
| `custom-elements/append-children-to-new-parent-cycle.html` | 1 | 1 | 0 |
| `custom-elements/attribute-changed-callback.html` | 13 | 0 | 13 |
| `custom-elements/builtin-coverage.html` | 441 | 110 | 331 |
| `custom-elements/connected-callbacks-html-fragment-parsing.html` | 8 | 0 | 8 |
| `custom-elements/connected-callbacks-template.html` | 1 | 0 | 1 |
| `custom-elements/connected-callbacks.html` | 40 | 3 | 37 |
| `custom-elements/cross-realm-callback-report-exception.html` | 5 | 0 | 5 |
| `custom-elements/custom-element-reaction-queue.html` | 6 | 0 | 6 |
| `custom-elements/customized-built-in-constructor-exceptions.html` | 5 | 0 | 5 |
| `custom-elements/disconnected-callbacks.html` | 40 | 3 | 37 |
| `custom-elements/element-internals-aria-element-reflection.html` | 1 | 0 | 1 |
| `custom-elements/element-internals-behaviors.tentative.html` | 15 | 6 | 9 |
| `custom-elements/element-internals-shadowroot.html` | 7 | 0 | 7 |
| `custom-elements/enqueue-custom-element-callback-reactions-inside-another-callback.html` | 8 | 0 | 8 |
| `custom-elements/form-associated/ElementInternals-NotSupportedError.html` | 1 | 0 | 1 |
| `custom-elements/form-associated/ElementInternals-behavior-accessibility.tentative.html` | 1 | 0 | 1 |
| `custom-elements/form-associated/ElementInternals-form.html` | 2 | 0 | 2 |
| `custom-elements/form-associated/ElementInternals-labels.html` | 3 | 0 | 3 |
| `custom-elements/form-associated/ElementInternals-reportValidity-bubble.html` | 1 | 1 | 0 |
| `custom-elements/form-associated/ElementInternals-reportValidity-delegatesFocus.html` | 1 | 1 | 0 |
| `custom-elements/form-associated/ElementInternals-setFormValue-nullish-value.html` | 2 | 0 | 2 |
| `custom-elements/form-associated/ElementInternals-setFormValue.html` | 55 | 0 | 55 |
| `custom-elements/form-associated/ElementInternals-submit-behavior-dialog.tentative.html` | 2 | 0 | 2 |
| `custom-elements/form-associated/ElementInternals-submit-behavior.tentative.html` | 1 | 0 | 1 |
| `custom-elements/form-associated/ElementInternals-target-element-is-held-strongly.html` | 1 | 0 | 1 |
| `custom-elements/form-associated/ElementInternals-validation.html` | 14 | 0 | 14 |
| `custom-elements/form-associated/disabled-delegatesFocus.html` | 1 | 0 | 1 |
| `custom-elements/form-associated/fieldset-elements.html` | 1 | 0 | 1 |
| `custom-elements/form-associated/focusability.html` | 1 | 0 | 1 |
| `custom-elements/form-associated/form-associated-callback.html` | 5 | 0 | 5 |
| `custom-elements/form-associated/form-disabled-callback.html` | 10 | 1 | 9 |
| `custom-elements/form-associated/form-elements-namedItem.html` | 3 | 0 | 3 |
| `custom-elements/form-associated/form-reset-callback.html` | 3 | 0 | 3 |
| `custom-elements/form-associated/label-delegatesFocus.html` | 2 | 0 | 2 |
| `custom-elements/historical.html` | 3 | 3 | 0 |
| `custom-elements/htmlconstructor/newtarget-customized-builtins.html` | 10 | 0 | 10 |
| `custom-elements/htmlconstructor/newtarget.html` | 10 | 2 | 8 |
| `custom-elements/microtasks-and-constructors.html` | 5 | 2 | 3 |
| `custom-elements/overwritten-customElements-global.html` | 4 | 0 | 4 |
| `custom-elements/parser/parser-constructs-custom-element-in-document-write.html` | 1 | 0 | 1 |
| `custom-elements/parser/parser-constructs-custom-element-synchronously.html` | 1 | 0 | 1 |
| `custom-elements/parser/parser-constructs-custom-elements-with-is.html` | 2 | 2 | 0 |
| `custom-elements/parser/parser-constructs-custom-elements.html` | 2 | 2 | 0 |
| `custom-elements/parser/parser-custom-element-in-foreign-content.html` | 1 | 0 | 1 |
| `custom-elements/parser/parser-fallsback-to-unknown-element.html` | 4 | 0 | 4 |
| `custom-elements/parser/parser-sets-attributes-and-children.html` | 5 | 1 | 4 |
| `custom-elements/parser/parser-uses-constructed-element.html` | 2 | 0 | 2 |
| `custom-elements/parser/parser-uses-registry-of-owner-document.html` | 1 | 0 | 1 |
| `custom-elements/parser/serializing-html-fragments-customized-builtins.html` | 3 | 1 | 2 |
| `custom-elements/perform-microtask-checkpoint-before-construction.html` | 2 | 0 | 2 |
| `custom-elements/prevent-extensions-crash.html` | 1 | 1 | 0 |
| `custom-elements/pseudo-class-defined-customized-builtins.html` | 1 | 0 | 1 |
| `custom-elements/pseudo-class-defined-print.html` | 1 | 1 | 0 |
| `custom-elements/pseudo-class-defined.html` | 1 | 0 | 1 |
| `custom-elements/range-and-constructors.html` | 2 | 0 | 2 |
| `custom-elements/reaction-timing.html` | 3 | 1 | 2 |
| `custom-elements/reactions/Animation.html` | 3 | 1 | 2 |
| `custom-elements/reactions/AriaMixin-element-attributes.html` | 16 | 0 | 16 |
| `custom-elements/reactions/AriaMixin-string-attributes.html` | 80 | 0 | 80 |
| `custom-elements/reactions/AriaMixin-string-attributes.tentative.html` | 8 | 0 | 8 |
| `custom-elements/reactions/Attr.html` | 2 | 1 | 1 |
| `custom-elements/reactions/CSSStyleDeclaration.html` | 1 | 0 | 1 |
| `custom-elements/reactions/ChildNode.html` | 7 | 1 | 6 |
| `custom-elements/reactions/DOMStringMap.html` | 8 | 4 | 4 |
| `custom-elements/reactions/DOMTokenList.html` | 19 | 7 | 12 |
| `custom-elements/reactions/Document.html` | 12 | 0 | 12 |
| `custom-elements/reactions/Element.html` | 47 | 18 | 29 |
| `custom-elements/reactions/ElementContentEditable.html` | 2 | 0 | 2 |
| `custom-elements/reactions/HTMLAnchorElement.html` | 1 | 0 | 1 |
| `custom-elements/reactions/HTMLElement.html` | 22 | 0 | 22 |
| `custom-elements/reactions/HTMLOptionElement.html` | 1 | 0 | 1 |
| `custom-elements/reactions/HTMLOptionsCollection.html` | 5 | 0 | 5 |
| `custom-elements/reactions/HTMLOutputElement.html` | 2 | 0 | 2 |
| `custom-elements/reactions/HTMLSelectElement.html` | 5 | 0 | 5 |
| `custom-elements/reactions/HTMLTableElement.html` | 10 | 0 | 10 |
| `custom-elements/reactions/HTMLTableRowElement.html` | 1 | 0 | 1 |
| `custom-elements/reactions/HTMLTableSectionElement.html` | 2 | 0 | 2 |
| `custom-elements/reactions/HTMLTitleElement.html` | 1 | 0 | 1 |
| `custom-elements/reactions/NamedNodeMap.html` | 14 | 6 | 8 |
| `custom-elements/reactions/Node.html` | 14 | 6 | 8 |
| `custom-elements/reactions/ParentNode.html` | 4 | 2 | 2 |
| `custom-elements/reactions/Range.html` | 10 | 4 | 6 |
| `custom-elements/reactions/Selection.html` | 1 | 1 | 0 |
| `custom-elements/reactions/ShadowRoot.html` | 3 | 1 | 2 |
| `custom-elements/reactions/customized-builtins/HTMLAreaElement.html` | 16 | 0 | 16 |
| `custom-elements/reactions/customized-builtins/HTMLBaseElement.html` | 4 | 0 | 4 |
| `custom-elements/reactions/customized-builtins/HTMLButtonElement.html` | 20 | 0 | 20 |
| `custom-elements/reactions/customized-builtins/HTMLCanvasElement.html` | 4 | 0 | 4 |
| `custom-elements/reactions/customized-builtins/HTMLDataElement.html` | 2 | 0 | 2 |
| `custom-elements/reactions/customized-builtins/HTMLDetailsElement.html` | 1 | 0 | 1 |
| `custom-elements/reactions/customized-builtins/HTMLEmbedElement.html` | 8 | 0 | 8 |
| `custom-elements/reactions/customized-builtins/HTMLFieldSetElement.html` | 4 | 0 | 4 |
| `custom-elements/reactions/customized-builtins/HTMLImageElement.html` | 22 | 0 | 22 |
| `custom-elements/reactions/customized-builtins/HTMLInputElement.html` | 1 | 1 | 0 |
| `custom-elements/reactions/customized-builtins/HTMLLIElement.html` | 6 | 0 | 6 |
| `custom-elements/reactions/customized-builtins/HTMLLabelElement.html` | 2 | 0 | 2 |
| `custom-elements/reactions/customized-builtins/HTMLMapElement.html` | 2 | 0 | 2 |
| `custom-elements/reactions/customized-builtins/HTMLMediaElement.html` | 28 | 0 | 28 |
| `custom-elements/reactions/customized-builtins/HTMLMetaElement.html` | 6 | 0 | 6 |
| `custom-elements/reactions/customized-builtins/HTMLMeterElement.html` | 12 | 0 | 12 |
| `custom-elements/reactions/customized-builtins/HTMLModElement.html` | 8 | 0 | 8 |
| `custom-elements/reactions/customized-builtins/HTMLOListElement.html` | 6 | 0 | 6 |
| `custom-elements/reactions/customized-builtins/HTMLOptGroupElement.html` | 4 | 0 | 4 |
| `custom-elements/reactions/customized-builtins/HTMLParamElement.html` | 4 | 0 | 4 |
| `custom-elements/reactions/customized-builtins/HTMLProgressElement.html` | 4 | 0 | 4 |
| `custom-elements/reactions/customized-builtins/HTMLQuoteElement.html` | 4 | 0 | 4 |
| `custom-elements/reactions/customized-builtins/HTMLSlotElement.html` | 2 | 0 | 2 |
| `custom-elements/reactions/customized-builtins/HTMLSourceElement.html` | 10 | 0 | 10 |
| `custom-elements/reactions/customized-builtins/HTMLStyleElement.html` | 2 | 0 | 2 |
| `custom-elements/reactions/customized-builtins/HTMLTableCellElement.html` | 16 | 0 | 16 |
| `custom-elements/reactions/customized-builtins/HTMLTableColElement.html` | 2 | 0 | 2 |
| `custom-elements/reactions/customized-builtins/HTMLTimeElement.html` | 2 | 0 | 2 |
| `custom-elements/reactions/with-exceptions.html` | 1 | 0 | 1 |
| `custom-elements/registries/Construct.html` | 3 | 0 | 3 |
| `custom-elements/registries/CustomElementRegistry-define.html` | 3 | 0 | 3 |
| `custom-elements/registries/CustomElementRegistry-initialize.html` | 13 | 0 | 13 |
| `custom-elements/registries/CustomElementRegistry-multi-register.html` | 2 | 0 | 2 |
| `custom-elements/registries/CustomElementRegistry-upgrade.html` | 5 | 0 | 5 |
| `custom-elements/registries/Document-createElement.html` | 1 | 0 | 1 |
| `custom-elements/registries/Document-createElementNS.html` | 1 | 0 | 1 |
| `custom-elements/registries/Document-customElementRegistry.html` | 4 | 0 | 4 |
| `custom-elements/registries/Document-importNode.html` | 1 | 0 | 1 |
| `custom-elements/registries/Element-customElementRegistry-exceptions.html` | 3 | 0 | 3 |
| `custom-elements/registries/Element-customElementRegistry.html` | 11 | 0 | 11 |
| `custom-elements/registries/Element-innerHTML.html` | 12 | 0 | 12 |
| `custom-elements/registries/ShadowRoot-init-customElementRegistry.html` | 12 | 0 | 12 |
| `custom-elements/registries/ShadowRoot-init-declarative.html` | 3 | 0 | 3 |
| `custom-elements/registries/ShadowRoot-innerHTML.html` | 4 | 0 | 4 |
| `custom-elements/registries/constructor-reentry-with-different-definition.html` | 4 | 0 | 4 |
| `custom-elements/registries/define-customized-builtins.html` | 1 | 0 | 1 |
| `custom-elements/registries/define.html` | 1 | 0 | 1 |
| `custom-elements/registries/element-mutation-null-registry-removal.html` | 1 | 0 | 1 |
| `custom-elements/registries/element-mutation.html` | 15 | 0 | 15 |
| `custom-elements/registries/per-document.html` | 1 | 0 | 1 |
| `custom-elements/registries/scoped-custom-element-registry-customelementregistry-attribute.html` | 23 | 0 | 23 |
| `custom-elements/registries/scoped-registry-append.html` | 16 | 0 | 16 |
| `custom-elements/registries/scoped-registry-define-upgrade-criteria.html` | 14 | 1 | 13 |
| `custom-elements/registries/scoped-registry-define-upgrade-order.html` | 7 | 0 | 7 |
| `custom-elements/registries/scoped-registry-effective-global-registry.html` | 1 | 0 | 1 |
| `custom-elements/registries/scoped-registry-initialize-upgrades.html` | 12 | 0 | 12 |
| `custom-elements/registries/scoped-registry-initialize.html` | 1 | 0 | 1 |
| `custom-elements/registries/scoped-registry-registry-define-get-etc.html` | 7 | 0 | 7 |
| `custom-elements/registries/upgrade.html` | 5 | 0 | 5 |
| `custom-elements/registries/valid-custom-element-names.html` | 1975 | 104 | 1871 |
| `custom-elements/state/ElementInternals-states.html` | 4 | 0 | 4 |
| `custom-elements/state/state-css-selector-nth-of.html` | 2 | 0 | 2 |
| `custom-elements/state/state-css-selector-shadow-dom.html` | 3 | 0 | 3 |
| `custom-elements/state/state-css-selector.html` | 10 | 0 | 10 |
| `custom-elements/state/state-pseudo-class.html` | 8 | 2 | 6 |
| `custom-elements/throw-on-dynamic-markup-insertion-counter-construct.html` | 11 | 0 | 11 |
| `custom-elements/throw-on-dynamic-markup-insertion-counter-reactions.html` | 11 | 0 | 11 |
| `custom-elements/upgrading.html` | 28 | 5 | 23 |
| `custom-elements/upgrading/Document-importNode-customized-builtins.html` | 2 | 0 | 2 |
| `custom-elements/upgrading/Document-importNode.html` | 2 | 0 | 2 |
| `custom-elements/upgrading/Node-cloneNode-customized-builtins.html` | 1 | 0 | 1 |
| `custom-elements/upgrading/Node-cloneNode.html` | 9 | 2 | 7 |
| `custom-elements/upgrading/upgrade-custom-element-error-event.html` | 4 | 0 | 4 |
| `custom-elements/upgrading/upgrading-enqueue-reactions.html` | 5 | 0 | 5 |
| `custom-elements/upgrading/upgrading-parser-created-element.html` | 6 | 0 | 6 |
| `custom-elements/when-defined-reentry-crash.html` | 1 | 1 | 0 |
| `html/semantics/scripting-1/the-template-element/additions-to-parsing-xhtml-documents/node-document.html` | 5 | 0 | 5 |
| `html/semantics/scripting-1/the-template-element/additions-to-parsing-xhtml-documents/template-child-nodes.html` | 4 | 0 | 4 |
| `html/semantics/scripting-1/the-template-element/additions-to-serializing-xhtml-documents/outerhtml.html` | 3 | 0 | 3 |
| `html/semantics/scripting-1/the-template-element/additions-to-the-steps-to-clone-a-node/template-clone-children.html` | 3 | 0 | 3 |
| `html/semantics/scripting-1/the-template-element/additions-to-the-steps-to-clone-a-node/templates-copy-document-owner.html` | 5 | 0 | 5 |
| `html/semantics/scripting-1/the-template-element/definitions/template-contents-owner-document-type.html` | 4 | 0 | 4 |
| `html/semantics/scripting-1/the-template-element/definitions/template-contents-owner-test-001.html` | 2 | 0 | 2 |
| `html/semantics/scripting-1/the-template-element/definitions/template-contents-owner-test-002.html` | 3 | 0 | 3 |
| `html/semantics/scripting-1/the-template-element/definitions/template-contents.html` | 1 | 0 | 1 |
| `html/semantics/scripting-1/the-template-element/innerhtml-on-templates/innerhtml.html` | 4 | 0 | 4 |
| `html/semantics/scripting-1/the-template-element/reflect-html-for.tentative.html` | 1 | 0 | 1 |
| `html/semantics/scripting-1/the-template-element/serializing-html-templates/outerhtml.html` | 3 | 0 | 3 |
| `html/semantics/scripting-1/the-template-element/template-construction-in-inactive-document-crash.html` | 1 | 1 | 0 |
| `html/semantics/scripting-1/the-template-element/template-element/content-attribute.html` | 7 | 0 | 7 |
| `html/semantics/scripting-1/the-template-element/template-element/node-document-changes.html` | 6 | 0 | 6 |
| `html/semantics/scripting-1/the-template-element/template-element/template-as-a-descendant.html` | 12 | 6 | 6 |
| `html/semantics/scripting-1/the-template-element/template-element/template-construction-in-inactive-document-crash.html` | 1 | 1 | 0 |
| `html/semantics/scripting-1/the-template-element/template-element/template-content-hierarcy.html` | 2 | 0 | 2 |
| `html/semantics/scripting-1/the-template-element/template-element/template-content-in-inactive-document-crash.html` | 1 | 1 | 0 |
| `html/semantics/scripting-1/the-template-element/template-element/template-content-move-to-inactive-document-crash.html` | 1 | 1 | 0 |
| `html/semantics/scripting-1/the-template-element/template-element/template-content-node-document.html` | 3 | 0 | 3 |
| `html/semantics/scripting-1/the-template-element/template-element/template-content.html` | 216 | 0 | 216 |
| `html/semantics/scripting-1/the-template-element/template-element/template-descendant-body.html` | 1 | 1 | 0 |
| `html/semantics/scripting-1/the-template-element/template-element/template-descendant-frameset.html` | 3 | 0 | 3 |
| `html/semantics/scripting-1/the-template-element/template-element/template-descendant-head.html` | 1 | 0 | 1 |
| `html/semantics/scripting-1/the-template-element/template-element/template-element-clone-into-inactive-document-crash.html` | 1 | 0 | 1 |
| `html/semantics/scripting-1/the-template-element/template-element/template-set-inner-html-in-inactive-document-crash.html` | 1 | 1 | 0 |
| `html/semantics/scripting-1/the-template-element/template-table-crash.html` | 1 | 1 | 0 |
| `shadow-dom/Document-prototype-adoptNode.html` | 2 | 0 | 2 |
| `shadow-dom/Document-prototype-currentScript.html` | 8 | 0 | 8 |
| `shadow-dom/Document-prototype-importNode.html` | 2 | 0 | 2 |
| `shadow-dom/Element-interface-attachShadow-custom-element.html` | 6 | 3 | 3 |
| `shadow-dom/Element-interface-attachShadow.html` | 6 | 3 | 3 |
| `shadow-dom/Element-interface-shadowRoot-attribute.html` | 3 | 2 | 1 |
| `shadow-dom/Extensions-to-Event-Interface.html` | 16 | 4 | 12 |
| `shadow-dom/HTMLSlotElement-interface.html` | 18 | 1 | 17 |
| `shadow-dom/Node-prototype-cloneNode.html` | 4 | 2 | 2 |
| `shadow-dom/Range-prototype-insertNode.html` | 1 | 1 | 0 |
| `shadow-dom/ShadowRoot-interface.html` | 12 | 4 | 8 |
| `shadow-dom/Slottable-mixin.html` | 4 | 0 | 4 |
| `shadow-dom/assign-slottables-after-removing-shadow-tree-from-document.html` | 1 | 0 | 1 |
| `shadow-dom/attach-shadow-non-html-namespace.html` | 304 | 38 | 266 |
| `shadow-dom/attachShadow-with-ShadowRoot.html` | 2 | 0 | 2 |
| `shadow-dom/build-deep-detached-shadow-then-append-text.html` | 1 | 1 | 0 |
| `shadow-dom/capturing-and-bubbling-event-listeners-across-shadow-trees.html` | 5 | 0 | 5 |
| `shadow-dom/cross-shadow-boundary-selection-remove-splittext-crash.html` | 1 | 1 | 0 |
| `shadow-dom/cross-shadow-boundary-selection-splittext-crash.html` | 1 | 1 | 0 |
| `shadow-dom/event-composed-path-after-dom-mutation.html` | 2 | 0 | 2 |
| `shadow-dom/event-composed-path-with-related-target.html` | 13 | 0 | 13 |
| `shadow-dom/event-composed-path.html` | 11 | 0 | 11 |
| `shadow-dom/event-composed.html` | 9 | 2 | 7 |
| `shadow-dom/event-dispatch-order.tentative.html` | 1 | 0 | 1 |
| `shadow-dom/event-inside-shadow-tree.html` | 12 | 0 | 12 |
| `shadow-dom/event-inside-slotted-node.html` | 20 | 0 | 20 |
| `shadow-dom/event-on-pseudo-element-crash.html` | 1 | 1 | 0 |
| `shadow-dom/event-post-dispatch-no-listeners.html` | 5 | 0 | 5 |
| `shadow-dom/event-post-dispatch.html` | 1 | 0 | 1 |
| `shadow-dom/event-with-related-target.html` | 18 | 0 | 18 |
| `shadow-dom/execcommand-insertList-in-shadow.html` | 1 | 0 | 1 |
| `shadow-dom/form-control-form-attribute.html` | 3 | 2 | 1 |
| `shadow-dom/getElementById-dynamic-001.html` | 1 | 1 | 0 |
| `shadow-dom/getElementById-dynamic-002.html` | 1 | 1 | 0 |
| `shadow-dom/historical.html` | 5 | 5 | 0 |
| `shadow-dom/imperative-slot-api-crash.html` | 1 | 0 | 1 |
| `shadow-dom/imperative-slot-api-cross-shadow-root.html` | 2 | 0 | 2 |
| `shadow-dom/imperative-slot-api-disconnected.html` | 1 | 0 | 1 |
| `shadow-dom/imperative-slot-api-slotchange.html` | 13 | 0 | 13 |
| `shadow-dom/imperative-slot-api.html` | 16 | 1 | 15 |
| `shadow-dom/imperative-slot-assign-not-slotable-crash.html` | 1 | 1 | 0 |
| `shadow-dom/imperative-slot-fallback-clear.html` | 2 | 0 | 2 |
| `shadow-dom/imperative-slot-initial-fallback.html` | 2 | 0 | 2 |
| `shadow-dom/inserting-fragment-under-shadow-host.html` | 1 | 0 | 1 |
| `shadow-dom/invalidate-shadow-dom-crash.html` | 1 | 1 | 0 |
| `shadow-dom/nested-slot-remove-crash.html` | 1 | 1 | 0 |
| `shadow-dom/shadow-host-child-restyle-crash.html` | 1 | 1 | 0 |
| `shadow-dom/shadow-root-clonable.html` | 6 | 0 | 6 |
| `shadow-dom/slot-dir-attach-child-crash.html` | 1 | 1 | 0 |
| `shadow-dom/slot-dir-attach-child-details-crash.html` | 1 | 1 | 0 |
| `shadow-dom/slot-dir-attach-child-meter-crash.html` | 1 | 1 | 0 |
| `shadow-dom/slot-dir-attach-child-progress-crash.html` | 1 | 1 | 0 |
| `shadow-dom/slot-dir-attach-child-textarea-crash.html` | 1 | 1 | 0 |
| `shadow-dom/slot-reconciliation-at-node-removal.html` | 1 | 0 | 1 |
| `shadow-dom/slot-with-slottable-slot-in-display-none-subtree-crash.html` | 1 | 1 | 0 |
| `shadow-dom/slotchange-customelements.html` | 1 | 0 | 1 |
| `shadow-dom/slotchange-event.html` | 32 | 0 | 32 |
| `shadow-dom/slotchange.html` | 17 | 0 | 17 |
| `shadow-dom/slots-fallback-in-document.html` | 1 | 0 | 1 |
| `shadow-dom/slots-fallback.html` | 13 | 0 | 13 |
| `shadow-dom/slots-outside-shadow-dom.html` | 1 | 0 | 1 |
| `shadow-dom/slots.html` | 26 | 0 | 26 |

## 失败聚类（根因 → 影响 subtests 量级）

| 根因聚类 | 影响量级 | 证据示例 |
|---|---|---|
| 1. shadow 树 appendChild/查询断裂——`shadowRoot.appendChild is not a function`（R2926 的 shadow handle 容器未接 appendChild 全语义） | ~112+ | capturing-and-bubbling-event-listeners-across-shadow-trees |
| 2. `CustomElementRegistry` 接口对象缺失（`customElements` 实例的 ctor 未暴露） | ~88 | CustomElementRegistry.html |
| 3. customized builtins（`is=` / `extends`）语义面缺失（ctor 原型不对齐 + `setNamedItem`/NamedNodeMap 反应链） | ~500+ | builtin-coverage 331F、Document-createElement* |
| 4. CEReactions per-API 反应（reactions/ 59 用例——attribute 变更须入 upgrade/reaction queue） | ~59 用例整簇 | reactions/Animation.html 等 |
| 5. slot 全链路为零（`assignedNodes is not a function`、slotchange、`el.slot`/`assignedSlot` IDL、imperative assign） | shadow-dom 647 中的大头 | HTMLSlotElement-interface、slotchange-event、slots.html |
| 6. template DOM 层占位（contents 内联文档树 → template-* 断言失败；`testInIFrame` 变体链 iframe 面） | 296 中的大头 | template-content.html、definitions/* |
| 7. `attachShadow` host 约束校验缺失（非 HTML 命名空间 host 须抛 NotSupportedError） | ~266 的子集 | attach-shadow-non-html-namespace 266F |
| 8. ElementInternals / form-associated（attachInternals 等——Support Envelope 只评估不实施） | ~50 | ElementInternals-validation |
| 9. adoptedCallback 派发路径缺失（document.adoptNode / importNode 跨文档） | ~40 | adopted-callback.html |
| 10. whenDefined/upgrade 遗留 PoC 简化（RangeError 栈溢出簇 + `[[Prototype]]` 断言） | ~30 | upgrading.html、CustomElementRegistry-constructor |

## skip 域（与 fetch-web-components-subset.sh 头注释同一份规则）

渲染级 composed tree（slot-fallback-content-00{1..8}、layout-*、directionality-*、restyle-*、
shadow-style-*、invalidate-sibling-*、manual-slot-assignment-*、imperative-slot-layout-invalidation-*、
user-agent-shadow*、nested-hover-*、accesskey*、touch-*、wheel-*、scroll-*、offset*、focus-within-*）、
几何命中四面（caretPositionFromPoint / elementFromPoint / highlightsFromPoint / offsetX-offsetY）、
drag related-target 两面、focus/focus-navigation 交互面、crashtests/、declarative/、leaktests/、
reference-target/、the-template-element 的 css-user-agent-style-sheet-*（reftest）、
*-ref.html 参照页、.window.js/.xhtml/.svg/.window 形态。
渲染级部分**等用户点名 Shadow DOM 深结构专项**（goal 排除条款），不算未满足 DC。

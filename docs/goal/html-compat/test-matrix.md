# HTML 行为兼容测试矩阵

WPT 是外部规范 Oracle，不能替代 `local_unit` 或 `local_integration`。

| feature_id | spec_link | fr | implementation | local_unit | local_integration | wpt | status |
|---|---|---|---|---|---|---|---|
| form-fixture-scenario | local fixture contract | FR-001 | browser test helper | `scenario_success_runs_typed_steps` / `scenario_failure_reports_exact_step_and_state` | `form_fixture_complete_multiprocess_semantics` | - | pass |
| text-input-basic | https://html.spec.whatwg.org/multipage/input.html#text-(type=text)-state-and-search-state-(type=search) | FR-001 | existing renderer input path | `form_interaction_fixture_complete_sequence` | `form_fixture_complete_multiprocess_semantics` | - | pass |
| textarea-ime-basic | https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element | FR-001 | existing retained form state | `ime_commit_after_preedit_updates_live_webview` | `form_fixture_complete_multiprocess_semantics` | - | pass |
| sequential-focus-basic | https://html.spec.whatwg.org/multipage/interaction.html#sequential-focus-navigation | FR-001 | existing focus route | `form_interaction_fixture_complete_sequence` | `form_fixture_complete_multiprocess_semantics` | - | pass |
| checkbox-basic | https://html.spec.whatwg.org/multipage/input.html#checkbox-state-(type=checkbox) | FR-001 | existing checkbox default action | `form_interaction_fixture_complete_sequence` | `form_fixture_complete_multiprocess_semantics` | - | pass |
| radio-basic | https://html.spec.whatwg.org/multipage/input.html#radio-button-state-(type=radio) | FR-001 | existing radio default action | `form_interaction_fixture_complete_sequence` | `form_fixture_complete_multiprocess_semantics` | - | pass |
| form-reset-basic | https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#resetting-a-form | FR-001 | existing reset action | `form_interaction_fixture_complete_sequence` / `test_is_submit_button` | `form_fixture_complete_multiprocess_semantics` | - | pass |
| form-submit-cancel | https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-submission-2 | FR-001 | existing submit action | `form_interaction_fixture_complete_sequence` / `test_is_submit_button` | `form_fixture_complete_multiprocess_semantics` | - | pass |
| document-title-script-update | https://html.spec.whatwg.org/multipage/dom.html#document.title | FR-001 | renderer frame publish | `form_interaction_fixture_complete_sequence` | `form_fixture_complete_multiprocess_semantics` | - | pass |
| page-node-identity | https://dom.spec.whatwg.org/#concept-node | FR-004/FR-005 | page-runtime + hit-test + paint IPC | `page_node_ref_rejects_navigation_and_document_replacement` / `same_node_handle_is_distinct_across_document_scopes` | `page_target_is_scoped_to_snapshot_document_generation` | - | pass |
| pointer-paired-target | https://w3c.github.io/uievents/#events-mouseevent-event-order | FR-004 | browser interaction routing | `page_target_is_scoped_to_snapshot_document_generation` | `pressed_target_pairs_release_and_cancels_when_document_changes` | - | pass |

状态值：`planned`、`partial`、`pass`、`blocked`。

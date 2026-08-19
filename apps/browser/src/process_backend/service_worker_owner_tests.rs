use std::path::PathBuf;

use super::*;

#[test]
fn service_worker_authority_exists_only_for_committed_navigation() {
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(801);
    let renderer_id = 91;
    let url = "https://example.test/page";
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    backend.stage_indexed_db_navigation(renderer_id, url, 1);

    assert!(!backend.committed_document_urls.contains_key(&renderer_id));
    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: url.into(),
            navigation_epoch: 1,
        },
    );
    assert_eq!(
        backend.committed_document_urls.get(&renderer_id).map(String::as_str),
        Some(url)
    );

    backend.stage_indexed_db_navigation(renderer_id, "https://next.test/", 2);
    assert!(!backend.committed_document_urls.contains_key(&renderer_id));
}

#[test]
fn mismatched_navigation_commit_does_not_grant_service_worker_authority() {
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(802);
    let renderer_id = 92;
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    backend.stage_indexed_db_navigation(renderer_id, "https://expected.test/", 3);

    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: "https://attacker.test/".into(),
            navigation_epoch: 3,
        },
    );

    assert!(!backend.committed_document_urls.contains_key(&renderer_id));
}

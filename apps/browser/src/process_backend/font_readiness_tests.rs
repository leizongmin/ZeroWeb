use super::*;

#[test]
fn capture_readiness_requires_a_matching_new_navigation_commit() {
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab = TabId(840);
    let renderer = 94;
    backend.tab_to_renderer.insert(tab, renderer);
    let commit = |url: &str, epoch| NavigationCommittedParams {
        url: url.into(),
        navigation_epoch: epoch,
    };
    assert!(!backend.handle_navigation_committed(tab, renderer, commit("https://page.test/", 2)));
    backend.stage_indexed_db_navigation(renderer, "https://page.test/", 2);
    assert!(!backend.handle_navigation_committed(tab, renderer, commit("https://page.test/", 1)));
    assert!(!backend.handle_navigation_committed(tab, renderer, commit("https://other.test/", 2)));
    assert!(backend.handle_navigation_committed(tab, renderer, commit("https://page.test/", 2)));
    assert!(
        !backend.handle_navigation_committed(tab, renderer, commit("https://other.test/", 2)),
        "a previous committed epoch is not sufficient to accept another completion"
    );
}

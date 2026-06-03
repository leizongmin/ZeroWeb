//! Site isolation tests for uncovered paths.

use zero_security::origin::Origin;
use zero_security::site_isolation::{IsolationPolicy, SiteIsolationManager};

#[test]
fn test_site_isolation_none_policy() {
    let mgr = SiteIsolationManager::new(IsolationPolicy::None);
    let parent = Origin::parse("https://example.com").unwrap();
    let iframe = Origin::parse("https://evil.com").unwrap();
    assert!(!mgr.needs_separate_process(&parent, &iframe));
    assert!(mgr.are_in_same_process(&parent, &iframe));
}

#[test]
fn test_site_isolation_site_isolated_policy() {
    let mut mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);
    let origin1 = Origin::parse("https://example.com").unwrap();
    let origin2 = Origin::parse("https://evil.com").unwrap();
    let parent = Origin::parse("https://parent.com").unwrap();

    assert!(mgr.needs_separate_process(&parent, &origin1));
    let pid1 = mgr.get_or_create_process(&origin1);
    let pid2 = mgr.get_or_create_process(&origin2);
    assert_ne!(pid1, pid2);
    assert!(!mgr.are_in_same_process(&origin1, &origin2));
}

#[test]
fn test_site_isolation_strict_origin() {
    let mgr = SiteIsolationManager::new(IsolationPolicy::StrictOriginIsolated);
    let parent = Origin::parse("https://example.com").unwrap();
    let same_site = Origin::parse("https://sub.example.com").unwrap();
    // Even same-site needs separate process in strict mode
    assert!(mgr.needs_separate_process(&parent, &same_site));
}

#[test]
fn test_site_isolation_can_access_parent_dom() {
    let mgr = SiteIsolationManager::new(IsolationPolicy::None);
    let parent = Origin::parse("https://example.com").unwrap();
    let same = Origin::parse("https://example.com").unwrap();
    assert!(mgr.can_access_parent_dom(&parent, &same));
}

#[test]
fn test_site_isolation_get_process_unregistered() {
    let mgr = SiteIsolationManager::new(IsolationPolicy::None);
    let origin = Origin::parse("https://example.com").unwrap();
    assert!(mgr.get_process_for_origin(&origin).is_none());
}

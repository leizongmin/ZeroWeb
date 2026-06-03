//! Browser shell tests for uncovered paths.

use zero_browser_shell::{BrowserShell, FindState};

#[test]
fn test_browser_shell_empty_initially() {
    // Test lines 78-80 - Browser is initially not empty
    let browser = BrowserShell::new();

    // Browser should not be empty (creates initial tab)
    assert!(!browser.is_empty());
    assert_eq!(browser.tab_count(), 1);
}

use super::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub(super) struct FixtureState {
    pub reason: String,
    pub name: String,
    pub note: String,
    pub subscribe: bool,
    pub plan: String,
    pub focused: String,
    pub result: String,
}

#[derive(Clone, Debug, Default)]
pub(super) struct StateExpectation {
    pub reason: Option<String>,
    pub name: Option<String>,
    pub note: Option<String>,
    pub subscribe: Option<bool>,
    pub plan: Option<String>,
    pub focused: Option<String>,
}

impl StateExpectation {
    pub fn reason(mut self, value: &str) -> Self {
        self.reason = Some(value.to_string());
        self
    }

    pub fn name(mut self, value: &str) -> Self {
        self.name = Some(value.to_string());
        self
    }

    pub fn note(mut self, value: &str) -> Self {
        self.note = Some(value.to_string());
        self
    }

    pub fn subscribe(mut self, value: bool) -> Self {
        self.subscribe = Some(value);
        self
    }

    pub fn plan(mut self, value: &str) -> Self {
        self.plan = Some(value.to_string());
        self
    }

    pub fn focused(mut self, value: &str) -> Self {
        self.focused = Some(value.to_string());
        self
    }

    fn check(&self, actual: &FixtureState) -> Result<(), String> {
        check_field("reason", self.reason.as_ref(), &actual.reason)?;
        check_field("name", self.name.as_ref(), &actual.name)?;
        check_field("note", self.note.as_ref(), &actual.note)?;
        if let Some(expected) = self.subscribe
            && expected != actual.subscribe
        {
            return Err(format!(
                "state.subscribe: expected {expected:?}, got {:?}",
                actual.subscribe
            ));
        }
        check_field("plan", self.plan.as_ref(), &actual.plan)?;
        check_field("focused", self.focused.as_ref(), &actual.focused)
    }
}

fn check_field(name: &str, expected: Option<&String>, actual: &str) -> Result<(), String> {
    if let Some(expected) = expected
        && expected != actual
    {
        return Err(format!("state.{name}: expected {expected:?}, got {actual:?}"));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub(super) enum HtmlStep {
    Click(String),
    TypeText(String),
    PressKey(String),
    ImePreedit(String),
    ImeCommit(String),
    AssertState(StateExpectation),
    AssertOutput(String),
    AssertChecked { selector: String, expected: bool },
    AssertFocused(String),
    AssertUrl(String),
}

impl HtmlStep {
    fn description(&self) -> String {
        match self {
            Self::Click(selector) => format!("click({selector})"),
            Self::TypeText(text) => format!("type_text({text:?})"),
            Self::PressKey(key) => format!("press_key({key})"),
            Self::ImePreedit(text) => format!("ime_preedit({text:?})"),
            Self::ImeCommit(text) => format!("ime_commit({text:?})"),
            Self::AssertState(_) => "assert_state".to_string(),
            Self::AssertOutput(expected) => format!("assert_output({expected:?})"),
            Self::AssertChecked { selector, expected } => {
                format!("assert_checked({selector}, {expected})")
            }
            Self::AssertFocused(selector) => format!("assert_focused({selector})"),
            Self::AssertUrl(expected) => format!("assert_url({expected:?})"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ScenarioDiagnostics {
    pub phase: String,
    pub url: String,
    pub navigation_epoch: u64,
    pub snapshot_sequence: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ScenarioError {
    pub step: usize,
    pub description: String,
    pub detail: String,
    pub diagnostics: ScenarioDiagnostics,
}

impl std::fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "step {} {} failed: {}; phase={} url={:?} navigation_epoch={} snapshot_sequence={}",
            self.step,
            self.description,
            self.detail,
            self.diagnostics.phase,
            self.diagnostics.url,
            self.diagnostics.navigation_epoch,
            self.diagnostics.snapshot_sequence
        )
    }
}

impl std::error::Error for ScenarioError {}

pub(super) trait HtmlScenarioHost {
    fn click(&mut self, selector: &str) -> Result<(), String>;
    fn type_text(&mut self, text: &str) -> Result<(), String>;
    fn press_key(&mut self, key: &str) -> Result<(), String>;
    fn ime_preedit(&mut self, text: &str) -> Result<(), String>;
    fn ime_commit(&mut self, text: &str) -> Result<(), String>;
    fn fixture_state(&self) -> Result<FixtureState, String>;
    fn output_text(&self) -> Result<String, String>;
    fn checked(&self, selector: &str) -> Result<bool, String>;
    fn focused(&self) -> Result<String, String>;
    fn url(&self) -> Result<String, String>;
    fn poll(&mut self);
    fn diagnostics(&self) -> ScenarioDiagnostics;
}

pub(super) struct HtmlScenario<'a, H> {
    host: &'a mut H,
    steps: Vec<HtmlStep>,
    assertion_timeout: Duration,
}

impl<'a, H: HtmlScenarioHost> HtmlScenario<'a, H> {
    pub fn new(host: &'a mut H) -> Self {
        Self {
            host,
            steps: Vec::new(),
            assertion_timeout: Duration::from_secs(10),
        }
    }

    #[cfg(test)]
    fn with_assertion_timeout(mut self, timeout: Duration) -> Self {
        self.assertion_timeout = timeout;
        self
    }

    pub fn click(mut self, selector: &str) -> Self {
        self.steps.push(HtmlStep::Click(selector.to_string()));
        self
    }

    pub fn type_text(mut self, text: &str) -> Self {
        self.steps.push(HtmlStep::TypeText(text.to_string()));
        self
    }

    pub fn press_key(mut self, key: &str) -> Self {
        self.steps.push(HtmlStep::PressKey(key.to_string()));
        self
    }

    pub fn ime_preedit(mut self, text: &str) -> Self {
        self.steps.push(HtmlStep::ImePreedit(text.to_string()));
        self
    }

    pub fn ime_commit(mut self, text: &str) -> Self {
        self.steps.push(HtmlStep::ImeCommit(text.to_string()));
        self
    }

    pub fn assert_state(mut self, expected: StateExpectation) -> Self {
        self.steps.push(HtmlStep::AssertState(expected));
        self
    }

    pub fn assert_output(mut self, expected: &str) -> Self {
        self.steps.push(HtmlStep::AssertOutput(expected.to_string()));
        self
    }

    pub fn assert_checked(mut self, selector: &str, expected: bool) -> Self {
        self.steps.push(HtmlStep::AssertChecked {
            selector: selector.to_string(),
            expected,
        });
        self
    }

    pub fn assert_focused(mut self, selector: &str) -> Self {
        self.steps.push(HtmlStep::AssertFocused(selector.to_string()));
        self
    }

    pub fn assert_url(mut self, expected: &str) -> Self {
        self.steps.push(HtmlStep::AssertUrl(expected.to_string()));
        self
    }

    pub fn run(mut self) -> Result<(), ScenarioError> {
        let steps = std::mem::take(&mut self.steps);
        for (index, step) in steps.iter().enumerate() {
            if let Err(detail) = self.run_step(step) {
                return Err(ScenarioError {
                    step: index + 1,
                    description: step.description(),
                    detail,
                    diagnostics: self.host.diagnostics(),
                });
            }
        }
        Ok(())
    }

    fn run_step(&mut self, step: &HtmlStep) -> Result<(), String> {
        match step {
            HtmlStep::Click(selector) => self.host.click(selector),
            HtmlStep::TypeText(text) => self.host.type_text(text),
            HtmlStep::PressKey(key) => self.host.press_key(key),
            HtmlStep::ImePreedit(text) => self.host.ime_preedit(text),
            HtmlStep::ImeCommit(text) => self.host.ime_commit(text),
            HtmlStep::AssertState(expected) => self.wait_for(|host| expected.check(&host.fixture_state()?)),
            HtmlStep::AssertOutput(expected) => self.wait_for(|host| {
                let actual = host.output_text()?;
                if actual == expected.as_str() {
                    Ok(())
                } else {
                    Err(format!("output: expected {expected:?}, got {actual:?}"))
                }
            }),
            HtmlStep::AssertChecked { selector, expected } => self.wait_for(|host| {
                let actual = host.checked(selector)?;
                if actual == *expected {
                    Ok(())
                } else {
                    Err(format!("{selector}.checked: expected {expected}, got {actual}"))
                }
            }),
            HtmlStep::AssertFocused(expected) => self.wait_for(|host| {
                let actual = host.focused()?;
                if actual == expected.as_str() {
                    Ok(())
                } else {
                    Err(format!("focus: expected {expected:?}, got {actual:?}"))
                }
            }),
            HtmlStep::AssertUrl(expected) => self.wait_for(|host| {
                let actual = host.url()?;
                if actual == expected.as_str() {
                    Ok(())
                } else {
                    Err(format!("url: expected {expected:?}, got {actual:?}"))
                }
            }),
        }
    }

    fn wait_for<F>(&mut self, mut check: F) -> Result<(), String>
    where
        F: FnMut(&mut H) -> Result<(), String>,
    {
        let deadline = Instant::now() + self.assertion_timeout;
        loop {
            match check(self.host) {
                Ok(()) => return Ok(()),
                Err(error) if Instant::now() < deadline => {
                    self.host.poll();
                    std::thread::sleep(Duration::from_millis(10));
                    let _ = error;
                }
                Err(error) => return Err(error),
            }
        }
    }
}

pub(super) struct BrowserScenarioHost<'a> {
    app: &'a mut BrowserApp,
    tab_id: TabId,
    gpu_present: bool,
    hit_centers: HashMap<String, (f32, f32)>,
}

impl<'a> BrowserScenarioHost<'a> {
    pub fn new(app: &'a mut BrowserApp, tab_id: TabId, gpu_present: bool) -> Self {
        Self {
            app,
            tab_id,
            gpu_present,
            hit_centers: HashMap::new(),
        }
    }

    fn page_html(&self) -> Result<String, String> {
        self.app
            .page_html_for_test(self.tab_id)
            .ok_or_else(|| "page HTML snapshot unavailable".to_string())
    }

    fn point_for_id(&mut self, expected_id: &str) -> Result<(f32, f32), String> {
        // 首次点击前命中缓存可能仍是 about:blank 首帧（wait_for_snapshot_after 在
        // 首帧即过）——20s 内重试重扫，直到目标元素帧到达（macos-aarch64 慢 runner
        // 首帧渲染可 >10s，曾固定 step1 失败；R2414 同族时序）。
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if self.hit_centers.is_empty() {
                self.scan_hit_centers();
            }
            if let Some(point) = self.hit_centers.get(expected_id).copied() {
                return Ok(point);
            }
            if Instant::now() >= deadline {
                return Err(format!("no hit-test point for #{expected_id}"));
            }
            self.hit_centers.clear();
            self.poll();
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn scan_hit_centers(&mut self) {
        let (logical_width, logical_height) = self.app.content_logical_size();
        let mut bounds: HashMap<String, (f32, f32, f32, f32)> = HashMap::new();
        for y in (0..logical_height).step_by(4) {
            for x in (0..logical_width).step_by(4) {
                if let Some(hit) = self.app.hit_test_page_element_for_test(self.tab_id, x as f32, y as f32)
                    && let Some(id) = hit.id
                {
                    bounds
                        .entry(id)
                        .and_modify(|(min_x, min_y, max_x, max_y)| {
                            *min_x = min_x.min(x as f32);
                            *min_y = min_y.min(y as f32);
                            *max_x = max_x.max(x as f32);
                            *max_y = max_y.max(y as f32);
                        })
                        .or_insert((x as f32, y as f32, x as f32, y as f32));
                }
            }
        }
        self.hit_centers = bounds
            .into_iter()
            .map(|(id, (min_x, min_y, max_x, max_y))| (id, ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)))
            .collect();
    }
}

impl HtmlScenarioHost for BrowserScenarioHost<'_> {
    fn click(&mut self, selector: &str) -> Result<(), String> {
        let id = selector
            .strip_prefix('#')
            .ok_or_else(|| format!("M0 click requires an id selector, got {selector:?}"))?;
        self.hit_centers.clear();
        let (document_x, document_y) = self.point_for_id(id)?;
        let (content_x, content_y, _, _) = self.app.page_content_rect();
        let physical_x = (content_x + document_x * self.app.scale_factor) as f64;
        let physical_y = (content_y + document_y * self.app.scale_factor) as f64;
        self.app.handle_mouse_move(physical_x, physical_y);
        self.app.handle_mouse_click(physical_x, physical_y, true, "Left");
        self.app.handle_mouse_click(physical_x, physical_y, false, "Left");
        let actual = self.app.page_event_target_for_test(self.tab_id).unwrap_or("<none>");
        if actual == selector {
            Ok(())
        } else {
            Err(format!(
                "click {selector} at document ({document_x:.1}, {document_y:.1}) targeted {actual}"
            ))
        }
    }

    fn type_text(&mut self, text: &str) -> Result<(), String> {
        for ch in text.chars() {
            self.app.handle_key(&ch.to_string(), true, None);
        }
        Ok(())
    }

    fn press_key(&mut self, key: &str) -> Result<(), String> {
        self.app.handle_key(key, true, None);
        Ok(())
    }

    fn ime_preedit(&mut self, text: &str) -> Result<(), String> {
        self.app.handle_ime(zero_host_runtime::event::ImeEvent::Preedit {
            text: text.to_string(),
            cursor: Some((text.len(), text.len())),
        });
        Ok(())
    }

    fn ime_commit(&mut self, text: &str) -> Result<(), String> {
        self.app
            .handle_ime(zero_host_runtime::event::ImeEvent::Commit(text.to_string()));
        Ok(())
    }

    fn fixture_state(&self) -> Result<FixtureState, String> {
        let state = if let Ok(html) = self.page_html() {
            zero_engine::query_text_from_html(&html, "#test-state")
        } else {
            self.app
                .page_title_for_test(self.tab_id)
                .and_then(|title| title.strip_prefix("ZERO_TEST_STATE:").map(str::to_string))
                .ok_or_else(|| "page state unavailable from HTML snapshot or title".to_string())?
        };
        serde_json::from_str(&state).map_err(|error| format!("invalid #test-state {state:?}: {error}"))
    }

    fn output_text(&self) -> Result<String, String> {
        if let Ok(html) = self.page_html() {
            return Ok(zero_engine::query_text_from_html(&html, "#result"));
        }
        Ok(self.fixture_state()?.result)
    }

    fn checked(&self, selector: &str) -> Result<bool, String> {
        if let Ok(html) = self.page_html() {
            return Ok(zero_engine::has_attribute(&html, selector, "checked"));
        }
        let state = self.fixture_state()?;
        match selector {
            "#subscribe" => Ok(state.subscribe),
            "#plan-basic" => Ok(state.plan == "basic"),
            "#plan-pro" => Ok(state.plan == "pro"),
            _ => Err(format!("no checked query mapping for {selector}")),
        }
    }

    fn focused(&self) -> Result<String, String> {
        self.app
            .page_event_target_for_test(self.tab_id)
            .map(str::to_string)
            .ok_or_else(|| "page focus target unavailable".to_string())
    }

    fn url(&self) -> Result<String, String> {
        self.app
            .page_url_for_test(self.tab_id)
            .ok_or_else(|| "page URL unavailable".to_string())
    }

    fn poll(&mut self) {
        if self.gpu_present {
            self.app.poll_tab_fetch_with_gpu_present_for_test();
        } else {
            self.app.poll_tab_fetch();
        }
    }

    fn diagnostics(&self) -> ScenarioDiagnostics {
        ScenarioDiagnostics {
            phase: if self.app.page_html_for_test(self.tab_id).is_some() {
                "hit-test".to_string()
            } else {
                "awaiting-snapshot".to_string()
            },
            url: self
                .app
                .page_url_for_test(self.tab_id)
                .unwrap_or_else(|| "<unavailable>".to_string()),
            navigation_epoch: self.app.navigation_epoch_for_test(self.tab_id),
            snapshot_sequence: self.app.snapshot_seq_for_test(self.tab_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeHost {
        state: FixtureState,
        output: String,
        focused: String,
        click_error: Option<String>,
        diagnostics: ScenarioDiagnostics,
    }

    impl HtmlScenarioHost for FakeHost {
        fn click(&mut self, selector: &str) -> Result<(), String> {
            if let Some(error) = &self.click_error {
                return Err(format!("{error}: {selector}"));
            }
            self.focused = selector.to_string();
            Ok(())
        }

        fn type_text(&mut self, text: &str) -> Result<(), String> {
            self.state.name.push_str(text);
            Ok(())
        }

        fn press_key(&mut self, _key: &str) -> Result<(), String> {
            Ok(())
        }

        fn ime_preedit(&mut self, _text: &str) -> Result<(), String> {
            Ok(())
        }

        fn ime_commit(&mut self, text: &str) -> Result<(), String> {
            self.state.note.push_str(text);
            Ok(())
        }

        fn fixture_state(&self) -> Result<FixtureState, String> {
            Ok(self.state.clone())
        }

        fn output_text(&self) -> Result<String, String> {
            Ok(self.output.clone())
        }

        fn checked(&self, _selector: &str) -> Result<bool, String> {
            Ok(self.state.subscribe)
        }

        fn focused(&self) -> Result<String, String> {
            Ok(self.focused.clone())
        }

        fn url(&self) -> Result<String, String> {
            Ok(self.diagnostics.url.clone())
        }

        fn poll(&mut self) {}

        fn diagnostics(&self) -> ScenarioDiagnostics {
            self.diagnostics.clone()
        }
    }

    #[test]
    fn scenario_success_runs_typed_steps() {
        let mut host = FakeHost::default();
        HtmlScenario::new(&mut host)
            .click("#name")
            .type_text("abc")
            .assert_state(StateExpectation::default().name("abc"))
            .assert_focused("#name")
            .run()
            .expect("scenario");
    }

    #[test]
    fn scenario_failure_reports_exact_step_and_state() {
        let mut host = FakeHost {
            diagnostics: ScenarioDiagnostics {
                phase: "assertion".to_string(),
                url: "about:blank".to_string(),
                navigation_epoch: 7,
                snapshot_sequence: 11,
            },
            ..Default::default()
        };
        let error = HtmlScenario::new(&mut host)
            .with_assertion_timeout(Duration::ZERO)
            .click("#name")
            .type_text("abc")
            .assert_state(StateExpectation::default().name("wrong"))
            .run()
            .expect_err("must fail");

        assert_eq!(error.step, 3);
        assert_eq!(error.description, "assert_state");
        assert!(error.detail.contains("expected \"wrong\", got \"abc\""));
        assert_eq!(error.diagnostics.navigation_epoch, 7);
        assert_eq!(error.diagnostics.snapshot_sequence, 11);
        assert!(error.to_string().contains("step 3"));
    }

    #[test]
    fn form_fixture_reports_missing_control_stage() {
        let mut host = FakeHost {
            click_error: Some("no hit-test point".to_string()),
            diagnostics: ScenarioDiagnostics {
                phase: "hit-test".to_string(),
                url: "https://zero.test/forms".to_string(),
                navigation_epoch: 12,
                snapshot_sequence: 34,
            },
            ..Default::default()
        };

        let error = HtmlScenario::new(&mut host)
            .click("#missing")
            .run()
            .expect_err("missing control must fail");
        assert_eq!(error.step, 1);
        assert_eq!(error.description, "click(#missing)");
        assert!(error.detail.contains("#missing"));
        assert_eq!(error.diagnostics.phase, "hit-test");
        assert_eq!(error.diagnostics.url, "https://zero.test/forms");
        assert_eq!(error.diagnostics.navigation_epoch, 12);
        assert_eq!(error.diagnostics.snapshot_sequence, 34);
    }
}

//! # zero-ui-forms
//!
//! 表单（spec §8.4.1 `zero-ui-forms` / FR-016 / §8.4.1B 设置页·偏好页 / §8.8 validation/dirty/
//! touched/submit lifecycle 测）。
//!
//! 提供 [`FieldState`]（单字段 value/dirty/touched/error）+ [`Validator`] 体系（必填/长度/组合）+
//! [`FormState`]（多字段聚合，dirty/touched/is_valid 跟踪 + submit 生命周期）。

use compact_str::CompactString;

// ── 校验 ───────────────────────────────────────────────────────────────────────

/// 校验错误（携带 message id，spec FR-013：可见文案走 message id）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(pub String);

impl ValidationError {
    pub fn new(message_id: &str) -> ValidationError {
        ValidationError(message_id.to_string())
    }
}

/// 校验器 trait。
pub trait Validator {
    fn validate(&self, value: &str) -> Result<(), ValidationError>;
}

/// 必填校验。
pub struct Required {
    pub error_msg: &'static str,
}

impl Default for Required {
    fn default() -> Required {
        Required {
            error_msg: "form.error.required",
        }
    }
}

impl Validator for Required {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            Err(ValidationError(self.error_msg.to_string()))
        } else {
            Ok(())
        }
    }
}

/// 最小长度校验（trim 后字符数）。
pub struct MinLength {
    pub min: usize,
    pub error_msg: &'static str,
}

impl MinLength {
    pub fn new(min: usize) -> MinLength {
        MinLength {
            min,
            error_msg: "form.error.too_short",
        }
    }
}

impl Validator for MinLength {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        if value.trim().chars().count() < self.min {
            Err(ValidationError(self.error_msg.to_string()))
        } else {
            Ok(())
        }
    }
}

/// 最大长度校验（trim 后字符数）。
pub struct MaxLength {
    pub max: usize,
    pub error_msg: &'static str,
}

impl MaxLength {
    pub fn new(max: usize) -> MaxLength {
        MaxLength {
            max,
            error_msg: "form.error.too_long",
        }
    }
}

impl Validator for MaxLength {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        if value.trim().chars().count() > self.max {
            Err(ValidationError(self.error_msg.to_string()))
        } else {
            Ok(())
        }
    }
}

/// 组合校验：按序运行所有校验器，返回首个失败（全过则 Ok）。
pub struct All {
    pub rules: Vec<Box<dyn Validator>>,
}

impl All {
    pub fn new(rules: Vec<Box<dyn Validator>>) -> All {
        All { rules }
    }
}

impl Validator for All {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        for r in &self.rules {
            r.validate(value)?;
        }
        Ok(())
    }
}

// ── 字段状态 ───────────────────────────────────────────────────────────────────

/// 字段状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldState {
    pub value: String,
    pub dirty: bool,
    pub touched: bool,
    pub error: Option<ValidationError>,
}

impl FieldState {
    pub fn new(value: &str) -> FieldState {
        FieldState {
            value: value.to_string(),
            dirty: false,
            touched: false,
            error: None,
        }
    }

    /// 设置值；与当前值不同则置 dirty。
    pub fn set(&mut self, value: &str) {
        if self.value != value {
            self.value = value.to_string();
            self.dirty = true;
        }
    }

    /// 标记为已交互（失焦/提交时）。
    pub fn touch(&mut self) {
        self.touched = true;
    }

    /// 用校验器校验；置 touched，更新 error，返回是否通过。
    pub fn validate(&mut self, validator: &dyn Validator) -> bool {
        self.touched = true;
        match validator.validate(&self.value) {
            Ok(()) => {
                self.error = None;
                true
            }
            Err(e) => {
                self.error = Some(e);
                false
            }
        }
    }
}

// ── 表单状态 ───────────────────────────────────────────────────────────────────

/// 提交结果（submit 生命周期，§8.8）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitResult {
    /// 校验全过：含各字段名→值（按注册顺序）。
    Ok(Vec<(CompactString, String)>),
    /// 有字段失败：含字段名→错误（仅失败字段，按注册顺序）。
    Err(Vec<(CompactString, ValidationError)>),
}

/// 表单状态：聚合多个命名字段 + 各自校验器，跟踪 dirty/touched/valid，驱动 submit 生命周期。
#[derive(Default)]
pub struct FormState {
    /// (name, state, validator) 三元组，按注册顺序。
    entries: Vec<(CompactString, FieldState, Box<dyn Validator>)>,
}

impl FormState {
    pub fn new() -> FormState {
        FormState::default()
    }

    /// 注册一个字段（builder，链式）。
    pub fn field<V: Validator + 'static>(mut self, name: &str, initial: &str, validator: V) -> FormState {
        self.entries
            .push((CompactString::new(name), FieldState::new(initial), Box::new(validator)));
        self
    }

    /// 字段数。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 设置某字段值（不存在的字段忽略）。
    pub fn set(&mut self, name: &str, value: &str) {
        if let Some((_, state, _)) = self.entry_mut(name) {
            state.set(value);
        }
    }

    /// 标记某字段 touched（失焦）。
    pub fn touch(&mut self, name: &str) {
        if let Some((_, state, _)) = self.entry_mut(name) {
            state.touch();
        }
    }

    /// 校验单个字段（置 touched + 更新 error）；返回是否通过。字段不存在 → true。
    pub fn validate(&mut self, name: &str) -> bool {
        match self.entry_mut(name) {
            Some((_, state, validator)) => state.validate(validator.as_ref()),
            None => true,
        }
    }

    /// 校验全部字段（各自置 touched + 更新 error）；返回是否全过。
    pub fn validate_all(&mut self) -> bool {
        let mut all_ok = true;
        for (_, state, validator) in &mut self.entries {
            if !state.validate(validator.as_ref()) {
                all_ok = false;
            }
        }
        all_ok
    }

    /// 取某字段当前值。
    pub fn value(&self, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(n, _, _)| n.as_str() == name)
            .map(|(_, s, _)| s.value.as_str())
    }

    /// 取某字段当前错误。
    pub fn error(&self, name: &str) -> Option<&ValidationError> {
        self.entries
            .iter()
            .find(|(n, _, _)| n.as_str() == name)
            .and_then(|(_, s, _)| s.error.as_ref())
    }

    /// 任意字段 dirty → true。
    pub fn is_dirty(&self) -> bool {
        self.entries.iter().any(|(_, s, _)| s.dirty)
    }

    /// 任意字段 touched → true。
    pub fn is_touched(&self) -> bool {
        self.entries.iter().any(|(_, s, _)| s.touched)
    }

    /// 是否全部字段当前无错误（不重新校验，仅看 state.error）。
    pub fn is_valid(&self) -> bool {
        self.entries.iter().all(|(_, s, _)| s.error.is_none())
    }

    /// 提交：先 `validate_all`；全过返回 `Ok(各字段值)`，否则返回 `Err(失败字段错误)`。
    pub fn submit(&mut self) -> SubmitResult {
        let all_ok = self.validate_all();
        if all_ok {
            SubmitResult::Ok(
                self.entries
                    .iter()
                    .map(|(n, s, _)| (n.clone(), s.value.clone()))
                    .collect(),
            )
        } else {
            SubmitResult::Err(
                self.entries
                    .iter()
                    .filter_map(|(n, s, _)| s.error.clone().map(|e| (n.clone(), e)))
                    .collect(),
            )
        }
    }

    /// 复位所有字段（清 dirty/touched/error，值保留）。
    pub fn reset(&mut self) {
        for (_, state, _) in &mut self.entries {
            state.dirty = false;
            state.touched = false;
            state.error = None;
        }
    }

    fn entry_mut(&mut self, name: &str) -> Option<&mut (CompactString, FieldState, Box<dyn Validator>)> {
        self.entries.iter_mut().find(|(n, _, _)| n.as_str() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_validator_and_dirty() {
        let mut f = FieldState::new("");
        assert!(!f.validate(&Required::default())); // 空 → 错误
        assert!(f.error.is_some());
        assert!(f.touched, "validate marks touched");
        f.set("Zero");
        assert!(f.dirty);
        assert!(f.validate(&Required::default())); // 非空 → 通过
        assert!(f.error.is_none());
    }

    #[test]
    fn length_validators() {
        let min = MinLength::new(3);
        assert!(min.validate("abc").is_ok());
        assert!(min.validate("ab").is_err());
        let max = MaxLength::new(5);
        assert!(max.validate("hello").is_ok());
        assert!(max.validate("toolong").is_err());
    }

    #[test]
    fn all_combinator_returns_first_failure() {
        let all = All::new(vec![Box::new(Required::default()), Box::new(MinLength::new(3))]);
        // 空串 → 必填失败（首个）。
        assert!(all.validate("").is_err());
        // 短串 → 必填过、长度失败。
        assert!(all.validate("ab").is_err());
        // 合法 → 全过。
        assert!(all.validate("abcd").is_ok());
    }

    #[test]
    fn form_submit_lifecycle_valid() {
        // §8.8 submit lifecycle：填值 → submit → Ok(values)；dirty/touched 跟踪。
        let mut form =
            FormState::new()
                .field("name", "", Required::default())
                .field("email", "a@b.c", Required::default());
        assert!(!form.is_dirty(), "initially clean");
        form.set("name", "Ada");
        assert!(form.is_dirty(), "dirty after set");
        // 提交（全过）。
        match form.submit() {
            SubmitResult::Ok(values) => {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0].0.as_str(), "name");
                assert_eq!(values[0].1, "Ada");
                assert_eq!(values[1].1, "a@b.c");
            }
            SubmitResult::Err(_) => panic!("valid form should submit Ok"),
        }
        assert!(form.is_touched(), "submit touched all fields");
        assert!(form.is_valid());
    }

    #[test]
    fn form_submit_lifecycle_invalid_returns_errors() {
        let mut form = FormState::new()
            .field("name", "", Required::default())
            .field("email", "", Required::default());
        match form.submit() {
            SubmitResult::Err(errs) => {
                assert_eq!(errs.len(), 2, "both required-empty fields fail");
                assert_eq!(errs[0].0.as_str(), "name");
            }
            SubmitResult::Ok(_) => panic!("invalid form should submit Err"),
        }
        assert!(!form.is_valid());
    }

    #[test]
    fn form_single_field_validate_and_error_lookup() {
        let mut form = FormState::new().field("name", "a", MinLength::new(3));
        assert!(!form.validate("name"), "too short → fail");
        assert!(form.error("name").is_some());
        form.set("name", "abc");
        assert!(form.validate("name"), "now valid");
        assert!(form.error("name").is_none());
        // 不存在的字段。
        assert!(form.validate("missing"));
        assert!(form.error("missing").is_none());
        assert!(form.value("missing").is_none());
    }

    #[test]
    fn form_reset_clears_state_keeps_values() {
        let mut form = FormState::new().field("name", "init", Required::default());
        // 改成空 → dirty，校验失败。
        form.set("name", "");
        assert!(form.is_dirty());
        form.validate_all();
        assert!(!form.is_valid(), "empty name fails required");
        assert!(form.is_touched());
        // 复位：清 dirty/touched/error，值保留。
        form.reset();
        assert!(!form.is_dirty(), "reset clears dirty");
        assert!(!form.is_touched(), "reset clears touched");
        assert!(form.is_valid(), "reset clears errors");
        assert_eq!(form.value("name"), Some(""), "value preserved after reset");
    }
}

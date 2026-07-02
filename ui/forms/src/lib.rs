//! # zero-ui-forms
//!
//! 表单（spec §8.4.1 `zero-ui-forms` / FR-016 / §8.4.1B 设置页）。
//!
//! M1 提供 FieldState（value/dirty/touched/error）+ 校验器。

/// 校验错误（message id，spec FR-013）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(pub String);

/// 校验器trait。
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
    pub fn set(&mut self, value: &str) {
        if self.value != value {
            self.value = value.to_string();
            self.dirty = true;
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_validator_and_dirty() {
        let mut f = FieldState::new("");
        assert!(!f.validate(&Required::default())); // 空 → 错误
        assert!(f.error.is_some());
        f.set("Zero");
        assert!(f.dirty);
        assert!(f.validate(&Required::default())); // 非空 → 通过
        assert!(f.error.is_none());
    }
}

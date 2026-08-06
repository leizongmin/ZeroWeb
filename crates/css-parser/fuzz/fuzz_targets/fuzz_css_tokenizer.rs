//! Fuzz target：CSS tokenizer（手写状态机，fuzz 收益最高的目标）。
//!
//! 喂入任意字节串（UTF-8 可转换部分），全量收集 token，覆盖
//! tokenizer 的转义/注释/字符串/URL/数值等状态分支。

#![no_main]

use libfuzzer_sys::fuzz_target;
use zero_css_parser::tokenizer::Tokenizer;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let _ = Tokenizer::new(input).collect_tokens();
    }
});

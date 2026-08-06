//! Fuzz target：CSS 样式表解析（tokenizer + parser + 选择器/值解析全链）。
//!
//! 覆盖 parse_stylesheet 的规则块/声明/选择器分支；解析失败路径与
//! 成功路径都必须无 panic（fuzz 的断言目标：不崩溃、不死循环、内存有界）。

#![no_main]

use libfuzzer_sys::fuzz_target;
use zero_css_parser::parser::Parser;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let _ = Parser::parse_stylesheet(input);
    }
});

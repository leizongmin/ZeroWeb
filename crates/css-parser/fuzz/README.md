# zero-css-parser fuzz targets（C1）

css-parser 的 tokenizer 与 parser 均为手写实现，解析不可信输入——fuzz 的目标：
**不崩溃、不死循环、内存有界**（配合 test-guard 式包裹运行）。

## 本地运行（需 nightly toolchain + cargo-fuzz）

```bash
rustup toolchain install nightly
cargo +nightly install cargo-fuzz

# tokenizer（核心目标，先跑）
cargo +nightly fuzz run fuzz_css_tokenizer -- -timeout=5 -max_len=4096

# parser 全链
cargo +nightly fuzz run fuzz_css_parser -- -timeout=5 -max_len=8192
```

## 崩溃复现

```bash
cargo +nightly fuzz run fuzz_css_tokenizer artifacts/fuzz_css_tokenizer/crash-*
```

## 说明

- fuzz crate 独立于 workspace（cargo-fuzz 惯例），不参与 `cargo test --workspace`。
- 正则回归（panic 修复）方式：修复后把崩溃样本加入
  `crates/css-parser/tests/` 作为回归用例。

# ZeroWeb PSL (`zero-psl`)

> 公共后缀列表（PSL）解析与注册域名（eTLD+1）提取

## 概述

`ZeroWeb PSL` (`zero-psl`) 提供 WHATWG / Mozilla [Public Suffix List] 语义下的「公共后缀」识别与「注册域名」（eTLD+1）提取。注册域名是 [cookie same-site] 判定与 [site-isolation] 进程边界的核心输入——例如 `a.github.io` 与 `b.github.io` 在朴素「取末两段」算法下会被误判为同站，正确的 PSL 规则（`*.github.io` 为公共后缀）应将二者判为**不同**注册域。

## 主要功能

- **三类规则解析** — 普通规则（`com` / `co.uk`）、通配规则（`*.ck` / `*.github.io`）、例外规则（`!www.ck`），语法与上游 `public_suffix_list.dat` 一致
- **PSL 匹配算法** — 按 [Public Suffix List Algorithm]：命中规则取标签数最多者；同标签数时普通规则优先于通配规则；例外规则在与通配规则同时命中时胜出，把公共后缀「回退一个标签」
- **注册域名提取** — `registrable_domain()` 输出 eTLD+1：IPv4/IPv6 字面量原样返回；单标签主机（`localhost`）原样返回；主机等于公共后缀本身时返回该主机
- **全局共享实例** — `shared()` 提供惰性初始化、线程安全的默认列表
- **可注入完整数据** — `from_rules()` 接受任意 PSL 文本，供项目的许可证合规流程注入经审核的完整上游数据
- **许可证洁净** — 上游 [public_suffix_list.dat] 由 Mozilla 以 MPL-2.0 管理；本 crate 默认内置的 `DEFAULT_RULES` 是 ZeroWeb 项目原创的小型规则集（gTLD/ccTLD 标签 + 文档化平台公共后缀），保持 MIT 兼容

## 使用示例

```rust
use zero_psl::PublicSuffixList;

let psl = PublicSuffixList::shared();

// github.io 是公共后缀（*.github.io）→ 注册域为左侧一标签
assert_eq!(psl.registrable_domain("a.github.io"), "a.github.io");
assert_eq!(psl.registrable_domain("b.github.io"), "b.github.io");

// 普通后缀：注册域 = 公共后缀 + 左侧一个标签
assert_eq!(psl.registrable_domain("news.example.co.uk"), "example.co.uk");

// 例外规则：!www.ck → www.ck 本身是注册域
assert_eq!(psl.registrable_domain("www.ck"), "www.ck");

// IP 字面量 / 单标签主机原样返回
assert_eq!(psl.registrable_domain("127.0.0.1"), "127.0.0.1");
assert_eq!(psl.registrable_domain("localhost"), "localhost");

// 注入完整上游数据（如经许可证合规流程审核的 PSL 文本）
let full = PublicSuffixList::from_rules(include_str!("audited_psl.txt"));
assert_eq!(full.registrable_domain("www.example.co.uk"), "example.co.uk");
```

[Public Suffix List]: https://publicsuffix.org/
[Public Suffix List Algorithm]: https://github.com/publicsuffix/list/wiki
[public_suffix_list.dat]: https://publicsuffix.org/list/public_suffix_list.dat
[cookie same-site]: https://datatracker.ietf.org/doc/html/draft-ietf-httpbis-rfc6265bis
[site-isolation]: https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/site-isolation.md

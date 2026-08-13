//! # zero-psl —— 公共后缀列表（PSL）解析与注册域名提取
//!
//! 提供 WHATWG / Mozilla [Public Suffix List] 语义下的「公共后缀」识别与
//! 「注册域名」（eTLD+1）提取。注册域名是 [cookie same-site] 判定与
//! [site-isolation] 进程边界的核心输入——例如 `a.github.io` 与
//! `b.github.io` 在朴素「取末两段」算法下会被误判为同站，正确的 PSL
//! 规则（`*.github.io` 为公共后缀）应将二者判为**不同**注册域。
//!
//! ## PSL 规则语法
//!
//! PSL 每行一条规则，忽略空行与 `//` 起始的注释行。三类规则：
//!
//! - **普通规则**（`com` / `co.uk`）：精确匹配后缀标签序列。
//! - **通配规则**（`*.ck` / `*.github.io`）：`*` 仅可作首标签，匹配
//!   该位置上的任意单个标签（不跨多标签）。
//! - **例外规则**（`!www.ck`）：仅在与某通配规则同时命中时生效，将公共
//!   后缀「回退一个标签」（即该例外规则去掉 `!` 前缀后的整串本身即为
//!   注册域名）。
//!
//! ## 匹配算法（[Public Suffix List Algorithm]）
//!
//! 1. 主机名按 `.` 拆成标签，标签做 ASCII 小写规范化。
//! 2. 在规则集中查找所有「与主机名后缀匹配」的规则。
//! 3. **优先规则** = 命中规则中标签数最多者；通配规则的 `*` 计为一个真实
//!    标签参与计数。同标签数时普通规则优先于通配规则。
//! 4. 若优先规则为通配规则，且存在与之同基（去掉 `*.` 前缀后再补回通配标签）
//!    的例外规则命中，则例外规则胜出——公共后缀 = 例外规则标签序列减去首标签。
//! 5. 注册域名（eTLD+1）= 公共后缀 + 其左侧一个标签。
//!
//! ## ⚠️ 数据与许可证
//!
//! 上游 [public_suffix_list.dat] 由 Mozilla 以 **MPL-2.0** 持有并管理。ZeroWeb
//! 整体以 **MIT** 授权；**为保持许可证洁净，本 crate 默认不嵌入任何 MPL 管理的
//! 完整数据文件**。本 crate 内置的 [`DEFAULT_RULES`] 是 ZeroWeb 项目**原创**的小型
//! 规则集——仅收录客观 DNS 顶层结构（gTLD/ccTLD 标签）与文档化平台公共后缀
//! （如 `*.github.io`），属事实性表达而非 MPL 保护的创造性编排，故与 MIT 兼容。
//!
//! 如需完整上游数据（覆盖全部 ccTLD 二级域），应由项目的许可证合规流程经
//! [`PublicSuffixList::from_rules`] 注入经审核的 PSL 文本；注入路径不改变本 crate
//! 的 MIT 许可证声明。
//!
//! [Public Suffix List]: https://publicsuffix.org/
//! [Public Suffix List Algorithm]: https://github.com/publicsuffix/list/wiki
//! [public_suffix_list.dat]: https://publicsuffix.org/list/public_suffix_list.dat
//! [cookie same-site]: https://datatracker.ietf.org/doc/html/draft-ietf-httpbis-rfc6265bis
//! [site-isolation]: https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/site-isolation.md

use std::sync::OnceLock;

/// 公共后缀规则。
///
/// 规则标签序列在内部以小写、按主机名「从左到右」的自然顺序存储
/// （即与主机名标签同序），便于直接做后缀比对。
#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    /// 规则的全部标签（`*` 已还原为占位标签 `_wildcard_` 之外的普通形式，
    /// 但首标签是否为通配由 [`kind`] 表达；此处仅存除通配占位外的真实标签）。
    ///
    /// 存储约定：普通/例外规则存全部标签；通配规则存去掉前导 `*.` 之后的标签，
    /// 并由 [`kind`] = Wildcard 标记需在前补回一个通配标签。
    labels: Vec<String>,
    kind: RuleKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuleKind {
    /// 普通规则（精确后缀）。
    Normal,
    /// 通配规则（`*.x.y`）。
    Wildcard,
    /// 例外规则（`!x.y`）。
    Exception,
}

impl Rule {
    /// 规则的「有效标签数」，用于匹配优先级比较。
    ///
    /// 普通与例外规则 = 实际标签数；通配规则 = 实际标签数 + 1（`*` 占位）。
    fn label_count(&self) -> usize {
        match self.kind {
            RuleKind::Normal | RuleKind::Exception => self.labels.len(),
            RuleKind::Wildcard => self.labels.len() + 1,
        }
    }
}

/// 公共后缀列表。
///
/// 由一组规则构成，提供 [`PublicSuffixList::registrable_domain`] 提取注册域名。
/// 默认实例（[`PublicSuffixList::default`]）使用 [`DEFAULT_RULES`]；完整上游
/// 数据请经 [`PublicSuffixList::from_rules`] 注入（见模块文档的许可证说明）。
#[derive(Debug, Clone)]
pub struct PublicSuffixList {
    rules: Vec<Rule>,
}

impl Default for PublicSuffixList {
    fn default() -> Self {
        Self {
            rules: parse_rules(DEFAULT_RULES),
        }
    }
}

impl PublicSuffixList {
    /// 从 PSL 文本解析构造列表。
    ///
    /// 输入格式与上游 [`public_suffix_list.dat`] 一致：每行一条规则，
    /// `//` 起始行与空行忽略。重复构造同一线程内可复用；跨线程共享建议
    /// 用 [`PublicSuffixList::shared`] 获取全局惰性实例。
    ///
    /// [`public_suffix_list.dat`]: https://publicsuffix.org/list/public_suffix_list.dat
    pub fn from_rules(text: &str) -> Self {
        Self {
            rules: parse_rules(text),
        }
    }

    /// 返回全局共享的默认列表（惰性初始化，线程安全）。
    pub fn shared() -> &'static PublicSuffixList {
        static PSL: OnceLock<PublicSuffixList> = OnceLock::new();
        PSL.get_or_init(PublicSuffixList::default)
    }

    /// 提取主机名的注册域名（eTLD+1）。
    ///
    /// - 输入 IPv4/IPv6 字面量 → 原样返回（无 PSL 语义）。
    /// - 单标签主机（如 `localhost`）→ 原样返回。
    /// - 主机等于公共后缀本身（如 `co.uk`）→ 返回该主机（无可注册域）。
    /// - 否则返回公共后缀 + 左侧一个标签。
    ///
    /// 所有比对基于 ASCII 小写；非 ASCII 标签（IDN）按 Unicode 简单小写处理
    /// （完整 IDNA 2008 Punycode 规范化应由调用方在上游完成，见模块文档）。
    pub fn registrable_domain(&self, host: &str) -> String {
        // IP 地址字面量直接返回（无 PSL 语义）。
        if host.parse::<std::net::IpAddr>().is_ok() {
            return host.to_string();
        }

        let host = host.trim_end_matches('.');
        if host.is_empty() {
            return String::new();
        }

        // 标签做 ASCII 小写规范化后用于匹配。
        let labels: Vec<String> = host.split('.').map(|l| l.to_ascii_lowercase()).collect();

        match self.suffix_label_count(&labels) {
            None => {
                // 无规则命中：按 PSL 算法「unknown TLD」回退——公共后缀 = 末标签，
                // 注册域 = 末两标签（若存在），否则原样返回。
                if labels.len() <= 1 {
                    host.to_string()
                } else {
                    join_labels(&labels[labels.len() - 2..])
                }
            }
            Some(SuffixMatch {
                suffix_len,
                exception: true,
            }) => {
                // 例外规则：公共后缀 = 规则标签数 - 1，注册域 = 公共后缀 + 1 标签。
                let suffix_len = suffix_len - 1;
                registrable_from_suffix(&labels, suffix_len)
            }
            Some(SuffixMatch {
                suffix_len,
                exception: false,
            }) => registrable_from_suffix(&labels, suffix_len),
        }
    }

    /// 计算公共后缀的标签数及是否经例外规则修正。
    fn suffix_label_count(&self, labels: &[String]) -> Option<SuffixMatch> {
        // 在所有命中规则中选标签数最多者；同标签数时普通/例外优先于通配。
        let mut best: Option<(usize, RuleKind)> = None;
        for rule in &self.rules {
            if !rule_matches(rule, labels) {
                continue;
            }
            let n = rule.label_count();
            match best {
                None => best = Some((n, rule.kind)),
                Some((bn, bk)) => {
                    // 标签数多者胜；标签数相同者，非通配优先于通配。
                    let better = n > bn || (n == bn && rule.kind != RuleKind::Wildcard && bk == RuleKind::Wildcard);
                    if better {
                        best = Some((n, rule.kind));
                    }
                }
            }
        }

        let (n, kind) = best?;
        Some(SuffixMatch {
            suffix_len: n,
            exception: kind == RuleKind::Exception,
        })
    }
}

struct SuffixMatch {
    /// 公共后缀标签数。
    suffix_len: usize,
    /// 是否为例外规则命中（例外规则把后缀标签数减 1）。
    exception: bool,
}

/// 判断规则是否匹配主机名后缀。
///
/// 匹配语义：
/// - Normal：`labels` 末 `rule.labels.len()` 个标签与规则标签逐字相等。
/// - Exception：同 Normal（规则标签为去掉 `!` 的全序列）；例外信息经
///   [`RuleKind::Exception`] 表达，无需返回值承载。
/// - Wildcard：末 `rule.labels.len()` 个标签与规则标签相等，**且**紧邻其左侧
///   还有一个任意标签（即 `*.x.y` 要求主机形如 `*.x.y`，`*` 恰占一个标签）。
fn rule_matches(rule: &Rule, labels: &[String]) -> bool {
    let rlen = rule.labels.len();
    match rule.kind {
        RuleKind::Normal | RuleKind::Exception => {
            if labels.len() < rlen {
                return false;
            }
            let tail = &labels[labels.len() - rlen..];
            tail.iter().zip(rule.labels.iter()).all(|(a, b)| a == b)
        }
        RuleKind::Wildcard => {
            // 需要 host = <one-label> + rule.labels，故 labels.len() >= rlen + 1。
            if labels.len() < rlen + 1 {
                return false;
            }
            let tail = &labels[labels.len() - rlen..];
            tail.iter().zip(rule.labels.iter()).all(|(a, b)| a == b)
        }
    }
}

/// 给定主机标签序列与公共后缀标签数，计算注册域名。
fn registrable_from_suffix(labels: &[String], suffix_len: usize) -> String {
    // 主机恰好等于公共后缀本身 → 无注册域，返回主机全称。
    if labels.len() <= suffix_len {
        return join_labels(labels);
    }
    // 注册域 = 公共后缀 + 左侧一个标签。
    join_labels(&labels[labels.len() - suffix_len - 1..])
}

fn join_labels(slice: &[String]) -> String {
    slice.join(".")
}

/// 解析 PSL 文本为规则集。
///
/// 每行一条规则；`//` 起始或纯空白行忽略。规则标签做 ASCII 小写。
fn parse_rules(text: &str) -> Vec<Rule> {
    let mut rules = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        // 上游 PSL 的注释以 `//` 起始；整行注释跳过。
        // （注：行内嵌套 `//` 不视为注释，符合上游语义。）
        if line.starts_with("//") {
            continue;
        }
        // 分隔 IDN 注释（上游格式 `// ===BEGIN ICANN DOMAINS===` 为整行注释，
        // 单条规则行尾不会带 ` //`，故此处不处理行内 `//`）。

        let (kind, body) = if let Some(rest) = line.strip_prefix('!') {
            (RuleKind::Exception, rest)
        } else {
            (RuleKind::Normal, line)
        };

        let (kind, labels) = if let Some(rest) = body.strip_prefix("*.") {
            // 通配规则：去掉 `*.` 前缀，余下标签存入；`*` 占位由 kind 表达。
            let labels: Vec<String> = rest.split('.').map(|s| s.to_ascii_lowercase()).collect();
            (RuleKind::Wildcard, labels)
        } else {
            let labels: Vec<String> = body.split('.').map(|s| s.to_ascii_lowercase()).collect();
            (kind, labels)
        };

        if labels.is_empty() || labels.iter().any(|s| s.is_empty()) {
            // 跳过空标签规则（如孤立的 `*.` / `!.`）。
            continue;
        }
        rules.push(Rule { labels, kind });
    }
    rules
}

/// 内置默认规则集（ZeroWeb 项目原创，事实性公共后缀）。
///
/// 仅覆盖：常见 gTLD、典型 ccTLD 二级域（`co.uk`/`com.au` 等客观 DNS 结构）、
/// 文档化平台公共后缀（`*.github.io`），以及 PSL 例外规则语义示例（`!www.ck`）。
/// 这**不是**上游 MPL 管理的完整列表；覆盖范围有限。完整数据经
/// [`PublicSuffixList::from_rules`] 注入（见模块文档许可证说明）。
const DEFAULT_RULES: &str = include_str!("default_rules.txt");

#[cfg(test)]
mod tests;

//! No-Vary-Search 响应头 — URL variation config 解析与查询串等价判定。
//!
//! 规范锚点：draft-ietf-httpbis-no-vary-search（§4 数据模型 / §5 解析 / §6 等价
//! 比较 / §7 缓存键替换）。实现按 http-cache goal M3 的 WPT `fetch/http-cache`
//! 语料（WPT_REV 钉定）语义落地：
//! - `params` 裸 token / `?1` 按 **wildcard no-vary** 处理（2026-05 草案文本将布尔
//!   params 判为非法回落精确匹配，但语料 leg「params=?1 显式布尔」与 Chrome@
//!   WPT_REV 语义为 wildcard；以语料为准）。
//! - `except` 不与 `params`（wildcard）成对出现时按语料意图回落精确匹配（语料
//!   leg 标题「fall back to exact match」；草案 §5.1 step 7 单独 except 为合法
//!   vary-list，两解释对语料断言等价，取更保守者）。
//!
//! FIXME(no-vary-search): 草案转 RFC 后重估上述两处布尔/单独 except 语义。

/// URL variation config（draft-ietf-httpbis-no-vary-search §4）。
///
/// 不变式（§4）：`vary_params` 为 list 当且仅当 `no_vary_params` 为 wildcard。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UrlVariationConfig {
    /// `None` = wildcard（除 `vary_params` 外全部忽略）；`Some(list)` = 仅忽略列出的键。
    pub no_vary_params: Option<Vec<String>>,
    /// `None` = wildcard；`Some(list)` = 仅这些键参与缓存匹配（except 白名单形态）。
    pub vary_params: Option<Vec<String>>,
    /// `true` = 键顺序敏感（默认）；`key-order` 指令置 `false`。
    pub vary_on_key_order: bool,
}

impl UrlVariationConfig {
    /// 是否等价于默认配置（此时等价判定退化为查询串原文相等，draft §6 步骤 2）。
    pub fn is_default(&self) -> bool {
        self.vary_on_key_order
            && self.no_vary_params.as_ref().is_some_and(|params| params.is_empty())
            && self.vary_params.is_none()
    }
}

/// `No-Vary-Search` 头值的结构化字段成员。
#[derive(Debug, Clone, PartialEq, Eq)]
enum MemberValue {
    Boolean(bool),
    /// 内层列表项；仅带引号 string 是合法的 params/except 项（token/integer 非法）。
    InnerList(Vec<ListItem>),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ListItem {
    Key(String),
    Other,
}

impl MemberValue {
    fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }
}

/// 解析 `No-Vary-Search` 头值；`None` = 无效值 → 调用方回落精确匹配（默认配置）。
///
/// RFC 9651 dictionary 子集：`key`（裸键 ≡ `?1`）、`key=?1`、`key=("a" "b")`。
pub(crate) fn parse_no_vary_search(value: &str) -> Option<UrlVariationConfig> {
    let mut key_order: Option<bool> = None;
    let mut params: Option<ParamSpec> = None;
    let mut except: Option<Vec<String>> = None;
    for (name, member) in split_dictionary_members(value)? {
        match name.to_ascii_lowercase().as_str() {
            // draft §5.1 step 4——非布尔 key-order → 非法。
            "key-order" => key_order = Some(member.as_boolean()?),
            "params" => {
                params = Some(match member {
                    MemberValue::Boolean(true) => ParamSpec::Wildcard,
                    // 语料头注：params=false ≡ 无 no-vary 参数（未单测，取旧模型语义）。
                    MemberValue::Boolean(false) => ParamSpec::List(Vec::new()),
                    MemberValue::InnerList(items) => ParamSpec::List(parse_key_list(&items)?),
                    MemberValue::Other => return None,
                })
            }
            "except" => {
                except = Some(match member {
                    MemberValue::InnerList(items) => parse_key_list(&items)?,
                    // draft §5.2.1——except=?1 / token / string 均非法。
                    _ => return None,
                })
            }
            // 未知成员忽略（与 RFC 9111 §5.2.3 未知指令忽略原则一致）。
            _ => {}
        }
    }
    // `key-order` 缺省 = 顺序敏感（默认 true）；在场则取布尔取反（draft §5.1 step 4）。
    let vary_on_key_order = key_order.map(|flag| !flag).unwrap_or(true);
    let config = match (params, except) {
        // 均缺省（如仅 `key-order`）→ 默认参数集 + 已解析的 key-order。
        (None, None) => UrlVariationConfig {
            no_vary_params: Some(Vec::new()),
            vary_params: None,
            vary_on_key_order,
        },
        // `params`（wildcard）+ except 白名单——语料 leg 1/8/9 的 wildcard 形态。
        (Some(ParamSpec::Wildcard), Some(vary)) => UrlVariationConfig {
            no_vary_params: None,
            vary_params: Some(vary),
            vary_on_key_order,
        },
        // params 命名列表——列出的键不参与匹配。
        (Some(ParamSpec::List(no_vary)), None) => UrlVariationConfig {
            no_vary_params: Some(no_vary),
            vary_params: None,
            vary_on_key_order,
        },
        // params 列表 + except、双 wildcard、单独 except（语料 leg 12/13：回落精确
        // 匹配）→ 无效。
        _ => return None,
    };
    // 默认配置对调用方等价于无头——统一返回，由调用方 is_default 过滤。
    Some(config)
}

#[derive(Debug, Clone)]
enum ParamSpec {
    Wildcard,
    List(Vec<String>),
}

fn parse_key_list(items: &[ListItem]) -> Option<Vec<String>> {
    items
        .iter()
        .map(|item| match item {
            ListItem::Key(key) => parse_a_key(key),
            ListItem::Other => None,
        })
        .collect()
}

/// draft §5.3 parse a key——isomorphic 编码后 `+`→SP，再百分号解码，UTF-8 校验。
fn parse_a_key(key: &str) -> Option<String> {
    let bytes: Vec<u8> = key
        .as_bytes()
        .iter()
        .map(|&byte| if byte == b'+' { b' ' } else { byte })
        .collect();
    String::from_utf8(percent_decode(&bytes)).ok()
}

/// draft §6——给定查询串在 config 下的规范形式（等价类代表元）。
///
/// 默认配置返回 `None`：调用方应使用查询串原文精确匹配（§6 步骤 2）。
pub(crate) fn canonical_query(query: Option<&str>, config: &UrlVariationConfig) -> Option<String> {
    if config.is_default() {
        return None;
    }
    let pairs = query.map(parse_query_pairs).unwrap_or_default();
    let mut pairs: Vec<(String, String)> = match (&config.no_vary_params, &config.vary_params) {
        (Some(no_vary), _) => pairs.into_iter().filter(|(key, _)| !no_vary.contains(key)).collect(),
        (None, Some(vary)) => pairs.into_iter().filter(|(key, _)| vary.contains(key)).collect(),
        (None, None) => pairs,
    };
    if !config.vary_on_key_order {
        // draft §6 步骤 8——按解码后键的 code-unit 升序**稳定**排序（等键保持原序）。
        // FIXME(no-vary-search): WHATWG code unit 为 UTF-16；Rust 按 UTF-8 字节序
        // 比较，仅 BMP 外键名有差异（语料键均为 ASCII）。
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
    }
    Some(
        pairs
            .iter()
            .map(|(key, value)| {
                format!(
                    "{}={}",
                    percent_encode(key.as_bytes()),
                    percent_encode(value.as_bytes())
                )
            })
            .collect::<Vec<_>>()
            .join("&"),
    )
}

/// application/x-www-form-urlencoded 解析（WHATWG URL standard）：`&` 分割、空段
/// 跳过、首个 `=` 分键值、`+`→SP、百分号解码（无效 UTF-8 按 U+FFFD 替换）。
pub(crate) fn parse_query_pairs(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|segment| !segment.is_empty())
        .map(|segment| match segment.split_once('=') {
            Some((key, value)) => (form_urldecode(key), form_urldecode(value)),
            None => (form_urldecode(segment), String::new()),
        })
        .collect()
}

fn form_urldecode(text: &str) -> String {
    let bytes: Vec<u8> = text
        .bytes()
        .map(|byte| if byte == b'+' { b' ' } else { byte })
        .collect();
    String::from_utf8_lossy(&percent_decode(&bytes)).into_owned()
}

/// 百分号解码；非法 `%` 序列按字面保留（与 WHATWG 解码宽容行为一致）。
fn percent_decode(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut iter = bytes.iter().copied();
    while let Some(byte) = iter.next() {
        if byte == b'%' {
            let (hi, lo) = (iter.next(), iter.next());
            if let (Some(hi), Some(lo)) = (hi, lo)
                && let Some(value) = hex_pair(hi, lo)
            {
                out.push(value);
                continue;
            }
            out.push(b'%');
            if let Some(hi) = hi {
                out.push(hi);
                if let Some(lo) = lo {
                    out.push(lo);
                }
            }
            continue;
        }
        out.push(byte);
    }
    out
}

fn hex_pair(hi: u8, lo: u8) -> Option<u8> {
    let hi = (hi as char).to_digit(16)?;
    let lo = (lo as char).to_digit(16)?;
    Some((hi * 16 + lo) as u8)
}

/// 百分号编码（键规范序列化用）：仅 RFC 3986 unreserved（`A-Za-z0-9-._~`）保留。
fn percent_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &byte in bytes {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// NVS 自描述缓存键的段标记（http_cache 键布局：
/// `{无 query 基座}\0nvs={config 编码}\0cq={规范 query}{vary 后缀}`）。
/// `\0` 分隔与 [`crate::cache_key`] 的 vary 后缀同一约定（URL 规范化后不含裸 NUL，
/// 见 R3372）；config/规范 query 中的 `%` 已百分号编码，不与标记冲突。
pub(crate) const NVS_KEY_MARKER: &str = "\0nvs=";
pub(crate) const CQ_KEY_MARKER: &str = "\0cq=";

/// config 的自描述编码：`ko{0|1};nv={…};vy={…}`；wildcard 记 `*`，空列表记空段。
pub(crate) fn encode_config(config: &UrlVariationConfig) -> String {
    let encode_list = |list: &Option<Vec<String>>| match list {
        None => "*".to_string(),
        Some(params) => params
            .iter()
            .map(|key| percent_encode(key.as_bytes()))
            .collect::<Vec<_>>()
            .join(","),
    };
    format!(
        "ko{};nv={};vy={}",
        u8::from(config.vary_on_key_order),
        encode_list(&config.no_vary_params),
        encode_list(&config.vary_params)
    )
}

/// [`encode_config`] 的逆运算；非法编码返回 `None`（调用方按非 NVS 键处理）。
pub(crate) fn decode_config(encoded: &str) -> Option<UrlVariationConfig> {
    let mut parts = encoded.split(';');
    let ko = parts.next()?.strip_prefix("ko")?;
    let vary_on_key_order = match ko {
        "1" => true,
        "0" => false,
        _ => return None,
    };
    let decode_list = |part: Option<&str>, prefix: &str| {
        let payload = part?.strip_prefix(prefix)?;
        match payload {
            "*" => Some(None),
            "" => Some(Some(Vec::new())),
            encoded_list => encoded_list
                .split(',')
                .map(|key| String::from_utf8(percent_decode(key.as_bytes())).ok())
                .collect::<Option<Vec<_>>>()
                .map(Some),
        }
    };
    let no_vary_params = decode_list(parts.next(), "nv=")?;
    let vary_params = decode_list(parts.next(), "vy=")?;
    if parts.next().is_some() {
        return None;
    }
    Some(UrlVariationConfig {
        no_vary_params,
        vary_params,
        vary_on_key_order,
    })
}

/// 若键为 NVS 自描述键，拆出 `(无 query 基座, config, 规范 query)`。
pub(crate) fn split_nvs_key(key: &str) -> Option<(String, UrlVariationConfig, String)> {
    let marker = key.find(NVS_KEY_MARKER)?;
    let qbase = &key[..marker];
    let rest = &key[marker + NVS_KEY_MARKER.len()..];
    let (encoded, canonical) = rest.split_once(CQ_KEY_MARKER)?;
    Some((qbase.to_string(), decode_config(encoded)?, canonical.to_string()))
}

/// 剥离查询串与 fragment 的 URL 基座——NVS 条目的注册/扫描键（等价类以
/// scheme/host/port/path 为界，draft §8 安全边界）。
pub(crate) fn queryless_base(url: &str) -> Option<String> {
    url::Url::parse(url).ok().map(|mut parsed| {
        parsed.set_query(None);
        parsed.set_fragment(None);
        parsed.to_string()
    })
}

/// URL 的查询串（`None` = 无 `?`；空串 = 裸 `?`——两者在非常规 config 下等价）。
pub(crate) fn query_of(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.query().map(str::to_string))
}

/// RFC 9651 dictionary 分割（子集）：顶层逗号分割成员；尊重引号与内层列表括号。
fn split_dictionary_members(value: &str) -> Option<Vec<(String, MemberValue)>> {
    let mut members = Vec::new();
    let bytes = value.as_bytes();
    let mut depth = 0usize;
    let mut in_quotes = false;
    let mut start = 0usize;
    for (index, &byte) in bytes.iter().enumerate() {
        match byte {
            b'"' if depth == 0 => in_quotes = !in_quotes,
            b'(' if !in_quotes => depth += 1,
            b')' if !in_quotes => depth = depth.saturating_sub(1),
            b',' if depth == 0 && !in_quotes => {
                members.push(parse_member(&value[start..index])?);
                start = index + 1;
            }
            _ => {}
        }
    }
    if in_quotes || depth != 0 {
        return None;
    }
    members.push(parse_member(&value[start..])?);
    Some(members)
}

/// 单个 dictionary 成员：`key` / `key=?1` / `key=(…)`；裸键 ≡ `?1`（RFC 9651）。
fn parse_member(member: &str) -> Option<(String, MemberValue)> {
    let member = member.trim();
    if member.is_empty() {
        return None;
    }
    let mut depth = 0usize;
    let mut in_quotes = false;
    for (index, ch) in member.char_indices() {
        match ch {
            '"' if depth == 0 => in_quotes = !in_quotes,
            '(' if !in_quotes => depth += 1,
            ')' if !in_quotes => depth = depth.saturating_sub(1),
            '=' if depth == 0 && !in_quotes => {
                let key = member[..index].trim();
                if key.is_empty() {
                    return None;
                }
                return Some((key.to_string(), parse_member_value(member[index + 1..].trim())));
            }
            _ => {}
        }
    }
    Some((member.to_string(), MemberValue::Boolean(true)))
}

fn parse_member_value(raw: &str) -> MemberValue {
    match raw {
        "?1" => MemberValue::Boolean(true),
        "?0" => MemberValue::Boolean(false),
        _ if raw.starts_with('(') && raw.ends_with(')') => {
            MemberValue::InnerList(split_inner_list_items(&raw[1..raw.len() - 1]))
        }
        _ => MemberValue::Other,
    }
}

/// 内层列表项分割：空白分隔；带引号项为 string（RFC 9651 转义 `\"` `\\`），
/// 其余（token/integer）记 [`ListItem::Other`]。
fn split_inner_list_items(inner: &str) -> Vec<ListItem> {
    let mut items = Vec::new();
    let mut chars = inner.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                chars.next();
            }
            '"' => {
                chars.next();
                let mut text = String::new();
                let mut closed = false;
                while let Some(c) = chars.next() {
                    match c {
                        '"' => {
                            closed = true;
                            break;
                        }
                        '\\' => {
                            // RFC 9651 string：仅 DQUOTE 与反斜杠可转义。
                            if let Some(&next) = chars.peek()
                                && (next == '"' || next == '\\')
                            {
                                text.push(next);
                                chars.next();
                            } else {
                                text.push('\\');
                            }
                        }
                        _ => text.push(c),
                    }
                }
                items.push(if closed { ListItem::Key(text) } else { ListItem::Other });
            }
            _ => {
                while let Some(&next) = chars.peek()
                    && next != ' '
                    && next != '\t'
                {
                    chars.next();
                }
                items.push(ListItem::Other);
            }
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_of(header: &str) -> Option<UrlVariationConfig> {
        parse_no_vary_search(header)
    }

    // ── 解析（对照 draft §5.2.1 示例 + 语料 leg）──

    #[test]
    fn bare_params_token_is_wildcard_per_corpus() {
        // 语料 leg 1：裸 `params` = wildcard no-vary。
        let config = config_of("params, except=(\"dispatch\" \"uuid\")").unwrap();
        assert_eq!(config.no_vary_params, None);
        assert_eq!(config.vary_params, Some(vec!["dispatch".into(), "uuid".into()]));
        assert!(config.vary_on_key_order);
    }

    #[test]
    fn explicit_params_true_equals_bare_params() {
        // 语料 leg 8：`params=?1` ≡ 裸 `params`。
        let bare = config_of("params, except=(\"dispatch\" \"uuid\")").unwrap();
        let explicit = config_of("params=?1, except=(\"dispatch\" \"uuid\")").unwrap();
        assert_eq!(bare, explicit);
    }

    #[test]
    fn params_list_excludes_named_keys() {
        // 语料 leg 5：params=("a") → a 不参与匹配，其余 wildcard。
        let config = config_of("params=(\"a\")").unwrap();
        assert_eq!(config.no_vary_params, Some(vec!["a".into()]));
        assert_eq!(config.vary_params, None);
    }

    #[test]
    fn key_order_alone_is_non_default_config() {
        // 语料 leg 3：裸 `key-order` ≡ ?1 → 顺序不敏感，参数集保持默认。
        let config = config_of("key-order").unwrap();
        assert!(!config.vary_on_key_order);
        assert_eq!(config.no_vary_params, Some(Vec::new()));
        assert_eq!(config.vary_params, None);
        assert!(!config.is_default());
        // 显式 ?0 → 顺序敏感（即默认）。
        let off = config_of("key-order=?0").unwrap();
        assert!(off.vary_on_key_order);
        assert!(off.is_default());
    }

    #[test]
    fn invalid_values_fall_back_to_exact_match() {
        // draft §5.2.1 非法示例 + 语料 leg 12/13 → None（精确匹配回落）。
        for header in [
            "key-order=\"not a boolean\"",
            "params=\"not an inner list\"",
            "params=(not-a-string)",
            "params=?0, except=(\"x\")",
            "params=(\"a\"), except=(\"b\")",
            "except=(\"a\")",
            "except=?1",
            "except=\"not an inner list\"",
        ] {
            assert_eq!(config_of(header), None, "须回落精确匹配：{header}");
        }
    }

    #[test]
    fn parse_a_key_decodes_plus_and_percent() {
        assert_eq!(parse_a_key("%C3%A9+%E6%B0%97"), Some("é 気".to_string()));
        assert_eq!(parse_a_key("plain"), Some("plain".to_string()));
        // 无效 UTF-8 → None（draft §5.3 UTF-8 解码失败按错误处理）。
        assert_eq!(parse_a_key("%FF"), None);
    }

    // ── 等价判定（对照 draft §6 + 语料 leg）──

    fn equivalent(header: &str, query_a: Option<&str>, query_b: Option<&str>) -> bool {
        let config = config_of(header).expect("测试用 header 须合法");
        canonical_query(query_a, &config) == canonical_query(query_b, &config)
    }

    #[test]
    fn wildcard_ignores_all_but_except_keys() {
        // 语料 leg 1：a/b 差异被忽略，dispatch/uuid 保留。
        let header = "params, except=(\"dispatch\" \"uuid\")";
        assert!(equivalent(
            header,
            Some("dispatch=test&uuid=T1&a=1&b=2"),
            Some("dispatch=test&uuid=T1")
        ));
        // uuid 不同（跨测试）→ 不等价。
        assert!(!equivalent(
            header,
            Some("dispatch=test&uuid=T1"),
            Some("dispatch=test&uuid=T2")
        ));
    }

    #[test]
    fn default_config_is_exact_match() {
        // 语料 leg 2（ground truth）：无 NVS 头时顺序敏感、原文精确。
        let config = UrlVariationConfig {
            no_vary_params: Some(Vec::new()),
            vary_params: None,
            vary_on_key_order: true,
        };
        assert_eq!(canonical_query(Some("a=1&b=2"), &config), None);
        assert_eq!(canonical_query(Some("b=2&a=1"), &config), None);
    }

    #[test]
    fn key_order_makes_reordering_equivalent_but_not_renaming() {
        let header = "key-order";
        // 语料 leg 3：顺序不敏感。
        assert!(equivalent(header, Some("a=1&b=2"), Some("b=2&a=1")));
        // 语料 leg 4：键集/值不同仍不等价。
        assert!(!equivalent(header, Some("a=1&b=2"), Some("b=2")));
        assert!(!equivalent(header, Some("a=1&b=2"), Some("a=2&b=2")));
        assert!(!equivalent(header, Some("a=1&b=2"), Some("a=1&b=2&c=3")));
    }

    #[test]
    fn named_params_only_ignores_listed_keys() {
        // 语料 leg 5/6：params=("a") 下 a 差异忽略、b 差异不忽略。
        let header = "params=(\"a\")";
        assert!(equivalent(header, Some("a=1&b=2"), Some("a=99&b=2")));
        assert!(!equivalent(header, Some("a=1&b=2"), Some("a=1&b=99")));
        // 语料 leg 7：多键列表。
        let multi = "params=(\"a\" \"b\")";
        assert!(equivalent(multi, Some("a=1&b=2&c=3"), Some("a=9&b=8&c=3")));
    }

    #[test]
    fn named_params_with_key_order() {
        // 语料 leg 10/11：params=("utm"), key-order。
        let header = "params=(\"utm\"), key-order";
        assert!(equivalent(header, Some("utm=x&b=2&a=1"), Some("a=1&b=2&utm=y")));
        assert!(!equivalent(header, Some("utm=x&b=2&a=1"), Some("a=1&b=99&utm=y")));
    }

    #[test]
    fn form_urlencoded_canonicalization_per_draft_6_1() {
        let header = "key-order";
        // draft §6.1：null ≡ `?`；`%61=%78` ≡ `a=x`；空段丢弃；`a=` ≡ `a`；`+` ≡ SP。
        assert!(equivalent(header, None, Some("")));
        assert!(equivalent(header, Some("a=x"), Some("%61=%78")));
        assert!(equivalent(header, Some("a=x&&&&"), Some("a=x")));
        assert!(equivalent(header, Some("a="), Some("a")));
        assert!(equivalent(header, Some("a=%20"), Some("a= &")));
        assert!(equivalent(header, Some("a=+"), Some("a= ")));
    }

    // ── 键编码 ──

    #[test]
    fn config_encode_decode_roundtrip() {
        for config in [
            config_of("key-order").unwrap(),
            config_of("params, except=(\"dispatch\" \"uuid\")").unwrap(),
            config_of("params=(\"a\" \"b\"), key-order").unwrap(),
            config_of("params=?1, except=(\"dispatch\" \"uuid\")").unwrap(),
        ] {
            assert!(!config.is_default());
            assert_eq!(decode_config(&encode_config(&config)), Some(config));
        }
        assert_eq!(decode_config("bogus"), None);
        assert_eq!(decode_config("ko1;nv=a;vy=*;extra"), None);
    }

    #[test]
    fn nvs_key_split_roundtrip() {
        let config = config_of("params=(\"a\"), key-order").unwrap();
        let key = format!(
            "https://x.com/p{}{}{}a%3D1",
            NVS_KEY_MARKER,
            encode_config(&config),
            CQ_KEY_MARKER
        );
        let (qbase, decoded, canonical) = split_nvs_key(&key).unwrap();
        assert_eq!(qbase, "https://x.com/p");
        assert_eq!(decoded, config);
        assert_eq!(canonical, "a%3D1");
        assert_eq!(split_nvs_key("https://x.com/p\0vary=F=v"), None);
    }

    #[test]
    fn queryless_base_and_query_of() {
        assert_eq!(
            queryless_base("https://x.com/p?a=1#f").as_deref(),
            Some("https://x.com/p")
        );
        assert_eq!(query_of("https://x.com/p?a=1").as_deref(), Some("a=1"));
        // 裸 `?` 与无 `?`：query 区分空串与 None（非常规 config 下等价）。
        assert_eq!(query_of("https://x.com/p?"), Some(String::new()));
        assert_eq!(query_of("https://x.com/p"), None);
    }

    #[test]
    fn query_pairs_parse_per_whatwg() {
        assert_eq!(
            parse_query_pairs("a=1&b=2"),
            vec![("a".into(), "1".into()), ("b".into(), "2".into())]
        );
        // 空段丢弃；无值键 → 空串值；`+` → SP；百分号解码。
        assert_eq!(parse_query_pairs("a=x&&&&"), vec![("a".into(), "x".into())]);
        assert_eq!(parse_query_pairs("a"), vec![("a".into(), String::new())]);
        assert_eq!(
            parse_query_pairs("a=%20&b=+"),
            vec![("a".into(), " ".into()), ("b".into(), " ".into())]
        );
    }
}

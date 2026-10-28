//! URL 序列化组件 + IDNA IgnoreInvalidPunycode 回退（net-api 二期 M2-S2，从
//! `js_dom_bridge.rs` 拆出控制主文件行数）。
//!
//! 回退背景：WHATWG URL spec「domain to ASCII」非严格路径自 2023 起
//! IgnoreInvalidPunycode=true（[url.spec#concept-domain-to-ASCII](https://url.spec.whatwg.org/#concept-domain-to-ascii)，
//! whatwg/url#737）——`xn--` 标签 punycode 解码/验证失败时**原样保留**、整域不失败；
//! chromium/firefox/safari 全过。本仓 `url 2.5.8 → idna 1.1.0` 在
//! `uts46.rs to_ascii_from_cow` 硬编码 `ErrorPolicy::FailFast`（该枚举仅
//! FailFast/MarkErrors 两档，无 IgnoreInvalidPunycode 开关），解码无效即整域拒绝。
//! 回退实现：host 中「纯 ASCII、小写后 xn-- 前缀、[a-z0-9-] 形、crate 拒绝」的
//! 标签换占位标签重解析，序列化后按标签回写原样。期望失败面（标签混入非 ASCII、
//! `%` 等非法字符）不参与替换，维持 TypeError。

/// URL 序列化组件集（[`serialize_url_components`] 产出，[`components_to_json`] 消费）。
#[cfg(feature = "script-runtime")]
pub(crate) struct UrlComponents {
    pub(crate) protocol: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) hostname: String,
    pub(crate) port: String,
    pub(crate) host: String,
    pub(crate) origin: String,
    pub(crate) pathname: String,
    pub(crate) search: String,
    pub(crate) hash: String,
    pub(crate) href: String,
}

/// `Url` → 组件集。net-api 收尾：url crate 的 username()/password() getter 在
/// setter 后内部字节偏移不一致时可 panic（url 2.5.8 lib.rs:2881 "byte range
/// starts at 6 but ends at 4"——url-setters.any hostile username/password 面）。
/// 捕获 → None（调用方空串收场，不连累整跑）。
#[cfg(feature = "script-runtime")]
pub(crate) fn serialize_url_components(url: &url::Url) -> Option<UrlComponents> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let scheme = url.scheme();
        let hostname = url.host_str().unwrap_or("");
        let port = url.port().map(|p| p.to_string()).unwrap_or_default();
        let host = if port.is_empty() {
            hostname.to_string()
        } else {
            format!("{hostname}:{port}")
        };
        UrlComponents {
            protocol: format!("{scheme}:"),
            username: url.username().to_string(),
            password: url.password().unwrap_or("").to_string(),
            hostname: hostname.to_string(),
            port,
            host,
            origin: url.origin().ascii_serialization(),
            pathname: url.path().to_string(),
            // URL spec §search getter：「query 为 null 或空串返空串，否则 "?" + query」——
            // 裸尾 `?` 被 url crate 解析为 Some("")，须与 null 同归空串（WorkerLocation_
            // search_empty 面）。
            search: match url.query() {
                Some("") | None => String::new(),
                Some(q) => format!("?{q}"),
            },
            hash: url.fragment().map(|f| format!("#{f}")).unwrap_or_default(),
            href: url.as_str().to_string(),
        }
    }))
    .ok()
}

/// 组件集 → JSON 串（protocol/username/password/hostname/port/host/origin/
/// pathname/search/hash/href）。`js_dom_bridge::parse_url_to_json` 与
/// `set_url_part` 共用。
#[cfg(feature = "script-runtime")]
pub(crate) fn components_to_json(c: &UrlComponents) -> String {
    use super::json_str;
    format!(
        r#"{{"protocol":{},"username":{},"password":{},"hostname":{},"port":{},"host":{},"origin":{},"pathname":{},"search":{},"hash":{},"href":{}}}"#,
        json_str(&c.protocol),
        json_str(&c.username),
        json_str(&c.password),
        json_str(&c.hostname),
        json_str(&c.port),
        json_str(&c.host),
        json_str(&c.origin),
        json_str(&c.pathname),
        json_str(&c.search),
        json_str(&c.hash),
        json_str(&c.href),
    )
}

/// `Url` → JSON 串（组件路径的便捷封装）。
#[cfg(feature = "script-runtime")]
pub(crate) fn serialize_url(url: &url::Url) -> String {
    serialize_url_components(url)
        .map(|c| components_to_json(&c))
        .unwrap_or_default()
}

/// 占位标签前缀（纯 [a-z0-9]，crate 必然接受；回写按「全标签形 + 计数恰等」甄别，
/// 原 host 合法含同形标签则计数不符 → 放弃回退维持原失败，绝不错误回写）。
const ZW_PC_PLACEHOLDER_PREFIX: &str = "zwpcfb";

/// IDNA 回退可行的特殊 scheme（spec「special schemes」；host 走 domain 解析 +
/// IDNA，其余 scheme 的 opaque host 不涉 punycode）。
const ZW_PC_SPECIAL_SCHEMES: [&str; 6] = ["http", "https", "ws", "wss", "ftp", "file"];

/// 逐标签甄别：可回退 = 纯 ASCII 可打印（0x21..0x7F，天然排除 C0/空格/DEL）、
/// 小写后 xn-- 前缀、不含 spec forbidden domain code point（% < > [ \ ] ^ | :；
/// # / ? @ 由 authority 拆分天然排除）、crate 拒绝（用 url crate 自身探测——
/// 与生产解析同算法，不加依赖）。非 ASCII 标签与 `%`（forbidden domain 面，
/// expected-null）不参与，维持原失败。
#[cfg(feature = "script-runtime")]
fn pc_label_rescuable(label: &str, lower: &str) -> bool {
    lower.starts_with("xn--")
        && label.bytes().all(|b| {
            (0x21..=0x7e).contains(&b) && !matches!(b, b'%' | b'<' | b'>' | b'[' | b']' | b'\\' | b'^' | b'|' | b':')
        })
        && url::Url::parse(&format!("https://{lower}/")).is_err()
}

/// 占位标签形判定（`zwpcfb<纯数字>`）。
#[cfg(feature = "script-runtime")]
fn pc_label_is_placeholder(label: &str) -> bool {
    label.starts_with(ZW_PC_PLACEHOLDER_PREFIX)
        && label[ZW_PC_PLACEHOLDER_PREFIX.len()..]
            .bytes()
            .all(|b| b.is_ascii_digit())
}

/// host[:port] 串中 host span 的拆分（userinfo 剥离取最后一个 '@'；port 取最后
/// 一个 ':' 且其后全 ASCII 数字；`[` 开头为 IPv6 字面量 → None 无 IDNA 标签面）。
#[cfg(feature = "script-runtime")]
fn pc_split_host_port(authority: &str) -> Option<(usize, &str, usize)> {
    let host_off = authority.rfind('@').map(|i| i + 1).unwrap_or(0);
    let host_port = &authority[host_off..];
    if host_port.starts_with('[') {
        return None;
    }
    let port_rel = host_port
        .rfind(':')
        .filter(|i| host_port[i + 1..].bytes().all(|b| b.is_ascii_digit()));
    let host_len = port_rel.unwrap_or(host_port.len());
    Some((host_off, &host_port[..host_len], host_len))
}

/// 序列化 host 串（hostname/host/href host span 同构）的占位标签回写：第 i 个
/// 占位形标签 → 第 i 个原样标签；出现次数必须恰等于 rescues 数，否则 None
///（放弃回退维持原失败——防原 host 合法含同形标签时错误回写）。
#[cfg(feature = "script-runtime")]
fn pc_patch_host_str(serialized: &str, rescues: &[String]) -> Option<String> {
    // 先数后换：占位形标签数必须恰等于 rescues 数（原 host 合法含同形标签 →
    // 计数不符 → 放弃回退维持原失败，绝不错误回写）。
    if serialized.split('.').filter(|l| pc_label_is_placeholder(l)).count() != rescues.len() {
        return None;
    }
    let mut count = 0usize;
    let mut out = String::with_capacity(serialized.len());
    for (i, label) in serialized.split('.').enumerate() {
        if i > 0 {
            out.push('.');
        }
        if pc_label_is_placeholder(label) {
            out.push_str(&rescues[count]);
            count += 1;
        } else {
            out.push_str(label);
        }
    }
    Some(out)
}

/// IgnoreInvalidPunycode 回退解析（仅绝对特殊 scheme 输入）：主解析失败后，把
/// host 中可回退的 xn-- 标签换成占位标签重解析，返回（占位 URL, 有序原样标签）。
/// 其余失败（无 xn-- 可回退 / 混入非回退形态 / 占位计数不符）→ None 维持原行为。
#[cfg(feature = "script-runtime")]
pub(crate) fn pc_lenient_parse(input: &str) -> Option<(url::Url, Vec<String>)> {
    let colon = input.find(':')?;
    if !ZW_PC_SPECIAL_SCHEMES.contains(&input[..colon].to_ascii_lowercase().as_str()) {
        return None;
    }
    let rest = &input[colon + 1..];
    if !rest.starts_with("//") {
        return None;
    }
    let after = &rest[2..];
    let auth_end = after.find(['/', '?', '#']).unwrap_or(after.len());
    let (host_off, host, host_len) = pc_split_host_port(&after[..auth_end])?;
    // 逐标签甄别 + 占位重建（原样标签保留大小写交 crate 归一）。
    let mut rescues: Vec<String> = Vec::new();
    let mut rebuilt_host = String::with_capacity(host.len());
    for (i, label) in host.split('.').enumerate() {
        if i > 0 {
            rebuilt_host.push('.');
        }
        let lower = label.to_ascii_lowercase();
        if pc_label_rescuable(label, &lower) {
            rescues.push(lower);
            rebuilt_host.push_str(ZW_PC_PLACEHOLDER_PREFIX);
            rebuilt_host.push_str(&(rescues.len() - 1).to_string());
        } else {
            rebuilt_host.push_str(label);
        }
    }
    if rescues.is_empty() {
        return None;
    }
    let host_abs = colon + 3 + host_off;
    let mut rebuilt = String::with_capacity(input.len());
    rebuilt.push_str(&input[..host_abs]);
    rebuilt.push_str(&rebuilt_host);
    rebuilt.push_str(&input[host_abs + host_len..]);
    url::Url::parse(&rebuilt).ok().map(|u| (u, rescues))
}

/// 回退路径的组件回写：hostname/host/origin/href 的 host 部分按标签回写原样
/// （pathname/search/hash/username/password/protocol 无占位——只在 host 标签
/// 替换，不跨组件）。计数不符 → None。
#[cfg(feature = "script-runtime")]
pub(crate) fn pc_patch_components(mut c: UrlComponents, rescues: &[String]) -> Option<UrlComponents> {
    let hostname = pc_patch_host_str(&c.hostname, rescues)?;
    // origin 重建：特殊 scheme = protocol + "//" + host[:port]（port 数字无占位）。
    let origin = format!(
        "{}//{}",
        c.protocol,
        if c.port.is_empty() {
            hostname.clone()
        } else {
            format!("{}:{}", hostname, c.port)
        }
    );
    // href 的 authority host span 定位（与序列化后扫描同构）后回写。
    let href = {
        let h = &c.href;
        let scheme_end = h.find("://")? + 3;
        let after = &h[scheme_end..];
        let auth_end = after.find(['/', '?', '#']).unwrap_or(after.len());
        let (host_off, host_span, host_len) = pc_split_host_port(&after[..auth_end])?;
        let patched = pc_patch_host_str(host_span, rescues)?;
        let mut out = String::with_capacity(h.len());
        out.push_str(&h[..scheme_end + host_off]);
        out.push_str(&patched);
        out.push_str(&h[scheme_end + host_off + host_len..]);
        out
    };
    c.hostname = hostname;
    // host 组件 = hostname[:port]（port 数字无占位）——url.host getter 面。
    c.host = if c.port.is_empty() {
        c.hostname.clone()
    } else {
        format!("{}:{}", c.hostname, c.port)
    };
    c.origin = origin;
    c.href = href;
    Some(c)
}

#[cfg(all(test, feature = "script-runtime"))]
mod tests {
    use super::*;

    #[test]
    fn lenient_rescues_invalid_punycode_label() {
        let (url, rescues) = pc_lenient_parse("https://xn--ab-j1t/x").expect("rescuable");
        assert_eq!(rescues, vec!["xn--ab-j1t".to_string()]);
        assert_eq!(url.host_str(), Some("zwpcfb0"));
        let c = pc_patch_components(serialize_url_components(&url).unwrap(), &rescues).unwrap();
        let json = components_to_json(&c);
        assert!(json.contains(r#""hostname":"xn--ab-j1t""#), "{json}");
        assert!(json.contains(r#""host":"xn--ab-j1t""#), "{json}");
        assert!(json.contains(r#""href":"https://xn--ab-j1t/x""#), "{json}");
        assert!(json.contains(r#""origin":"https://xn--ab-j1t""#), "{json}");
    }

    #[test]
    fn lenient_keeps_non_rescuable_labels_failing() {
        // 非 ASCII 混入（expected-null P4 面）与 `%`（forbidden domain 面）不回退。
        assert!(pc_lenient_parse("https://xn--a-\u{e4}.pt/x").is_none());
        assert!(pc_lenient_parse("https://xn---%-u4o.xn--gff52t/").is_none());
        // 有效 punycode（crate 接受）不触发回退。
        assert!(pc_lenient_parse("https://xn--fa-hia.de/").is_none());
    }

    #[test]
    fn lenient_rescues_ascii_punycode_labels_beyond_hyphen_shape() {
        // `,` 非 forbidden domain code point（spec 非 fatal 验证错误，IdnaTestV2
        // 「V7 (含) + ignored 标注」期望 PASS 面）；crate deny list 过严 → 回退。
        let (url, rescues) = pc_lenient_parse("https://xn--6,-7i3cj157d.xn--0f9ao925c/x").expect("rescuable");
        // 第二标签 U1（连字位）crate 亦拒 → 一并回退原样保留（语料期望 verbatim）。
        assert_eq!(
            rescues,
            vec!["xn--6,-7i3cj157d".to_string(), "xn--0f9ao925c".to_string()]
        );
        let c = pc_patch_components(serialize_url_components(&url).unwrap(), &rescues).unwrap();
        assert_eq!(c.hostname, "xn--6,-7i3cj157d.xn--0f9ao925c");
    }

    #[test]
    fn lenient_multi_label_with_port_and_userinfo() {
        let (url, rescues) = pc_lenient_parse("https://u@xn--ab-j1t:8080/x?q=1#f").expect("rescuable");
        assert_eq!(rescues, vec!["xn--ab-j1t".to_string()]);
        assert_eq!(url.host_str(), Some("zwpcfb0"));
        assert_eq!(url.port(), Some(8080));
        let c = pc_patch_components(serialize_url_components(&url).unwrap(), &rescues).unwrap();
        assert!(c.href.starts_with("https://u@xn--ab-j1t:8080/x?q=1#f"), "{}", c.href);
        assert_eq!(c.origin, "https://xn--ab-j1t:8080");
    }

    #[test]
    fn patch_aborts_on_placeholder_shape_collision() {
        // 原 host 合法含同形标签 → 计数不符 → None（维持原失败，不错误回写）。
        assert!(pc_patch_host_str("zwpcfb0.zwpcfb0", &["xn--ab-j1t".to_string()]).is_none());
        assert_eq!(
            pc_patch_host_str("zwpcfb0.ok", &["xn--ab-j1t".to_string()]).as_deref(),
            Some("xn--ab-j1t.ok")
        );
    }

    #[test]
    fn lenient_ignores_relative_and_non_special() {
        assert!(pc_lenient_parse("/relative/path").is_none());
        assert!(pc_lenient_parse("non-special:opaque").is_none());
        assert!(pc_lenient_parse("about:blank").is_none());
        assert!(pc_lenient_parse("https://[::1]:8080/").is_none());
    }
}

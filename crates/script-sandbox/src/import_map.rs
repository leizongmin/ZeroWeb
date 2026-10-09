//! Import map — WHATWG HTML 模块说明符映射。
//!
//! 规范：<https://html.spec.whatwg.org/multipage/webappapis.html#import-maps>
//! 实现映射 JSON 解析（register an import map 的成员归一化，含
//! normalize a specifier key）与说明符解析（resolve a module specifier，
//! 含 resolve an imports match / resolve a URL-like module specifier）。
//!
//! 与完整规范的已知偏差（本切片边界，见各 FIXME）：
//! - 映射键迭代顺序为 `serde_json::Map` 的排序序而非 JSON 插入序（serde_json
//!   未启用 `preserve_order`）；仅在嵌套尾斜杠前缀键 / 多 scope 相互重叠时可观察。
//! - null 条目（Blocked）规范要求抛 TypeError 并终止全部回退；调用方
//!   （[`crate::es_module`]）当前无错误通道，按 Miss 降级继续。

use serde_json::Value;
use url::Url;

/// 说明符映射表：`键 → 映射目标`；`None` 为显式阻断条目（JSON null）。
/// 用 `Vec` 而非 `Map` 以保留解析顺序（scope 列表同理）。
type SpecifierMap = Vec<(String, Option<String>)>;

/// 解析后的 import map。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImportMap {
    /// 顶层 `imports` 映射。
    imports: SpecifierMap,
    /// `scopes`：scope 前缀（归一化 URL）→ 该 scope 的映射表。
    scopes: Vec<(String, SpecifierMap)>,
}

/// 单次说明符解析结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportMapResolution {
    /// 成功映射为 URL（字符串形式）。
    Resolved(String),
    /// 命中 null 条目 — 规范要求抛 TypeError 且不再回退。
    Blocked,
    /// 无匹配，调用方按无 import map 的行为继续。
    Miss,
}

impl ImportMap {
    /// 解析 import map JSON（`<script type="importmap">` 的文本内容）。
    ///
    /// `base_url` 为文档基址，用于归一化 URL 形态的键与 scope 前缀。
    /// JSON 非法或根/成员结构不符 → `Err`（规范：整张 map 不注册）；
    /// 空键、不可解析的 scope 前缀等不良成员按规范告警并跳过该成员。
    pub fn parse(json: &str, base_url: &Url) -> Result<Self, String> {
        let root: Value = serde_json::from_str(json).map_err(|e| format!("invalid import map JSON: {e}"))?;
        let Some(obj) = root.as_object() else {
            return Err("import map root is not an object".into());
        };
        let mut map = Self::default();
        if let Some(imports) = obj.get("imports") {
            let Some(imports) = imports.as_object() else {
                return Err("import map \"imports\" is not an object".into());
            };
            map.imports = normalize_specifier_map(imports, base_url);
        }
        if let Some(scopes) = obj.get("scopes") {
            let Some(scopes) = scopes.as_object() else {
                return Err("import map \"scopes\" is not an object".into());
            };
            for (prefix, inner) in scopes {
                // scope 前缀按基址 URL 解析；失败 → 跳过该 scope。
                let Ok(parsed) = base_url.join(prefix) else {
                    continue;
                };
                let Some(inner) = inner.as_object() else {
                    continue;
                };
                map.scopes
                    .push((parsed.to_string(), normalize_specifier_map(inner, base_url)));
            }
        }
        Ok(map)
    }

    /// [`Self::parse`] 的字符串基址变体——宿主侧（renderer/browser）无 `url`
    /// 依赖时的入口；基址解析失败 → `Err`（整张 map 不注册）。
    pub fn parse_from_page(json: &str, base_url: &str) -> Result<Self, String> {
        let base = Url::parse(base_url).map_err(|e| format!("invalid base URL: {e}"))?;
        Self::parse(json, &base)
    }

    /// 解析模块说明符（resolve a module specifier）。
    ///
    /// `base_url` 为导入方脚本基址（referring script's base URL）的字符串形式。
    /// 解析失败（基址非法）按 Miss 处理，由调用方走无 map 的回退路径。
    pub fn resolve(&self, specifier: &str, base_url: &str) -> ImportMapResolution {
        // https://html.spec.whatwg.org/multipage/webappapis.html#resolve-a-module-specifier
        let Ok(base) = Url::parse(base_url) else {
            return ImportMapResolution::Miss;
        };
        let as_url = resolve_url_like(specifier, &base);
        let normalized = as_url
            .as_ref()
            .map(|u| u.to_string())
            .unwrap_or_else(|| specifier.to_string());
        // map 阶段（scopes 选择 + 顶层 imports 回退）命中 → 直接返回。
        let via_map = self.match_map_tables(&base, &normalized, as_url.as_ref());
        if via_map != ImportMapResolution::Miss {
            return via_map;
        }
        // 仍无命中 → URL 形态说明符按 asURL 原样返回
        // （裸说明符由调用方按未映射处理，规范在此抛 TypeError）。
        match as_url {
            Some(url) => ImportMapResolution::Resolved(url.to_string()),
            None => ImportMapResolution::Miss,
        }
    }

    /// 仅当映射表实际命中（精确键或尾斜杠前缀键）时返回映射 URL；未命中返回
    /// `None`（含 Blocked 按模块注释 FIXME 降级）。与 [`Self::resolve`] 的差别在
    /// asURL 回退不参与——调用方（模块抓取/编译接线）借此保证无命中路径的字符串
    /// 形态与既有 `resolve_document_url` 产物逐字节一致（在跑站点资产零漂移）。
    pub fn resolve_mapped(&self, specifier: &str, base_url: &str) -> Option<String> {
        let base = Url::parse(base_url).ok()?;
        let as_url = resolve_url_like(specifier, &base);
        let normalized = as_url
            .as_ref()
            .map(|u| u.to_string())
            .unwrap_or_else(|| specifier.to_string());
        match self.match_map_tables(&base, &normalized, as_url.as_ref()) {
            ImportMapResolution::Resolved(url) => Some(url),
            _ => None,
        }
    }

    /// resolve a module specifier 的 map 阶段：按序尝试命中的 scopes，再回退顶层
    /// imports；全未命中返回 [`ImportMapResolution::Miss`]。
    fn match_map_tables(&self, base: &Url, normalized: &str, as_url: Option<&Url>) -> ImportMapResolution {
        let serialized_base = base.to_string();
        // scopes：scopePrefix 等于基址，或以 U+002F 结尾且是基址的代码单元前缀。
        for (prefix, scope_map) in &self.scopes {
            let in_scope =
                *prefix == serialized_base || (prefix.ends_with('/') && serialized_base.starts_with(prefix.as_str()));
            if !in_scope {
                continue;
            }
            match resolve_imports_match(normalized, as_url, scope_map) {
                ImportMapResolution::Miss => continue,
                other => return other,
            }
        }
        resolve_imports_match(normalized, as_url, &self.imports)
    }
}

/// 归一化一张说明符映射（register an import map 的成员处理 + normalize a specifier key）。
fn normalize_specifier_map(map: &serde_json::Map<String, Value>, base_url: &Url) -> SpecifierMap {
    // https://html.spec.whatwg.org/multipage/webappapis.html#normalize-a-specifier-key
    let mut out = Vec::new();
    for (key, value) in map {
        // 空键：告警并跳过该成员。
        if key.is_empty() {
            continue;
        }
        // URL 形态键解析为序列化 URL；裸键原样保留。
        let normalized_key = resolve_url_like(key, base_url)
            .map(|u| u.to_string())
            .unwrap_or_else(|| key.clone());
        // 成员值：字符串保留；其余（含 null）→ 显式阻断条目（规范：非字符串置 null）。
        let value = value.as_str().map(str::to_string);
        out.push((normalized_key, value));
    }
    out
}

/// resolve a URL-like module specifier：`/`、`./`、`../` 前缀（含 `//` scheme 相对）
/// 按基址解析为 URL；否则按绝对 URL 解析；裸说明符返回 None。
fn resolve_url_like(specifier: &str, base_url: &Url) -> Option<Url> {
    // https://html.spec.whatwg.org/multipage/webappapis.html#resolving-a-url-like-module-specifier
    if specifier.starts_with('/') || specifier.starts_with("./") || specifier.starts_with("../") {
        return base_url.join(specifier).ok();
    }
    Url::parse(specifier).ok()
}

/// resolve an imports match：在单张映射表中按序找精确键或尾斜杠前缀键。
fn resolve_imports_match(normalized: &str, as_url: Option<&Url>, map: &SpecifierMap) -> ImportMapResolution {
    // https://html.spec.whatwg.org/multipage/webappapis.html#resolve-an-imports-match
    for (key, value) in map {
        if key == normalized {
            // null 条目：显式阻断，终止整个解析（不回退）。
            return match value {
                Some(url) => ImportMapResolution::Resolved(url.clone()),
                None => ImportMapResolution::Blocked,
            };
        }
        // 前缀键：以 U+002F 结尾、是 normalized 的代码单元前缀，且 URL 形态
        // 说明符不是非 special URL（裸说明符无此限制）。
        if key.ends_with('/') && normalized.starts_with(key.as_str()) && as_url.is_none_or(is_special) {
            let Some(value) = value else {
                return ImportMapResolution::Blocked;
            };
            let after_prefix = &normalized[key.len()..];
            // afterPrefix 以映射目标为基址解析；目标或 afterPrefix 解析失败
            // → 规范要求 TypeError（阻断）。
            let Ok(target) = Url::parse(value) else {
                return ImportMapResolution::Blocked;
            };
            return match Url::options().base_url(Some(&target)).parse(after_prefix) {
                Ok(url) => ImportMapResolution::Resolved(url.to_string()),
                Err(_) => ImportMapResolution::Blocked,
            };
        }
    }
    ImportMapResolution::Miss
}

/// URL special scheme（https/http/ws/wss/ftp/file）。
fn is_special(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https" | "ws" | "wss" | "ftp" | "file")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_at(json: &str, base: &str) -> ImportMap {
        ImportMap::parse(json, &Url::parse(base).unwrap()).unwrap()
    }

    // github.com 实际部署形态：顶层扁平 exact-key imports，无 scopes。
    const GITHUB_MAP: &str = r#"{
        "imports": {
            "react": "https://github.githubassets.com/assets/react-e27d1b3e03961e68.js",
            "react-dom": "https://github.githubassets.com/assets/react-dom-e5fd46a22d5c4058.js",
            "react-dom/client": "https://github.githubassets.com/assets/react-dom-client-1b4a3ee065998cea.js",
            "scheduler": "https://github.githubassets.com/assets/scheduler-58b860b049ca307c.js"
        }
    }"#;

    #[test]
    fn flat_exact_key_map_resolves_bare_specifier() {
        let map = parse_at(GITHUB_MAP, "https://github.com/");
        assert_eq!(
            map.resolve("react", "https://github.com/microsoft/vscode"),
            ImportMapResolution::Resolved("https://github.githubassets.com/assets/react-e27d1b3e03961e68.js".into())
        );
        // 带子路径的裸键（"react-dom/client"）精确匹配，不受 "react-dom" 遮蔽。
        assert_eq!(
            map.resolve("react-dom/client", "https://github.com/microsoft/vscode"),
            ImportMapResolution::Resolved(
                "https://github.githubassets.com/assets/react-dom-client-1b4a3ee065998cea.js".into()
            )
        );
    }

    #[test]
    fn unmapped_bare_specifier_misses() {
        let map = parse_at(GITHUB_MAP, "https://github.com/");
        assert_eq!(
            map.resolve("left-pad", "https://github.com/"),
            ImportMapResolution::Miss
        );
    }

    #[test]
    fn url_like_specifier_passthrough_via_as_url() {
        let map = parse_at(GITHUB_MAP, "https://github.com/");
        // 绝对 URL 不在 map 中 → asURL 回退原样返回。
        assert_eq!(
            map.resolve("https://example.com/a.js", "https://github.com/"),
            ImportMapResolution::Resolved("https://example.com/a.js".into())
        );
        // 相对形态按导入方基址解析。
        assert_eq!(
            map.resolve("./dep.js", "https://github.com/assets/entry.js"),
            ImportMapResolution::Resolved("https://github.com/assets/dep.js".into())
        );
    }

    #[test]
    fn trailing_slash_prefix_mapping() {
        let map = parse_at(
            r#"{"imports": {"lib/": "https://cdn.example/lib/"}}"#,
            "https://x.test/",
        );
        assert_eq!(
            map.resolve("lib/util.js", "https://x.test/page"),
            ImportMapResolution::Resolved("https://cdn.example/lib/util.js".into())
        );
        // 非该前缀不受影响。
        assert_eq!(map.resolve("libx.js", "https://x.test/page"), ImportMapResolution::Miss);
    }

    #[test]
    fn null_entry_blocks() {
        let map = parse_at(r#"{"imports": {"blocked": null}}"#, "https://x.test/");
        assert_eq!(
            map.resolve("blocked", "https://x.test/page"),
            ImportMapResolution::Blocked
        );
    }

    #[test]
    fn non_string_value_becomes_blocked_entry() {
        // 规范：成员值非字符串（含数字）置 null（阻断）。
        let map = parse_at(r#"{"imports": {"num": 42}}"#, "https://x.test/");
        assert_eq!(map.resolve("num", "https://x.test/page"), ImportMapResolution::Blocked);
    }

    #[test]
    fn scope_selects_by_base_url_prefix() {
        let json = r#"{
            "imports": {"react": "https://cdn.example/react.js"},
            "scopes": {"/private/": {"react": "https://cdn.example/private-react.js"}}
        }"#;
        let map = parse_at(json, "https://x.test/");
        assert_eq!(
            map.resolve("react", "https://x.test/private/page"),
            ImportMapResolution::Resolved("https://cdn.example/private-react.js".into())
        );
        assert_eq!(
            map.resolve("react", "https://x.test/public/page"),
            ImportMapResolution::Resolved("https://cdn.example/react.js".into())
        );
    }

    #[test]
    fn scope_exact_base_url_match() {
        let json = r#"{
            "imports": {"m": "https://cdn.example/m.js"},
            "scopes": {"https://x.test/exact": {"m": "https://cdn.example/exact-m.js"}}
        }"#;
        let map = parse_at(json, "https://x.test/");
        // 精确等于 scope 前缀（无尾斜杠相等分支）。
        assert_eq!(
            map.resolve("m", "https://x.test/exact"),
            ImportMapResolution::Resolved("https://cdn.example/exact-m.js".into())
        );
    }

    #[test]
    fn empty_key_is_skipped() {
        let map = parse_at(r#"{"imports": {"": "https://x.test/a.js"}}"#, "https://x.test/");
        assert!(map.imports.is_empty());
        assert_eq!(map.resolve("", "https://x.test/"), ImportMapResolution::Miss);
    }

    #[test]
    fn relative_key_normalized_against_parse_base() {
        // URL 形态键在解析期按文档基址归一化为序列化 URL。
        let map = parse_at(
            r#"{"imports": {"./rel.js": "https://x.test/a.js"}}"#,
            "https://x.test/dir/page",
        );
        assert_eq!(
            map.imports,
            vec![("https://x.test/dir/rel.js".into(), Some("https://x.test/a.js".into()))]
        );
        // 导入 "./rel.js"（URL 形态）与归一化键精确匹配。
        assert_eq!(
            map.resolve("./rel.js", "https://x.test/dir/other.js"),
            ImportMapResolution::Resolved("https://x.test/a.js".into())
        );
    }

    #[test]
    fn invalid_json_and_bad_shapes_err() {
        let base = Url::parse("https://x.test/").unwrap();
        assert!(ImportMap::parse("not json", &base).is_err());
        assert!(ImportMap::parse("[1,2]", &base).is_err());
        assert!(ImportMap::parse(r#"{"imports": "nope"}"#, &base).is_err());
        assert!(ImportMap::parse(r#"{"scopes": 3}"#, &base).is_err());
        // 无 imports/scopes 的合法空 map。
        assert_eq!(ImportMap::parse("{}", &base).unwrap(), ImportMap::default());
    }

    #[test]
    fn bad_scope_prefix_is_skipped() {
        // 空 scope 前缀按基址解析为合法 scope（RFC 3986 空引用 = 基址本身）；
        // 只有解析失败的键才跳过。
        let ok = parse_at(
            r#"{"scopes": {"": {"react": "https://x.test/r.js"}}}"#,
            "https://x.test/",
        );
        assert_eq!(ok.scopes.len(), 1);
        let map = parse_at(
            r#"{"scopes": {"https://[bad": {"react": "https://x.test/r.js"}}}"#,
            "https://x.test/",
        );
        assert!(map.scopes.is_empty());
    }

    #[test]
    fn non_special_url_skips_prefix_matching() {
        // data: URL（非 special scheme）形态说明符不参与前缀匹配：
        // 前缀键 "data:text/" 本可命中 normalized 前缀，但 asURL 非 special → 跳过，
        // 最终按 asURL 原样返回。
        let map = parse_at(r#"{"imports": {"data:text/": "https://x.test/a"}}"#, "https://x.test/");
        assert_eq!(
            map.resolve("data:text/plain,x", "https://x.test/page"),
            ImportMapResolution::Resolved("data:text/plain,x".into())
        );
    }

    #[test]
    fn resolve_mapped_only_reports_actual_matches() {
        let map = parse_at(GITHUB_MAP, "https://github.com/");
        // 裸说明符命中 → 映射 URL。
        assert_eq!(
            map.resolve_mapped("react", "https://github.com/page"),
            Some("https://github.githubassets.com/assets/react-e27d1b3e03961e68.js".into())
        );
        // 未命中裸说明符 → None（调用方走原生解析）。
        assert_eq!(map.resolve_mapped("left-pad", "https://github.com/page"), None);
        // 未命中的 URL 形态说明符 → None（asURL 回退不参与——接线侧零漂移的关键）。
        assert_eq!(
            map.resolve_mapped(
                "https://github.githubassets.com/assets/entry.js",
                "https://github.com/page"
            ),
            None
        );
        // 相对形态未命中 → None。
        assert_eq!(
            map.resolve_mapped("./dep.js", "https://github.com/assets/entry.js"),
            None
        );
    }
}

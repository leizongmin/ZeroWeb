//! Import map — WHATWG HTML 模块说明符映射。
//!
//! 规范：<https://html.spec.whatwg.org/multipage/webappapis.html#import-maps>
//! 实现映射 JSON 解析（register an import map 的成员归一化，含
//! normalize a specifier key 与映射值按基址归一化）与说明符解析
//! （resolve a module specifier，含 resolve an imports match /
//! resolve a URL-like module specifier）。
//!
//! 与完整规范的已知偏差（本切片边界，见各 FIXME）：
//! - 键迭代顺序为 `serde_json::Map` 的排序序而非 JSON 插入序（serde_json
//!   未启用 `preserve_order`）：注册收尾已按规范要求重排为代码单元降序
//!   （s262），仅当多个不同原始键归一化为同一键时，取值归属按排序序而非
//!   JSON 插入序判定。
//! - null 条目（Blocked）规范要求抛 TypeError 并终止全部回退：es_module
//!   transform 面（s259）与 webview/宿主编译·动态解析面（s260，经
//!   [`ImportMap::resolve_mapped`] 三态直报）均已按错误通道终止。

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
    /// `base_url` 为文档基址，用于归一化 URL 形态的键、scope 前缀与映射值
    /// （值按基址解析为地址 URL 存储）。JSON 非法或根/成员结构不符 → `Err`
    /// （规范：整张 map 不注册）；空键、不可解析的 scope 前缀等不良成员按
    /// 规范告警并跳过该成员；不可解析的映射值、尾斜杠键的地址不带尾斜杠
    /// → 置 null（阻断条目），该成员仍注册。
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
            // sort and normalize scopes：scope 前缀同按代码单元降序——更具体
            // （更长）前缀先迭代命中。
            // https://html.spec.whatwg.org/multipage/webappapis.html#sort-and-normalize-scopes
            map.scopes.sort_by(|a, b| code_unit_cmp(&b.0, &a.0));
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

    /// 仅当映射表实际命中（精确键或尾斜杠前缀键）时报告三态结果：
    /// [`ImportMapResolution::Resolved`]（映射 URL）/[`ImportMapResolution::Blocked`]
    /// （null 条目）/[`ImportMapResolution::Miss`](无命中)。与 [`Self::resolve`]
    /// 的差别在 asURL 回退不参与（Miss 恒表无命中）——调用方（模块抓取/编译/
    /// 动态解析接线）借此保证无命中路径的字符串形态与既有 `resolve_document_url`
    /// 产物逐字节一致（在跑站点资产零漂移），并按 Blocked 终止全部回退
    /// （spec：null 条目抛 TypeError，不落 asURL/相对解析——s260 三面接线）。
    pub fn resolve_mapped(&self, specifier: &str, base_url: &str) -> ImportMapResolution {
        let Ok(base) = Url::parse(base_url) else {
            return ImportMapResolution::Miss;
        };
        let as_url = resolve_url_like(specifier, &base);
        let normalized = as_url
            .as_ref()
            .map(|u| u.to_string())
            .unwrap_or_else(|| specifier.to_string());
        self.match_map_tables(&base, &normalized, as_url.as_ref())
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

/// 归一化一张说明符映射（register an import map 的成员处理 + normalize a specifier key
/// + sorting and normalizing a module specifier map 的值归一化与降序排序步骤）。
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
        // 成员值（sorting and normalizing 步骤 3-6）：非字符串（含 null）→ 显式
        // 阻断条目；字符串按基址解析为地址 URL 并存序列化形式——解析失败、或
        // 尾斜杠键的地址不以尾斜杠收尾 → 置 null（阻断条目），该成员仍注册。
        // https://html.spec.whatwg.org/multipage/webappapis.html#sort-and-normalize-a-module-specifier-map
        let value = match value.as_str() {
            Some(s) => resolve_url_like(s, base_url).and_then(|url| {
                if key.ends_with('/') && !url.as_str().ends_with('/') {
                    None
                } else {
                    Some(url.to_string())
                }
            }),
            None => None,
        };
        out.push((normalized_key, value));
    }
    // 收尾步骤：按键代码单元降序排列——resolve an imports match 按序首个命中
    // 生效，降序使嵌套尾斜杠前缀键最长前缀优先（"foo/bar/" 先于 "foo/"）。
    // https://html.spec.whatwg.org/multipage/webappapis.html#sort-and-normalize-a-module-specifier-map
    out.sort_by(|a, b| code_unit_cmp(&b.0, &a.0));
    out
}

/// 代码单元序比较（排序步骤的 "code unit less than"）：按 UTF-16 代码单元
/// 逐一比较；与 Rust 字节序仅在非 BMP 字符与 U+E000..=U+FFFF 区段交错时
/// 有别，这里按规范精确实现。
fn code_unit_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
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
            // 目标解析失败臂对 parse 产物不可达（映射值在注册期已归一化为绝对
            // URL，s258 阻断面收敛），保留作存储值不变式守卫。
            let Ok(target) = Url::parse(value) else {
                return ImportMapResolution::Blocked;
            };
            // afterPrefix 以映射目标为基址解析；解析失败，或解析产物序列化不以
            // 目标序列化为代码单元前缀（回溯越出前缀域）→ 规范要求 TypeError
            // （阻断）。
            // https://html.spec.whatwg.org/multipage/webappapis.html#resolve-an-imports-match
            return match Url::options().base_url(Some(&target)).parse(after_prefix) {
                Ok(url) if url.as_str().starts_with(target.as_str()) => ImportMapResolution::Resolved(url.to_string()),
                _ => ImportMapResolution::Blocked,
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
        // 最终按 asURL 原样返回。（映射值须以尾斜杠收尾——尾斜杠键的地址不带
        // 尾斜杠会在注册期置 null，见 s258 归一化钉面。）
        let map = parse_at(r#"{"imports": {"data:text/": "https://x.test/"}}"#, "https://x.test/");
        assert_eq!(
            map.resolve("data:text/plain,x", "https://x.test/page"),
            ImportMapResolution::Resolved("data:text/plain,x".into())
        );
    }

    // s260：resolve_mapped 返回三态（Resolved/Blocked/Miss）——None 塌缩曾把
    // Blocked 降级为 Miss，接线侧 parent-join 继续抓取被阻断 URL（bypass 面）。
    #[test]
    fn resolve_mapped_only_reports_actual_matches() {
        let map = parse_at(GITHUB_MAP, "https://github.com/");
        // 裸说明符命中 → 映射 URL。
        assert_eq!(
            map.resolve_mapped("react", "https://github.com/page"),
            ImportMapResolution::Resolved("https://github.githubassets.com/assets/react-e27d1b3e03961e68.js".into())
        );
        // 未命中裸说明符 → Miss（调用方走原生解析）。
        assert_eq!(
            map.resolve_mapped("left-pad", "https://github.com/page"),
            ImportMapResolution::Miss
        );
        // 未命中的 URL 形态说明符 → Miss（asURL 回退不参与——接线侧零漂移的关键）。
        assert_eq!(
            map.resolve_mapped(
                "https://github.githubassets.com/assets/entry.js",
                "https://github.com/page"
            ),
            ImportMapResolution::Miss
        );
        // 相对形态未命中 → Miss。
        assert_eq!(
            map.resolve_mapped("./dep.js", "https://github.com/assets/entry.js"),
            ImportMapResolution::Miss
        );
    }

    // s260：null 条目经 resolve_mapped 报 Blocked（此前 Option 签名塌缩为 None =
    // Miss，接线侧 parent-join 继续抓取被阻断 URL）。精确键与 URL 形态键两臂。
    #[test]
    fn resolve_mapped_null_entry_is_blocked_not_miss_s260() {
        let map = parse_at(r#"{"imports": {"gone": null}}"#, "https://x.test/");
        assert_eq!(
            map.resolve_mapped("gone", "https://x.test/page"),
            ImportMapResolution::Blocked
        );
        let map2 = parse_at(r#"{"imports": {"https://x.test/gone.js": null}}"#, "https://x.test/");
        assert_eq!(
            map2.resolve_mapped("https://x.test/gone.js", "https://x.test/page"),
            ImportMapResolution::Blocked
        );
    }

    // ---- s258：映射值注册期归一化（sorting and normalizing a module specifier
    // map 步骤 4-7）——值按文档基址解析为地址 URL，解析失败或尾斜杠键的地址
    // 不带尾斜杠 → 置 null（阻断条目），该成员仍注册。----
    // https://html.spec.whatwg.org/multipage/webappapis.html#sort-and-normalize-a-module-specifier-map

    #[test]
    fn path_absolute_value_normalized_at_registration_s258() {
        // 尾斜杠前缀键 + path-absolute 映射值：注册期按文档基址归一化；此前值
        // 原样保留、resolve 期裸 Url::parse 失败 → 误 Blocked（s258 修复面）。
        let map = parse_at(
            r#"{"imports": {"utils/": "/assets/utils/"}}"#,
            "https://x.test/dir/page",
        );
        assert_eq!(
            map.resolve("utils/a.js", "https://x.test/dir/page"),
            ImportMapResolution::Resolved("https://x.test/assets/utils/a.js".into())
        );
        // resolve_mapped 面（模块抓取/编译接线）同样拿到绝对 URL。
        assert_eq!(
            map.resolve_mapped("utils/a.js", "https://x.test/dir/page"),
            ImportMapResolution::Resolved("https://x.test/assets/utils/a.js".into())
        );
    }

    #[test]
    fn relative_value_normalized_at_registration_s258() {
        // 精确键 + 相对映射值：注册期归一化，解析产物为绝对 URL（此前原样返回
        // 相对串，靠接线侧 resolve_document_url 再拼基址偶然可用）。
        let map = parse_at(
            r#"{"imports": {"shim": "./shim.js", "up": "../up.js"}}"#,
            "https://x.test/dir/page",
        );
        assert_eq!(
            map.resolve("shim", "https://x.test/dir/other.js"),
            ImportMapResolution::Resolved("https://x.test/dir/shim.js".into())
        );
        assert_eq!(
            map.resolve("up", "https://x.test/dir/other.js"),
            ImportMapResolution::Resolved("https://x.test/up.js".into())
        );
        // 归一化发生在注册期：存储值已是序列化 URL（顺序为 s262 注册收尾的
        // 代码单元降序排列）。
        assert_eq!(
            map.imports,
            vec![
                ("up".into(), Some("https://x.test/up.js".into())),
                ("shim".into(), Some("https://x.test/dir/shim.js".into())),
            ]
        );
    }

    #[test]
    fn unresolvable_value_becomes_null_entry_s258() {
        // 步骤 5：映射值不是 URL 形态（裸串解析失败）→ 置 null（阻断条目），
        // 该成员仍注册；此前裸串原样保留并按 Resolved 返回。
        let map = parse_at(r#"{"imports": {"foo": "bar"}}"#, "https://x.test/");
        assert_eq!(map.resolve("foo", "https://x.test/page"), ImportMapResolution::Blocked);
    }

    #[test]
    fn trailing_slash_key_requires_trailing_slash_address_s258() {
        // 步骤 6：尾斜杠键的地址序列化不以尾斜杠收尾 → 置 null（阻断条目）；
        // 此前按该地址当基址解析出父级路径（"https://cdn.example/a.js"）。
        let map = parse_at(r#"{"imports": {"lib/": "https://cdn.example/lib"}}"#, "https://x.test/");
        assert_eq!(
            map.resolve("lib/a.js", "https://x.test/page"),
            ImportMapResolution::Blocked
        );
    }

    #[test]
    fn scope_value_normalized_against_parse_base_s258() {
        // scope 内映射值同经注册期归一化（normalize_specifier_map 共用）。
        let map = parse_at(
            r#"{"imports": {}, "scopes": {"/s/": {"utils/": "/assets/utils/"}}}"#,
            "https://x.test/dir/page",
        );
        assert_eq!(
            map.resolve("utils/a.js", "https://x.test/s/app.js"),
            ImportMapResolution::Resolved("https://x.test/assets/utils/a.js".into())
        );
    }

    #[test]
    fn backtracking_above_prefix_target_blocked_s259() {
        // resolve an imports match 步骤 6：afterPrefix 解析产物序列化不以映射
        // 目标序列化为代码单元前缀（回溯越出前缀域）→ TypeError（阻断）；
        // 此前按成功返回越界 URL。
        let map = parse_at(
            r#"{"imports": {"lib/": "https://x.test/a/b/"}}"#,
            "https://x.test/dir/page",
        );
        assert_eq!(
            map.resolve("lib/../c.js", "https://x.test/dir/page"),
            ImportMapResolution::Blocked
        );
        // 前缀域内组合不受影响（判别臂）。
        assert_eq!(
            map.resolve("lib/util.js", "https://x.test/dir/page"),
            ImportMapResolution::Resolved("https://x.test/a/b/util.js".into())
        );
    }

    // ---- s262：映射表与 scope 列表的注册期排序（sort and normalize 两算步的
    // 降序收尾）——resolve an imports match / scopes 按序首个命中生效，降序使
    // 嵌套尾斜杠前缀键最长前缀优先、嵌套 scope 更具体前缀优先。此前按
    // `serde_json::Map` 升序迭代入 `Vec`，最短前缀 / 最浅 scope 遮蔽。----
    // https://html.spec.whatwg.org/multipage/webappapis.html#sort-and-normalize-a-module-specifier-map
    #[test]
    fn nested_prefix_keys_longest_prefix_wins_s262() {
        let map = parse_at(
            r#"{"imports": {"a/": "https://x.test/short/", "a/b/": "https://x.test/long/b/"}}"#,
            "https://x.test/",
        );
        // "a/b/" 代码单元大于 "a/"，降序先迭代 → 最长前缀命中。
        assert_eq!(
            map.resolve("a/b/c.js", "https://x.test/page"),
            ImportMapResolution::Resolved("https://x.test/long/b/c.js".into())
        );
        // resolve_mapped 面（模块抓取/编译接线）同经 match_map_tables 顺序。
        assert_eq!(
            map.resolve_mapped("a/b/c.js", "https://x.test/page"),
            ImportMapResolution::Resolved("https://x.test/long/b/c.js".into())
        );
    }

    #[test]
    fn nested_scopes_most_specific_wins_s262() {
        let map = parse_at(
            r#"{"imports": {"m": "https://x.test/top.js"}, "scopes": {
                "/s/": {"m": "https://x.test/shallow.js"},
                "/s/t/": {"m": "https://x.test/deep.js"}
            }}"#,
            "https://x.test/",
        );
        // "/s/t/" 代码单元大于 "/s/"，降序先迭代 → 更具体 scope 命中。
        assert_eq!(
            map.resolve("m", "https://x.test/s/t/page"),
            ImportMapResolution::Resolved("https://x.test/deep.js".into())
        );
        // 浅层 scope 页面不受影响（判别臂）。
        assert_eq!(
            map.resolve("m", "https://x.test/s/page"),
            ImportMapResolution::Resolved("https://x.test/shallow.js".into())
        );
    }
}

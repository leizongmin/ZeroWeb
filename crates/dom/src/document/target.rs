//! Document `:target` 伪类判定 —— 拆自 `mod.rs`（rule 5 单文件 <2000 行，R3283）。
//!
//! 本模块为 [`super::Document`] 的「目标锚点」面（`:target` 的权威判定）。R3283 为闭合 DOM
//! 选择器与 style-system CSS 的一致性，把 `:target`（CSS Selectors L3 §6.6.2：唯一匹配当前
//! 文档 URL fragment 的元素）从「CSS 解析器识别但 DOM `query.rs` 与 style-system matcher 双双
//! 走 `_ => false`」补全为 DOM/CSS 同源判定。
//!
//! 作为 `document` 模块的**子模块**，可访问 [`super::Document`] 的私有字段（`nodes`/`url`/`id_map`）
//! 与 `mod.rs` 的私有查询助手——Rust 隐私规则：私有项对定义模块及其后代可见，故无需任何可见性
//! 改动（行为新增重组，镜像 R3281 `lang_dir.rs`、R3280 `form_state.rs`、R3164 `shadow.rs` 拆分模式）。

use super::Document;

impl Document {
    /// `:target` 的权威判定（CSS Selectors L3 §6.6.2）。
    ///
    /// 当前文档 URL 的 fragment（`#` 后部分，经百分号解码）所指向的唯一元素——即
    /// `getElementById(decoded_fragment)` 命中的元素。fragment 为空或无元素命中时无 `:target`。
    /// HTML §5.8：fragment 解码后须为合法 `id`；多个同 id 元素时取 `id_map` 记录的首个
    /// （树序首个，与 `get_element_by_id` 一致）。
    ///
    /// 供 DOM `:target` 选择器（`element_matches_selector`）与 style-system `:target` CSS
    /// 匹配共享，保证选择器与样式一致。读取当前文档 URL（`self.url`，导航层 `set_url` 注入），
    /// 无 URL 或无 fragment → 无目标元素。
    pub fn is_target_element(&self, node: crate::node::NodeId) -> bool {
        // 提取 fragment：URL 末个 `#` 之后的子串（去 `#`）。无 `#` → 无 fragment → 无 :target。
        // 仅取字符串运算，不依赖 url crate（dom crate 无 url 依赖，避免新增）。
        let Some(fragment) = self.url.as_deref().and_then(|u| u.rsplit_once('#').map(|(_, f)| f)) else {
            return false;
        };
        let decoded = percent_decode(fragment);
        // 空 fragment（如 `page#`）不指向任何元素。
        if decoded.is_empty() {
            return false;
        }
        // get_element_by_id 走 id_map（树序首个同 id 元素），返回的节点即 :target。
        self.get_element_by_id(&decoded) == Some(node)
    }
}

/// 百分号解码（RFC 3986 §2.1，仅 fragment 用途的最小实现）。
///
/// `%HH`（HH 为合法十六进制）→ 对应字节；`%` 后非两位十六进制 → 原样保留 `%`（容错，
/// 非法编码不抛错，按字面比较）。UTF-8 多字节序列经逐字节解码后天然重组。空输入 → 空输出。
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2]))
        {
            out.push((h << 4) | l);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    // 容错：非 UTF-8 字节序列（罕见，非法 % 编码）退回 lossy，避免 panic。
    String::from_utf8(out).unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).into_owned())
}

/// 十六进制字符 → 0–15，非法 → None。
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

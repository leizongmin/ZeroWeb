//! R5314（XHTML 自闭合桥接）：content_is_xml 文档的非 void 自闭合起始标签预变换。
//!
//! html5ever 按 HTML 语义解析，`<div/>` 的 `/` 仅被 acknowledge 后忽略（WHATWG
//! HTML tree construction：self-closing flag 对 HTML 元素 ignored），元素保持
//! 开启吞掉后继兄弟；chromium 对 .xht（application/xhtml+xml）走 XML 解析，`/`
//! 即空元素闭合（XML 1.0 §3.1 EmptyElemTag）。ZW 无 XML 解析通道，content_is_xml
//! 文档在树形上与 chromium 分叉：兄弟嵌进自闭合元素内部（块级 margin 折叠把兄
//! 弟链压成一个可见盒——numbers-units-010/012/013 ref 双盒渲单盒）。
//! 本模块在喂给 html5ever 前把 `<tag … />` 预变换为 `<tag …></tag>`，解析产物
//! = XML 空元素语义。FIXME(html5ever): 接入 xml5ever/XML 解析通道后移除。

use std::borrow::Cow;

/// R5314 开关：`ZW_XHTML_SELF_CLOSE=0` 回退 HTML 旧序（自闭合仅 acknowledge 后
/// 忽略、元素开启）。LazyLock 构造期单次读取——进程内 set_var 须先于首个 XHTML
/// 文档解析（独立测试进程，R5313 教训）。
static XHTML_SELF_CLOSE: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_XHTML_SELF_CLOSE").as_deref() != Ok("0"));

/// R5314 判据：源级预扫描 DOCTYPE 声明的 **PUBLIC 引用串**含 "xhtml"（大小写不
/// 敏感）——与 `Document::detect_and_set_content_is_xml` 的 parsed public_id 判据
/// 同源，SYSTEM-only 形态两处一致不判 XML（文档标志与预变换不散页）。
fn source_is_xhtml_doctype(html: &str) -> bool {
    let b = html.as_bytes();
    // 首个 `<!doctype`（ci）：只扫 `<` 位点，文档头部即命中。
    let mut start = None;
    let mut s = 0;
    while let Some(p) = html[s..].find('<') {
        let a = s + p;
        if a + 9 <= b.len() && b[a + 1..a + 9].eq_ignore_ascii_case(b"!doctype") {
            start = Some(a);
            break;
        }
        s = a + 1;
    }
    let Some(s) = start else { return false };
    // 声明串到收尾 `>`（引号感知——public/system id 语法上可含 `>`）。
    let mut quote = 0u8;
    let mut j = s + 9;
    let mut end = b.len();
    while j < b.len() {
        let c = b[j];
        if quote != 0 {
            if c == quote {
                quote = 0;
            }
        } else if c == b'"' || c == b'\'' {
            quote = c;
        } else if c == b'>' {
            end = j;
            break;
        }
        j += 1;
    }
    let decl = &html[s..end];
    let Some(p) = decl.to_ascii_lowercase().find("public") else {
        return false;
    };
    // PUBLIC 后首个引号串 = public id（spec doctype 声明序：PUBLIC "pub" "sys"?）。
    let db = decl.as_bytes();
    let mut k = p + 6;
    while k < db.len() && db[k].is_ascii_whitespace() {
        k += 1;
    }
    if k >= db.len() || (db[k] != b'"' && db[k] != b'\'') {
        return false;
    }
    let q = db[k];
    let Some(rel) = decl[k + 1..].find(q as char) else {
        return false;
    };
    decl[k + 1..k + 1 + rel].to_ascii_lowercase().contains("xhtml")
}

/// R5314 辅助：带收尾定界的整段原样拷贝；返回新扫描位点（未收尾 → 拷剩余、
/// 返 `html.len()`）。
fn copy_verbatim_until(out: &mut String, html: &str, i: usize, delim: &str, delim_len: usize) -> usize {
    match html[i..].find(delim) {
        Some(e) => {
            out.push_str(&html[i..i + e + delim_len]);
            i + e + delim_len
        }
        None => {
            out.push_str(&html[i..]);
            html.len()
        }
    }
}

/// R5314 主变换：XHTML 文档（[`source_is_xhtml_doctype`] 命中）中非 void 元素的
/// 自闭合起始标签 `<tag … />` → `<tag …></tag>`，解析产物 = XML 空元素语义。
/// 非 XHTML 文档零拷贝直返（`Cow::Borrowed` 快路径，纯 HTML 语料零开销）。
/// 字节面状态机：注释/CDATA/XML PI 整段原样拷贝（XML 字面语义内不出真实标签）；
/// 标签体引号感知（`<div title=">"/>` 的引号内 `>` 不误判标签收尾）；`/>` 前邻
/// `=`（未引号属性值收尾 `/`，XML 本身非法）保守跳过不改写。
pub(crate) fn expand_xhtml_self_closing(html: &str) -> Cow<'_, str> {
    if !source_is_xhtml_doctype(html) {
        return Cow::Borrowed(html);
    }
    if !*XHTML_SELF_CLOSE {
        return Cow::Borrowed(html);
    }
    let b = html.as_bytes();
    // 标签名字节（XML NameChar 渲染面子集）：含 `-`/`_`/`.`/`:`——`<my-tag/>`、
    // `<svg:rect/>` 的名字须取全，闭合标签才补得对。
    let is_name_byte = |c: u8| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.' | b':');
    let mut out = String::with_capacity(html.len() + 64);
    let mut i = 0;
    let mut changed = false;
    while i < b.len() {
        if b[i] != b'<' {
            // 快路径：非标记区整段拷贝（到下一个 `<`）。
            let next = b[i..].iter().position(|&x| x == b'<').map_or(b.len(), |p| i + p);
            out.push_str(&html[i..next]);
            i = next;
            continue;
        }
        let rest = &b[i..];
        if rest.starts_with(b"<!--") {
            i = copy_verbatim_until(&mut out, html, i, "-->", 3);
        } else if rest.starts_with(b"<![CDATA[") {
            i = copy_verbatim_until(&mut out, html, i, "]]>", 3);
        } else if rest.starts_with(b"<?") {
            // XML PI 收尾 `?>`（PI 内容可含 `<`，不可按标签扫）。
            i = copy_verbatim_until(&mut out, html, i, "?>", 2);
        } else if rest.starts_with(b"<!") || rest.starts_with(b"</") {
            // 声明（doctype 等）/结束标签：到 `>` 原样拷贝。
            i = copy_verbatim_until(&mut out, html, i, ">", 1);
        } else {
            // 起始标签：取名字，引号感知扫到 `>`，判自闭合。
            let tag_start = i + 1;
            let name_len = b[tag_start..].iter().take_while(|&&c| is_name_byte(c)).count();
            let name = &html[tag_start..tag_start + name_len];
            let mut quote = 0u8;
            let mut j = tag_start + name_len;
            let mut self_close_at = None;
            let end = loop {
                if j >= b.len() {
                    break b.len();
                }
                let c = b[j];
                if quote != 0 {
                    if c == quote {
                        quote = 0;
                    }
                } else if c == b'"' || c == b'\'' {
                    quote = c;
                } else if c == b'>' {
                    // 自闭合判定：`>` 前紧邻 `/`，且 `/` 前非 `=`（未引号值尾守卫）。
                    if j > tag_start && b[j - 1] == b'/' && (j < 2 || b[j - 2] != b'=') {
                        self_close_at = Some(j - 1);
                    }
                    break j + 1;
                }
                j += 1;
            };
            // void 元素自闭合本就是 HTML 合法形态（br/img/…），原样交 html5ever；
            // 名字空（`< />`）不改写。
            match self_close_at {
                Some(sc) if name_len > 0 && !crate::serializer::is_void_element(name) => {
                    out.push_str(&html[i..sc]);
                    out.push_str("></");
                    out.push_str(name);
                    out.push('>');
                    changed = true;
                }
                _ => out.push_str(&html[i..end]),
            }
            i = end;
        }
    }
    if changed { Cow::Owned(out) } else { Cow::Borrowed(html) }
}

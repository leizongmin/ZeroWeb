//! k117（css-mixins-1 §cssom）：`@mixin` 规则 CSSOM 读回单源——canonical cssText
//! 序列化＋cssRules 子规则树（raw token 重建，防编译 AST「裸声明合成 &」与显式
//! `&` 块的同构歧义）。供 wire 臂（[`super::css_wire`] `@mixin` 条目）与
//! insertRule 写面回调 `__zw_parse_mixin_rule` 单实现双通道（k115
//! `__zw_parse_function_rule` 同构）。纯函数，无 DOM 状态。
//!
//! canonical 形（WPT mixin-cssom.tentative 期望 template literal 逐字节）：
//! ```text
//! @mixin --m1 {
//!   @result {
//!   color: green;
//!   & { --foo: bar; }
//! }
//! }
//! ```
//! —— `@mixin <name>[ (<params>)] {` 头行（空参数列表剥 `()`）、体行恒 2 空格
//! 缩进（`@result` 内部与头行平级——Chrome 序列化怪形照抄）、直缀声明一行一条、
//! 嵌套块单行（块内声明 `; ` 连写）、`@result` 闭 `}` 与外层闭 `}` 顶格。
//! `@layer` 在任何深度丢弃（css-mixins-1：@layer invalid inside mixins——
//! sibling 保留，driving: mixin-parsing layer_dropped 系）。
//!
//! https://drafts.csswg.org/css-mixins-1/#cssom

use zero_css_parser::Token;

/// cssRules 子规则物化树节点。
pub(crate) struct MixinChild {
    /// `"style"`（CSSStyleRule 形——`.style` 写面）｜`"result"`｜`"contents"`｜
    /// `"at"`（@media 等通用块保形）。
    pub kind: &'static str,
    /// style 臂选择器串（`&`／`div`）。
    pub selector_text: String,
    /// style 臂声明连写（`color: green;`）。
    pub style_text: String,
    /// 子规则 canonical 全文。
    pub css_text: String,
    /// 嵌套子规则。
    pub children: Vec<MixinChild>,
    /// k149：合成 `&` 载体标志（块内裸声明合成的首条 style child——行集逐 decl
    /// 行 `  name: value;`）vs 显式嵌套块 style child（行集单行 `sel { decls }`）。
    /// shim 父链行集重组按此分流（cssText 写面父链重算）。
    pub synthetic: bool,
}

/// `@mixin` 单规则解析产物（canonical 头部＋cssText＋cssRules 树）。
pub(crate) struct MixinRuleParts {
    /// 定义名（dashed-ident 原形，`--m1`）。
    pub name: String,
    /// canonical 参数串（`--arg <length>: 1em, --other-arg`——`type()` 单分量
    /// 剥壳、联合保壳、`type(*)` 略）；空参数列表／无参数列表恒空串。
    pub params_text: String,
    /// canonical 全文。
    pub css_text: String,
    /// cssRules 物化树。
    pub children: Vec<MixinChild>,
}

/// 声明连写（`n: v; n2: v2;`——cssom-1 serialize a declaration block 形，与
/// [`super::css_wire`] wire 通道同款）。
fn decls_joined(decls: &[(String, String)]) -> String {
    decls
        .iter()
        .map(|(n, v)| format!("{}: {};", n, v))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 值段 tokens → canonical 文本（Display join；tokenizer 已把连续空白并成单
/// Whitespace，join 即单空格分隔）。FIXME(k117+)：string 字面量内空白形会被
/// Display/空白跳过路径破坏——mixin 语料值无此形，通用值保真待按需。
fn tokens_text(tokens: &[Token]) -> String {
    let mut out = String::new();
    for t in tokens {
        if matches!(t, Token::Comment(_)) {
            continue;
        }
        if matches!(t, Token::Whitespace) {
            if !out.ends_with(' ') && !out.is_empty() {
                out.push(' ');
            }
            continue;
        }
        out.push_str(&t.to_string());
    }
    out.trim().to_string()
}

/// `type()` 剥壳（k115 css_type_text 同语义——css-mixins-1 参数/returns `<css-type>`
/// 文法：单分量剥壳、联合（含 `|`）保 `type()` 包裹、`type(*)` 等价无类型）。
fn strip_type_fn(ty: &str) -> &str {
    let s = ty.trim();
    if let Some(inner) = s.strip_prefix("type(").and_then(|r| r.strip_suffix(')')) {
        let inner = inner.trim();
        return if inner == "*" { "*" } else { inner };
    }
    s
}

fn css_type_text(ty_canon: &str) -> String {
    if ty_canon.contains('|') {
        format!("type({})", ty_canon)
    } else {
        ty_canon.to_string()
    }
}

/// 头部解析产物：名＋canonical 参数串。
struct Header {
    name: String,
    params_text: String,
}

/// 头部 token 级解析：`--name [ type(...) ] [: default]` 逗号分隔（参数表括号
/// 深度计——`type(<length> | auto)` 内括号不误切）。名 token 双形：无参列表形
/// `@mixin --m4 {`（`--m4` 后空白 → Ident）／参数列表形 `@mixin --m1() {`
/// （`(` 紧随 → Function token 自带 `(` 语义——css-syntax-3 function token，
/// 探针实证 `--m1()` tokenize 形）。
fn parse_header(tokens: &[Token]) -> Option<Header> {
    let mut i = 0usize;
    // 跳过前导空白
    while matches!(tokens.get(i), Some(Token::Whitespace) | Some(Token::Comment(_))) {
        i += 1;
    }
    let (name, started_in_fn) = match tokens.get(i)? {
        Token::Ident(n) => (n.clone(), false),
        Token::Function(n) => (n.clone(), true),
        _ => return None,
    };
    if !name.starts_with("--") {
        return None;
    }
    i += 1;
    // 参数列表（可选）：Ident 名形消费显式 LParen；Function 名形 `(` 已随名吞
    // ——两种形进入参数体后 depth 基准同为 1（配对 RParen 到 0 即参数表闭）。
    let mut params_text = String::new();
    let started = if started_in_fn {
        true
    } else if matches!(tokens.get(i), Some(Token::LParen)) {
        i += 1;
        true
    } else {
        false
    };
    if started {
        let mut depth = 1i32;
        let mut seg: Vec<Token> = Vec::new();
        let mut segments: Vec<Vec<Token>> = Vec::new();
        while let Some(t) = tokens.get(i) {
            match t {
                Token::LParen | Token::Function(_) => {
                    depth += 1;
                    seg.push(t.clone());
                }
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    seg.push(t.clone());
                }
                Token::Comma if depth == 1 => {
                    segments.push(std::mem::take(&mut seg));
                }
                Token::Eof => break,
                _ => seg.push(t.clone()),
            }
            i += 1;
        }
        if !seg.is_empty() {
            segments.push(seg);
        }
        let parts: Vec<String> = segments.iter().filter_map(|s| canonical_param(s)).collect();
        params_text = parts.join(", ");
    }
    Some(Header { name, params_text })
}

/// 单参数段 canonical：`--name [ <type> ] [: default]`（`type()` 剥壳、`:` 前
/// 无空格后有、default 原文保形）。无 `--` 前缀名（非法）返 None 整段丢弃。
fn canonical_param(seg: &[Token]) -> Option<String> {
    let mut i = 0usize;
    while matches!(seg.get(i), Some(Token::Whitespace) | Some(Token::Comment(_))) {
        i += 1;
    }
    // 名双形：`--arg `（后随空白 → Ident）／`--other-arg)`（名后紧邻 RParen
    // 时 token 为 Function——css-syntax-3 function token 语义）。
    let pname = match seg.get(i)? {
        Token::Ident(n) => n.as_str(),
        Token::Function(n) => n.as_str(),
        _ => return None,
    };
    if !pname.starts_with("--") {
        return None;
    }
    let mut out = crate::js_dom_bridge::css_wire::serialize_css_ident(pname);
    i += 1;
    // pname 与 type 注解间空白（`--arg type(<length>)` 形——无空白紧邻形为
    // Function 判定域，此处 canonical 化空白分隔形）。
    while matches!(seg.get(i), Some(Token::Whitespace) | Some(Token::Comment(_))) {
        i += 1;
    }
    // type(...) 注解（Function("type") 起的括号段）
    if let Some(Token::Function(f)) = seg.get(i)
        && f.eq_ignore_ascii_case("type")
    {
        // 收集到匹配 RParen（Function 自带 `(` 语义——深度从 1 起算）
        let mut ty = String::from("type(");
        let mut depth = 1i32;
        i += 1;
        while let Some(t) = seg.get(i) {
            match t {
                Token::LParen | Token::Function(_) => {
                    depth += 1;
                    ty.push_str(&t.to_string());
                }
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        ty.push(')');
                        i += 1;
                        break;
                    }
                    ty.push(')');
                }
                Token::Whitespace => {
                    if !ty.ends_with(' ') {
                        ty.push(' ');
                    }
                }
                Token::Eof => break,
                _ => ty.push_str(&t.to_string()),
            }
            i += 1;
        }
        let canon = strip_type_fn(&ty);
        if canon != "*" {
            out.push(' ');
            out.push_str(&css_type_text(canon));
        }
    } else if matches!(seg.get(i), Some(Token::Delim('<'))) {
        // k117：canonical 再解析形（wire 读面 `__zw_parse_mixin_rule` 对
        // canonical cssText 二次解析——输入 type 注解已是剥壳后直传文本
        // `<length>`（Delim('<')…Delim('>') 三 token，无 type() 壳））。
        // 收集到匹配 `>` 原文保形即终形（无二次 canonical 加工）。
        let mut ty = String::from("<");
        let mut depth = 1i32;
        i += 1;
        while let Some(t) = seg.get(i) {
            match t {
                Token::Delim('<') => {
                    depth += 1;
                    ty.push('<');
                }
                Token::Delim('>') => {
                    depth -= 1;
                    ty.push('>');
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                Token::Whitespace => {
                    if !ty.ends_with(' ') {
                        ty.push(' ');
                    }
                }
                Token::Eof => break,
                _ => ty.push_str(&t.to_string()),
            }
            i += 1;
        }
        out.push(' ');
        out.push_str(ty.trim());
    }
    // default 段（`:` 后到段尾）
    let mut j = i;
    while matches!(seg.get(j), Some(Token::Whitespace) | Some(Token::Comment(_))) {
        j += 1;
    }
    if matches!(seg.get(j), Some(Token::Colon)) {
        let dv = tokens_text(&seg[j + 1..]);
        if !dv.is_empty() {
            out.push_str(": ");
            out.push_str(&dv);
        }
    }
    Some(out)
}

/// 体 raw 重建（单源）：返回行集（每行含 2 空格缩进）＋子规则树。
/// 裸声明集聚合为首条 style child（合成 `&` 规则语义——invalidation 前置
/// `cssRules.length == 2` 形：`[style(&{color}), style(&{--foo})]`）。
fn scan_block(tokens: &[Token]) -> (Vec<String>, Vec<MixinChild>) {
    let mut lines: Vec<String> = Vec::new();
    let mut children: Vec<MixinChild> = Vec::new();
    // 裸声明集（合成 & style child 载体）
    let mut bare: Vec<(String, String)> = Vec::new();

    let mut i = 0usize;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Whitespace | Token::Comment(_) | Token::Semicolon => {
                i += 1;
            }
            Token::RBrace | Token::Eof => break,
            Token::AtKeyword(name) => {
                let lname = name.to_ascii_lowercase();
                if lname == "layer" {
                    // k117：@layer 任何深度丢弃（extent 消耗——`;` 或匹配块）。
                    i = skip_extent(tokens, i);
                    continue;
                }
                if lname == "contents" {
                    // `@contents;` → 行（Statement 体）
                    lines.push("  @contents;".to_string());
                    children.push(MixinChild {
                        kind: "contents",
                        selector_text: String::new(),
                        style_text: String::new(),
                        css_text: "@contents;".to_string(),
                        children: Vec::new(),
                        synthetic: false,
                    });
                    i = skip_to_semicolon(tokens, i);
                    continue;
                }
                // @result / @media / @supports / @container / @starting-style /
                // @scope：块形展开（@result 递归行展开；其余 child kind "at"
                // 同构展开——cssText 无全文断言面，取与 @result 同构形）。
                let is_result = lname == "result";
                if let Some((prelude, body, next)) = collect_block(tokens, i) {
                    let (child_lines, child_children) = scan_block(&tokens[body.0..body.1]);
                    let header = if is_result {
                        "  @result {".to_string()
                    } else {
                        format!("  {} {{", tokens_text(&tokens[prelude.0..prelude.1]))
                    };
                    let mut sub = vec![header];
                    sub.extend(child_lines);
                    sub.push("}".to_string());
                    let css_text = sub.join("\n");
                    lines.extend(sub);
                    children.push(MixinChild {
                        kind: if is_result { "result" } else { "at" },
                        selector_text: String::new(),
                        style_text: String::new(),
                        css_text,
                        children: child_children,
                        synthetic: false,
                    });
                    i = next;
                } else {
                    i += 1;
                }
            }
            _ => {
                // 声明段（`ident: value;`）或嵌套块（`prelude { ... }`）：
                // 向前探 `:`（声明）vs `{`（块）判定。
                match classify_member(tokens, i) {
                    MemberKind::Decl(decl_end, next) => {
                        let seg = &tokens[i..decl_end];
                        if let Some(ci) = seg.iter().position(|t| matches!(t, Token::Colon)) {
                            let prop = tokens_text(&seg[..ci]);
                            let value = tokens_text(&seg[ci + 1..]);
                            if !prop.is_empty() {
                                bare.push((prop, value));
                            }
                        }
                        i = next;
                    }
                    MemberKind::Block(block_prelude, body, next) => {
                        let prelude_text = tokens_text(&tokens[block_prelude.0..block_prelude.1]);
                        let (child_lines, child_children) = scan_block(&tokens[body.0..body.1]);
                        // 嵌套块单行形：块内行合成单行（`& { --foo: bar; }`）——
                        // 块内裸声明行保留原形、嵌套块行 join 空格。k149：逐行 trim
                        // 后 join——多 decl 块体（写面 flush 回写单行串再 parse）子行
                        // 前导缩进叠加 `bar;   color` 三空格 round-trip 漂移实证
                        //（mixin-invalidation t1 判据 cssText 期望形不符根因）。
                        let inner = child_lines
                            .iter()
                            .map(|l| l.trim())
                            .collect::<Vec<_>>()
                            .join(" ");
                        let line = format!("  {} {{ {} }}", prelude_text, inner.trim());
                        lines.push(line.clone());
                        // style child（selectorText + styleText——块内裸声明集）
                        let bare_text = collect_bare_decls(&tokens[body.0..body.1]);
                        children.push(MixinChild {
                            kind: "style",
                            selector_text: prelude_text,
                            style_text: bare_text,
                            css_text: line.trim_start().to_string(),
                            children: child_children,
                            synthetic: false,
                        });
                        i = next;
                    }
                    MemberKind::Junk => {
                        i += 1;
                    }
                }
            }
        }
    }

    // 裸声明集 → 首条 style child（合成 `&` 规则——前置插入保源序）
    if !bare.is_empty() {
        let style_text = decls_joined(&bare);
        let css_text = format!("& {{ {} }}", style_text);
        children.insert(
            0,
            MixinChild {
                kind: "style",
                selector_text: "&".to_string(),
                style_text,
                css_text,
                children: Vec::new(),
                synthetic: true,
            },
        );
        // 行集：裸声明逐行前置（`  color: green;`）
        let mut decl_lines: Vec<String> = bare.iter().map(|(n, v)| format!("  {}: {};", n, v)).collect();
        decl_lines.extend(lines);
        lines = decl_lines;
    }
    (lines, children)
}

/// 块内裸声明集连写（嵌套块 style child 的 styleText——`& { --foo: bar; }`
/// 写面 `--foo`/`color` setter 载体）。
fn collect_bare_decls(tokens: &[Token]) -> String {
    let mut bare: Vec<(String, String)> = Vec::new();
    let mut i = 0usize;
    while i < tokens.len() {
        match classify_member(tokens, i) {
            MemberKind::Decl(decl_end, next) => {
                let seg = &tokens[i..decl_end];
                if let Some(ci) = seg.iter().position(|t| matches!(t, Token::Colon)) {
                    let prop = tokens_text(&seg[..ci]);
                    let value = tokens_text(&seg[ci + 1..]);
                    if !prop.is_empty() {
                        bare.push((prop, value));
                    }
                }
                i = next;
            }
            MemberKind::Block(_, _, next) => {
                i = next;
            }
            MemberKind::Junk => {
                i += 1;
            }
        }
    }
    decls_joined(&bare)
}

/// 区间（tok index 对）。
struct Span(usize, usize);

/// 体成员分类：声明（到顶层 `;`）/ 嵌套块（`{` 深度配对）/ 无法识别单步。
enum MemberKind {
    /// 声明段结束（`:` 所在段，不含 `;`）＋下一扫描位。
    Decl(usize, usize),
    /// 块前奏区间＋块体区间＋下一扫描位。
    Block(Span, Span, usize),
    Junk,
}

fn classify_member(tokens: &[Token], start: usize) -> MemberKind {
    let mut i = start;
    let mut depth: i32 = 0;
    while let Some(t) = tokens.get(i) {
        match t {
            Token::Whitespace | Token::Comment(_) => {}
            Token::LBrace if depth == 0 => {
                // 块：前奏 = start..i；体 = 配对间。
                let mut d = 1usize;
                let mut j = i + 1;
                while let Some(t2) = tokens.get(j) {
                    match t2 {
                        Token::LBrace => d += 1,
                        Token::RBrace => {
                            d -= 1;
                            if d == 0 {
                                return MemberKind::Block(Span(start, i), Span(i + 1, j), j + 1);
                            }
                        }
                        Token::Eof => break,
                        _ => {}
                    }
                    j += 1;
                }
                return MemberKind::Block(Span(start, i), Span(i + 1, j), j);
            }
            Token::Semicolon if depth == 0 => {
                return MemberKind::Decl(i, i + 1);
            }
            Token::LParen | Token::LBracket | Token::Function(_) => depth += 1,
            Token::RParen | Token::RBracket => depth -= 1,
            Token::AtKeyword(name) if depth == 0 && name.eq_ignore_ascii_case("layer") => {
                // 块内 @layer 丢弃：声明段截断（parsing 'a nested style rule'
                // 形 `div { @layer bar; color: green }`——color 保留独立段）。
                let skip = skip_extent(tokens, i);
                if skip > i {
                    return MemberKind::Decl(i, skip);
                }
            }
            _ => {}
        }
        i += 1;
    }
    if i > start {
        MemberKind::Decl(i, i)
    } else {
        MemberKind::Junk
    }
}

/// `@layer` extent 消耗：`;` 或匹配块（深度计），返回下一扫描位。
fn skip_extent(tokens: &[Token], at: usize) -> usize {
    let mut i = at;
    let mut depth: i32 = 0;
    while let Some(t) = tokens.get(i) {
        match t {
            Token::LBrace => depth += 1,
            Token::RBrace => {
                depth -= 1;
                if depth <= 0 {
                    return i + 1;
                }
            }
            Token::Semicolon if depth == 0 => return i + 1,
            Token::Eof => return i,
            _ => {}
        }
        i += 1;
    }
    i
}

/// `@contents;` 语句段消耗。
fn skip_to_semicolon(tokens: &[Token], at: usize) -> usize {
    let mut i = at;
    while let Some(t) = tokens.get(i) {
        match t {
            Token::Semicolon => return i + 1,
            Token::LBrace => return skip_extent(tokens, i),
            Token::Eof => return i,
            _ => i += 1,
        }
    }
    i
}

/// at 块收集：前奏（at keyword 起到 `{`）＋块体（配对间）＋下一扫描位。
fn collect_block(tokens: &[Token], at: usize) -> Option<(Span, Span, usize)> {
    match classify_member(tokens, at) {
        MemberKind::Block(pre, body, next) => Some((pre, body, next)),
        _ => None,
    }
}

/// `@mixin` 单规则 extent → canonical 产物。非 `@mixin`／头部非法／体缺失返
/// None（回调契约：空串 = 解析失败 → 写面 SyntaxError，k115 同约定）。
pub(crate) fn parse_mixin_rule(text: &str) -> Option<MixinRuleParts> {
    let trimmed = text.trim();
    // 体缺失补空块（insertRule `@mixin --m;` 残形容错——k115 同款）。
    let owned;
    let rule_text = if trimmed.ends_with('}') {
        trimmed
    } else {
        owned = format!("{trimmed} {{}}");
        owned.as_str()
    };
    let mut tokens: Vec<Token> = zero_css_parser::Tokenizer::new(rule_text).map(|s| s.token).collect();
    tokens.push(Token::Eof);
    let mut i = 0usize;
    while matches!(tokens.get(i), Some(Token::Whitespace) | Some(Token::Comment(_))) {
        i += 1;
    }
    let Token::AtKeyword(an) = tokens.get(i)? else {
        return None;
    };
    if !an.eq_ignore_ascii_case("mixin") {
        return None;
    }
    parse_mixin_at(&tokens, i).map(|(parts, _)| parts)
}

/// tokens 版 `@mixin` 单规则解析（`tokens[at]` = `@mixin` AtKeyword，尾部含
/// Eof 哨兵）→（产物，下一扫描位＝体闭 `}` 之后）。头部非法／体缺失返 None。
fn parse_mixin_at(tokens: &[Token], at: usize) -> Option<(MixinRuleParts, usize)> {
    let i = at + 1;
    let end = tokens.len() - 1; // Eof 位
    let Header { name, params_text } = parse_header(&tokens[i..end])?;
    // 体 = 首个 `{` 到配对 `}`
    let lb = i + tokens[i..end].iter().position(|t| matches!(t, Token::LBrace))?;
    let mut depth = 0usize;
    let mut body_end = None;
    for (k, t) in tokens.iter().enumerate().skip(lb + 1) {
        match t {
            Token::LBrace => depth += 1,
            Token::RBrace => {
                if depth == 0 {
                    body_end = Some(k);
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    let be = body_end?;
    let body = &tokens[lb + 1..be];

    let (lines, children) = scan_block(body);
    let css_text = render_mixin_css_text(&name, &params_text, &lines);
    Some((
        MixinRuleParts {
            name,
            params_text,
            css_text,
            children,
        },
        be + 1,
    ))
}

/// canonical cssText 组装：`@mixin <name>[ (<params>)] {\n<lines>\n}`（空参数
/// 列表/无参数列表剥 `()`——mixin-cssom 期望形）。
fn render_mixin_css_text(name: &str, params_text: &str, lines: &[String]) -> String {
    let mut css = format!("@mixin {}", name);
    if !params_text.is_empty() {
        css.push_str(&format!("({})", params_text));
    }
    css.push_str(" {\n");
    for l in lines {
        css.push_str(l);
        css.push('\n');
    }
    css.push('}');
    css
}

/// 顶层 `@mixin` 预扫（wire 臂单源——parse 输出 generic `Rule::At` 不保 raw 体
/// 形，canonical 从 raw tokens 重建）：按文档序产出各规则 canonical cssText。
/// 深度计过块体（嵌套位 `@mixin` 非 css-mixins-1 文法，不收）。
pub(crate) fn scan_top_level_mixins(text: &str) -> Vec<String> {
    let mut tokens: Vec<Token> = zero_css_parser::Tokenizer::new(text).map(|s| s.token).collect();
    tokens.push(Token::Eof);
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut i = 0usize;
    while i < tokens.len() {
        match &tokens[i] {
            Token::LBrace => {
                depth += 1;
                i += 1;
            }
            Token::RBrace => {
                depth -= 1;
                i += 1;
            }
            Token::AtKeyword(an) if depth == 0 && an.eq_ignore_ascii_case("mixin") => {
                match parse_mixin_at(&tokens, i) {
                    Some((parts, next)) => {
                        out.push(parts.css_text);
                        i = next;
                    }
                    None => {
                        i += 1;
                    }
                }
            }
            _ => i += 1,
        }
    }
    out
}

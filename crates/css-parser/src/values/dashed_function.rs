//! css-mixins-1 `<dashed-function>` 调用实参文法验证（写面拒收单源）。
//!
//! 标准属性值中的 dashed-function 调用（`--fn(…)`）实参文法
//!（<https://drafts.csswg.org/css-mixins-1/#function-rule>；保留形见
//! <https://github.com/w3c/csswg-drafts/issues/11749>）：
//! - 实参列表顶层逗号分隔；`--fn()`（及仅空白）为零实参合法形。
//! - 实参段顶层坏词元恒非法：`;`（Semicolon）、`!`（Delim('!')）、
//!   `}`（RBrace）、`]`（RBracket）——css-syntax 保留词元在调用实参位
//!   无文法名分（`!` 为 important 位、`;` 为声明界、`}`/`]` 失衡闭块）。
//! - 空实参非法：`--fn(,)` / `--fn(a,,b)` / `--fn(a, )` 等逗号分隔段
//!   须含至少一个非空白 component value。
//! - 块实参（`{…}`）须独占其实参段且块体非空：`--fn(123 {})` /
//!   `--fn({}1)` 块非独占非法；`--fn({})` / `--fn({ })` 空体非法；
//!   `--fn({,})` / `--fn({{}})` / `--fn({[]})` 块体有内容合法。
//! - 保留形（#11749「bad declaration」同形）：实参首 component 为
//!   dashed-ident 且（跨空白）次 component 为 `:`——未来命名实参语法
//!   保留，现文法拒收。`--fn(50px --myident:)` 合法（dashed-ident 非实参
//!   首）；块内豁免 `--fn({--myident:})` 合法。
//! - 嵌套 dashed-function 递归同文法（任意深度）；非 dashed 函数
//!   （var()/calc()/url()…）体按词元深度透明走行，其内部顶层逗号不切段。
//! - 词法期 Error 词元（bad-url 等）落实参段顶层即非法
//!   （`--fn(url(\<bad-escape>))`）。

use crate::tokenizer::{Spanned, Token, Tokenizer};

/// 判定声明值是否通过 dashed-function 调用实参文法。
///
/// 值不含 dashed-function 调用词元恒 `true`（无调用＝零漂移——门由调用方
/// 按「含调用词元即委托本函数」口径接入，检测在本函数内权威完成）；含
/// 调用时逐调用（含任意深度嵌套）校验实参列表。
pub fn dashed_function_value_valid(value: &str) -> bool {
    let mut tokens = Tokenizer::new(value).map(|s: Spanned| s.token);
    while let Some(t) = tokens.next() {
        if let Token::Function(name) = t {
            // Function 词元含配对 `(` 的消耗；dashed 名即调用文法域。
            if name.starts_with("--") && !validate_call_args(&mut tokens) {
                return false;
            }
        }
    }
    true
}

/// 实参段首/次 component 记型（保留形 #11749 判定用）。
#[derive(Clone, Copy, PartialEq)]
enum CompKind {
    /// dashed-ident 词元（`--x`，Ident 词元且 `--` 前缀）。
    DashedIdent,
    /// `:` 词元。
    Colon,
    /// 其余 component（含嵌套调用/块——调用词元整体记一 component）。
    Other,
}

/// 校验一个 dashed-function 调用的实参列表。调用点位于 Function 词元已被
/// 取走之后；返回时已消耗至匹配 `)`（含）。提前遇 EOF＝未闭合，判非法。
fn validate_call_args(tokens: &mut impl Iterator<Item = Token>) -> bool {
    // 段判定状态（逗号切分；顶层＝调用实参位 depth 0）。
    let mut any_component = false; // 全调用出现过非空白顶层 component
    let mut any_comma = false;
    let mut comp_count = 0; // 当前段非空白 component 数（块独占判定）
    let mut first_comp: Option<CompKind> = None;
    let mut second_comp: Option<CompKind> = None;
    let mut has_block = false; // 当前段含 `{…}` 块
    let mut block_nonempty = false; // 块体（外层直接子级）有非空白词元
    let mut bad = false; // 顶层坏词元/Error/保留形
    let mut depth = 0i32; // 嵌套深度（Function/LParen/LBracket/LBrace +1）

    loop {
        let t = match tokens.next() {
            Some(t) => t,
            None => return false, // 未闭合调用（EOF 先于匹配 `)`）
        };
        // 块体非空标记：首个块的外层直接子级（pre-match depth==1）非空白
        // 词元即内容（`{{}}` 的内层 LBrace 亦计；`{ }` 仅空白为空体）。
        // 闭合词元不计：RBrace＝本块收尾（`{}` 立即闭合形不得自证非空）、
        // RParen＝块内闭调用失衡形（该路径另有 bad/独占门）。
        if has_block && depth == 1 && !matches!(t, Token::Whitespace | Token::RBrace | Token::RParen) {
            block_nonempty = true;
        }
        // 段收尾共用判定（逗号界与右括号终界同门）：空段/块非独占/块空体/
        // 保留形（首 dashed-ident 次 `:`）。
        let segment_bad = comp_count == 0
            || (has_block && comp_count != 1)
            || (has_block && !block_nonempty)
            || (first_comp == Some(CompKind::DashedIdent) && second_comp == Some(CompKind::Colon));
        match t {
            Token::RParen if depth == 0 => {
                // 实参列表结束；零实参形＝无 component 无逗号（segment_bad
                // 的 comp_count==0 门被 any_comma||any_component 前置豁免）。
                if (any_comma || any_component) && segment_bad {
                    return false;
                }
                return !bad;
            }
            Token::Comma if depth == 0 => {
                if segment_bad {
                    return false;
                }
                any_comma = true;
                comp_count = 0;
                first_comp = None;
                second_comp = None;
                has_block = false;
                block_nonempty = false;
            }
            Token::Whitespace if depth == 0 => {}
            Token::Function(name) => {
                // 嵌套调用：dashed 名递归同文法（其词元整段被递归消耗，
                // 本层深度不计数）；非 dashed 名透明走行（depth +1）。
                let dashed = name.starts_with("--");
                if depth == 0 {
                    comp_count += 1;
                    any_component = true;
                    record_comp(&mut first_comp, &mut second_comp, CompKind::Other);
                }
                if dashed {
                    if !validate_call_args(tokens) {
                        bad = true;
                    }
                } else {
                    depth += 1;
                }
            }
            Token::LParen | Token::LBracket => {
                if depth == 0 {
                    comp_count += 1;
                    any_component = true;
                    record_comp(&mut first_comp, &mut second_comp, CompKind::Other);
                }
                depth += 1;
            }
            Token::LBrace => {
                if depth == 0 {
                    comp_count += 1;
                    any_component = true;
                    record_comp(&mut first_comp, &mut second_comp, CompKind::Other);
                    has_block = true;
                }
                depth += 1;
            }
            Token::RBrace | Token::RBracket => {
                if depth == 0 {
                    // 失衡闭块词元落实参顶层：`--fn(})` / `--fn(])`。
                    bad = true;
                } else {
                    depth -= 1;
                }
            }
            Token::RParen => depth -= 1,
            Token::Semicolon if depth == 0 => bad = true,
            Token::Delim('!') if depth == 0 => bad = true,
            Token::Error(_) if depth == 0 => bad = true,
            Token::Colon if depth == 0 => {
                comp_count += 1;
                any_component = true;
                record_comp(&mut first_comp, &mut second_comp, CompKind::Colon);
            }
            _ => {
                if depth == 0 {
                    comp_count += 1;
                    any_component = true;
                    let kind = match &t {
                        Token::Ident(name) if name.starts_with("--") => CompKind::DashedIdent,
                        _ => CompKind::Other,
                    };
                    record_comp(&mut first_comp, &mut second_comp, kind);
                }
            }
        }
    }
}

/// 段首/次 component 记型（仅前两次；保留形判定只需首二）。
fn record_comp(first: &mut Option<CompKind>, second: &mut Option<CompKind>, kind: CompKind) {
    if first.is_none() {
        *first = Some(kind);
    } else if second.is_none() {
        *second = Some(kind);
    }
}

//! k113：css-mixins-1 `@mixin` / `@apply` 解析期展开机制。
//!
//! [css-mixins-1](https://drafts.csswg.org/css-mixins-1/)。双程门下（
//! `parse_stylesheet` 仅当输入含 `@apply` 子串时启用），pass 1 把全表 `@mixin`
//! 定义收入 [`MixinRegistry`]（前向引用——`@apply` 可引用其后定义的 mixin，
//! driving: apply-within-mixin）；pass 2 编译期在嵌套 `@apply` 处展开：
//!
//! - `@result` 块解包（内容提升）；体级裸常规声明/裸样式规则丢弃
//!   （missing-result1/2——无 `@result` 的 mixin 不产出）。
//! - `@contents` 拼接：apply 块在场即用（空块也压制回退——contents-rule m9），
//!   否则 `@contents { fallback }` 回退块。
//! - 条件外壳（@media/@supports/@container/…）保留并递归（包 `@result` 解包、
//!   包局部改名——mixin-conditionals）；体级 `@layer` 丢弃（层指派语义 FIXME k114+）。
//! - 参数绑定：位置实参按序；`{...}` 整块解包；全 Delim 实参/缺省非法 → 整条跳过；
//!   args 多于参数 → 跳过；缺参无缺省不绑定不跳过（括号形 m2c／裸形 m6b——
//!   参数引用落元素级缺席 → IACVT→unset 继承／var() 回退兑现）。
//! - 卫生改名：每实例 `--zwmx{n}{orig}`；局部（体级 `--*` 声明）与参数入实例帧，
//!   引用改名仅 `var()`/`env()` 首实参 Ident，帧链尾→首查找、未命中回落原名
//!   （元素级）——调用点求值（m13）与外层参数内层可见（m10）同机制。
//! - 调用环：名字栈 skip-in-progress（mixin-cycle `--last-processed: c`）。
//!
//! 未命中/跳过路径的 AST 输出与旧 generic/Statement 路径逐字节一致
//!（mixin-parsing CSSOM 红线）；无 `@apply` 的样式表单程逐位不变。

use super::Parser;
use crate::ast::{Rule, Selector};
use crate::tokenizer::{Token, Tokenizer};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

/// `@mixin` 定义（css-mixins-1 §mixin-rule）。body 以原始 token 序列保留，
/// 展开时按调用实例做结构变换（见模块文档）。
#[derive(Clone)]
pub(super) struct MixinDef {
    /// 参数表（位置参数）；`type()` 注解非法的参数 `type_ok = false`。
    pub(super) params: Vec<MixinParam>,
    /// 体原始 tokens（`{`/`}` 之间，不含两端）。
    pub(super) body: Vec<Token>,
    /// k116 跨表聚合源序（文档序表位次；单表路径恒 0）。同名多定义裁决键：
    /// css-mixins-1 最后定义胜——晚序聚合定义不被早序表收集覆盖；同序保持
    /// insert 后者胜（同表源序/单表既有语义不变）。
    pub(super) order: u32,
    /// k117 层序 rank（css-cascade-5 §cascading-utilities：分层定义按级联层
    /// 序裁决，未分层（unlayered）定义强于所有层——[`Self::UNLAYERED`]）。
    /// 层内定义 rank = 层首次声明/出现序（`@layer a, b;` 声明序定层序）。
    pub(super) layer: u32,
}

impl MixinDef {
    /// 未分层定义 rank（css-cascade-5：unlayered styles win over layered）。
    pub(super) const UNLAYERED: u32 = u32::MAX;
}

/// 参数项（css-mixins-1 §mixin-args）：`--name [type(<syntax>)] [: default]`。
#[derive(Clone)]
pub(super) struct MixinParam {
    /// 参数名（dashed-ident，含前导双杠）。
    pub(super) name: String,
    /// `type()` 注解语法合法性；false = 该参数视作未绑定（引用走元素级缺席 →
    /// IACVT），mixin 本体仍展开——m9「refuses illegal syntax parameters, but
    /// does not fail entire mixin」。FIXME(k114+)：type 语义域（m8/m15——typed
    /// 强制/typed 缺省元素级解析）。
    pub(super) type_ok: bool,
    /// 缺省值原始 tokens（`:` 后，剥边 ws）；无缺省 = None。
    pub(super) default: Option<Vec<Token>>,
}

/// 一次展开实例的改名帧：原参数/局部名 → 实例唯一名（`--zwmx{n}{orig}`）。
#[derive(Clone)]
pub(super) struct MixinFrame {
    pub(super) map: HashMap<String, String>,
}

/// 样式表级 `@mixin` 注册表＋展开簿记。k116 起注册表可携全文档聚合（跨样式表/
/// 导入域 `@apply` 可见性——engine collect 站按文档序聚合 [`MixinDefs`] 后逐表
/// 携注册 parse）。k117 起聚合裁决层序感知（`MixinDef.layer`——css-cascade-5
/// 强层胜、unlayered 最强，mixin-layers e3）。FIXME(k116+)：shadow DOM 域
///（mixin-shadow-dom——shadow 树级联面缺位前置）。
pub(super) struct MixinRegistry {
    pub(super) defs: HashMap<String, MixinDef>,
    /// 展开实例序号（卫生改名前缀，每表从 0 起）。
    pub(super) seq: Cell<usize>,
    /// 调用环栈：展开中的 mixin 名（skip-in-progress）。
    pub(super) stack: RefCell<Vec<String>>,
}

/// 编译期 `@mixin` 上下文：注册表＋当前帧链。None = 无展开（单程 / 未过双程门）。
pub(super) type Mx<'a> = Option<(&'a MixinRegistry, &'a [MixinFrame])>;

/// 体变换上下文。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    /// mixin 体级（参数/局部作用域）：`--*` 声明=局部（卫生改名保留），裸常规
    /// 声明与裸样式规则丢弃，`@result` 解包。
    Body,
    /// `@result` 内容级：声明（原名）与样式规则保留，嵌套 `@result` 丢弃。
    Result,
}

impl<'a> Parser<'a> {
    /// 顶层/块体 `@mixin` 注册表收集（`consume_one_rule` 臂）：span = 整条 extent
    /// token 切片（含 AtKeyword）。仅双程 pass 1（`collect_mixins = Some`）有实效。
    /// 头部非法（名非 dashed-ident / 参数项非 dashed / 参数表尾 junk）不注册——
    /// 早定义保留（mixin-basic invalid-name、mixin-parameters junk 判例）。
    pub(super) fn collect_mixin_from_extent(&mut self, span: &[Token]) {
        if self.collect_mixins.is_none() {
            return;
        }
        let Some(lb) = span.iter().position(|t| matches!(t, Token::LBrace)) else {
            return;
        };
        // 块体 = `{` 与配对 `}` 之间（不含两端）；未闭合（EOF）不注册。
        let mut depth = 0usize;
        let mut body_end = None;
        for (k, t) in span.iter().enumerate().skip(lb + 1) {
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
        let Some(be) = body_end else { return };
        if let Some((name, params)) = parse_mixin_header(&span[1..lb]) {
            if let Some(map) = self.collect_mixins.as_mut() {
                register_ordered(
                    map,
                    name,
                    params,
                    span[lb + 1..be].to_vec(),
                    self.mixin_order,
                    self.collect_layer,
                );
            }
        }
    }

    /// 嵌套 `@mixin`（样式规则体/条件体内）收集入口：prelude 串重 token 化
    ///（consume_nested_at_rule 已有 prelude 串 + body 原始 tokens）。
    pub(super) fn collect_mixin_nested(&mut self, prelude: &str, body: &[Token]) {
        if self.collect_mixins.is_none() {
            return;
        }
        let ptoks: Vec<Token> = Tokenizer::new(prelude).map(|s| s.token).collect();
        if let Some((name, params)) = parse_mixin_header(&ptoks) {
            if let Some(map) = self.collect_mixins.as_mut() {
                register_ordered(map, name, params, body.to_vec(), self.mixin_order, self.collect_layer);
            }
        }
    }
}

/// 带序注册（k116 跨表聚合裁决）：同名晚序定义不被早序收集覆盖——css-mixins-1
/// 同名多定义最后定义胜（document order），引擎按文档序逐表收集/解析各表并携
/// 全文档聚合注册表；同序（同表源序、单表既有路径）保持 insert 后者胜。
fn register_ordered(
    map: &mut HashMap<String, MixinDef>,
    name: String,
    params: Vec<MixinParam>,
    body: Vec<Token>,
    order: u32,
    layer: u32,
) {
    if let Some(existing) = map.get(&name) {
        // k117 层序优先：已注册定义处于更强层（rank 更大）则不被弱层收集覆盖
        //（css-cascade-5 层序先于文档序）；unlayered（MAX）强于所有层。
        if existing.layer > layer {
            return;
        }
        // 同层：晚序胜（跨表聚合源序——css-mixins-1 document order 最后定义
        // 胜）；同序（同表源序、单表既有路径）保持 insert 后者胜。
        if existing.layer == layer && existing.order > order {
            return;
        }
    }
    map.insert(
        name,
        MixinDef {
            params,
            body,
            order,
            layer,
        },
    );
}

/// k116：文档级 `@mixin` 定义聚合载体（跨 crate opaque——engine collect 站按
/// 文档序对各表文本 [`MixinDefs::collect`] 聚合（同名晚序胜），再逐表携聚合
/// [`Parser::parse_stylesheet_with_mixins`]——`@apply` 跨样式表/导入域可见
///（https://drafts.csswg.org/css-mixins-1/#using-mixins）。
#[derive(Default, Clone)]
pub struct MixinDefs {
    pub(super) inner: HashMap<String, MixinDef>,
    /// k117 跨表级联层序登记表（css-cascade-5：层按首次声明序排序，跨样式表
    /// 累积——`@layer a, b;` 声明与 `@layer x { }` 块首现皆登记；collect 逐表
    /// 传入 pass-1，登记结果回写累积）。
    pub(super) layer_seq: Vec<String>,
}

impl MixinDefs {
    /// 空聚合（单表路径语义）。
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            layer_seq: Vec::new(),
        }
    }

    /// 文档序收集一张表的 `@mixin` 定义进聚合（`order` = 表文档序位次）。
    /// `input` 无 `@mixin` 子串时零成本返回（多数表 bail——聚合只对含定义的表
    /// 付 pass-1 成本）。晚序聚合同名定义不被早序收集覆盖（`register_ordered`
    /// 裁决——聚合复用 pass-1 收集循环，预填即现有聚合）；k117 层序表跨表
    /// 累积（`@layer` 声明/块首现按序登记）。
    pub fn collect(&mut self, order: u32, input: &str) {
        // k117：`@layer a, b;` 声明可出现在无 `@mixin` 的表——层序表跨表累积
        // 需要该表也过 pass-1 登记（纯定义聚合门放宽为定义或层声明门）。
        if !input.contains("@mixin") && !input.contains("@layer") {
            return;
        }
        let mut tokens: Vec<Token> = Tokenizer::new(input).map(|s| s.token).collect();
        tokens.push(Token::Eof);
        let prefill = std::mem::take(&mut self.inner);
        let layer_prefill = std::mem::take(&mut self.layer_seq);
        let (defs, layer_seq) = super::Parser::run_mixin_collect_pass(&mut tokens, prefill, order, layer_prefill);
        self.inner = defs;
        self.layer_seq = layer_seq;
    }

    /// 聚合是否为空（engine 侧：空聚合逐表回落单程 parse——无 `@mixin` 文档
    /// 逐位既有路径）。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// `@apply` 展开站（`compile_nested_at_rules` 调用）。`parent` = 应用侧自身选择器。
/// 返回 None = 名未命中 / 实参畸形跳过（调用方回落旧路径逐字节形）；
/// 环跳过返回 `Some(vec![])`（展开无产出）。
pub(super) fn expand_apply(
    registry: &MixinRegistry,
    frames: &[MixinFrame],
    prelude: &str,
    contents: Option<&[Token]>,
    parent: &[Selector],
) -> Option<Vec<Rule>> {
    let (name, args) = parse_apply_prelude(prelude)?;
    let def = registry.defs.get(&name)?;
    // 调用环（css-mixins-1 循环引用 → 跳过；skip-in-progress——mixin-cycle
    // a→b→c→(b 在展开中)→跳过，`--last-processed: c` 兑现）。
    if registry.stack.borrow().iter().any(|n| n == &name) {
        return Some(Vec::new());
    }
    // 参数绑定（css-mixins-1 §mixin-args
    // <https://drafts.csswg.org/css-mixins-1/#mixin-arguments>）：位置实参按序；
    // args 多于参数 → 整条跳过（m5）；缺参无缺省不绑定不跳过——括号形 m2c
    //（"Too few parameters and no default ignores mixin" 判据 ground truth＝
    // 展开继续、参数化声明元素级 var 缺席 → IACVT→unset 继承黑）；裸形 m6b
    //（var() 回退兑现 navy）。
    let nparams = def.params.len();
    if let Some(args) = &args {
        if args.len() > nparams {
            return None;
        }
    }
    let mut bindings: Vec<(String, Vec<Token>)> = Vec::new();
    for (i, p) in def.params.iter().enumerate() {
        let raw = match (args.as_ref().and_then(|a| a.get(i)), p.default.as_ref()) {
            (Some(a), _) => clean_arg(a)?,    // 非法实参 → 整条跳过（m17 !!!）
            (None, Some(d)) => clean_arg(d)?, // 缺省非法 → 整条跳过（m16 !!）
            (None, None) => continue,         // 缺参无缺省 → 不绑定（括号形 m2c／裸形 m6b——引用落元素级缺席）
        };
        if !p.type_ok {
            continue; // type() 非法 → 不绑定（m9——引用落元素级缺席 → IACVT）
        }
        bindings.push((p.name.clone(), raw));
    }
    // 实例帧：参数绑定先入（同名局部被参数遮蔽——entry 不覆盖）；局部随体变换补入。
    let seq = registry.seq.get();
    registry.seq.set(seq + 1);
    let mut child = MixinFrame { map: HashMap::new() };
    let mut renamed: Vec<String> = Vec::with_capacity(bindings.len());
    for (orig, _) in &bindings {
        let new = format!("--zwmx{}{}", seq, orig);
        child.map.insert(orig.clone(), new.clone());
        renamed.push(new);
    }
    let mut frames_child: Vec<MixinFrame> = frames.to_vec();
    frames_child.push(child);

    // 参数绑定声明置于展开片段首位（css-mixins-1 §mixin-args）；实参值按调用方帧
    // 改名（调用点求值——m13 链：外层 var(--outer) 以外层帧改名后再内层绑定）。
    let mut toks: Vec<Token> = Vec::new();
    for ((_, raw), new_name) in bindings.iter().zip(&renamed) {
        let mut val = raw.clone();
        rename_var_env_refs(&mut val, frames);
        toks.push(Token::Ident(new_name.clone()));
        toks.push(Token::Colon);
        toks.extend(val);
        toks.push(Token::Semicolon);
    }
    // 结构变换：@result 解包 / @contents 拼接 / 条件外壳递归 / 局部卫生改名。
    transform_mixin_body(&def.body, Ctx::Body, contents, seq, &mut frames_child, &mut toks);
    // 全量 var()/env() 引用改名（子帧链；已改名引用非帧键 → 恒等回落，幂等安全）。
    rename_var_env_refs(&mut toks, &frames_child);

    // 环栈压栈 → 子编译（嵌套 @apply 经展开站递归）→ 弹栈。
    registry.stack.borrow_mut().push(name);
    let rules = Parser::compile_at_body(toks, Some(parent), Some((registry, &frames_child)));
    registry.stack.borrow_mut().pop();
    Some(rules)
}

/// `@mixin` 头解析：`--name [参数表]`。参数表形 `--name(...)` 经 Function token
///（dashed-ident 后跟 `(` 被 css-syntax 词法化为 Function）或裸形 Ident。返回
/// None = 整条定义无效（不注册，早定义保留）。
fn parse_mixin_header(prelude: &[Token]) -> Option<(String, Vec<MixinParam>)> {
    let toks = strip_ws(prelude);
    let (name, items_toks) = match toks.first()? {
        Token::Ident(n) if n.starts_with("--") => (n.clone(), strip_ws(&toks[1..])),
        Token::Function(n) if n.starts_with("--") => {
            // 参数表形：截到配对 `)`（末位即配对——prelude 不含 at-rule 后续 tokens，
            // 保险起见仍做配对扫描；未闭合 = 定义无效）。
            let mut depth = 1usize;
            let mut end = toks.len();
            for (k, t) in toks.iter().enumerate().skip(1) {
                match t {
                    Token::Function(_) | Token::LParen | Token::LBracket => depth += 1,
                    Token::RParen | Token::RBracket => {
                        depth -= 1;
                        if depth == 0 {
                            end = k;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if depth != 0 {
                return None;
            }
            (n.clone(), strip_ws(&toks[1..end]))
        }
        _ => return None,
    };
    let mut params = Vec::new();
    if !items_toks.is_empty() {
        for item in split_top_level_commas_tok(items_toks) {
            let item = strip_ws(&item);
            let Token::Ident(pname) = item.first()? else {
                return None;
            };
            if !pname.starts_with("--") {
                return None; // junk 参数项（junk-in-parameter-list）→ 整条无效
            }
            let mut j = 1;
            while matches!(item.get(j), Some(Token::Whitespace | Token::Comment(_))) {
                j += 1;
            }
            let mut type_ok = true;
            if let Some(Token::Function(f)) = item.get(j) {
                if f.eq_ignore_ascii_case("type") {
                    // type(<syntax>)：截到配对 `)`，内容首 token `Delim('<')` 且末
                    // token `Delim('>')` 才合法（type(length) 等非法 → 参数不绑定，
                    // 不毒化整条——m9）。
                    let mut depth = 1usize;
                    let mut k = j + 1;
                    let mut first_content: Option<&Token> = None;
                    let mut last_content: Option<Token> = None;
                    while k < item.len() {
                        match &item[k] {
                            Token::Function(_) | Token::LParen => depth += 1,
                            Token::RParen => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            Token::Whitespace | Token::Comment(_) => {}
                            t => {
                                if first_content.is_none() {
                                    first_content = Some(t);
                                }
                                last_content = Some(t.clone());
                            }
                        }
                        k += 1;
                    }
                    if depth != 0 {
                        return None; // 未闭合 type( → 参数表畸形 → 整条无效
                    }
                    type_ok = matches!(first_content, Some(Token::Delim('<')))
                        && matches!(last_content, Some(Token::Delim('>')));
                    j = k + 1;
                }
            }
            let after = strip_ws(&item[j..]);
            let default = match after.first() {
                None => None,
                Some(Token::Colon) => {
                    let d = strip_ws(&after[1..]);
                    if d.is_empty() {
                        return None; // `--p:` 空缺省 = 参数表畸形
                    }
                    Some(d.to_vec())
                }
                Some(_) => return None, // 注解/缺省后 junk → 整条无效
            };
            params.push(MixinParam {
                name: pname.clone(),
                type_ok,
                default,
            });
        }
    }
    Some((name, params))
}

/// `@apply` prelude 解析：`--name`（裸形）或 `--name(args)`（Function token 形）。
/// 返回 (名, Some(实参列表)=括号形 / None=裸形)。名非 dashed-ident / 裸形带尾随 /
/// 括号未闭合 → None（走未命中路径 = 旧行为）。
fn parse_apply_prelude(prelude: &str) -> Option<(String, Option<Vec<Vec<Token>>>)> {
    let raw: Vec<Token> = Tokenizer::new(prelude).map(|s| s.token).collect();
    let toks = strip_ws(&raw);
    match toks.first()? {
        Token::Ident(n) if n.starts_with("--") => {
            if strip_ws(&toks[1..]).is_empty() {
                Some((n.clone(), None))
            } else {
                None
            }
        }
        Token::Function(n) if n.starts_with("--") => {
            let mut depth = 1usize;
            let mut end = toks.len();
            for (k, t) in toks.iter().enumerate().skip(1) {
                match t {
                    Token::Function(_) | Token::LParen | Token::LBracket => depth += 1,
                    Token::RParen | Token::RBracket => {
                        depth -= 1;
                        if depth == 0 {
                            end = k;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if depth != 0 {
                return None;
            }
            Some((n.clone(), Some(split_top_level_commas_tok(strip_ws(&toks[1..end])))))
        }
        _ => None,
    }
}

/// 实参/缺省值清洗：剥边 ws、整体单块 `{...}` 解包（m3 `{green}`）、全 Delim 判
/// 非法（m16 `!!` / m17 `!!!`——css-mixins-1 §mixin-args invalid argument）。
/// None = 非法（整条展开跳过）。
fn clean_arg(arg: &[Token]) -> Option<Vec<Token>> {
    let a = strip_ws(arg);
    if let (Some(Token::LBrace), Some(Token::RBrace)) = (a.first(), a.last()) {
        // 仅整体单块才解包（内部深度首尾配对平衡）；解包后递归再验。
        let mut depth = 0usize;
        let mut balanced = true;
        for t in &a[1..a.len() - 1] {
            match t {
                Token::LBrace => depth += 1,
                Token::RBrace => {
                    if depth == 0 {
                        balanced = false;
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        if balanced {
            return clean_arg(&a[1..a.len() - 1]);
        }
    }
    if a.is_empty() || a.iter().all(|t| matches!(t, Token::Delim(_))) {
        return None;
    }
    Some(a.to_vec())
}

/// mixin 体结构变换（token 级游标）：
/// - Body ctx：`--*` 声明=局部（卫生改名保留）；裸常规声明/裸样式规则丢弃
///   （mixin-basic missing-result1/2）；`@result` 解包（切 Result ctx）；体级
///   `@layer` 丢弃；`@contents` 拼接；`@apply` 原样保留（子编译重捕获后按当前
///   帧链展开——apply-within-mixin / m10 / mixin-cycle）；其余块形 @规则外壳
///   保留＋体递归（@media/@supports/@container 包 @result/局部——mixin-conditionals）。
/// - Result ctx：声明（原名）与样式规则保留；嵌套 `@result` 丢弃；`@contents`
///   拼接；`@apply` 原样保留；条件外壳保留＋体递归。
///
/// var()/env() 引用改名由调用方全量统一做（见 [`rename_var_env_refs`]）。
fn transform_mixin_body(
    body: &[Token],
    ctx: Ctx,
    contents: Option<&[Token]>,
    seq: usize,
    frames: &mut Vec<MixinFrame>,
    out: &mut Vec<Token>,
) {
    let mut i = 0usize;
    while i < body.len() {
        match &body[i] {
            Token::Whitespace | Token::Comment(_) | Token::Semicolon => i += 1,
            Token::AtKeyword(name) => {
                let (prelude_end, block) = scan_at_extent(body, i);
                let end = extent_end(prelude_end, block);
                match name.to_ascii_lowercase().as_str() {
                    // @apply 原样保留（含块形 contents）——展开站按当前帧链处理。
                    "apply" => out.extend(body[i..end].iter().cloned()),
                    "contents" => match contents {
                        // apply 块在场即用（空块也压制回退——contents-rule m9）。
                        Some(c) => out.extend(c.iter().cloned()),
                        None => {
                            if let Some((bs, be)) = block {
                                out.extend(body[bs..be].iter().cloned());
                            }
                        }
                    },
                    // @result 解包（仅体级；Result 内嵌套 @result 丢弃）。
                    "result" if ctx == Ctx::Body => {
                        if let Some((bs, be)) = block {
                            transform_mixin_body(&body[bs..be], Ctx::Result, contents, seq, frames, out);
                        }
                    }
                    // 体级 @layer 丢弃。FIXME(k114+)：@result 内 @layer 指派语义域。
                    "layer" => {}
                    _ => {
                        // 条件/未知块形：外壳保留＋体递归（ctx 语义保持——Body 内
                        // 条件包局部仍改名，mixin-conditionals m3/m4）；语句形丢弃。
                        if let Some((bs, be)) = block {
                            out.push(Token::AtKeyword(name.clone()));
                            out.extend(body[i + 1..prelude_end].iter().cloned());
                            out.push(Token::LBrace);
                            transform_mixin_body(&body[bs..be], ctx, contents, seq, frames, out);
                            out.push(Token::RBrace);
                        }
                    }
                }
                i = end;
            }
            _ => {
                let (content_end, next, block_lb) = scan_decl_or_rule(body, i);
                if let Some(lb) = block_lb {
                    // 样式规则：Result 保留（选择器原样 + 体递归——体内 @contents 拼接、
                    // @apply 原样留给子编译重捕获，contents-rule m3 `@result { &.a {
                    // @contents … } }`）/ Body 丢弃（missing-result1）。
                    if ctx == Ctx::Result {
                        out.extend(body[i..lb].iter().cloned());
                        out.push(Token::LBrace);
                        transform_mixin_body(&body[lb + 1..content_end], Ctx::Result, contents, seq, frames, out);
                        out.push(Token::RBrace);
                    }
                } else {
                    let seg = &body[i..content_end];
                    let starts_custom = matches!(seg.first(), Some(Token::Ident(p)) if p.starts_with("--"));
                    match (ctx, starts_custom) {
                        // Result：原名保留（m-cycle `--last-processed`、m1 条件
                        // `--custom-property`——元素级原名可观察）。
                        (Ctx::Result, _) => {
                            out.extend(seg.iter().cloned());
                            out.push(Token::Semicolon);
                        }
                        // Body 局部：卫生改名保留（mixin-locals 全域）。
                        (Ctx::Body, true) => {
                            if let Some(Token::Ident(p)) = seg.first() {
                                out.push(Token::Ident(lookup_or_insert_local(frames, p, seq)));
                                out.extend(seg[1..].iter().cloned());
                                out.push(Token::Semicolon);
                            }
                        }
                        // Body 裸常规声明丢弃（mixin-locals m1 `font-size: 200px`）。
                        (Ctx::Body, false) => {}
                    }
                }
                i = next;
            }
        }
    }
}

/// var()/env() 首实参 Ident 改名（帧链尾→首查找，未命中回落原名＝元素级）。
/// 线性游标各自命中嵌套 var()；仅函数实参位，属性名位置不涉。
fn rename_var_env_refs(toks: &mut [Token], frames: &[MixinFrame]) {
    let mut i = 0usize;
    while i < toks.len() {
        let is_ref = matches!(&toks[i], Token::Function(f)
            if f.eq_ignore_ascii_case("var") || f.eq_ignore_ascii_case("env"));
        if is_ref {
            let mut j = i + 1;
            while matches!(toks.get(j), Some(Token::Whitespace | Token::Comment(_))) {
                j += 1;
            }
            if let Some(Token::Ident(name)) = toks.get(j) {
                let name = name.clone();
                if name.starts_with("--") {
                    let renamed = lookup_rename(frames, &name);
                    if renamed != name {
                        toks[j] = Token::Ident(renamed);
                    }
                }
            }
        }
        i += 1;
    }
}

/// 帧链引用改名：尾→首（最近展开实例优先），未命中回落原名。
fn lookup_rename(frames: &[MixinFrame], name: &str) -> String {
    for f in frames.iter().rev() {
        if let Some(n) = f.map.get(name) {
            return n.clone();
        }
    }
    name.to_string()
}

/// 局部卫生名：链尾帧 entry——同名参数已绑定则用参数名（参数遮蔽局部），否则
/// 插入 `--zwmx{seq}{orig}` 新名。
fn lookup_or_insert_local(frames: &mut [MixinFrame], orig: &str, seq: usize) -> String {
    let frame = frames.last_mut().expect("mixin expansion child frame");
    if let Some(n) = frame.map.get(orig) {
        return n.clone();
    }
    let new = format!("--zwmx{}{}", seq, orig);
    frame.map.insert(orig.to_string(), new.clone());
    new
}

/// @规则 extent 扫描（游标 i 在 AtKeyword 上）：返回 (prelude 终点, Option<块体
///（含首尾不含）>)。语句形在顶层 `;`/`}` 处止（`}` 不消耗）；未闭合块 → None。
fn scan_at_extent(body: &[Token], i: usize) -> (usize, Option<(usize, usize)>) {
    let mut depth = 0usize;
    let mut j = i + 1;
    while j < body.len() {
        match &body[j] {
            Token::Function(_) | Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth = depth.saturating_sub(1),
            Token::Semicolon | Token::RBrace if depth == 0 => return (j, None),
            Token::LBrace if depth == 0 => {
                let mut d = 0usize;
                let mut k = j + 1;
                while k < body.len() {
                    match &body[k] {
                        Token::LBrace => d += 1,
                        Token::RBrace => {
                            if d == 0 {
                                return (j, Some((j + 1, k)));
                            }
                            d -= 1;
                        }
                        _ => {}
                    }
                    k += 1;
                }
                return (j, None);
            }
            _ => {}
        }
        j += 1;
    }
    (j, None)
}

/// extent 终点：块形 = 块体终点 + 1（越过配对 `}`）；语句形 = prelude 终点 + 1
///（越过 `;`；`}` 止形不消耗终点——与 scan_at_extent 配对）。
fn extent_end(prelude_end: usize, block: Option<(usize, usize)>) -> usize {
    match block {
        Some((_, be)) => be + 1,
        None => prelude_end + 1,
    }
}

/// 顶层项扫描（声明/样式规则区分）：首个 depth-0 `LBrace` → 样式规则；`;`/`}`/
/// EOF → 声明。返回 (内容终点 excl 终止符, 下一项起点, Some(`{` 位)=块形 / None=声明)。
fn scan_decl_or_rule(body: &[Token], i: usize) -> (usize, usize, Option<usize>) {
    let mut depth = 0usize;
    let mut j = i;
    while j < body.len() {
        match &body[j] {
            Token::Function(_) | Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth = depth.saturating_sub(1),
            Token::LBrace if depth == 0 => {
                let mut d = 0usize;
                let mut k = j + 1;
                while k < body.len() {
                    match &body[k] {
                        Token::LBrace => d += 1,
                        Token::RBrace => {
                            if d == 0 {
                                return (k, k + 1, Some(j));
                            }
                            d -= 1;
                        }
                        _ => {}
                    }
                    k += 1;
                }
                return (k, k, Some(j));
            }
            Token::Semicolon if depth == 0 => return (j, j + 1, None),
            Token::RBrace if depth == 0 => return (j, j, None),
            _ => {}
        }
        j += 1;
    }
    (j, j, None)
}

/// token 级顶层逗号切分（深度感知：Function/LParen/LBracket 计深）。
fn split_top_level_commas_tok(toks: &[Token]) -> Vec<Vec<Token>> {
    // 空输入 = 零实参（`--m()` 括号形无实参——seed 空组会变单个空实参，
    // clean_arg 判非法致 `--m14()` 缺省绑定整条跳空）。
    if toks.is_empty() {
        return Vec::new();
    }
    let mut out = vec![Vec::new()];
    let mut depth = 0usize;
    for t in toks {
        match t {
            Token::Function(_) | Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth = depth.saturating_sub(1),
            Token::Comma if depth == 0 => {
                out.push(Vec::new());
                continue;
            }
            _ => {}
        }
        out.last_mut().expect("seeded").push(t.clone());
    }
    out
}

/// 剥两端空白/注释。
fn strip_ws(toks: &[Token]) -> &[Token] {
    let mut s = toks;
    while matches!(s.first(), Some(Token::Whitespace | Token::Comment(_))) {
        s = &s[1..];
    }
    while matches!(s.last(), Some(Token::Whitespace | Token::Comment(_))) {
        s = &s[..s.len() - 1];
    }
    s
}

//! CSS 自定义函数（css-mixins-1 `@function` / `<dashed-function>`，k109 刀）。
//!
//! 三件：注册表收集（§2.1 顶层规则）、`--name(...)` 调用形扫描替换（§3）、
//! 函数体求值（§3.1 假设子元素注册语义 ＋ §2.3.1 shadowing）。函数内
//! random 键按 css-values-5 §9.4.1 ua-ident 捕获——键名拼接
//! `{fname}-{internal}` 后复用既有 [`substitute_random_values`]（零改
//! random.rs）。
//!
//! k119 会话作用域重构：调用方作用域穿线（§2.3.1 动态作用域——嵌套调用
//! 可见外层函数局部/参数的已解析值）、绑定期实参 var 代入＋块实参解包、
//! 局部三层视图（原始/会话/已解析）、CSS 宽关键字局部/缺省语义（§2.3.1）、
//! 求值失败 `Invalid`（消费声明 IACVT，不再字面外溢）。
//!
//! <https://drafts.csswg.org/css-mixins-1/>
//! <https://drafts.csswg.org/css-values-5/#random-evaluating>

use std::collections::{HashMap, HashSet};

use zero_css_parser::Stylesheet;
use zero_css_parser::ast::{FunctionBodyItem, FunctionCondKind, FunctionDecl, FunctionRule, Rule};
use zero_css_parser::parser::split_top_level_commas;
use zero_css_parser::values::{
    CalcContext, CalcExpr, RandomEvalContext, eval_calc_with_context, parse_angle_to_radians, parse_calc, parse_color,
    parse_length, substitute_random_values,
};

use crate::computed::{SCOPE_IACVT, resolve_env, resolve_env_and_var, substitute_var_text_attributed};
use crate::if_function::{self, IfEvalCtx, IfOutcome};

/// `<dashed-function>` 调用替换产物：`Value`＝替换文本；`Cycle`＝函数
/// 调用环（css-mixins-1 §3 invalid at substitution——调用链互指/自指）
/// → 消费声明整声明 IACVT（k112；k129 收窄：**产物重扫**环收口为本调用
/// `Invalid`——css-values-5 §cyclic-substitution-contexts 环不整链上抛，
/// 检出点在实参替换/局部展开/预检的仍整链传播）；`Invalid`＝调用求值失败
///（缺 result 描述符/必需实参缺席且无缺省——§3 调用 invalid at
/// substitution）→ 消费声明整声明 IACVT（k119：调用形不再保留原文——
/// dashed-function-eval「Missing result descriptor」「Missing only
/// argument」判例期望 guaranteed-invalid 空值）。未注册名仍保留原文
///（guaranteed-invalid 同向基线——`unregistered_name_kept_verbatim` 钉）。
#[derive(Debug)]
pub(crate) enum DashOutcome {
    /// 替换文本＋attr-security taint 标志（k126，css-values-5 §attr-security
    /// ——求值链内任一 attr() 替换产物携带 url → tainted，向上逐层合并，
    /// 顶层消费站对取 URL 属性裁决 IACVT）。
    Value(String, bool),
    Cycle,
    Invalid,
}

/// 求值深度上限（循环防护：自引用/互引用函数链 guaranteed-invalid，
/// css-mixins-1 §3 invalid at substitution——调用形保留原文交下游拒）。
const MAX_EVAL_DEPTH: usize = 32;

/// var()/attr() 站迭代替换收敛上限（k127）：attr() 属性值产物可含 var()
/// 引用、var() 回退值可含 attr()，单趟站序（k126 var→attr）覆盖不了
/// attr↔var 交替链——迭代至不动点；上限＝环保护，交替不收敛（attr 产物
/// var 回指被求值声明自身）按环处置 → guaranteed-invalid（规范 cyclic
/// substitution context 同向）。波数只随 var↔attr 交替计（var 链本身
/// 递归一趟全解），8 波余量充分。
const SUBSTITUTE_FIXPOINT_LIMIT: usize = 8;

/// 扫描样式表顶层与 `@layer` 块内 `@function` 规则，构建「函数名 → 规则」
/// 映射。
///
/// 同名裁决（css-mixins-1 §2.1.1「the rule in the stronger cascade layer
/// wins, and rules defined later win within the same layer」）：层 rank 大者
/// 胜、同 rank 后扫胜（扫描天然文档序——sheets 链序＋表内规则序，直接覆盖
/// 即后者胜；rank 取自 k128 [`FunctionRule::layer`]，与 k117 @mixin 同源
/// layer_seq 全局层序表）。`@layer` 嵌套块内层 rank 同源递归收集；@media
/// 等条件块不递归（@property 同型——全局注册无作用域）。
pub(crate) fn collect_custom_functions<'a>(
    stylesheets: impl IntoIterator<Item = &'a Stylesheet>,
) -> HashMap<String, FunctionRule> {
    let mut map: HashMap<String, FunctionRule> = HashMap::new();
    for ss in stylesheets {
        collect_functions_in_rules(&ss.rules, &mut map);
    }
    map
}

/// k128：规则列表内 `@function` 收集（`@layer` 块递归——层内定义按 rank
/// 参与同名裁决；其他组规则不递归）。
fn collect_functions_in_rules(rules: &[Rule], map: &mut HashMap<String, FunctionRule>) {
    for rule in rules {
        match rule {
            Rule::Function(fr) => {
                match map.get(&fr.name) {
                    // 强层不被弱层覆盖（同 rank 落到 else＝后扫胜）。
                    Some(existing) if existing.layer > fr.layer => {}
                    _ => {
                        map.insert(fr.name.clone(), fr.clone());
                    }
                }
            }
            Rule::Layer(lr) => collect_functions_in_rules(&lr.rules, map),
            _ => {}
        }
    }
}

/// k131 Face C（css-shadow-1 §css-tree-scoped-name，
/// https://drafts.csswg.org/css-shadow-1/#css-tree-scoped-reference）：`@function`
/// 查表链——内层（声明所在 shadow 树）表在前、链尾＝文档表。同名解析链首命中
/// （内层屏蔽外层）；命中位置同时给出被调函数的定义域子链（`tables[pos..]`）——
/// 函数体内嵌套调用按定义域链继续解析（外树声明不可见内树定义，封装边界同向）。
pub(crate) struct FnChain<'a> {
    tables: Vec<&'a HashMap<String, FunctionRule>>,
}

impl<'a> FnChain<'a> {
    /// 内部构造：表序内层→外层、链尾文档表（站侧 [`crate::StyleSystem`] 装填）。
    pub(crate) fn new(tables: Vec<&'a HashMap<String, FunctionRule>>) -> Self {
        Self { tables }
    }

    /// 单表链（Face C 前全局注册表形态——测试捷径与空 scope 链同构）。
    #[cfg(test)]
    pub(crate) fn doc(table: &'a HashMap<String, FunctionRule>) -> Self {
        Self { tables: vec![table] }
    }

    /// 同名链查：返回（规则，定义域子链）。定义域子链＝自命中表起向外——体内
    /// 嵌套调用在该子链上继续解析；全链未命中＝调用形保留原文（guaranteed-
    /// invalid 通道不变）。
    pub(crate) fn lookup(&self, name: &str) -> Option<(&'a FunctionRule, FnChain<'a>)> {
        for (pos, table) in self.tables.iter().enumerate() {
            if let Some(rule) = table.get(name) {
                return Some((
                    rule,
                    FnChain {
                        tables: self.tables[pos..].to_vec(),
                    },
                ));
            }
        }
        None
    }
}

/// 值文本是否含 `<dashed-function>` 调用形（`--ident(`，ident 边界感知）。
///
/// 站前 any() 预滤门（4.75 attr 站 has_attr_word 同型）；假阳（url 载荷、
/// 字符串字面量内形似体）由站内逐点查注册表天然兜住。
pub(crate) fn has_dashed_function_call(value: &str) -> bool {
    let chars: Vec<(usize, char)> = value.char_indices().collect();
    for k in 0..chars.len() {
        let (i, c) = chars[k];
        if c != '-' || !value[i..].starts_with("--") {
            continue;
        }
        // ident 边界：`a--f(` 是单 ident `a--f` 的函数形，非双杠名调用。
        if i > 0 && is_ident_char(chars[k - 1].1) {
            continue;
        }
        let mut j = k + 2;
        while j < chars.len() && is_ident_char(chars[j].1) {
            j += 1;
        }
        if j > k + 2 && j < chars.len() && chars[j].1 == '(' {
            return true;
        }
    }
    false
}

/// `<dashed-function>` 替换（css-mixins-1 §3）：扫描值文本中已注册
/// `--name(...)` 调用形，实参先于调用属性语境替换（§3 arbitrary
/// substitution 先行——嵌套函数递归＋random 按位求值），参数按位匹配
/// （尾逗号/纯空白实参移除，§3）后求值函数体，产物原位替换。
///
/// `registry` = 调用方定义域函数链（k131 Face C [`FnChain`]：顶层站＝按胜者
/// 声明 scope_root 装链，实参嵌套调用沿用调用方链；函数体内＝被调方定义域
/// 子链——链首命中即同名裁决，命中位置界定体内嵌套调用继续解析的子链）。
///
/// `scope` = 调用方会话作用域（k119 作用域穿线：顶层站＝元素计算表；
/// 函数体内嵌套调用＝被调方 caller 的已解析作用域——§2.3.1 局部/参数
/// 动态作用域对外层函数调用可见，dashed-function-eval「Substitute local
/// from outer scope」判例族）。
///
/// guaranteed-invalid（未注册名/深度超限）：该调用形保留原文——下游 apply
/// 对函数形语法拒（IACVT 等价；基线本就把函数形留作字面值，语义同向）。
/// 求值失败（缺 result/必需实参缺席）→ [`DashOutcome::Invalid`]——消费
/// 声明 IACVT（k119，§3 invalid at substitution）。调用环（栈上重入，
/// k112）：整链 [`DashOutcome::Cycle`]——消费声明 IACVT。字符串字面量/
/// 引号域内的形似体本层按文本扫描处理，FIXME 随 tokenizer 级替换刀收严。
#[allow(clippy::too_many_arguments)]
pub(crate) fn substitute_dashed_functions(
    value: &str,
    consuming_prop: &str,
    registry: &FnChain,
    scope: &HashMap<String, String>,
    element_seed: Option<u64>,
    font_base_px: Option<f64>,
    depth: usize,
    call_stack: &mut Vec<String>,
    attr_reader: &dyn Fn(&str) -> Option<String>,
    random_ctx: &RandomEvalContext,
    types: &HashMap<String, String>,
    cond: &CondEvalCtx,
) -> DashOutcome {
    if depth >= MAX_EVAL_DEPTH {
        return DashOutcome::Value(value.to_string(), false);
    }
    let chars: Vec<(usize, char)> = value.char_indices().collect();
    let mut out = String::with_capacity(value.len());
    // k126：taint 累积（css-values-5 §attr-security）——实参替换/函数求值
    // 端 attr() 产物 url 污染逐层 OR 合并，随 Value 通道向上传播。
    let mut taint_acc = false;
    let mut last = 0usize; // 未产段起点（字节偏移）
    let mut k = 0usize; // chars 游标
    while k < chars.len() {
        let (i, c) = chars[k];
        if c == '-' && value[i..].starts_with("--") && (i == 0 || !is_ident_char(chars[k - 1].1)) {
            let mut j = k + 2;
            while j < chars.len() && is_ident_char(chars[j].1) {
                j += 1;
            }
            if j > k + 2 && j < chars.len() && chars[j].1 == '(' {
                let name = &value[i..chars[j].0];
                if let Some((rule, def_site)) = registry.lookup(name) {
                    if let Some(cm) = matching_close_paren(&chars, j) {
                        // k112 调用环（css-mixins-1 §3 invalid at
                        // substitution）：栈上重入 → 整链 Cycle。
                        if call_stack.iter().any(|n| n == name) {
                            return DashOutcome::Cycle;
                        }
                        let inner = &value[chars[j].0 + 1..chars[cm].0];
                        // §3：实参先行替换——嵌套 dashed-function 递归（深度
                        // 护栏）＋random 调用属性语境求值（写面 fold 已烤键
                        // 原样保留，omitted 形本站补展开；键序数＝消费属性
                        // 文本序）。
                        let mut args = Vec::new();
                        let mut args_taint = false;
                        for a in split_top_level_commas(inner) {
                            let a = a.trim().to_string();
                            if a.is_empty() {
                                continue; // §3 纯空白实参移除
                            }
                            let nested = match substitute_dashed_functions(
                                &a,
                                consuming_prop,
                                registry,
                                scope,
                                element_seed,
                                font_base_px,
                                depth + 1,
                                call_stack,
                                attr_reader,
                                random_ctx,
                                types,
                                cond,
                            ) {
                                DashOutcome::Cycle => return DashOutcome::Cycle,
                                DashOutcome::Invalid => return DashOutcome::Invalid,
                                // k126：实参 taint 并入本次调用累积（实参文本
                                // 经参数绑定进函数体，污染随求值传播）。
                                DashOutcome::Value(v, t) => {
                                    args_taint |= t;
                                    taint_acc |= t;
                                    v
                                }
                            };
                            args.push(
                                substitute_random_values(&nested, consuming_prop, random_ctx)
                                    .map(std::borrow::Cow::into_owned)
                                    .unwrap_or(nested),
                            );
                        }
                        out.push_str(&value[last..i]);
                        call_stack.push(name.to_string());
                        let evaluated = evaluate_custom_function(
                            rule,
                            &args,
                            &def_site,
                            scope,
                            element_seed,
                            font_base_px,
                            depth + 1,
                            consuming_prop,
                            call_stack,
                            args_taint,
                            attr_reader,
                            random_ctx,
                            types,
                            cond,
                        );
                        let splice = match evaluated {
                            Ok(Some((product, eval_taint, inner_scope))) => {
                                taint_acc |= eval_taint;
                                // 产物可含 body 级嵌套调用（untyped result
                                // 引用他函数）——深度护栏内再扫；调用栈保持
                                // push 态（互调环归因——k112 T12 判据）；
                                // k119 作用域穿线：嵌套调用以被调方 caller 的
                                // 已解析作用域（inner_scope）为基座。
                                match substitute_dashed_functions(
                                    &product,
                                    consuming_prop,
                                    &def_site,
                                    &inner_scope,
                                    element_seed,
                                    font_base_px,
                                    depth + 1,
                                    call_stack,
                                    attr_reader,
                                    random_ctx,
                                    // k123a：产物嵌套调用以被调方帧表穿线
                                    //（值与型别同帧——per-frame 精确归属）。
                                    &frame_types_of(types, rule),
                                    cond,
                                ) {
                                    DashOutcome::Cycle => {
                                        // k129：产物重扫环收口＝本调用 invalid
                                        //（css-values-5
                                        // §cyclic-substitution-contexts——
                                        // cyclic substitution context 使所在
                                        // 声明 invalid，环不再整链上抛）——
                                        // 「Cycle through local, other function,
                                        // fallback in function」判据：内层调用
                                        // invalid → 外层局部按 Invalid 臂落遮蔽
                                        // 标记 → 消费侧 fallback 生效。元素级
                                        // 消费两语义同向（IACVT → fallback）。
                                        call_stack.pop();
                                        return DashOutcome::Invalid;
                                    }
                                    DashOutcome::Invalid => {
                                        call_stack.pop();
                                        return DashOutcome::Invalid;
                                    }
                                    // k126：产物嵌套替换的 taint 并入累积。
                                    DashOutcome::Value(v, t) => {
                                        taint_acc |= t;
                                        v
                                    }
                                }
                            }
                            // 求值失败（缺 result 描述符/必需实参缺席且无
                            // 缺省，k119）→ Invalid——消费声明 IACVT。
                            // Err = 会话内嵌套调用环（k112 T12 归因锚）→
                            // Cycle 整链传播（消费声明 IACVT 同向）。
                            Ok(None) => {
                                call_stack.pop();
                                return DashOutcome::Invalid;
                            }
                            Err(()) => {
                                call_stack.pop();
                                return DashOutcome::Cycle;
                            }
                        };
                        call_stack.pop();
                        out.push_str(&splice);
                        last = chars[cm].0 + 1;
                        k = cm + 1;
                        continue;
                    }
                }
            }
        }
        k += 1;
    }
    out.push_str(&value[last..]);
    DashOutcome::Value(out, taint_acc)
}

/// k124 条件判定语境（css-mixins-1 §conditional-rules 求值端）：@function 体
/// 条件组三臂判定环境，由调用站按元素上下文构造（4.755 站＝media_ctx_for()
/// ＋容器链快照）。`None` 臂按 false（规范：条件不可判定 → 组不生效）。
pub(crate) struct CondEvalCtx {
    /// 视口媒体语境（`@media` 臂）。
    pub media: Option<zero_css_parser::media_query::MediaContext>,
    /// 容器上下文（`@container` 臂——无名条件＝最近容器）。
    pub container: Option<crate::matcher::ContainerContext>,
}

/// k124：条件组展平（css-mixins-1 §conditional-rules）——条件满足的组按源
/// 序原地展开为声明序列，last-wins 语义与平面声明一致（§4 执行模型）。条件
/// 不满足的组整体跳过；嵌套组递归（外真内真才进）。
fn flatten_function_body(items: &[FunctionBodyItem], cond: &CondEvalCtx, out: &mut Vec<FunctionDecl>) {
    for item in items {
        match item {
            FunctionBodyItem::Decl(d) => out.push(d.clone()),
            FunctionBodyItem::Conditional(g) => {
                if function_cond_holds(&g.kind, cond) {
                    flatten_function_body(&g.children, cond, out);
                }
            }
        }
    }
}

/// 条件三臂判定（k124）：`@supports` → 静态声明/选择器支持判定；`@media` →
/// 视口语境（逗号列表任一为真＝OR）；`@container` → 容器语境（无名条件＝
/// 最近容器；合成无名 [`ContainerRule`] 复用规则域求值——判据页全为无名
/// 尺寸条件）。
fn function_cond_holds(kind: &FunctionCondKind, cond: &CondEvalCtx) -> bool {
    match kind {
        FunctionCondKind::Supports(c) => crate::evaluate_supports_condition(c),
        FunctionCondKind::Media(queries) => match &cond.media {
            Some(ctx) => queries
                .iter()
                .any(|q| zero_css_parser::media_query::evaluate_media_query(q, ctx)),
            None => false,
        },
        FunctionCondKind::Container(c) => match &cond.container {
            Some(ctx) => {
                let synthetic = zero_css_parser::ast::ContainerRule {
                    name: None,
                    condition: c.clone(),
                    extra_conditions: Vec::new(),
                    rules: Vec::new(),
                };
                crate::matcher::evaluate_container_condition(&synthetic, Some(ctx))
            }
            None => false,
        },
    }
}

/// 求值单个函数体（css-mixins-1 §3.1 / §4 执行模型；k119 会话作用域重构）。
///
/// `incoming` = 调用方作用域（k119 穿线：顶层站＝元素计算表；函数体嵌套
/// 调用＝caller 的已解析作用域——§2.3.1 动态作用域；k129 入口按
/// css-variables-1 §3.4 对环成员预置遮蔽标记——见 [`detect_var_reference_cycles`]）。
/// 返回
/// `Ok(Some((产物, attr taint, 本调用已解析作用域)))`——作用域供嵌套调用
/// 穿线，taint 随 [`DashOutcome::Value`] 通道上传（k126，css-values-5
/// §attr-security）；`Ok(None)` = 调用求值失败（缺 result 描述符/必需实参
/// 缺席且无缺省/typed 局部或返回强制失败/产物含顶层块/result var 链不可解
/// /函数域 attr() 文法 invalid——k126）→ Invalid；`Err(())` = 会话内嵌套
/// 调用环 → Cycle 整链传播（k112 T12 归因锚）。typed **实参**强制失败不整
/// 调用 invalid（≡IACVT 实参 → 缺省/遮蔽标记，k120）。`args_tainted` =
/// 实参替换链带入的 taint（实参文本经 var(--a) 引用进产物）。
///
/// 三层作用域视图：
/// - `view`：var 链解析视图＝incoming＋params＋全部局部**原始值**（后者
///   胜）——局部解析顺序无关（`--y: var(--x); --x: 1px` 反向引用可解）。
/// - `raw_scope`：if() 会话视图（k112 语义保持——声明序逐条覆盖，环移除）。
/// - `scope_out`：解析产物作用域＝incoming＋params＋局部**已解析终值**
///   （IACVT 遮蔽标记/关键字移除）——嵌套调用与 result 的基座（「Inner
///   function call should see resolved outer locals」双序判例）。
#[allow(clippy::too_many_arguments)]
fn evaluate_custom_function(
    rule: &FunctionRule,
    args: &[String],
    def_site: &FnChain,
    incoming: &HashMap<String, String>,
    element_seed: Option<u64>,
    font_base_px: Option<f64>,
    depth: usize,
    consuming_prop: &str,
    call_stack: &mut Vec<String>,
    args_tainted: bool,
    attr_reader: &dyn Fn(&str) -> Option<String>,
    random_ctx: &RandomEvalContext,
    types: &HashMap<String, String>,
    cond: &CondEvalCtx,
) -> Result<Option<(String, bool, HashMap<String, String>)>, ()> {
    if depth >= MAX_EVAL_DEPTH {
        return Ok(None);
    }
    // k126：函数域 taint 累积（css-values-5 §attr-security）——实参带入＋
    // 三域（参数缺省/局部声明/result）attr() 替换污染 OR 合并。
    let mut taint_acc = args_tainted;
    let fname = &rule.name;
    // k124：条件组展平（css-mixins-1 §conditional-rules——条件满足的组按源
    // 序展开，§4 last-wins 语义保持）。展平后全流程按平面声明处理。
    let mut flat_decls: Vec<FunctionDecl> = Vec::new();
    flatten_function_body(&rule.body, cond, &mut flat_decls);
    // S5 常规属性门（k111 同口径）：if() 只在自订属性声明代入——消费面
    // 常规属性时体内 if() 保持原文外溢（元素级 4.76 站同门拒）。FIXME 随
    // 全值域刀放开。
    let s5_custom = consuming_prop.starts_with("--");

    // k123a 修正：本帧型别表（调用方帧 types ∪ 本帧参数型别——frame_types_of
    // 精确归属；if() 查询与局部展开同源）。FIXME（per-frame provenance）
    // 已建模摘除。
    let my_types = frame_types_of(types, rule);

    // ── 参数绑定（css-mixins-1 §mixin-args / §3）────────────────────────
    // k119 绑定期解析：实参先于绑定做调用方作用域 var()/env() 代入（§3
    // arbitrary substitution 在调用点——「var() in argument resolved before
    // call」判例：函数体局部不得回染实参）；IACVT 实参（无效 var 无回退）
    // 视同缺席 → 缺省。块实参 `{...}` 整体解包（css-values-5
    // §component-function-commas——「Passing list as only argument」族）。
    // 缺省在参数部分作用域解析（已绑参数可见——「Default referencing
    // another parameter, local interference」判例：缺省读参数绑定而非局部
    // 遮蔽值）；CSS 宽关键字缺省按关键字语义收尾。必需实参缺席且无缺省 →
    // 整调用 invalid（Ok(None)）。
    // k129：incoming 依赖图环预置（css-variables-1 §3.4 cycles——环内属性
    // 全部 invalid at computed-value time＝遮蔽标记处置）。元素级 gather 表
    // 保留环原文，会话内 var 代入撞交替环 → Err → 消费声明 invalid；预置
    // 标记后标记分支按缺席处置 → 消费侧 fallback 生效（「Observing property
    // cycle locally」判据）。主引用位建边、fallback 子树排除＝惰性边语义
    //（unused-fallback 判据族守成）。会话域变换——元素级表零扩散。
    let mut incoming_eff = incoming.clone();
    for member in detect_var_reference_cycles(incoming) {
        incoming_eff.insert(member, SCOPE_IACVT.to_string());
    }
    // 影子绑定：下游视图（参数部分作用域/三层视图/inherit 父帧/缺省域）
    // 统一吃环预置表——嵌套调用穿线同源（inner_scope 已携带标记）。
    let incoming = &incoming_eff;
    let mut param_scope: HashMap<String, String> = incoming.clone();
    let mut scope: HashMap<String, String> = incoming.clone();
    for (idx, param) in rule.params.iter().enumerate() {
        let arg_present = matches!(args.get(idx), Some(a) if !a.is_empty());
        let bound = if arg_present {
            let a = args.get(idx).expect("arg_present");
            let after_env = resolve_env(a).unwrap_or_else(|| a.to_string());
            match substitute_var_text_attributed(&after_env, consuming_prop, incoming) {
                Ok(v) => Some(unwrap_block_arg(&v)),
                Err(_) => None, // 实参 IACVT（在场但 guaranteed-invalid）
            }
        } else {
            None
        };
        let raw = match bound {
            Some(v) if !v.is_empty() => v,
            // IACVT 实参（在场但 guaranteed-invalid）→ 缺省；无缺省 →
            // guaranteed-invalid 遮蔽标记（§3——标记不整调用 invalid：
            // 「IACVT argument shadows outer scope」回退可用，元素同名
            // 不回落）。
            _ if arg_present => match &param.default {
                // k126：缺省域 attr() taint 并入累积。
                Some(d) => {
                    let (v, t) = resolve_default_value(
                        d,
                        param.name.as_str(),
                        &param_scope,
                        incoming,
                        consuming_prop,
                        attr_reader,
                    );
                    taint_acc |= t;
                    // k129：缺省嵌套 dashed 调用展开——栈上再入 → 整调用
                    // invalid（expand_default_dashed_calls 注）。
                    match expand_default_dashed_calls(
                        &v,
                        consuming_prop,
                        def_site,
                        &param_scope,
                        element_seed,
                        font_base_px,
                        depth,
                        call_stack,
                        attr_reader,
                        random_ctx,
                        &my_types,
                        cond,
                    ) {
                        Err(()) => return Ok(None),
                        Ok(None) => SCOPE_IACVT.to_string(),
                        Ok(Some((x, t2))) => {
                            taint_acc |= t2;
                            x
                        }
                    }
                }
                None => SCOPE_IACVT.to_string(),
            },
            // 缺席实参 → 缺省；无缺省 → 整调用 invalid（§3——
            // 「Missing only argument」判例）。
            _ => match &param.default {
                Some(d) => {
                    let (v, t) = resolve_default_value(
                        d,
                        param.name.as_str(),
                        &param_scope,
                        incoming,
                        consuming_prop,
                        attr_reader,
                    );
                    taint_acc |= t;
                    // k129：缺省嵌套 dashed 调用展开——栈上再入 → 整调用
                    // invalid（expand_default_dashed_calls 注）。
                    match expand_default_dashed_calls(
                        &v,
                        consuming_prop,
                        def_site,
                        &param_scope,
                        element_seed,
                        font_base_px,
                        depth,
                        call_stack,
                        attr_reader,
                        random_ctx,
                        &my_types,
                        cond,
                    ) {
                        Err(()) => return Ok(None),
                        Ok(None) => SCOPE_IACVT.to_string(),
                        Ok(Some((x, t2))) => {
                            taint_acc |= t2;
                            x
                        }
                    }
                }
                None => return Ok(None),
            },
        };
        // typed 参数＝假设元素按参数类型注册（§3.1）→ 会话内求值＋类型强制
        //（k120）；untyped 参数原样（参数值 if() 站内联 FIXME——语料无参数级
        // if() 形）。类型强制失败 ≡ IACVT 实参（css-mixins-1「Argument
        // captures IACVT due to type mismatch」/「Arguments are defaulted on
        // type mismatch」判例）→ 缺省（缺省再败 → 遮蔽标记）——不整调用
        // invalid；IACVT 遮蔽使元素同名不回落（「IACVT argument shadows
        // outer scope, type mismatch」）。
        let value = match &param.ty {
            Some(ty) => {
                let key = format!("{fname}-{}", param.name);
                match eval_typed_value(&raw, Some(ty), &param_scope, &key, element_seed, font_base_px) {
                    Some(v) => v,
                    None => match &param.default {
                        Some(d) => {
                            let (dv, t) = resolve_default_value(
                                d,
                                param.name.as_str(),
                                &param_scope,
                                incoming,
                                consuming_prop,
                                attr_reader,
                            );
                            taint_acc |= t;
                            // k129：缺省嵌套 dashed 调用展开——栈上再入 →
                            // 整调用 invalid（expand_default_dashed_calls 注）。
                            let dv = match expand_default_dashed_calls(
                                &dv,
                                consuming_prop,
                                def_site,
                                &param_scope,
                                element_seed,
                                font_base_px,
                                depth,
                                call_stack,
                                attr_reader,
                                random_ctx,
                                &my_types,
                                cond,
                            ) {
                                Err(()) => return Ok(None),
                                Ok(None) => SCOPE_IACVT.to_string(),
                                Ok(Some((x, t2))) => {
                                    taint_acc |= t2;
                                    x
                                }
                            };
                            eval_typed_value(&dv, Some(ty), &param_scope, &key, element_seed, font_base_px)
                                .unwrap_or_else(|| SCOPE_IACVT.to_string())
                        }
                        None => SCOPE_IACVT.to_string(),
                    },
                }
            }
            None => raw,
        };
        param_scope.insert(param.name.clone(), value.clone());
        scope.insert(param.name.clone(), value);
    }

    // ── 局部三层视图（见 fn 文档）────────────────────────────────────
    let mut final_raw: HashMap<String, String> = HashMap::new();
    for decl in &flat_decls {
        if decl.name.starts_with("--") {
            final_raw.insert(decl.name.clone(), decl.value.clone());
        }
    }
    let mut view: HashMap<String, String> = scope.clone();
    for (k, v) in &final_raw {
        view.insert(k.clone(), v.clone());
    }
    let mut raw_scope: HashMap<String, String> = view.clone();
    let mut scope_out: HashMap<String, String> = scope.clone();

    // k129：局部声明依赖图环预置（css-variables-1 §3.4 cycles——环成员全部
    // invalid at computed-value time）。与循环内逐条求值同形账：raw_scope 留
    // 原文＋scope_out 遮蔽标记，成员跳过求值直接落账——环命中 fallback 逃逸
    // 通道（[`resolve_var_reference_at`] 环命中侧 fallback 解出）不适用（
    // 「Using cyclic values with no fallback」判据期望 guaranteed-invalid，
    // fallback 破环即伪值「3 1」形态）。主引用位建边、fallback 子树排除——
    // 「Local with self-cycle in unused fallback」守成键要求 fallback 不参与
    // 环成员资格（主链可解即非环）。
    let local_cycles = detect_var_reference_cycles(&final_raw);

    // 局部解析产物。
    enum Local {
        /// 终值（入 scope_out）。
        Value(String),
        /// guaranteed-invalid 遮蔽标记（入 scope_out——不沿链回落调用方）。
        Marker,
        /// 关键字移除（不入 scope_out——回落参数/调用方同名值）。
        Removed,
    }
    // locals：声明序后者胜（§4）。与 typed 参数同名者继承参数注册语义
    //（值覆写、类型仍在——§2.3.1 shadowing；③⑦ 断言定谳）→ 会话内求值。
    // k112：untyped local 含 if() → 先会话内站（锚＝局部名；raw_scope 逐条
    // 覆盖语义保持——T1-T11 判据），R1 环 → IACVT。
    // k119：解析管线 var()/env()（锚＝局部名，视图作用域——自指环 →
    // IACVT）→ 嵌套 dashed 调用以**视图作用域**展开（穿线——内层调用可见
    // 全部局部链值）→ CSS 宽关键字按关键字语义收尾（§2.3.1：initial 移除
    // 局部、inherit 取调用方作用域值、unset/revert 族 → guaranteed-invalid
    // 遮蔽）。产物入 scope_out；原始值副本留 raw_scope（if() 语义不变）。
    for decl in &flat_decls {
        if !decl.name.starts_with("--") {
            continue;
        }
        // k129：环成员跳过求值（预置账已落——见 `local_cycles` 注）。
        if local_cycles.contains(decl.name.as_str()) {
            raw_scope.insert(decl.name.clone(), decl.value.clone());
            scope_out.insert(decl.name.clone(), SCOPE_IACVT.to_string());
            continue;
        }
        let typed_ty = rule
            .params
            .iter()
            .find(|p| p.name == decl.name)
            .and_then(|p| p.ty.clone());
        let outcome = if let Some(ty) = typed_ty {
            match eval_typed_value(
                &decl.value,
                Some(&ty),
                &view,
                &format!("{fname}-{}", decl.name),
                element_seed,
                font_base_px,
            ) {
                Some(v) => Local::Value(v),
                None => {
                    // typed 局部求值失败 → IACVT 遮蔽（§2.3.1 local
                    // registration——「Invalid value for typed local becomes
                    // IACVT」：消费 var 走 fallback，不整调用 invalid）。
                    raw_scope.remove(&decl.name);
                    scope_out.insert(decl.name.clone(), SCOPE_IACVT.to_string());
                    continue;
                }
            }
        } else {
            let stage1 = if s5_custom && if_function::has_if_call(&decl.value) {
                match if_session_eval(
                    &decl.value,
                    &decl.name,
                    &raw_scope,
                    random_ctx,
                    attr_reader,
                    depth,
                    &my_types,
                ) {
                    IfOutcome::Substituted(v) => {
                        raw_scope.insert(decl.name.clone(), v.clone());
                        v
                    }
                    IfOutcome::Iacvt => {
                        raw_scope.remove(&decl.name);
                        scope_out.insert(decl.name.clone(), SCOPE_IACVT.to_string());
                        continue;
                    }
                    IfOutcome::Unchanged => decl.value.clone(),
                }
            } else {
                decl.value.clone()
            };
            // k123：inherit() 记法替换（css-values-5 §inherit-notation ×
            // css-mixins-1 栈帧语义——「inherit() refers to parent stack
            // frame」族）：父栈帧＝调用方已解析作用域（incoming——k119 穿
            // 线产物/元素 gather 表）。局部声明不遮蔽父帧读取（inherit()
            // 语义即跳过本帧）。guaranteed-invalid（遮蔽标记/缺席）无
            // fallback → 遮蔽 IACVT（同 var Err 口径）。
            // <https://drafts.csswg.org/css-values-5/#inherit-notation>
            let stage1 = if has_inherit_call(&stage1) {
                match substitute_inherit_calls(&stage1, incoming) {
                    Ok(v) => v,
                    Err(()) => {
                        raw_scope.insert(decl.name.clone(), stage1);
                        scope_out.insert(decl.name.clone(), SCOPE_IACVT.to_string());
                        continue;
                    }
                }
            } else {
                stage1
            };
            // var()/env() 作用域代入（锚＝局部名——自指环 → IACVT）。
            // k127：var()/attr() 迭代替换至不动点（css-mixins-1
            // §locally-substitute-a-var——「var() in attribute value」判据
            // 族：attr() 属性值产物可含 var() 引用，须按本帧局部视图续解；
            // 交替不收敛（--x: attr(data-x) 产物 var(--x) 回指自身）按环
            // 处置 → IACVT 遮蔽）。attr 站走函数求值域宽松通道（Syntax 型
            // 别属性值含替换词 splice 原文延迟文法裁决，css-values-5
            // §8.7.1 step 6 先替后查）。文法 invalid → IACVT 遮蔽（同 var
            // 环口径）；产物 taint 并入函数域累积（保守口径：局部声明污染
            // 上浮至整调用产物——taint 判定按产物 url 型别，判据域内污染
            // 必被 result 引用消费）。
            // <https://drafts.csswg.org/css-values-5/#attr-security>
            let mut resolved = resolve_env(&stage1).unwrap_or_else(|| stage1.clone());
            let mut iacvt = false;
            let mut waves = 0usize;
            loop {
                let mut changed = false;
                match substitute_var_text_attributed(&resolved, consuming_prop, &view) {
                    Ok(v) => {
                        changed |= v != resolved;
                        resolved = v;
                    }
                    Err(_) => {
                        iacvt = true;
                        break;
                    }
                }
                match zero_css_parser::values::substitute_attr_values_fn_domain(&resolved, attr_reader) {
                    Some((std::borrow::Cow::Owned(s), t)) => {
                        taint_acc |= t;
                        resolved = s;
                        changed = true;
                    }
                    Some((std::borrow::Cow::Borrowed(_), _)) => {}
                    None => {
                        iacvt = true;
                        break;
                    }
                }
                if !changed {
                    break;
                }
                waves += 1;
                if waves > SUBSTITUTE_FIXPOINT_LIMIT {
                    iacvt = true;
                    break;
                }
            }
            if iacvt {
                raw_scope.insert(decl.name.clone(), stage1);
                scope_out.insert(decl.name.clone(), SCOPE_IACVT.to_string());
                continue;
            }
            // 嵌套 dashed 调用展开（视图作用域穿线；调用栈沿用站栈——互调
            // 环归因 T12）。Cycle → 整链传播。
            let expanded = match substitute_dashed_functions(
                &resolved,
                consuming_prop,
                def_site,
                &view,
                element_seed,
                font_base_px,
                depth + 1,
                call_stack,
                attr_reader,
                random_ctx,
                &my_types,
                cond,
            ) {
                DashOutcome::Cycle => return Err(()),
                DashOutcome::Invalid => {
                    scope_out.insert(decl.name.clone(), SCOPE_IACVT.to_string());
                    continue;
                }
                // k126：局部嵌套替换 taint 并入累积。
                DashOutcome::Value(v, t) => {
                    taint_acc |= t;
                    v
                }
            };
            match classify_css_wide_keyword(expanded.trim()) {
                Some(CssWideKeyword::Initial) => Local::Removed,
                Some(CssWideKeyword::Inherit) => {
                    // inherit＝调用方作用域该名取值（§2.3.1）；缺席/标记 →
                    // guaranteed-invalid 遮蔽。
                    match incoming.get(decl.name.as_str()).map(|v| v.as_str()) {
                        Some(v) if v != SCOPE_IACVT => {
                            match substitute_var_text_attributed(v, consuming_prop, incoming) {
                                Ok(r) => Local::Value(r),
                                Err(_) => Local::Marker,
                            }
                        }
                        _ => Local::Marker,
                    }
                }
                Some(_) => Local::Marker, // unset/revert/revert-layer/revert-rule → IACVT 遮蔽
                None => Local::Value(expanded),
            }
        };
        match outcome {
            Local::Value(v) => {
                scope_out.insert(decl.name.clone(), v);
            }
            Local::Marker => {
                scope_out.insert(decl.name.clone(), SCOPE_IACVT.to_string());
            }
            Local::Removed => {
                // 关键字移除＝回落下一层作用域（参数绑定 → 调用方同名值——
                // §2.3.1「Local variable with initial keyword」：局部 initial
                // 后参数绑定重新可见）。
                match scope.get(&decl.name) {
                    Some(next_level) => {
                        scope_out.insert(decl.name.clone(), next_level.clone());
                    }
                    None => {
                        scope_out.remove(&decl.name);
                    }
                }
            }
        }
    }

    // result：声明序后者胜（§4 执行模型）。k112：含 if() 先会话内站
    //（锚＝消费属性；raw_scope 视图）后 var 代入——站内支路 var 以锚归因
    //（var(--消费名) 环 → IACVT），未取支路不求值（R4）。
    // k119：var 代入改锚定式（Err → 调用 invalid——guaranteed-invalid 链
    // 不再字面外溢，消费声明 IACVT）。
    let Some(result) = flat_decls.iter().rev().find(|d| d.name == "result") else {
        // 缺 result 描述符 → 调用 invalid（§3——「Missing result descriptor」）。
        return Ok(None);
    };
    let mut text_input = result.value.clone();
    if s5_custom && if_function::has_if_call(&text_input) {
        match if_session_eval(
            &text_input,
            consuming_prop,
            &raw_scope,
            random_ctx,
            attr_reader,
            depth,
            &my_types,
        ) {
            IfOutcome::Substituted(v) => text_input = v,
            // result 级环/无命中支路无 else → 调用 invalid（k119——原字面
            // 外溢改 Invalid：消费声明 IACVT 同向可观察）。
            IfOutcome::Iacvt => return Ok(None),
            IfOutcome::Unchanged => {}
        }
    }
    // k123：result 站 inherit() 替换（父栈帧＝调用方作用域）。guaranteed-
    // invalid 无 fallback → 调用 invalid（k119 Err → 调用 invalid 同向）。
    if has_inherit_call(&text_input) {
        match substitute_inherit_calls(&text_input, incoming) {
            Ok(v) => text_input = v,
            Err(()) => return Ok(None),
        }
    }
    // k127：result 域 var()/attr() 迭代替换至不动点（同局部域口径——
    // attr() 产物 var() 词按本帧 scope_out 续解，「var() in attribute
    // value substitutes locally」判据族；交替不收敛按环处置 → 调用
    // invalid，k119 Err 同向）。attr 站走函数求值域宽松通道（Syntax 型别
    // 属性值含替换词 splice 原文延迟文法裁决，css-values-5 §8.7.1 step 6
    // 先替后查；typed result 由下方 returns 强制收口）。文法 invalid →
    // 调用 invalid；产物 taint 并入累积。
    // <https://drafts.csswg.org/css-values-5/#attr-security>
    let mut text = resolve_env(&text_input).unwrap_or_else(|| text_input.clone());
    let mut waves = 0usize;
    loop {
        let mut changed = false;
        match substitute_var_text_attributed(&text, consuming_prop, &scope_out) {
            Ok(v) => {
                changed |= v != text;
                text = v;
            }
            Err(_) => return Ok(None),
        }
        match zero_css_parser::values::substitute_attr_values_fn_domain(&text, attr_reader) {
            Some((std::borrow::Cow::Owned(s), t)) => {
                taint_acc |= t;
                text = s;
                changed = true;
            }
            Some((std::borrow::Cow::Borrowed(_), _)) => {}
            None => return Ok(None),
        }
        if !changed {
            break;
        }
        waves += 1;
        if waves > SUBSTITUTE_FIXPOINT_LIMIT {
            return Ok(None);
        }
    }
    // 块产物门（k120）：产物含顶层 `{}` 块 → 调用 invalid（消费声明侧
    // custom property 值拒顶层块——「Passing {} as argument」族对称补齐）。
    if product_has_top_level_block(&text) {
        return Ok(None);
    }
    match rule.returns.as_deref() {
        Some(ty) => {
            let v = eval_typed_value(
                &text,
                Some(ty),
                &scope_out,
                &format!("{fname}-result"),
                element_seed,
                font_base_px,
            );
            match v {
                // typed 返回强制失败 → 调用 invalid（§3——「Literal result,
                // typed return, mismatch」：非环语义走 Invalid，不冒 Cycle）。
                None => Ok(None),
                Some(v) => Ok(Some((v, taint_acc, scope_out))),
            }
        }
        None => Ok(Some((text, taint_acc, scope_out))),
    }
}

/// 会话内 if() 站求值（k112，css-mixins-1 §locally-substitute-a-var ×
/// css-values-5 §if-notation）：函数作用域视图作 customs（locals/params
/// 遮蔽元素基座），环归因锚＝被求值声明（局部名或消费属性）。attr 读取
/// 与 random 语境沿用调用站语境（attr 元素级；random 键 prop 维＝锚名）。
fn if_session_eval(
    text: &str,
    anchor: &str,
    scope: &HashMap<String, String>,
    random_ctx: &RandomEvalContext,
    attr_reader: &dyn Fn(&str) -> Option<String>,
    depth: usize,
    types: &HashMap<String, String>,
) -> IfOutcome {
    let ctx = IfEvalCtx {
        customs: scope,
        attr_reader,
        random_ctx,
        type_lookup: Some(types),
        font_base_px: random_ctx.font_base_px,
    };
    if_function::substitute_if_calls(text, anchor, &ctx, depth)
}

/// 值文本是否含 `inherit(` 调用形（ASCII ci；ident 边界感知——
/// `--inherit(`/`reinherit(` 非独立函数名）。字符串字面内形似体由本层
/// 文本扫描处理，FIXME 随 tokenizer 级替换刀收严。
fn has_inherit_call(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    for k in 0..bytes.len().saturating_sub(7) {
        if &bytes[k..k + 8] == b"inherit(" {
            // ident 边界：前缀 ident 字符 → 非独立函数名。
            if k > 0 {
                let prev = lower[..k].chars().next_back().expect("k>0 非空");
                if is_ident_char(prev) {
                    continue;
                }
            }
            return true;
        }
    }
    false
}

/// k123：会话内 inherit() 记法替换（css-values-5 §inherit-notation ×
/// css-mixins-1 栈帧语义）。`inherit(--name[, fallback])` 取**父栈帧**
/// （调用方已解析作用域）值：在帧值 splice（已解析文本，不重扫——拼接
/// 内容内形似体保字面，零重入面）；SCOPE_IACVT 遮蔽标记/缺席＝父帧
/// guaranteed-invalid → 有 fallback 取 fallback（原样拼接，下游 var 站
/// 续解）、无 fallback → `Err(())`（guaranteed-invalid 链——声明 IACVT/
/// 调用 invalid）。非 `--` 首参（标准属性查询——级联面缺位）保持字面
/// （FIXME）。父帧值恒已解析（k119 穿线产物/元素 gather 表）——无环面。
/// <https://drafts.csswg.org/css-values-5/#inherit-notation>
fn substitute_inherit_calls(text: &str, caller_frame: &HashMap<String, String>) -> Result<String, ()> {
    let lower = text.to_ascii_lowercase();
    let lbytes = lower.as_bytes();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut out = String::with_capacity(text.len());
    let mut last = 0usize; // 未产段起点（字节偏移）
    let mut k = 0usize; // chars 游标
    while k + 7 < chars.len() {
        let (i, c) = chars[k];
        if (c == 'i' || c == 'I') && &lbytes[i..i + 8] == b"inherit(" {
            // ident 边界：前缀 ident 字符 → 非独立函数名。
            let boundary_ok = k == 0 || !is_ident_char(chars[k - 1].1);
            if boundary_ok {
                // "inherit" 纯 ASCII → 开括号恰在 chars[k+7]。
                if let Some(cm) = matching_close_paren(&chars, k + 7) {
                    let inner = &text[chars[k + 7].0 + 1..chars[cm].0];
                    let parts = split_top_level_commas(inner);
                    let name = parts.first().map(|s| s.trim()).unwrap_or("");
                    let fallback = parts.get(1).map(|s| s.trim());
                    if name.starts_with("--") && parts.len() <= 2 {
                        out.push_str(&text[last..i]);
                        match caller_frame.get(name) {
                            Some(v) if v != SCOPE_IACVT => out.push_str(v),
                            _ => match fallback {
                                Some(fb) => out.push_str(fb),
                                None => return Err(()),
                            },
                        }
                        last = chars[cm].0 + 1;
                        k = cm + 1;
                        continue;
                    }
                    // 非 `--` 首参/多参形 → 字面透传（FIXME 标准属性查询域）。
                }
            }
        }
        k += 1;
    }
    if last == 0 {
        return Ok(text.to_string());
    }
    out.push_str(&text[last..]);
    Ok(out)
}

/// CSS 宽关键字分类（css-values-4 §css-wide-keywords；ASCII 大小写不敏感，
/// 整值修剪后单 ident 才命中）。局部/缺省值解析管线的收尾分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssWideKeyword {
    Initial,
    Inherit,
    Unset,
    Revert,
    RevertLayer,
    RevertRule,
}

/// 值文本是否恰为一个 CSS 宽关键字（trim 后整串匹配，大小写不敏感）。
/// revert-rule：css-mixins-1 §2.3.1 局部域关键字（csswg-drafts#10443）。
fn classify_css_wide_keyword(value: &str) -> Option<CssWideKeyword> {
    let v = value.trim();
    if v.is_empty() || !v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return None;
    }
    let lower = v.to_ascii_lowercase();
    match lower.as_str() {
        "initial" => Some(CssWideKeyword::Initial),
        "inherit" => Some(CssWideKeyword::Inherit),
        "unset" => Some(CssWideKeyword::Unset),
        "revert" => Some(CssWideKeyword::Revert),
        "revert-layer" => Some(CssWideKeyword::RevertLayer),
        "revert-rule" => Some(CssWideKeyword::RevertRule),
        _ => None,
    }
}

/// 块实参解包（css-values-5 §component-function-commas，k119）：实参整体为
/// 单个顶层平衡 `{...}` 块时剥壳取内容（`{1px, 2px}` → `1px, 2px`——列表
/// 作实参）；只剥一层（`{{}}` → `{}`——内容仍为块值，「Passing {} as
/// argument」）；非整体块（首块中途闭合，`{a}{b}`）与无块形原样返回。
/// 剥壳后空内容返回空串（调用方按缺席处置——空块调用形 k118 写面已拒，
/// 此臂为嵌套/变量间接形兜底）。
fn unwrap_block_arg(arg: &str) -> String {
    let a = arg.trim();
    if !(a.starts_with('{') && a.ends_with('}') && a.len() >= 2) {
        return a.to_string();
    }
    let mut depth = 0i32;
    let mut closed_at_end = false;
    for (k, ch) in a.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    closed_at_end = k + 1 == a.len();
                    break;
                }
            }
            _ => {}
        }
    }
    if depth != 0 || !closed_at_end {
        return a.to_string();
    }
    a[1..a.len() - 1].trim().to_string()
}

/// 参数缺省值解析（k119 绑定期）：var()/env() 在参数部分作用域代入（已绑
/// 参数可见——「Default referencing another parameter, local interference」
/// 判例读参数绑定而非局部遮蔽值；锚＝参数名——缺省自指环 → IACVT）；
/// attr() 替换（k126，css-values-5 §attr-notations——替换序 var→attr，
/// 「attr() in default parameter value」判例）＋taint 回传；CSS 宽关键字
/// 按关键字语义收尾（§2.3.1：initial/unset/revert 族 → guaranteed-invalid
/// 遮蔽标记；inherit → 调用方作用域该参数名取值）。返回（值, taint）。
fn resolve_default_value(
    default: &str,
    param_name: &str,
    param_scope: &HashMap<String, String>,
    incoming: &HashMap<String, String>,
    consuming_prop: &str,
    attr_reader: &dyn Fn(&str) -> Option<String>,
) -> (String, bool) {
    let after_env = resolve_env(default).unwrap_or_else(|| default.to_string());
    let resolved = match substitute_var_text_attributed(&after_env, consuming_prop, param_scope) {
        Ok(v) => v,
        // 缺省 var 链不可解 → guaranteed-invalid 遮蔽标记。
        Err(_) => return (SCOPE_IACVT.to_string(), false),
    };
    // k126：attr() 站（css-values-5 §attr-notations）——文法 invalid →
    // IACVT 遮蔽标记（同 var 链不可解口径）；产物 taint 回传。
    // <https://drafts.csswg.org/css-values-5/#attr-security>
    let (resolved, taint) = match zero_css_parser::values::substitute_attr_values_checked(&resolved, attr_reader) {
        Some((std::borrow::Cow::Owned(s), t)) => (s, t),
        Some((std::borrow::Cow::Borrowed(_), _)) => (resolved, false),
        None => return (SCOPE_IACVT.to_string(), false),
    };
    match classify_css_wide_keyword(resolved.trim()) {
        Some(CssWideKeyword::Inherit) => match incoming.get(param_name).map(|v| v.as_str()) {
            Some(v) if v != SCOPE_IACVT => {
                let v = substitute_var_text_attributed(v, consuming_prop, incoming)
                    .unwrap_or_else(|_| SCOPE_IACVT.to_string());
                // FIXME: inherit 分支产物不补 attr 重裁（判据域无此形）。
                (v, taint)
            }
            _ => (SCOPE_IACVT.to_string(), false),
        },
        Some(_) => (SCOPE_IACVT.to_string(), false),
        None => (resolved, taint),
    }
}

/// k129：参数缺省产物嵌套 dashed 调用展开（css-mixins-1 §3.1——缺省值属
/// 函数求值域，嵌套调用照展开；调用栈沿用本帧 push 态——栈上再入即检出）。
/// `Err(())`＝缺省展开撞调用环 → 调用方整调用 invalid（§3.1 cyclic
/// substitution context，「Function in a cycle with its own default」判据）；
/// `Ok(None)`＝嵌套调用求值失败 → 参数值按遮蔽标记处置（同 IACVT 实参
/// 口径）；`Ok(Some)`＝展开产物＋attr taint（k126 完整性）。
#[allow(clippy::too_many_arguments)]
fn expand_default_dashed_calls(
    text: &str,
    consuming_prop: &str,
    def_site: &FnChain,
    scope: &HashMap<String, String>,
    element_seed: Option<u64>,
    font_base_px: Option<f64>,
    depth: usize,
    call_stack: &mut Vec<String>,
    attr_reader: &dyn Fn(&str) -> Option<String>,
    random_ctx: &RandomEvalContext,
    types: &HashMap<String, String>,
    cond: &CondEvalCtx,
) -> Result<Option<(String, bool)>, ()> {
    match substitute_dashed_functions(
        text,
        consuming_prop,
        def_site,
        scope,
        element_seed,
        font_base_px,
        depth + 1,
        call_stack,
        attr_reader,
        random_ctx,
        types,
        cond,
    ) {
        DashOutcome::Cycle => Err(()),
        DashOutcome::Invalid => Ok(None),
        DashOutcome::Value(v, t) => Ok(Some((v, t))),
    }
}

/// k129：自订属性依赖图环成员检测（css-variables-1 §3.4 cycles——环内
/// 属性全部 invalid at computed-value time）。引用边按 var() **主引用位**
/// 建边（fallback 子树整段跳过＝惰性求值语义——主链可解时 fallback 不
/// 参与依赖，「Local with self-cycle in unused fallback」判据族守成）；
/// 仅收录表内存在的目标（表外引用不构成表内环）。返回环成员名集（自引用
/// 环含自身）。纯文本扫描——字符串字面内形似体由本层文本扫描处理，
/// FIXME 随 tokenizer 级替换刀收严（同 [`has_inherit_call`] 口径）。
fn detect_var_reference_cycles(map: &HashMap<String, String>) -> HashSet<String> {
    // 建边：name → 主引用位 var() 目标（ASCII ci 函数名——CSS 函数名
    // 大小写不敏感；自订属性名本身大小写敏感，目标名自原值切片）。
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    for (name, value) in map {
        let mut refs: Vec<String> = Vec::new();
        let lower = value.to_ascii_lowercase();
        let bytes = lower.as_bytes();
        let mut k = 0usize;
        while k + 4 <= bytes.len() {
            if &bytes[k..k + 4] == b"var(" {
                // 主引用位名：var( 后跳空白取自订属性名。
                let mut m = k + 4;
                while m < bytes.len() && bytes[m].is_ascii_whitespace() {
                    m += 1;
                }
                if bytes[m..].starts_with(b"--") {
                    let mut e = m + 2;
                    while e < bytes.len() && is_ident_char(bytes[e] as char) {
                        e += 1;
                    }
                    if e > m + 2 && map.contains_key(&value[m..e]) {
                        refs.push(value[m..e].to_string());
                    }
                }
                // 跳过整个 var(...)（fallback 子树排除＝惰性边语义）。
                let mut depth = 1i32;
                let mut m2 = k + 4;
                while m2 < bytes.len() && depth > 0 {
                    match bytes[m2] {
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        _ => {}
                    }
                    m2 += 1;
                }
                k = m2;
            } else {
                k += 1;
            }
        }
        edges.insert(name.clone(), refs);
    }
    // DFS 三色环检测（白/灰/黑）＋显式灰栈：回边命中灰节点 → 灰栈自命中
    // 点至栈顶整段记成员（自引用环＝单元素段）。
    fn dfs<'a>(
        node: &'a str,
        edges: &'a HashMap<String, Vec<String>>,
        state: &mut HashMap<&'a str, u8>,
        stack: &mut Vec<&'a str>,
        members: &mut HashSet<String>,
    ) {
        state.insert(node, 1);
        stack.push(node);
        if let Some(targets) = edges.get(node) {
            for target in targets {
                let target = target.as_str();
                match state.get(target) {
                    Some(0) | None => dfs(target, edges, state, stack, members),
                    Some(1) => {
                        if let Some(pos) = stack.iter().position(|&x| x == target) {
                            members.extend(stack[pos..].iter().map(|name| (*name).to_string()));
                        }
                    }
                    _ => {}
                }
            }
        }
        stack.pop();
        state.insert(node, 2);
    }
    let mut state: HashMap<&str, u8> = edges.keys().map(|k| (k.as_str(), 0)).collect();
    let mut stack: Vec<&str> = Vec::new();
    let mut members = HashSet::new();
    for key in edges.keys() {
        if state.get(key.as_str()) == Some(&0) {
            dfs(key.as_str(), &edges, &mut state, &mut stack, &mut members);
        }
    }
    members
}

/// typed 值会话内求值（css-mixins-1 §3.1 假设子元素注册语义）：作用域
/// var 代入后 random 替换——键 ua-ident 捕获（css-values-5 §9.4.1 type
/// that causes evaluation），键名＝`{fname}-{internal}`（参数/局部名带双杠
/// → 三连杠拼形；result 无杠名 → 单杠拼形）。None＝guaranteed-invalid
///（不可解 random 或类型强制失败）。
///
/// k120 类型强制层：random 替换产物按型别计算/校验（`<length>` calc 化简
/// ＋px canonical、`<angle>` 归一 deg、`<time>` 归一 s、`<number>`/`<color>`
/// 校验透传）；`type(...)` 包装取内层单型，`+` 多值逐分量；联合型/未知型
/// 保守不查（求值恒过——零 P2F 面）。
/// <https://drafts.csswg.org/css-mixins-1/#function-rule>
fn eval_typed_value(
    text: &str,
    ty: Option<&str>,
    scope: &HashMap<String, String>,
    key_property: &str,
    element_seed: Option<u64>,
    font_base_px: Option<f64>,
) -> Option<String> {
    let resolved = resolve_env_and_var(text, scope);
    let ctx = RandomEvalContext {
        element_seed,
        pct_typed: false,
        font_base_px,
    };
    let substituted = substitute_random_values(&resolved, key_property, &ctx).map(std::borrow::Cow::into_owned)?;
    match parse_type_spec(ty) {
        None => Some(substituted),
        Some((kind, multi)) => typed_compute(&substituted, kind, multi, font_base_px),
    }
}

/// typed 型别规约（css-mixins-1 §2.1 `<css-type>`）：已知单型返回
/// `(kind, multi)`；`type(...)` 包装取内层单型、`+` 多值后缀折叠；联合型
/// 与未知型（`<integer>`/`<percentage>`/`<string>`/自定义 ident 等）→ None
/// ＝不查不化简（透传）。
fn parse_type_spec(ty: Option<&str>) -> Option<(TypedKind, bool)> {
    let raw = ty?.trim();
    let inner = raw
        .strip_prefix("type(")
        .and_then(|r| r.strip_suffix(')'))
        .map(str::trim)
        .unwrap_or(raw);
    let (base, multi) = match inner.strip_suffix('+') {
        Some(b) => (b.trim(), true),
        None => (inner, false),
    };
    let kind = match base.to_ascii_lowercase().as_str() {
        "<length>" => TypedKind::Length,
        "<number>" => TypedKind::Number,
        "<angle>" => TypedKind::Angle,
        "<time>" => TypedKind::Time,
        "<color>" => TypedKind::Color,
        _ => return None,
    };
    Some((kind, multi))
}

/// 已知单型（css-values-4 §6 基础型；k120 求值域）。
#[derive(Debug, Clone, Copy, PartialEq)]
enum TypedKind {
    /// `<length>`：calc 化简＋px canonical 序列化。
    Length,
    /// `<number>`：可解析校验，原文透传（不 canonical 化——random/原样键零漂移）。
    Number,
    /// `<angle>`：四单位归一 canonical deg。
    Angle,
    /// `<time>`：s/ms 归一 canonical s。
    Time,
    /// `<color>`：可解析校验，原文透传。
    Color,
}

/// typed 分量求值：单分量 → canonical 产物文本（Number/Color 校验透传）；
/// None＝类型强制失败（guaranteed-invalid）。
fn typed_component(kind: TypedKind, comp: &str, font_base_px: Option<f64>) -> Option<String> {
    match kind {
        TypedKind::Number => comp.parse::<f64>().map(|_| comp.to_string()).ok(),
        TypedKind::Color => parse_color(comp).map(|_| comp.to_string()),
        TypedKind::Length => {
            // plain token 先行；calc()/math 函数形回落 parse_calc（计算树
            // 整体求值——Length 叶走 resolve_length_to_px 同一 px 基）。
            let expr = match parse_length(comp) {
                Some(lv) => CalcExpr::Length(lv),
                None => parse_calc(comp)?,
            };
            let ctx = CalcContext {
                font_size: font_base_px,
                root_font_size: font_base_px,
                ..CalcContext::default()
            };
            let px = eval_calc_with_context(&expr, &ctx)?;
            Some(format!("{px}px"))
        }
        TypedKind::Angle => {
            let rad = parse_angle_to_radians(comp)?;
            Some(format!("{}deg", rad * 180.0 / std::f64::consts::PI))
        }
        TypedKind::Time => {
            let t = parse_time_token(comp)?;
            Some(format!("{t}s"))
        }
    }
}

/// `<time>` 分量解析（css-values-4 §6.3：s/ms 两单位；ms 归一秒）。
fn parse_time_token(comp: &str) -> Option<f64> {
    let lower = comp.trim().to_ascii_lowercase();
    if let Some(v) = lower.strip_suffix("ms") {
        v.trim().parse::<f64>().ok().map(|x| x / 1000.0)
    } else {
        lower.strip_suffix('s').and_then(|v| v.trim().parse::<f64>().ok())
    }
}

/// typed 求值：单值或 `+` 多值（空白切分量，空格单间隔连回）。
fn typed_compute(text: &str, kind: TypedKind, multi: bool, font_base_px: Option<f64>) -> Option<String> {
    if !multi {
        return typed_component(kind, text.trim(), font_base_px);
    }
    let comps: Vec<&str> = text.split_ascii_whitespace().collect();
    if comps.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(comps.len());
    for c in comps {
        out.push(typed_component(kind, c, font_base_px)?);
    }
    Some(out.join(" "))
}

/// k123a 修正：per-frame 型别表（调用方帧型别 ∪ 本函数参数型别——同名
/// 参数本函数胜，精确归属）。值出自哪层帧，型别随哪层帧——k122 的
/// params 全局联合对同名跨函数参数（`--g(--c <color>)` vs
/// `--f(--c type(*)`)）HashMap 迭代序不定＝非确定型别（「Universally
/// typed」键 `lime`≡`#0f0` 同色解析碰撞撞出 P2F），改帧链穿线定谳。
fn frame_types_of(caller: &HashMap<String, String>, rule: &FunctionRule) -> HashMap<String, String> {
    let mut t = caller.clone();
    for p in &rule.params {
        if let Some(ty) = &p.ty {
            t.insert(p.name.clone(), ty.clone());
        }
    }
    t
}

/// k122：style() 冒号等值类型感知比较（css-mixins-1 §2.1 typed 参数＝
/// 局部注册语义，function-parameter-types 判据族）。双侧按型别**解析为
/// 可比值再比较**（`<color>` "red"≡"#f00"≡Rgba；`<length>` px 数值；
/// `<angle>` rad／`<time>` s 数值；`<number>` f64）。联合型/未知型/
/// type(*)（parse_type_spec 不识）或任一侧解析失败 → None＝回退裸文本
/// 比较（观测恒同——解析确定性 ⇒ 同文本同解析值，唯一翻转 False→True，
/// 构造性零 P2F）。
/// <https://drafts.csswg.org/css-mixins-1/#function-rule>
pub(crate) fn typed_text_eq(a: &str, b: &str, ty: &str, font_base_px: Option<f64>) -> Option<bool> {
    let (kind, multi) = parse_type_spec(Some(ty))?;
    typed_parse_eq(a, b, kind, multi, font_base_px)
}

/// typed 双侧解析等值：单分量直比；`+` 多值分量序对齐逐位比（分量数不
/// 等 → None 回退）。比较失败面（解析失败）→ None。
fn typed_parse_eq(a: &str, b: &str, kind: TypedKind, multi: bool, font_base_px: Option<f64>) -> Option<bool> {
    if !multi {
        return typed_single_eq(a.trim(), b.trim(), kind, font_base_px);
    }
    let ca: Vec<&str> = a.split_ascii_whitespace().collect();
    let cb: Vec<&str> = b.split_ascii_whitespace().collect();
    if ca.is_empty() || ca.len() != cb.len() {
        return None;
    }
    for (x, y) in ca.iter().zip(cb.iter()) {
        if !typed_single_eq(x, y, kind, font_base_px)? {
            return Some(false);
        }
    }
    Some(true)
}

/// 单分量解析等值：双侧解析成功 → 比解析值；任一失败 → None（回退）。
fn typed_single_eq(a: &str, b: &str, kind: TypedKind, font_base_px: Option<f64>) -> Option<bool> {
    match kind {
        TypedKind::Number => match (a.parse::<f64>(), b.parse::<f64>()) {
            (Ok(x), Ok(y)) => Some(x == y),
            _ => None,
        },
        TypedKind::Color => match (parse_color(a), parse_color(b)) {
            (Some(x), Some(y)) => Some(x == y),
            _ => None,
        },
        TypedKind::Length => match (typed_length_px(a, font_base_px), typed_length_px(b, font_base_px)) {
            (Some(x), Some(y)) => Some(x == y),
            _ => None,
        },
        TypedKind::Angle => match (parse_angle_to_radians(a), parse_angle_to_radians(b)) {
            (Some(x), Some(y)) => Some(x == y),
            _ => None,
        },
        TypedKind::Time => match (parse_time_token(a), parse_time_token(b)) {
            (Some(x), Some(y)) => Some(x == y),
            _ => None,
        },
    }
}

/// `<length>` 分量解析为 px 数值（plain token 先行；calc()/math 函数形
/// 回落 parse_calc——typed_component Length 臂同一 px 基）。
fn typed_length_px(comp: &str, font_base_px: Option<f64>) -> Option<f64> {
    let expr = match parse_length(comp) {
        Some(lv) => CalcExpr::Length(lv),
        None => parse_calc(comp)?,
    };
    let ctx = CalcContext {
        font_size: font_base_px,
        root_font_size: font_base_px,
        ..CalcContext::default()
    };
    eval_calc_with_context(&expr, &ctx)
}

/// 顶层块扫描（k120 块产物门）：任一 `{` 出现在 `()`/`[]`/`{}` 括号深度
/// 全零处 → true。产物含顶层块 → 调用 invalid（消费声明侧 custom property
/// 值拒顶层块——「Passing {} as argument」族：`--expected: {}` 已被声明
/// 门 IACVT，实参块字面透传不得绕过同门）。
fn product_has_top_level_block(text: &str) -> bool {
    let (mut dp, mut db, mut dc) = (0i32, 0i32, 0i32);
    for ch in text.chars() {
        match ch {
            '(' => dp += 1,
            ')' => dp -= 1,
            '[' => db += 1,
            ']' => db -= 1,
            '{' => {
                if dp == 0 && db == 0 && dc == 0 {
                    return true;
                }
                dc += 1;
            }
            '}' => dc -= 1,
            _ => {}
        }
    }
    false
}

/// CSS ident 字符判据（ASCII 域＋非 ASCII 域收宽——k109 页面全 ASCII，
/// 非 ASCII ident 收宽为最小保守形）。
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || !c.is_ascii()
}

/// 括号深度扫描取匹配闭括号（chars 游标制）。引号域内括号本层不豁免，
/// FIXME 与调用形扫描同刀收严。
fn matching_close_paren(chars: &[(usize, char)], open: usize) -> Option<usize> {
    let mut d = 0i32;
    for (m, (_, ch)) in chars.iter().enumerate().skip(open) {
        match ch {
            '(' => d += 1,
            ')' => {
                d -= 1;
                if d == 0 {
                    return Some(m);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::Parser;

    const SEED: Option<u64> = Some(7);

    fn registry(css: &str) -> HashMap<String, FunctionRule> {
        collect_custom_functions(std::iter::once(&Parser::parse_stylesheet(css)))
    }

    /// 测试捷径：k112 签名（attr 无读取、random SEED、空调用栈）。
    fn sub(
        value: &str,
        prop: &str,
        reg: &HashMap<String, FunctionRule>,
        customs: &HashMap<String, String>,
    ) -> DashOutcome {
        let attr = |_: &str| None::<String>;
        let ctx = RandomEvalContext {
            element_seed: SEED,
            pct_typed: false,
            font_base_px: None,
        };
        let mut stack = Vec::new();
        let types = HashMap::new();
        let cond = CondEvalCtx {
            media: None,
            container: None,
        };
        substitute_dashed_functions(
            value,
            prop,
            &FnChain::doc(reg),
            customs,
            SEED,
            None,
            0,
            &mut stack,
            &attr,
            &ctx,
            &types,
            &cond,
        )
    }

    fn sub_value(
        value: &str,
        prop: &str,
        reg: &HashMap<String, FunctionRule>,
        customs: &HashMap<String, String>,
    ) -> String {
        match sub(value, prop, reg, customs) {
            DashOutcome::Value(v, _) => v,
            DashOutcome::Cycle | DashOutcome::Invalid => panic!("unexpected non-Value for {value:?}"),
        }
    }

    /// 钉：untyped result＝原文替换（random as-is 留外层属性求值）。
    #[test]
    fn untyped_result_keeps_raw_random() {
        let reg = registry("@function --f() { result: random(--foo property-index-scoped, 1, 1e6); }");
        let out = sub_value("--f()", "--number", &reg, &HashMap::new());
        assert!(
            out.to_ascii_lowercase().contains("random("),
            "raw random preserved; got {out}"
        );
    }

    /// 钉：untyped 两 locals＝var 作用域代入后原文两份（外层属性序数 1/2
    /// 键分异值——④ 断言机制）。
    #[test]
    fn untyped_two_locals_resolve_scope() {
        let reg = registry(
            "@function --locals() { --x: random(--foo property-index-scoped, 1, 1e6); \
             --y: random(--foo property-index-scoped, 1, 1e6); result: var(--x) var(--y); }",
        );
        let out = sub_value("--locals()", "--numbers", &reg, &HashMap::new());
        let n = out.to_ascii_lowercase().matches("random(").count();
        assert_eq!(n, 2, "both locals substituted as raw text; got {out}");
        assert!(
            !out.contains("var("),
            "function-local vars resolved in-station; got {out}"
        );
    }

    /// 钉：typed result＝会话内数值化（键 `{fname}-result`）——同名同键
    /// 恒同值（确定性），异名异键异值（② 断言机制）。
    #[test]
    fn typed_result_numeric_and_fname_scoped() {
        let css = "@function --f-number() returns <number> { result: random(--foo property-index-scoped, 1, 1e6); } \
                   @function --g-number() returns <number> { result: random(--foo property-index-scoped, 1, 1e6); }";
        let reg = registry(css);
        let f1 = sub_value("--f-number()", "--number", &reg, &HashMap::new());
        let f2 = sub_value("--f-number()", "--number", &reg, &HashMap::new());
        let g = sub_value("--g-number()", "--number", &reg, &HashMap::new());
        let v1: f64 = f1.trim().parse().expect("typed result is numeric");
        assert!((1.0..=1e6).contains(&v1), "in range; got {f1}");
        assert_eq!(f1, f2, "same fname same key deterministic");
        assert_ne!(f1, g, "distinct fnames capture distinct keys");
    }

    /// 钉：typed default 参数＝会话内求值（键 `{fname}-参数名` 三连杠形）——
    /// 异函数异值（⑥ 断言机制）。
    #[test]
    fn typed_default_argument_evaluates_per_fname() {
        let css = "@function --f-default-arguments(--x <number>: random(--foo property-index-scoped, 1, 1e6)) { result: var(--x); } \
                   @function --g-default-arguments(--x <number>: random(--foo property-index-scoped, 1, 1e6)) { result: var(--x); }";
        let reg = registry(css);
        let f = sub_value("--f-default-arguments()", "--number", &reg, &HashMap::new());
        let g = sub_value("--g-default-arguments()", "--number", &reg, &HashMap::new());
        f.trim().parse::<f64>().expect("default-arg result numeric");
        assert_ne!(f, g, "default random keyed by fname");
    }

    /// 钉：实参 random＝调用属性语境先替换（烤键同键同值）——异函数同值
    ///（⑧ 断言机制）；嵌套 dashed-function 实参递归。
    #[test]
    fn argument_random_at_call_site_equal_across_fnames() {
        let css = "@function --f-arguments(--x <number>) { result: var(--x); } \
                   @function --g-arguments(--x <number>) { result: var(--x); } \
                   @function --h-inner() returns <number> { result: 42; }";
        let reg = registry(css);
        let mk = |name: &str| {
            let call = format!("{name}(random(--foo property-index-scoped, 1, 1e6))");
            sub_value(&call, "--number", &reg, &HashMap::new())
        };
        let f = mk("--f-arguments");
        let g = mk("--g-arguments");
        let vf: f64 = f.trim().parse().expect("call-site random resolved to number");
        assert!((1.0..=1e6).contains(&vf), "in range; got {f}");
        assert_eq!(f, g, "same call-site key equal across fnames");
        let nested = sub_value("--f-arguments(--h-inner())", "--number", &reg, &HashMap::new());
        assert_eq!(nested.trim(), "42", "nested dashed call substituted in argument");
    }

    /// 钉：local 覆写 typed 参数＝仍按参数类型会话内求值（键 fname-参数名）——
    /// 数值化且异函数异值（⑦ 断言机制）。
    #[test]
    fn local_shadowing_typed_param_stays_registered() {
        let css = "@function --f-locals-arguments(--x <number>) { --x: random(--foo property-index-scoped, 1, 1e6); result: var(--x); } \
                   @function --g-locals-arguments(--x <number>) { --x: random(--foo property-index-scoped, 1, 1e6); result: var(--x); }";
        let reg = registry(css);
        let f = sub_value("--f-locals-arguments(0)", "--number", &reg, &HashMap::new());
        let g = sub_value("--g-locals-arguments(0)", "--number", &reg, &HashMap::new());
        let vf: f64 = f
            .trim()
            .parse()
            .expect("shadowed local evaluated in-session, not raw arg");
        assert!((1.0..=1e6).contains(&vf), "local value in range; got {f}");
        assert_ne!(f, g, "shadowed local keyed by fname");
    }

    /// 钉：typed result var×2＝每引用新鲜序数键（-1/-2）——两位置异值
    ///（⑤ 断言机制，`<number>+` 双位）。
    #[test]
    fn typed_result_fresh_index_per_reference() {
        let reg = registry(
            "@function --loc-reg() returns <number>+ { --x: random(--foo property-index-scoped, 1, 1e6); \
             result: var(--x) var(--x); }",
        );
        let out = sub_value("--loc-reg()", "--numbers", &reg, &HashMap::new());
        let parts: Vec<f64> = out.split_whitespace().filter_map(|s| s.parse().ok()).collect();
        assert_eq!(parts.len(), 2, "two typed positions; got {out}");
        assert_ne!(parts[0], parts[1], "fresh index per reference");
    }

    /// 钉：未注册名＝调用形保留原文（guaranteed-invalid 同向基线）。
    #[test]
    fn unregistered_name_kept_verbatim() {
        let reg = registry("@function --f() { result: 1; }");
        let out = sub_value("var(--x, --nope()) --nope(1)", "--number", &reg, &HashMap::new());
        assert_eq!(out, "var(--x, --nope()) --nope(1)");
    }

    /// 钉（k119 升级）：缺参无 default＝调用求值失败 → `DashOutcome::Invalid`
    ///（消费声明 IACVT——css-mixins-1 §3 invalid at substitution；
    /// dashed-function-eval「Missing only argument」判例期望 guaranteed-
    /// invalid 空值，原调用形保留语义废止）；纯空白实参移除不误判（§3）。
    #[test]
    fn missing_arg_without_default_is_invalid_and_blank_args_dropped() {
        let reg = registry("@function --f-arguments(--x <number>) { result: 1; }");
        let missing = sub("--f-arguments()", "--number", &reg, &HashMap::new());
        assert!(
            matches!(missing, DashOutcome::Invalid),
            "no default → invocation invalid; got {missing:?}"
        );
        let blank = sub_value("--f-arguments( , 42 )", "--number", &reg, &HashMap::new());
        assert_eq!(blank.trim(), "1", "blank arg removed → 42 binds to first param");
    }

    /// 钉（k119 升级）：untyped 参数缺省（无类型无 default）＝调用求值失败
    /// → `DashOutcome::Invalid`。
    #[test]
    fn untyped_param_missing_is_invalid() {
        let reg = registry("@function --f(--x) { result: var(--x); }");
        let out = sub("--f()", "--number", &reg, &HashMap::new());
        assert!(
            matches!(out, DashOutcome::Invalid),
            "missing untyped arg → Invalid; got {out:?}"
        );
    }

    /// 钉（k119）：缺 result 描述符＝调用求值失败 → `DashOutcome::Invalid`
    ///（「Missing result descriptor」判例）。
    #[test]
    fn missing_result_descriptor_is_invalid() {
        let reg = registry("@function --f() { }");
        let out = sub("--f()", "--actual", &reg, &HashMap::new());
        assert!(matches!(out, DashOutcome::Invalid), "no result → Invalid; got {out:?}");
    }

    /// 钉：函数调用环（自引用，k112）——k129 产物重扫环收口为本调用
    /// `Invalid`（css-values-5 §cyclic-substitution-contexts——cyclic
    /// substitution context 使所在声明 invalid；消费声明 IACVT 同向，
    /// 元素级消费侧 fallback 两语义一致，「Dashed-function, self-cycle」
    /// 判据期望 PASS），不挂死。
    #[test]
    fn cyclic_call_contained_as_invalid() {
        let reg = registry("@function --a() { result: --a(); }");
        let out = sub("--a()", "--number", &reg, &HashMap::new());
        assert!(
            matches!(out, DashOutcome::Invalid),
            "self recursion → Invalid; got {out:?}"
        );
    }

    /// 钉（k112 T1）：if() 条件读局部遮蔽值——local --x: 3px 压元素
    /// --x: 1px，条件按局部真。
    #[test]
    fn if_condition_reads_local_shadow() {
        let reg = registry("@function --f() { --x: 3px; result: if(style(--x: 3px): PASS; else: FAIL;); }");
        let customs = m(&[("--x", "1px")]);
        let out = sub_value("--f()", "--actual", &reg, &customs);
        assert_eq!(out.trim(), "PASS", "condition reads local --x; got {out}");
    }

    /// 钉（k112 T2/T5）：条件指定值 var() 局部代入。
    #[test]
    fn if_condition_specified_var_reads_local() {
        let reg = registry("@function --f() { --x: 3px; result: if(style(--y: var(--x)): PASS; else: FAIL;); }");
        let customs = m(&[("--x", "1px"), ("--y", "3px")]);
        let out = sub_value("--f()", "--actual", &reg, &customs);
        assert_eq!(
            out.trim(),
            "PASS",
            "condition specified value substitutes locally; got {out}"
        );
    }

    /// 钉（k112 T3/T6）：支路 var() 局部代入。
    #[test]
    fn if_branch_var_reads_local() {
        let reg = registry("@function --f() { --x: PASS; result: if(style(--true): var(--x); else: FAIL;); }");
        let customs = m(&[("--true", "true"), ("--x", "FAIL")]);
        let out = sub_value("--f()", "--actual", &reg, &customs);
        assert_eq!(out.trim(), "PASS", "branch substitutes local --x; got {out}");
    }

    /// 钉（k112 T9）：local 自指环 → 整局部声明 IACVT（作用域移除）→
    /// result var(--x, PASS) 走回退。
    #[test]
    fn if_cycle_local_var_fallback() {
        let reg =
            registry("@function --f() { --x: if(style(--true): var(--x); else: FAIL;); result: var(--x, PASS); }");
        let customs = m(&[("--true", "true"), ("--x", "FAIL")]);
        let out = sub_value("--f()", "--actual", &reg, &customs);
        assert_eq!(out.trim(), "PASS", "local cycle → IACVT → var fallback; got {out}");
    }

    /// 钉（k112 T10）：条件裸名自读 → 锚名比对 R1 → IACVT → 回退。
    #[test]
    fn if_condition_self_cycle_iacvt() {
        let reg = registry("@function --f() { --x: if(style(--x): FAIL1; else: FAIL2;); result: var(--x, PASS); }");
        let customs = m(&[("--x", "1px")]);
        let out = sub_value("--f()", "--actual", &reg, &customs);
        assert_eq!(out.trim(), "PASS", "condition self-read → CycleOn → IACVT; got {out}");
    }

    /// 钉（k112 T7）：支路内 dashed 调用——会话内 if() 选中支路原文外溢
    /// 产物重扫展开。
    #[test]
    fn dashed_in_if_branch_expands() {
        let reg = registry(
            "@function --f() { result: if(style(--true): --g(); else: FAIL;); } \
             @function --g() { result: PASS; }",
        );
        let customs = m(&[("--true", "true")]);
        let out = sub_value("--f()", "--actual", &reg, &customs);
        assert_eq!(out.trim(), "PASS", "branch --g() expanded by product rescan; got {out}");
    }

    /// 钉（k112 T12）：互调函数环 → 消费声明 IACVT 信号；无环嵌套调用
    /// 不受牵连。k129 环收口语义：检出点（产物重扫/局部展开）环不再整链
    /// 上抛——本形 `--local: --g()` 展开链内层 invalid → 局部落遮蔽标记 →
    /// result var 无 fallback → 调用 invalid＝`Invalid`（消费声明 IACVT
    /// 同向，「Cycle through local, other function」判据族元素级结果
    /// 不变）。
    #[test]
    fn function_recursion_cycle_poisons_declaration() {
        let reg = registry(
            "@function --f() { --local: --g(); result: var(--local); } \
             @function --g() { result: if(style(--true): --f()); }",
        );
        let customs = m(&[("--true", "true")]);
        let out = sub("--f()", "--tmp", &reg, &customs);
        assert!(
            matches!(out, DashOutcome::Invalid),
            "mutual recursion → Invalid; got {out:?}"
        );
        let ok = registry("@function --h() { result: --g2(); } @function --g2() { result: PASS; }");
        let out2 = sub_value("--h()", "--tmp", &ok, &HashMap::new());
        assert_eq!(out2.trim(), "PASS", "acyclic nested call unaffected; got {out2}");
    }

    // ── k129：依赖图环（css-variables-1 §3.4 × dashed-function-cycles 判据页）──

    /// 钉（k129 修 A）：元素级 incoming 环成员预置遮蔽标记——消费侧
    /// fallback 生效（「Observing property cycle locally」判据期望
    /// PASS-x PASS-y；刀前产物＝交替环 Err → 调用 invalid）。
    #[test]
    fn incoming_property_cycle_shadows_with_fallback() {
        let reg = registry("@function --f() { result: var(--x, PASS-x) var(--x, PASS-y); }");
        let customs = m(&[("--x", "var(--y)"), ("--y", "var(--x)")]);
        let out = sub_value("--f()", "--actual", &reg, &customs);
        assert_eq!(out.trim(), "PASS-x PASS-y", "incoming cycle → fallbacks; got {out}");
    }

    /// 钉（k129 修 B）：局部声明互指环（fallback 字面）→ 成员全部 IACVT
    /// ——环命中 fallback 破环通道不适用（「Using cyclic values with no
    /// fallback」判据期望 guaranteed-invalid；刀前产物＝伪值「3 1」）。
    #[test]
    fn local_var_ring_without_fallback_is_invalid() {
        let reg = registry("@function --f() { --y: var(--x, 1); --x: var(--y, 3); result: var(--x) var(--y); }");
        let out = sub("--f()", "--actual", &reg, &HashMap::new());
        assert!(matches!(out, DashOutcome::Invalid), "local ring → Invalid; got {out:?}");
    }

    /// 钉（k129 修 B 守成）：主引用位建边、fallback 子树排除——主链可解
    /// 即非环，fallback 内自指不毒化（「Local with self-cycle in unused
    /// fallback」判据期望 PASS）。
    #[test]
    fn unused_fallback_self_reference_is_not_a_cycle() {
        let reg = registry("@function --f() { --y: PASS; --x: var(--y, var(--x)); result: var(--x, FAIL); }");
        let out = sub_value("--f()", "--actual", &reg, &HashMap::new());
        assert_eq!(out.trim(), "PASS", "fallback-only self ref resolvable; got {out}");
    }

    /// 钉（k129 修 B 守成）：局部自指环成员 IACVT——消费侧 fallback 生效
    ///（「Local with self-cycle」判据期望 PASS）。
    #[test]
    fn local_self_cycle_uses_fallback() {
        let reg = registry("@function --f() { --x: var(--x); result: var(--x, PASS); }");
        let out = sub_value("--f()", "--actual", &reg, &HashMap::new());
        assert_eq!(out.trim(), "PASS", "local self-cycle → fallback; got {out}");
    }

    /// 钉（k129 修 C'）：跨函数局部环——内层调用 invalid 收口 → 局部遮蔽
    /// 标记 → 消费侧 fallback 生效（「Cycle through local, other function,
    /// fallback in function」判据期望 PASS；刀前产物＝Cycle 整链上抛 →
    /// 调用 invalid）。
    #[test]
    fn cross_function_local_cycle_falls_back() {
        let reg =
            registry("@function --f() { --a: --g(); result: var(--a, PASS); } @function --g() { result: var(--a); }");
        let out = sub_value("--f()", "--actual", &reg, &HashMap::new());
        assert_eq!(out.trim(), "PASS", "contained cycle → fallback in caller; got {out}");
    }

    /// 钉（k129 修 C' 守成）：环经未用局部——局部检出点环保持整链传播
    ///（`Cycle`；result 字面不得外泄——「Cycle through unused local」判据
    /// 期望元素级 var fallback PASS，需调用级 invalid 信号）。
    #[test]
    fn cycle_through_unused_local_propagates_cycle() {
        let reg = registry("@function --f() { --unused: --f(); result: FAIL-result; }");
        let out = sub("--f()", "--actual", &reg, &HashMap::new());
        assert!(
            matches!(out, DashOutcome::Cycle),
            "unused-local cycle → Cycle; got {out:?}"
        );
    }

    /// 钉（k129 修 D）：缺省值嵌套本函数调用——栈上再入检出 → 整调用
    /// invalid（「Function in a cycle with its own default」判据期望元素级
    /// var fallback PASS，需调用级 invalid 信号；刀前产物＝缺省字面绑定
    /// 「10px」）。
    #[test]
    fn default_reentry_cycle_invalidates_call() {
        let reg = registry("@function --f(--x, --y: --f(13px)) { result: 10px; }");
        let out = sub("--f(42px)", "--actual", &reg, &HashMap::new());
        assert!(
            matches!(out, DashOutcome::Invalid),
            "default re-entry → Invalid; got {out:?}"
        );
    }

    /// 钉（k129 修 D 守成）：参数缺省 var 互指环照旧互遮蔽（「Cyclic
    /// defaults」判据期望 42px PASS-y PASS-z——无 dashed 调用缺省零扰）。
    #[test]
    fn cyclic_var_defaults_still_shadow() {
        let reg = registry(
            "@function --f(--x, --y: var(--z), --z: var(--y)) { result: var(--x, FAIL) var(--y, PASS-y) var(--z, PASS-z); }",
        );
        let out = sub_value("--f(42px)", "--actual", &reg, &HashMap::new());
        assert_eq!(
            out.trim(),
            "42px PASS-y PASS-z",
            "var-default ring → shadow markers; got {out}"
        );
    }

    /// 钉（k112 S5）：常规属性消费面 → 体内 if() 原文外溢（元素级
    /// 4.76 同门拒，attr-security 红线同向）。
    #[test]
    fn s5_regular_prop_keeps_if_raw() {
        let reg = registry("@function --f() { result: if(style(--true): PASS; else: FAIL;); }");
        let customs = m(&[("--true", "true")]);
        let out = sub_value("--f()", "width", &reg, &customs);
        assert!(
            out.to_ascii_lowercase().contains("if("),
            "regular-property consumption keeps if() raw (S5); got {out}"
        );
    }

    // ── k119 会话作用域重构钉（dashed-function-eval.html 判例族）────────

    /// 钉（k119 M1 穿线）：外层函数局部对嵌套调用可见（动态作用域——
    /// 「Substitute local from outer scope」：局部遮蔽元素同名）。
    #[test]
    fn nested_call_sees_outer_local() {
        let reg = registry(
            "@function --f() { --x: PASS; result: --g(); } \
             @function --g() { result: var(--x); }",
        );
        let customs = m(&[("--x", "FAIL")]);
        let out = sub_value("--f()", "--actual", &reg, &customs);
        assert_eq!(
            out.trim(),
            "PASS",
            "inner call sees caller local over element; got {out}"
        );
    }

    /// 钉（k119 M1 穿线传递）：隔帧局部可见（「Referencing local two frames
    /// up」）；同名函数异帧异值（「Same function with different scopes」）。
    #[test]
    fn scope_threading_transitive_and_frame_isolated() {
        let reg = registry(
            "@function --a() { --x: 1; result: --b(); } \
             @function --b() { result: --c(); } \
             @function --c() { result: var(--x); } \
             @function --one() { --x: 1; result: --f(); } \
             @function --two() { --x: 2; result: --f(); } \
             @function --f() { result: var(--x); }",
        );
        let customs = m(&[("--x", "0")]);
        let up = sub_value("--a()", "--actual", &reg, &customs);
        assert_eq!(up.trim(), "1", "grandparent local visible; got {up}");
        let frames = sub_value("--one() --two() --f()", "--actual", &reg, &customs);
        assert_eq!(frames.trim(), "1 2 0", "per-frame locals distinct; got {frames}");
    }

    /// 钉（k119 M1 已解析值）：内层调用见外层局部**已解析**链值，声明序
    /// 无关（「Inner function call should see resolved outer locals」双序）。
    #[test]
    fn inner_call_sees_resolved_outer_locals_both_orders() {
        let mk = |flip: bool| {
            if flip {
                "@function --a() { --x: var(--px); --y: --b(); result: var(--y); } \
                 @function --b() { result: var(--x, FAIL); }"
            } else {
                "@function --a() { --x: --b(); --y: var(--px); result: var(--x); } \
                 @function --b() { result: var(--y, FAIL); }"
            }
        };
        for (tag, css) in [("fwd", mk(false)), ("rev", mk(true))] {
            let reg = registry(css);
            let customs = m(&[("--px", "10px")]);
            let out = sub_value("--a()", "--actual", &reg, &customs);
            assert_eq!(
                out.trim(),
                "10px",
                "{tag}: inner call sees resolved outer local; got {out}"
            );
        }
    }

    /// 钉（k119 M1/M5 IACVT 局部遮蔽）：无效 var 局部＝遮蔽标记（外层/元素
    /// 同名不回落，回退可用——「IACVT outer local shadows property」）；
    /// 局部 var 链跨函数以原始链值可解（「Referencing outer local containing
    /// var()」——外层局部已解析值不被内层同名局部回染）。
    #[test]
    fn iacvt_local_shadows_and_cross_function_chain() {
        let reg = registry(
            "@function --a() { --x: var(--unknown); result: --b(); } \
             @function --b() { result: var(--x, PASS); } \
             @function --f() { --y: 1; --x: var(--y); result: --g(); } \
             @function --g() { --y: 0; result: var(--x); }",
        );
        let customs = m(&[("--x", "FAIL"), ("--y", "0")]);
        let shadow = sub_value("--a()", "--actual", &reg, &customs);
        assert_eq!(shadow.trim(), "PASS", "IACVT local shadows caller scope; got {shadow}");
        let chain = sub_value("--f()", "--actual", &reg, &customs);
        assert_eq!(chain.trim(), "1", "outer local resolved in own session; got {chain}");
    }

    /// 钉（k119 M2 绑定期解析）：实参 var() 调用点代入——函数体局部不回染
    ///（「var() in argument resolved before call」）；缺省在参数部分作用域
    /// 解析（「Default referencing another parameter, local interference」：
    /// 缺省读参数绑定，局部 --x 不回染缺省）。
    #[test]
    fn binding_time_resolution() {
        let reg = registry(
            "@function --f(--x) { --one: FAIL; result: var(--x); } \
             @function --d(--x, --y: var(--x)) { --x: 17px; result: var(--x) var(--y); }",
        );
        let customs = m(&[("--one", "1px"), ("--x", "FAIL"), ("--y", "FAIL")]);
        let arg = sub_value("--f(calc(100px + var(--one)))", "--actual", &reg, &customs);
        assert_eq!(
            arg.trim(),
            "calc(100px + 1px)",
            "arg var resolved at call site; got {arg}"
        );
        let def = sub_value("--d(5px)", "--actual", &reg, &customs);
        assert_eq!(
            def.trim(),
            "17px 5px",
            "default reads param binding not local shadow; got {def}"
        );
    }

    /// 钉（k119 M2 IACVT 实参兑缺省＋M5 遮蔽）：无效 var 实参→缺省（
    /// 「IACVT arguments are defaulted」）；无缺省→遮蔽标记（回退可用，
    /// 元素同名不回落——「IACVT argument shadows outer scope」）。
    #[test]
    fn iacvt_args_defaulted_and_marker_shadow() {
        let reg = registry(
            "@function --f(--x: 1, --y, --z: 3) { result: var(--x) var(--y) var(--z); } \
             @function --s(--x) { result: var(--x, PASS); }",
        );
        let customs = m(&[("--x", "FAIL")]);
        let defaulted = sub_value("--f(var(--unknown), 2, var(--unknown))", "--actual", &reg, &customs);
        assert_eq!(
            defaulted.trim(),
            "1 2 3",
            "IACVT args fall to defaults; got {defaulted}"
        );
        let marker = sub_value("--s(var(--unknown))", "--actual", &reg, &customs);
        assert_eq!(
            marker.trim(),
            "PASS",
            "IACVT arg shadows element --x, fallback wins; got {marker}"
        );
    }

    /// 钉（k119 M4 块实参解包）：整体单块剥壳（「Passing list as only
    /// argument」）、只剥一层（`{{}}` → `{}`）、非整体块原样。
    #[test]
    fn block_arg_unwrap() {
        let reg = registry("@function --f(--x) { result: var(--x); }");
        let list = sub_value("--f({1px, 2px})", "--actual", &reg, &HashMap::new());
        assert_eq!(list.trim(), "1px, 2px", "block unwrapped to list; got {list}");
        // k120 块产物门：解包后产物仍含顶层块 → 调用 invalid（WPT
        // 「Passing {} as argument」expected ""——消费声明侧同门 IACVT）。
        let nested = sub("--f({{}})", "--actual", &reg, &HashMap::new());
        assert!(
            matches!(nested, DashOutcome::Invalid),
            "top-level block product → Invalid; got {nested:?}"
        );
        let partial = sub("--f({a}{b})", "--actual", &reg, &HashMap::new());
        assert!(
            matches!(partial, DashOutcome::Invalid),
            "non-whole blocks product → Invalid; got {partial:?}"
        );
    }

    /// 钉（k119 M7 关键字局部语义，§2.3.1）：initial 移除局部回落参数
    ///（「Local variable with initial keyword」）；inherit 取调用方作用域值
    ///（顶层＝元素、嵌套＝caller 局部）；unset 族 → IACVT 遮蔽走回退；
    /// 调用产物关键字同语义（「Keyword can be returned from function into
    /// local variable」）。
    #[test]
    fn css_wide_keyword_local_semantics() {
        let reg = registry(
            "@function --i(--x: FAIL1) { --x: FAIL2; --x: initial; result: var(--x); } \
             @function --h(--x: FAIL1) { --x: FAIL2; --x: inherit; result: var(--x); } \
             @function --hg(--x) { --x: PASS; result: --h(FAIL3); } \
             @function --u() { --x: unset; result: var(--x, PASS); } \
             @function --r() { result: initial; } \
             @function --g(--x: PASS) { --x: FAIL1; --x: --r(); result: var(--x, FAIL2); }",
        );
        let customs = m(&[("--x", "PASS")]);
        let initial = sub_value("--i(PASS)", "--actual", &reg, &customs);
        assert_eq!(
            initial.trim(),
            "PASS",
            "initial removes local, param resurfaces; got {initial}"
        );
        let inherit_top = sub_value("--h(FAIL3)", "--actual", &reg, &customs);
        assert_eq!(
            inherit_top.trim(),
            "PASS",
            "inherit reads element value; got {inherit_top}"
        );
        let inherit_nested = sub_value("--hg(FAIL4)", "--actual", &reg, &customs);
        assert_eq!(
            inherit_nested.trim(),
            "PASS",
            "nested inherit reads caller local; got {inherit_nested}"
        );
        let unset = sub_value("--u()", "--actual", &reg, &HashMap::new());
        assert_eq!(
            unset.trim(),
            "PASS",
            "unset local → IACVT shadow, fallback; got {unset}"
        );
        let returned = sub_value("--g()", "--actual", &reg, &HashMap::new());
        assert_eq!(
            returned.trim(),
            "PASS",
            "keyword via call product triggers local semantics; got {returned}"
        );
    }

    /// 钉（k119 M7 关键字缺省语义）：initial 缺省 → IACVT 遮蔽走回退；
    /// inherit 缺省 → 调用方作用域取值（「Default with inherit keyword」
    /// ——空参与显式参对照）。
    #[test]
    fn css_wide_keyword_default_semantics() {
        let reg = registry(
            "@function --di(--x: initial) { result: var(--x, PASS); } \
             @function --dh(--x: inherit) { result: var(--x); }",
        );
        let customs = m(&[("--x", "PASS1")]);
        let di = sub_value("--di()", "--actual", &reg, &HashMap::new());
        assert_eq!(di.trim(), "PASS", "initial default → IACVT → fallback; got {di}");
        let dh1 = sub_value("--dh()", "--actual", &reg, &customs);
        let dh2 = sub_value("--dh(PASS2)", "--actual", &reg, &customs);
        assert_eq!(dh1.trim(), "PASS1", "inherit default reads caller scope; got {dh1}");
        assert_eq!(dh2.trim(), "PASS2", "explicit arg beats inherit default; got {dh2}");
    }

    fn m(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    // ── k120 typed 求值域钉（css-mixins-1 §3.1 类型强制）────────────────

    /// 钉（k120）：typed result calc 化简＋px canonical（「Literal result,
    /// typed return, calc」：calc(12px + 1px) → 13px）。
    #[test]
    fn typed_result_calc_simplifies() {
        let reg = registry("@function --f() returns <length> { result: calc(12px + 1px); }");
        let out = sub_value("--f()", "--actual", &reg, &HashMap::new());
        assert_eq!(out.trim(), "13px", "typed result calc simplified; got {out}");
    }

    /// 钉（k120）：typed result 类型不符 → 调用 invalid（「Literal result,
    /// typed return, mismatch」：12s 于 <length> → guaranteed-invalid）。
    #[test]
    fn typed_result_mismatch_is_invalid() {
        let reg = registry("@function --f() returns <length> { result: 12s; }");
        let out = sub("--f()", "--actual", &reg, &HashMap::new());
        assert!(
            matches!(out, DashOutcome::Invalid),
            "12s on <length> → Invalid; got {out:?}"
        );
    }

    /// 钉（k120）：typed 实参 calc 化简（「Typed parameter with calc()」）。
    #[test]
    fn typed_arg_calc_simplifies() {
        let reg = registry("@function --f(--x <length>) { result: var(--x); }");
        let out = sub_value("--f(calc(100px + 1px))", "--actual", &reg, &HashMap::new());
        assert_eq!(out.trim(), "101px", "typed arg calc simplified; got {out}");
    }

    /// 钉（k120）：typed 多参数单位归一（「Various typed parameters」：
    /// px 整数化＋turn→deg＋ms→s canonical）。
    #[test]
    fn typed_args_canonical_units() {
        let reg =
            registry("@function --f(--x <length>, --y <angle>, --z <time>) { result: var(--x) var(--y) var(--z); }");
        let out = sub_value(
            "--f(calc(100px + 1px), 1turn, 1000ms)",
            "--actual",
            &reg,
            &HashMap::new(),
        );
        assert_eq!(out.trim(), "101px 360deg 1s", "units canonicalized; got {out}");
    }

    /// 钉（k120）：typed 实参 var 代入后化简（「var() in argument resolved
    /// before call, typed」——绑定期调用点代入，函数体局部不回染）。
    #[test]
    fn typed_arg_var_resolved_then_simplified() {
        let reg = registry("@function --f(--x <length>) { --one: FAIL; result: var(--x); }");
        let customs = m(&[("--one", "1px")]);
        let out = sub_value("--f(calc(100px + var(--one)))", "--actual", &reg, &customs);
        assert_eq!(
            out.trim(),
            "101px",
            "arg var resolved at call site then simplified; got {out}"
        );
    }

    /// 钉（k120）：typed 实参类型不符兑缺省（「Arguments are defaulted on
    /// type mismatch」：red/360deg 失败 → 缺省 1/3，合法 2 透传）。
    #[test]
    fn typed_arg_mismatch_falls_to_default() {
        let reg = registry(
            "@function --f(--x <number>: 1, --y <number>, --z <number>: 3) { result: var(--x) var(--y) var(--z); }",
        );
        let out = sub_value("--f(red, 2, 360deg)", "--actual", &reg, &HashMap::new());
        assert_eq!(out.trim(), "1 2 3", "mismatched args defaulted; got {out}");
    }

    /// 钉（k120）：typed 缺省带引用化简（「Typed default with reference」：
    /// calc(var(--x) + 1px) 读参数绑定 5px → 6px）。
    #[test]
    fn typed_default_with_reference_simplifies() {
        let reg =
            registry("@function --f(--x: 5px, --y <length>: calc(var(--x) + 1px)) { result: var(--x) var(--y); }");
        let customs = m(&[("--x", "FAIL"), ("--y", "FAIL")]);
        let out = sub_value("--f()", "--actual", &reg, &customs);
        assert_eq!(
            out.trim(),
            "5px 6px",
            "typed default simplified against param binding; got {out}"
        );
    }

    /// 钉（k120）：typed 实参失败 IACVT 遮蔽＋回退可用（「Argument captures
    /// IACVT due to type mismatch」与「IACVT argument shadows outer scope,
    /// type mismatch」——元素 --x: FAIL 在场不得回落，双形对照）。
    #[test]
    fn typed_arg_mismatch_iacvt_and_shadow() {
        let reg = registry("@function --f(--x <length>) { result: var(--x, PASS); }");
        let no_outer = sub_value("--f(red)", "--actual", &reg, &HashMap::new());
        assert_eq!(no_outer.trim(), "PASS", "mismatch → IACVT → fallback; got {no_outer}");
        let customs = m(&[("--x", "FAIL")]);
        let shadow = sub_value("--f(red)", "--actual", &reg, &customs);
        assert_eq!(
            shadow.trim(),
            "PASS",
            "mismatch IACVT shadows outer element --x; got {shadow}"
        );
    }

    /// 钉（k120）：typed result 拒 CSS 宽关键字（「Can not return CSS-wide
    /// keyword as length」：revert-layer 于 <length> → invalid → fallback）。
    #[test]
    fn typed_result_rejects_css_wide_keyword() {
        let reg = registry("@function --f() returns <length> { result: revert-layer; }");
        let out = sub("--f()", "--actual", &reg, &HashMap::new());
        assert!(
            matches!(out, DashOutcome::Invalid),
            "CSS-wide keyword on <length> → Invalid; got {out:?}"
        );
    }

    /// 钉（k120）：typed 局部求值失败 → IACVT 遮蔽走回退，不整调用
    /// invalid（「Invalid value for typed local becomes IACVT」：--c: 3
    /// 于 <color> → var(--c, PASS) → PASS）。
    #[test]
    fn typed_local_invalid_becomes_iacvt() {
        let reg = registry("@function --f(--c <color>) { --c: 3; result: var(--c, PASS); }");
        let out = sub_value("--f(#f00)", "--actual", &reg, &HashMap::new());
        assert_eq!(out.trim(), "PASS", "typed local mismatch → IACVT → fallback; got {out}");
    }

    /// 钉（k120 零漂移组）：plain token canonical 恒同（100px/1）、type(*)
    /// 万能型豁免（calc 原样透传——「Untyped parameter with calc()」）。
    #[test]
    fn typed_zero_drift_passthrough() {
        let reg = registry(
            "@function --fp(--x <length>) { result: var(--x); } \
             @function --fn2(--x <number>) { result: var(--x); } \
             @function --fu(--x type(*)) { result: var(--x); }",
        );
        let px = sub_value("--fp(100px)", "--actual", &reg, &HashMap::new());
        assert_eq!(px.trim(), "100px", "plain px passthrough; got {px}");
        let num = sub_value("--fn2(1)", "--actual", &reg, &HashMap::new());
        assert_eq!(num.trim(), "1", "plain number passthrough; got {num}");
        let untyped = sub_value("--fu(calc(100px + 1px))", "--actual", &reg, &HashMap::new());
        assert_eq!(
            untyped.trim(),
            "calc(100px + 1px)",
            "type(*) calc verbatim; got {untyped}"
        );
    }

    /// k128：`@layer` 块内 `@function` 收集＋层序裁决矩阵（css-mixins-1
    /// §2.1.1 stronger cascade layer wins＋同层后定义胜；driving:
    /// function-layer.html 判据 7 形同构）。unlayered（u32::MAX）强于所有
    /// 层，匿名层按出现序占 rank。
    #[test]
    fn collect_layer_rank_arbitration() {
        // 匿名单层：层内定义收集生效（刀前只扫顶层＝函数丢失）。
        let reg = registry("@layer { @function --f() { result: 1px; } }");
        assert_eq!(reg["--f"].body.len(), 1, "anonymous-layer fn collected");

        // 匿名两块同名：后块胜（rank 1 > 0——「Last anonymous layer wins」）。
        let reg = registry(
            "@layer { @function --f() { result: 1px; } } \
             @layer { @function --f() { result: 2px; } }",
        );
        let r = sub_value("--f()", "--actual", &reg, &HashMap::new());
        assert_eq!(r.trim(), "2px", "later anonymous layer wins; got {r}");

        // 顶层胜匿名层（两序——unlayered MAX 强于所有 rank，与文档序无关）。
        let reg = registry(
            "@layer { @function --f() { result: 1px; } } \
             @function --f() { result: 3px; }",
        );
        let r = sub_value("--f()", "--actual", &reg, &HashMap::new());
        assert_eq!(r.trim(), "3px", "unlayered wins (decl after); got {r}");
        let reg = registry(
            "@function --f() { result: 3px; } \
             @layer { @function --f() { result: 2px; } }",
        );
        let r = sub_value("--f()", "--actual", &reg, &HashMap::new());
        assert_eq!(r.trim(), "3px", "unlayered wins (decl before); got {r}");

        // 命名层文档序：后块层更强（base 后 theme → theme 胜）。
        let reg = registry(
            "@layer base { @function --f() { result: 10px; } } \
             @layer theme { @function --f() { result: 20px; } }",
        );
        let r = sub_value("--f()", "--actual", &reg, &HashMap::new());
        assert_eq!(r.trim(), "20px", "later named layer wins; got {r}");

        // 预声明序：`@layer theme, base;` 使 base 首现序更强（文档序 base 块
        // 在前仍胜——「Named layers, reordered」）。
        let reg = registry(
            "@layer theme, base; \
             @layer base { @function --f() { result: 10px; } } \
             @layer theme { @function --f() { result: 20px; } }",
        );
        let r = sub_value("--f()", "--actual", &reg, &HashMap::new());
        assert_eq!(r.trim(), "10px", "predeclared layer order wins; got {r}");

        // @media 等条件块不递归（@property 同型——全局注册无作用域）。
        let reg = registry("@media screen { @function --g() { result: 1px; } }");
        assert!(!reg.contains_key("--g"), "conditional block not collected");

        // 跨表：后表 unlayered 胜前表层内定义（rank 全局裁决）。
        let s1 = Parser::parse_stylesheet("@layer { @function --f() { result: 1px; } }");
        let s2 = Parser::parse_stylesheet("@function --f() { result: 9px; }");
        let reg = collect_custom_functions([&s1, &s2]);
        let r = sub_value("--f()", "--actual", &reg, &HashMap::new());
        assert_eq!(r.trim(), "9px", "later sheet unlayered wins; got {r}");
    }
}

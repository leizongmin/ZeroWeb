# HTML Semantics 兼容 — html/semantics 语义面（主攻大域）

**版本**: v2.0
**日期**: 2026-10-21（v1.0 立项 2026-10-08）
**状态**: Active——M2 切片 20 收口（details/summary 深层面 R4009 +18P），下一步 M2 切片 21：details name= 互斥簇（name-attribute.html 18F + file Timeout，interactive-elements 最大剩余杠杆）
**执行模式**: WPT 驱动（上游 html/semantics corpus 为验收标尺）+ 语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——HTML 语义面）

> **▶ 当前状态（2026-10-21 M2 切片 20）**：details/summary 深层面（R4009，
> rollover 接续——代码为上会话 429 中断遗留 WIP，本会话 rebase 后续推，
> 宿主路径双 toggle 缺陷被集成测试证伪后重设计收口）。四层：toggle 事件
> 任务队列（part03 `_zwDetailsToggleQ4009`——per-element 去重/同任务合并/
> 净零仍派/同态不排程/detached 派发；单一 choke point 落 part04 `open`
> IDL setter + attr 尾钩，页面 JS 与 plan script_set_open 同经；trusted +
> ToggleEvent 原型身份）+ summary 激活行为（part03 bubble 站钩后代 target
> 面 + part04 click() post 分支本体面，首个 summary 元素子身份比对，交互
> 豁免含祖先面）+ ToggleEvent 接口（part05 `_defineEventSubclass`）+ 宿主
> 空格路由（webview SUMMARY → plan_summary）。双 toggle 修复：plan_summary
> 移除 followup toggle（page-runtime），发射权统一归队列——集成断言
> `click:false,toggle:true,click:true` 维持不变，事件形状由同步裸 Event
> 近似转正为 spec 口径。part60 T1–T9 契约。九域 4855→**4873P**（66.09%），
> 对 s19 逐 subtest diff **净 +18P、回归 0**，全部落 interactive-elements
> （77→95P）：toggleEvent.html 1P/2F+未上报→9P/0F、activation-behavior
> 2P/7F→9P/0F、anchor-without-link 0P/2F→2P/0F、details-keyboard
> -activation 0P/1F→1P/0F。gates：zero-engine --lib 3358P/0F、page-runtime
> lib 136P/0F、webview lib 785P/0F、renderer activation 9P/0F、integration
> html_compat 21P/0F、clippy workspace 过、fmt 本切片文件 clean、零新增
> 依赖。账本见
> [evidence/2026-10-21-m2s20-r4009.md](html-semantics-compat/evidence/2026-10-21-m2s20-r4009.md)。
> **下一步**：M2 切片 21——details name= 属性互斥簇
> （name-attribute.html 18F + file Timeout，单文件占 interactive-elements
> 剩余 264F 的 19）；余挂账 toggleEvent 尾部 ontoggle 面、
> ToggleEvent.source/commandfor、anchor-with-inline 导航缺口（导航域）、
> parser form element pointer 4F 同根维持单独立项征询不越界。


> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：继
> net-api（79.7% 收口）之后的下一个"整域收割"级主攻——html/semantics 覆盖
> forms/embedded-content/tabular-data 等真实站点核心语义面；form-validation 已
> 打过样（monthly 高亮项），全语义面从未系统圈过。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：表单校验/控件状态面有存量
> （form-validation + html-syntax M 系遗产）；各子域（forms 约束校验/控件状态/
> 提交语义、tabular-data 表格模型、embedded-content 媒体与 iframe 元素属性等）
> 通过率未知——M1 分域基线定形。
>
> **▶ 域边界（先例：R4114-N 深结构归档条款）**：本 goal 只做 **DOM/语义/API 面**
> （元素属性/IDL 接口/约束/状态机）；布局渲染差异（table 布局、multicol、控件
> 视觉）归 rendering-compat。已归档 html-compat 的余账关系 M1 盘点核对防重复
> 计账。

## Mission

以 WPT html/semantics corpus 分域为验收标准，按子域切片推进（建议序：forms
约束与控件状态 → tabular-data → document-metadata/grouping/text-level →
embedded-content → links/microdata），每域独立收口出数字。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **分域收口制**：一个子域一个数字，不设总通过率门禁（corpus 太大）
3. **轻量修复优先**：约束校验/IDL 面走 shim；触 DOM parser 核心的深结构改动
   单独立项征询

覆盖范围（十个初始子域，fetch 脚本 DIRS 同清单）：forms（最大，含深层子目录
按 web-animations 先例追加）/ embedded-content / document-metadata /
grouping-content / text-level-semantics / tabular-data / interactive-elements /
scripting-1 / links / microdata。

### 排除（明确不在范围内）

- **table/multicol 等布局渲染差异** — rendering-compat 域
- **iframe 浏览上下文树** — navigation-compat M3 门控域
- **媒体元素播放语义** — media 系已归档 goal 域（embedded-content 中的媒体
  元素属性面归本 goal，播放管线不碰）
- **ARIA/无障碍映射** — 未立项域，挂账

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | 元素 IDL 面/约束校验/状态机语义 | part 系 JS 语义 |
| dom/style-system | 属性解析与反射（不碰 parser 核心与布局） | 轻量优先 |
| WPT 资产 | html/semantics 十子域导入（分批） | fetch 脚本 + 账本回填 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 rendering-compat**：布局/绘制差异记账回流（table 布局是其活跃工作面），
  本 goal 不越界修渲染
- **与 navigation-compat M3**：embedded-content 的 iframe 面只做元素属性，
  frame 树语义挂其门控
- **与 css-parser 流（本会话 fuzz 修复侧）**：CSS 解析面零交集

---

## Done Criteria

- **DC-1** 十子域导入 + 分域基线 + csv planned 行回填（每域独立数据行）
- **DC-2** 逐域收口：forms / tabular-data / document-metadata / grouping /
  text-level / interactive-elements / scripting-1 / links / microdata /
  embedded-content 各自"分级通过率不再下行两个连续轮次 + 余案归因挂账"
- **DC-3** 回归测试账本纪律（imported-tests.txt）；make test 全绿、make reftest
  零回归、product-smoke 逐字节恒值

## 里程碑

M1 导入与分域基线（✅ 2026-10-15）→ M2 forms 域（最大杠杆，多切片）→
M3 其余九域依序推进 → M4 总收口归档。

## 轮次账

- **2026-10-21 M2 切片 20**：details/summary 深层面（R4009，rollover 接续
  ——上会话 429 中断遗留 WIP 本会话续推收口）——toggle 事件任务队列
  （part03：去重/合并/净零/同态不排程/detached；choke point part04 `open`
  IDL setter + attr 尾钩）+ summary 激活行为（part03 bubble 站钩 + part04
  post 分支，首个 summary 元素子 + 交互豁免含祖先面）+ ToggleEvent 接口
  （part05）+ 宿主空格路由（webview SUMMARY → plan_summary）。**宿主双
  toggle 缺陷重设计**：plan_summary followup toggle 与队列双派被集成测试
  证伪（`toggle:true,toggle:true`），移除 followup 发射权归队列，断言
  `click:false,toggle:true,click:true` 不变、事件形状转正。part60 T1–T9。
  九域 4855→4873P（66.09%），对 s19 逐 subtest diff 净 +18P、回归 0，
  全落 interactive-elements（77→95P）：toggleEvent 9P/0F、
  activation-behavior 9P/0F、anchor-without-link 2P/0F、
  details-keyboard-activation 1P/0F。zero-engine --lib 3358P/0F、
  page-runtime 136P/0F、webview 785P/0F、renderer activation 9P/0F、
  integration html_compat 21P/0F、clippy/fmt 过、零新增依赖。挂账：
  details name= 互斥簇 18F+Timeout（下一切片首选）、toggleEvent 尾部
  ontoggle 内容属性面、ToggleEvent.source/commandfor、anchor-with-inline
  synthetic click 导航缺口（导航域）、parser form pointer 4F、
  navigation-compat M3 项维持。
- **2026-10-21 M2 切片 19**：label 本体激活转发（R4008，rollover 双触发
  两会话同树协作合账，commit 88793f87d）——shim 交互豁免身份比较（bubble
  站挂钩 + R155 post 段，target 即 label 本体不豁免）+ 宿主面上移（renderer
  activate_label_at 既有 + webview user_actions plan 链递归 Activate，
  labelable 过滤防环/取消抑制/disabled noop）；shim 不承接宿主 generic
  Event（首发 shim finally 段转发被 renderer 双激活集成测试证伪后重设计，
  checkbox 双翻）。part60 W1–W6 重写 + integration 三面新测。九域
  4848→4855P（65.91%），对 s18 逐 subtest diff 净 +7P、回归 0：
  proxy-modifier tentative 4F 全翻绿（主面）、label-forwarded-click-pointer
  -properties 未上报→2P/0F（isTrusted trusted 面随宿主派发通道翻绿）、
  simulated-click-inert 翻绿。zero-engine --lib 3355P/0F、webview lib
  785P/0F、renderer activation OK、integration html_compat 21P/0F、
  clippy/fmt 过、零新增依赖。协作分工：对侧会话代码提交 + 全量 corpus，
  本会话 kill 冗余全量轮 + docs 合账 + 账本独立复算。挂账：parser form
  element pointer 表规则 4F 同根（深结构单独立项）、details 键盘激活 /
  summary toggle coalescing、navigation-compat M3 项维持。
- **2026-10-21 M2 切片 18**：label.form IDL（R4007，rollover 接续）——
  `.form` getter 分发区补 LABEL 分支，`_zwLabeledControl` 取控件后委托
  控件 `.form`（R4005 关联面全继承），label 自身 form 属性不参与。
  part60 契约 1 测（W1–W3）。九域 4838→4848P（65.82%，账本条目口径），
  对 s17 逐 subtest diff 净 +10P、回归 0：form.html label.form 8 案
  翻绿（7P/10F→15P/2F）、dialog autofocus 两案意外翻绿（焦点链联带）。
  zero-engine --lib 3325P/0F、make test 76 腿 22222P/0F 全绿、clippy/fmt
  过、零新增依赖。挂账：parser form element pointer 表规则 4F 同根
  （深结构单独立项）、proxy-modifier tentative 4F、details 键盘激活 /
  summary toggle coalescing、navigation-compat M3 项维持。
- **2026-10-21 M2 切片 17**：form owner 动态关联（R4005/R4006，rollover
  接续）——`.form` getter 重写（JS 视图 parentNode 三族统一上行 + form
  attr 控件所在树查找 + latest-wins 读 + OBJECT gate）、R52 消零 purge
  WeakRef 复活（identity 断修复、泄漏口径不变）、gEBI pending 窗口同 id
  树序取首 + `_r125Overridden` 改名门。part60 契约 1 测（W1–W4）。九域
  4774→4838P（65.90%），对 s16 逐 subtest diff 净 +64P、回归 0：
  form_attribute.html 43P/57F→100P/0F 满绿、association.window.js 翻绿、
  object.form/form-elements-filter +1+1、select-tab* 意外翻绿 2、
  load-error-events 上报展开 +2。zero-engine --lib 3310P/0F、make test
  全腿绿（GPU adapter 94P/2F 容差漂移结构不阻塞非本轮、归 rendering 流
  观察账）、clippy/fmt 过、零新增依赖。全量轮 5400s 限被兄弟流并行负载
  顶爆、10800s 放行记 evidence。挂账：label.form IDL（form.html 余 10F，
  下一切片）、form_owner_and_table parser 指针表规则 2F（深结构）、
  proxy-modifier 4F、details 键盘激活 / summary toggle coalescing、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 16**：label 激活行为面（R4004）——交互内容判定
  存在性走 has 句柄（get_attr_handle 缺失返空串致恒真误判）、R368 盖章
  属性回填、label 激活转发三路归一（bubble 站挂钩新设 + post/trap 迁移
  `_zwLabeledControl` for 感知 + 交互 target 豁免补齐 + 双闸互斥收敛
  自转发死循环与 post+trap 双转）+ plainParent hop/R11 媒体补链/DFS 控件
  查找/盖章子事件方法转 live。part60 契约 1 测。九域 4710→4774P，对 s15
  逐 subtest diff 净 +64P、回归 0：clicking-noninteractive-* 57F 全数
  翻绿、意外翻绿 7 案（label-inside-anchor/proxy-click Timeout→Pass、
  dialog 系 5 案含 inert-label-focus 观察账消账）。zero-engine --lib
  3289P/0F、make test 主矩阵绿 + browser bin 腿既有失败 stash 对照定谳
  非本轮、clippy/fmt 过、零新增依赖。make test 与 clippy 并行触发 guard
  总内存超限（8GB 预算下编译峰值 ~8.8GB，16GB 放行）记 evidence。挂账：
  form_attribute 动态关联 ~50F（下一切片）、proxy-modifier tentative 4F、
  details 键盘激活 / summary toggle coalescing（M1 起既有）、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 15**：labels IDL 簇 + label.focus() 前递
  （R4003/R4003b/R4003c，rollover 双触发两会话同树合记）——part03
  `_liveQueryCollection` 泛化（自定义快照 + NodeList 原型 target +
  `__zwNL` 印记透读）、part04 labeled control 算法 + labels 反查活集合
  （[SameObject] per-element-key 缓存居 part01 共享作用域）+ focus 前递
  + body/html offsetParent→null。part60 契约 3 测。九域 4683→4710P
  （64.00%），对 s14 逐 subtest diff 净 +27P、回归 0（labelable-elements
  +21、button/input-labels +1+1、forward-focus +4；inert-label-focus
  失败模式漂移 +0，run2 二进制未含 R4003c）。zero-engine --lib
  3273P/0F、make test 58 腿绿 + browser bin 腿 xvfb 复跑 665P/0F、
  clippy/fmt 过、零新增依赖。并发碰头信号（共享 target 指纹 E0425、
  run kill 史 ×4、06:43 一次死因未考证）记 evidence。挂账：
  clicking-noninteractive-* 57F（下一切片首选）、form_attribute 动态
  关联 ~50F、proxy-modifier 4F、inert-label-focus 负载超时观察账、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 14**：meter 实际值算法 + double setter（R4002）——
  part04 `_zwMeterActualValues`（六 IDL getter 缺省/钳制 + 浮点整串解析，
  前导 + 跳过为语料钉死行为；R5009 片 e 旧前缀解析修正）+ 六 setter
  WebIDL double 转换（非有限 TypeError）+ best representation 写属性 +
  METER 移出 value 纯反射族（part03 host-read 委托支/part05 tag-gate
  不变）。part60 契约 1 测（W1–W5）。meter 簇 14P/48F→62P/0F，forms
  3688→3736，九域 4662→4683P（63.48%），对 s13 逐 subtest diff 净 +21P。
  **跨流窗口 −27P 对照实验归因、meter 排除**：B2 提交树 16 变更文件复跑
  与 B1 全量账本持平（账本绑定提交树）；无 meter 对照重建原样复现；
  scripting-1 −24 主假设归 script-sandbox 模块缓存三部曲（8458c9223/
  a85c7ade4——安静机恒 Timeout 而 s13 负载下 18P，方向与边界漂移相悖）、
  interactive-elements −3 签名匹配 s7 runner 通道 realm 家族（Fail
  "wrong global"）与窗口候选 3ccfdc479/b2fde4965/368ce9158 并存两说，
  commit 级 bisect 因旧 commit v8 源码构建指纹受阻留属主流。本切片由
  rollover 双触发并发会话同树推进，记账已合并（见切片账本说明）。make
  test 59 腿 58 绿 + 1 既有无 display 环境腿（xvfb 同二进制 665P/0F）、
  clippy/fmt 过、零新增依赖。挂账：labels IDL ~30F（下一切片）、
  form_attribute 动态关联 ~50F（中深）、clicking-noninteractive-* 57F、
  module/dialog 跨流碰头两项（属主流核账）、navigation-compat M3 项维持。
- **2026-10-19 M2 切片 13**：selected autocomplete value 算法 + HTMLCollection
  named access（R4001）——part04 category 全表 + token 算法（INPUT/SELECT/
  TEXTAREA getter，FORM/其余保持枚举反射；field token 小写为 corpus+Chrome
  67/67 钉死行为）、part05 setter 缓存旁路、part03 `_liveQueryCollection`
  named property 补齐（`document.forms.X` 可达）、part03 `.form` 表补
  FIELDSET（named access 修复暴露 vacuous-pass 真值化）。part60 契约
  W1–W5；part15 R3204 input 缺省断言 'on'→''（spec 修正）。九域
  4581→4662P（63.20%），对 s12 逐 subtest diff 净 +81P、回归 0（−11 条
  上报漂移全为 runner 超时边界，P 零损失）。zero-engine --lib 3245 全过、
  make test 全腿绿、clippy/fmt 过、零新增依赖。挂账：meter 净化 44F
  （下一切片）、labels IDL ~30F、form_attribute 动态关联 ~50F（中深）、
  encoding 三簇/double-submit 归 navigation-compat M3 维持、R3204 类
  既有契约钉旧行为风险记入（动枚举反射分发区先跑 part10/part15）。
- **2026-10-19 M2 切片 12**：populateForm 同 turn 二次插入 sibling 断链
  修复（R4000）——`_zwPendingParsedForSel` 候选收集 `candCount > 1`
  提前 return 缺陷删除（签名盖章截断致第二批 pending 节点 hits 恒 0，
  兄弟导航断链；歧义保守语义由 hits 唯一性判定承担不变）。part59 契约
  1 测（W11）。解锁切片 11 flag 契约：form-submission-algorithm t2/t3
  Fail→Pass，九域 4579→4581P（62.10%），对 s11 逐 subtest diff 净
  +2P、回归 0（账本绑定 rebase 后合并树，对 rebase 前复跑 0 变化）。
  zero-engine --lib 3224 全过、clippy/fmt 过、browser bin 腿 :102
  665P/0F、quickjs 矩阵腿全绿、零新增依赖。挂账：
  form-submission 余 F 归 iframe 导航基建簇（navigation-compat M3）、
  既有小挂账维持、二批插入后 body.childNodes 融合视图丢 base 尾节点
  （corpus 无消费面，观察项）。
- **2026-10-19 M2 切片 11**：firing submission events flag + dialog
  returnValue WebIDL 化 + 宿主 click 坐标注入（R3999）——每 form 粒度
  `__zwFiringSubmissionEvents`（spec 步 5.1/5.2/5.4/5.7，入口早退 +
  窗口置位 + finally 清位，仅激活/requestSubmit 通道）；returnValue
  setter DOMString 转换（null→"null"，undefined→"" 兼容例外）；泛型
  click offsetX/offsetY 派发期注入（与 image selected coordinates 同
  口径，事件类保持泛型不改 R108 面；并行 regression-guard 流协作归并，
  账本排查记录 3）。part59 契约 3 测 + part14 期望修正。九域
  4577→4579P（62.07%），对 s10 逐 subtest diff 净 +2P、回归 0：
  dialog-return-value 翻绿、dialog-form-submission 8/8。zero-engine
  --lib 3193 全过、make test 全腿绿（browser 腿 :102 复跑 664P/0F）、
  clippy/fmt 过、零新增依赖。挂账：populateForm 同 turn 二次插入 sibling 断链
  （R3254-K3 面，挡 form-submission-algorithm t2/t3，flag 契约已就位）、
  既有小挂账维持。
- **2026-10-18 M2 切片 10**：form 提交 construct-entry-list 对齐 + dialog
  提交分支（R3910）——constructing entry list flag 共享（submit 入口
  早退先于 submit 事件、构造窗口置位/finally 清位）、formdata 事件共享
  条目存储派发（handler 增删改写提交条目）、action 解析按 spec 顺序后移
  （旧位序丢 formdata/dialog 面）；method=dialog 分支（无祖先 return、
  image 坐标、optional value、_zwDialogClose）；close-the-dialog 步 1
  无 open 早退（四处共享路径）；SubmitEvent/FormDataEvent 构造器
  WebIDL 化。part59 契约 6 测。forms 3602→3605P、dialog-element
  38→47P、九域 4565→4577P（62.05%），对 s9 账本逐 subtest diff 净
  +12P、回归 0。zero-engine --lib 3169 全过、make test 全腿绿
  （browser 腿 :102 复跑）、clippy/fmt 过、零新增依赖。挂账：
  form-double-submit-requestsubmit 36F→Timeout（iframe 导航基建，归
  navigation-compat M3）、submit-dialog-close-event 挂起维持、
  firing submission events flag（t2-t4）、image 坐标 offsetX 面（t5）、
  dialog-return-value returnValue setter String 强转 1F。
- **2026-10-18 M2 切片 9**：selectedness/selectedIndex 语义（R3909）——
  selectedness setting（受影响者胜 affKey + size 1 非 disabled 回落）、
  ask-for-a-reset 汇流钩（`_mo_notify` childList 触发、批插依序、innerHTML
  容器合流、Blink 重回落）、selected 属性变更步两半、selectedIndex setter
  （可全不选）、select.value setter 双形态真态写（旧 guard 双重排除丢写面
  根因修复）、form.reset 复位算法、select.remove(n)/add 祖先检查 + 元素代理
  add、select value-missing（placeholder 四条件）、Option() 构造器
  （属性只随 defaultSelected + clean 层）；读侧 selectedIndex/value/视图
  全改 JS 真态，pristine-only size 1 跳 disabled 回落。part59 契约 12 测。
  select-value-selectedOption +8、selected-index 13/13、select-validity
  6/6 等**净 +34P**；forms 3569→3602P、九域 4531→4565P（61.59%），对
  s8/s6 账本 diff 均零回归。zero-engine --lib 3138 全过、make test 全腿绿、
  clippy/fmt 过、零新增依赖。挂账：option.form 1F、text-setter 1F、
  replaceChild sel 形落尾缺口、t8g 墙钟钉 clippy 并行负载边缘越限观察账。
- **2026-10-18 M2 切片 8**：HTMLOptionsCollection/option list 基建
  （R3908/R3908b）——`_zwListOfOptions` 统一构造（嵌套 optgroup 排除、
  select/hr/datalist 整枝）、HTMLOptionsCollection 代理（length/索引
  setter、replace 取先删后插序规避 overlay identity miss）、
  `_zwIsSelectTag` 前缀判定六处、selectedOptions 代理三 trap + 视图归一、
  option.selected dirty selectedness（不反射；宿主激活脚本
  script_gen.rs 分流属性写）。part58 契约（W1–W9）。common 三文件
  15P、selectedOptions 簇 18P、script-mutation 5P 等**净 +54P**；forms
  3515→3569P、九域 4477→4531P（61.13%），对 s7/s6 账本 diff 均零回归。
  zero-engine --lib 3111 全过、make test 全腿绿（browser 腿 :102 复跑，
  s6 同款环境归因）、clippy/fmt 过、零新增依赖。挂账：ask-for-reset 3F、
  selected-index 7F、add/remove 4F、constructor dirtiness 3F、validity 7F、
  通用 replaceChild sel 形 oldChild 落尾缺口（独立排查）。
- **2026-10-18 M2 切片 7**：option 元素文本/反射语义（R3907）——
  `.text` getter「collect text value + strip and collapse」（script
  命名空间排除模型、子列表递归、NBSP 保留）、`.label`/`.value` 无 ns
  存在即采纳 + collapse 回落（R122 实例层判定）、`.text` setter
  textContent 委托；part03 助手 4 个（R3028 latest-wins 先探、64 层
  防环）+ part04 setter 支，part57 `test_option_semantics_r3907` 契约
  （W1–W7；part16 近 2000 行上限新切片文件）。option 族 36→101P
  （+65P），forms 3450→3515P、九域 4409→4477P（60.40%），对 s6 账本
  逐 subtest diff 零回归。zero-engine --lib 3107 全过、clippy/fmt 过、
  零新增依赖。**stash 对照实验**排除 dialog +3P 因果（runner 案级
  realm 身份非确定性，归 wpt-runner 通道挂账；d11 detached-document
  showModal 不抛为 `_zwIsConnected` 预存缺口）。挂账：option 余簇 7F
  异根（selectedness dirtiness 5F、form owner 1F、isConnected 面 1F）、
  HTMLOptionsCollection/option list 基建簇 ~45F+。
- **2026-10-17 M2 切片 6**：文本控件选区语义（R3906）——selection
  setter「set the selection range」算法 + value setter 选区折叠 +
  异步 select 事件（part06 `_zwScheduleSelectEvent`）+ textarea 换行
  归一修正 + attr 回落 latest-wins + defaultValue= 真脏态守卫 +
  type 变态复位 direction 归一（初版裸 'none' 满账本暴露 80F 同名
  回归后修正，作废首轮取账重跑）+ runner variant meta 展开（R329
  机制复用）。part16 `test_textfieldselection_r3906` 契约（W1–W9）。
  textfieldselection 784P/0F、type-change-state 462P/0F；select-event
  0→270P、selection-not-application variant 化 183+79P；forms
  2848→3450P、九域 3807→4409P（4409/7412 = 59.48%，+602P，分母
  +499），对 s5 账本逐 subtest diff 零回归。zero-engine --lib 3100
  全过、clippy/fmt 过、零新增依赖；make test 三个 browser shell
  间歇失败归因为环境面（X :100 catlink 桥死亡 + load 抖动，隔离与
  main CI 双证）。挂账：editing 管线 `_textSelection` 裸 'none'
  观察账；066.html 预存 hang。
- **2026-10-17 M2 切片 5**：select.showPicker() spec 检查序（R3905）——
  part03 方法分发区新支（disabled→InvalidStateError 先于
  being-rendered→NotSupportedError（display:none + cv:hidden 祖先）先于
  NotAllowedError 门、`_zwTransientActive` 同步消费、multiple/size 不抛），
  part16 R3905 契约单测（9 场景 + 抛错面激活不消费钉针）。show-picker
  六文件 0→7P、customizable-select/nested-options 联带 +1P，forms
  2840→2848P、九域 3799→3807P（零回归；分母 6913 不变）。
  `make test` 全绿、clippy/fmt 过、零新增依赖。教训：corpus 全量轮
  禁与 make test/clippy 并行（首轮负载噪声作废）。
- **2026-10-17 M2 切片 4**：valueAsNumber/valueAsDate setter 抛错面
  （R3904）——part04 两 setter 按 spec setter steps 重排（valueAsNumber
  ±Infinity→TypeError 先于适用性、不适用→InvalidStateError、NaN→''；
  valueAsDate WebIDL `Date?` 转换先行（undefined→null）、不适用
  （含 datetime-local）→InvalidStateError、null/undefined/Invalid Date
  →清空），part16 R3904 契约单测（步序钉针 + 三清空面）。typeerror
  两文件 8F + datetime-local-valueasdate 1F 翻绿、同族 invalidstateerr
  两文件联带 +2P，九域 3788→3799P（零回归；分母 6913 不变）。
  `make test` 全绿、guarded-clippy/fmt 过、零新增依赖。
- **2026-10-16 M2 切片 3**：input.showPicker() spec 检查序 + 瞬态激活消费
  （R3903）——part03 方法分发区新支（不可变检查先于激活门/消费、readonly
  适用 12 类型表、NotAllowedError 门、`_zwTransientActive` 同步消费、无
  focus），latest-wins 属性读三级回落，part16 契约单测。show-picker 三文件
  0→129P（user-gesture 63 / disabled-readonly 44 / does-not-focus 22），
  forms 2697→2829P、九域 3626→3788P（零回归；分母 6656→6913 系 img
  naturalWidth 文件完整上报）。`make test` 全绿、guarded-clippy/fmt 过
  （fmt 既有违规系并行会话遗留，见切片账本）、零新增依赖。
- **2026-10-16 M2 切片 2c**：stepUp/stepDown 全七类型 spec 步进算法 +
  valueAsNumber/valueAsDate IDL 全类型面（R3902c）——part03 `_parseHtmlDateValue`
  date/week/datetime-local 回读校验 + `_formatHtmlDateValue` week/
  datetime-local 新支 + stepUp/stepDown 分支重写（`_reflectedTypeEnum` 类型门、
  `_suDefaultStep`/`_suScale` 缺省表、`_suParseBound` 类型域解析、`_suKInt`
  相对容差近整判定、步进基与 spec 步 3/4/5/7/8/9/10；sel 路径 min/step 走
  `__zw_get_attr_lw` latest-wins 读）+ valueAsNumber getter 全类型，part04
  valueAsNumber setter date 族/month 扩面，part16 R3317 契约修正（方法恒
  暴露 + InvalidStateError + 离格 snap→6）。stepping 簇 +68P、联带 IDL 面
  +64P，九域 3494→3626P（零回归）。`make test` 全绿、guarded-clippy/fmt 过、
  零新增依赖。
- **2026-10-16 M2 切片 2b**：dirty checkedness/value + files IDL +
  `:active`/`:hover` + compound pending 回落（R3902b）——query.rs
  Active/Hover 识别臂，part01 dirty/files 两 map，part03 files/indeterminate
  `_realTag` 门 + 非 file 恒 null + checked 脏态优先，part04 setAttribute/
  removeAttribute('value') 已脏不覆写 + clone 传播 dirty value/checkedness/
  indeterminate + files setter（FileList 存引用/null no-op/否则 TypeError），
  part05 `.checked=` 置脏 + DataTransfer.files FileList 视图，part06 compound
  pending 回落（`[` 预门）。cloning-steps/untrusted-key-event/files/
  checkable-active-space 四簇 +125P，九域 3369→3494P（零回归）。
  `make test` 全绿、guarded-clippy/fmt 过、零新增依赖。
- **2026-10-16 M2 切片 2a**：focus/focus-visible 面（R3902）——part06
  focus-visible 模态旗标，part04 焦点派发接线 + matches() `:focus`/
  `:focus-visible` 分支（isConnected 连接性判定），part01 rAF 预算
  64→512（promise_test 串行链 double-rAF 断链根因）。focus
  -dynamic-type-change 双文件 0→120P、forms +121P、九域 3248→3369P
  （零回归）。`make test` 全绿、clippy/fmt 过、零新增依赖。
- **2026-10-15 M2 切片 1**：input value IDL 面（R3901）——part01 四值模式/
  净化/类型迁移助手，part03 getter 模式分派，part04 setter 分派 + setAttribute/
  cloneNode 接线，part05 `input.type=` setter，用户编辑通道
  （`__zw_text_input`/`__zw_text_delete`/execCommand/script_gen 三脚本）净化
  绕行旗标。the-input-element 528→951P、forms 1897→2320P、九域全量
  2715→3248P（+533，零回归）。中途 13 回归（textarea 缓存写误删）当日定位
  恢复（经验已沉淀 docs/learnings/bugs/2026-10/）。`make test` 全绿、
  clippy/fmt 过；csv 行回填全量数字。零新增依赖。
- **2026-10-15 M1**：fetch 脚本九域+叶子助资产（jsdelivr CDN 回落先落入
  lib.sh 共享路径）；runner 通道 `testharness-html-semantics`（九域顶层递归
  扫描 + skip 规则双侧同源）；基线 42.63%（2715/6369，1151 案），分域账与
  失败簇归因 evidence 落档；csv 数据行回填。挂账：span-limits /
  downloading-resources 两处挂死排除（FIXME）。`make test` 全绿、
  clippy/fmt 过、零新增依赖。

## 依赖约束备注

form-validation 遗产为 forms 域起点；html-syntax-compat（编号 40，已收口）的
parser 面资产为消费基础。

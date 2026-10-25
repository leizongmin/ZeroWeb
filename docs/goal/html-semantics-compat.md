# HTML Semantics 兼容 — html/semantics 语义面（主攻大域）

**版本**: v2.0
**日期**: 2026-10-25（v1.0 立项 2026-10-08）
**状态**: Active——M2 切片 84 分诊收口（dialog closedby 家族「Chrome-parity 同红」假设重估：chrome-158 全绿实证过时，23F 转可施工；根因隔离到 gBCR materialize 滞后——showModal/showPopover 后 rect 经 4 rAF 恒零、~100ms 才有效），下一步 Rust 侧定证 take_probe flush/version-gate 链 + 修复 rect 新鲜度（closedby 23F + abspos-dialog-layout 5F 共桥）；**用户决策 2026-10-23**：parser form element pointer 4F **缓行**——与 declarative shadow DOM 打捆 parser 批次、M2 收口后尾段单独立项（[决策记录](html-semantics-compat/evidence/2026-10-23-user-decision-parser-form-pointer-defer.md)），流按其余挂账序推进

> **▶ 当前状态（2026-10-25 M2 切片 84 分诊收口，closedby 同红假设重估轮）**：
> A 族 closedby×slot 16F 挂账依据（crbug 417571176 同红，2026-04）重估
> **过时**：wpt.fyi chrome-158.0.8122.0 summary 实证 closedby 家族全绿
> （complex 40/40、simple 6/6、closedby 27/27、bounds 14/14、corner
> 30/30、stacked 4/4、start-open 2/2）——closedby 已 ship，**23F 转可
> 施工**（complex 16 + simple 2 + bounds 2 + corner 1）。**根因隔离**
> （三步插桩探针，临时已删）：失败首腿全是 light-dismiss 误触发——
> 首测点击 popover 的 ld 落点 px=0/py=0（corpus clickOn 2-rAF 后采样
> rect 恒零）→ s30 backdrop retarget → popover pass 误隐；gBCR 时序
> 探针证 showModal/showPopover 后 sync/2raf/4raf 三拍 rect 恒零、
> +100ms 才有效，且序列化 htmlLen +23（closedby+open）与 rect 转有效
> 同拍——mutation 批 materialize（apply→render→rect 快照）在 rAF 驱动
> 节拍整链滞后 ~5+ 探测轮。**下一步**：Rust 侧定证 take_probe
> flush/version-gate/refresh 链哪环漏批（候选：mutation_version 门
> 时序、runner_probe_html 基线、timer 泵与 rAF 拍批次归属），修复 rect
> 新鲜度（closedby 23F 与 abspos-dialog-layout 5F 共桥——本轮顺带观测
> `:modal` UA position:fixed 未上、dialog 盒 in-flow）。账本见
> [evidence/2026-10-25-m2s84-closedby-parity-recheck.md](html-semantics-compat/evidence/2026-10-25-m2s84-closedby-parity-recheck.md)。
> **范围**：零代码轮（s53 先例），不改生产面；corpus 不可改（2-rAF
> 采样是上游语料行为，Chrome 同步 rect 语义使其天然正确——修复须落
> 我方 rect 新鲜度）。
> **下一步**：s85 修复片——Rust 契约测试复现 gBCR 滞后 + take_probe
> 链定证 + materialize 时机修复；验证门 closedby 家族 23F 红转绿 +
> 域/pin 逐位持平。
> **切片间隙**：本轮零代码；下一挂账 webview/runner 面（rect 快照链
> ——本流工作面），间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 83 收口，handle 形元素 matches complex 读面轮）**：
> s82 遗留 matches complex 面分诊**推翻非平凡判定**：probe 实证 gap 非
> 同 turn 窗口而是**永久**——post-drain `matches('#anc span')` 仍 false；
> qS 倒查命中并登记 R100 锚，但 t6 锚臂 host `__zw_matches` 读原始
> dom_html 快照（apply 后锚不可解析），'0' 无条件 return 短路。s82 已
> 落双形态父链解析 + compound 匹配器，JS 侧判定材料齐备（无需 Rust
> 回调）。**修复**：part05 `_zwHandleMatchesComplex` 三态契约（true/
> false/null——null=有不可判 complex，落回锚臂 host 全语义，pseudo 面
> 不劣化）+ part04 t6 臂重排（R134 → s83 JS 走 → 锚臂 → regex）。列表
> 逐 complex 判定；compound 带 pseudo/组合器非后代/子代保守不判（防
> over-match）；48 跳封顶；不越 shadow 边界。**关键序**：JS 走必须先
> 于锚臂——锚臂 '0' 短路后置补偿不可达（probe 定位）。契约
> dom_bridge_tests `handle_element_matches_complex_s83`（18 断言）。
> **验证**：probe 同 turn + post-drain 全 Pass（临时已删）+ engine lib
> **3523P/0F** + 域（FILTER=embedded-content）**976P/252F/30T 逐位
> 持平 REG 0** + interactive pin **439P/44F/0T 逐位持平** + fmt clean
> + clippy 仅 s78 既有警告。账本见
> [evidence/2026-10-25-m2s83-handle-matches-complex.md](html-semantics-compat/evidence/2026-10-25-m2s83-handle-matches-complex.md)。
> **范围**：兄弟组合器（`+`/`~`）保守 false（反链无前兄弟指针，语料
> 0 需求）；锚臂 host 快照底层未动（JS 走已覆盖可判面，host 修复仅剩
> pseudo complex 面，无已知消费方）。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集成
> 面，启动前查 oracle master 碰撞记录）。
> **切片间隙**：本轮 js_dom_shim 两件 + 契约 dom_bridge_tests；下一
> 挂账 layout 集成面需跨流协调，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 82 收口，handle 形元素 closest 读面轮）**：
> s81 范围注记的「同 turn insert 身份缺口」分诊升级：createElement 产物
> 是 handle 形代理（`__zwSelector` 恒 null），closest 的 `!sel` 门使其
> **永久盲**（非同 turn 窗口——已 append 排水后仍 null；身份永驻 handle
> 形）。分诊矩阵实证：matches 单 compound true（t6 臂）、complex false
> （stale 缺口）、closest ×3 全 null；语料需求 0（117F × createElement
> 39 处 × closest/matches 共现 = 0），立项按 spec 定义 + 真实站点
> slice（jQuery/Swiper 创建元素 `closest`）。**修复双臂**
> （part05 `_zwHandleClosest` + part04 closest 入口 handle 分支）：
> ①R100 锚回（曾 sel 向解析过）→ host `__zw_closest` 全语义；②JS 单
> compound 祖先走——父链双形态解析（sel→`__zw_parent` 视图文档；handle
> →`_zwNodeParent` 反链），48 跳封顶（R125 口径），nodeType≠1 停
> （不越 shadow 边界——spec closest 祖先轴不含 shadow host），complex/
> 伪类保守 null（`_matchCompoundOf` 不求值 pseudos，防 over-match）。
> 契约 dom_bridge_tests `handle_element_closest_s82`（15 断言）。
> **验证**：probe 4 面 Pass（临时已删）+ engine lib **3522P/0F** +
> 域（FILTER=embedded-content）**976P/252F/30T 逐位持平（REG 0——
> 语料 0 需求预判实证）** + interactive pin **439P/44F/0T 逐位持平** +
> fmt clean + clippy 仅 s78 既有 concat! 警告（不在本 diff）。账本见
> [evidence/2026-10-25-m2s82-handle-closest.md](html-semantics-compat/evidence/2026-10-25-m2s82-handle-closest.md)。
> **范围**：matches complex stale 面（handle 形锚回 host 快照 miss）
> 记遗留不建——需 Rust 视图文档 matches 回调或 query_all 回退，非平凡。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集成面，
> 启动前查 oracle master 碰撞记录）或 matches complex stale 面分诊。
> **切片间隙**：本轮 js_dom_shim 两件 + 契约 dom_bridge_tests；下一
> 挂账同面或 layout 集成面，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 81 收口，查询面 id 改名可见性轮）**：
> R125（s79）读侧只接 gEBI；查询面（querySelector/querySelectorAll/
> matches/closest）同 turn 改名 `#new` miss、`#old` stale hit。probe
> 分轮定位**根因补刀**：`el.id` IDL 反射读走纯快照（与 getAttribute
> 的 R2995 lw 链同 turn 分歧）——`_matchCompoundOf` ids 校验读
> `p.id` 故 token 形态拉真全灭。**修复两刀**：①part03 id 臂 lw 化
> （共享路径，DOM 反射 spec 同步语义；`_zwRawIds`/handle 形保持）；
> ②part05 s81 helpers + 六挂点双臂补偿——stale 剔除（lw 现值复验，
> 全形态）+ 单 compound 拉真（R125 表候选 + in-doc 门 + full
> compound 复验 + 子树门；qSA 文档序归并）。R3190 属性语义不动
> （补偿只动选择器 id 约束）。契约 dom_bridge_tests
> `query_id_rename_same_turn_visibility_s81`（17 断言）。
> **验证**：细粒度 probe 12 面 Pass（临时已删）+ engine lib
> **3521P/0F** + 域（FILTER=embedded-content）**976P/252F/30T 逐位
> 持平（REG 0 / GAIN 0——s80 预判无直接语料依赖，实证）** +
> interactive pin **439P/44F/0T 逐位持平** + fmt clean。
> 账本见
> [evidence/2026-10-25-m2s81-query-id-rename-visibility.md](html-semantics-compat/evidence/2026-10-25-m2s81-query-id-rename-visibility.md)。
> **范围**：complex 祖先段约束 pull-in 不接（single 门，stale 剔除
> 仍全形态生效）；同 turn insert 元素无 `__zwSelector`（批末 apply
> 才有身份）上行 closest/query 盲区——结构性身份缺口与改名无关
> （无 id 对照面同样 null 实证），后续独立切片。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集
> 成面，启动前查 oracle master 碰撞记录）或 s81 遗留同 turn insert
> 身份缺口分诊。
> **切片间隙**：本轮 js_dom_shim 四件 + 契约 dom_bridge_tests；
> 下一挂账同面或 layout 集成面，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 80 收口，跨批 selector 迁移 journal 轮）**：
> s79 遗留 runner 缺陷 C（hash-name-reference-dynamic 3P/5F）定位并修复。
> s79 假说「序列化/apply 只经客户端缓存属性」**证伪**：Rust 侧 attr-trace
> 实证 mutations 队列不 drain（1→9 单调）、同键 lw 读亦通——丢点在
> **apply 解析**：全以创建期 sel 入队（proxy 闭包 sel 冻结），批 2 改名
> 落管线 doc 后，批 3 起旧键属性写 `find_by_selector` 恒 miss 走
> lenient skip 静默丢写；序列化无 name / 视图查询 miss / gEBI 旧键落空
> 均为下游表现。**修复（s80 journal）**：`apply_dom_mutations_full` 新参
> `sel_renames`——SetAttr(id) 落地记（改名前 sel → `#新id`），失配臂
> `journal_find_sel_rename` 追链（4 跳封顶）重试；直查命中优先（新元素
> 抢占旧 id 不劫持）。持有方 `RenderPipeline::sel_renames`，导航边界清、
> 同页 `render_html` 重建不清（selector→selector 映射不受 slotmap 换代
> 约束——首版在重建点清空致语料无效，复跑定位后修正）。契约
> dom_bridge_tests `test_apply_sel_rename_journal_cross_batch_attr_s80`。
> **验证**：engine lib **3515P/0F** + 域（FILTER=embedded-content）
> **976P/252F/30T（GAIN 5 / REG 0，hash-name-dynamic 3P/5F→8P/0F）** +
> interactive pin **439P/44F/0T 逐位持平**。范围：journal 只挂
> SetAttr/RemoveAttr 失配臂（SetText/SetStyle 等无语料实证面暂不扩）。
> 账本见
> [evidence/2026-10-25-m2s80-cross-batch-sel-rename-journal.md](html-semantics-compat/evidence/2026-10-25-m2s80-cross-batch-sel-rename-journal.md)。
> **下一步**：`querySelector('[id=…]')` 同 turn 改名可见性（共享查询
> 视图面）或 720-stretch 渲染展开 22F（需布局几何，归 layout 集成面，
> 启动前查 oracle master 碰撞记录）。
> **切片间隙**：本轮 js_dom_bridge + pipeline + webview 导航清点 +
> 契约 dom_bridge_tests；下一挂账同面或 layout 集成面，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 79 收口，IDL id 改名 R125 对称臂轮）**：
> s78 未解决项 hash-name-reference-dynamic 5F 分诊——**控制面原记
> 「testdriver-Actions 面」证伪**：runner 自带 test_driver.Actions stub，
> 8 行点击全链路可达（ZW_TD_DEBUG resolve 迹：首点击 `#area` 全通，首个
> DOM 变更后 map 解析失败）。5F 真实面 = shim 两缺陷（本轮修复）+ runner
> 属性传播缺陷（未解决，后续切片）。**修复（R5279 两刀）**：①part04
> `.id=` set trap 补 R125 sel-based id 覆盖表登记（与 setAttribute('id')
> 对称——缺登记同 execute getElementById miss 新 id；latest-wins 缺写使
> 「改名回原 id」被残留旧条目毒化）；②part06 getElementById 第二条
> pending-by-id 兜底环补 `_r125Overridden` 门（快照命中被覆盖表剔除后
> 不得经该环复活）。契约 part90 `idl_id_rename_getelementbyid_r5279`
> （8 断言，撤任一刀对应转红）。**验证**：engine lib **3512P/0F** + 域
> （FILTER=embedded-content）**971P/257F/30T 持平** + interactive pin
> **439P/44F/0T 逐位持平**（R5279 只增 id 覆盖登记，语料面零数字变化）。
> **新诊断（未解决）**：runner 命令 turn 的 resolve 时点**属性写丢失**
> （`setAttribute('name')` 不在 live 态亦不入序列化——id 写有客户端缓存
> 可见、name 写不可见，疑序列化/apply 管线只覆盖客户端缓存属性）+
> **覆盖表缓存代理空子树**（re-parse 代际后 gEBI 命中代理
> `querySelectorAll('area')` → 0）——hash-name-dynamic 5F 实际归属此组合
> 面；resolve 迹仪器化方法已验证（ZW_TD_DEBUG + 临时探针，跑完即删）。
> 账本见
> [evidence/2026-10-25-m2s79-idl-id-rename-r125.md](html-semantics-compat/evidence/2026-10-25-m2s79-idl-id-rename-r125.md)。
> **下一步**：runner 属性传播缺陷切片（refresh 管线属性 mutation 应用 +
> 代际代理失效，面在 tests/wpt-runner + shim 视图生命周期）或 720-stretch
> 渲染展开 22F（需布局几何，归 layout 集成面）。
> **切片间隙**：本轮 part04/06 shim + 契约 part90；下一挂账 runner 面
> 在 tests/wpt-runner，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 78 收口，F4 image-map 命中轮）**：
> s67 八族地图 F4 收口（the-area-element 三文件 area-coords 32F +
> area-shape 17F + area-processing 14F）。失败面修正：控制面原记 84F 系
> 旧账——实测 68F 中 5F 是 hash-name-reference-dynamic 的 testdriver-Actions
> 面（runner 语料缺 testdriver.js，指针/键盘合成输入族，本切片外），
> image-map 实际可收面 63F。**根因**：runner headless 无渲染管线——host
> `__zw_elementFromPoint`（R2924 HitTestCache）与 `__zw_getBoundingClientRect`
> 均缺席，elementFromPoint 缺席臂此前只有 SVG 形状回落，usemap img 形状
> 命中面为空 → 63F 全 null。**修复（R5278 三刀）**：①解析核（part03）——
> float-list 逐字解析（分隔集 ASCII 空白+逗号+分号、`\v` 非空白、垃圾跳读、
> 浮点前缀截断含指数 `.4`/`1e1`/`2in`、error→0）+ shape 枚举（ASCII
> case-insensitive，missing/invalid → rect）+ 点入形状（rect min/max 归一
> 半开框、circle d²≤r²、poly on-edge 预检**边界含入** + even-odd 半开边
> 射线——star (101,101)/(199,101)/hitPoly (119,101) 三行 corpus 定谳、
> default 恒真）+ hash-name reference；②命中面（part06 `_zwImgMapHitSel`）
> ——elementFromPoint 缺席臂（SVG 后置臂，产品路径零波及），img 框 (0,0)+
> IDL 尺寸近似（FIXME 真实几何归 layout 集），无形状覆盖/usemap 不可解析 →
> img 自身；③契约 part90 两测试（65 断言）。**排障留档**：裸跑 runner 缺
> Makefile 目标的 `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`，interactive pin 假红
> 7 行（滚动/点击时序面）；stash 验证基线同红排除代码回归后改走 make 目标
> ——runner 数果实测必须走 Makefile 目标。**验证**：域（make + FILTER）run3
> **907P/320F/31T → 971P/257F/30T**（GAIN 64 = area 63 + usemap-casing
> 加成 1 / 0 P→F；audio-lazy T→P 为已记抖动族顺风向；裸跑/make/全域截断跑
> 三方 embedded-content 数字逐位一致）+ interactive pin **439P/44F/0T 逐位
> 持平**（make 目标重跑后）+ engine lib **3499P/0F** + clippy/fmt clean。
> 账本见
> [evidence/2026-10-25-m2s78-image-map-hit.md](html-semantics-compat/evidence/2026-10-25-m2s78-image-map-hit.md)。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集成面）或
> testdriver-Actions 支持面（hash-name-dynamic 5F + 指针/键盘合成输入族）。
> **切片间隙**：本轮 part03/06 shim + 契约 part90；下一挂账 720-stretch 需
> layout 面协同，间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-25 M2 切片 71 收口，F1b SVG 固有尺寸轮）**：
> s67 八族地图 F1b 收口（naturalWidth-naturalHeight-width-height 的 SVG
> data URL 族 77P/181F + svg-img-density-vs-zoom 1P/8F）。**根因**：runner
> 位图解码面不解 SVG——host `__zw_get_image_size` 返回 viewBox 原始宽高
> （缺省 100×100），非 CSS-images natural dimensions 组合表；resourceState
> 无固有轴元数据，密度校正无法逐轴表达语料 data-density 语义（attr 轴校正、
> 默认/AR 轴不校正）；display:none 未渲染 IDL 面（whatwg#11287）恒密度校正。
> **修复（R5277 五刀）**：①`__zwSvgPayload5277` data:SVG payload 提取
> （percent/base64/raw）；②`__zwSvgNaturalDims5277` 固有尺寸核（无引号 attr
> 按 HTML tokenizer 语义、length 非负 number|px、viewBox AR、w+h/w/h/none
> 组合表）；③sweep SVG 核前置 + 覆写面——携 axes（本核已写）幂等跳过、
> host 态（无 axes）原位覆写不重派 load、非 SVG 维持 host 幂等；④密度逐轴
> 折算（attr 轴按选中密度校正、默认/AR 轴恒原始；无 axes 旧态默认 both，
> raster 语义不变）；⑤未渲染 IDL 臂——display:none + loaded 回原始未校正
> intrinsic（内联样式直读近似，FIXME 样式表/继承面）+ avail 传播携 axes。
> **运行期 fast path 撤除留档**：fetch 臂 SVG 前置在 129 img 模板克隆风暴下
> 超 `_deferBudget=256` 静默丢回调 → inflight 泄漏 → load 链死锁（run5 全域
> 1001/1258 行截断实证），同步 dispatch 变体仍抖（77P/181F 不稳）——终版
> 撤除，运行期 SVG 语义归 host 解码器切片。契约 part89 两测试（27 断言，
> (h) 覆写面 load 不重派钉）。**验证**：域 **747P/479F/32T → 907P/320F/31T**
> （GAIN 160 = 160 F→P / 0 P→F；唯一 T 变化 video-loading-lazy-autoplay
> T→F 为 s70 已记环境抖动行翻回）+ naturalWidth 文件 77P/181F → **236P/22F**
> （3× 稳定）+ interactive **439P/44F/0T 逐位持平** + engine lib
> **3496P/0F** + clippy/fmt clean。账本见
> [evidence/2026-10-25-m2s71-svg-intrinsic-dims.md](html-semantics-compat/evidence/2026-10-25-m2s71-svg-intrinsic-dims.md)。
> **下一步**：F4 image-map 命中测试 84F（elementFromPoint 几何）优先，或
> 720-stretch 渲染展开 22F（需布局几何，归 layout 集成面）。
> **切片间隙**：本轮 part03/04 shim + 契约 part89；下一挂账 F4 亦 engine 面，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-25 M2 切片 70 收口，F2 source-set 族轮）**：
> s67 八族地图 F2 收口（parse-a-srcset 49 + select-an-image-source 13 +
> update-the-source-set 84 + relevant-mutations 46 + current-pixel-density
> 21 + 邻族 7）。**根因**：srcset/sizes/picture-source 选择面仅 R4162/R2419
> 「首 URL」近似——描述符语法/DPR 择面/media+type 门/relevant mutations
> 触发/密度校正全缺。**修复（R5277 九刀）**：①part03 共享选择核
> （parse-srcset/parse-sizes/create-source-set/update-source-set/
> select-image-source 五核，spec §4.8.4.3.7-12 逐字）；②候选 URL 就地绝对化
> （`_zwResolveFetchUrl` 文档基含 `<base href>`——available-images 键/请求
> URL/currentSrc 同源；raw/绝对键并存把同图拆两个 avail 表项，
> image-loading-lazy-available P→F 实证）；③全状态写入面切选中 URL（sweep
> srcset-only img / `.src` IDL 链 / revalidate）+ 密度入 resourceState +
> naturalWidth 密度折算；④relevant mutations 触发面（src/srcset/sizes 恒重
> 验、crossorigin/referrerpolicy content-state 枚举比对、SOURCE 变迁 → 其后
> img 兄弟、picture 树插入/移除）+ restart 收窄为选中 URL 变更（同 URL 命中
> 照常重派 load——spec maybe-omit-events 门）；⑤null selection（src/srcset
> 移除 → broken，error 仅在仍有关联时派）；⑥detached 更新延一步（重读
> loading——src-then-lazy-load 时序）+ lazy 视口门只拦 fetch 不拦 avail
> settle（avail 先于 lazy 决定）+ scrollIntoView 视口旗标近似（F5 几何收口
> 前债务标记）；⑦broken 无 width/height 属性 → IDL 0（s68 误正改 0——
> 300/150 是 UA icon 非 IDL 面）；⑧css-parser media_query.rs MQ4
> error-handling——非空 inner 无法解析 → 整条不匹配，不再退化空条件 all 恒
> 匹配（`all and (!)` 面 + matchMedia false 对齐，钉定测试撤换）；⑨extract.rs
> 候选收集接解析核。契约 part88 五测试。**验证**：域 **607P/678F/37T →
> 747P/479F/32T**（GAIN 133 = 133 F→P / 0 P→F；run3 捕获唯一 P→F 由②收口
> run4 复验翻回；run3→run4 差恰 2 行）+ interactive **439P/44F/0T 逐位持平**
> + engine lib **3484P/0F**（+5 契约）+ css-parser lib **3003P/0F** +
> clippy/fmt clean。账本见
> [evidence/2026-10-25-m2s70-source-set.md](html-semantics-compat/evidence/2026-10-25-m2s70-source-set.md)。
> **下一步**：F1b naturalWidth SVG 固有尺寸/密度校正尾片（210F 余量）或 F4
> image-map 命中测试 84F（elementFromPoint 几何）——按共享路径收益取一。
> **切片间隙**：本轮 part03/04/05 shim + extract.rs + css-parser media_query
> + 契约 part88（part85 键位随绝对化更新）；下一挂账 F1b/F4 亦 engine 面，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 69 收口，F3 img.decode() 轮）**：
> s67 八族地图 F3 收口（60 子测试 60F → 58P/2F）。**根因**：decode IDL 无
> 实现（get trap 无臂 + SVGImageElement 原型无方法）。**修复（R5276 三刀）**：
> ①共享解码核 `__zwImgDecodeCore`（part03 IIFE 顶层，IMG trap/detached 动态
> 臂/SVG 原型三方复用）——select-image-source 近似候选（picture 首 source
> srcset 首 token，data: URL 不逗号截断）**调用时捕获** + 恒异步 settle +
> settle/快捷面双点变异与活跃性复检（探针钉定中断前同步即决稿三缺口：同步
> 即决漏 adopt 时序、pinned 恒拒臂挡 adopt-in、settle 重读误判同 turn
> attach）+ 自驱 fetch 链，decode 只读不写 settle 状态；②part03 detached
> 产物动态臂（调用时 parentNode 链判活跃，adopt-in 后可达走核，属性空回落
> src expando）+ SVGImageElement.prototype.decode（href → xlink:href）；
> ③part04 trap 臂瘦身为闭包供给。extract.rs collapsible_if（s68 遗留
> clippy，非行为变更）。契约 part86 两测试（决策序九臂 + adopt/SVG/picture
> 九臂）。**验证**：域 **549P/738F/35T → 607P/678F/37T**（GAIN 58 = 58
> F→P；子测试总数 1322 守恒；逐子测试 diff 非 decode 变化仅 2 条）+ 残留
> 2F = quick-attach empty picture cached/uncached 时序差（issue 10531 待改
> 行为，归 F2 请求对象模型）+ 2 案 F→T（audio/video loading-lazy-autoplay
> **预存环境抖动**——stash 后 HEAD 复跑同 Timeout 实证，非本切片）+ lib
> **3477P/0F**（+2 契约）+ clippy/fmt clean。账本见
> [evidence/2026-10-24-m2s69-img-decode.md](html-semantics-compat/evidence/2026-10-24-m2s69-img-decode.md)。
> **下一步**：F1b naturalWidth SVG 固有尺寸/密度校正尾片（210F 余量，
> intrinsic dimensions 算法）或 F2 select-image-source/currentSrc/srcset/
> sizes（~300F 最大单族——decode 已消费 select-image-source 近似，开片有
> 地基）——按共享路径收益取一开片。
> **切片间隙**：本轮 part03/part04 shim + 契约 part86 + extract.rs（s68
> 遗留 clippy）；下一挂账 F1b/F2 亦 engine 面，间隙开放，开工前查 oracle
> master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 68 收口，F1 img 静态加载管线轮）**：
> s67 八族地图 F1 首靶收口（原估 ~260F，实测归因后 raster src 克隆可翻
> 部分本轮全收，SVG 尾片归 F1b）。**根因**：img settle 只接 `.src` IDL
> setter 通道（R56h），parser 静态 `<img src>` 无 fetch/settle——
> naturalWidth 恒 0、load/error 恒不派；叠加 template 内容漏抓（zero_dom
> fragment 不在文档树，extract_img_srcs 不达）、broken 终态缺失（error 后
> complete 恒 false）、运行期事件在页面脚本阶段即时派（spec：DCL 不等图
> 片、图片延迟 window load）。**修复（R5275 六刀）**：①part04
> `__zwFireStaticImgSettle` sweep（`__zw_get_image_size` 快照命中 → loaded
> + load + available-images；未命中 → broken + error；lazy/空 src 跳过）+
> 闸门队列 drain + `__zwImgLoadBlocked` 在飞判定；②part04 getter 面——
> `_zwImgNaturalDim`（lazy 未 settle → 0 防快照绕门）+ IMG width/height
> 缺失回落自然尺寸 + currentSrc 绝对 URL + complete getter 改读
> latest-wins（R3204 陈旧快照坑）；③part05 R56h src 臂重写——restart 语
> 义（同 URL 幂等）+ available-images 同步 settle（状态同步可读、load 事件
> `_defer` 后续任务派）+ `_zwImgInflight` 在飞计数 + `__zwReadyState ===
> 'loading'` 闸门入队；④extract.rs 递归收集 template contents `<img>`；
> ⑤testharness.rs 生命周期拆三任务链（interactive+DCL / settle 独立任务
> 微任务边界排水 / load 等待任务 100ms 重试至图片不在飞——trickle(dN) 慢
> 图延迟 window load 语义）；⑥available-images 臂事件 `_defer` 化（probe14
> 实证同步即派丢 src 后注册的 onload）。契约 part85 两测试（sweep settle
> 四臂 + 闸门/available-images/幂等）。**验证**：域 **461P/816F/40T →
> 549P/738F/35T**（GAIN 88 = 78 F→P + 5 T→P + 5 新注册，三数守恒无回归空
> 间）+ naturalWidth 30P/228F→48P/210F + delay-load 7 案全绿 + invisible
> /already-loaded/available-images-onload 各 +1 + interactive
> **439P/44F/0T 逐位持平** + lib **3474P/0F** + 翻转力实证（撤钩子调用/
> 撤 sweep 体 → part85 双红，恢复字节一致）。账本见
> [evidence/2026-10-24-m2s68-img-static-loading.md](html-semantics-compat/evidence/2026-10-24-m2s68-img-static-loading.md)。
> **下一步**：F1b naturalWidth SVG 固有尺寸/密度校正尾片（210F 余量，
> intrinsic dimensions 算法）或 F3 img.decode() 56F（promise 面接 settle
> 状态机，F1 地基已备）——按共享路径收益取一开片。
> **切片间隙**：本轮 part04/05 shim + extract.rs + testharness.rs + 契约
> part85；下一挂账 F1b/F3 亦 engine 面，间隙开放，开工前查 oracle
> master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 67 收口，embedded-content 开片分诊轮）**：
> 两件事。**① 域分诊八族地图落档（819F 全归因）**：F1 img 静态加载管线
> ~260F（settle 只接 `.src` IDL setter 通道 R56h，parser 静态 img 无
> fetch/load 事件、naturalWidth 恒 0）+ F2 select-an-image-source/currentSrc
> /srcset/sizes ~300F（currentSrc 恒 ''，全族算法未实现，独立大切片）+
> F3 img.decode() 56F + F4 image-map 命中测试 84F（elementFromPoint 几何
> 底座→渲染流记账回流）+ F5 lazy-loading ~49F（视口几何缺失，F1 后重
> 分诊）+ F6 canvas 53F（getContext options 序 + Tab 序，逐案分诊）+
> F7 object/embed 窄面 25F + F8 杂项 2F。域刷新基线 **458P/819F/40T**
>（执行案 231 与 M1 一致；subtest 1060→1317 为 R5235 帧驱动后此前
> Timeout 悬挂案完成注册）。**② F7 null 反射收口（R5274 一刀）**：part04
> get-trap IFRAME 分支后补 OBJECT/EMBED 分支——contentDocument/
> contentWindow 恒 null（spec「does not represent a nested browsing
> context → null」尾步）+ getSVGDocument 返 null 函数（GetSVGDocument
> Support）；EMBED 无 contentDocument/contentWindow 成员保持 undefined。
> 契约 part84 两测试（钉①五臂 + 钉② IFRAME R130 面不回归）。**验证**：
> 域 **458P/819F/40T → 461P/816F/40T**（GAIN 3 / REG 0）+ interactive
> 439P/44F/0T 逐位持平 + lib **3469P/0F**（+2 契约）+ 翻转力实证（撤刀
> → 钉①红，恢复字节一致）。账本见
> [evidence/2026-10-24-m2s67-embedded-content-triage.md](html-semantics-compat/evidence/2026-10-24-m2s67-embedded-content-triage.md)。
> **下一步**：F1 img 静态加载管线开片（~260F 最大可收口单族；DCL 扫描
> settle + loading=lazy 跳过 + setAttribute('src') 通道，R5254 两段式
> 先例）。
> **切片间隙**：本轮 part04 get-trap + 契约 part84；下一挂账 F1 亦
> engine shim/host 桥面，间隙开放，开工前查 oracle master 碰撞记录
>（2026-10-24 已查：oracle master 记「engine 面占用标记已退场清除」，
> 无新占用）。
>
> **▶ 上轮状态（2026-10-24 M2 切片 66 收口，text-level-semantics 开片轮）**：
> 两件事。**① the-a-element 13F 两族全收（R5273 三刀）**：a.text
> getter/setter = textContent 精确委托（9F；option 的 strip-and-collapse
> 不适用）+ a/area toString 品牌检查（4F；WebIDL es-stringifier step 2，
> **探针实证 trap 臂才是 createElement 产物 toString 实际供给面**——原型
> defineProperty 不达，双侧同步防直装路径）。**② 共享路径根修（part05
> overlay 种子）**：a.text 委托暴露预存缺陷——textContent=（sel 形）写后
> 同 turn childNodes.length===0 而 firstChild 已见新值（探针钉定与 .text
> 无关）。根因 = 注册文本节点无 pending link，overlay added 并入跳过而
> removed 剔除生效，childNodes 与 firstChild 两视图分裂（R5269 probe62
> childLen=0 同根源）。修 = `_zwOverlayPendingChildNodes` 入口
> `_zwTextElsBySel` 种子（命中即以注册节点为基；appendChild 并入/
> removeChild 剔除/apply 后不双计均保持）。**验证**：域 **34P/0F**
>（exit 0 首次）+ links 119P/20F/4T（stash 二分与 HEAD 逐值一致，预存）
> + document-metadata 59P/59F/18T + tabular 130P/4F + interactive
> 439P/44F + engine lib **3463P/0F**。账本见
> [evidence/2026-10-24-m2s66-text-level-a-element.md](html-semantics-compat/evidence/2026-10-24-m2s66-text-level-a-element.md)。
> **下一步**：embedded-content / scripting-1 / microdata 按挂账序开片
>（分诊轮先行）。
> **切片间隙**：本轮 part03/part04/part05 shim + part05 overlay；下一挂
> 账 embedded-content 分诊未定，间隙开放，开工前查 oracle master 碰撞
> 记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 65 收口，http-equiv 分诊 + content-language pragma 轮）**：
> 两件事。**① http-equiv-and-name 2F 分诊（本域 shim 面最后未分诊项，
> 分诊轮全归档）**：-1 = CanvasText 系统色 × color-scheme applied-color
> 链 → 渲染流记账回流（与 s64 color-scheme 23F 同桶）；-2 = 本流修复。
> **② 修复面（R5272，dom crate 两刀）**：①`parse_selector_chain` 复合伪类
> 名字边界——名字取 `(`/`#`/`.`/`[`/`:` 最早命中、仅 `(` 按含参取 args，
> 旧 `r.find('(')` 扫过整段余串把 `:root:lang(de-DE)` 的 `root:lang` 当
> 名字 → SyntaxError；②`matches_lang` pragma-set default language 回落
>（spec「language of a node」Otherwise 分支）——祖先链全无 lang 时吃
> `<meta http-equiv=content-language>`（含逗号失效、首 ASCII 空白 token、
> 文档序最后胜出；空 lang = explicitly unknown 不回落；HTTP 头回落
> FIXME），R3281 共享权威面 style-system `:lang()` CSS 匹配同步吃到。
> 回归测试 zz_r5272×2。**验证**：域 **59P/59F/18T**（+1 NotRun 为 CSP
> tentative 行与本修复无关）+ interactive 439P/44F + style-system lib
> **2449P/0F**（共享权威零回归）+ zero-dom lib **868P/0F**（+2）。账本见
> [evidence/2026-10-24-m2s65-http-equiv-content-language.md](html-semantics-compat/evidence/2026-10-24-m2s65-http-equiv-content-language.md)。
> **下一步**：text-level-semantics 开片（M1 基线 21P/13F），分诊轮先行。
> 余账全归档：color-scheme 23F + http-equiv-and-name-1 1F + 管道/应用面
> ~27F 渲染流、CSP tentative + parse-world 底座。
> **切片间隙**：本轮 dom crate（query.rs/lang_dir.rs）；下一片
> text-level-semantics 分诊轮不改引擎生产面，间隙开放，开工前查 oracle
> master 碰撞记录（2026-10-24 已查：渲染流在 ::first-line/::selection/
> paint 面，无 dom/shim 占用）。
>
> **▶ 上轮状态（2026-10-24 M2 切片 64 收口，document-metadata 分诊 + link/style 窄面轮）**：
> 三件事。**① color-scheme 归属核对（定档渲染流）**：10F 死因 =
> corpus 打包缺口（缺 support/compute-root-color-scheme.js，上游有）
> ——按「打包缺口本地补」先例恢复（gitignored 本地数据，**步进
> 52P/54F/18T → 52P/66F/18T** +12F 归因入账）；断言面 23F 塌缩单根
>（inline CanvasText 系统色 × color-scheme 解析 + colorScheme IDL +
> root 应用色）= applied-color → 渲染流记账回流。**② link/style 事件
> 族分诊（35F+18T 四类）**：资源管道 ~13F + applied CSSOM ~13F + CSP
> tentative 底座 7F+5T + parse-world 底座 1F 挂账；shim 窄面 7F 本轮
> 全收。**③ 修复面（R5271）**：①link.fetchPriority 枚举反射（合法
> ascii 小写、missing/invalid → auto，2F）；②link.relList supported
> tokens（link types 表 20 关键字，supports() 1F）；③style.sheet
> [SameObject] WeakMap 缓存 + 直接子指纹失效（有效子变更换新对象 /
> 空 fragment 身份保持）+ connected 门（不在文档返 null）+
> StyleSheet 基接口 brand（instanceof 可达，LinkStyle 2F）。**验证**：
> 域 **58P/60F/18T** + tabular 130P/4F（终版二进制）+ interactive
> 439P/44F + lib **3458P/0F**（CSSOM 契约零回归）。账本见
> [evidence/2026-10-24-m2s64-metadata-triage-linkstyle.md](html-semantics-compat/evidence/2026-10-24-m2s64-metadata-triage-linkstyle.md)。
> **下一步**：http-equiv-and-name 2F 分诊（本域 shim 面最后未分诊项）；
> 域收口后 text-level-semantics 开片（M1 基线 21P/13F）。余账全归档：
> color-scheme 23F + 管道/应用面 ~27F 渲染流、CSP tentative + parse-world
> 底座。
> **切片间隙**：本轮 part04/part06 JS shim；下一挂账 http-equiv 分诊
> 未定，间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 63 收口，document-metadata base 族轮）**：
> document-metadata 第二实施切片：**43P/63F/18T → 52P/54F/18T**
>（GAIN 9 / REG 0），the-base-element 13/13 全绿。**修复面
>（R5270 三刀）**：①`_zwDocBaseURL` scheme 排除——spec 文档基 URL
> 算法末步 data/javascript scheme **立即返 fallback**（非继续找下一
> base，base-data 案 1 期望 document.URL 实证）。②pending 增补 base
> 先扫——R161 归并把 pending 追加快照尾树序不保（head.prepend 新
> base 先于快照 base 评估，案 3 实证）；corpus 未钉 pending valid
> base 树序案，先扫近似记已知限制。③BASE href IDL（R5008 块内分
> 支）——缺失/空串 → 文档地址；解析失败 → **原属性串**
>（base_href_invalid corpus 钉定 "//test:test"，首版「失败 → 文档
> 地址」翻红回退修正）；LINK 保持 R5008 口径（harness resolveUrl
> 消费面）。**验证**：the-base-element 13/13 + tabular 130P/4F +
> interactive 439P/44F + lib **3456P/0F**（_zwDocBaseURL 共享面
> a/form action、baseURI、img src 零回归；+2 为 oracle 流新增）。
> 账本见
> [evidence/2026-10-24-m2s63-base-url-family.md](html-semantics-compat/evidence/2026-10-24-m2s63-base-url-family.md)。
> **下一步**：meta color-scheme 10F 归属核对（疑 style-system/渲染
> 面）；link/style 事件 30F+18T 分诊（runner file 通道能力上限）；
> style type/media + CSP tentative 待分诊。
> **切片间隙**：本轮 part03；下一挂账分诊未定，间隙开放，开工前查
> oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 62 开片，document-metadata title.text 族轮）**：
> 两件事。**① tabular-data 余 4F 分诊定档（不修）**：processing-model-1
> 四案全是表格布局 span 几何（colspan/col span=1000 字面列数摊宽、
> rowspan=0 不参与行高分布；offsetWidth/clientHeight 读），控制面排除
> 清单明确归 rendering-compat——本流收口定格 **130P/4F**，余案归因
> 挂账（DOM 语义面 IDL clamp 15 断言已全绿）。**② document-metadata
> 开片（M1 基线 31P/75F/18T → 43P/63F/18T，GAIN 12 / REG 0）**：
> Mission 序 tabular-data 后下一域，域内从未切片。**修复面（R5269，
> title.text 族 12F 全收）**：①title.text getter（part03）——child text
> content 直接 Text 子原样拼接（comment/元素子不含、无空白归一；
> title.text-01 断言 .text="TEXT"≠textContent）；sel 形 SetText 后融合
> 视图不载新文本——walk 空且 lw 非空才直取 latest-wins（appendChild
> 变更链 walk 非空自然排除，lw 不可直接作信号——title.text-01 首版
> 回退实证）。②title.text setter（part04）——R3907 option.text 同口径
> 委托 textContent。③document.title setter（part06）——改经元素
> textContent setter 落地（自带宿主写 + 本地文本子注册；旧裸
> __zw_set_text 只写宿主，本地文本子 stale 实证）。**验证**：
> the-title-element 13/13 + tabular 130P/4F + interactive 439P/44F +
> lib 3454P/0F（R3035 共享面零回归）。账本见
> [evidence/2026-10-24-m2s62-title-text-family.md](html-semantics-compat/evidence/2026-10-24-m2s62-title-text-family.md)。
> **下一步**：base 族 9F（HTMLBaseElement.href IDL + base 感知 URL 解析）；
> meta color-scheme 10F 归属核对；link/style 事件 30F+18T 分诊
> （runner file 通道能力上限）；style type/media + CSP tentative 待分诊。
> **切片间隙**：本轮 part03/04/06；下一挂账 base 族亦 shim/net 面，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 61 收口，insertRow tbody 目标 + XML 导入链轮）**：
> tabular-data 域第四实施切片：**127P/7F → 130P/4F**（GAIN 3 / REG 0），
> interactive-elements 全域 **439P/44F/0T** 复跑逐值不变，zero-engine
> lib **3454P/0F**。**修复面（R5268）**：①table.insertRow append 位
> （part03）——删 `hasSection` 分支（旧：rows 空 + thead/tfoot 存在时新
> 行**直挂 table**），统一「`_zwTblChildIdxLast` 最后 tbody，无则新建
> tbody 挂 table」——spec append 位 = 最后 tbody 元素子，thead/tfoot
> 不是插入目标（table-insertRow thead/tfoot 案 2F 同修）；连带多 tbody
> append 位从「最后有行的 tbody」纠正为「最后 tbody 元素子」。②快照
> 元素级 getElementsByTagName（part02）——`_zwParseEl` 缺方法（仅 doc 级
> 有），委托 querySelectorAll 一行补齐（tBodies.html XML documentElement
> 查询面）。③importNode 快照域产物物化（part06）——`_zwParseEl`/
> `__zwIsMEl` 源 deep import 经 `_wcRebuildAsHandle` 物化为 handle 树
> （t8q adoptNode 同判据同重建器；raw _zwMEl 副本 append 进活树后
> `.tBodies` 读 undefined 实证）；浅克隆与真 wrapper 源不进分支零变化。
> **验证**：probe 6 案全绿（已删）+ table-insertRow 3/3 + tBodies 1/1 +
> 两域复跑零回归。账本见
> [evidence/2026-10-24-m2s61-insertrow-tbody-xml-import.md](html-semantics-compat/evidence/2026-10-24-m2s61-insertrow-tbody-xml-import.md)。
> **下一步**：tabular-data 余 4F 全在 processing-model-1——rowspan-0×2
> （clientHeight 渲染门）+ col-span-limits×2（span 处理模型），待分诊；
> cells/tBodies HTMLCollection 类型一致化（无断言，低优先）。
> **切片间隙**：本轮 part02/03/06；下一挂账分诊未定，间隙开放，开工前
> 查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 60 收口，table.rows HTMLCollection 轮）**：
> tabular-data 域第三实施切片：**122P/12F → 127P/7F**（GAIN 5 / REG 0），
> interactive-elements 全域 **439P/44F/0T** 复跑逐值不变，zero-engine
> lib **3454P/0F**（+5 为 oracle 流新增测试）。**修复面（R5267）**：
> ①`_zwMakeHTMLCollection` 加 **rebuild 钩子**——[SameObject] 集合的 live
> 读重建形态：每次属性读重算成员表（排序/过滤天然最新），不注册写侧
> live 维护（无 matches 不进 `_zwLiveCollections`；与 matches 形态互斥
> 守卫，既有调用方零变化）。②rows getter（part03）裸数组 → 集合形态：
> `_zwRowsCollCache` WeakMap 缓存（R3908 selectedOptions 同口径，键 =
> 元素 proxy；rebuild 闭包持 sel/handle 原始值防自持环）+ rebuild 每读
> 重算 `_zwTableRowsAll`/`_zwTblChildAll`（s59 地基复用）。table-rows
> 断言面全落既有完备工厂：instanceof（`_zwHCPrototype` 链）、named
> getter（id 全元素 + name 仅 HTML-ns）、ownKeys（[indices…, names…]
> 集合序）、gOPD、canonical 索引；live 断言（removeChild 后 namedItem
> →null）由重建天然过。**范围界定**：仅 rows 收进集合；cells/tBodies
> 保持数组（类型一致化留挂账）；`_rowList`/`_cellList` 消费面经 trap
> 语义兼容零改动。**验证**：table-rows 5/5 全绿 + 连带面逐文件复核零
> 回归（remove-row/insertCell/rowIndex/sectionRowIndex/cells/cellIndex/
> createTBody/table-insertRow 持平）。账本见
> [evidence/2026-10-24-m2s60-table-rows-htmlcollection.md](html-semantics-compat/evidence/2026-10-24-m2s60-table-rows-htmlcollection.md)。
> **下一步**：tabular-data 余 7F——table-insertRow 2F（无 tr 时插入位
> 语义：spec **末 tbody**，现实现首 tbody）+ tBodies 1F（DOMParser XML
> documentElement.getElementsByTagName 缺）可并片；处理模型 4F 待分诊。
> **切片间隙**：本轮 part03/05；下一挂账亦 shim 面——间隙开放，开工前
> 查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 59 收口，tr 族索引/集合读侧轮）**：
> tabular-data 域第二实施切片：**94P/40F → 122P/12F**（GAIN 28 / REG 0），
> interactive-elements 全域 **439P/44F/0T** 复跑逐值不变，zero-engine
> lib **3449P/0F**。**根因（R5266）**：tr 族六 getter（rowIndex/
> cellIndex/sectionRowIndex/rows/tBodies/cells）旧实现经 selector 查询
> 三缺陷——detached script-created 子树不可解析恒空 / 匹配全部后代非
> 直接子 / 无 namespace 过滤。**修复面**：①part03 新地基四件——
> `_zwIsHtmlNsEl`（**严格** xhtml 串判定，探针实证 createElementNS("",
> x) 产物 namespaceURI 读 **null**、解析树/createElement 读 xhtml 串；
> 区别于 `_zwTblChildIdx` 宽松口径不动）+ `_zwTblChildAll` 直接子收集 +
> `_zwTableRowsAll`（spec rows 排序：thead 行 → 直接 tr + tbody 行 →
> tfoot 行）+ `_zwIdxOfSelf` 身份计位（childNodes 缓存 proxy 恒等）。
> ②六 getter 重写：rowIndex 门槛 **parent**（table）或 section+**grandparent**
> table（非最近祖先，table>div>tr 案 -1）；sectionRowIndex 父 table/section；
> cellIndex 父 HTML tr 直接 td/th 子；handle-only created 全覆盖。③has
> trap 名册（part05，R5249 tag-gate 模式）——`"cellIndex" in th` 等恒
> false 补六名。**连带收口**：remove-row 3F + thead/tbody/tfoot rows 3F
> + table-insertRow 1F（`_rowList`/`_cellList` 转读修好的 rows/cells）。
> **lib 测试契约修补**（part15 R3243 三测试）：rows/cells 读侧走
> childNodes 融合视图后，测试换代点只 bump 宿主代际未调 shim 侧挂钩的
> 缺口显形（跨场景 pending 桶残留，探针实证 len=5 含建行）——按
> part12/38/41 既有契约补 `__zw_reset_pending_state` + 两处同场景续跑
> 步先重播种 pristine 快照。**验证**：五目标文件 45 断言全绿；域
> 122P/12F/0T；interactive-elements 439P/44F/0T（REG 0）；lib 3449P/0F；
> fmt/clippy 干净。账本见
> [evidence/2026-10-24-m2s59-tr-index-family.md](html-semantics-compat/evidence/2026-10-24-m2s59-tr-index-family.md)。
> **下一步**（s60 已收口）：table-rows 5F 已清；余挂账 table-insertRow
> 2F / tBodies 1F / 处理模型 4F / cells-tBodies 类型一致化。
>
> **▶ 上轮状态（2026-10-24 M2 切片 58 收口，tabular-data create/delete + setter 族轮）**：
> tabular-data 域首个实施切片：**67P/67F → 94P/40F**（GAIN 27 / REG 0），
> interactive-elements 全域 98 files **439P/44F/0T** 复跑逐值不变。
> **修复面三件（R5265）**：①table create/delete 派生族（part03）——
> createTBody 插**最后一个 HTML-ns tbody 之后**（createTBody.html 15 案
> 钉定，嵌套不作锚）、createTHead 有守卫+before tbody/tfoot、createTFoot
> 有守卫+append、createCaption 有守卫+首子、delete 三族删首匹配直接子；
> 门统一 `_realTag` 后缀取 `.split(':').pop()`（createElementNS 前缀案
> 放行）。②caption/tHead/tFoot getter 收紧（`_zwTblChildIdx` 精确索引 +
> 非 HTML ns 排除）。③IDL setter 族（part05 set trap 新分支 + part04
> set trap 补 receiver）——null 删首匹配、非 section/caption 形态
> TypeError、localName 不符/祖先环 HierarchyRequestError、caption 插
> **首子**（caption_001 firstChild 钉定）、thead/tfoot 有既有先删后原位
> 插（防 selector 重解析误删同形新子）。**本轮 bug 两个同 commit 修**：
> caption 插入段困嵌套 if/else 末臂永不可达（探针入口插桩实锤
> setter 路径从未调 insertBefore；修=卫语句+平铺共享段）；createTBody
> 位置误诊为包装身份断裂（同型元素 identity 断言失败优先怀疑顺序——
> 12 探针逐因子排除后插桩抓 `at:0` 实锤）。**验证**：目标案全绿
>（createTBody 15/15、caption_001 5/5、tHead 3/3、tFoot 2/2）；全域
> interactive-elements 439P/44F/0T（REG 0）；zero-engine lib 3448P/0F。
> 账本见
> [evidence/2026-10-24-m2s58-tabular-create-delete-family.md](html-semantics-compat/evidence/2026-10-24-m2s58-tabular-create-delete-family.md)。
> **下一步**（s59 已收口前两见）：tr 族 21F 已清；余挂账 table-rows
> 5F / remove-row 3F + section rows 3F / insertRow / tBodies 1F / 处理
> 模型 4F。
>
> **▶ 上轮状态（2026-10-24 M2 切片 57 收口，selectAll inert 作用域 + 钉案轮）**：
> 45F 全量复核（逐案读测试源钉断言机制）+ D 族唯一非几何门案收口
>（R5264），全域 438P/45F/0T → **439P/44F/0T**（GAIN 1 / REG 0）。**方案
> 取舍**：s46 五桶分组沿用并逐案核对——`inert-node-is-unselectable`
> 断言链 `document.execCommand('SelectAll')` →
> `getSelection().toString()`，此前 execCommand selectAll 未接（permissive
> no-op → toString 恒 ''），且模态 inert 遮罩是**纯 DOM 树行走**（区别于
> D 族其余案的 gBCR/elementFromPoint 几何门）。**修复**（part06）：
> execCommand 新增 selectall 分支——`_zwDialogModalTopSel()` 命中 → 栈顶
> 模态 dialog 子树，否则 body；复用 `_selection.selectAllChildren`（R2804
> 面）；不派 beforeinput/input（选区命令不经编辑宿主事件序，与 undo/redo
> 同口径）；`_zwQueryCommandState` 补真实反射。**钉案成果**（跨流协调
> 交付物）：余 44F = 渲染流门 ~21F（abspos 居中 6 / default-color 2 /
> overlay 2 / top-layer-position 1 / non-modal 1 / multiple-centered 1 /
> scroll-height 1 / visibility-hidden UA 1 / bounds-clicking 2 /
> modal-selection 拖选几何 1 / dialog-inert 1 / ancestor-is-inert 1 /
> inert-svg-hittest 1 / showModal top-layer 绘制序 1 / closed-details-
> layout-apis 1）+ shadow 底座 20F（closedby-complex 16 + focus-previous-
> outside 3 + form-submit-shadow 1，declarative shadow 待用户点名）+
> corner-cases ESC 1F（s53 定性 divergence 不弯就）。**验证**：目标
> 1F→1P；全域 439P/44F/0T；zero-engine lib 3448P/0F；对照面
> not-highlighted（reftest 视觉）/ with-modal-dialogs-and-iframes /
> 两 crash 测试（语料不载）零波及。账本见
> [evidence/2026-10-24-m2s57-selectall-inert-scope.md](html-semantics-compat/evidence/2026-10-24-m2s57-selectall-inert-scope.md)。
> **下一步**：本流 shim 面清空——跨流协调（渲染流按名取案，飞书已通知）
> + declarative shadow 联合切片待用户点名。**切片间隙**：本轮零 .rs 变更
>（仅 JS shim），下一挂账开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 56 收口，light dismiss 两腿轮）**：
> s49 定性「指针管线深水」的
> dialog-light-dismiss-pointer-capture.html（touch/mouse 双腿）**2F → 2P
> 全收口**（R5263），全域 436P/47F/0T → **438P/45F/0T**（GAIN 2 / REG 0
> / Timeout 0）。**方案取舍**：s49 三洞试错后回退定性指针管线深水；本轮
> 复核推翻前提——管线 capture 重定向面已全通（part04 setPointerCapture
> pending override + part06 `_zwProcessPendingCapture` up 派发前换防 +
> 尾簇 12 换防后取 capturedSel），真缺口唯一：light dismiss 决策仍是
> R4300 单腿（pointerdown 即关），capture 无从影响。spec 已演进两腿
>（down 记 nearest clicked dialog 祖先，up sameTarget 比对后关），实现
> 以**两态分类**等价替换身份模型：down `_zwDialogLdPick` 反文档序首个
> open+any 定类（'in' 链内 / 'out' 链外）存槽，up
> `__zw_dialog_light_dismiss_up` 重分类同态比对，相等且 'out' 才关
>（选择器保持 R5234 口径）；全腿组合等价（in/in、out/in、in/out 不关，
> out/out 关）——capture 面（down 'out' + capture 把 up 重定向 dialog 内
> 'in'）由此不关门。槽卫生：down 终态必写槽（早退也清）+ up 读取即清。
> popover pass 维持 down 腿单发（R5234 绿面零变化）。**验证**：目标
> 2F→2P（pointerId 全管线默认 1 与 `st.capture['1']` 对齐实证）；全域
> 438P/45F/0T；part60 R4 契约两腿化 + out→in capture 门腿（r4d）+ R6
> down+up 对（r4d 首轮 fallback `'button'` 命中 l1 首按钮误关，改
> `'#inner3'` 唯一 id——契约侧 bug）；zero-engine lib 3448P/0F；
> fmt/clippy 清。账本见
> [evidence/2026-10-24-m2s56-dialog-light-dismiss-two-leg.md](html-semantics-compat/evidence/2026-10-24-m2s56-dialog-light-dismiss-two-leg.md)。
> **下一步**：B 族清空，挂账序——C 族渲染流 20F 跨流协调（归渲染流）；
> declarative shadow 联合切片待用户点名。**切片间隙**：本切片为 engine
> shim 面（part01/part06），下一挂账开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 55 收口，xhr-response graft 轮）**：
> s54 分诊轮挂账的 xhr-response 1F **窄域收口**（R5262），F 族
> name-attribute.html 三案全清（**18P/0F**），全域 435P/48F/0T →
> **436P/47F/0T**（GAIN 1 / REG 0）。**方案取舍**：s54 定性 parse-world
> 基面三件套升级（波及 DOMParser/XHR/responseXML 全语料）vs 底座挂账；
> 本轮复核出第三条窄域路线——场景需求全落 `xhr.response.body` 单节点，
> 而 `_zwParseEl` 的 R3019 mutTree 桥（`_ensureMutTree`）本就把该节点惰性
> 物化为 wrapper-world `_zwMEl` 实体子树，仅差物化时机（惰性→主动）、
> 接口原型接线（裸 `_zwMEl` 不接线，R125 在 `_zwMBuildNode` 只覆盖解析
> 子）、`doc.body` 指向（仍是快照壳）。**修复**（part03 XHR HTML 文档
> 分支）：body `_ensureMutTree()` 主动物化 + R125 配方接线
>（`__zwHtmlTagIface.body` → `HTMLBodyElement.prototype`）+ `doc.body`
> 换实体节点。读语义零变化（旧 firstChild 等读面本就惰性经同一 mtree）；
> DOMParser 语料零波及（graft 仅 XHR `responseType='document'` HTML MIME
> 分支，`_zwParsedDoc` 工厂与 `_zwParseEl` 基面未动）；head/documentElement
> 保持快照面（场景不触及）。wrapper-world 实体面经 s54 探针全链验证
>（innerHTML 写解析接 R4100、qSA 返真实 parentNode 链完整节点、open IDL
> + toggle 队列全通）。**验证**：name-attribute 17P/1F → **18P/0F**；
> xhr 语料 stash 前后双侧同跑 net-api xhr 切片 **580P/14F/21T 逐位一致**
>（基线二进制 R5262 锚点 0 / graft 后 1 双验）；escaping.html
>（html-syntax，XHR document 读面消费方）4 子测试全 Pass；zero-engine
> lib 3448P/0F；探针页已删。账本见
> [evidence/2026-10-24-m2s55-xhr-body-graft.md](html-semantics-compat/evidence/2026-10-24-m2s55-xhr-body-graft.md)。
> **下一步**：F 族清空，续挂账序——B 族 pointer-capture 2F 分诊（s49
> 定性指针管线深水）或 C 族渲染流 20F 跨流协调；declarative shadow
> 联合切片待用户点名。**切片间隙**：本切片为 engine shim 面（part03），
> 下一挂账开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 54 收口，name-attribute 3F 分诊轮）**：
> F 族 name-attribute.html 三案分诊完成——**2F shim 收口 + 1F 底座缺口
> 挂账**，全域 433P/50F/0T → **435P/48F/0T**（GAIN 2 / REG 0）。
> **R5260**（mutation event and toggle event order → Pass）：R4009 toggle
> 事件任务队列旧 in-place 去重使 e0 解析期合并事件留队首；spec「queue a
> details toggle event task」合并语义为**移除已排 task + 重排队尾**（
> oldState 取首排程 pre-state）。修复换新 entry 对象 + old 承接
> prevEntry.old——旧 timer 经 fire 闭包身份守卫（`!== entry`）自动作废，
> 兼容 part81 `__zw_setTimeout` stub 环境；spec 推演 FIFO [e3,e2,e1,e0]
> 对齐测试期望。**R5261**（connected-in-template → Pass）：plain wrapper
> 模板 content 视图（part03 `_zwContentView`，模板 childNodes 内联存储
> 形态）缺 ParentNode 元素族 getter，`content.firstElementChild` 恒
> undefined、setup 断言即败；part04 `_tplContent`（sel/handle 形态，
> R151）有全套故 `template-in-disconnected` 本来就过。补
> firstChild/lastChild/children/firstElementChild/lastElementChild/
> childElementCount 六 getter（R151 同款形状）；探针全链：inner
> instanceof HTMLDivElement、innerHTML 解析互斥 [1,0]、qSA 全通。
> **xhr-response 1F 挂账**：`xhr.response.body` 是 `_zwParseEl` 快照（
> protoChain `_zwParseEl > Object > null` 无接口原型），场景链需三件套
> 探针逐步实证——①instanceof HTMLBodyElement（R130 setPrototypeOf 会剪
> 掉 `_zwParseEl.prototype` 全部方法面）；②innerHTML 写解析（ctor own
> data property 赋值仅覆写字符串，R3019 mutTree 桥只读 rewire 无
> setter）；③open IDL（qSA 经 `_zwWrapQ` 返回 `_zwParseEl`，`e.open`
> undefined）。定性 parse-world 写面切片，波及 DOMParser/XHR/responseXML
> 全语料，单独立项排期。**验证**：name-attribute 单案 15P/3F →
> **17P/1F**；zero-engine lib 3448P/0F（part60 R4100 契约 per-element
> 形态不断言跨元素序，重排队零回归）；探针页已删；fmt/clippy 不适用（
> diff 仅 JS shim）。账本见
> [evidence/2026-10-24-m2s54-details-name-3f-triage.md](html-semantics-compat/evidence/2026-10-24-m2s54-details-name-3f-triage.md)。
> **下一步**：parse-world 写面三件套切片（接口原型双链保留 / innerHTML
> setter 进解析 / open IDL）排期，或续挂账序 B 族 pointer-capture 2F、
> C 族渲染流 20F；declarative shadow 联合切片待用户点名。**切片间隙**：
> 本切片为 engine shim 面（part03），下一挂账开工前查 oracle master
> 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 53 收口，零代码轮）**：corner-cases
> 「focusin calls showModal, modeless」ESC 1F 复审定性——**spec 一致行为，
> 测试期望分歧**。四重证据同向（ESC 后 dialog 重开 open=true）：live
> spec close-the-dialog 恢复门（anchor 后代 OR wasModal → focusing steps
> → focusin → 处理器 showModal；showModal 前置守卫无一命中、无重入门）；
> Chromium main 源码（Close() 焦点恢复在 is_closing_ 释放后、showModal
> 无守卫）；直探 Chromium 151 与 **Chrome for Testing 158.0.8118.0
>（wpt.fyi 同 build）**真 ESC——皆重开；我方 runner 探针序列与直探逐步
> 同构。**反证与矛盾**：wpt.fyi 同 revision（c645564b0e，2026-10-23
> run）chrome-158 + firefox-161 summary 双双 30/30 绿（压缩格式已验证
> c=[passes,total]）——与同 build 直探矛盾只能落 wptrunner 环境层（页面
> 可见语义内构造不出 closed 读点世界）；s41 依据「Chrome-parity 同红」
>（crbug 417571176，2026-04）已被双引擎绿取代（该 crbug 的 CL 即 wpt
> #58337 断言形式修复）。**收口**：1F 保留标 spec-consistent known
> divergence，不弯就（s41 R5244 试错实测 REG 4 + 违背 spec/直探）；
> 全域 433P/50F/0T 零变化。账本见
> [evidence/2026-10-24-m2s53-corner-cases-esc-divergence.md](html-semantics-compat/evidence/2026-10-24-m2s53-corner-cases-esc-divergence.md)。
> **下一步**：F 族 name-attribute 3F 分诊（mutation events / XHR 解析 /
> template 附着底座缺失——评 shim 可收口面 vs 底座缺口）；B 族
> pointer-capture 2F（s49 定性指针管线深水）、C 族渲染流 20F 维持跨流
> 挂账；declarative shadow 联合切片待用户点名。**切片间隙**：本轮零代码
> 未触 engine 生产面，间隙维持开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 52 收口）**：R5257 focus 滚动进视口
> + R5258 moveBefore——**interactive-elements 文件级 Timeout 清零**（50F
> 全为明确 Fail）。**分诊**（本轮刀前）：shim 面红名单三案——
> focus-after-close 文件级 Timeout（6/7 Pass 悬挂末腿）、
> movebefore-setup 1F（`new_parent.moveBefore` TypeError）、showModal 1F
> （rendering on top——top-layer 绘制序渲染流面，转 C 族记账）。**R5257**
>（focus-after-close Timeout → 7 子测试全 Pass）：探针双根因——proxy
> focus() 无滚动步（spec focusing steps 尾 UA 滚动面）+ 视口 scroll 事件
> 只派 window（document 监听永收不到）。三修：①focus 尾 nearest 滚动
>（gBCR 出界才滚，R3047/R3051 面复用）；②几何 fallback——动态创建元素
> 无布局几何（runner gBCR 零盒实证），读 proxy inline style（handle-only
> 可达）position 非 static + top 可解析 → 文档 y 顶对齐（FIXME: 样式表
> 定位随渲染流底座）；③no-scroll 变体 `__zwFocusNoScroll` 旗——R4600
> dialog 内部 focusing steps（show/showModal/close 恢复/fixup）包旗不滚
>（Chromium close 恢复不滚，同测试 scrollTop==0 断言面）；④`_zwFireScroll`
> 补 document 级派发（非冒泡 bubble 腿门控，window 不双计）。**R5258**
>（movebefore-setup 1F → 1P）：Element.moveBefore = WebIDL 校验 + 委派
> insertBefore 机械——免 removal steps 语义（shim 移动面本无 dialog 删除
> 钩子，组栈/open 态天然原地保持；FIXME: 跨文档同文档门随 parser 批次）；
> ESC 断言链走既有 R5236/R4300 组栈序。**契约**（part83）：nearest 滚动
> 位 + 双通道恰一次/showModal-close 零滚动 + 焦点恢复/moveBefore open 态
> 保持 + ESC 关组栈顶/NotFoundError 零滚动。**验证**：focus-after-close
> **Timeout → 7 全 Pass**、movebefore-setup **1F → 1P**；全域 98 files
> 483 subtest **433P/50F/0T 对 s51 逐位 REG 0 / GAIN 3**；zero-engine lib
> 3448P/0F（+契约 part83）；fmt/clippy clean（test-guard 包裹）；release
> runner 含 R5257×4/R5258×1 锚点。账本见
> [evidence/2026-10-24-m2s52-focus-scroll-movebefore.md](html-semantics-compat/evidence/2026-10-24-m2s52-focus-scroll-movebefore.md)。
> **下一步**：declarative shadow 联合切片仍待用户点名（渲染流协调）；
> 挂账序续 C 族渲染流 20F（top-layer 绘制序 showModal 1F / modal-selection
> 1F / inert 族 4F——elementFromPoint/hit-test 缺口）与 B 族指针管线深水、
> F 族 name-attribute 3F 逐案分诊。**切片间隙**：本切片为 engine shim 面
>（part01/04 + 契约 part83），下一挂账 C 族为渲染流面需跨流协调，开工前
> 查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 51 收口）**：R5256 dialog 属性移除面
> + A/H 分诊改记。**分诊**（本轮刀前）：A 族 closedby×slot 16F（
> dialog-popover-closedby-complex 全部「shadow DOM slots」变体）与 H 前置
> 4F **同根**——探针实证 `<template shadowrootmode=open>` 在页面视图被
> 拆包平铺（part03:7639「parser 占位」互证）、shadowRoot null、shim
> attachShadow 虚拟树不下传 dom（attach_shadow 生产零接线）、shadow 内容
> rect=0/hit-test null（几何面未通）；clickOn 坐标驱动下 shim 虚拟单边
> 提升会 REG 现存 4 Pass（点击落 (0,0) 全员误关）。**改记**：A+H 统一
> 「declarative shadow parser 提升 + shadow 几何渲染流联合切片」，渲染流
> 半边维持用户决策门禁（web-components 归档流明示等点名协调
> rendering-compat；与 parser form pointer 缓行决策同域）。**R5256 两刀**
>（dialog shim 面）：①open 属性移除面关闭步——`_zwDialogAttrClosedR5256`
>（part01）清模态/组栈 + close/toggle 排队，无 cancel/无 beforetoggle/
> 焦点不恢复/returnValue 不变（tentative 断言契约；旧注记「不派 close」
> 已被 2025 attribute change steps 变更推翻）；removeAttribute 尾钩 +
> IDL falsy 臂双接通，**close watcher 门**（pre-snapshot 模态态/组栈双
> 信号——属性直开无 watcher 仅隐藏，首版 REG dialog-open 一案当场修正）；
> ②detached 文档 dialog 三方法——showModal 抛 InvalidStateError、show
> 静默返、close no-op（dialog-active-document 旧产物 TypeError 面）。
> **契约**（part82）：模态移除/IDL falsy/wasOpen 门/同任务 toggle 合并/
> detached 三方法/属性直开零 close 六腿。**验证**：close-via-attribute
> **2F → 2P**、active-document **1F → 1P**、dialog-open 维持 3P；全域
> 98 files 483 subtest **431P/51F/1T 对 s50 逐位 REG 0 / GAIN 3**；
> zero-engine lib 3447P/0F（+契约 part82）；fmt/clippy clean（test-guard
> 包裹）；release runner 含 R5256 锚点。账本见
> [evidence/2026-10-24-m2s51-dialog-attr-removal.md](html-semantics-compat/evidence/2026-10-24-m2s51-dialog-attr-removal.md)。
> **下一步**：declarative shadow 联合切片待用户点名（渲染流协调）；续
> dialog shim 面红名单（focus-after-close 1T / showModal 1F /
> dialog-inert 1F 等逐案分诊）。C 族渲染流 20F 与指针管线维持跨子系统
> 单独排期。**切片间隙**：本切片为 engine shim 面（part01/03/04），
> 下一挂账预期仍 shim 面或待决策项，间隙开放，开工前查 oracle master
> 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 50 收口）**：toggleEvent 1T 收口
> （R5254 parser 初始 open details toggle 补偿两段式）。**分诊**：
> toggleEvent.html 文件级 Timeout——11 async_test 中 9 Pass、pending: 2
> 永挂（t9 依赖 inline ontoggle 在 parser 声明 open 时 resolve promise；
> 末案 DOMParser 产物 ontoggle）。根因：R4009 toggle 队列只挂写路径尾钩
>（IDL setter / setAttribute / removeAttribute），parser 初始 open 由
> host 解析直建不经 JS 写路径；innerHTML / createElement-HTML / 包装
> 世界三解析面已接 R4100 扫描，**唯主文档初始 HTML 与 DOMParser 快照面
> 缺**。**两段式**（单段直扫两难均实证：注册期直扫 task 派发早于页面
> 脚本——runner 面 arm 到真线程 timer，t9 resolver 未就位永挂；DCL 直
> 扫分不清 parser open 与运行时 open——activation-behavior coalesced 案
> toggle 计数 2≠1 回归）：注册期 mark（`__zwDetailsInitOpenMark5254`，
> 页面脚本前文档纯 parser 态，open 者身份 key 入集）+ DCL fire
>（`__zwDetailsInitOpenFire5254`，capture listener，只对「初始 open 且
> 当前仍 open」走既有 `_zwToggleQueueNotify4009` 入队，标记集一次性消
> 费）；`__zw_reset_form_state` 尾清集重标记（跨导航）。**刀二
> DOMParser**：快照架构死穴——扫描快照 ≠ 用户 querySelector 手持快照，
> 实例属性 handler 不可达（首版直调计数 0 实证）；改 parseFromString
> 返回前扫 `details[open]`（打 open IDL getter + setTimeout task）+
> `_zwParseEl.prototype` ontoggle accessor 按身份键（`_zwEvKey`，同源
> 重解析稳定）登记 `_zwParsedToggleH` 全局表，task 经身份键寻址派发。
> **契约**（part81）：(a) 主文档初始 open 恰好一次 toggle（closed→open
> + target.open 真值）+ 同态写不重派（t9 断言面）；(b) 初始 closed 不
> 派；(c) DOMParser 产物收到 toggle + target.open 真值；(c2) 无 open
> details 零 task。时序模拟：记录式 `__zw_setTimeout` stub + 手动
> `__zw_fire` + 手动派 DCL。**验证**：toggleEvent.html **Timeout → 11
> 子测试全 Pass**；activation-behavior coalesced 案两段式回绿；全域
> 98 files **483 subtest 428P/54F/1T 对 s49 逐位 REG 0 / GAIN 2**
>（fail 集逐位一致，子测试总数 482→483 为 Timeout 不再吞 2 案）；
> zero-engine lib 3446P/0F（+契约 part81）；fmt/clippy clean
>（test-guard 包裹）；release runner 含 R5254 锚点（grep 实证）。
> 账本见
> [evidence/2026-10-24-m2s50-parser-open-toggle.md](html-semantics-compat/evidence/2026-10-24-m2s50-parser-open-toggle.md)。
> **下一步**：按挂账序候选 A 族 closedby×slot 16F（Chrome-parity 同红
> 钉住，slot 分配 shim 面）或 H 前置 4F（declarative shadow parser +
> 渲染流联合切片）；F 族 name-attribute 3F 与 DOMParser 快照架构
>（live 文档面）维持挂账。**切片间隙**：本切片为 engine shim 面
>（part01/02/03 + callbacks.rs 注册尾），下一挂账 A/H 均预期跨
> parser/shim/渲染流，间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 49 收口）**：B 族 light dismiss 5F 全挂账
> 零净变更 + F 族分诊 + R5253 收口 details auto-expand 1F。**B 族定性**：
> bounds-clicking 2F——探针实证 elementFromPoint 对 dialog 自身盒命中返
> null（modal+modeless 同断、后代正常），渲染流命中缺口并 C 族记账；
> pointer-capture 2F——R5252 两腿模型（down/up 双 target 判定 + capture
> retarget 救回）试错三洞（backdrop retarget 使 down 腿 contains=true 短
> 路漏记、capture 残留未隐式释放误救回、corner-cases modal 组入口永挂），
> git 回退定性**指针管线深水**，需先有指针序列契约面再动；corner-cases
> 1F 既有挂账（modeless ESC 竞态）。**F 族分诊**：closed-details-layout-
> apis 1F 渲染流面；name-attribute 3F（mutation events / XHR 解析 /
> template 附着底座未实现）；toggleEvent 1T 文件级 Timeout（9 子测试
> Pass 后 completion 未触发，未深查待分诊）。**R5253 两刀**（part02
> `_scrollToAnchorForHash`）：几何量测前插 spec 展开步（target 链逐个
> setAttribute('open','')，toggle/name 组链自动）+ 仅真展开时挂 rAF 补偿
> 臂（属性 mutation 批应用后重入：展开步 no-op + 既有滚动按新几何落位，
> 既有滚动面零变化）。**契约环境差异注记**：bridge 契约 sandbox 无帧驱
> 动/布局供给面，滚动落位不可建模（帧驱动 + 手动 tick 仍红）——契约钉
> (a) 展开/(b) 链外不误开/(c) no-crash，落位面由 WPT 单案承载。**验证**：
> auto-expand 单案 **Fail → Pass**；interactive-elements 全域 98 files
> 482 subtest **426P/54F/2T 对 s48 逐位 REG 0 / GAIN 1**；zero-engine
> lib 3445P/0F（+契约 part80）；fmt/clippy clean（test-guard 包裹）；
> release runner 含 R5253 JS 串锚点。账本见
> [evidence/2026-10-24-m2s49-fragment-details-expand.md](html-semantics-compat/evidence/2026-10-24-m2s49-fragment-details-expand.md)。
> **下一步**：按挂账序候选 A 族 closedby×slot 16F（Chrome-parity 同红
> 钉住）或 H 前置 4F（declarative shadow parser+渲染流联合切片）；
> toggleEvent 1T 分诊可作小切片插入。C 族渲染流 20F 与指针管线两腿模型
> 需跨子系统协调，单独排期。**切片间隙**：本切片为 engine shim 面
>（part02 fragment 导航），下一挂账 A/H 均预期跨 parser/shim/渲染流，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 48 收口）**：D 族 focus/inert 10F
> 逐案单跑取证后重分诊——可收口面仅 focusability 1F（R5251）。**重
> 分诊**：focus-previous-outside 3F + form-submit-dialog-shadow 1F →
> **挂账（H 族前置）**——首层根因是 `<template shadowrootmode=open>`
> 声明式 shadow root 从未被提升（`host.shadowRoot` 返 null，
> `querySelector` 即炸），shim 全 part 无 shadowrootmode 处理
>（attachShadow 命令式已有），4F 记入 H 族「parser+渲染流联合切片」
> 范围，R4600 焦点恢复 shadow/slot 边界是其后续面；dialog-inert 1F +
> modal-dialog-ancestor-is-inert 1F + inert-svg-hittest 1F +
> inert-node-is-unselectable 1F + modal-dialog-selection 1F → **挂账
>（渲染流面）**——elementFromPoint 对 open dialog 返 null / 点击穿透
> inert 祖先 / 选择窗口空串，同底座 = 命中测试几何链缺失，与 C 族
> top-layer/UA 布局 15F 合并记账。**根因**（focusability）：Tab 可聚焦
> 判定两处均无 editing host 分支——runner `dispatch_tab_focus` 页面候选
> 序选择器不含 contenteditable 宿主（本案观测路径，testdriver
> send_keys TAB）；host `FocusManager::scan_node` 同缺口（MoveFocus 效果
> 面）。spec：editing host 是 focusable area（HTML §focusable-area）。**落
> 地（R5251 两刀）**：runner dispatch_tab_focus 候选选择器尾补
> `[contenteditable]` + host 门（`isContentEditable`（R3187 枚举态）真
> 且父不可编辑 → 入零序；嵌套 editable 非 host / `contenteditable=false`
> 孤岛不入序；显式 tabindex 优先原臂，原生可聚焦元素落原 else 臂——首版
> 误让 button 落 CE 臂 continue 掉，回归当场修正）；dom FocusManager 补
> `contenteditable_state`/`is_editable`/`is_editing_host` 三 helper +
> scan_node 零序分支。**验证**：dialog-focusability 单案目标子测试
> **Fail → Pass**；interactive-elements 全域 98 files 482 subtest
> **425P/55F/2T 对 s47 逐位 REG 0 / GAIN 1**；zero-dom lib 866P/0F
>（+3 新单测，fail-first 先行）；zero-engine lib 3444P/0F（+契约
> part02 R5251）；fmt/clippy clean（test-guard 包裹）；release runner
> 含 R5251 JS 串锚点。账本见
> [evidence/2026-10-24-m2s48-editing-host-focusable.md](html-semantics-compat/evidence/2026-10-24-m2s48-editing-host-focusable.md)。
> **下一步**：按挂账序转 B 族 light dismiss 5F 分诊（invoker/backdrop
> 面）；H 前置 4F 与渲染流面 5F（并入 C 族记账）维持挂账；parser form
> element pointer 4F **已缓行**（2026-10-23 用户决策：与 declarative
> shadow DOM（H）打捆 parser 批次、M2 收口后尾段单独立项）。
> **切片间隙**：本切片为 runner 页面序脚本面（testharness.rs）+ dom
> FocusManager 面（crates/dom）——下一挂账 B 族 light dismiss 预期
> engine shim 面（part01 dialog 事件），间隙开放，开工前查 oracle
> master 碰撞记录。
>
> **▶ 上轮状态（2026-10-23 M2 切片 46 收口）**：余 60F 归因分诊落档（六族
> 地图——A closedby×slot 交叉 16F（crbug 417571176 Chrome-parity 同红）、
> B light dismiss 5F、C top-layer/UA 布局 15F（渲染流面）、D focus/inert
> 10F、E dialog API 语义 5F（可 shim 收口候选）、F details 零散 4F/1T +
> G 历史 menu 面 3F）+ 族 G 顺手收口（R5249）。**根因**（三半边「IDL 名
> 无 tag 门」同模式）：①part05 has trap track IDL 列 label/kind/srclang/
> default（media 切片 3 加）无条件命中——历史 menu/menuitem（menuitem 已
> 删为 HTMLUnknownElement）`'label' in el` 假阳性；②set trap R3069 臂
> FLAT 表 type 无 tag 排除——`menu.type=` 误写内容属性；③同臂 label 显式
> 列（R3069-fix）无 tag 门。**落地（R5249 双刀）**：has trap kind/srclang
> 收 TRACK 门、label/default 收 TRACK+OPTION 门（HTMLOptionElement 读侧
> R3069-fix 对称）；R3069 臂条件首项 MENU/MENUITEM 负门——赋值落 expando、
> 内容属性不变，name 臂自带门与 FLAT 其余名零变化。**验证**：
> contextmenu-historical 单案 **11/11 Pass**；interactive-elements 全域
> 98 files 482 subtest **423P/57F/2T 对 s45 逐位 REG 0 / GAIN 3**（GAIN
> 即三案）；lib **3442P/0F**（+2 契约 part78）；契约翻转力实证（tag 读恒
> TRACK → 钉 1(a) 红；负门短路 → 钉 1(c) 红）。**已知边界**：menu.type/
> label 读侧 FLAT 读仍返内容属性值（真浏览器 undefined，测试无此面）；
> track 接口其余成员 in 仍无条件（corpus 无 menu 面观测）。gates：clippy
> --workspace -D warnings（test-guard 包裹）/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s46-historical-menu-idl-gate.md](html-semantics-compat/evidence/2026-10-23-m2s46-historical-menu-idl-gate.md)。
> **下一步**：按挂账序 E 族 dialog API 语义 5F（close-via-attribute 移除
> open 属性触发 closing/close 事件 2F 为首靶，active-document/showModal
> throw/movebefore 跟进）；parser form element pointer 4F **已缓行**
>（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆 parser 批次、
> M2 收口后尾段单独立项）；declarative shadow DOM 同打捆待触发。**切片
> 间隙**：本切片为 shim 面，下一挂账 E 族亦 shim 面——间隙开放，开工前查
> oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-23 M2 切片 45 收口）**：s37 已知边界「点击
> summary-less details 内非激活后代不翻转」收口（2026-10-23 重新定性：
> spec 无 summary 翻转是 UA affordance 非 MUST，corpus 无观测面——性质
> 同 s42/s43：spec 正确性收口 + 契约钉住）。**根因**（通道分裂）：details
> 激活面两条通道各断半边——①`.click()` 合成通道 part04 click 默认动作段
> 只处理 receiver 自身三站（LABEL/SUMMARY/DETAILS），非激活后代无 nearest
> 上溯；②JS dispatch 通道 part03 R112 post act:details 门
> `_zwUaDispatch === true` 挡 `new MouseEvent`（无 UA 印记、无 synthetic
> 标）。**落地（R5248 双刀）**：part04 nearest 上溯分支——receiver 非
> 三站且 `_zwFindClickActivation` 命中 act:details 时
> `_zwDetailsActivateR4400` 翻转（spec the-details-element activation
> behaviour + legacy-pre-activation nearest ancestor (or self) with
> activation behavior；notPrevented 门内；receiver 三站让位防双翻；交互
> 后代 act:button/act:a/act:summary/checkbox 命中时 type 豁免自然不落）；
> part03 门放宽 `(_zwUaDispatch === true || _zwSyntheticClick !== true)`
> ——JS dispatch 进同款翻转，`.click()` 合成标保持不进（单翻互斥），
> preventDefault/connected 门由消费块入口守卫继承；hasSum 门槛让位不变
> （summary 站优先，R4009 面不回归）。**验证**：探针七案修后全翻且单翻
> 互斥成立；interactive-elements 全域 98 files 482 subtest **420P/60F/2T
> 对 s44 逐位 REG 0 / GAIN 0**（R4400 既有面无回归，corpus 无观测面无
> GAIN）；lib **3440P/0F**（+2 契约 part77）；契约翻转力实证（nearest
> 分支短路 → 钉 1(a) 红；门收紧 → 钉 1(c) 红）。**已知边界**：corpus 无
> 观测面（无 GAIN）；detached details 后代点击不翻转（R112 connected
> 门，spec 无条件但 corpus 无观测面）维持现状。gates：clippy --workspace
> -D warnings（test-guard 包裹）/fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md](html-semantics-compat/evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md)。
> **下一步**：按挂账序其余余案归因；parser form element pointer 4F
> **已缓行**（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆
> parser 批次、M2 收口后尾段单独立项）；declarative shadow DOM 同打捆
> 待触发。**切片间隙**：本切片为 shim 面，下一挂账开工前查 oracle
> master 碰撞记录（间隙开放）。
>
> **▶ 上轮状态（2026-10-23 M2 切片 44 收口）**：s39 剩余 1F「focus-after-close
> Complex dialog usage focus 恢复红（activeElement 得 div）」收口。**根因**
> （六层钻透）：Complex 案跨 testdriver 多命令 execute_script，宿主 apply
> 换代发生在 show 与 close 之间——①静态树节点 handle 形 proxy 无
> `_zwNodeParent` 反链记账（R91 只记 JS 动态操作）→ ②isConnected 反链
> miss + rect 探测兜底空 → 双假阴性 disconnected → ③R114 detached 臂误入
> → ④activeElement getter 的 R4600 focus-chain retarget 上行解析出
> **runner 投影容器形**——BND dump 实证 `_shadowHandles` 跨执行残留注册
> （`__n1/__n3/__n5/__n8`，meta hostSel:null、hostHandle 指向换代后复用的
> 活 handle）→ 误穿边界回吐投影 host DIV；⑤host tagName 防线被复用活
> handle 绕过、⑥`hostSel==null` 判据亦不成立（页面自建 shadow 的动态 host
> meta.hostSel 同为 null）——均不可判据。**落地（R5247，判据回归 spec）**：
> part04 getter + part01 `_zwIsConnected` 前置复核——`__zw_selector_for_handle`
> 宿主持久反查（不随换代清）+ querySelector 同一性核验（命中同 handle 且
> `__zw_contains` 在档才短路 true；`_zwRemovedHandles` 守门保持 remove 语义；
> shadow 树内锚宿主扁平 selector 命中别元素不短路——R91 反链 shadow 穿越
> 权威判定保持；part04 复核须前置——反链循环含中途 return 后置不可达）；
> part06 activeElement getter retarget 语义前提门——锚经同一性核验确认
> light 树直返锚（spec 焦点链 retargeting 仅当锚身处 shadow 树才穿边界；
> 判据与 stale 注册形态解耦），host 解析空壳兜底回落原锚。**验证**：
> focus-after-close 单案 **6/6 Pass**（Complex + shadow 双案同时翻绿）；
> interactive-elements 全域 98 files 482 subtest **420P/60F/2T 对 s43 逐位
> REG 0 / GAIN 1**（Fail 清单 diff 仅 Complex 案一行）；lib **3437P/0F**
>（+2 契约 part76）；契约翻转力实证（撤前置复核 → 钉 1(a) 红；light 门
> 无条件短路 → 钉 2(b) 红）。**已知边界**：focus-after-close 文件级
> Timeout（dont_scroll：focus 不滚 viewport → scroll 永不发 → promise
> 永挂）维持 s39 既有独立挂账；dialog-focus-previous-outside 3F 既有挂账。
> gates：clippy --workspace -D warnings（test-guard 包裹）/fmt clean、零
> 新增依赖。账本见
> [evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md](html-semantics-compat/evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md)。
> **下一步**：按挂账序 summary-less details 内非激活后代翻转（s37 邻近
> 缺口，真浏览器会翻转；2026-10-23 重新定性：spec 无 summary 翻转是 UA
> affordance 非 MUST，corpus 无观测面，优先级维持挂账序）；parser form
> element pointer 4F **已缓行**（2026-10-23 用户决策：与 declarative
> shadow DOM（H）打捆 parser 批次、M2 收口后尾段单独立项）；declarative
> shadow DOM 同打捆待触发。**切片间隙**：本切片为 shim 面，下一挂账
> summary-less 翻转亦 shim 面——间隙开放，开工前查 oracle master 碰撞
> 记录。
>
> **▶ 上轮状态（2026-10-23 M2 切片 43 收口）**：挂账「replaceWith
> self-removal 父桶记账」收口（s26 既有挂账，R371 模板残留）。**根因**：
> replaceWith sel 臂 remove 步只覆盖 apply 面（Remove/RemoveHandle
> mutation）与 sel/handle 字符串标记表（parentNode/isConnected 读），合并
> 视图子读（childNodes overlay / R322 归并查询）按**父桶 removed identity**
> 剔除——桶缺 removed 项 → 被替换者（快照 sel 节点）在合并视图残留至 apply
> 换代，视图 ≠ spec 终态。**落地（R5246 一刀）**：remove 步后经 canonical
> 记账入口 `_zwHCLiveInvalidate([], [被替换者 proxy], 父sel, null)` 直调
> （父桶 removed + 全局 pending-removed + live 集合失效一体；子树展平含
> template 内容子）；proxy identity 取基列表条目（`_r371Self`，与 overlay
> 剔除同一缓存对象）；记账先于 R371 rekey → dst 桶必存在，rekey 走合并分支
>（promote 分支保留护栏）。**验证**：契约
> test_dialog_replacewith_parent_bucket_accounting_r5246（part75 新文件，
> R1 corpus 同形全树视图退出 + R2 body 直子面）+ 既有
> r321_replacewith_fragment_flatten 断言更新（kids 前导宿主残留 DIV 消失，
> 视图即 spec 终态）；interactive-elements 全域 98 files 482 subtest
> **419P/61F/2T 对 s42 逐位 REG 0 / GAIN 0**（挂账本就「对扫描无害」，
> corpus 无观测面——切片性质同 s42：spec 正确性收口 + 契约钉住）；lib
> **3434P/0F**。**已知边界**：sel 臂 replaceWith 的 MO record 面（插入/
> 移除两段，R117 handle 臂已有）独立挂账不扩面。gates：clippy
> --workspace -D warnings（test-guard 包裹）/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s43-replacewith-parent-bucket.md](html-semantics-compat/evidence/2026-10-23-m2s43-replacewith-parent-bucket.md)。
> **下一步**：按挂账序 summary-less details 内非激活后代翻转（s37 邻近
> 缺口，真浏览器会翻转）、focus-after-close Complex dialog usage focus
> 恢复深层（s39 剩余 1F）；parser form element pointer 4F **已缓行**
>（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆 parser
> 批次、M2 收口后尾段单独立项）；declarative shadow DOM 同打捆待触发。
>
> **切片间隙宣告义务（2026-10-23 用户决策，配合
> [oracle 流解封协议](archive/incremental-rendering-oracle/master.md)）**：每个
> shim 面切片 push 后，若下一挂账不触 engine 生产面（余下挂账均为 shim
> 面），在「当前状态」宣告切片间隙开放；开工下一切片前先查 oracle
> master 碰撞记录有无占用标记，有则等待或改排其他 shim 挂账。
>
> **▶ 上轮状态（2026-10-23 M2 切片 41 收口）**：挂账「ESC 组栈边缘乱序」
> 定性收口（非代码轮）。**探针钉定**（zz-probe-s41，验证后已删）：
> corner-cases「focusin calls showModal, modeless」ESC 案事件序 =
> show() 聚焦空 modeless dialog（subject fallback）→ close-the-dialog
> 焦点恢复尾步 anchor 门命中（焦点锚=subject 后代）→ prevFocused
> focusing steps 触 focusin → 处理器 re-entrant showModal 重开。**spec
> 对账**：恢复门（锚 shadow-including 后代 OR wasModal）、show()/
> showModal() 双记录 prevFocused、focusing steps「not focusable →
> nothing」与现行文本逐字一致——R4600/R5236 无漂移。**试错回退**
> （R5244 subject 非 focusable 门，未合入）：目标案 GAIN 1 但 REG 4
> （dialog-focus-shadow「show:」三臂 + focus-previous-outside——Chrome
> 断言面「show() 无 focus delegate 聚焦 dialog 自身」，whatwg/html#8199
> 只移除 TAB 序可聚焦），两面矛盾 Chrome 选边聚焦面。**Chrome-parity
> 同红证据**：crbug 417571176（父 376516550 closedby 特性）2026-04
> 仍 "still failing according to wpt.fyi"，上游已改一次 assertion 仍未
> 过；本项目按 Chrome 拉齐，修复后随 corpus 刷新自然转绿。**收口**：
> 挂账关闭（语料钉住 + crbug 引用入账）；interactive-elements 全域
> 98 files **419P/61F/2T 对 s40 逐位 REG 0**、lib 3432P/0F、零代码净
> 变化。账本见
> [evidence/2026-10-23-m2s41-esc-stacked-parity.md](html-semantics-compat/evidence/2026-10-23-m2s41-esc-stacked-parity.md)。
> **下一步**：按挂账序非模态 close 不出组栈、R371 模板残留、
> summary-less details 内非激活后代翻转（s37）、focus-after-close
> Complex dialog usage focus 恢复深层（s39 剩余 1F）；parser form
> element pointer 4F **已缓行**（2026-10-23 用户决策：与 declarative
> shadow DOM（H）打捆 parser 批次、M2 收口后尾段单独立项）；
> declarative shadow DOM 同打捆待触发。
>
> **▶ 上轮状态（2026-10-23 M2 切片 40 收口）**：挂账 custom command
> （`--` 前缀）CommandEvent 派发收口（s24 defer）。**spec 钉定**：现行
> button activation behavior——validity Custom 态对任意元素 true →
> fire command 事件（command=值本身大小写保留、source=invoker、
> cancelable=true）→ Custom 态 return 无内建行为；派发形状
> bubbles=false/composed=true/isTrusted=true/无 relatedTarget（上游
> button-event-dispatch 语料钉死）。**落地（R5243 两刀）**：①part05
> `_defineEventSubclass('CommandEvent','Event',[command,source])`
> （ToggleEvent 同厂）；②part01 `_zwInvokerCommandActivate` custom
> 分支构造 CommandEvent + trusted 印记（R312 通道）经
> `_dispatchWithBubble` 在 target 派发后 return——分派在既有调用方
> 门内（disabled/preventDefault 零新写），`cmdCustom` 原值与 `cmd`
> 小写拆分、内建关键字面不变。**集群**：interactive-elements 全域
> 98 files 482 subtest **419P/61F/2T 与 s39 逐位打平**；dialog 集群
> 82 files 对 s35 账本 **REG 0 / GAIN 5**（全为 s39 既有收益）；
> forms/the-button-element 18 files 54 subtest 对 s24 逐位一致
> **REG 0**（corpus 零 custom command 消费）。**已知边界**：现行
> spec「invalid → Unknown 不派事件」未跟进（维持 R4400 auto 映射，
> corpus 钉住，跟进涉 dialog/details 内建关键字重构独立切片）；
> popover 关键字 + oncommand IDL + button command/commandFor IDL
> 反射维持挂账。gates：lib **3432P/0F**（+1 契约 part73
> test_invoker_custom_command_event_r5243）、clippy/fmt clean、零
> 新增依赖。账本见
> [evidence/2026-10-23-m2s40-custom-command-event.md](html-semantics-compat/evidence/2026-10-23-m2s40-custom-command-event.md)。
> **下一步**：按挂账序 ESC 组栈边缘乱序、非模态 close 不出组栈、
> R371 模板残留、summary-less details 内非激活后代翻转（s37）、
> focus-after-close Complex dialog usage focus 恢复深层（s39 剩余
> 1F）；parser form element pointer 4F **已缓行**（2026-10-23 用户
> 决策：与 declarative shadow DOM（H）打捆 parser 批次、M2 收口后
> 尾段单独立项）；declarative shadow DOM 同打捆待触发。
>
> **▶ 上轮状态（2026-10-23 M2 切片 39 收口）**：挂账 focus-in-shadow
> 跨流收口（focus-after-close「moved to shadow root」案 +
> dialog-focus-shadow-double-nested）。**探针钉定**（zz-probe 五迭代，
> 验证后已删）四层根因：①sel 模板 content 子列表 per-read 重建，跨读
> mutation 落空（spec content getter 恒返同一 fragment）；②content
> 条目 parentNode 指模板 wrapper，replaceWith 落错列表（探针 D
> `pidIsView=false`）；③`_zwMEl.replaceWith` 只有 remove 半边，替换者
> 落空；④focus delegate 非 dialog 分支缺 get-the-focusable-area 递归，
> 嵌套 shadow host 被跳过后焦点落树序后一 focusable（探针 E
> `osraeTag=BUTTON`≠autofocus 锚）。**落地（R5241 四刀 + R5242 一刀 +
> 收尾失效）**：①part04 content 子列表首读物化缓存（缓存数组即活
> 树）；②parentNode/parentElement getter 槽位消费补 parentHandle 分支
> （focus chain 上行达 shadow 边界）；③`_tcParent = _tplContent`（条
> 目父 = 视图本体）；④part03 replaceWith 补插入半边（plain 父门控，
> host proxy 面零变化）；⑤part01 focus delegate 递归（dialog 分支
> seq-focusable 优先序不动）；⑥innerHTML= setter sel 模板缓存失效
> （R5196 lib 红面收口，视图身份不动）。**集群**：
> double-nested **2F → 2P 全绿**；focus-after-close **2F+1T → 1F+1T**；
> interactive-elements 全域 98 files 419P/61F/2T 对 s38 账本
> **REG 0 / GAIN 3**；forms 域 401 files 3902P/707F/20T/1U
> **REG 0 / GAIN 3**（GAIN 为 label 文件尾 3 案 s38 跑时截断未注册，
> 共同 48 案逐位一致）。**已知边界**：focus-after-close 余 1F
> （Complex dialog usage focus 恢复，既有）；content 缓存不回灌非
> innerHTML= 的宿主侧子树变化。gates：lib **3431P/0F**、clippy/fmt
> clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s39-focus-in-shadow.md](html-semantics-compat/evidence/2026-10-23-m2s39-focus-in-shadow.md)。
> **下一步**：按挂账序 custom command `--` 前缀 CommandEvent defer、
> ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留、summary-less
> details 内非激活后代翻转（s37）、focus-after-close Complex dialog
> usage focus 恢复深层（s39 剩余 1F）；parser form element pointer 4F
> **已缓行**（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆
> parser 批次、M2 收口后尾段单独立项，触发条件见
> [决策记录](html-semantics-compat/evidence/2026-10-23-user-decision-parser-form-pointer-defer.md)）。
>
> **▶ 上轮状态（2026-10-23 M2 切片 38 收口）**：挂账 appendChild move
> 后兄弟读 stale 收口。**探针钉定**（zz-probe 两迭代，验证后已删）：
> R334 appendChild/insertBefore wire 异步 drain，兄弟读无 pending 补偿
> （parentNode 有槽位补偿、兄弟没有）；两遍调用序列进一步钉出**共享视
> 图插位缺陷**——R182 重定位 pos 在旧位剔除前计算，剔除后未同步，
> cleanup `insertBefore(input, dialog)` 形态视图成 [dialog, input,
> script]（真树 [input, dialog, script]）；shadow 容器案露第三层——
> 上一子测试 stale parentSel 槽未覆写。**落地（R5240 三刀）**：①part04
> 兄弟 trap pending gate（sel 槽→融合视图定位 / 容器 ariaSync→
> registry 定位）；②part05 overlay 旧位剔除 pos 同步（**共享路径**，
> childNodes/children 全消费面受益）；③容器挂载路径 dual-form 槽覆写
> （R97 noWire 同款形状）。**集群**：focus-after-close **3F+1T →
> 2F+1T**（moved-in-between 案全绿）；interactive-elements 全域 98
> files 416P/64F/2T 对 s37 账本 **REG 0 / GAIN 1**；forms 域 4627
> subtest 逐位一致（REG 0）。**已知边界**：moved-to-shadow-root 案兄
> 弟断言过、红点推进到 `shadowRoot.activeElement`（focus-in-shadow 跨
> 流缺口，与 dialog-focus-previous-outside 同族，新挂账）。gates：lib
> **3430P/0F**（+1 为渲染流新增）、clippy/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s38-pending-move-sibling.md](html-semantics-compat/evidence/2026-10-23-m2s38-pending-move-sibling.md)。
> **下一步**：按挂账序 parser form element pointer 4F（用户决策门禁，
> 单独立项征询）、custom command `--` 前缀 CommandEvent defer、ESC 组
> 栈边缘乱序、非模态 close 不出组栈、R371 模板残留、summary-less
> details 内非激活后代翻转（s37）、focus-in-shadow
> shadowRoot.activeElement + focusing steps 跨 shadow 恢复（s38 新，与
> dialog-focus 族跨流缺口同族）；declarative shadow DOM（H 定性后）待
> parser/渲染流联合切片。
>
> **▶ 上上轮状态（2026-10-23 M2 切片 37 收口）**：挂账 anchor-with-inline
> 导航缺口收口。**探针钉定**（单案隔离；多案探针 onhashchange 竞争污
> 染作废）：裸 a 后代 click 也不导航——R112 post 段 nearest activation
> 无 act:a 分支（落 input-change 兜底）；svg 案露出第二根因——summary
> 豁免祖先 walk 不认 SVG ns a[href]（`_zwIsInteractiveContentEl` ns
> 门），summary 误翻转。**落地（R5239）**：①R112 post 补 act:a/act:area
> 分支执行 `_zwAnchorActivate`（receiver 自身让位防双导航）；②summary
> 消费面祖先 walk 补 A/AREA href 判定（不扩共享交互函数）。**集群**：
> anchor-with-inline-element **Timeout 全红 → 5P/0F**；A/B 对照 summary
> 域 REG 0 / **GAIN 5**（窄面 20P/0F/0T 全绿）、details/label 逐位一
> 致；interactive-elements 全域 98 files 415P/65F/2T（Timeout 2 文件均
> 既有挂账）、dialog 子集对 s35 REG 0 / GAIN 1；forms 4627 subtest 计
> 数逐位一致。**已知边界**：点击 summary-less details 内非激活后代不
> 翻转（同根邻近缺口，真浏览器会翻转，记待办）。gates：lib **3429P/
> 0F**、clippy/fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s37-anchor-inline-activation.md](html-semantics-compat/evidence/2026-10-23-m2s37-anchor-inline-activation.md)。
> **下一步**：按挂账序 parser form element pointer 4F、custom command
> `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不出组
> 栈、R371 模板残留、summary-less details 内非激活后代翻转（s37 新）；
> declarative shadow DOM（H 定性后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 36 收口）**：挂账 impl-doc wrapper
> requestClose 收口。**探针钉定**：dialog-requestclose 面 5 files 78P/1F，
> 唯一红正是挂账项；合成 doc（`_zwMEl` 产物）dialog 四方法全缺，R125
> 原型链已接 HTMLDialogElement.prototype 但 prototype 空占位。**落地
>（R5238）**：`HTMLDialogElement.prototype.requestClose`（R4100
> details.name 同款 plain 世界供给惯例）——spec「request to close」
> 步骤照抄：open 缺 return、ownerDocument ≠ 主文档（非 fully active）
> return；trap 世界走 part04 R4200 版先于原型命中，双世界不重叠。
> **集群**：requestclose 面 **78P/1F → 79P/0F**（EXIT 1→0）、dialog
> 集群 82 files 对 s35 账本 **REG 0 / GAIN 1**（351P/59F/1T）、forms 域
> 4627 subtest 对 s34 基线计数逐位一致。**已知边界**：detached doc
> dialog 的 close/show/showModal 三方法仍缺（挂账范围外，探针 3F
> 实证）。R5237 撞渲染流，改 R5238。gates：lib **3429P/0F**、clippy/
> fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s36-wrapper-requestclose.md](html-semantics-compat/evidence/2026-10-23-m2s36-wrapper-requestclose.md)。
> **下一步**：按挂账序 anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边
> 缘乱序、非模态 close 不出组栈、R371 模板残留；detached doc dialog
> close/show/showModal 三方法（新）；declarative shadow DOM（H 定性
> 后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 35 收口）**：挂账 E 余项 ESC 栈序
> 收口，**closedby 双语料面全清**（simple 6/6 + complex normal 面 s34
> 已绿）。**落地（R5236）**：`__zw_esc_dialog_cancel` 两段式——①
> `_zwTopLayer`（showModal+showPopover 共享插入序）反走，栈顶 popover
> 命中即 hidePopover、先碰 open dialog 切组栈；②R4300 组栈原逻辑不动
> （惰性补登记 + none 挡板 + 尾→头首个 open 组）。spec 钉定：close
> watcher 组栈，none dialog 组挡「再往下」不挡「更晚注册」——两段拼接
> 与 watcher 组栈语义在所有混合序一致。**集群**：82 files 411 subtest
> **350P/60F/1T**，同 key 对 s34 ON 基线 diff **REG 0 / GAIN 5**；回归
> 面 canceling 族 8P/0F、keyboard 77 键 A/B 逐位一致。设计迭代：首版
> 统一扫描破 part60 R2（lib test 抓住），两段式恢复。gates：zero-engine
> lib **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s35-esc-unified-toplayer.md](html-semantics-compat/evidence/2026-10-23-m2s35-esc-unified-toplayer.md)。
> **下一步**：按挂账序 impl-doc wrapper requestClose（s36 已收口）、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close
> 不出组栈、R371 模板残留；declarative shadow DOM（H 定性后）待
> parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 34 收口）**：挂账 F 帧驱动快照新鲜
> 化。**落地（R5235）**：Makefile `testharness-html-semantics` 入口默认
> `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（R2713a/R3254-KP4 既有开关，
> keyboard 套件同款入口模式；直接调 runner 维持 OFF 旧路径）。**对账三
> 面**：dialog filter 82 files 411 subtest **337P/73F/1T → 345P/65F/1T
> REG 0 / GAIN 8**（closedby-complex normal dialogB/popoverB——快照新
> 鲜后命中循环拿到真实盒）；testharness-html 合成面 7 files 14 subtest
> 逐字节一致；forms 域 401 files **4627 subtest 逐位一致**。**同日续勘
> 归因修正**：**挂账 G 移除**——spec 原文（#blocked-by-a-modal-dialog）
> 就是「栈顶模态 + flat tree 后代豁免」无祖先链豁免，gate 只判栈顶与
> spec 一致，三层栈探针是人造面（误关真因 = 快照滞后已修）；**挂账 H
> 重新定性**——最小探针实证 `<template shadowrootmode>` declarative
> shadow root 不支持（shadowRoot null、template 当活 DOM=R371 伪象），
> shadow slots 16F 归因 declarative shadow DOM 缺口（parser + shadow 树
> 渲染，后者归渲染流），修 light dismiss flat contains 是修在伪象上。
> **挂账 I（新）**：全量 testharness-html-semantics（2594 用例）>5400s
> 跑不完且 `--json` 缓冲到最后超时全丢——runner 增量输出或 make 分域入
> 口。gates：runner Makefile 一行无 Rust 变化、零新增依赖。账本见
> [evidence/2026-10-23-m2s34-raf-frame-driven-default.md](html-semantics-compat/evidence/2026-10-23-m2s34-raf-frame-driven-default.md)。
> **下一步**：E 余项 ESC 栈序（simple 5F——统一 top-layer 序覆盖
> popover 候选，涉 R4300 组栈改造）或按挂账序 impl-doc wrapper
> requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留；declarative shadow DOM
> （H 定性后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 33 收口）**：light dismiss 两遍模型
>（挂账 E 第一刀）。**落地（R5234）**：`__zw_dialog_light_dismiss_check`
> 两遍重写——①popover pass（`_zwTopLayer` 反走自顶向下，stale/摘属性/
> 断连跳过，contains 即级联停，否则 `_zwHidePopover`）；②dialog pass
>（反文档序首个 open+any 一锤定音：backdrop 命中自身 → requestClose；
> contains → break 内层挡外层；否则 requestClose——一次 pointerdown 至
> 多关一个 dialog）；③shadow 守卫（targetSel 解析不到 → 保守跳过）。
> **集群**：82 files 411 subtest **337P/73F/1T**，同 key 411 对 s32 逐位
> diff **REG 0 / GAIN 16**（closedby-complex 的 dialogA/outside-all/
> popoverA 面）。**探针钉定零盒根因 = 快照滞后**：`waitForRender`（双
> rAF）在 shim OFF 模式是同步 stub 不驱动 render，测试体内 gBCR 永远读
> 上一拍快照；`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（R2713a 既有开关）
> 实验实证 closedby 46 子测全出 **25P/21F** 对 OFF 同 key **回归 0
> 增益 +8**（normal dialogB/popoverB 全绿）——未默认启用（全集群对账
> 独立切片，挂账 F）。**新挂账**：F 帧驱动快照新鲜化（runner）、G
> runner modal retarget 多模态规则（下层 modal 盒内点击误 retarget
> 栈顶）、H shadow 影域 light dismiss（shadow slots 16F，target 解析
> 穿 shadow）、E 余项 ESC 栈序维持（simple 5F）。gates：zero-engine lib
> **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s33-light-dismiss-two-pass.md](html-semantics-compat/evidence/2026-10-23-m2s33-light-dismiss-two-pass.md)。
> **下一步**：挂账 F（帧驱动快照新鲜化 + 全集群对账，解锁 normal
> dialogB/popoverB 8 案与 simple Timeout 面已实证）或按挂账序 G/H、
> impl-doc wrapper requestClose、anchor-with-inline 导航缺口、parser
> form element pointer 4F、custom command `--` 前缀 CommandEvent defer、
> ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 32 收口）**：popover 有效性语义。
> **红因链破案**：closedby-complex 40F 统一死于 `matches(':popover-open')`
> SyntaxError（host 引擎无该伪类 → promise_test 未处理拒绝全红）；
> closedby-simple 1F = `resetDialogOuterTest` 前置 `hidePopover()` 过度
> 抛出，文件级 Timeout = replica promise_test 不接体 rejection、其余 5
> 体同点 reject 挂死 completion callback（`pending:5 tests:6` 实测吻合）。
> **spec 钉定**：hide popover 算法仅在 validityResult 为 DOMException 时
> 抛——hidden/disconnected 态不匹配返 false 布尔 → hidePopover 静默
> no-op（popover-utils「should not throw」语料权威）。**落地三件**
>（R5233）：shim `_zwHidePopover` lenient 化（非 popover
> NotSupportedError / showing+断连 InvalidStateError / 其余静默）+
> matches 陷阱 `:popover-open` 裸伪类特判（:focus/:modal 同款 JS 侧特判）
> + part09 契约测试翻转与三态断言。**集群**：82 files 411 subtest
> **321P/89F/1T**，同 key 406 对 s31 逐位 diff **REG 0 / GAIN 0**；
> closedby-simple Timeout 解体 → 6 子测收敛 5F+1P；complex 40F 红因迁移
> 为统一行为缺口。**探针钉定两缺口**（一次 outside click → dA/dB 全关、
> pA/pB 不关）：**挂账 E（下一切片主目标）popover light dismiss 未建**
>（R3071 defer 至今）+ **dialog light dismiss 链停**（一次 click 应只关
> topmost clicked dialog 链）；simple 第二 ESC 断言面（ESC 栈序）同挂。
> focus-after-close.html Timeout 既有挂账维持。gates：zero-engine lib
> **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s32-popover-open-semantics.md](html-semantics-compat/evidence/2026-10-23-m2s32-popover-open-semantics.md)。
> **下一步**：挂账 E（popover light dismiss + 链停语义，engine click 路
> 径）或按挂账序挂账 A（强制同步布局冲刷）、impl-doc wrapper
> requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 31 收口）**：dialog showModal 布局
> 域。**谜题破案**：s30「dialog[open] computed block 但 gBCR 全零」是探
> 针伪象——gBCR 读 renderer 上一帧 rect 快照（rect_bridge），open 同 tick
> 读拿到旧布局；等帧后全部有盒。探测环刷新顺序（flush→timers→刷新）钉
> 死 stale 窗口：abspos-dialog-layout 9 案中 6 案红因 = showModal 同 turn
> 测量双侧 stale（expected 300/got 0），**挂账 A（本流）：强制同步布局
> 冲刷**（gBCR/offset* 读前冲刷 + 重排，CSSOM view 语义）。**`:modal`
> 通道落地**：模态性只在 shim JS 栈、DOM 序列化不携带 → 镜像 :focus 注
> 入通道（R4241）建全链——dom `Document.modal_dialogs`/`is_modal_dialog`
> + pipeline `inject_modal_state`（fresh-parse 后注入）+ webview 转发 +
> runner 探测环轮询 `_zwDialogModalAllSels()`（shim 全量导出，栈序、嵌
> 套模态皆匹配）+ matcher `:modal` 臂——作者 `dialog:modal` 规则自此真
> 实匹配。**UA 定位注入实验矩阵**（回归 0 纪律定性）：规范照抄（inset
> 0 + margin auto）= 9 回归（布局对 inset 指定 + size auto 走满视口拉
> 伸，light-dismiss 盒外点击全族失效）；inset auto 近似 = 3 回归（出流
> 重叠、selection/editing 点击几何漂移）；不注入 = **0 回归 0 增益**
>（320P/85F/2T 与 s30 逐位打平）→ 终态不注入。**挂账 B（渲染流）：
> abspos 收缩/fit-content 语义补齐**，UA 规则随该切片按规范回填（A 形
> 态实验已端到端验证轮询链）；s30「ancestor-is-inert 红因 = UA
> position:absolute 缺失」修正为「注入受布局收缩缺口阻断」。挂账 D：
> getComputedStyle 路径 `:modal` 注入（低优）。gates：zero-dom lib
> **863P/0F**、zero-engine lib **3428P/0F**（+1）、zero-style-system lib
> **2446P/0F**（+1）、clippy -D warnings、fmt clean、零新增依赖；渲染/
> UA 样式净变化为零免 product-smoke。账本见
> [evidence/2026-10-23-m2s31-dialog-modal-channel.md](html-semantics-compat/evidence/2026-10-23-m2s31-dialog-modal-channel.md)。
> **下一步**：挂账 A（强制同步布局冲刷，解锁 abspos/non-modal 布局语
> 料同 turn 断言面）或按挂账序 popover 有效性语义（`:popover-open` 选择
> 器 + hidePopover not-showing 抛 + popover-simple 余 hang）、impl-doc
> wrapper requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 30 收口）**：rendering inert 域第一
> 刀——hit-test 切面收口。宿主指针命中（runner `resolve_pointer_target`
> 候选循环）不排除模态 inert 子树，点击模态外元素照常命中。修复三件：
> ① 命中候选循环接 inert 排除（跳过 `!_zwDialogModalFocusGateOk(e)`——
> s29 门语义与 hit-test inert 排除同源直接复用，门补 globalThis 导出供
> 页面域调用，缺省回落原行为）；② backdrop 命中回落：候选全排除且栈顶
> 模态存在 → target=栈顶模态 dialog（新增 part01 `_zwDialogModalTopSel`；
> ::backdrop 无布局盒的近似）——真 Chrome 仲裁（ancestor 语料独立探针）
> 实测「点击 inert 区域 target=HTML」（backdrop 被 display:none 不渲染、
> 穿透到根），回落读 styleSheets 文本扫 `::backdrop{...display:none}`
> 命中则 target=documentElement；③ light dismiss backdrop 臂
>（`__zw_dialog_light_dismiss_check` 加 px/py）：backdrop 回落 target=
> dialog 曾压制 dismiss——target=栈顶模态 dialog 本身且落点在 gBCR 外 =
> 点击 backdrop → 照发 close request（真 Chrome md:dialog 且关闭）。
> 账本：dialog 集群 82 files **320P/85F/2T**（s29 同集 317P/87F/3T），
> 同 key 回归 **0**、增益 2（blocks-mouse-events F→P、light-dismiss-
> touch F→P 连带）+ backdrop-receives-element-events **Timeout→Pass**
>（3T 基线之一收口，backdrop 命中面直接语料）。gates：engine lib
> **3427P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s30-modal-hit-test.md](html-semantics-compat/evidence/2026-10-23-m2s30-modal-hit-test.md)。
> **本轮钉定的相邻归因**：ancestor-is-inert 红因已换——dialog showModal
> 后 UA `position:absolute` 缺失（探针 pos=static、盒 (101,201,50x50)
> 落进父 div 内容区），归 abspos/layout 账（与「showModal 居中」同根），
> 本轮命中面修复已为该案铺平；inert-svg-hittest `:active` 断言引擎恒
> false（matcher 运行时伪类臂缺 active/hover），归交互态伪类账；
> selection 两案依赖指针拖选→Selection 链路，挂账；dialog-inert 依赖
> 居中布局，归 abspos 账。
> **下一步**：dialog UA showModal 布局（absolute + inset:0 + margin:auto
> 居中——abspos-dialog-layout ~10F 与 ancestor-is-inert/dialog-inert 同
> 根）或按挂账序 popover 有效性语义（`:popover-open` 选择器 +
> hidePopover not-showing 抛 + popover-simple 余 hang）、impl-doc
> wrapper requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 29 收口）**：模态焦点门接页面
> focus()——s28 门只覆盖宿主驱动路径（pre-focus 步、mousedown 默认动
> 作），页面直调 `el.focus()` 不在门内，inert-node-is-unfocusable 13F。
> 修复四件：① `part04 focus()` trap 接门（已聚焦 no-op 之后、失连臂之
> 前；失连元素门内跳过保 R114/T55 detached 契约）；② 门位在 R4003b
> label 前递**之后**——前递源（label 本就非 focusable area）不受模态
> inert 门，inert 语义作用在焦点落点，前递目标经递归 focus() 再入 trap
> 自行过门（inert-label-focus 断言面钉次序：门前置时该案 PASS→FAIL，
> 移位后归位）；③ showModal 栈推进移到 focusing steps 之前（嵌套模态
> 子树 focusing steps 过门）；④ R4600b inert 分支 focus fixup 落 body
> 走 `__zwModalGateBypass`（fixup 是 UA 级焦点回落非 focusing steps，
> Chromium 同款 activeElement===body）。门包含判定补 shadow-including：
> `proxy.contains` 只覆盖 light DOM，既有契约 R4600 R8（delegatesFocus
> 影子树内锚）暴露缺口——新增 part01 `_zwDialogShadowInclContains`
>（parentNode 上行遇 shadow root（`__zwHandle` 命中 `_shadowHandles`）
> 换 host 继续、frag 容器经 `_shadowHandleMeta` 跳 host，路数同 part06
> activeElement retarget，64 步 guard 防反链环）。契约
> `test_dialog_modal_focus_gate_s28` 增 G5/G6（页面 focus() 拒焦／失连
> 归解析节点所有权）。dialog 集群 82 files 407 subtest **317P/87F/3T**
>（s28 同集 303P/101F/3T），逐 subtest diff 同 key 回归 **0**、增益
> **14**（inert-node-is-unfocusable 13F 全清 15/15 +
> remove-dialog-should-unblock-document 连带 F→P）；gates：engine lib
> **3426P/0F**（含 G5/G6）。账本见
> [evidence/2026-10-23-m2s29-page-focus-gate.md](html-semantics-compat/evidence/2026-10-23-m2s29-page-focus-gate.md)。
>
> **▶ 上轮状态（2026-10-23 M2 切片 28 收口）**：corner-cases 余 2F 收口
> ——先建真 Chrome 基准仲裁（playwright-core + chromium-1243，CDP 真实
> 输入层；test_driver 桥接补 ButtonType/特殊键映射），语料基线 29/30
> 唯一失败＝esc-modeless。两案定性：esc-modeless 非 bug（真 Chrome 事件
> 流与引擎逐条一致，上游期望 closed 在 Chrome 自身即失败，记账 Chrome
> parity 非 gap）；click-modal 引擎分歧（真浏览器不可能聚焦模态外元素，
> runner pre-focus 步泄焦）→ 修复＝模态 inertness 焦点切面：`part01
> _zwDialogModalFocusGateOk`（栈顶 open+模态 dialog 子树外非 focusable
> area，stale 栈项跳过，stacked modal 仅栈顶生效）+ `part06
> _zwIsFocusableSel` 接入（宿主 pre-focus 步与 mousedown 默认动作共用
> 门）。契约 `test_dialog_modal_focus_gate_s28` G1–G4。集群 **303P/101F
> /3T**，回归 0 增益 1（corner-cases **29P/1F == 真 Chrome 29/30**）。
> 账本见
> [evidence/2026-10-23-m2s28-modal-focus-gate.md](html-semantics-compat/evidence/2026-10-23-m2s28-modal-focus-gate.md)。
>
> **▶ 上轮状态（2026-10-23 M2 切片 27 收口）**：corner-cases 毒化链根因
> 收口（R4620 批内 selector 时序）——根因在宿主共享 apply 路径而非调用方：
> JS 侧同 dispatch 的 op 引用批开始时的树态（spec insertBefore 的 ref 节点
> 对象不因前序 remove 失效），而 drain 顺序重放时前序 Remove 摘除 dialog →
> ref script 的 `:nth-child` 索引漂移（3→2）→ INS target 失配 warn+skip →
> dialog 永久 detached、同批 SetAttr 连锁失配（remove_child 已清 id_map）→
> 记录毁灭后 29 案 setup 全部 closedBy=none、ESC close request 找不到
> dialog（探针宿主 warn 实锤，R361 stash 走不到——target 先失败）。修复＝
> `apply_dom_mutations_full` 批前预解析 insert-adjacent 族四变体
> （Html/Text/Element/SelElement，同构失配语义）target 为 NodeId（不随树
> 态漂移）+ 四臂失配查表兜底（ref 被批内摘除时维持 skip，语义对齐 JS 抛错
> 面）。契约 `dom_bridge_tests::test_apply_ins_target_pre_remove_index_drift_r4620`
> （修复前红/修复后绿）。dialog 集群 82 files **302P/102F/3T**（s26 同集
> 272P/135F，+30P/−33F），逐 subtest 对 s26 账本 diff 同 key 回归 **0**、
> 同 key 增益 27（全落 corner-cases 毒化案）；corner-cases **1P/29F→28P/2F**，
> 3T 全为 s26 基线既有；focus-shadow 50P/0F、closedby 27/27、requestclose
> 75P 维持；九域全量留待下轮刷新（本轮证据＝集群逐 subtest diff）。gates：
> engine lib **3416P/0F**（含新契约）、clippy -D warnings、fmt clean、零新
> 增依赖。账本见
> [evidence/2026-10-23-m2s27-r4620.md](html-semantics-compat/evidence/2026-10-23-m2s27-r4620.md)。
> 余 2F＝`focusin calls showModal` 变体（focusin handler 内 re-entrant
> showModal 的 ESC/light-dismiss 判定边缘；clicking/modal 键 s25 同 key
> 既有）。
> **下一步**：corner-cases 余 2F（re-entrant showModal 边缘，同域小残差）
> 或按挂账序推进 inert 计算态 13F、abspos/layout ~10F（rendering 域）、
> popover 有效性语义（`:popover-open` 选择器 + hidePopover not-showing
> 抛 + popover-simple 余 hang）、impl-doc wrapper requestClose、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不
> 出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-22/23 M2 切片 26 收口）**：dialog focusing steps 家族
> 收口（R4600 + rollover 挂死修复 R4600c/d）——spec show/showModal step 的
> run the dialog focusing steps、
> close-the-dialog 的焦点恢复落地：`_zwDialogFocusingStepsR4600`（autofocus
> 委派 → focusable area 递归判定（tag 表 + disabled/hidden/inert 子树传播/
> closed dialog 排除/delegatesFocus）→ focus 派 focusin）+
> `_zwDialogRestoreFocusR4600`（previously focused 捕获与恢复）+
> activeElement JS-observable（part06 focus chain retargeting：shadow 树内
> 沿 parentNode 上行解析焦点锚）。R4600b：focus-shadow 余 4F 根因＝R371
> replaceWith 重键的 `_r371Next` 后继快照取到**换行空白文本节点**（nodeType
> 3 无 selector），元素级 QSA R322 归并锚点只认 sel 字符串 → 恒尾插 → 焦点
> 扫描先命中后随兄弟；修复＝快照经 `_childNodeList(父sel)` 元素扫描推进到
> 最近后随**元素**兄弟。rollover 会话 R4600c/d 修 s26 树全量暴露的 3 文件
> 确定性整文件挂死（dialog-autofocus 族，load+focusin 双到齐缺 focusin
> ——runner 预注入 autofocus 先于页脚本，focusing steps 事件相位派发时
> 页监听器未注册且对已聚焦元素不重派）：part06 load 分支补 spec
> flushing-autofocus-candidates（processed flag + 首个合格候选 defer
> task，已聚焦态补发事件相位）+ part01 focusable area 补 pending-removed
> 门（元素级 QSA 无 overlay，同步摘除后仍命中）。testdriver click/down
> focus 步骤经 `_zwFocusSel` 门。
> part66 `test_dialog_focusing_steps_r4600` R1–R13 新契约。九域对账
> 5070→**5116P**（68.32%，Timeout 173 持平、subtests 7488 不变）：
> R4600c/d 隔离对照 **+3P/0 回归**（dialog-autofocus 族 3 挂死文件转
> Pass）；余 +43P 为 R4600 主体（focus-shadow 50F→0、closedby 27/27
> 维持）；同 key 回归 27 全落 corner-cases 毒化链（下述，s26 树已
> 同态）。corner-cases **1P/29F**（s25 28P/2F）：R4600 使 focusin 真实
> 触发音量 handler（remove+reinsert 共享 dialog），test#0 通过但毒化共享
> dialog → 后续 setup 全部 closedBy=none/conn=false——**新暴露**非新引入
> （s25 缺口休眠因 focusing steps 无操作），毒组合＝[reinsert#1+show+drain+
> ESC#1+focus+reinsert#2]，E1-E8 单因子全过；三层缝隙移交下轮：①R334 重
> 插 wire 完整性 ②ESC 派发锚 activeElement 反查 miss ③`_zw_esc_dialog_cancel`
> QSA-miss 顺栈 purge。另发现未修：R371 rekey 时 dst 桶不存在 → 被替换
> template 不入父桶 removed 合并视图残留（对扫描无害，spec 违例候选下轮）。
> **下一步**：corner-cases 毒化链三层缝隙（R334 重插 wire 完整性/ESC 派发
> 锚点/组栈 purge）；余挂账 inert 计算态 13F、abspos/layout ~10F（rendering
> 域）、popover 有效性语义（`:popover-open` 选择器 + hidePopover not-showing
> 抛 + popover-simple 余 hang）、impl-doc wrapper requestClose、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不
> 出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-22 M2 切片 25）**：corner-cases hang 收口（R4500）
> ——真根因横切两层：①shim `close()` IDL 包装透传 helper 的 wasOpen
> boolean（spec void）；②语料 testharness `is_valid_cleanup_result` 对
> cleanup 返非 undefined **及** `on_error` 对 cleanup 抛错均 `tests.abort()`
> → 完成回调不达 → file Timeout 吞全文件。`t.add_cleanup(() =>
> dialog.close())` 返 false 一个值吞 27-29 案；s23「detach 窗口身份链」
> 归因作废（hang 案实为前一案 cleanup abort，探针五文件 DA/DB 对照 +
> partial-flush 形态学定位）。修复：part04 三处 close 包装 void 化 +
> part14 R3290 两处旧断言更正 + part60 `test_dialog_void_return_r4500`
> （四方法全生命周期 + 八方法 void 族审计无第二 offender）。九域
> 5016→**5070P**，同 key 回归 **0**、翻转 **0**；Timeout 180→173（7 文件
> 解除吞没：closedby 1P→27/27、corner-cases 1P→28P/2F、focus-shadow
> 50F 新上报等），+55F 为新上报真断言非回归（分母 +102 漂移，百分比口径
> 不可比）。余 1 hang：popover-closedby-simple（cleanup `hidePopover()`
> 抛 not-showing → abort 同族——病根 popover 有效性语义，popover 域挂
> 账）。corner-cases 余 2F＝closing steps 内 focus fixup 不派 focusin
> （focusing steps 家族）。
> **下一步**：focusing-steps 家族（focus fixup→closing steps 派 focusin、
> autofocus 委派、activeElement 语义——corner-cases 2F + focus-shadow
> 50F + focusing-steps/activeElement 原挂账 ~15F 同域合并攻）；余挂账
> inert 计算态（inert-node-is-unfocusable 13F 精化）、abspos/layout
> ~10F（rendering 域）、popover 有效性语义（`:popover-open` 选择器 +
> hidePopover not-showing 抛，popover-simple 余 hang）、impl-doc wrapper
> requestClose、anchor-with-inline 导航缺口、parser form element pointer
> 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、
> 非模态 close 不出组栈。


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

- **2026-10-24 M2 切片 67**：embedded-content 开片分诊轮——域刷新
  458P/819F/40T 八族地图（F1 img 静态加载管线 ~260F / F2
  select-an-image-source ~300F / F3 decode 56F / F4 image-map 命中 84F
  渲染流回流 / F5 lazy ~49F / F6 canvas 53F / F7 object|embed 窄面 25F /
  F8 杂项 2F）+ F7 null 反射收口（R5274 一刀，part04 get-trap
  OBJECT/EMBED 分支——contentDocument/contentWindow/getSVGDocument 恒
  null 面）+ 契约 part84 两钉 + 翻转力实证。域 **461P/816F/40T**（GAIN 3
  / REG 0）；interactive 439P/44F/0T 持平；lib **3469P/0F**（+2）。账本
  [evidence/2026-10-24-m2s67-embedded-content-triage.md](html-semantics-compat/evidence/2026-10-24-m2s67-embedded-content-triage.md)。
- **2026-10-23 M2 切片 46**：余 60F 归因分诊六族地图落档（A–G，含
  Chrome-parity 同红族 A/B 与可 shim 收口候选 E）+ 历史 menu 面 IDL 归属
  收口（R5249 双刀，shim part04/part05）——has trap track IDL 列
  kind/srclang/label/default tag 门（TRACK 与 TRACK+OPTION）+ R3069 写臂
  MENU/MENUITEM 负门（FLAT type 与显式 label 列无 tag 门致 menu.type=/
  menu.label= 误写内容属性）。契约 part78 两钉（menu 排除五臂 + track/
  option/FLAT 面四臂不回归）+ 翻转力实证。contextmenu-historical
  **11/11 Pass**；interactive-elements 全域 98 files **423P/57F/2T 对 s45
  逐位 REG 0 / GAIN 3**；lib **3442P/0F**（+2 契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s46-historical-menu-idl-gate.md](html-semantics-compat/evidence/2026-10-23-m2s46-historical-menu-idl-gate.md)。
- **2026-10-23 M2 切片 45**：s37 已知边界「点击 summary-less details 内
  非激活后代不翻转」收口（R5248 双刀，shim part03/part04）——part04
  click 默认动作段 nearest activation 上溯分支（`.click()` 合成半边，
  receiver 三站让位 + 交互后代 type 豁免 + notPrevented 门继承）+
  part03 R112 post act:details 门放宽至 JS dispatch 通道
  （`_zwSyntheticClick` 标互斥防双翻，preventDefault/connected 门由入口
  守卫继承）。契约 part77 两钉（翻转/互斥六臂 + 让位/豁免三臂）+ 翻转力
  实证。interactive-elements 全域 98 files **420P/60F/2T 对 s44 逐位
  REG 0 / GAIN 0**（corpus 无观测面）；lib **3440P/0F**（+2 契约）；零
  新增依赖。账本
  [evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md](html-semantics-compat/evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md)。
- **2026-10-23 M2 切片 44**：s39 剩余 1F focus-after-close Complex 案收口
  （R5247，shim part01/part04/part06 三处）——跨代 handle proxy 孤儿
  isConnected 假阴性修正（宿主持久反查 + querySelector 同一性核验前置
  复核）+ activeElement 焦点链 retarget light 锚误穿收口（light 锚直返
  门，判据与 stale `_shadowHandles` 投影注册形态解耦）。契约 part76 两钉
  （孤儿 isConnected / light-shadow retarget 双向）+ 翻转力实证。
  focus-after-close 单案 **6/6 Pass**；interactive-elements 全域 98 files
  **420P/60F/2T 对 s43 逐位 REG 0 / GAIN 1**；lib **3437P/0F**（+2 契约）；
  零新增依赖。账本
  [evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md](html-semantics-compat/evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md)。
- **2026-10-23 M2 切片 43**：replaceWith self-removal 父桶记账收口（R5246
  一刀，s26 既有挂账「R371 rekey 时 dst 桶不存在，不记父桶 removed」）——
  sel 臂 remove 步补 `_zwHCLiveInvalidate` 直调（父桶 removed identity +
  全局 pending-removed + live 失效一体），合并视图残留终结、视图即 spec
  终态；记账先于 rekey，promote 分支留护栏。契约 part75
  test_dialog_replacewith_parent_bucket_accounting_r5246（R1/R2）+ part24
  r321 kids 断言更新（宿主残留 DIV 消失）。interactive-elements 全域
  98 files **419P/61F/2T 对 s42 逐位 REG 0 / GAIN 0**；lib **3434P/0F**
  （+1 契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s43-replacewith-parent-bucket.md](html-semantics-compat/evidence/2026-10-23-m2s43-replacewith-parent-bucket.md)。
- **2026-10-23 M2 切片 42**：非模态 close 不出组栈收口（R5245 一刀，
  s23 已知保守项）——close-the-dialog 组栈出栈移出模态臂，modeless
  show/close 循环 stale 项无限增长终结；契约 part74
  test_dialog_group_stack_hygiene_r5245（R1–R4 行为不变量：循环/栈序/
  全关 no-op/模态臂保持）。interactive-elements 全域 98 files
  **419P/61F/2T 对 s41 逐位 REG 0 / GAIN 0**；lib **3433P/0F**（+1
  契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s42-nonmodal-close-stack.md](html-semantics-compat/evidence/2026-10-23-m2s42-nonmodal-close-stack.md)。
- **2026-10-23 M2 切片 41**：挂账定性轮——「ESC 组栈边缘乱序」收口为
  Chrome-parity 同红（corner-cases「focusin calls showModal, modeless」
  ESC 案：crbug 417571176、父 376516550，2026-04 wpt.fyi 仍红）。探针
  钉定事件序 + spec 逐字对账（R4600 恢复门/R5236 组栈零漂移）；R5244
  假设门（subject 非 focusable no-op）GAIN 1 / REG 4（dialog-focus-shadow
  Chrome「show() 聚焦 dialog 自身」面）试错回退，零代码净变化。全域
  419P/61F/2T 对 s40 逐位 REG 0、lib 3432P/0F、零新增依赖。账本
  [evidence/2026-10-23-m2s41-esc-stacked-parity.md](html-semantics-compat/evidence/2026-10-23-m2s41-esc-stacked-parity.md)。
- **2026-10-23 M2 切片 40**：custom command（`--` 前缀）CommandEvent
  派发收口（R5243，s24 defer）——part05 CommandEvent 接口
  （`_defineEventSubclass`，command=''、source=null，无 relatedTarget）
  + part01 `_zwInvokerCommandActivate` custom 分支派发后 return
  （command=原值大小写保留、source=invoker、cancelable=true、
  bubbles=false/composed=true、R312 trusted 印记；分派在调用方
  disabled/preventDefault 门内零新写）。interactive-elements 全域
  98 files **419P/61F/2T 对 s39 打平**；dialog 集群对 s35 **REG 0 /
  GAIN 5**（s39 既有收益）；forms/the-button-element 对 s24 逐位一致。
  gates：engine lib **3432P/0F**（+1 契约 part73）、clippy/fmt 过、
  零新增依赖。账本
  [evidence/2026-10-23-m2s40-custom-command-event.md](html-semantics-compat/evidence/2026-10-23-m2s40-custom-command-event.md)。
  挂账：现行 spec invalid→Unknown 面（跟进涉内建关键字重构）、
  popover 关键字、oncommand IDL、command/commandFor IDL 反射、
  ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留、
  summary-less details 非激活后代翻转、focus-after-close Complex
  dialog usage 深层。
- **2026-10-23 M2 切片 27**：corner-cases 毒化链根因收口（R4620 批内
  selector 时序）——根因在宿主共享 apply 路径：drain 顺序重放时前序
  Remove 摘除移动语义节点 → 后续 INS ref 的 `:nth-child` 索引漂移 →
  target 失配 warn+skip → dialog 永久 detached + 同批 SetAttr 连锁失配
  （JS 侧 op 引用批开始时树态，spec insertBefore 的 ref 不因前序 remove
  失效）。修复＝`apply_dom_mutations_full` 批前预解析 insert-adjacent
  族四变体 target 为 NodeId + 四臂失配查表兜底；契约
  `test_apply_ins_target_pre_remove_index_drift_r4620`（前红后绿）。
  dialog 集群 82 files 272P→**302P**（−33F），同 key 回归 **0**、增益 27
  （corner-cases 1P/29F→**28P/2F**，余 2F＝re-entrant showModal 边缘）；
  3T 全为 s26 基线既有；九域全量留待下轮刷新。gates：engine lib
  3416P/0F、clippy/fmt 过、零新增依赖。账本
  [evidence/2026-10-23-m2s27-r4620.md](html-semantics-compat/evidence/2026-10-23-m2s27-r4620.md)。
  挂账：corner-cases 余 2F（re-entrant showModal）、inert 计算态 13F、
  abspos/layout ~10F（rendering 域）、popover 有效性语义、impl-doc
  wrapper requestClose、anchor-with-inline 导航缺口、parser form element
  pointer 4F、custom command `--` 前缀、ESC 组栈边缘乱序、非模态 close
  不出组栈、R371 模板残留。
- **2026-10-22/23 M2 切片 26**：dialog focusing steps 家族（R4600）+
  rollover 挂死修复（R4600c/d）——show/showModal 跑 spec dialog
  focusing steps（autofocus 委派 → focusable area 递归判定 → focus 派
  focusin）+ close-the-dialog 焦点恢复 + activeElement 焦点链
  retargeting + R4600b replaceWith 空白锚点（focus-shadow 50F→0）；
  rollover 会话修 3 文件确定性整文件挂死：part06 load 分支补 spec
  flushing-autofocus-candidates + part01 focusable area 补
  pending-removed 门。九域 5070→**5116P**（68.32%），Timeout 173 持平，
  R4600c/d 隔离对照 +3P/0 回归；同 key 回归 27 全落 corner-cases
  毒化链（R4600 激活 focusin → 毒化共享 dialog，新暴露非新引入）。
  make test 76 腿全绿、clippy/fmt 不涉（纯 .js）、零新增依赖。账本
  [evidence/2026-10-22-m2s26-r4600.md](html-semantics-compat/evidence/2026-10-22-m2s26-r4600.md)。
  挂账：corner-cases 毒化链三层缝隙（下一切片首选）、inert 计算态
  13F、abspos/layout ~10F（rendering 域）、popover 有效性语义、
  impl-doc wrapper requestClose、anchor-with-inline 导航缺口、parser
  form element pointer 4F、custom command `--` 前缀、ESC 组栈边缘
  乱序、非模态 close 不出组栈、R371 模板残留。
- **2026-10-22 M2 切片 25**：corner-cases hang 排查（R4500）——横切根因
  收口：shim `close()` void 违约（透传 wasOpen boolean）× 语料
  testharness cleanup abort 语义（返非 undefined / 抛错均
  `tests.abort()`）→ file Timeout 吞全文件；s23「detach 窗口身份链」
  归因作废。修复 part04 三处 close 包装 + part14 两处旧断言更正 +
  part60 void 族契约 `test_dialog_void_return_r4500`（R1-R3）。九域
  5016→5070P（同 key 回归 0、翻转 0），Timeout 180→173（closedby
  1P→27/27、corner-cases 1P→28P/2F 等 7 文件解除吞没），+55F 为新上报
  真断言非回归。余 popover-closedby-simple 1 hang（cleanup hidePopover
  抛 not-showing → abort 同族，popover 域挂账）。gates：engine lib
  3394P/0F、webview 785→786P/0F、page-runtime 136P/0F、integration lib
  787P/0F、clippy/fmt 过、零新增依赖；中途 stash→rebase→pop 并入远端
  共享面提交（floats + workers 呈现管线，同动 part04.js 无冲突）。证据
  见 [evidence/2026-10-22-m2s25-r4500.md](html-semantics-compat/evidence/2026-10-22-m2s25-r4500.md)。
- **2026-10-22 M2 切片 24**：ToggleEvent.source / commandfor 簇
  （R4400）——toggle-source 三文件 3/10→10/10，九域 5008→5016P（+8 回归
  0）。含横切修复：user_actions.rs plan 通道私有批回灌共享队列全史 +
  prevented request-close 丢弃 invoker source。证据见
  [evidence/2026-10-22-m2s24-r4400.md](html-semantics-compat/evidence/2026-10-22-m2s24-r4400.md)。
- **2026-10-22 M2 切片 23**：dialog closedby 簇（R4300，rollover 接续
  ——上会话 429 中断遗留实现+契约 WIP，本会话核验收口）——两段推进：
  ①runner 通道：inline_local_scripts 根路径按 wpt_root 内联（域外助资
  产 `/common/top-layer.js`、`/close-watcher/resources/helpers.js` 等
  可达，harness 本体先前已被 replace_script_source 消费无双注入）+
  send_keys uE00C Escape 臂（`__zw_dispatch_key_at_active` 焦点元素三
  形态派发：`__zwSelector`/`_proxyCache` 反查/wrapper 直派；未取消 →
  `__zw_esc_dialog_cancel` 默认动作）+ click 命令 light dismiss 前步 +
  goal 脚本 5 助资产 fetch；②产品语义：closedBy IDL（小写化 + auto 派
  生 + 反射）+ `_zwDialogClosedByState` + close watcher 组栈
  `_zwDialogModalStack`（show/showModal 压组、首个 open 组即停、组内
  none 挡下探、markup/IDL open 惰性补登记、stale 顺栈清出）+
  `__zw_esc_dialog_cancel` 重写（closedby 资格门 + 组栈序 +
  `_zwDialogRequestClose`）+ `__zw_dialog_light_dismiss_check`（任何
  open+any dialog 包含链外落点 → close request；`__zw_pointer_down_
  sequence` 头部钩与 runner click 共享）+ `__zw_reset_form_state` 清
  栈。part60 R1–R5 契约。九域 4980→5008P（68.02%），对 s22 逐 subtest
  diff 净 +28P、回归 1：interactive-elements 202→227P（canceling 族假
  Fail 转真断言：ESC 主断言/组栈 none 挡板 4P/light dismiss 9P/
  cancel-events 族新上报 4P/closedby 逐案 7P/non-modal-canceling 1P）
  + scripting-1 +3（SVG script scheduler 意外翻绿稳定，机制未定位观
  察账），余七域 0 变化；回归 1（bounds-clicking (1,1)：dialog UA
  abspos 布局/hit test 缺口，归 rendering 域）。zero-engine --lib
  3386P/0F、page-runtime 136P/0F、webview 785P/0F、integration
  html_compat 21P/0F、clippy（test-guard 20G 包裹）/fmt 过、零新增依
  赖。挂账：corner-cases ~28 案 file Timeout（focusin remove+reinsert
  → showModal → ESC close 不达，detach 窗口身份链）、ToggleEvent
  .source/commandfor 8F 真断言（下一切片）、ESC 组栈惰性补登记边缘乱
  序（保守）、非模态 close 不出栈（读侧 purge 兜）、bounds-clicking
  (1,1)（rendering 域）、余 s22 挂账维持。账本
  [evidence/2026-10-22-m2s23-r4300.md](html-semantics-compat/evidence/2026-10-22-m2s23-r4300.md)。
- **2026-10-22 M2 切片 22**：dialog 元素簇（R4200，跨午夜会话）——两段
  推进：①part01 dialog 语义主体重构：toggle 事件任务队列（R4009 同款
  per-element 去重合并、preOpen 调用方写前捕获）+ close 事件排队异步
  （spec queue an element task，trusted 通道，旧同步派发 onclose 早于后
  续脚本）+ show/showModal 请求态语义（modal 上 show 抛/非模态上
  showModal 抛，旧静默切态）+ beforetoggle 同步可取消 + 写后重查 +
  close-the-dialog 核心共用 + `_zwDialogRequestClose`（cancel 可取消 +
  重入 busy 守卫，part04 DIALOG tag 门分发）；②共享基建：part03
  Promise.withResolvers polyfill + `_realTag` sel 印章（detach 窗口）、
  part05 `_insertAdjacentVariadic` sel 项（prepend(parsedEl) 丢失）、
  part04 matches 裸属性 lw + addEventListener `{signal}` per-record +
  set trap open moAttr 尾钩、part01 show/showModal/close 调用点 MO
  record（aria helper 共用防双计）。part60 T1–T8 新契约 +
  part14 ④⑧/part59 W3 契约同步（close 排队断言）。九域 4887→4980P
  （67.54%），对 s21 逐 subtest diff 净 +93P、回归 0，全落
  interactive-elements（109→202P）：requestclose 0→75P、toggle-events
  0→12P、close/close-event-async/requestclose-2/-recurse/-closedby/
  submit-dialog-close-event 各 +1。zero-engine --lib 3378P/0F、
  page-runtime 136P/0F、webview 785P/0F、integration html_compat 21P/0F
  （包名 zero-integration-tests 口径补正）、clippy/fmt 过、零新增依赖。
  挂账：closedby 簇 ~85F runner testharness helper 缺口（下一切片）、
  inert 计算态 ~15F、focusing steps ~15F（wrong global realm 面）、
  abspos/layout ~10F（rendering 域）、impl-doc wrapper requestClose、
  MO 内部写仅覆盖 dialog open（保守面）、`_zwSelSeenTag` 幽灵 sel 误
  服务（保守已知）、其余 s21 挂账维持。
- **2026-10-21 M2 切片 21**：details name= 互斥簇（R4100）——spec
  exclusive accordions 双 prong（others 开写面关其他 open / given 插入与
  name 变更面自身让位）+ 组口径（同树根 + 同名非空 name + DETAILS）。
  核心结构发现：innerHTML-into-sel 解析产物 `_zwMEl` plain wrapper 与
  t8r 物化 identity 元素混合同树——统一 node-based 核心
  `__zwDetailsExclusivityNode4100`（childNodes DFS 组枚举、`_elKey`/对象
  身份双态）+ R4009 队列第 4 参 plainNode 通道 + 解析扫描顺序插入模拟
  （树序第一 open 胜出，CeAttach 整批一次扫描）+ 六挂接面 + 解析面三处。
  part01 `_REFLECTED_NAME_TAGS` details + part03 原型 name accessor。
  取舍：解析面排程 toggle（spec 字面）与 test 3 期望序互斥，取 spec 口
  径维持 test 3 F。part60 T1–T8 契约。九域 4873→4887P（66.29%），对 s20
  逐 subtest diff 净 +14P、回归 0，全落 interactive-elements（95→109P）：
  name-attribute 2P/14F+Timeout→15P/3F、fragment-insertion 1P/0F。
  zero-engine --lib 3374P/0F、page-runtime+webview 785P/0F、integration
  html_compat 21P/0F、clippy/fmt 过、零新增依赖。挂账：toggleEvent 尾部
  ontoggle（document-parse 面）、name-attribute test 3 期望序互斥观察、
  xhr/template setup error、dialog 元素簇（下一切片）。账本
  [evidence/2026-10-21-m2s21-r4100.md](html-semantics-compat/evidence/2026-10-21-m2s21-r4100.md)。
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

# properties-value 系 260F 根因勘察定谳——clone fixture 子树 gCS 通道失联（2026-11-24）

r79 处方（kc5r78 NOTES §4）预定的并行勘察窗产物。四轮单页探针（r75 bin 253ed910
单臂，页放 css/values 语料域外通道定向跑，页跑完即删并核验），全部证据在本目录
`results/css-values.json` 与 `transcript-probe-clone-gcs*.txt`。

## 1. 结论

properties-value 系 260F（003 F=122／implicit-001 F=60／inherit-003 F=60／002 F=18）
的 wholesale fail 形（`assert_not_equals: initial and target values may not match
got disallowed value ""`）根因＝**clone fixture 的后代元素（`.container`/`.transition`）
在宿主 gCS by-handle 通道整体不可解析**：每属性读全部返空串 → from==to='' →
values case 逐属性 fail；样式不落地 → 无 transition → 无 TransitionEnd → events
case 全灭；001／inherit-001／inherit-002 三页变体因 `document.getAnimations()`
返回 shim 侧动画对象而 transitionend 永不触发 → `allTransitionsCompleted` 永不
回调 → 页挂起被 90s 外部墙钟杀（Timeout 形，0F 0P）。

**NOTES §3「transition 简写/transition-property 值串化通道 wholesale 缺口」假说
被本轮探针推翻**——串化与级联通道全部健康（见 §3 反证组）。

## 2. 探针矩阵（v1 probe79／v2 probe79b／v3 probe79c／v4 probe79d）

| # | 形态 | 结果 | 判读 |
|---|------|------|------|
| P1(v1) | clone 后代 inline style → gCS | Fail got "" | clone 后代 gCS 失联 |
| P2(v1) | clone 后代 class-rule（动态 style 元素）→ gCS | Fail got "" | 同上 |
| P3(v1) | createElement inline → gCS | Pass 41px | 动态元素 handle 通道健康 |
| P4(v1) | createElement class-rule → gCS | Pass 53px | 动态元素级联健康 |
| P6(v1) | 静态元素 + 动态 style 元素 rule → gCS | Pass 61px | 动态样式表健康 |
| Q1(v2) | clone 后代 inline（同 turn） | Fail got "" | 复证 P1 |
| Q2(v2) | clone 后代 inline（setTimeout 50ms 跨 turn） | Fail got "" | 跨 turn 不愈 |
| Q3(v2) | **clone 根** inline → gCS | **Pass 19px** | 根可解析——断点只在后代 |
| Q4(v2) | `document.querySelector('#test-N .transition')` | **null** | 文档级查询通道也看不到后代 |
| R1-R3(v3) | clone 根 children.length=1／body.contains(后代)=true／gEBI(根)=非 null | Pass | JS registry 树连通；gEBI 走 pending 索引可见 |
| R4(v3) | createElement + innerHTML 物化子树 → 后代 gCS | **Pass 900** | 同为 SetInnerHtmlOnHandle+物化链的 createElement 路径健康 |
| D1(v4) | 身份字段普查 tSel/tHdl/fSel/fHdl | `null\|__n2\|null\|__n0` | 后代无 `__zwSelector`（null），路由确证走 handle |
| D2(v4) | clone 后代 gCS 路由显式核 | got "" @HANDLE:__n6 | by-handle 通道返空，shim 路由无罪 |

## 3. 定谳推理链

1. D1：clone 后代 `__zwSelector=null` → shim gCS 走 `__zw_get_computed_style_by_handle`
   （callbacks.rs:2916）→ D2 得空串 → 失联点在宿主 handle 视图解析
   （`computed_style_by_handle`，computed_style_cache.rs:862：`nodes.get(handle)`
   miss＋正置表 miss → `(None, None, "")` wholesale 空串；`styles.get` miss 同形）。
2. Q3 根可解析 vs Q1/Q2/D2 后代不可解析——断点精确定位在 **cloneNode deep 分支
   的子树 journal 链**（part04 cloneNode：CreateElement 根→SetAttrOnHandle→
   `SetInnerHtmlOnHandle`（此时根仍 detached）→ 物化 CreateElement/AppendChildByHandle
   链→`id` 改名→AppendChild 进 #offscreen）与 handle 视图 replay
   （`ensure_handle_view_slot` → `apply_dom_mutations_full(&m[applied..])`）之间。
3. R4 反证：createElement+innerHTML+物化子树的 journal 形态与 clone 链同构
   （SetInnerHtmlOnHandle detached 期 journal＋物化 creates＋AppendChildByHandle），
   后代 gCS 却健康——排除「detached 期 SetInnerHtmlOnHandle 一票否决」的简单形。
4. 与 createElement 链的剩余差异（候选断点，待实施轮一次插桩跑定谳）：
   - clone 根先带 id/class 属性再 innerHTML；
   - `id` 改名（fixture→test-N）触发 R45 批内 selector 改写追链；
   - append 父为 #offscreen（positional sel）vs body；
   - cloneNode deep 分支 journal 时序与物化替代物（`_zwFragmentAdded`／
     `_wcRebuildAsHandle`／R2928 惰性物化）的多代 handle 交错。
5. 次生独立面：Q4 文档级复杂选择器查询对 clone 后代 miss（R322 pending 归并
   compound-only 门把链形挡到 host 快照；gEBI 走 R5001 pending 索引所以 R3 可见）
   ——独立 shim 缺口，非本轮 gCS 站点，r80 可并列收。

## 4. 修复站候选（r80 实施轮取舍）

- (a) part04 cloneNode deep 分支 journal 形态：innerHTML/物化链延后到 append 之后
  （或根 attach 后重放）；
- (b) `ensure_handle_view_slot`/`apply_dom_mutations_full` 对 clone 链的注册丢失点
  修复（一次插桩跑定位后收口）；
- (c) shim gCS 对 clone 后代改走可靠通道（兜底形，不推荐首选取）。

预注册门建议（r80）：properties-value 系四 fail 页 F2P≈260−N（N=次要显影面），
三 Timeout 页转绿（pages 完成不再挂），P2F=0 fail-stop，css-multicol 哨兵
SET-IDENTICAL（刀 COLS 窗口基线沿用 r78 探针锚 P=1021 F=592）。

## 5. 证据清单

- `.acceptance/wptfyi-css/kc5r79-scout/results/css-values.json`（末次跑合并 JSON；
  四轮 transcript 均在 `transcript-probe-clone-gcs*.txt`）
- 探针脚本 `probe-clone-gcs.sh`（flock＋systemd transient 6G/RuntimeMaxSec=300，
  页每次跑完即删并核验 PAGE-GONE；四轮发射日志 probe-run*.log）
- 对照数据：kc5r78/results-r75/css-transitions.json（260F fail 形与 Timeout 形原账）

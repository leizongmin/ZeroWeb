# dc2-s10k16 刀⑯ 判定：激活链元素级 getElementsByClassName 基座面（2026-11-25）

## 刀面（根因）

WPT `dom/events/Event-dispatch-single-activation-behavior.html` 全页 **132F wholesale**，
runner 日志逐案同报 `TypeError: e.getElementsByClassName is not a function`：corpus 对
`template.content.children[i].cloneNode(true)` 产物调**元素级** getElementsByClassName。
R151（k15 前刀 fragment 面，part04:2236-2260）只挂 content fragment 本体；part03
`_zwMEl` 装配块产物（part02 `content` getter → `_ensureMutTree()` 链下游）元素本体缺
该方法。

修复＝part03 `_zwMEl` 装配块 R516（+30 行，单文件）：

- `node.getElementsByClassName(names)`：自 `this` 起 children 子树递归 walker
  （`_zwDescChildrenGet` 同款 nodeType===1 过滤），classList **全含 token AND** 匹配；
- 空/空白 token 集 → `[]` 不抛（part06 R185 同口径，防 `[].every` 恒真）；
- DOMString 转换先行（`String(names)`，`null`→`"null"` 单 token 命中面）；
- 不含 self（spec dom-element-getelementsbyclassname 步骤 1-3）；与 R151 fragment
  面、part06 文档级 face 同口径。

## 验证链

- probe_tmp_k16（已删）：runner 日志 detail census（132× 同 TypeError）→ P8 live 域
  有 GECN／P9 parse-snapshot 产物缺＝根因域定位（part02 content getter 链）→ 修复后
  snapKid/snapClone = function；P11 矩阵 cb/form/details/hash 组合链 solo 全过。
- 钉测 `part162_s10k16_activation_gecn_pins`（九段 stage 化 try/catch）1 passed：
  ① 解析快照 template.content 克隆元素 GECN＋live wrapper 子树查询；② token 全含
  AND（miss/hit）；③ 不含 self；④ 空/空白 token 不抛；⑤ DOMString null 转换；
  ⑥ corpus 同构双 fresh clone 挂同一 live target（form pair submit 链＋checkbox pair
  oninput 链）；⑥b parent=form clone＋target=cb clone 入 container 单激活；
  ⑥c checkbox 直挂链；⑦ details ontoggle＋open 翻转。
  - 首版钉测 `RangeError: Maximum call stack size exceeded` stage 隔离定谳＝**已挂
    MEl clone 重挂 live 父 re-parent**（appendChild 独立面，corpus 不触——corpus 每
    pair fresh clone）。改 corpus 同构形后全绿。挂账见下。
- 门禁：fmt 0；全量 zero-engine lib **4274 passed＋1 failed**（s330 前科红非本刀，
  +1＝本钉）；workspace clippy 0 warning；release 构建 0。
- 刀体 commit：89d39b27a → rebase 重放 **e9e6238df**（上游 731873eb0＝html-semantics
  s745 docs-only 零码面）；lei-pre-commit-guard PASS（0 发现 0 提醒，3 文件 +165 行）。
- 判读绑定二进制 md5 **31809eca644c7056a4b45a44e6fa74e2**（特征串 `_r516Wanted`×3／
  `R516`×1 内嵌验证；launcher EXPECT_MD5 断言先行）。

## 电池 v1（五腿 join，23:22:35Z–23:56:00Z＝33.4min）

| 腿 | preF（k15 终态锚） | postF | GONE | P2F | NET_P | post totals |
|---|---|---|---|---|---|---|
| WC | 1118 | 1118 | 0 | 0 | 0 | 18539P/1118F/20T |
| domparsing | 222 | 222 | 0 | 0 | 0 | 1313P/222F/1T |
| sanitizer | 50 | 50 | 0 | 0 | 0 | 706P/50F/1T |
| trusted-types | 362 | 362 | 0 | 0 | 0 | 1728P/362F/64T/17NR |
| **dom** | **496** | **485** | **11** | **0** | **+11** | 57277P/485F/12T |

**join GONE=11／P2F=0／NET +11，零附带损伤**（WC/DP/SAN/TT F 集逐键恒等）。

### GONE 构成（11）

全部为 child=`<LABEL><BUTTON>` 形跨 8 parent 形（A／AREA／DETAILS／FORM+BUTTON×2
变体／FORM+INPUT×3 变体／INPUT×2 变体／LABEL+INPUT+SPAN）：LABEL 包 BUTTON 子激活
链在 GECN 可达后整族翻面（单一名字族，无散点）。

### GECN TypeError 清账证明

- `getElementsByClassName is not a function`：pre（k15 dom log）**132** → post **0**；
- `is not a function` 总计数 301 → 169（Δ=132 精确＝本页 wholesale 族；残 169 属
  createNodeIterator/createValueRange 等其他页既有域，不在本刀面）。

## 残面 census（121F，挂刀⑰）

页面 GECN 门解除后残 121F 全为**激活 containment 组合断言失配**（非异常）：

1. child=FORM 形（FORM+submit INPUT／FORM+submit·reset BUTTON）× 各 parent：
   嵌套双层提交链只激活内层 face（~46）；
2. parent=INPUT（void 元素作父嵌套形，probe P11 已知 acts=[]）：~20；
3. a／area 参与 cross 组合：hashchange window handler 异步 face（corpus
   promise_test＋double step_timeout，runner 判定待 probe）；
4. details／label 跨边界转发组合散点。

## 挂账

- **刀⑰ 候选 A**：激活 containment 组合面 ~121F（本页残面；probe 先行 instrument
  acts 数组映射 produced-vs-expected）；
- **刀⑰ 候选 B**：TT script-enforcement-001..004 族 ~104F（provenance 状态机跨
  mutation API，架构面）；次选 WC adopted-callback 66F／TT HTMLElement-generic 48F；
- 已挂 MEl clone 重挂 live 父 re-parent → `RangeError`（appendChild 独立面，
  corpus 不触，钉测已避开并注记）；
- san 残面 50F 挂账池照旧；协议刀（InsertAdjacentHtmlOnHandle）挂账。

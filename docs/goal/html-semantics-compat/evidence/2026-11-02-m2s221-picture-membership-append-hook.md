# M2 s221 — picture 成员变更 selection 记录钩 + append 路由补齐

日期：2026-11-02。切片号 s221（承 s220）。

## 入场定刀位

s220 挂账「img-into-picture 插入钩（appendChild trap IMG 直接父 PICTURE →
微任务 re-record，001 断言 4 收口位）」单点小刀。

## 勘根

两段定谳：

- **钩位段**：part03 `__zwImgTreeMutated5277` IMG 臂（s70 起）对「img 插入
  且父为 picture」已走 `__zwImgRevalidate5277`（fetch 半）；001 断言 4 缺的
  是 selection 记录微任务半（s220 面同根）→ IMG 臂补 Queue 即可，无需新
  trap 臂。
- **路由段**（补 Queue 后 001 仍红定谳）：二进制新鲜度核对排除陈旧；读
  `Element.append()` 实现（part04:10197）——现代 API 走 `_appendVariadic`
  专路（内部 Rust callback），**不经 appendChild trap 尾（7830）**，
  TreeMutated 从未触发。modern API 覆盖面普查：appendChild/insertBefore/
  removeChild 已有 TreeMutated 尾钩；`append`/`replaceChildren` 走
  `_appendVariadic`（无尾钩）；`prepend` 走 `_insertAdjacentVariadic`（无
  在手 added 列表）；`before`/`after`/`insertAdjacent*` 参考元素非父容器
  （需父解析）。

001 断言 3（append 后 sync 读仍 c）天然保持：入队实例未落，记录表持旧值。

## 施工（shim 2 文件 +15，语料零改动，零 Rust）

- **part03** TreeMutated IMG 臂：`__zwImgRevalidate5277` 前补
  `__zwImgSelectionQueue5277`（picture 成员变更也是 selection 面 relevant
  mutation；sync 读见旧记录、微任务落新值）。
- **part04** `Element.append()` impl 既有 `added` 循环（_ceApplyConn/
  _zwStartConnectedIframe 同环）补 TreeMutated 调用（element/picture 门在
  TreeMutated 内，text 节点早退）。
- prepend/replaceChildren/before/after/insertAdjacent* 不入刀：无语料驱动
  （挂账新记）；不为一次使用扩面。

## 验证

- shim 拼接 15 part → node --check SYNTAX_OK。
- 靶语料 solo：current-request-microtask **001 F→P**（断言 4 data:,b ✓）+
  002 保持 Pass；img-picture-ancestor **4/4 保持**。
- zero-engine 全量 **3684P + 0F**（与 s220 基线持平——含并行 main 流
  22acb33eb typing 面合入后的组合态，零回归）。日志
  evidence/logs/s221-zero-engine-full.log。
- embedded-content 域 A/B（A = s221 修复后、B = s220 基线日志）：共享键
  **1257/1257**，漂移恰 **1 F→P**（current-request-microtask——全靶面），
  **P→F = 0**，其他位移（含 Timeout 摆动）**0**。日志 evidence/logs/
  s221-embedded-content-wip.log。方向口径同 s220（$2 = A 侧）。
- 本切片纯 JS（git diff --stat +15/−0）——fmt/clippy 不适用。

## 挂账新记

- **modern API img 树变更面余项**：prepend/replaceChildren/before/after/
  insertAdjacent{HTML,Text,Element} 不经 appendChild trap 尾亦无 TreeMutated
  汇流——无语料驱动（A/B 零位移实证非当前缺口），picture 成员语料出现时
  再收。
- TreeMutated IMG **移除**臂（img 移出 picture → selection 重记录）未挂
  Queue——spec 对称面，无语料驱动。

## 承袭挂账（不动）

fail-to-resolve 4F（URL 解析器面爆炸半径大独立勘根）；not-broken 2F（attr
sizing/rect 面）；decode quick-attach 2F；update-the-source-set
`;encodings=` 1F；update-media/source-relevant-mutations 选源响应面余案；
layout-engine 1px 钳（sizing.rs:2899-2900/3136 渲染流）；error.html 破图
alt 7F；sizes-auto Timeout 68 案；parse-a-srcset URL serializer 1F；SVG
data-URL intrinsic 3F + img naturalWidth SVG viewBox 族 22F（decoder 耦合
renderer 流 ③A）；svg-img-density-vs-zoom 7F；decode 2F；lazy-loading 族
~24F；part06 `_zwFieldsetDisableFocusReset`（s215）；part03
`_zwMElCheckValidity` FORM 短路（s210）；onblur flake 族（s209~）；
beforeinput.tentative；非 canvas 面一般渲染门 FIXME(rendering)（s218）；
flat tree slot FIXME(flat-tree)（s218）；用户门禁三项（GB-20261031
customizable select 177F / form-nameditem override builtins / form owner
table 缓行族）。

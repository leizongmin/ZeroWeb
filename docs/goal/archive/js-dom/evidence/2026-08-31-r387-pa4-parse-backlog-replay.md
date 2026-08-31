# R387 — pa4-lite 解析积压 MO 回放 + 动态 classic 脚本插入期执行（M4 已知 Fail 收口）

**日期**: 2026-08-31
**HEAD**: `c1a518bb0`（基线 `fa672ed08`，R386 收官态）
**切片**: pending-apply RFC pa4-lite（`docs/specs/js-dom-pending-apply-lifecycle-rfc.md` §2 pa4 的
shim 侧最小实现）+ 动态脚本执行（spec `prepare the script element` 缺口，探针发现于同用例链）
**改动面**: `js_dom_shim/part01.js`（回放）+ `js_dom_shim/part04.js`（appendChild 脚本执行）+
`js_dom_bridge_tests/part25.rs`（回归测试）

---

## 1. 切片缘起

M4 已知 Fail 集合中 `MutationObserver-document` 3F 定性为 parse-time MO 架构域（R373 备档）。
R385 收官后按 RFC §0.3 pa4「parse-segment 回放」方向重评：**不需要 host 分段 delta**——
shim 侧已有 `document.currentScript`（R3258 文档序锚）+ 融合 childNodes 视图（R51/R55），
注册点的「解析积压」可纯 shim 合成。

## 2. 实现

### 2.1 解析积压回放（part01 `_moReplayParseBacklog`）

**根因**：本仓架构「整树解析完 → 按文档序执行脚本」，解析插入先于脚本执行；
`_mo_notify` 只在 JS mutation 时发声——注册脚本之后的解析插入（`<p id=n00>` 等）永不产生
record（"parser insertion mutations" assert_unreached 直接根因）。

**方案**：`observe(document, {subtree, childList})` 且处于 classic 脚本执行期
（`document.currentScript` 存在）时，以 currentScript 为位置锚，枚举**同父（body 直下）之后、
直至（含）下一个 `<script>` 元素**的段内节点，按序合成 childList record（addedNodes=[节点]、
previousSibling=前驱、target=父容器 proxy；段尾 script 的解析期文本子单发一条 target=script 的
record），投递到 'doc' 站（R188 通道，requireSubtree 语义不变）。

**保守门**（防 spurious record 波及 135 个既有 MO Pass）：
1. 仅 document 目标 + `subtree+childList` 双开；
2. 仅 body 直下脚本回放——嵌套脚本（如 removal 用例的 div 内 s011）的后续兄弟在真实流式
   解析下**尚未插入**，回放会产生测试不期望的 record（首轮实测 removal 用例 count 1→2 失败
   形态恶化，加门后回到其文档化的 nextSibling parse-position 可见性失败形态）；
3. 段止于首个 script 元素（流式解析的微任务 checkpoint 恰落在脚本执行间）；
4. 回放失败静默（不阻断 observe 注册）。

**同 turn JS 插入零误报**：JS mutation 在 host apply 前不在快照树中——注册时快照含的
「脚本之后的元素」只可能是解析产物或**先前轮次**的 JS 产物（前者正是回放目标，后者的段
边界门天然排除 append-to-end 形态）。

### 2.2 动态 classic 脚本插入期执行（part04 appendChild 尾部）

回放使 "parser script insertion mutation" 的 observer 触发后，暴露第二层缺口：
`n00.appendChild(newScript)`（createElement + textContent= + appendChild）后脚本**不执行**——
`inserted_element` 永不被 append，batch2 只有 1 条 record（期望 2）。

**spec**：无 async/defer 的 classic 脚本在「becomes ready to be script-executed」（插入文档）
即同步执行——SPA 加载器/分析 SDK 的标准装载路径。修：appendChild 主路径返回前，SCRIPT
handle 子经融合 `textContent` getter 取源码（**探针实证** `textContent=` 经 `_zwRegisterTextEl`
落文本注册表而非 `_handleChildren`——R377 的 registry 收集对 textContent 形态恒空，首版
钩子 no-op），`(0,eval)` 全局作用域执行 + `_zwRanScripts` run-once 标记（复用 R377 语义）+
异常 report-the-exception。仅 handle 子（动态创建形态）；sel 静态子解析期已跑过不重跑。

## 3. 验证

| 门 | 结果 |
|----|------|
| 目标用例 | MutationObserver-document：**"parser insertion mutations" F→P**（3F→2F）；setup 恒 P；余 2F 定性不变（见 §4） |
| MutationObserver 全族 | 135P/3F → **136P/2F**（零回归） |
| dom/nodes 全量 | 12791P/**3F**（基线 4F，-1；Fail 集合 = document 2F + thcrash 1F，余恒等） |
| dom/events 全量 | 597P/1F/10T——1F = click-on-absolute-pseudo（基线同）；10T = 文档化轮转族（Event-dispatch-click 单跑复验 32P，R384 先例） |
| engine v8 / quickjs 单测 | 2510P（+1 回归测试 `r387_dynamic_script_append_executes`）/ 1482P |
| `make test` 全量 | **18505P / 0F**（EXIT 0；R386 基线 18504 +1 新测试） |
| bench-gate 定向（zero-engine/webview/script-sandbox，load1=0.3 空窗） | **GATE PASS 42/42（NEW=0）** |
| clippy v8 + quickjs（`-D warnings`） | 零警告 |
| fmt | 无 diff |

## 4. 剩余 2F 定性（R385 Fail 集合收敛后）

| 用例 | 失败断言 | 定性 |
|------|----------|------|
| MutationObserver-document "parser script insertion mutation" | `previousSibling` identity：期望 `#s002` query proxy，实得 positional-selector wrapper（`html>body>script:nth-child(10)`） | **L2 身份域**——解析元素 childNodes 包装的 selector 规范化（id 优先 vs positional）是 R43/R334 同族身份统一问题，随 L2 主线 |
| MutationObserver-document "removal of parent during parsing" | `nextSibling` 期望 null 实得 `#s012` | **parse-position 可见性架构域**（R373 定档不变）——JS 观察到全预解析树，需「解析位置面」才可修，深结构 |

## 5. 教训

1. **回放门不足即失败形态恶化**——嵌套脚本段的 spurious record 使 removal 用例从
   nextSibling 断言失败变 count 失败；保守门（body 直下）不仅修复还原能力，也是失败
   形态保持可归因的前提。
2. **textContent= 的落点与 innerHTML= 不同**——前者走文本注册表、后者走
   `_handleChildren`/解析视图；R377 复用时必须探针验证源码读取面（首版钩子静默 no-op）。
3. 架构域 Fail 的「不可能」定性要按 RFC 切片重评——pa4 原案（host 分段 delta）是深结构，
   shim 侧以 R3258 锚点 + 融合视图即可服务主要断言面。

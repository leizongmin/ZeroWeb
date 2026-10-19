# P6b 修复——escaping noscript 真实漂移归因 + 收口（2026-10-20）

## 结果

`escaping.html` **8/9 → 9/9 全绿**（div.innerHTML 翻绿），全通道
**61229/61333 = 99.83%**（P6 冻结 61228 + 1）。vs-M4 漂移账清零：form-pointer
（P6 改判 M4 既有账，已修）+ escaping noscript（本轮归因修复）+ script-inserted
双案（rendering-compat 域挂账维持）。

## 归因（bisect 落点有效 + 机制闭环）

git bisect（f54ab9285 good → HEAD）首个坏 commit **9dcd16661（2026-10-10，
fix(engine-dom) innerHTML 本地视图产物物化真 handle 树——bilibili nano 小部件
修复）**。好端点先验（P6 教训）：f54ab9285 上 escaping.html 9/9（M3a/M4 证据
实记），bisect 落点为行为性 commit，链路单调闭合。

机制：9dcd16661 前模板/div innerHTML 产物是 plain `_zwMEl`——`noscript.innerHTML`
读走 `_zwMSerialize`（有 R5001 noscript literal 分支：parent 为 noscript 且
scripting enabled → data 直出）。物化为真 handle 树后，读走 part04 innerHTML
getter 的 handle 融合序列化路径——该路径对文本子**无条件 `_zwMEscapeText`**，
缺 noscript 分支 → 主文档形态字面 `& <>` 被再转义为 `&amp; &lt;&gt;`。

## 修复（R5197）

- **part04 innerHTML getter 融合序列化**：文本子分支补 noscript literal 判据
  （对齐 `_zwMSerialize` R5001）。旗标判定：子节点 `__zwTreeScripting === false`
  印章优先，回落 ownerDocument `__zwScriptingEnabled === false`
  （contents owner doc / detached doc 工厂恒盖，spec 无 browsing context →
  scripting disabled）。
- **`_wcRebuildAsHandle` 印章传递**：text node 物化（新建主文档 textNode）传递
  源 `__zwTreeScripting === false` 印章——disabled 惯性树的序列化判据在 handle
  域存续。
- spec 依据：noscript 内容模型随 scripting 旗标（enabled → rawtext，序列化直出；
  disabled → markup 解析实体解码，序列化再转义）。
  https://html.spec.whatwg.org/multipage/parsing.html#serialising-html-fragments

## 常驻测试

- `test_noscript_serialization_literal_escape_r5197`（part21.rs）双面：主文档
  div 直出字面 `& <>` + 模板 content 再转义 `&amp;&nbsp;&lt;&gt;`。
- WPT escaping.html 9/9（通道常驻）。

## 门禁与账目

- 全通道 61229/61333（+1P）；fmt / guarded clippy / make test 同轮复验
  （GPU 2F 既有碰头信号维持）。
- vs-M4 漂移账清零（余 script-inserted 双案归 rendering-compat 域）；escaping
  域恢复 M3a 全绿态。

## 组合态复测（2026-10-20，rebase origin/main 4bc987928 后）

落地前按 run-rules §10 组合态核对重跑全通道：**61233/61333 = 99.83%**
（Pass 61233 + Fail 87 + Timeout 13，分母 61333 不变）。相对上段 61229 记账
**+4P / −9F / +5T 全部落在 render-blocking 挂账域**（65F→58F，其中
remove-element-unblocks-rendering.optional 4 子测含 module 双变体 Timeout→Pass，
与并行 script-sandbox 流 8458c9223/a85c7ade4 ES module 每文档缓存落地方向
吻合；Timeout 8→13 为同域 case 形态迁移）。本 goal 域逐桶核账零回归：
escaping.html 9/9、form-pointer Pass、serializing 双域 137+112 全绿、charset
22F（XHR 新账逐案吻合）、parsing 4F（tokenizer 缝 2 + 读一致性 2）、
document.all 1F、aria 2F（async-apply 缝）——与残差账结构一致。门禁同轮实测：
part21 r5197 常驻测试绿 / fmt clean / guarded clippy 干净 / make test 全绿。

# M3 切片 R92 — lit 风格模板渲染进 shadow root

**日期**: 2026-08-17
**Commit**: `f59a4be7`

## 资产

WC e2e 断言组 6 `wc_lit_style_template_render`——lit 的核心渲染原语（innerHTML 模板 + 插值 + re-render 覆盖）进 shadow root：
- `render(root, title, val)` 写 `<style>:host</style><div class=card><h2 id=t>…</h2><p class=v>…</p><span class=nested><em>deep</em></span></div>`
- 断言：shadowRoot 可读 / `querySelector('h2')` 文本 / `#t` / `.v` / 后代组合器 `.nested em`（depth-3）/ `.card` childNodes=3 / `<style>` 命中 / 二次渲染 `Title-99` 覆盖

## 根因与修复

**根因**：innerHTML= 在 handle 容器（shadow root）的解析子树是 `_zwMEl` 快照代理（无 `__zwHandle`）——只 depth-1 子入 `_handleChildren` registry，`_handleSubtreeNodes` 的 registry DFS 收不到嵌套层 → h2/#t/.v 全 miss。

**修复**：`_handleSubtreeNodes`（part05）visit 对 handle-less 解析节点直接沿 childNodes 展开——元素子入 result、文档序、兄弟上下文（prevSibling/prevSiblings）、infoByProxy 对象 identity、expandDeep 递归嵌套（组合器链可回溯）。

## probe 方法论

1. selector-kind 分解（div:1 / h2:0 / #t:0，card.childNodes 可见 H2）→ 一次定位层级边界（depth-1 在、嵌套不在）。
2. 手动 appendChild 对照（manual-h2:1）→ 排除选择器匹配器本身，锁定数据源。

## 结果

| 项 | 前 | 后 |
|----|----|----|
| nodes | — | 净 +3（Node-parentElement 修复 + 2 flake 消退） |
| traversal / events / collections | — | per-case 不变（1593P/11F） |
| integration | 772 | **773（+lit 组）** |
| engine v8 / quickjs | — | 2188 / 1427 全绿 |

fmt 无 diff；clippy 双矩阵零警告；pre-commit-guard PASS。

## 延后

- Proxy-ctor 桥（new 捕获 this 移植到升级 proxy——解 lit constructor 内初始化限制）：本轮未做，下轮候选。

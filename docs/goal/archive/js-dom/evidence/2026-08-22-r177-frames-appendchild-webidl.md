# R177 Evidence — window.frames + appendChild WebIDL/adopt 语义（M4）

**日期**: 2026-08-22
**切片**: M4 轻量——Node-removeChild 9F + Node-appendChild 7F 双收口（两用例全 100%），全量净 +16P/-16F
**改动面**: part01（frames）+ part04（appendChild WebIDL/doctype-guard/adopt）+ part05（iframe 空文档 docEl/text 节点）+ part06（doctype mutation 面）

## 一、window.frames（两用例的共同前置）

**根因**：`frames` 集合完全缺失——`frames[0].document` 直接 ReferenceError 整簇 fail
（两用例 16F 中 9F 是它）。

**实现**（part01）：Proxy 动态枚举——每次 get 现查 `document.querySelectorAll('iframe')`
取 contentWindow。首版**快照对象**失败（建 frames 时 about:blank iframe 的
contentWindow 尚未 lazy 物化——快照 miss 返 undefined），Proxy 动态读保证任意时刻
拿到已注册 iframe。spec：HTML「window named access」索引访问面。

## 二、Node-removeChild（9F→0F，28P/28 全 100%）

| 修 | 内容 |
|----|------|
| frames | 上文 Proxy |
| 空文档 documentElement | about:blank iframe doc 的 docEl 为 null（markup 空无根元素）——`doc.documentElement.appendChild` TypeError；补合成 html 元素（HTML ns + appendChild/removeChild/导航面——spec：HTML 文档恒有 html 根） |
| text 节点 ownerDocument | iframe `createTextNode` 产物缺 ownerDocument 字段——补 `ownerDocument: doc` |
| text removeChild 不抛 | text 节点无 childNodes 字段 → `Node.prototype.removeChild` own-childNodes 分支落 lenient return；补 `childNodes: []`（叶子视图——视图在即按视图校验抛 NotFoundError） |

## 三、Node-appendChild（7F→0F，11P/11 全 100%）

| 修 | 内容 |
|----|------|
| WebIDL TypeError 前置 | 元素 appendChild + 叶子（text/comment）mutation 族的 null/非 Node 参数——TypeError 先于一切 pre-insert 步骤（spec WebIDL nullable Node；旧版 null 落 no-op 或 HierarchyRequestError 均错） |
| doctype mutation 面 | 主文档 doctype 缺 appendChild/insertBefore——补抛 HierarchyRequestError（叶子节点语义；旧 `node.appendChild is not a function` TypeError 非 DOMException） |
| Document 不能作 child | `document.body.appendChild(frameDoc)` 静默 no-op → HierarchyRequestError（spec pre-insert 步骤 1） |
| 跨文档 adopt | plain 子（iframe 工厂产物，无 handle）append 到主文档元素后 ownerDocument 重指主 document（spec `concept-node-adopt`；sel 父 + handle 父两条路径都补 defineProperty 遮蔽） |

## 四、验证

| 门 | 结果 |
|----|------|
| Node-removeChild | 19P/9F → **28P/0F（100%）** |
| Node-appendChild | 4P/7F → **11P/0F（100%）** |
| 全量 dom WPT polyfill | **9613P/251F/19T**（R176 9597P/267F——**净 +16P/-16F** 全来自两用例，零回归） |
| 全量 dom WPT native | **9613P/251F/19T**，per-file 与 polyfill 零差异 |
| `make test` | 66 套件 **18125P/0F**（首跑 SW 已知 flake 1F，二次全量绿——service-workers 流域观察项） |
| fmt / clippy | 干净 |

## 五、下一步（R178）

- 全量 fail Top 簇：Event-dispatch-single-activation-behavior 14F /
  node-creation-realm 13F / Range-attribute-nodes 11F。
- tree order 2F 记 RFC（identity 归一域）。
- M2/M6 面：S6 高层 API 去字符串 / native dom_bindings 补齐。

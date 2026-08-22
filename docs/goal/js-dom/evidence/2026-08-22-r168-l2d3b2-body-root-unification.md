# R168 Evidence — L2-d3b2 body 根归一收口 bubbles 残留（M1）

**日期**: 2026-08-22
**Commit**: `9d62b27fa`（rebase 后）
**切片**: M1 L2-d3b2——R167 残留的 Event-dispatch-bubbles 两变体（cloneNode/new Document，12 vs 14 站）修复

## 一、定位过程（二分链，对同类问题有方法论价值）

1. 沙箱探针（new Document + 7 站注册）**14 站全过** → 差异不在链派发本身。
2. runner 内探针（zz- 页面）**也过** → 差异不在 runner 环境。
3. **复刻原用例 helpers testChain → fail**；自有 makeChain → pass——同文件同序最小对。
4. 逐 token 对比锁定：makeChain `target` 先查（建树）；原版 `targetsForDocumentChain` 先查（body 第三个）。
5. **probe4 链 dump 实证**：fail 变体链止于 `_tree` 根（BODY），bodyInChain:**false**——注册的 body 站（D 域 wrapper）与链上 `_tree` 根（C 域）**身份错位**。
6. 根因：body 查询（R161 容器例外走 JSON 往返）的 key.outer 是 **host re-parse 序列化**，与树根 `_zwMSerialize` 的属性序/转义细节不一致 → 归一键 miss → wrapper 产物。

## 二、修复（三件，每件由一次中间回归实证约束）

| 件 | 内容 | 中间回归教训 |
|----|------|--------------|
| root-hit 特判 | `_zwMFindRealNode`：root 有 `_zwOwnerDetDoc` 印章 + 键 tag 段与 root nodeName 一致 → root 本体即真实节点（doc 级唯一 body 语义） | 首版全 tag 命中不限印章 → 元素子树查询返 root 自身（Element-matches 55F / ParentNode 241F）——spec descendants-only 不可破 |
| 印章只盖根 | `ensureTree` 的 stamp 从全树 DFS 改 `_tree` 根独占 | 全树 stamp 使树内任意元素作查询 root 时命中特判（ParentNode "got root" 139F） |
| body 属性落根 | iframe 文档的 `_r159BodyAttrs`（R159 提取的原始 attrs 串）解析到树根（id/class IDL 反射 + setAttribute） | 归一后 `querySelector('body')` 产物是树根，id 空 → WPT expected id "body" fail（+2F） |

## 三、验证（全量）

| 门 | 结果 |
|----|------|
| Event-dispatch-bubbles（两文件） | **10P/0F**（基线 10P；R167 曾 6P/4F） |
| Element-matches / webkitMatchesSelector | 3F（= 基线） |
| ParentNode-querySelector-All | 33F（= 基线；中途 241F/139F/35F 三次回归当轮修平） |
| 全量 dom WPT polyfill | **9522P/343F/18T**（R167 9518P/347F——**净 +4P/-4F**） |
| 全量 dom WPT native | **9521P/343F/19T**，per-file fail 与 polyfill **零差异** |
| `make test` | 66 套件 **18068P/0F** |
| fmt / clippy（v8 + quickjs 矩阵） | 干净 |

## 四、下一步（R169）

- **d3c**：doc 上下文 compound gate（queryBody 形态门扩 `_queryTreeByCompound` 全形态——R165 实证 doc 上下文无回归，消 doc 上下文 compound 的 JSON 往返）。
- **d3d**：element/fragment 本树化（R165 902F 回归面被桥+root-hit 消解大半后重评估）。

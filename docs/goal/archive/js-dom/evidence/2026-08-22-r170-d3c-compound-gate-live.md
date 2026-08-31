# R170 Evidence — L2-d3c compound gate 落地（doc 上下文本树查询开通）

**日期**: 2026-08-22
**Commit**: `0a4146465`
**切片**: M1 L2-d3c——R165 预留的 compound 解析 gate 正式启用（doc 上下文 compound 形态消 JSON 往返）+ 两个单变量实验暴露的配套修复

## 一、定位链（单变量实验法）

1. R169 假设（iframe 双工厂）**被推翻**：src-iframe（WPT 真实形态）的 bodyHtml
   有内容、树正常（probe 实证 `cls:1|tagCls:1`）；R169 探针用 srcdoc（无 src
   分支建空文档）是伪根因。
2. gate-on 单变量实验：ParentNode 904F（与 R169 的 905F 同源）。
3. **DBG 内探针**（queryBody 命中分支注入诊断）：`[id="root"]:hits=1:t=DIV`——
   树查**命中正确节点**，但外层 `element.tagName` 空。
4. **根因**：`queryOne` → `_zwWrapCached(a[0])`——a[0] 是**真实节点**（无 `.tag`
   字段，是 `.tagName`）→ key 的 tag 段空 → **键撞车**命中无关缓存条目 → 空壳。
   gate 前该路径只收 JSON info（有 `.tag`），形态切换暴露了 key 构造的单一形态假设。
5. 修 key 后 904F→35F；剩 +2F = 解析器**吞空白**（`#descendant div` 的 token
   抽取把组合器形态误判为同 compound，返 root 自身）——入口加
   `/[\s>+~:]/` 整体拒绝。

## 二、落地内容

| 件 | 内容 |
|----|------|
| gate 启用 | compound 形态（tag?/#id/.class×n/[attr]/[attr="v"]，顺序任意）走 `_queryTreeByCompound` 本树遍历；零命中/中止回落 JSON（host 权威） |
| key 双形态 | `_zwWrapCached` 的 key 构造兼容 JSON info（`.tag`/`.outer`）与真实节点（`.tagName`/`.outerHTML`） |
| 组合器守卫 | 含空白/`>`/`+`/`~`/`:` 的形态整体拒绝走 JSON |

## 三、验证

| 门 | 结果 |
|----|------|
| ParentNode-querySelector-All | 33F（= 基线；中途 904F→35F→33F 两步修平） |
| Element-matches / webkitMatchesSelector | 3F（= 基线） |
| Event-dispatch-bubbles | 0F（保持） |
| 全量 dom WPT polyfill | **9521P/343F/19T**（R168 9522P/343F/18T——±1P/1T 边缘漂移，per-file fail 零差异） |
| 全量 dom WPT native | **9522P/343F/18T**，per-file 与 polyfill 零差异 |
| `make test` | 66 套件全绿（SW 1 flake 单跑绿，webview 零改动） |
| fmt / clippy | 干净 |

## 四、下一步（R171）

- **d3d 重评估**：element/fragment 上下文本树化（R165 902F 回归面的剩余部分——
  key 双形态修复可能已消解其大半，重跑形态门扩展实验）。
- Element-matches 剩 3F（`[*|TiTlE]` 树碎片化域）/ ParentNode 剩 33F 聚类。

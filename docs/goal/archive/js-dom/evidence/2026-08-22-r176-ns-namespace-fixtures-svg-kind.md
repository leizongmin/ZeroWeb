# R176 Evidence — Document-createElement-namespace 40F 全收口：fixture 子目录 fetch + svg kind（M4）

**日期**: 2026-08-22
**切片**: M4 轻量——Document-createElement-namespace 40F → **0F（51P/51 全 100%）**，全量净 +40P/-40F
**改动面**: fetch-dom-subset.sh（fixture 子目录 + 空文件放行 + 扩展过滤）+ part04（.svg kind）+ part05（svg contentType 分支）

## 一、根因（两层）

1. **fixture 子目录未 fetch**（30F 层）——`fetch-dom-subset.sh` 的
   `fetch_dir_html` 只拉 `.html`/`.js` 且不递归子目录；
   `Document-createElement-namespace-tests/`（4 扩展 × 10 形态 = 40 外部
   文档）整个缺失 → iframe `contentDocument` null → "Cannot read
   properties of null (reading 'contentType')" 整簇 fail。
2. **`.svg` kind 缺失**（10F 层）——fixture 就位后剩 `.svg` 扩展族：
   R175 的 kind 三分漏 svg → contentType 'application/xml' ≠ 期望
   'image/svg+xml'。

## 二、修复（三件）

| 件 | 内容 |
|----|------|
| **fetch 脚本** | ① `fetch_dir_html` 增拉该 fixture 子目录（41 文件）；② 扩展过滤追加 `.xml/.xhtml/.svg`；③ **空文件放行**——`empty.*` fixture 是设计为 0 字节的测试形态，旧 `test -s` 把 curl 成功的空响应当失败（.tmp 残留使 fixture 永远拉不到），改 `test -e`（curl --fail 已保证非 2xx 报错） |
| **part04 kind** | `.svg` 扩展 → 'svg' kind（四分：.xhtml/.html/.svg/其它） |
| **part05 分支** | 'svg' kind → contentType 'image/svg+xml'（真浏览器按扩展判 SVG 文档；createElement 语义与 XML 同——ns 按文档派生 null） |

## 三、验证

| 门 | 结果 |
|----|------|
| Document-createElement-namespace | 11P/40F → **51P/0F（100%）** |
| 全量 dom WPT polyfill | **9597P/267F/19T**（R175 9557P/307F——**净 +40P/-40F**，全部来自本簇，零新增） |
| 全量 dom WPT native | **9596P/267F/20T**，per-file 与 polyfill 零差异（唯一分歧 = 已知 flaky insertBefore-iframe-crash 超时互换） |
| `make test` | 66 套件 **18123P/0F** 一次通过 |
| fmt / clippy | 干净 |

## 四、下一步（R177）

- 全量 fail Top 簇重聚类：Event-dispatch-single-activation-behavior 14F /
  node-creation-realm 13F / Range-attribute-nodes 11F。
- tree order 2F 记 RFC（identity 归一域）。
- M2/M6 面：S6 高层 API 去字符串 / native dom_bindings 补齐。

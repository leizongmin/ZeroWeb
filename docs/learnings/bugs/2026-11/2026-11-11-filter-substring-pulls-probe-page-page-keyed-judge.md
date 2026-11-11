---
date: 2026-11-11
modules: tests/wpt-runner, editing-compat
---

# FILTER 子串匹配会拉入探针页——judge 键剥页名即碰撞，判读必须页键过滤

## 问题

outdent 刀门禁批 judge 首跑出现 **P2F=5 + new-row 1** 伪象：刀体只加不改，
indent/outdent 邻页零预期变化，P2F 不可能非零。

## 根因

wpt-runner 的 `FILTER=outdent.html` 是**子串匹配**，会同时拉入
`run/zz-probe-outdent.html`（探针页文件名含语料页名子串）。批日志
`P=2753` = 语料 2538 + 探针 215 即拉入实证。

而 judge 比对脚本把行解析成键时**剥掉页名前缀**（`run/zz-probe-outdent.html?…`
→ `…`），探针键与语料键同形。后果两条：

1. 探针新红键覆盖语料同名键 → 误计 P2F；
2. 探针新增键（如 PICKS-FOUND）被判 new-row。

## 修法

judge 行过滤加页键谓词，键保留页名：

```python
if line.startswith(("Pass ", "Fail ")) and "run/outdent.html?" in line.split(" :: ", 1)[0]
```

修后真值 F2P=161 / P2F=0 / lost=0 / residF=12 定谳。

## 通用规则

- **凡新增与语料页同名的探针页（或任何 FILTER 命名的超集页），judge 必须按页键过滤**，不能按剥页名键判读；
- 或者探针命名避开语料页名子串（例如语料 `outdent.html` 存在时不用 `xx-outdent.html` 命名）——但页键过滤仍是判读侧根本防线，命名规避只是减噪；
- `P>0` 形计数门不防碰撞（见姊妹篇 subset-tests 空转假绿），**恰集计数 + 键清单**才是有效门。

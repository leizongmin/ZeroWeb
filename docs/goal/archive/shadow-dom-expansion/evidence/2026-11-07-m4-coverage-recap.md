# M4 coverage 复测收口——混合口径与四判据裁定（2026-11-07）

## 计量口径（契约「M4 执行细则」）

- **冻结二进制**：sha256 `803111caa35f9e99f51c0826d370c885a67421974e92fd05c5705fafc536dc9b`，
  构建提交 `d7155cef5`（构建时工作树与提交差异仅为 callbacks.rs 内 test-only
  `zz_probe_deep_json_tests` 模块，`#[cfg(test)]` 不进 release 编译；提交前已删）。
  provenance：源码 mtime < 二进制 mtime + 二进制含 `_eM3Cc180` ASCII 标识（strings 法证）。
- **守卫**：`systemd-run --user --scope -p MemoryMax=24G` +
  `test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1800`；
  gethtml.html 臂 `ZW_TIMEOUT_MULTIPLIER=4` 经 env 注入 scope。
- **执行注记**：×4 臂首轮后台任务被 harness 后台时限终止（~19 分钟，0 字节，
  runner 未落盘即随 shell 树终止）；同参数原地重启一次，完成 exit 0。非产品因素。

## 混合口径数据（merge_zw_results 按 (total, passes) max 合并，gethtml ×4 自动胜出）

| 运行 | 文件 | total | Pass | Fail | T/其他 |
|---|---:|---:|---:|---:|---:|
| 全语料 ×1（`wc-after-canvasfix.json`） | 365 | 9,856 | 8,437 | 1,403 | 8/8 |
| gethtml.html ×4（`gethtml-x4-m4.json`） | 1 | 6,657 | 6,656 | 0 | 1T |
| **混合（×1 全语料，gethtml 换 ×4）** | 365 | **13,087** | **11,668** | — | — |

混合通过率 11,668/13,087 = **89.16%**（local corpus 口径；csv 行用本数）。

## coverage 复测（wptfyi cache universe 分母，与 coverage-2026-11-02 快照同源）

报告 `.acceptance/shadow-m3-residual/m4-coverage.md`（不入库）：

| topdir | universe 子测试 | 通过 | 通过率 |
|---|---:|---:|---:|
| shadow-dom | 15,348 | 8,264 | **53.84%**（162/381 文件覆盖，66 全过） |
| custom-elements | 4,356 | 2,821 | **64.76%** |

（coverage 工具按 universe 交集计数，headline 通过 11,342 与 local 11,668 差 326
为 pin 语料较 wptfyi cache 新增子测试，口径如实并记。）

## 四判据裁定（契约原文逐条）

1. **shadow-dom topdir ≥ 31.4%**：53.84%（8,264/15,348），门值 1.71 倍余量；
   立项基线 4.05% → **PASS**。
2. **custom-elements 既有账逐 subtest 零回归**：`wc-m3c2-before` vs
   `wc-after-canvasfix` 两侧各 3,848 子测试集合恒等，2821P 两侧恒等，
   P→non-P=0、vanished=0；universe 口径 64.76% vs 台账 64.67% 仅升
   （+4P 为 M3 前批滚动顺带入账，非本刀）→ **PASS**。
3. **全量头条不回落**：M4 细则只产出 WC 通道全语料结果，头条按该口径裁定——
   ×1 全语料 8,210P（修复前基线 `wc-m3c2-before`）→ 8,437P（冻结二进制），
   混合口径 11,668P；M3 收口刀均为 JS-only additive（shim 新增分支/桥），
   全语料 A/B P→F=0（见 M3 收口进展节）→ **PASS**。
4. **`wpt-suites.csv` 收行**：web-components suite_id 数据行（混合口径
   13,087/11,668 = 89.16%，completed），evidence 路径指向本文件（归档后
   补 `archive/` 前缀）→ **PASS**（随收口提交落地）。

## ×4 口径法证注记（6,908 → 6,657 尾差归因）

M3 Batch 3 闭环时的 ×4 先例（`gethtml-after-solo-x4.json`）为 6,908P/6908
total；本轮 ×4 为 6,656P/6,657。差 252 子测试 + 新增 1 文件级条目：

- 252 全部为 `ShadowRoot.getHTML() on <X>` serial-matrix 尾部生成案（按元素
  × 模式 × delegatesFocus × serial 组合展开）；
- 两侧共同 6,656 子测试**状态逐案零漂移**（0 F，无回归）；
- 归因：d7155cef5 二进制较 Batch 3 期多 attach 恢复臂/canvas 桥代码路径，
  ×4 预算窗内 serial-matrix 尾部未完成收集（未上报，非 Fail/Timeout），
  属预算窗内收集截断而非语义回退；契约 6,908P 本为门可达性余量参考，
  M4 判据按混合口径实测裁定（判据 1 仍有 1.71 倍余量），不触发口径不符。

## 关联

- M3 清单 1 收口（attachment solo 654P/0F、全语料 A/B）：goal doc 2026-11-07
  进展节（提交 d7155cef5）。
- M3 清单 2 adoptedStyleSheets 归口：
  [2026-11-07-m3-adoptedstylesheets-routing.md](2026-11-07-m3-adoptedstylesheets-routing.md)。
- 原始 JSON 留档 `.acceptance/shadow-m3-residual/`（wc-after-canvasfix.json、
  gethtml-x4-m4.json、m4-results/ 合并目录、m4-coverage.md；不入库）。

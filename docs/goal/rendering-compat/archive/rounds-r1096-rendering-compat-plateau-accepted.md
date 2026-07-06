# R1096（2026-07-06）：rendering-compat 双 plateau 接受·+232 secure（opt-in）·forward 转其他目标

承 R1094（+232 实测）+ R1095（三位一体 feature-on 证伪）。本轮系统性深挖 font-wall 与结构性两条前线，确证**双 plateau**，接受现状，secure +232，rendering-compat 转成熟期。

## 双 plateau 实证

### font-wall 前线：+232 是天花板

- R1094 全 corpus A/B 实测 freetype-raster = **+232 oracle-pass（4759→4991，+2.23pp），零回归**（C-dep 价值从推测升级为实测）。
- R1095 三位一体（store-gate 放开 + paint 发 baseline + renderer 应用 bitmap.y_offset，feature-on + cfg-gated）= **css-text -42 net-negative**，broad 回归（line-breaking -20/white-space -7/...）→ **per-glyph metric 放置在 feature-on 也证伪**。
- 结论：font-wall 是精调启发式平衡（baseline/strut/half-leading/placement 四轴 R990/R953/R817/R841 共调谐），**单轴"几何更正确"必打破平衡**。第 9 次单点 net-negative（R834/R836/R849/R875/R1052/R1056/R1067/R1090 + R1095）。**+232 是 font-wall 的实际极限**。

### 结构性前线：fresh 实证确认 plateau

css-tables（41 失败，结构性，比 multicol 可控）新鲜 oracle 实证（不假设记忆）：top 失败**全部是先前已归类的 blocker**，无新鲜簇：

| 失败案 | z_vs_chr | 归类 |
|---|---|---|
| table-cell-width-0 | 20.09% | R97 intrinsic = taffy-blocked |
| baseline-vertical | 12.45% | baseline-export（R304 deferred）|
| table-row-group-color-inheritance-001 | 10.74% | table-fixup 结构性 |
| collapsed-border-vertical-lr/sideways/rtl ×3 | 4.73-4.80% | writing-modes（R109-blocked）|
| min-max-size-table-content-box | 4.27% | subtle sizing 边缘 |

结构性 lever 全景（master.md R1092 plateau + 本轮新鲜确认）：
- **multicol Phase 2**（嵌套/breaking 碎片化）：spec 在（column-aware-IFC-spec.md），Phase 1 已停（R381: 0/16 案匹配单层+balance+明确高度+纯 inline），真失败全需 Phase 2 多 session hardcore。
- **baseline-export**（flex+grid+multicol baseline）：R304 deferred，跨切最高杠杆但需 taffy 0.8+ 接入或自建 inline-box baseline 合成。
- **taffy 升级**（R304）：541 ref + 108 alignment + native float 冲突，全量迁移 deferred。
- **vertical-mode IFC**（writing-modes 725 失败）：R109 深系统（block-flow + inline-flow + line-height-vertical + emphasis），3 证单轴 net-negative。
- **css-position**（40 失败）：R1092 已证几何正确，diff 是 font-wall（非结构性）。

## +232 secure 状态

- `freetype-raster` feature LANDED（R1068），**default-off**——CI 纯 Rust 不变，main CI 三平台绿。
- 本地全验证（R1094）：`cargo build/test/clippy --features freetype-raster` 三关全绿（gcc 14.2 bundled FreeType2+libpng）；浏览器 CLI 直转零代码改动（`cargo run --features zero-render-foundation/freetype-raster --bin zero-browser`）。
- **opt-in 即可用**：任何 release/产品构建加 `--features freetype-raster`（或 `--features freetype-raster` 经 wpt-runner 顶层 feature）即得 +232。

## default-flip 阻塞（CI 工程子项，非 rendering）

- 本地 default-on（render-foundation `default = ["freetype-raster"]`）build/test/clippy 全绿，精确 CI 命令（`cargo check --target x86_64-unknown-linux-gnu`）本地通过。
- 但 `freetype-raster-cross-platform` workflow（6 target，复现 main CI 的 `--target` 用法）**两次 run 全 6 fail**（含 ubuntu-latest x86_64，本地却通过 = 环境问题）。
- **CI 日志取不到**（`gh run view --log` + API zip 均空/not-found）→ 无法从这边诊断。
- forward：需用户从 Actions UI（run 28774595109 / 28774102596）取日志定位 sysdep/缓存/工具链问题，修后翻 default。**非 rendering-compat lever，独立 CI 工程子项**。

## 裁决：接受 plateau

- font-wall（+232 极限）+ 结构性（fresh 确认 plateau）双前线 mature。
- **剩余 lever 全部多 session hardcore 或 blocked**，单 session 边际收益 ~0（plateau 定义）。
- +232 已 secure（opt-in），是本线索最后一个实质突破。
- **rendering-compat 转成熟期**：维持现状，forward 转向 ZeroWeb 其他更高杠杆目标（父目标 `docs/goal/zero-web.md`：JS/DOM 兼容、网络栈、安全/运行时、产品 UI 等）。rendering-compat 留 +232 + opt-in，default-flip 待 CI 解决。

## 勿再试（plateau 证据穷尽）

- font-wall 单点 metric 放置（R1095 第 9 证，feature-on 也负）
- font-wall 三方补偿常数（R876 谱系，R1067/R1090 net-neutral/negative）
- 结构性单 session fresh scan（R1087-R1093 + 本轮 css-tables，全归类）
- multicol Phase 1（R381，0/16，无目标案）

## 可立项目（若未来有 appetite，均多 session）

- baseline-export（跨 flex/grid/multicol baseline，~5-8 session）
- multicol Phase 2（spec 在，~5-10 session）
- taffy 0.8+ 升级（R304，~3-5 session，unblock 多个 taffy-blocked）
- vertical-mode IFC（R109 系统，~8+ session）

**门禁**：纯裁决 + 实证总结，零 net 源码（freetype-raster 仍 default-off，三位一体/default-flip 均已回退）。

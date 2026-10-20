# 编码兼容 — 运行时控制面板（master.md）

> **已归档（模式 A，2026-10-20）**：当前终态、历史验证与挂账保留；入口见 [归档目标](../encoding-compat.md)。

**入口文档**: [../encoding-compat.md](../encoding-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-19（三轮复跑——组合态 main 独立核对持平 99.98%，零源码改动验证轮）

## 当前状态

**目标 Completed（2026-10-02 M4 收口；2026-10-19 两轮复跑修齐）**：DC-1~4 全满足，
全语料 36 案 **12072/12075 = 99.98%**（轨迹 6.1% → 98.4% → 99.96% → 99.97% → 99.98%）。
门禁：make test 全绿 + clippy -D warnings + fmt 干净（本轮复验）。
判定文档：[evidence/2026-10-02-m4-dc-verdict.md](evidence/2026-10-02-m4-dc-verdict.md)；
复跑证据：[evidence/2026-10-19-revalidation-sab-regression.md](evidence/2026-10-19-revalidation-sab-regression.md)、
[evidence/2026-10-19-iso2022jp-clear-wasm-native-restore.md](evidence/2026-10-19-iso2022jp-clear-wasm-native-restore.md)。

**2026-10-19 复跑修齐（第一轮）**：收口后例行复跑揪出两根因——① runner 页面环境
WebAssembly.Memory 被桥接 polyfill 覆写（`b785548dc` 2026-10-17 runner 预热改走 with_dom
的副作用，SAB 面约 −80 案跨 goal 回归）→ dom_bridge 桥安装回挂原生 Memory（常驻单测
锁定）；② fetch 脚本未显式补拉 `/common/subset-tests.js`（资产不自足，分母 −10587）→
照 sab.js 先例显式补拉。残差 5 → 4（encodeInto detached ×1 被其他流引擎语义修齐顺势
清偿，110/111 → 111/111 跨 goal 正漂移）。

**2026-10-19 二轮复跑**：① iso-2022-jp fatal-stream 残差定性翻案——原记账「encoding_rs
上游契约」实为本方 shim 缺陷（stream 面 fatal 错误无条件重建 handle 丢转义态；spec 只在
do not flush 未置位时重置 decoder）→ shim 修复清偿，残差 4 → **3**（常驻单测锁定）；
② wasm/jsapi 通道核对（上轮留注记）实证塌缩 141/460——10-19「回挂即还原」判断对 jsapi
面不成立，桥整体替换原生 V8 WebAssembly（page-wasm 基线 715/719 系原生跑出）→ 桥安装
stash 原生引用（不可枚举 `__zwNative__` 锚点）+ runner 预热后整面回挂，通道 715/719
逐字还原、encoding 零回归（常驻单测锁定；生产 `__WASM_BRIDGE__` 桥语义不动）；③
encoding_rs 升级重评结账——上游 0.8.40+ 要求 rust 1.88（工作区 MSRV 1.85，升级须用户
拍板），且残差已清偿、升级不再绑定任何残差，维持挂账动机除名。

残差 3/12075 已分类定稿：detached-AB/SAB transfer 结构面 ×3（重入 = transfer/
structured-clone 管道落地）。文档级编码嗅探面已双向记账转 html-syntax-compat（其控制
面 P5）。

里程碑台账：M1（10-01）corpus 通道 + 基线 6.1%；M2（10-02）labels 全表 + legacy 解码
（encoding_rs host native + 有状态 decoder 表）98.4%；M3（10-02）utf-8 spec 状态机 +
TextEncoder/streams 语义 99.96%；复跑（10-19 一轮）回归修齐 99.97%；复跑（10-19 二轮）
iso-2022-jp 清偿 + wasm 通道还原 99.98%；复跑（10-19 三轮）组合态 main（渲染流 7 commit
之上）独立核对 12072/12075 逐字持平、3 Fail 与残差记账逐一吻合。逐轮证据见 evidence/
（m1-baseline / m2-labels-legacy-decode / m3-textencoder-modes-streams / m4-dc-verdict /
2026-10-19-revalidation-sab-regression / 2026-10-19-iso2022jp-clear-wasm-native-restore /
2026-10-19-revalidation-combined-main）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | encoding/ corpus 导入 + 基线 | ✅ M1（2026-10-01） |
| P2 | labels 标签匹配全表（数据化） | ✅ M2（api-invalid-label 3421/3421） |
| P3 | TextDecoder legacy 编码解码 | ✅ M2（legacy-mb 100%、single-byte 336/336、iso-2022-jp 34/34） |
| P4 | TextEncoder + BOM/fatal/ignore 模式 + 编码往返 | ✅ M3（10-19 二轮后 99.98%；残差 3 记账） |
| P5 | 文档级编码嗅探挂账定稿 | ✅ M4（转 html-syntax-compat P5，双向记账 2026-10-02） |

## 后续重入条件（记账，非待办）

- transfer/structured-clone 管道落地（workers/navigation 流域）→ 残差 3 案（detached-AB/
  SAB 面）可清偿。
- encoding_rs 升级（>0.8.35）维持挂账、动机除名——iso-2022-jp 残差已由 shim 修复清偿
  （10-19 二轮），且上游 0.8.40+ 要求 rust 1.88（工作区 MSRV 1.85，升级须用户拍板）；
  未来 MSRV ≥ 1.88 时顺带评估（streaming chunk-boundary 修复对本仓空块 guard
  workaround 或有化简空间，届时重审 text_encoding.rs 空块注记）。
- html-syntax-compat M1 落地 → 文档级嗅探面（bom-handling/eof-*/utf-32*/sniffing，
  wpt-data 已拉取）随其 corpus 一并基线。
- ~~wasm/jsapi 通道复跑~~ ✅ 10-19 二轮完成：塌缩 141/460 实证（桥整体替换原生）→
  修复后 715/719 逐字还原（锚点+回挂，常驻单测锁定；生产桥语义不动）。监控建议：后续
  goal 触碰 dom_bridge.rs wasm 段或 runner 预热序列时随变更跑 `make testharness-wasm`
  （语料已驻留，分钟级）。
- 数字维护：全量复跑 `make testharness-encoding` 后向 wpt-suites.csv 追加行。

**待用户决策清单**：（空）

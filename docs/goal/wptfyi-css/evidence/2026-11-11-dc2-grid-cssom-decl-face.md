# DC-2 grid 批次：CSSOM 声明面校验 + canonical 序列化 + supported property names

日期：2026-11-11 · 分支 main · 范围 css/css-grid 声明面（G1）

## 批次内容（两个提交）

1. **native 面**（17f30543c，先行落库）：`dom_bindings/grid_cssom.rs`——grid 家族
   14 属性 CSSOM 写面文法校验 + canonical 序列化（`<grid-line>`/grid-area 填充与
   尾部最小化/grid-auto-flow §3.5.3 简写归一/CSS 全局关键字放行）+ named query
   回调（native ObjectTemplate `in` 面）。css_style_declaration.rs setter 分派接线。
2. **shim 面**（本批）：corpus 语料跑 shim JS 世界，element style 是
   `_styleProxy`（sel/handle 面）/_zwMStyleProxy（物化 plain 面）两类 Proxy——
   声明写归一接宿主回调 `__zw_grid_normalize_decl`（复用 grid_cssom 单实现双通
   道），`in` 门接共享 has 臂 `_zwStyleHas`（`_zwStyleProxy`/`_zwMStyleProxy` 两
   面板共用；宿主清单 `__zw_style_supported_prop` = grid_cssom
   SUPPORTED_STYLE_PROPS 核心簇）。

## 探针考古（接线三连 bug，探针页 css/css-grid/zz-probe-grid.html 实证）

shim 面接线后探针行为纹丝不动，逐层法证（证据
`.acceptance/wptfyi-css/grid-batch/probe-archaeology.md`）：

1. **注册名/消费名不一致**：`register_callback` 名参逐字落 globalThis（snake_case
   `__zw_grid_normalize_decl`），shim 消费端写成 camelCase——typeof 守卫恒 false
   全落兜底 raw。直接调用诊断（host-callback-direct 案）实证宿主面挂载正常后定位。
2. **宿主回调返 JS 字符串**：shim 通用约定（part01c `=== 'false'` 先例），首版
   消费端用 `=== null`/`=== true` 布尔比较——拒绝值被当合法值存储、has 臂恒
   false。改 `=== 'null'`/`=== 'true'` 字面量。
3. **活跃面板误判**：has trap 补在 `_zwMStyleProxy`，探针身份诊断
   （`__zwRawDecl` 成员存在）实证 corpus 世界 element style 是 `_styleProxy`
   （sel/handle 面）——该面板本无 has trap。抽共享臂 `_zwStyleHas` 两面板接线，
   并放行 Object.prototype 基础成员与索引名（Proxy has trap 返 false 即「不存在」，
   不放行会误杀 `'toString' in style` 原型链真值面）。

## 数字

- 探针页：**20P/0F**（probe-final-20P.log）——声明面写归一（`row dense`→`dense`、
  garbage/双 ident 拒且旧值存活）、`in` 全绿（camel/kebab/color/gridArea）、
  宿主回调直接调用 7 断言。
- 锚页 grid-layout-properties.html：**13P → 94P（+81）**（anchor-
  grid-layout-properties-94P.log）——`in` 门（assert_true(prop in style)）全解锁。
- css-grid 分片基线 653 cases/1251P/6389F 不降核对待全量重跑（共享 release
  二进制电池结束后执行；本批跑单页锚 + 探针，分片重跑归后续轮次）。

## 残余（锚页 46F 分桶）

- **computed 值发射臂缺失**（~33）：`grid-row-start: '1 a'` 声明面写成功但
  getComputedStyle 返 initial 'auto'——grid-line/grid-area 族 computed 发射臂
  （cb967820e transforms 同型工作）。
- **stylesheet cascade 应用缺失**（~8）：`reset`/`initial value` 组——inline 移除
  后应回落页面样式表规则值（150px），现返 'none'——grid 属性进 cascade/apply 面。
- **span 形状序列化**（~5）：`span <custom-ident>` 组 computed 形状核对。

## 验证

- `cargo test -p zero-engine --lib`（隔离 target）：**3833 passed / 7 failed**，
  失败名单与上批证据（dc2-color-layers-alpha-decl.md）完全同名单（img/fetch/
  iframe/websocket 既有失败，与本批无关）；本批 pin（grid-auto-flow `dense`）与
  T54 共存全过。
- `cargo clippy -p zero-engine --all-targets` exit 0；`cargo fmt` 净。
- 提交门禁 lei-pre-commit-guard PASS（staged 6 文件 119 行）。

## 协作记录

实例 D 同窗提交 6e98ca835（T54 inline style 写面校验）叠于 Commit A 之上，
其 `__zw_validate_decl_value` 臂与 grid 归一臂正交互补（T54 表外放行 → grid 臂
接管拒/归一）；其 multitest 电池运行期间本批走隔离 target 构建
（/home/lei/work/zw-g1-target）零共享产物触碰。

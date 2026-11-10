# DC-2 · test262 export default 具名函数声明簇修复（2026-11-10）

接[三簇定性](dc2-test262-cluster-triage-2026-11-10.md)后的剩余簇归簇：
"Assignment to constant variable" 19 案疑点（上轮卡点清单首项）——**可修**，
根因在本仓 ESM 转换层，runner commit `e3b0caa2b`。

## 根因（engine 侧，非 V8 差异）

- 案形：`export default function fn() { fn = 2; return 1; }`（fixture）——
  spec 导出默认函数声明本地绑定与普通函数声明同为**可变** + default 活导出
  （#prod-ExportDeclaration）；Chrome 156 实测可赋值（`r1=1, default=2`）。
- 本仓 `script-sandbox/src/es_module.rs` `transform_export` 走 expr 分支发射
  `_exports.default = function fn(){…}`——函数表达式自带不可变自名绑定，体内
  `fn = 2` 报 "Assignment to constant variable"，导入侧活绑定恒断言失败。
- 上轮误判过一次方向：先怀疑 V8 版本差（v8 crate 150.2.0 = V8 15.0.245.2 vs
  Chrome 15.6），Chrome 复测推翻——与 eval-code 簇（真 V8 上游差异）不同，
  本簇是本仓转换层缺口。

## 变更与结果

- `default_function_decl_binding_name`：具名声明形态提取
  （`[async] function[*] Name(`；`functionX`/`asyncXxx` 表达式与匿名形态返回
  None）；`transform_export` 声明形态前置于 expr 分支发裸声明（hoisted）；
  `local_export_bindings` 收集 `(default, 本地名)` 别名对、体首活 getter 接线
  （入口与依赖内联同走 `build_module_script`，双路径覆盖）；匿名形态保持
  expr 分支（赋值目标 name inference 恰为 "default"）。
- 真语料 test262 通道对拍 DC-1 基线（release runner，test-guard 包裹）：
  - `language/module-code` 596 案：309 → **318 Pass（+9）**，0 回归
    （Timeout 3 案基线一致，TLA 竞态面）；
  - `language/expressions/dynamic-import` 941 案：696 → **714 Pass（+18）**，
    0 回归；
  - `eval-gtbndng-indirect-update-dflt` 家族 19 案（18 usage + 1 module）+
    export-default 具名声明 8 案（declaration-binding / asyncfunction /
    generator / eval-export-dflt-*-semi）= **+27**。
- 单测 3 新增（具名可变绑定+活导出 / 匿名回归面 / async+generator 形态）；
  script-sandbox lib 433 全绿；clippy `-D warnings` 干净；rebase 组合态复验
  全绿（远端并行流 `4d3bae756` 改 engine/js_dom_shim，文件不重叠）。

## 对目标净增的账

~75,032 + **27** ≈ **75,059 / 76,000**，余 ~941。

## 待办

- test262 剩余簇归簇（module early-error 132、SAB clone 106、ShadowRealm 57、
  expressions 负例 76、递归深度 34——逐簇定性可修 vs 引擎差异）
- encoding 残余可回收项（iframe 面 ~330 收益评估）
- DC-3 全量对账（同 revision 归一、验收线 1,393,955、邻域无回退）

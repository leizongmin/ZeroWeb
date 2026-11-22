---
date: 2026-11-23
modules: engine,script-sandbox
---

# engine 默认 v8 主通道：QuickJS-only 宿主桥改动在主回归通道静默失效

## 问题

刀③③b（Function 构造器 TT 编译门）首版把宿主跳板工厂
`__zwTtMakeCompileTrampoline` 只落在 quickjs_runtime.rs，enforced 探针页首轮
6F：工厂 `typeof === 'function'` 断言失败、依赖工厂的 4 个构造器臂全红、
2 个 string timer 不触发——所有「装门后应绿」的断言齐红，形似引擎缺能力。

## 根因

`crates/engine` Cargo.toml `default = ["v8"]`；`make testharness-*`、
`zero-wpt-runner` bin、zero-engine lib 测试**全部走 V8Sandbox**。QuickJS
（`--features quickjs`）只是矩阵支路（QUICKJS_TEST_CRATES 的 lib 测试）。
QuickJS-only 的 shim 宿主桥新增全局在主通道根本不存在——探针在主通道跑，
改动零可见，失败面却 100% 复现「功能缺失」症状，极易误判为引擎实现 bug。

## 解决

宿主桥臂（script-sandbox 新增宿主全局/回调类）必须双后端落点：v8_runtime.rs
（FunctionTemplate 工厂 + execute 无条件安装）与 quickjs_runtime.rs
（`Function::new` + `with_constructor`，install_callbacks 同批重挂）各一；
或显式划界注记「仅 QuickJS」并把验收限制在 `--features quickjs` 通道。

## 避免方式

- 写 shim 宿主函数前先确认通道：`grep 'default =' crates/engine/Cargo.toml`。
- shim 宿主桥刀的验收顺序固定：探针（主通道）→ 目标页 → lib 基线 → san
  恒等 → 全量 comm 对账；探针首轮「安装断言失败」先查通道再查实现。
- 另一面同刀教训：门缺席时部分语料断言「空洞通过」（基线 P 是政策从未被
  调用），装门后这些断言才真正执行——基线绿不代表机制对，ground truth 要
  读语料断言本体。

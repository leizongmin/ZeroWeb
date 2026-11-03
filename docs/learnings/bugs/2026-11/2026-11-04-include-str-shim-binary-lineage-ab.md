---
date: 2026-11-04
modules: engine, testing
---

# include_str! 内嵌 shim 的 A/B 判读必须先做二进制谱系验证

## 问题

编辑兼容 spin2 窗（gate28）多轮「守卫版/干净版」对比判读建立在错误的二进制
谱系上：diag1 20:36 插桩构建后，diag2/diag3/battery1/battery2-delfd 五次运行
全部 cargo up-to-date（0.2s Finished）复用同一插桩二进制，被当成「干净对照」；
battery2 电池中途一次 make 腿触发 36.48s 重建（shim 守卫 + 捎带 rust 变更），
同一电池前后两腿分属两代二进制。基于此的「diag3 守卫版定谳」「插桩版永不
完页」等结论全部失效，已勘误（evidence 2026-11-03-spin2-residual-hang-m670-guard.md
§1/§3、COORD 卅七/四十二轮）。

## 根因

crates/engine/src/js_dom_bridge.rs 用 `include_str!("js_dom_shim/*.js")` 把
shim 编译期内嵌进二进制：改 .js 文件不触发重建时，磁盘上每个二进制都继续
携带旧 shim 内容；cargo up-to-date 检查不产生任何警告；直接 exec 二进制
（不经 make/cargo）的运行冻结磁盘当时的代际。行级结果账本不会记录「这次跑
的是哪代 shim」，引用方默认「刚提交的源码 = 刚跑的二进制」，于是插桩、
守卫、干净三态在账本里被混为一谈。共享树多流并行会放大：一条流的重建会
静默换掉另一条流下一腿拿到的二进制。

## 解决与避免

- 判读任何 shim 相关 A/B 前先重建谱系：各运行日志 `grep -c '^Compiling'`
  + 日志 mtime 链定位每次重建点；0.2s up-to-date Finished = 未重建 = 沿用
  上代二进制。
- 用内容指纹直接验证：`strings -a target/release/zero-wpt-runner | grep -c
  <插桩/守卫 token>`，比对工作树 token 计数是否一致。
- include_str! 资产编辑后，重建成功的判据是日志里出现含对应 crate 名的
  Compiling 行，不是 Finished；脚本化重建必须断言 Compiling 行。
- 长电池/长探针期间冻结 crates 与 wpt-data 修改（共享树窗口纪律）；A/B
  只在同一指纹代际的运行之间做，跨代数据先做谱系映射。

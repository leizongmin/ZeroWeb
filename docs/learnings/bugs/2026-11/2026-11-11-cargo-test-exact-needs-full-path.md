---
date: 2026-11-11
modules: engine,testing
---

# cargo test --exact 须完整测试路径：裸名/短前缀全 filtered 0P 假绿 EXIT=0

## 问题描述

T58 集成钉链 attempt1（2026-11-11）：照抄 t57-mg 脚本形态写
`cargo test -p zero-engine --lib -- --exact navigator_pdf_viewer_enabled_t58`，
EXIT=0「全绿」——实际 `0 passed; 0 failed; 3844 filtered out`：名字没匹配任何
测试，libtest 对空匹配集照样退出 0。归因构建器的「22 passed」断言拦下该腿，
否则集成回执会以假绿入账。T60 实施期同坑再踩一次：`r5296_single_line_...`
裸测试名 + `--exact` → `0 passed; 3846 filtered out` EXIT=0 假绿；另有一次
`--exact` 误放 `--` 之前被 cargo 当未知参数拒绝（`error: unexpected argument
'--exact' found`）——两坑相继出现，均被「passed 数显式核对」纪律当场暴露。

## 根因分析

`--exact` 是 libtest 的**逐名全等匹配**过滤器：过滤器串必须与测试的完整
模块路径逐字符相等（`paint::tests::r5296_inline_bg_ml_gate_tests::r5296_...`）。
裸函数名、裸 `js_dom_bridge::tests::` 前缀尾、少一级模块，都是合法过滤器串
但匹配集为空；libtest 对「0 匹配」不报错（设计上允许按过滤跑子集），进程
退出码恒 0。于是三类失败形态全部洗白成「绿」：测试名错、模块路径层级错、
甚至测试文件被 mod 声明漏挂（编译器不报、过滤器照样 0 匹配）。cargo 参数
层还有第二坑：`--exact` 是 harness 参数，必须落在 `--` 之后，写在之前会被
cargo 本体拒绝（该形态反而 EXIT≠0，是显性失败）。

## 解决方案

1. **过滤器一律写完整模块路径**：`cargo test -p <pkg> --lib -- --exact
   <mod::path>::<test_name>`；路径从 `mod` 声明链逐级拼（文件在
   `src/paint/tests/` ⇒ 前缀 `paint::tests::` + 文件模块名）。
2. **passed 数显式核对是唯一判读标准**：`test result: ok. N passed` 的 N
   必须等于期望用例数且 >0，`filtered out` 数 ≠ 测试总数是 0P 假绿的直接
   签名（`0 passed; 3844 filtered out` 一眼可辨）；EXIT=0 单独不构成证据
   （T58 金标准：归因构建器写死 `N passed` 断言，N 不符即 fail）。
3. **脚本誊抄先验证再复用**：t57-mg 裸名形态本身就是错的（在盘脚本形态
   与其 18P log 不一致属转录回退），跨任务复用命令模板前先小样本实测
   passed>0，不继承上游脚本的过滤器形态。

## 如何避免

- 门禁脚本里每条 `--exact` 腿的判读式一律 `passed==N && N>0`，不写
  `EXIT==0` 单条件。
- 写测试腿命令时先跑一次裸路径核对 `running N tests` 头行与期望数一致，
  再纳入常驻脚本。
- 相关：`docs/learnings/bugs/2026-11/2026-11-08-hand-rolled-gate-legs-miss-standalone-binary-build.md`
  （手搓门禁腿缺前置构建——同一「假绿洗白」家族的构建侧形态）。

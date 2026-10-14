# WPT、testharness、reftest 与资产账本

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 `zero-wpt-runner` 的用例发现、资源恢复、比较语义和结果口径；修复兼容性时先确认用例实际走哪条 runner。页面算法见 page-core，产品实跑入口见 [项目验证](../../../../zeroweb-verify/SKILL.md)；这里未运行任何测试。

## Gotchas
- `--gpu` 目前仍是 CPU 回退桩，不调用 GpuRenderer；这个选项产生的 reftest 成绩不能证明生产 GPU 路径正确。 (`tests/wpt-runner/src/main.rs`)
- 上游文件加载器把 `about:blank` 参考转换为空 HTML，不访问磁盘；其他参考全部读取失败时整个 case 不进入执行集合，比较通过率前须核对加载警告与分母。 (`tests/wpt-runner/src/wpt_file_loader.rs`)
- `ZERO_REFTEST_STRICT` 按变量是否存在启用，而非解析值是否为 `1`；严格布局阈值为差异率 0.1%/通道差 2，文字为 0.5%/5，默认分类阈值明显更宽。 (`tests/wpt-runner/src/reftest.rs`)
- 同源 test/ref 都渲染为空可能得到假绿；结果中的 `test_near_solid` 是退化诊断，不能只读 `passed`。 (`tests/wpt-runner/src/reftest.rs`)
- 导航 HTML 的 variant-meta 会展开为带 query 的独立 case；历史上 Pass 175→190 时比例反而从 76.1%→74.5%，原因是分母扩张，不是既有用例回归。 (`tests/wpt-runner/src/testharness.rs`, git:8fcff6957b9d96d307c64d68cce24a185b0d2bf3)
- 三个本地 HTML/send_keys 钉测的 30s 是并行负载容限，不是产品交互延迟断言；修复只调整案例超时，没有放宽 JS 语义断言。 (`tests/wpt-runner/src/testharness.rs`, git:6e79393b69accc141c0956fa962365b595510cee)
- 当前 touch-events 通道只跑构造器/属性 HTML 面，排除 manual/crashtest 及含 test_driver/testdriver.js 的宿主注入面；goals README 的“已收口”仍保留宿主注入重入条件，不等于真实多点触摸全支持。 (`tests/wpt-runner/src/main.rs`, `tests/wpt-runner/src/testharness.rs`, `tests/wpt-runner/scripts/goals/README.md`, git:72c1560224cb2ddb47bfff1c521fbb677c8328d1, git:522cf8865dda9ef60a87c6d22bd304e41e2ae0a9)
- 导入器仍使用 live 上游 master，下载 ref 失败会删 test/ref 目标；已存在但不在 pinned 快照的本地资产可能只有一份，导入前须保全，不能以“wpt-data 可重拉”断言失败清理无损。 (`tests/wpt-runner/scripts/import-wpt-reftests.sh`, `.agents/skills/zeroweb-guidelines/SKILL.md`)

## Architecture
- 每个文件保留一个 case 和全部参考；match 要至少一张匹配，mismatch 要全部不匹配，不能把每条参考展开后按独立案例算通过率。 (`tests/wpt-runner/src/wpt_file_loader.rs`)
- 文件发现路径从 HTML fuzzy meta 取区间上界，再交给 `with_fuzzy_override`；MANIFEST 解析是另一来源，修改容差不能只改其中一条。 (`tests/wpt-runner/src/wpt_file_loader.rs`, `tests/wpt-runner/src/manifest.rs`)
- `imported-tests.txt` 保存 test/ref/relation/日期/修复备注，下载本体与生成 manifest 在外部 wpt-data；资源另由 `imported-resources.txt` 与同步脚本恢复，账本存在不等于本机资产齐全。 (`tests/wpt-runner/imported-tests.txt`, `tests/wpt-runner/scripts/import-wpt-reftests.sh`, `tests/wpt-runner/scripts/sync-imported-resources.sh`)
- testharness 的通用 corpus 预算缺省 30s，可由 `ZW_CORPUS_CASE_TIMEOUT_SECS` 覆盖；短 HTML 交互集合另有 10s 常量，不存在全 runner 单一超时。 (`tests/wpt-runner/src/testharness.rs`)

## Conventions
- 专项增加通常同时接通 main 子命令、testharness corpus/帮助资源及 Makefile 拉取入口；三者共同变更至少六次，遗漏接线会留下能编译但不可由目标命令执行的用例。 (`tests/wpt-runner/src/main.rs`, `tests/wpt-runner/src/testharness.rs`, `Makefile`, git:56f1a1177e32c3a056d7258c0134852d6f52029f, git:c92764ef55cf919349f2e3973b539945fe7e72b3, git:3b5abe108e51be57e0b99353f944847d92810b76)
- 本地等价 reftest 在内联定义、可恢复 HTML 孪生文件和常驻导入账本之间保持对应；只有临时下载目录里的案例不能承担 HEAD 常驻回归责任。 (`tests/wpt-runner/src/reftest_data/css_box.rs`, `tests/wpt-runner/imported-tests.txt`, git:18a15954cbf532c524a3914be0af8cd04122c40b, git:9bb12f84c47a0223955e899bf8ef53d9a56f5b0a, git:daac751261eaa08fd71403db954fac7d3148dbb7)

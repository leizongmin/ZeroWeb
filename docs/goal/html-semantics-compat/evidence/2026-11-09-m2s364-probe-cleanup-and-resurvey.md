# M2 切片 364：自建诊断探针清理 + 残红重勘（143/052/TLA 挂起）

**日期**: 2026-11-09
**目标对**: probe12/13 自建探针处置（s361 起在册「自建单列」项）+ 143/052 SVG 族重勘 + TLA 入口体挂起语义勘根
**结果**: probe12/13 及孤儿资源删除，语料口径 the-script-element **4F→2F**（残 F=143/052 长挂）；143/052 与 TLA 挂起语义重勘结论落账（维持挂账，理由钉死）。

## 勘根与处置

1. **probe12/13 定性**：恒红 dump 探针（`assert_array_equals(x, ["__FAIL__"])` 恒假带现场拼接），历史轮次诊断 defer 慢模块时序 / module async src-fast 执行序的临时物——诊断使命已毕（其观察面由 s352 写流嵌套链插入点序等切片消化），保留只污染域 F 口径。文件被 gitignore 规则忽略从未入库（`git ls-files` 零命中）——纯本地清理，upgrade 不回归。
2. **孤儿资源**：probe12-slow.js / probe12-fast.js 仅被 probe12 引用 → 同删；module/resources/{slow,fast}-module.js 有正当案共用（delay-load-event / inline-async-execorder / slow-cycle 等）→ 保留。
3. **probe8/9/10 系列定性（不删）**：行为钉非恒红（断言具体时序序列，当前 Pass）——本地回归保护保留；文件名 probe 命中 gitignore 未入库（跨 clone 不生效，转正与否挂账不急）。
4. **143 重勘落账**：solo 红态 `got ["inline script #1", "inline script #2"]` 缺 #3——SVG foreign content script 嵌套解析模型（内层 `<script>` 执行单元切分 + document.write 注入 `</script></script>` 双闭合的 tokenizer 时序）需要 parser 流式物化/调度架构支撑，非局部修复面，维持长挂（s106 归族结论复核确认）。
5. **052 重勘落账**：solo 红态 `got "script tags in DOM: 5" expected 4`——外链脚本执行时 inline #2 已超前物化进 DOM——外链脚本阻塞解析点推进（流式物化）缺失，同族大工程，维持长挂。
6. **TLA 入口体延迟属性访问挂起语义**：语料三案（basic/tla/cyclic）无钉面——tla 案入口自身无 TLA（仅依赖 TLA，已 Pass）；「runBody 挂起中属性访问返回值」在 tc39 proposal-defer-import-eval 属 issue #83 开放语义域（带 TLA 模块 defer 行为提案演进中）——语义决策未定 + 无钉面，维持挂账不施工。

## 验证

- **域 join**（A=s363-b 1129 键 vs B=s364-b 1127 键）：A-only 恰 = probe12/13 两键（删除目标）、B-only ∅、STATE-DIFF=0——零意外位移，**4F→2F、5T 不变**（残 F=143/052；T 名单与 s359 起同构）。
- runner 输出 1128 行 vs 1127 pair：1 行偶发重复（(key,state) sort -u 归一）。
- make test 不适用：本轮零代码改动（删件 gitignore 未入库；主仓改动仅文档/脚本挂账行）。

## 挂账 / 下一步

- probe8/9/10 转正决策（改名入 git 跟踪以获得跨 clone 回归保护——低优先，本地钉在位）。
- 143/052 流式物化架构改造（parser/script 调度，大工程，需切片分解）。
- SW module 静态 defer 求值语义；142 xlink:href write 注入面；T=5 长尾。

## 日志

- evidence/logs/s364-b-script-element.log（域跑 B 面，join 基线 s363-b-script-element.log）

# DC-2 语料扩面：13 缺语料目录接入 corpus 执行链（接线 + 基线）

**2026-11-10**。DC-1 缺语料桶 13 目录（css-transitions / css-animations /
cssom-view / css-view-transitions / geometry / css-properties-values-api /
css-highlight-api / css-page / css-scroll-anchoring / css-scrollbars /
css-overscroll-behavior / css-navigation / css-parser-api）一次接线：
fetch（41 号脚本 DIRS 扩面，pin 315976933）→ runner 语料通道（13 wrapper +
13 dispatch 臂 + 13 CLI 子命令）→ 分片（css-v2-face-d）。基线记录轮 JSON
落 `.acceptance/wptfyi-css/results-v3/`。

## 语料 sync 账面（逐目录 missing=0 对账，pin 树清单逐文件 + blob sha 复核）

```
css/css-transitions            上游 211   新拉 122  重拉 0   missing 0
css/css-animations             上游 291   新拉 0    重拉 0   missing 0
css/cssom-view                 上游 264   新拉 0    重拉 0   missing 0
css/css-view-transitions       上游 734   新拉 454  重拉 0   missing 0
css/geometry                   上游 30    新拉 30   重拉 0   missing 0
css/css-properties-values-api  上游 133   新拉 133  重拉 0   missing 0
css/css-highlight-api          上游 191   新拉 191  重拉 0   missing 0
css/css-page                   上游 464   新拉 464  重拉 0   missing 0
css/css-scroll-anchoring       上游 84    新拉 84   重拉 0   missing 0
css/css-scrollbars             上游 109   新拉 109  重拉 0   missing 0
css/css-overscroll-behavior    上游 13    新拉 13   重拉 0   missing 0
css/css-navigation             上游 2     新拉 2    重拉 0   missing 0
css/css-parser-api             上游 1     新拉 1    重拉 0   missing 0
css/support                    上游 20    新拉 0    重拉 0   missing 0
```

首轮 sync css/support 在 trees 递归清单步被网络截断（JSONDecodeError，
幂等续拉先例）；续跑 1 目录对账确认 20/20 已存在且 blob sha 与 pin 一致
（M0 批存量已覆盖）。

## 接线改动面（模板零改动复用，6 文件）

| 文件 | 内容 |
|---|---|
| `tests/wpt-runner/src/testharness_css_corpus.rs` | 13 个 `run_css_*_cases` wrapper（`run_css_corpus_dir` 逐目录委托）；cssom-view/geometry 无 `css-` 前缀，子命令映射走脚本显式臂 |
| `tests/wpt-runner/src/main.rs` | 帮助文本 + dispatch match 13 臂 + 13 个 `cmd_testharness_css_*`（共享 doc 注释） |
| `tests/wpt-runner/scripts/run-css-testharness-corpus.sh` | SUBCMD 显式映射 `cssom-view`/`geometry` 两臂（其余走 `css-*` → `testharness-<corpus>` 通用规则） |
| `Makefile` | `fetch-wpt-css-testharness-v3`（CSS_FETCH_ONLY 14 目录白名单 + GITHUB_TOKEN 由调用方 export）+ 13 个 `testharness-css-*` 目标（fetch 依赖 + test-guard 4G/8G/3600s） |
| `tests/wpt-runner/ci-shards.yml` | 新分片 `css-v2-face-d`（13 suites、fetch target、job_timeout 210；CI 超预算按 form-encode 先例拆） |
| `tests/wpt-runner/scripts/goals/41-css-testharness-compat-v2.sh` | DIRS 追加 13 目录（css/support 已在列） |

## 基线记录轮（runner 二进制同批一致，dom :not 批之前）

| corpus | cases | 子测试 | Pass | Fail | Timeout |
|---|---|---|---|---|---|
| css-animations | 123 | 963 | 225 | 731 | 7 |
| css-highlight-api | 16 | 104 | 11 | 93 | 0 |
| css-navigation | 1 | 1 | 0 | 0 | 1 |
| css-overscroll-behavior | 10 | 95 | 24 | 71 | 0 |
| css-page | 23 | 112 | 8 | 104 | 0 |
| css-parser-api | 1 | 1 | 0 | 1 | 0 |
| css-properties-values-api | 108 | 1149 | 107 | 1041 | 1 |
| css-scroll-anchoring | 70 | 90 | 15 | 62 | 13 |
| css-scrollbars | 25 | 83 | 33 | 50 | 0 |
| css-transitions | 118 | 1936 | 185 | 1728 | 23 |
| css-view-transitions | 142 | 1225 | 19 | 1165 | 41 |
| cssom-view | 207 | 1976 | 545 | 1429 | 2 |
| geometry | 26 | 827 | 209 | 618 | 0 |
| **合计（13 目录）** | — | **8562** | **1381** | **7093** | **88** |

口径说明：cases = 语料枚举后实际执行页数（skip 规则过滤后）；子测试数为
runner JSON 汇总。布局几何断言面（geometry / cssom-view 部分页）通过率
结构性偏低，归口布局流不属本 goal 分母修复面；动画/接口断言面的失败分桶
见下。

## 失败分桶（按 corpus）

残余 7,093F 按 top 失败消息实证归类（全为引擎能力缺口，非通道缺陷）：

| corpus | 主失败桶（条数级） | 归属面 |
|---|---|---|
| css-transitions | 插值前后值不变 687；TransitionEnd 未触发 39；Animation/effect undefined 66 | 过渡运行时（计算值插值 + transition 事件 + getAnimations） |
| css-animations | Animation/effect undefined 151；事件计数 0 39 | Web Animations API（CSSAnimation 对象 + animation 事件） |
| cssom-view | scrollHeight/Width 溢出几何 270+；scrollingElement.scroll 缺失 80 | 布局 overflow 几何（布局流域）+ CSSOM View 滚动 API |
| css-view-transitions | `document.startViewTransition` undefined 全域（assert_implements 58 + TypeError 26 等） | View Transitions API 未实现（整目录低账为预期） |
| css-properties-values-api | `CSS.registerProperty is not a function` 627+117+50 | @property / 注册属性 API 未实现 |
| geometry | DOMMatrix/DOMPoint/DOMRect 断言 618 | 几何数学面部分已过（209P），布局依赖面归布局流 |
| 其余小面 | highlight-api 93F（Highlight registry）、scroll-anchoring 62F+13T、overscroll 71F、page 104F、scrollbars 50F | 各自专项 API/布局面 |

Timeout 88 项集中在动画等待环（transitions 23 / view-transitions 41 /
scroll-anchoring 13），CASE_WALL_S=90 外部墙钟兜底，不挂链。

## 验证

- 语料 sync：41 号脚本逐目录「上游 N 新拉 M 重拉 K missing 0」，blob sha
  与 pin 一致（sync_hash_report 内联复核）。
- `cargo fmt --all -- --check` 清洁；`cargo clippy -p zero-wpt-runner
  --all-targets` exit 0（接线批编译面）。
- 13 corpus 全量记录轮经 test-guard（--per-proc-mem 4 --total-mem 8
  --time-limit 3600）+ systemd scope（MemoryMax 24G / RuntimeMaxSec
  7200-14400）执行，逐案独立进程 + CASE_WALL_S=90 外部墙钟。

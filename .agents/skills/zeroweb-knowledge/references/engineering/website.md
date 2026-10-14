# 官网页面、进度数据与发布链

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖静态首页、数据页与更新页；数据链是已提交 CSV→metrics generator→metrics.json→progress.js，周报另经 updates.json 供首页/归档页消费。本轮未生成数据、部署或浏览页面。

## Gotchas
- Pages 只由 main 上特定路径的 push 或手动触发；改月报、一般 README 或 Rust 不自动部署，必须核对实际 paths 条件而非假定所有 main 更新都会刷新官网。 (`.github/workflows/pages.yml`)
- metrics 性能读取仍精确限定 `github-ubuntu-latest`，而 bench-report 当前 CI platform_class 会附 CPU slug；新 CPU 后缀数据可能被排除，页面“同类 Runner”文案不能证明当前硬件趋势全被展示。 (`scripts/generate-website-metrics.py`, `scripts/bench-report.sh`, `website/progress.js`)
- WPT 月度序列只取 upstream 自一致性，按 CSV 行序覆盖每月值，不混 oracle；不能把曲线当 Chrome/Firefox 全量 wpt.fyi 兼容率。 (`scripts/generate-website-metrics.py`, `website/progress.js`)
- suites 15 列严格校验 status/harness/unit、通过数和比例；planned 必须无数字，active/completed 必须有日期与数字，不能用零值冒充未测。 (`scripts/generate-website-metrics.py`)
- 首页最新三条直接取 updates.json 前三项，不自动按日期排序；添加周报须维持数据顺序，否则新周报可能不显示在首页。 (`website/script.js`, `website/updates.json`)
- 月报生成默认已有文件即跳过以保留人工决策章节，force 才覆盖；GNU date 不可用时默认月份回退当前月，不保证 macOS 自动选上月。 (`scripts/generate-monthly-report.sh`)

## Architecture
- 同 suite_id 按日期优先、同日行序靠后取终态，随后按 active/completed/planned、比例降序、ID 排序；官网展示不是 CSV 的原始行顺序。 (`scripts/generate-website-metrics.py`)
- `latest_data_date` 是 WPT、性能和套件所有日期的最大值，不是每个图的刷新日期；某一套件更新即可使页头日期变新，性能曲线仍可能陈旧。 (`scripts/generate-website-metrics.py`)
- 三个页面共享 localStorage 的 zeroweb-language 与 navigator 语言回落；语言切换重新绘制已加载数据，数据页还同步 aria-pressed 和 document lang。 (`website/script.js`, `website/progress.js`, `website/updates.js`)
- suites 的名字/目录/注记经过 esc 后进入 HTML，updates 的版本控制双语文案直接构造 innerHTML；这条链信任仓库编辑数据，不应直接接未净化的外部提交内容。 (`website/progress.js`, `website/updates.js`)

## Patterns
- 数据页按 tests/subtests/files 显示各套件分母，planned 行显示空值；小屏隐藏范围/证据/日期列而保留通过数、比例与状态，不应把隐藏列误认为生成器缺数据。 (`website/progress.js`, `website/progress.css`)
- Pages 先生成 metrics 再上传整个 website artifact；本机是否有 metrics.json 不能证明部署数据新鲜，更新日志与指标分别来自不同来源链。 (`.github/workflows/pages.yml`, `scripts/generate-website-metrics.py`, `website/updates.json`)

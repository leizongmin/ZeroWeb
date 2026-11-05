# M2 批次 3 接线 + 干跑基线（2026-11-05）

## 1. 接线范围（四件套模板 × 7 目录）

B2 规则/条件/级联簇 7 目录入常驻通道，模板与 M1 批次 1-2 一致：

| 件 | 文件 | 内容 |
|---|---|---|
| 薄包装 | `tests/wpt-runner/src/testharness_css_corpus.rs` | 每目录 `run_<dir>_cases`（共享底座），批次 3 注释块 + 中文 `///` 文档（含 M0 分母） |
| 分派 | `tests/wpt-runner/src/main.rs` | 7 match arm + 7 cmd fn + help 7 行 |
| Makefile | `Makefile` | `.PHONY` 14 名 + 7 全量 target + 7 `-one` 定向 target |
| 分片脚本 | `tests/wpt-runner/scripts/run-css-testharness-corpus.sh` | 显式映射臂 +2：`mediaqueries`、`CSS2`（上游目录名无 `css-` 前缀；CSS2 子命令小写 `testharness-css2`，motion/cssom/fill-stroke 先例；`css-*` 前缀 5 目录走通用规则零改动） |

CSS2 特例：只含 html 面（runner 只收 `.html/.htm`；`.xht` XML 面不在
通道，M0 形态注记延续），数值门按 html 面折算 300。

验证：`cargo fmt --check` 干净；`cargo build --release -p zero-wpt-runner`
通过；7 target 实跑出汇总（见 §2）。

## 2. 干跑基线（TIME_LIMIT=540，7 目录全部完整跑完）

| 目录 | cases | Pass | Fail | Timeout | 通过率 | M0 分母 | 跑到的子测试 |
|---|---:|---:|---:|---:|---:|---:|---:|
| css/css-conditional | 216 | 1,806 | 822 | 75 | **66.8%** | 3,258 | 2,703（83%） |
| css/mediaqueries | 27 | 113 | 237 | 0 | **32.3%** | 1,768 | 350（20%） |
| css/css-cascade | 80 | 120 | 628 | 0 | **16.0%** | 1,750 | 748（43%） |
| css/CSS2（html 面） | 63 | 318 | 1,070 | 0 | **22.9%** | 2,548 | 1,388（54%） |
| css/css-forms | 7 | 0 | 65 | 0 | **0.0%** | 93 | 65（70%） |
| css/css-link-params | 4 | 6 | 20 | 0 | **23.1%** | 93 | 26（28%） |
| css/css-font-loading | 21 | 6 | 84 | 0 | **6.7%** | 241 | 90（37%） |
| **合计** | **418** | **2,369** | **2,926** | **75** | **44.1%** | 9,751 | 5,370（55%） |

- transcripts：`.acceptance/css-v2-m2/batch3/`（诊断留档，不入库）。
- Timeout 列为 runner 内部 per-case 超时（css-conditional 75，@media
  匹配长链案）；通过率按本地可执行分母（P+F+T）计。

形态要点：

- **css-conditional 66.8%** 为 css 域已导入目录最高带（@supports/@media
  条件解析与匹配面 zw 实现成熟）。
- **css-forms 0%（7 案 65 子测试全失败）**：表单 UI 样式（appearance/
  表单伪元素）未实现域，失败簇记 M3 甄别素材。
- **css-font-loading 6.7%**：FontFace/FontFaceSet 加载 API 面——M0 甄别
  归 B2「规则/条件/级联」桶，实测实为 **API 未实现域**（与 typed-om/
  WAAPI 同性质）；量级小（分母 241），挂账注记待 M3 甄别统一归口。
- CSS2 html 面 22.9%（63 案 1,388 子测试）——门公式折算份额（300×7%）
  的实测对照基础。

## 3. DC-4 门假设校验（B2 全簇）

门公式按 B2 html 面 7,503×7% = 525 增益计。批次 3 实测 **2,369
passes**——配额的 4.5 倍，**门假设成立**（css-conditional 单目录即贡献
1,806）。DC-4 ≥20,500 门维持不改。

## 4. 下一步

- M2 批次 4 = B3 混合布局 13 目录（分母 19,286，门增益按 2% 计 386），
  同款四件套接线 + 干跑基线；导入后 DC-2 的 weekly 分片接线核对统一做。

# R5451：clamp trim 后末保留行再对齐——ellipsis 计入对齐（032 对齐参与域修复，像素增益被盒高失配掩蔽）

日期：2026-11-07　域：`crates/layout-engine/src/inline/mod.rs`（apply_line_clamp_cap trim 分支尾部）
驱动案：css/css-overflow/line-clamp/block-ellipsis-032（right/justify 对齐参与，R5450 挂账②）
接续：R5450（红带 band 域收口，余量 ✗1.30% 归因对齐参与）的第 ② 层。

## 1. 修法（css-overflow-4 §block-ellipsis：ellipsis 计入末保留行参与对齐）

对齐 pass（break_lines.rs:1416）先于 clamp cap 执行且 content_width 含被剪 hang 宽
（R1338 豁免是 IFC 级 `preserve_whitespace`；032 容器 normal＋span pre-wrap 的 run 级
悬挂不入门）；kept-last 截断前恒非末行（truncate 前缀不变量）→ 再对齐值取 `text_align`。
trim 生效分支尾部追加（同门 `clamp_hang_trim`）：

- **right/center**：整行附加位移 Δ，使 kept + ellipsis 合宽按对齐值落位——
  `desired_first = line_limit − kept_w − ell_w`（right）／`(avail − kept_w − ell_w)/2`
  （center）；Δ = desired_first − first_run.x。032 right 盒：老 offset −5ch（"This"
  4ch 左裁），Δ +7ch 后行起点 2ch ＝ ref（27ch 行右对齐 29ch 盒）。
- **justify**：kept run 重均布至 `[left, line_limit − ell_w]`（ellipsis 作末项占 ell
  宽，per-gap extra 线性叠加在老 justify 累计位上）；**cleared tail 重折叠到末 kept
  run 新末端**——老 justify 把 hang run 也当内容分 gap，折叠位（trim point）与 kept
  端之间隔着旧 gap，uniform 位移不复 glued（R5297 band 域零延伸口径不变）。
- ellipsis advance 以首 kept run 字体度量（`advance_of('…', None, fs, is_ahem)`，与
  paint `measure_char_cached` fit 门同源预期，防误走 cut-back 截字形）；R4213 行组
  参照系未镜像（FIXME 另案）；整行皆 hangable（kept 0）不位移；Left 无位移（box0
  口径不变）。

## 2. 常驻测试

- `src/inline/tests/r5451_clamp_hang_realign_tests.rs`（单元 3）：right Δ 位移＋cleared
  glued＋band 上界 = line_limit − ell／center 合宽居中／justify 重均布（首 kept 原位、
  末 kept 端 = line_limit − ell、cleared 重折叠、band 零延伸）。
- `tests/r5451_clamp_hang_realign.rs`（集成 2，032 形态 Ahem 200px）：right kept 末端
  = 200−16＝184、清零 glued、band 上界守卫／left 负向（再对齐不触发，kept 原位 x=0）。
- kill-switch 与 R5449/R5450 同门（`ZW_CLAMP_HANG_TRIM`，OFF 钉沿用
  r5450_clamp_hang_x_collapse_kill_switch.rs——再对齐严格内含 trim 分支，OFF 全关）。

## 3. A/B 定谳（同二进制 kill-switch 双态）

- corpus 16594 双窗：ON 698F / OFF 697F；movers＝ON-only {table-anon-146,
  iprd-dynamic-006}、OFF-only {table-anon-147}＝在册互翻族，solo ON ×3 全 ✓0.00%
  洗清＝**真态 REGRESSED=∅ FIXED=∅**。
- block-ellipsis family：双窗逐案全同（3 稳定红＝021/032/bidi-003）零 churn。
- driving 032 solo：✗**1.30%**（与 R5450 逐像素同值，6233/480000）——**本切片像素
  零增益**，残差主体转移见 §4。

## 4. 残差重归因（REFTEST_DUMP fresh 像素域取证，R5450 方法注记纪律）

032 ✗1.30%（6138 diff px）分解：

- **主体＝盒高失配（新归因，R5450 挂账②勘误）**：test 页（clamp 路径）盒内容高
  **19px**（border 行 68/88-89/105-106/126-127/143-144/163-164），ref 页（normal
  路径）**23px**（68/93/110/135/152/177）——每盒 +4px 全部落在文字**下方**
  （盒内文字相对位置两态一致：text-top − box-top = 6px 同值），四盒累计 y 漂移
  0/+4/+9px → 边框整行差（~12 行 × 234px ≈ 2800px）＋全部文字行 y 错位逐像素 diff。
- **水平对齐域（本切片）已落像素**：box0 绿 span x 域两态全等 [116,207]；box1 绿
  span 右缘两态对齐（223 同值，R5450 态应为 ~175 的 −56px 偏移已修）——但 y 失配
  使每行墨迹均错位对，像素 % 不体现。
- 布局级定谳（引擎探针，estimate 度量）：mono nowrap/'…'/bold/clamp 四形态盒高全等
  18px → **4px 差只在 runner 字体环境出现**＝normal 路径行盒高与 clamp 路径
  Σ`LineBox.height` 对字体度量（ascent/descent）消费分叉，或 ref 页 '…' 内容字形
  fallback face 参与行盒高度而 paint 附加 ellipsis 不参与——两假说未定谳，需
  runner 字体环境探针（FontLoader 通路）定位。

## 5. 门禁矩阵

- fmt clean；clippy workspace 绿。
- layout-engine 全腿 `--no-fail-fast`：19 腿 **2476P/0F/2I**（lib 2435P = R5450 2432
  ＋3 单元钉；集成含 2 新钉零失败）。
- make reftest **725/725**。
- corpus 双窗 A/B 详 §3（真态零开关确定性差异）。
- product-smoke 全 PASS（welcome 17.23% 逐字节同值＋struct 全 PASS＋
  form-input-perf-gate）。
- bench-gate 定向 zero-layout-engine：见控制面补录（静窗纪律）。

## 6. 方法注记

- **像素域取证先行**：本切片初判「对齐参与＝残差主体」沿自 R5449 IFC dump 推理，
  fresh 双 PNG 逐行 diff 勘误——残差主体是盒高失配（整行边框差＋y 错位），水平
  对齐域只是次级。单看 case % 无法区分「修了没生效」与「修了被掩蔽」，必须逐域
  分解 diff 像素（边框行 / 文字行 / x 域 / y 域）后再归因。
- 探针环境（`LayoutEngine::new` 直 compute）无 FontLoader/advance 注入＝estimate
  度量，字体度量域问题在该环境**必然不可复现**——归因探针的环境保真度先于探针
  本身。

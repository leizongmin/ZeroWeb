# kc5r96 verdict——color 相对色刀（刀 C NeedsPlain 改道＋刀 A 通道叶＋刀 S supports 门＋刀 B f64 串化）（2026-11-25）

## 结论

- **刀面**（指纹 `ZW-KC5R96-RELATIVECOLOR-FACE`，电池代 b1 bin md5
  6cde107ca97736a200268a081749a842＝R96/R95 双指纹并存 strings=1/1；
  b0=kc5r95/bins/r95-animtimeline-runner 0ee0d454c54f5635ef25598fda802519
  R96 指纹=0；face＝8 文件，GENERATIONS.txt gen1/gen2 在册）：
  1. 刀 C＝part11-computed-color.js 覆盖版 `__zwCssColorNeedsPlain`
     currentcolor 臂后插 random( 早退 false——宿主是唯一 random 求值器，
     声明含 random( 一律交还宿主；毒化点＝覆盖链嵌套括号正则
     `[^)]*` 失配 canonical key → NeedsPlain=true → plain 抢答 alpha 塌 0
     （布防 §一定谳）。
  2. 刀 A＝css-parser random.rs args 级预代入（**形变如实记**：布防
     section 循环方案首跑哨兵出值 panic 定谳——parse_random_parts 首节
     分类拒 ident 节（`g + 51`）到不了循环）＝collect_relative_color_origins
     预扫（rgb/rgba 边界词＋match_paren 最近 scope＋首参 from＋
     split_origin_channels→parse→Rgba 通道化，values/color.rs 该 fn 转
     pub(crate)）＋args_has_channel_leaf 快速门（纯标识符 r/g/b/alpha）＋
     substitute_args_channel_leaves 按节代入（key 节 parse_key_section
     保原文；r/g/b→0-255、alpha→0-1；无标识符节透传；未知标识符
     fail-closed None）＋eval_random_call `origin: Option<[f64; 4]>` 参；
     section 循环还原零触。
  3. 刀 S＝part06.js supports 钩 random-aware 分支：
     `_zwRandomPlaceholderForm` 外层优先整体替换 random(...)/random-item(...)
     → '0'（≤8 轮收敛，括号失配/超深返 null）→ `__zwCssColorValid(替换形)`
     放行＝random 叶 relative 形 supports 判定真（键③ test_random_computed_value
     :117 前置硬门）。
  4. 刀 B＝**复活＝布防「移出」预注册被推翻如实记**。布防 B1e/B2e
     「宿主串化已通」实测的是写面 JS normalizer 解析字面形路径；
     impl1 探针 E2/E3/E7 通道值对而串化形塌 u8（rgb(51,51,255) 非
     color(srgb 0.2 0.2 1)）。D1-D4 诊断链定谳：random 形写面不可解析
     保原文（D4）→ station 替换落级联 → style.color 持 Mix/RelativeColor
     AST → gCS 无条件 resolve_color_current 量化 u8。修复＝paint/color.rs
     f64 核（`mix_srgb_f64` premultiplied alpha 插值＋
     `resolve_relative_color_rgb_f64`，量化版重构于其上）＋
     computed_style.rs "color"/"background-color" 臂改
     `color_computed_to_css` 三分派（ColorFunction srgb 站/Mix in srgb/
     RelativeColor rgb → `color(srgb f64)` canonical，format_num 5 位
     有效数字；其余变体原臂零漂移；**非 srgb 空间维持 u8 现状 FIXME**）。
     键② 精确形依赖 f64：(30+21)/2=25.5 → 25.5/255=0.1 精确（u8 round
     →26→0.10196 失配）。
- **布防预测账 as-run 全兑现**（判读门零修订；刀面两处预注册修订
  如上如实记）：

  | 门 | 布防预测 | as-run | 判 |
  |---|---|---|---|
  | G-F2P | =4 精确（:409 键① bg % 叶刀 C+A／:421 键② color-mix 裸数叶刀 C+A+B／:420 键③ relative 通道叶刀 C+A+S+B／:448 键④ shared-by-property 哨兵刀 C+A） | **=4** 刀内精确，out-of-registry=0 missing=0 | 兑现 |
  | G-P2F | =0 fail-stop | **=0** | 兑现 |
  | 哨兵 | 'Shared by property' Fail 集 1→**0** 清零（:448 键④翻 P） | **=0** | 兑现 |
  | held-F | 14 键持 F 程序化防漏 | **held-F-bad=0**（judge 全语料 held-F 计数 3131 与布防 registry 域 14 口径差＝WARN 非门，registry 域 14 键显式核对全持） | 兑现 |
  | G4 | SET-IDENTICAL 非 random 页 | **非豁免页 diff=0** | 兑现 |
  | G5 | GONE/NEW=0 | **9882/9882 = 0/0** | 兑现 |
  | NET | +4（6745→6749） | **6745→6749 = +4** | 兑现 |

- **探针门 probe-impl 双臂（impl4）定谳**：b1 十五门全 P（E1-E7 刀面
  七门＋D1-D4 刀 B 诊断四门）；b0 反证刀 B 只动 station 面（E1-E5/E7/D4
  全 F＋E6 三锚双臂恒等 P＋b0 D1-D3 P＝写面解析路径布防轮即通）。
  前代考古：impl1 串化形塌方→刀 B 复活；impl4 十五门全 P 后发射。
- **电池双腿** css-values RC=0/0 满量 257 页/9882 case（b0 05:10:34→
  05:13:48、b1 05:13:48→05:16:59，各 ~3min）＝**judge VERDICT PASS /
  verify VERIFY-PASS**（judge 七门＋verify 恒等式 +4==4−0＋基线 6745
  与 r95 收账逐位钉死）。judge/verify 预注册硬码零 bug 零修订一次性
  通过（连续第三轮）。
- **发射事故如实记**：电池首发 PROBE-PAGE-CLEAN=1（探针页未 trap-rm
  即发射）→ 即时 systemctl stop＋rm 探针页＋rm -rf results 双目录＋
  重发射 v2（PROBE-PAGE-CLEAN=0 核）。刀 S 语法险情如实记：
  `_zwRandomPlaceholderForm` 误插 `_zwCssNamespace` 对象字面量内
  （function 声明非法）→ 29 文件装配序 bundle `node --check` 揪出，
  移至字面量外。
- **门禁**：clippy 三包（zero-css-parser＋zero-engine＋zero-style-system
  --all-targets）RC=0；fmt --check RC=0 零 diff。judge/verify 复跑存档
  judge-r96-run.log／verify-r96-run.log＋judge/verify-r96-final.out
  RC=0/0。

## 挂账移交

1. **非 srgb 空间 gCS u8 现状 FIXME**（刀 B 三分派 else 原臂）——新登记。
2. **键 2 % 叶保留形分歧**（font-palette 引擎级域外）——延续。
3. **descriptor 层 4 键** @font-palette-values parser 刀域——延续。
4. **继承×挂表子元素链 FIXME**——延续。
5. **margin computed 通道 6 位截断精度形**（r89 移交）——延续。
6. **substitution 页余 8 键** var()/attr() 基座刀域——延续。
7. **wrapper 族 5 键全清**（math-depth +2／vtn ident +21／font-palette
   +2／animation-timeline +2／color 相对色 +4＝族累计 +31，6718→6749）。
8. **DC-3 周账** 12-01 窗口。

## 工件

- `.acceptance/wptfyi-css/kc5r96/`（本地不入库）：NOTES-kc5r96.md
  （§零机制/§一刀面/§二预测账/§三探针门/§四移交/§五收账）、
  judge-r96.py／verify-r96.py（预注册硬码）、judge-r96-run.log／
  verify-r96-run.log＋judge/verify-r96-final.out（复跑存档 RC=0/0）、
  battery-r96.log／battery-r96-final.log（双腿 RC=0/0＋发射四件套）、
  probe-impl-r96.log（impl4 双臂）、bins/r96-relcolor-runner＋
  GENERATIONS.txt（gen1/gen2）、results-k96-{b0,b1}／probe-results
  （JSON 结果集）。

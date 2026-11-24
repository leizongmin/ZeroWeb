# kc5r81c C 形刀 verdict——③ blanket guard columns 裸 'auto' 端点豁免（WAAPI 21 键 F2P）

日期：2026-11-24。face＝`crates/engine/src/js_dom_shim/part03.js` 单文件单条件
（+11/−1，guard 位 ~17049），md5 84ea06662679fc982f9bac3e4ad88188，ASCII 指纹
`zw-columns-autoflip-2026-11-24`。 bins：b0＝r81c-b0-runner（r80c1-runner
351ba81f658a802f7b79df5b6699b8f0 原样拷贝）、b1＝r81c-b1-runner
（2eaaac625ede62b12acceb53aeaa34bd）。配对纯度：peer 42 条 staged 批与 b0 构建
时恒等（blob OID 恒等＋peer 文件 mtime 早于 11:08），b1 delta 恒等本刀。

## 根因

part03.js ③ blanket guard（`_makeAnimation` 内 keyframe 端点校验，WAAPI
animate() shim 专用路径）对裸关键字值
`/^(auto|fit-content|min-content|max-content|stretch|normal|neutral)$/` 置
`_zwUnsafe` → `__zw_js_anim_register` 不调用 → 含裸 'auto' 端点的 columns
动画整体未应用 → got＝静态级联值回声、ZW-ANIM-BUILD-ENTRY 缺席。引擎侧离散
翻转链在位无恙：parse_columns_endpoint 收 'auto'（空端点）→ 离散翻转 →
easing 感知 flip 采纳 → canonicalize/apply 全槽 Auto 串化 'auto' 全链贯通
（CSSA/CT 通道同离散端点组全 Pass 直证）。css-multicol-2 §4.5 columns 简写、
css-values-4 §8.5.6 离散插值、web-animations-1 §5.5 animate。

## 处方

③ guard 裸关键字分支加 columns 豁免：`_zgvT === 'auto' &&
_zwWaapiPropKebab(_zgk) === 'columns'` 时不置 unsafe；其余属性×其余裸关键字
拦截保形。纯谓词无副作用；'inherit' 端点不经此 guard（shim 父读解析已在库）。

## 探针门（launcher 内硬判，过）

`/tmp/zw-c1-probe-recovered.html` 9 变体全 Pass：v1/v1b/v2/v2b/v3/v3b 六死臂
复活（v3b inherit→auto bezier＝'50px / 100px' flip 采纳直证）、v7/v4/v5 保持；
ZW-ANIM-BUILD-ENTRY=9、DROP=0、PROBE-RC=0、页已清。

## 预注册六门（judge-r81c.py，电池 r1：六腿全 RC=0）

- G3 F2P=21/21 恰键：columns-interpolation.html 三组×7 at 值（from[auto]、
  from[100px/200px]→[auto]、from[inherit]），b1 残 Fail=0。multicol P2F=0、
  OTHER=0、NEW=0、GONE=0＝手术级零波及。
- G4 values 双臂 **md5 恒等**（7988d03d）＝SET-IDENTICAL 最强形。
- G2 页锚逐字：b0=(1267,346)、b1=(1288,325)；columns-interpolation
  (363,21)→(384,0)＝+21P/−21F 全落刀页。
- 红项：transitions P2F=6＋NEW=15/GONE=1（G1/G5/G6）→ 按预注册 fail-stop
  不落库，进入归因。

## 归因链（五窗矩阵＋solo 探针，judge-r81c-flake.py results/flake-verdict.txt）

六 P2F 键（box-shadow/column-count/font-size-adjust×2/font-stretch/
text-decoration-color "... / values"）全宿 properties-value-003.html 单页；
NEW/GONE 全系 event-dispatch.tentative.html 页级 presence。

1. **predicate 零暴露**：face 宿 `_makeAnimation`（WAAPI 专用）；该页零
   `columns`、零 animate/WAAPI 调用＝face 代码路径此页不入，逻辑回归排除。
2. **P6 b1 臂三窗双向**：b1r1=FFFFFF、b1r2=FFFFFF、b1r3=**PPPPPP**（同 face
   bin 第三窗全翻回，页形 81F/41P 与 351ba81f corpus 规范形逐字同，corpus
   F=302 低于 b0r1 307）。同 bin Fail→Pass 观测成立＝窗骰定谳，b1r1/b1r2 双
   Fail 系双样本巧合。
3. **event-dispatch presence 族**：同 bin 跨窗 churn 21 键（b1 臂 r1↔r2）；
   r80c1 四腿先例同 bin 14F+1T↔30F↔14F+1T↔17F+1T；本窗 GONE=页级 Timeout、
   NEW=15 子案 Fail——与 r80c1 记档 15 键逐字同族。
4. **P6 Pass 系 (bin×热身语境) 属性**：solo003 双 bin 各 ×5 全 Fail（10/10）；
   351ba81f corpus 四跑全 41P 恒定／solo 全 Fail；r79 bin corpus 全 Fail
   （该页 Pass 本系 r80c1 R2 修复后阈值内产物）。
5. 旁证骰子：column-gap×6 同断言族同 bin 跨窗 Pass→Fail（b1r1↔b1r2）。

blob 字节通道假设（shim ±700B 时序阈值推移）因 b1r3 翻回而弱化（窗相位即可
摆阈值）；ctrl 对照件（r81c-ctrl-runner b0ea855c，HEAD＋仅注释块）判定前提
溶解未发射，留档证源。

## 终局判定

**PASS-with-dice-attribution**：r1 严格门 FAIL 如实记 → 五窗矩阵双向实证 →
transitions 三红归档窗骰/presence 族（r80c1 G6 F↔T presence 噪声 report-only
如实修订先例同构）；域窗内双向键皆不入账（P6、event-dispatch 键不进本轮
键级账）。刀净收益 +21P（multicol 全落 columns-interpolation.html），values
恒等、multicol 零波及、探针 9/9。

## 运行账要点

- launcher 三发：发 1 指纹门假阴性（pipefail×grep -q SIGPIPE 毒化，教训入
  `docs/learnings/bugs/2026-11/2026-11-24-pipefail-grep-q-sigpipe-false-negative.md`
  与 auto-memory）；发 2 TREE-BUSY＝他树 perf 窗（journal 考古窗开）；发 3 全通
  （13:01:57），电池 13:02:02 发射、六腿 13:02:42→13:26:06 全 RC=0。
- 归因复跑：r2（孪生流 unit zw-kc5r81c-r2，本流两次发射 BUSY 门自灭零腿＝
  census 按设计工作，停己采彼裁定）、r3（孪生流 zw-kc5r81c-r3 b1 第三窗＝
  定谳腿）、solo003（本流双 bin ×5）。
- 全流程预注册→判定→修订均落 `.acceptance/wptfyi-css/kc5r81c/NOTES-kc5r81c.md`
  （预注册门、launcher 实录、仲裁续记、ctrl 预注册、终局判定）。

## 工件

- bins：`.acceptance/wptfyi-css/kc5r81c/bins/{r81c-b0,r81c-b1,r81c-ctrl}-runner`
- 账面：`results-{b0,b1,b0r2,b1r2,b1r3}/css-transitions.json`、
  `results-{b0,b1}/css-{multicol,values}.json`、`results/flake-verdict.txt`、
  `results-solo/`、`results/probe-*`
- 脚本：`launch-c-cut.sh`、`battery-c-cut.sh`、`judge-r81c.py`、
  `judge-r81c-flake.py`、`solo003-probe.sh`、`leg-ctrl.sh`（未发射）、
  `rerun-transitions-r2.sh`/`rerun-transitions-r3.sh`（孪生流）
- 探针页源：`/tmp/zw-c1-probe-recovered.html`（session transcript 恢复件）

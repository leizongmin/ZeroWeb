# 插值鲸池分解（2026-11-22 05:3x–05:5x，fork-d 侦查笔）

> 本文件为 .acceptance/wptfyi-css/PREP-interp-pool-decomposition.md 的入库版，
> 基线：results-knife-csinterp-s2a-r4/b0-json（B0 md5 827497ef，HEAD
> fa89cbb31 账面态）；F2 见证窗独立 judge 复核同步在案
> （NOTES-arb-0525 fork-d 补笔，pristine 快照 -pristine-0548）。

# 插值鲸池分解 PREP（2026-11-22 05:3x，fork-d 只读侦查——零树触）

权威面：results-knife-csinterp-s2a-r4/**b0**-json（B0 md5 827497ef，基线 HEAD
fa89cbb31＝当前账面态）。池量：css-values interpolation 26 页 4,267 案
**2,474 F**；css-sizing 10 页 3,123 案 **864 F**；合计 36 页 **3,338 F，Timeout=0**
（时间线负延迟 delay:-50s 机制工作，纯值机制缺口）。

S2 verdict 旧注（@106783「~2,300F 皆 WA-3b 时间线门控，挂起至
pause/currentTime 落地」）**已过时**：`js_anim_registry.rs` seek 通道已落库
（3c1564bf3/aee9915cb，part03.js:16397 `__zw_js_anim_seek` 接线在库），HEAD
态 WAAPI 变体仍 72% 败——残面是值机制非时钟。

## 变体归因（抽样页合计，b0 全量口径）

| 变体 | 机制 | 失败/总数 | 归因 |
|---|---|---|---|
| Web Animations (WAAPI) | animate()+pause()+currentTime=50s | 508/706 (72%) | 见 FAM-A/B |
| CSS Animations | @keyframes + delay:-50s | ~380/~530 (72%) | 见 FAM-A/B |
| CS-single/CS-all | transition delay:-50s | 249/698 (36%) | 见 FAM-C/D |
| CS-discrete | allow-discrete | 214/686 (31%) | S2A 判死域（勿入） |

## 子族分解（按证据签名）

### FAM-A · gCS used-value 缺口（横切最大，估 >800F）

案证（interpolate-size-width 页 WAAPI/CS 全变体同签名）：
`width from [initial] to [50px] at(0) expect 150px got 0px`；at(0.6) got 50px
＝**from 捕获把 keyword（initial/auto/unset）解析成 0px 而非 used 150px**
（Chrome gCS(width) 返 used px）。t≥1 端点过＝to 侧正常。
同根横切：g38 挂账「gCS resolved-value 刀（width 槽 Auto gCS 'auto'≠'15px'）」、
properties-value 池 `/values` 臂 134F（gCS 读串空/关键字形）。
刀位＝style-system 序列化 resolved-value 通道 + engine 捕获路径。
**风险**：'auto' 序列化改 used px 是大 blast face——pages 里另有意断
'auto' 字面的域（cssom-view 等按 spec 只对 layout-related 返 used）。
须严格按 cssom-1 §resolved value 清单圈属性域再预注册。

### FAM-B · 单侧/中性 keyframe（WAAPI 单 keyframe 帧）

harness 对 neutral keyframe（initial/unset）两侧均**省略该帧**（interpolation-
testcommon.js :207 isNeutralKeyframe 门）→ 单 keyframe 动画＝隐式 0%/100%
帧取 underlying。引擎缺隐式帧回填（读 underlying computed）。
与 FAM-A 同域叠加（underlying=used value）。

### FAM-C · calc-size 求值基座（values 1,442F）

`calc-size-height/width-interpolation` 768+551F：`expected 75px got 0px`
＝中点 t 时 calc-size(size 基座) 求值得 0（intrinsic 基座插值上下文无测量）。
关联 S2A 判死教训：**输出必须走 canonical 解析回程**；r1–r4 四轮 fail-stop
家族本域关闭，重开须按其 verdict 两条件（b0 F 行圈域＋canonical 回程）。

### FAM-D · 序列化散点（立即刀候选，~50F）

1. **calc(infinity*1px) 数值语义**（calc-interpolation 20F，勘误：**非 KA2
   哨兵泄漏**——案证 `left from [0px] to [calc(infinity * 1px)]`：Chrome
   把 infinity 代换为有限数 i64::MAX 再数值 lerp（at 0.25→2^61、at -0.25→
   负外插、at 0→0px）；引擎逐 t 恒出 `9223372036854775807px`（∞ 短路直串）。
   CS-Anim/WAAPI 臂则 to 端点解析拒收→`got auto`（10F）。fix＝calc infinity
   代换口径统一（finite i64::MAX 数）或 lerp ∞ 端点数值化；须核 KA2 max-*
   Px(INFINITY)→'none' gCS 臂不回退（其值不经 calc eval，构造直出）。
2. **%-mix calc 串化**：`expected calc(0% + 25px) got 100px`（20F）＝%
   混合 calc 引擎提前求值；css-values-4 §mixed-percentage 要求保 calc 形
   （width 语境 % 可解析时才可 used 化）。
3. **空串期望**：`expected "" got calc(...)`（10F+10F）＝invalid 端点
   应拒收（setter 面）。

### FAM-E · composition 页全灭（interpolate-size-*-composition 110F/0P）

`Compositing CSS Animations` 变体＝animation-composition @keyframes 通道；
与 11-19 ANIMCOMP S2/S3 getAnimations 对象桥刀候选同域，归其 lane 不重复。

## 下刀排序建议（互斥认领，NOTES 追加）

1. **FAM-D1 ∞ 泄漏**（小、具体、有 P2F 风险面——KA2 自家引入，优先清偿）
2. **FAM-A gCS resolved-value 圈域刀**（最大横切；先 solo 探针页钉
   'auto'→used 清单域，判据页宽预注册）
3. FAM-C calc-size 基座（S2A 教训约束下重开）
4. FAM-D2/D3 随 D1 同窗小刀

## 全语料鲸面普查（05:44 补——r4 腿只测了 values+sizing 切片）

`*interpolation*.html` 全语料 302 文件：masking 37、transforms 33、
backgrounds 25、motion 21、values 19、filter-effects 15、gaps 14、fonts 13、
images 11、sizing 10、text 9、ui/multicol/CSS2 各 7……按本切片
36 页 3,338 F 的失败率（~66%）外推，**全鲸 ~10k+ F**（未实测，以
per-corpus 腿账为准）——插值机制族（FAM-A/B/C/D＋intrinsic keyword
machinery） collectively 门控的量级＝收 80k 缺口的主攻面候选。

## 纪律

- 本 PREP 零树触（F2 battery 在飞窗）；实施前按 S2A verdict 预注册纪律。
- b0 JSON 为 2026-11-22 05:20 账面态；后续轮实施时重取 HEAD 权威跑防漂移。

## 勘误附笔（05:5x，fork-d 独立复核——FAM-A 证伪确认）

05:52 侦查笔（5e01f4645）证伪本笔 FAM-A 段；fork-d 本地 b0 切片（values 3,440F＋
sizing 2,469F）独立复核确认：

- `from [initial/unset]` × width/height × interpolate-size/keyword 页失败 152 案
  （4 页各 36–40）——message 全谱**恒 pinned-to-value 形**（expected 38/50/62/74/110px
  插值列，got 恒 90px 含 t=-0.3/0/0.6/1.5），`capture-to-0px` 签名 **0 案**＝
  初稿「keyword 捕获解析成 0px」机制不存在；该 152 案归 FAM-B（中性 keyframe
  帧省略→单 keyframe 动画隐式帧未回填，引擎以声明帧双填）。
- `from [auto/unset/initial]` × width/height 全域 577 案逐页分解＝calc-size 系页
  （FAM-C calc-size 基座域）非 gCS 读出面；'auto' 值信号失败 0 案＝bridge 返数值
  正确（与 05:52 笔 4 语料 8,714F 圈域结论同向）。
- stretch 恒差 15 形本地实证 44 案（stretch-alias-*，expected 45 got 60）＝
  FAM-A' 结构候选成立。

结论：本笔 FAM-A 段作废，以 05:52 笔为准；FAM-B/C/D/E 分解不变。初稿
「interpolate-size-width at(0) expect 150px got 0px」案证系 r2 旧账面（05:20
b0 态已不含该形）——侦查笔引用旧案证未复核当轮 b0，记为教训。

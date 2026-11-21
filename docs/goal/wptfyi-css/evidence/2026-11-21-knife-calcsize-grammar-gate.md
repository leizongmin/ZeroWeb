# 刀 CALC-SIZE-GRAMMAR · calc-size 写面文法门（2026-11-21）

## 缘起与认领

原 fork（MATHFN r1 发射线）于 05:3x 认领（NOTES-knife-calcsize-grammar-
claim.md），face 落树后该 fork 殁于 11-20 夜 429 限流风暴（rc=6 撞
RANDOM3b 半成品编辑波后未重发）。本线接手：face 完好在树
（apply.rs +479 行 ±，marker ×1），电池脚本/census/patch 四件齐备，
08:27 实测三连静默（unit 无＋build 进程无＋.rs 编辑窗无）后发射。

## 机制

face = `calc_size_decl_gate` 尺寸槽整值文法门（apply.rs，白名单检查
之前——height/min-*/max-*/block-size/inline-size/flex-basis 十三槽同
收口）：

1. 单一顶层 `calc-size(<basis>, <sum>)` 形 → basis/sum 文法裁决
   （css-values-5 §8，<https://drafts.csswg.org/css-values-5/#calc-size>；
   basis 关键词域 per 属性、length-percentage 形、嵌套 calc-size 仅
   basis 位、sum 的 size 原子与关键字拒形全按 00:30 语料 94 案矩阵）；
2. 其余形（math 函数内嵌/多跨度/括号残缺）整拒——此前经
   `value_has_unimplemented_function` 放行＋math 门 substitution 豁免
   双通道原文透传 store（invalid 面 53 形 store 回显即 Fail）；
3. 替换函数族（var()/env()/attr()）与 random 族形不入本门
   （computed 期替换重裁/自有文法门）；
4. DEFINITE 臂（已落 `f53710dfc`）回归哨：`calc_size_definite_tests`
   2P 保持——strict 门叠层不误杀 definite 形（00:50 让位笔承诺项）。

钉测 `calc_size_grammar_gate` 先执后落库：3P/0F（gate 钉＋DEFINITE
双哨），零 DOA。

## 电池（unit zw-knife-calcsize-battery-0829）

08:27:12 发射（编译门 8.38s dev 检查绿），08:27:43 b0 build（face
`git apply -R` 剥除）→ 08:28:50 b1 build（face 回填）→ b0/b1
css-values 双腿（08:29:54/08:33:07）→ 08:36:01 GATE-DONE rc=0。
flock /tmp/knife-calcsize-battery.lock；test-guard 全包裹。

产物 `.acceptance/wptfyi-css/results-knife-calcsize/`：bin/b0-runner
`37a0b956b55b947d0746e22ac8f21cfd`、bin/b1-runner
`90f8bfd15da7e20acd61d642948fa1fa`（与 motion2-v4 b1 字节恒等＝同树
内容确定性构建互证）；b0/b1 css-values JSON/err 自包含；s1/s2/s3
md5 三窗＋ride-files.txt。

## 结果（css-values b0→b1）

- `calc-size-parsing.html`：P 35→88（**+53** 拒形翻账）
- `interpolate-size-parsing.html`：P 2→5（**+3**）
- 其余 254 页 SET-IDENTICAL；**gained=56 lost=0**，ASYM 零行
  （9866 断言双臂对称零幻影）。

恰集兑现预注册预测（脚本头 +53/+3）。应用面（height/width/box-sizing/
flex-basis 页）F 计数不变＝声明文法门只动拒收形，definite 求值臂
（+41 已账）零扰动。

## ride 与代际法证

- RIDE 5 文件（types.rs/es_module.rs/guidelines SKILL.md/rendering
  docs ×2 对端 WIP）双臂同载，S1/S3 md5 三窗零扰动。
- 本电池窗口（08:27–08:36）全程 systemd unit 存活跨两次 session
  重启（限流风暴），watcher 三度重建零影响账目；收账时点无并发
  cargo 进程。
- 收账前置测试执行：`cargo test -p zero-style-system --lib calc_size`
  → 3 passed 0 failed（2591 filtered）。

## 残面挂账

- CALCSIZE-CANON：canonical 序列化 6F（`calc-size(min-content, 2 *
  size)` 序形）；typed-om 19F（OM lane）。
- CALC-SIZE-INTRINSIC ~209F：box-sizing 30F＋`size` 词元 intrinsic
  形＋% 腿 gCS＋flex-basis layout Calc basis 消费（layout 期内在
  尺寸求值径）。
- calc-size 插值臂：四纯零页 3348F＋部分页 ~845F（height 222/width
  193/min ×2 118/max ×2 97）＝engine lerp 域，MOTION lane 后续。

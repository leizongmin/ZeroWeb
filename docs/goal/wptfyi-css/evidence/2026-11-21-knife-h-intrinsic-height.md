# 刀 H：calc-size 高度轴 intrinsic 测量机器（block_content_height）

日期：2026-11-21
face：crates/layout-engine/src/engine/calc_size.rs 单文件（patch-only 流：
`resolve_basis_px` 高度轴 intrinsic 臂——block_content_height 内容高测量
（intrinsic 关键词基 basis 禁宽度代高，css-sizing-3 intrinsic block-size）＋
auto 槽高度轴机器＋box-sizing 语义维＋sum % 语境修（父高 indefinite 时
sum 内 % 解析 0，css-values-5 §calc-size）
判别：FACE-MARKER `HKNIFE-H block-content-height`（b0=0/b1=1）＋M2（bin
md5 异）＋RIDE 12 文件 census 零扰动
电池：knife-h-intrinsic-height-battery.sh（对端 v2 unit
zw-knife-h-intrinsic-height-battery-v2，21:02 发射 21:19:07 GATE-DONE
rc=0，TRAP-STRIP reversed 树净）
bin：b0 `cd7e1e9d5b32191391f004b173328b71`（与 KA b0/CLIENTWIDTH-PADBOX
v2 b1 同源互证＝HEAD+ride 基线一致）/ b1
`f7f10af6e76ae000281ffe1d9cbfc92d`；PATCH-MD5
`a2c7c6510dfb4a5574ee471d65c098e6` 绑定

## 预注册（NOTES-knife-h-intrinsic-height-claim）

- 主靶 calc-size-height.html：+11（intrinsic/auto 机器 +8、% 语境 +3；
  G1b d50fb2f99 可能已吃部分——以本电池自产 b0 为基线逐案核对）
- 主靶 calc-size-height-box-sizing.html：height 槽 +5；min/max-height 槽
  4F 不应翻（越域）
- 观察窗：calc-size-flex-basis-on-column/row（column 主轴 flex-basis
  intrinsic 臂连带，翻 P2F 按面因果归因）
- 哨兵：sizing/flexbox/variables 页级 SET-IDENTICAL（face 无宽度轴变更；
  sum_cb 只 axis_h 分支）

## 结果（GATE PASS：F2P=12 全入账，P2F=1 定谳 flake 剔除）

- judge（case 级全四语料）：GRAND gained=12 lost=1 F2P=12 P2F=1 ASYM=0；
  **tally +12**（P2F flake 不入账）
- calc-size-height.html：b0 N=37 F=11 → b1 F=5＝**+6**（预注册 +11，
  残留 5F＝% 语境/intrinsic 混合 basis 深水区域内未竟：
  `calc-size(any,31%)`/`calc-size(max-content,31%)`（auto 容器）/
  `calc-size(min-content,size/2)`×2（auto+definite 容器）/
  `calc-size(any,31%+12px)`——挂 H 刀余量，非回归非越域）
- calc-size-height-box-sizing.html：b0 F=9 → b1 F=4＝**+5 恰合预注册**
  （height 槽 5F 全翻；min/max-height 槽 4F 保持＝越域哨符合）
- 观察窗兑现：flex-basis `calc-size(content, size)` +1（column 主轴
  intrinsic 臂正向连带，F2P 零对冲）
- 哨兵：css-sizing/css-flexbox/css-variables 全 F2P=0/P2F=0
  SET-IDENTICAL ✓
- P2F 唯一案 `Shared globally: random(--identifier, a, b)` 定谳 flake：
  CSS Values 5 random() 共享实例实现缺口固有掷币形——四采样交叉验证
  （KA b0/b1 双 Pass＋本电池 b0 Pass/b1 Fail），H face（calc-size 高度轴
  布局期）与 random() 值解析零机制关联；flake 域窗内双向不入账
- 有效性：FACE-MARKER census b0=0/b1=1（编排窗毒化排除）＋RIDE 12 文件
  零扰动＋bin md5 双代际记录

## 链史（发射竞态与死锁排障）

- 20:18 本流首发（zw-knife-h-intrinsic-height-battery）入 SERIALIZE-WAIT；
  KA 20:50:28 退出后 cwcv/H 序列化门互指死锁（cwcv 门排除字面不含
  h-intrinsic、H 门排除字面不含 cwcv）——20:57 stop H 打破，cwcv
  20:57:17 放行完成；H 重发撞 face 编排竞态 STRIP-CHECK-FAILED rc=2
  （对端贴 face 窗）让位
- 对端 21:02 发射 v2 unit（同脚本同 patch md5），b0 四腿 21:03–21:11、
  b1 四腿 21:11–21:19，GATE-DONE 21:19:07 rc=0

## 收尾

- face 入库（patch-only 流收账路径：staged blob 14ba89b45＝patch b-side
  精确一致）＋master.md 专行＋tally 累计 +12
- 余量挂账：①calc-size-height 残留 5F（% 语境/混合 basis）②random()
  共享实例缓存缺口（flake 根因，独立刀候选）③calc-size-height-box-sizing
  min/max-height 槽 4F（符号刀域）

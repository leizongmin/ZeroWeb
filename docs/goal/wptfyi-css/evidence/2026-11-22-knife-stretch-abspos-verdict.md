# 刀 STRETCH-ABSPOS 收账判定（2026-11-22 07:3x，fork-d）

## 面与电池

- face：`crates/layout-engine/src/engine/abspos.rs` 单文件 +51/-5
  （`stretch_abspos_direct_cb` 四臂：W/H definite inset 臂补自身 margin
  扣除＋W/H auto-inset 双 auto 臂＝static 位伸到 CB padding-box 远端；
  css-sizing-4 §6.2 / CSS2 §10.3.7 margin-box 填充约束式）。
  patch md5 `87da97cc0c7601b2b324490dba55f65d`。
- 电池：`results-knife-stretch-abspos`（unit zw-stretch-abspos-battery，
  07:20:24 发射 serialize 门过）双腿对称构建：b0 `be0e77ab`（HEAD
  剥面）/b1 `8600d4f4`（贴面）md5 异 M2 过。六相全 rc=0：
  sizing b0 07:21:25→values b0→sizing b1 07:28:30→values b1，
  EXIT rc=0 07:33:36。快照 `results-knife-stretch-abspos-pristine-0734`。

## 判定

- **主判据**：css-sizing gained=52 lost=0——预注册 4 页×12 恰集全翻
  （stretch-inline-size-001 / stretch-block-size-001 ＋ alias 孪生，
  abspos inset-inline:0 期望 55 形＋auto-inset 期望 50 形 × 双 writing
  mode × box-sizing 重测）＋keyword-sizes-on-abspos.html +4（同机制页
  域外溢出：abspos width:stretch 行，逐行核 mechanism 一致）。
- **哨兵**：css-values gained=0 lost=0 SET-IDENTICAL。
- **P2F=0 → COLLECT**。tally +13115 → **+13167**。
- 观测窗（r3，07:16）先行证 48 页域内 48/0＋float/controls 行零扰动
  （不在 face scope，挂后续刀）；正式电池双腿全语料复证 52/0。

## 域外挂账（后续刀候选）

1. **F 臂 float stretch**：float:left 行恒 60（padding-box 形）期望 45
   ＝converter is_float→percent(1.0) 落 padding-box 基箱，每 stretch 页
   2-4 行。
2. **C 臂 form controls**：input/textarea width:stretch 未施（178 UA
   默认），每页 4 行。
3. min/max-size stretch 页（46+46+39×2 F）＝percent(1.0) margin-ignoring
   形（R4224/R4237 注释自证），margin 扣除语义同本刀。
4. vertical-rl 变体与 block-size 页残行＝同臂轴向扩展。

# 刀 BFC-INLINE-MARGIN · inline-block 容器 taffy 父子 margin 折叠抑制（净 +4）

- 日期：2026-11-21；fork：9aae62cb 续开线；电池 unit：zw-knife-bfcim-battery-1521（仲裁线代管发射，GATE-DONE 15:41:13 rc=0）
- face：`crates/layout-engine/src/tree.rs` build_subtree establishes_bfc 臂纳入
  `DisplayValue::InlineBlock`——gate ①`!ctx.float_precedes`（R3755 float-adjacent
  负面排除）＋gate ②父容器非 Flex/InlineFlex/Grid/InlineGrid（vendored taffy
  `Overflow::is_scroll_container` 含 Hidden → flex 项自动最小尺寸钳 0，
  flexbox.rs:870/track_sizing.rs:598 消费点，taffy-local style/mod.rs:364）；
  模块顶层 `BFC_INLINE_MARGIN_FACE_MARKER`（ASCII，b1 rodata 探针）。
  钉测：`crates/layout-engine/src/tree/tests.rs::bfc_inline_margin_inline_block_taffy_isolation`
  （断言 taffy 输入层 overflow x/y Hidden，1 passed）。
- 机制：inline-block＝BFC root（CSS2 §9.4.1），与流内块子不折叠 margin
  （§8.3.1）；taffy 0.12 父子折叠把首子 margin-top 折出容器——
  calc-size-height-box-sizing.html 容器 gCS 期望 22（target border-box 19＋
  mt 2＋mb 1）实得 20（＝19＋mb，mt 2 丢失）。R3755 排除史＋R4251 flow-root
  gate 复启先例的同款复启。taffy overflow:Hidden 仅输入层，视觉裁剪走
  computed overflow（R3755 既证）。
- 电池：b0（HEAD＋ride 剥 face）vs b1（face 在树），ride 44 文件（URLS/S2
  各流 WIP 对称同载）；FM b0=0/b1=1、M2 异 md5、S1/S3 ride 窗内零扰动；
  语料 values/box/sizing/text/flexbox 五腿×双臂。patch 纯度经对端法证独立
  确认（tree.rs/tree/tests.rs 文件级独占，URLSMOD rc=6 的 face 提取过宽
  事故不涉及本刀，NOTES-post-genfam-queue-arbitration 15:31 笔）。
- 结果：RECONCILE values gained=4 lost=0，box/sizing/text/flexbox 全
  0/0，无 ASYM＝P2F 0。
  - 恰集：calc-size-height-box-sizing.html 页 P 0→4（+4）——四臂
    `height:auto`、`auto+border-box`、`calc-size(11px,size*2)`、
    `calc-size(11px,size*2)+border-box` 全断言过（容器 gCS＋gBCR 22/29/37/25
    系全兑现＝mt 入高机制生效）。
  - 预注册 6 臂中其余 2 臂（`calc-size(auto,size)`±border-box）未翻：
    失败点移至 target.gCS（expected 7px got 203px）——b0=b1 对称，系本刀
    构建窗（15:25）树中 S2b-v2 在途面改 calc-size(auto) target 解析的
    中间态（14:29 基线该臂 target 通过 7px），calc-size basis 域＝S2 lane，
    本 face 不可达亦未被推翻；其 S2 落库后 container 断言重达即应兑现。
    其余 9 臂失败挡 target.gCS（size*2 乘数/min-max basis/max-height
    覆盖＝S2/S3 域）与预注册一致。
  - flexbox 哨兵零位移＝gate ②证明面；width 页 container canary 未动
    ＝水平 shrink-to-fit 域未越；margin-collapse 族 SET-IDENTICAL。
- 限制：display:contents 父穿透漏面接受（罕见）；flow-root/contained
  flex 项同形钳 0 暴露已在库（R4251/R3755 同暴露），不回溯。
- 协调：队列仲裁（NOTES-post-genfam-queue-arbitration）队位
  GENFAM→URLSMOD→BFCIM→其余；9aae62cb 收账＋c095cd53 核产（一账两用）。
  本刀 battery 脚本头部序列化门（URLSMOD 进程探测＋busy-units 终门）由
  仲裁线两笔落地，本 fork 收讫。

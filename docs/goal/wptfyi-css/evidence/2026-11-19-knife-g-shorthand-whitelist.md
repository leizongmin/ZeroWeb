# 刀 G：typed-om 属性文法/opaque 兜底扩展（2026-11-19）

## 刀域与归因

css-typed-om `the-stylepropertymap/properties/` 19 页 175F（recon 17:1x，
g-knife-entries-draft.txt）：15 页 testUnsupportedPropertyTests opaque 往返面
87F + 4 页 typed 文法门 88F。认领与实施由 fork A 承接死流草案（门(4)=G2b
part10 落库 `bbc048e95` 兑现后入窗；NOTES 22:5x 认领段）。

## 机制（part10-typed-om.js 单文件 +28/−1）

- `_zwOmPropSyntax` +9 键：column-gap/row-gap（`normal|<length>|<percentage>`，
  css-box-gap/css-align-3 §gap）、overflow-x/y（css-overflow-3
  §overflow-control）、overscroll-behavior-x/y（css-overscroll-1）、
  font-synthesis 简写空数组走 `_zwOmCheckShorthandTokens` + 三个长手
  auto|none（css-fonts-4 §font-synthesis）。
- `_zwOmKnownExtra` +15 键：marker/grid-area/offset/grid-template/grid-gap/
  font/font-variant/text-decoration/grid/font-family/flex-flow/stroke/mask/
  fill/vertical-align——testUnsupportedPropertyTests computed 往返期望基类
  opaque（PropSyntax 优先命中，本表兜底）。
- `_zwOmGridClampPctProps` +2 键：column-gap/row-gap 负 % computed 钳 0
  （css-align-3 §gap used/computed 与 grid 轨道同合同）。

## 电池

knife-g-battery.sh（承 arith2 配方：patch 在盘/树净约定 + checkout/恢复
trap + RIDE census S1/S2/S3；build 段本次补齐 test-guard per-proc 6/
total 14/1800s——脚本初版裸 cargo 不可无人值守）。face 剥离树、patch
apply-check 过后发射。

**22:53 首发即败存档**：PATCH-APPLY-CHECK-FAILED＝face 先入树与正向
apply-check 互斥（fontcycle 流程习惯带错门），trap checkout+apply 净效果
零受害（两流 transcript 0 次 part10 编辑实证）；22:54:28 按约定重发。

## 结果（23:23 收账）

zw-g-battery-2302：22:54:28 发射，b0 build 79s / b1 build 70s，RIDE 11 文件
S1/S3 零扰动，23:23:27 GATE-DONE rc=0，trap TRAP-RESTORE applied。

RECONCILE（gained=152 lost=0 regressed=0 pages_gain=19 pages_loss=0，恰
19 预测页零外溢）：

- typed 文法门 4 页：font-synthesis 78->102（+24）/ gap 36->59（+23，含
  负 % 钳零臂）/ overflow 52->72（+20）/ overscroll-behavior 52->68（+16）
- opaque 往返面 15 页：vertical-align +16 / grid-area +7 / font +5 /
  grid-gap +5 / grid-template +6 / offset +6 / font-variant +4 /
  flex-flow +3 / font-family +3 / text-decoration +3 / grid +3 /
  fill +2 / marker +2 / mask +2 / stroke +2

四哨兵（animations/pva/masking/transitions）零差异（对账输出零行）。

预口径 ~165-175 vs 实收 152：draft 按失败行计数，runPropertyTests 每值形
整测翻转结构下部分行同测合并；账以本电池 b0/b1 逐页 Pass 集为准。

provenance：face patch `.acceptance/wptfyi-css/knife-g-face.patch`
（75 行，reverse/forward apply-check 双过）；JS 语法 node --check 过。

## 残面

- gap computed `normal` 关键字 get 回形（若断言期望 CSSKeywordValue 而非
  opaque 需补 keyword 面，draft §5 预案）
- marker `<url>` 形 opaque get 往返细节形（battery 未显形，暂不动）

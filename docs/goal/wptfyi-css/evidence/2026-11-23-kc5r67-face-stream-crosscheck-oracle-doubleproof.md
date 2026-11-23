# KC-5-r67 face 流复核笔：canonical 独立判读互证＋oracle 双证（2026-11-23 08:3x）

配套 097842c38（verify 流 r67 判读笔）。判定结论与该笔一致：三门全红，r67 face 不落库。本文只记 face 流复核位增量事实。

## 1. canonical 目录独立判读互证

- verify 流判读输入 = 快照目录 `solo-r67-snap-verify0823`（指纹 md5 3d156423，防覆写快照）。
- face 流判读输入 = canonical `solo-r67/`（发射实例收官后的现盘目录）。
- 同工具（`.acceptance/wptfyi-css/kc5r66-verify-json.py`）不同目录，输出逐键同值：**RESOLVED 2 / RESIDUAL-P2F 108 / REG 125 / F2P-BREAK 25**，GT-missing 0（b0/b1 各 5080 键×14 页）。
- ⇒ 判读链确定性双证（双目录 × 同工具 × 同数值）。face 流快照 `.acceptance/wptfyi-css/kc5r67-verdict.txt`。

## 2. oracle 双证：谓词非因（控制臂补全）

| 臂 | 谓词 | RESIDUAL | REG |
|---|---|---|---|
| r5 裸基（b1-json vs b0） | 零谓词 | 110 | 110 |
| r67（r5 基＋全谓词 trio） | 齐 | 108 | 125 |

- r5 裸基样例：exp `'0px '` got `'50 % '`；110 GT 键全部被打落。
- ⇒ 谓词增减不动 GT 轴崩盘（110→108 同量级）；「抑制/透传把 GT 键送回 b0 位」模型对 consult 驱动键不成立。
- b0 位机制锚：GT 键 b0 Pass 由 cssa consult/K-fold 折叠串驱动；consult None → cascade 裸 token 串（`'min - content '`/`'auto '`/`'50 %'`）≠ b0 位。

## 3. 勘误三（框架复权）

r66 归因笔（49/61/112/375）中「base 偏差＝残 61 主因」只对 r66 成立——坏栈 used_size_echo/K-fold 伪影是 r65 基特有。r67 干净 r5 基上残面 108 复现且形变（token 化 keyword echo，`'min - content '` 族）⇒ 跨代真根因＝apply-往返串化 vs cascade 静态折叠**通道不对称**（GT-110 根机制），与 07:4x ⑤ 案及 a1a2d962c 勘误收敛。

## 4. r68 靶面两案（通道对称化）

1. canonicalize 产出收敛 cascade 静态折叠形；
2. applier calc-size used 解析臂回退。

前置＝r5 patch 逐 hunk 二分定位塌缩 hunks（097842c38 已立案）；face 流认领静态定位（read-only，不动树）。谓词 trio 复用前提＝逐键 b0 双机制（consult 命中 vs None 级联）定谳；备份 `/tmp/kc5r67-scratch/trio.rs`。

## 5. 树态 provenance

- KC-5 核 13 文件＝HEAD＋r5 patch 12 驻树（`knife-kc5r5-fame2b-face.patch`，基 1160fc8af；4 文件基→HEAD 漂移经 apply -3 吸收）；cssa_registry.rs 回 HEAD，`git diff` 0 行已核。
- 残池键（which-value 784px、WA cis neutral）两代持续，另刀域。

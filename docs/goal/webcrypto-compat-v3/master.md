# Web Crypto 兼容三期运行时控制面板

**状态**: Active——M1 未启动（2026-11-01 立项 + 同日零遗留二次拍板扩围）
**最后更新**: 2026-11-01（立项 v1.1）

---

## 当前状态

2026-11-01 立项：二期收口唯一移交项 D1（已转正域按域立项）经用户当面拍板
（Claude Code 会话，2026-11-01；决策账面即
[入口文档](../webcrypto-compat-v3.md) v1.0），10 域 109 案/18,052 子测试
（universe 口径）承接为三期。

**同日二次拍板（零遗留终局）**：用户要求本 goal 执行完即 webcrypto 谱系
全部完成、无遗留——入口文档扩围 v1.1 收编四项：

1. **散新案 5 案**：pin→HEAD 漂移中不在两新族的 stable 新文件
   （export-key-error-order / okp_fixed_length / okp_malformed_asn1 /
   sharedarraybuffer / aes_kw_unwrapKey_failures），M1 一并入桶基线。
2. **worker 腿**：全子树 window/worker 双腿执行（runner 现 window 单腿；
   workers 通道 runner 能力在册，70 号先例），worker isolate 侧 crypto
   绑定核实/补齐，逐案归因不外挂。
3. **1T 销案**：algorithm-discards-context（二期定性 iframe 跨帧注入 +
   `top` 回调链 DOM 边界）本 goal 内修复。
4. **收口重定基**：终账切 HEAD 布局全量双腿账，旧 `.tentative.` 路径条目
   （锚内 7,320F 所在）退役；判据 DC-5②；加 **DC-5③ 谱系零移交**
   （一/二/三期全部闭合，非 P 子测试全部谱系内具名归因）。

- 输入：二期 [DC-2 甄别清单](../archive/webcrypto-compat-v2/evidence/2026-10-31-dc2-promotion-screening.md)（逐域分母/建议范围/依赖候选）
- 上游复证（2026-11-01）：HEAD `a0dd301b85…`，WebCryptoAPI 子树
  `084495e783…` 与甄别时（`cb6cc97ada…`）**逐字节相同**——tentative=0、
  未转正域 0、零漂移；pin→HEAD diff 80 增/65 删已逐文件枚举（入口文档
  上游事实节）。
- 底线：一期 33,109 分母零回归（25,788P/7,320F/1T）；重定基后由 DC-5②
  终账承接。
- 下一步：M1——38 号通道全子树新增式语料预置 + 分域基线（含散新案）+
  10 域 deps evidence + 选型征询；拍板前仅推进无新依赖面（门禁面/散新案）。

## 与兄弟 goal 的边界

- **25 号 html-semantics-compat（并行在跑）**：共享触面
  `js_dom_shim/part01.js`、`script_gen.rs`、`testharness.rs`；crypto 实现面
  （part01/01b/02 `__zw_crypto_*`）与其 forms 切片目录级分域但同文件——
  切片落账前对账 main 最新触面，切片号按 s162 撞号先例。
- **worker 腿接线触 runner/workers 通道基建**（testharness runner 侧），
  与 25 号流共享 testharness.rs——接线切片落账前对账。
- **33 号 editing-compat（ZeroWeb-3 工作区并行在跑）**：域不相交；合流经
  main，切片落账前 merge-base 对账。
- 23 号 selection-compat：无共享面。

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | 10 域新依赖选型（候选清单见二期 DC-2 甄别清单）：M1 deps evidence 落档后一并征询；拍板前收敛切片仅限门禁面/散新案（无新依赖域） | 待 M1 evidence |

## 碰撞管理记录（run-rules §9）

- 2026-11-01 立项时点：25 号 html-semantics-compat 并行在跑（s209 刚落
  main `785e8a166`，共享触面 part01.js/script_gen.rs/testharness.rs）；
  33 号 editing-compat 在 ZeroWeb-3 并行在跑。长跑/CI 按质量门禁守卫
  （`docs/engineering/quality-gates.md`），CI 发射前查在跑实例。

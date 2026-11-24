# DC-2 s10-registries 刀② 判读收账（2026-11-24，session 4c6091cf 续轮）

## 范围

刀② 首项：adopt/effective-registry 重钉面（刀① 判读档列出的候选首项）。编辑面
单文件：part03.js（`_zwCeAdoptStampSubtree` 走查 + appendChild frag 展平补章 +
adoptNode 非主文档臂 + `_ZW_CE_NULL_REG` 哨兵 + `_zwCeElRegSlots`/`_zwCeRegEntries`
WeakMap + `_zwCeDocDefault` 文档默认臂 + SR 递归）。零 Rust 行为面变更
（part147 桥内钉测为新增常驻回归载体，非行为修改）。

## 修复面（三刀）

1. **frag 展平补章**：R225 pre-insert 展平在 appendChild 体内清空 frag，frag 直子
   对本体 stamp/R191 走查双漏章（探针 H1 实证双走查零触达）。修复：展平前快照
   直子（`_zwCeFragKids`），展平后逐直子补章；**跨文档域限定**——同文档 frag 搬移
   无 adopt 步，旧语义零移（钉测 ⑤ 面证）。
2. **SR 递归**：shadow root 挂 host 影子槽非 childNodes，直子展平走查恒不可达
   （刀① 显式 slot 重钉臂死码化）。修复：nt=1 臂尾经 `host.shadowRoot` 入口续走
   `_walk(_srRec, _myEff, false)`；显式 slot 走 SR 臂重钉，declarative 无 slot
   维持 null 钉扎语义。
3. **SR 判据补备**：刀① wrapper（handle 域 wrap）host accessor 未必有；元素 slot
   走 WeakMap 而 `_zwCERegistry` own 属性仅 SR 载有 → presence 即 SR 判真补备。

## 探针判读（s10-probe-egr.html，wpt-data 刮擦页，用后已删）

终态 **7P/2F**（探针日志 md5 `edff7e6e7eea0b6d0462be1e11362a2d`）：

- Pass：A iframe realm / B direct-content DSR / C scoped-boundary / D/E/F/G adopt
  矩阵 / **I SR global-own 重钉** / **L scoped-own 持久** / **J detached-adopt 走查**。
- Fail ×2：B template-content DSR parse / B template-clone DSR append——已知
  B 域 DSR 物化缺口（FIXME 刀②续独立刀位），本刀前即败，非本刀回归。

### 探针方法教训（两则）

1. **scoped-own 是设计面非缺陷**：test I 初版用 scoped registry 钉 SR slot 断言
   重钉——spec/设计均为 scoped-own 持久（L 面证）。可观测重钉面须用 global-own
   slot（attachShadow({customElementRegistry: customElements})）。前轮「I-face
   失败」是测试设计错误，非 shim 缺陷；walk 计数器（walk=12 sr=2）亦证走查恒达。
2. **刮擦页三重追加伪影**：诊断测试块逐轮 append 未替换，文件含三份 I/DIAG/J/K
   副本，计数器膨胀（walk=12）即此伪影。自 orig 备份重建单份后一次判读即清。
   教训：诊断块更新一律整页重建（orig + 单份新块），不逐轮 append。

## 桥内常驻钉测（随本提交）

`crates/engine/src/js_dom_bridge_tests/part147_s10_adopt_restamp_pins.rs`
（md5 `3fc9b4c36228d967b343552ed98ddf6c`；include! 注册于 js_dom_bridge_tests.rs）：五面 ① frag 展平补章
（跨文档 appendChild(frag) 后 global-own 翻 null-reg）② SR global-own 重钉
③ scoped-own 持久 ④ null-own 保持 ⑤ 同文档 frag 搬移零移（域限定门）。
实测 `cargo test -p zero-engine --lib test_s10_adopt_restamp_pins` →
**1 passed; 0 failed**（test-guard --compile-first 6/12/2400 守卫内）。
探针页属 gitignored per-clone 刮擦面，常驻载体即此钉测。

## 电池判读（锚 3897181f4 detached，part03.js 脏态构建）

三腿全绿零漂移（per-(page,status) 键 join + membership 双证）：

| 腿 | post | 刀① 锚 | 键 join | membership |
|---|---|---|---|---|
| testharness-web-components | **17995P/4016F/19T** | 17995P/4016F/19T | 466/466 零计数漂移 | 恒等 |
| testharness-domparsing | **1264P/271F/1T** | 1264P/271F/1T | 80/80 零漂移 | 恒等 |
| testharness-sanitizer-api | **414P/342F/1T** | 414P/342F/1T | 42/42 零漂移 | 恒等 |

- 判读域：24323 结果行（wc 22030 + dp 1536 + san 757 = 24323；探针页已删，无
  scratch 键混入）。逐键 `LC_ALL=C awk` 提取、`join -t\t` 状态+页复合键、计数
  漂移 awk 比较、membership diff 恒等断言。
- sanitizer 腿首行 `new CustomElementRegistry()` → `__zwCeScopedActive` 门激活
  面在飞（gate-ON 全量走查含 adopt 补章/SR 递归路径），零漂移＝门路径行为中性
  双证。registries 簇页（gate ON 同面）零漂移同证。

## 碰撞记录（对 clean clone 构建的影响）

- origin/main `43ca50c9d`（css 流）含 6×E0609（`contain_intrinsic_width_auto`/
  `contain_intrinsic_height_auto` 字段定义存在于无已推 commit——css clone HEAD
  3f0040da5 平行未推行，其已推 computed_style.rs 亦无该字段）。**净效果：
  43ca50c9d 之后 clean clone `make build` 破**。
- 本刀判读锚退守最后已知好锚 3897181f4（detached + 脏态构建）。修复属 css 流
  （style-system 字段+访问面），不跨流代写；lark 已通报。本刀 JS 面字节独立于
  该破面（include_str shim + 锚态二进制）。

## 字节锚

- part03.js 终态 md5：`a4eb2f9f09228c7c8abf066b94d1607e`（诊断计数块除净：`__zwDiag` 计 0；拼接
  part01+01b+02+03+04+05+06 → node --check SYNTAX_OK；标识符后缀通配
  `grep -oh "<stem>[A-Za-z]*" | sort | uniq -c` 五 stem 单拼写）。
- 判读绑定二进制 md5：`e212845bd38b3a4a2f548123a8163735`。
- 电池日志（evidence 同提交 add -f）：dc2-s10-wc-post-knife2.log（`587f38a3b36ecb45f4ad660566b35064`）/
  dc2-s10-domparsing-post-knife2.log（`71b248b65663e9a5d173465e59605428`）/
  dc2-s10-sanitizer-post-knife2.log（`e1877af5754951c282a43634433eb5a9`）/ dc2-s10-probe-r5.log（`edff7e6e7eea0b6d0462be1e11362a2d`）。

## 刀②续候选（随档更新）

B 域 DSR 物化（探针 B×2 面 + 语料 56F 页主病）/ R90 trap scoped 原型臂 /
parse-time attr 区分 / DSR shadowrootcustomelementregistry 引擎 deep JSON /
importNode·upgrade-order·criteria 页 / IAH handle-only 插入复杂体（刀① 归因）。

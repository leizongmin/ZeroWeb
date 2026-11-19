# knife11 sep 刀体两次抹除定谳与字节位还原核验（d8abed68 主线，2026-11-07 14:31）

## 1. 事件时序（逐条有据）

1. **13:22:43** gates 批（batch-knife11-gates.sh）起飞 head=fdfa770f5：post md5
   `ad212b5f8ed9422bb835190abb43487d` ✓ + marker（paren old=0 new=3 / noparen
   old=1 new=1）✓ → enginelib **ROSTER OK(6-red)** → 三 post 腿出数
   （ip 6987P/208F rows 7195、iol 1759P/2F rows 1761、iul 2061P/4F rows 2065）。
2. **~13:24 pre 态构建假 ABORT——批脚本配方缺陷**：`git stash push -m
   knife11-gates-pre -- $SHIM4` 落 stash@{0} = `440687fbdb214b6e094a6750b0278fa189cd0539`
   后，PRE_EXPECT（gates 脚本 ：66）按 `git show HEAD:$f` **串接内容** md5 现算
   （`a1c305134c136987aea23fb80a330120`），而比对目标 md5snap（:24）是 **md5sum
   输出行再 md5**（got=`7cda92feaa8680b08359b4ac84a008cb` = 真 HEAD 态值）——
   两配方不可比，恒假阳性 → ABORT。
3. **ABORT 路径 pop 实际成功**（gates 脚本 :72）：日志中 `Dropped refs/stash@{0}
   (440687fbdb…)` 即 pop 的 apply+drop 输出——翻转回到树。
4. **14:09 第二次抹除**（执行者未署名）：四 shim mtime 同秒 14:09、内容回 HEAD
   （flip marker 全 0、md5=7cda92fe）；`js_dom_bridge_tests/part06.rs` r3254
   期望预改（div→p 双断言块）幸存。
5. **14:26:55 字节恢复**（执行者未署名，活跃轮）：四 shim 同秒重写，diff stat
   22+4+4+10 与 stash 形状全等；`git diff 440687fbdb -- crates/engine/src/
   js_dom_shim/` **空** = 现树 == stash blob 态逐字节（恢复源即该 stash 对象或
   同字节重放）。

## 2. 恢复态核验（本会话 14:29 只读）

- recipe md5（verdict 批 ：26 同配方：四文件路径 md5sum 输出再 md5）=
  **`ad212b5f8ed9422bb835190abb43487d`** ✓ ——**位还原**。
- marker 复核：paren 新形 `== 'div') ? 'div' : 'p'` part01×2 + part01f×1 = 3 ✓；
  noparen 新形 `__zwDefaultParagraphSeparator === 'div' ? 'div' : 'p'`
  （part01e:515）= 1 ✓；noparen 旧形 `=== 'p' ? 'p' : 'div'`（part06 值面）= 1 ✓。
- **位还原 → 399090bd VERDICT OK 与 knife11-mt-post.log 判读自动随附**（判读绑定
  现树字节态，无需重跑 mt）。
- stash 源对象 `440687fbdb214b6e094a6750b0278fa189cd0539` pop 后 unreachable 但
  仍在 odb（cat-file -t = commit）；刀体落地提交后该对象冗余，仅存档备查。

## 3. 分工与落地权

- 落地所有权 = census 轮不变（landing-division 与 clockskew 两笔记既有约定）；
  14:26:55 恢复者按活跃轮视为落地推进中。**本会话不做刀体落地提交，仅核验**。
- part06.rs r3254 期望预改在树（本会话所有物），与刀体同窗提交约定不变
  （landing-division §3：避免 shim=p / 期望=div 破头窗）。
- 本会话监听落地提交出现 → 五文件完整性（四 shim + part06.rs）+ md5 绑定 +
  push 状态核验；落地轮长期沉默或消亡时按「写者死于 push 前核验方代跑尾链」
  先例承接尾链。

## 4. 门禁批脚本缺陷移交（防重跑假 ABORT）

- `batch-knife11-gates.sh` :66 PRE_EXPECT 配方（blob 串接内容 md5）与 :24
  md5snap 配方（md5sum 行再 md5）不一致 → pre 检查恒假阳性。重跑 gates 前须修：
  PRE_EXPECT 改为对 HEAD blob 落临时文件后跑 md5snap 同配方，或逐文件
  `git show HEAD:$f | md5sum` 比对值集。
- pre 腿（HEAD 态三腿基线）因假 ABORT 未采；但零回归已由全分母 mt 终判覆盖
  （P2F=0 / post-only=0 / lost=0、rows 10092 恒一，单文件腿数据 ⊂ 全分母语料），
  pre 腿缺失**不构成落地阻塞**。

## 5. 本会话后续

监听落地提交与 push；落地出现后核验并按需补 lark 通知与控制面同步的核验记录；
v2 刀（typed-fate 25 键）归属按 twin 轮活性定，探针
`zz-probe-knife11-v2.html` 在语料盘待跑（须 batch.lock 串行）。

## 6. 补记（14:35，读 gates-closure 笔记后身份定谳）

census 轮 `2026-11-07-knife11-gates-closure.md` 识别了本笔记 §1.4/§1.5 的
未署名之手：**14:09:59-14:27:25 census 轮修复补跑 batch-knife11-pre.sh 的
stash push/pop 周期**——14:09 四文件回 HEAD = 其补跑 stash push（pre 态构建，
逐文件 `git show HEAD:$f | cmp` 口径修复）；14:26:55 恢复 = 其临时 stash
（`406e160d`）pop（slots 4→4 恒一）。即两次抹除/恢复均为落地权轮自身批周期，
非外力抹除；本笔记 §1.5「恢复源即 stash 对象 440687fbdb 或同字节重放」收敛
为后者（census 补跑自有 stash 同字节重放，`git diff 440687fbdb` 空仅证字节
等价）。配方缺陷诊断两轮独立一致（本笔记 §4 = closure §1）；pre 腿经补跑
三腿 judge 全 VERDICT OK（rows/P/F 逐键恒等、四红线全零），gates 门禁收口。

## 7. 补记（15:05，落地推送后核验）

- 落地已推 origin/main：`72da4f974`（五文件同窗 +41/-13，含本会话 part06.rs
  r3254 期望预改）+ `c700bfb2a`（gates 收口 + anchor-rebase 换基传递 + 控制面
  第十一刀收口行，方向表述正确）。
- 换基锚链：`ad212b5f`（12:51 判读锚，98b1e5296 基）→ `092cc4db`（14:39 现树
  锚，s328 a896a0707 落基，399090bd anchor-rebase §1/§2 传递定谳）→ 落地字节
  = landing-verify §1 复核一致。本会话一次 `bc9c927f` 读数系自制 recipe 路径
  标签伪差——锚配方 = md5sum **输出行含路径串**再 md5（verdict 批 ：26 全相对
  路径口径），裸文件名口径对同一字节得异值；字节层面 blob 对账：stash→落地
  part01e/part01f/part06.js 零差、part01.js 仅 s328 区 hunk、flip marker
  paren-new=2 完好。
- 勘误指针：落地正文首段「缺省翻置从 'p' 改为 'div'」方向写反（实际
  **'div'→'p'**，对齐 multitest 语料录音缺省而非 Chrome/spec 初始 div）——
  399090bd landing-verify §2 已记；控制面收口行方向正确，正文勘误随其后续
  docs 提交处理。
- v2 动向：本地 part01f.js :1533 区再现未署名 +10 行脏改（15:0x，恰 Fix A
  施工位点）——按 clockskew 分工先核 v2 认领轮（twin）活性，本会话不重复
  施工；探针与 Fix A/B 处方已随 verdict-verify/landing-division 两笔记在档。
本会话五份核验笔记随本文档提交存档；刀体五文件落地由 census 轮执行
（含本会话 part06.rs r3254 期望预改）。

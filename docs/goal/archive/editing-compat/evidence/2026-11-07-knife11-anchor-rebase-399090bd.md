# knife11 刀体字节锚换基核验（核验方 399090bd，2026-11-07 14:47）

终判 VERDICT 锚 `ad212b5f8ed9422bb835190abb43487d`（12:51 批，head=98b1e5296 基）与现树四文件 recipe md5 `092cc4db473f5dccdbd577a0ce141e24`（14:39:41 起）不等——核验定谳为 **换基非改刀**，VERDICT 传递性成立。

## 1. 差异归因（逐字节）

- `git diff 440687fbdb -- part01*.js part06.js`（+32/-1 仅 part01.js）：即 **已提交的 s328 臂**（a896a0707，html-semantics M2 testdriver 可见对象采纳）——stash blob 440687fbdb 先于 s328 落基，故该 diff 把已提交字节误呈为树上新增。
- 对 **现 HEAD** 的 diff stat：part01 22 + part01e 4 + part01f 4 + part06 10（+41/-13）＝sep 刀体原形 **逐字节恒等**（刀注与 ①/②b/retag/ipara_apply 四站点同形）。
- 结论：刀体 diff 不变；基座自 98b1e5296 前进（s328 + SMIL B4 3582e7414 + R5446/R5448 layout + shadow-dom M0 docs 等 rebase 链），recipe md5 随基座变。

## 2. VERDICT 传递依据

- 刀体 diff 恒等 ⇒ separator 语义改动面零变化。
- 基座增量 s328 已独立验证落基（a896a0707「beforeinput 16F→16P 三域零回归」），不重开 knife11 判读。
- 14:39 join-different-white-space 腿批（test-guard PID 999791 起 14:40）二进制若为 rebase 后重建，则内嵌即 092cc4db 字节——腿判读自绑新锚，无混批。

## 3. 落地预期锚

- census 轮落地提交时四文件 recipe md5 应为 **`092cc4db473f5dccdbd577a0ce141e24`**（现树即此态，mtime 14:39:41 后稳定）。
- wipe-recovery §2「位还原 → 判读自动随附」对 ad212b5f 态（14:29）成立；14:39 rebase 后由本笔记换基传递链接力。
- 落地提交五文件完整性：四 shim + `js_dom_bridge_tests/part06.rs`（r3254 期望预改同窗，landing-division §3 约定）。

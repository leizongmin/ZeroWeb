# CSINTERP-S2a r1 判定证据（2026-11-22 01:20，监视/四眼流）

## 判定

**不收账**。RECONCILE TOTAL gained=0/lost=0（values/transitions/sizing
三语料全零，sizing 哨兵 SET-IDENTICAL ✓）。预登记 F2P（interpolate-size
min/max 页 with-keywords 臂）未现＝采账判据条件(1)不成立；零散布位移/
零 P2F 成立＝face 零扰动安全但零增益。r2 不发：机制无效非运行污染，
重跑不改变结局。face patch（.acceptance knife-csinterp-s2a-face.patch）
归档不回贴——其 allow-discrete 门序修复仅当 lerp 机制以正确形式重启时
才有价值。

## 运行链与扰动仲裁（结论对结局无影响，存档备查）

- r1 00:54 发射（unit zw-knife-csinterp-s2a-r1；00:53 首发因脚本缺
  exec 位 Permission denied，chmod 后重发）。
- 窗内 ride 扰动：ride es_module.rs（s419 WIP）md5 S1=9d6449b6 →
  S2/S3=1c6f394f（＝HEAD 提交态，WIP 被「HEAD 态覆写」剥除，01:0x 前
  已复原 9d6449b6）→ S1/S3-MISMATCH 双旗标。二进制法证互证：b0-runner
  3 处 zwImpS 符号（复用旧 rlib）、b1-runner 0 处（窗内重编译）→
  两臂模块加载代码不同代。因结局全零，扰动无影响面；门抓捕有效。

## 机制事后检讨（b0 失败案消息定谳）

- kw↔px 对（264F）期望**数值化 used-size lerp**：`from [0px] to [auto]
  at (0.3)` 期望 `15px`＝对元素 auto 实高 50px 数值插值
  （interpolate-size: allow-keywords 语义），非 calc-size 符号形。
- face 产出 `calc-size(<kw>, <px> + size * <coef>)` 符号形，与上述期望
  零交集；符号形期望属 kw↔calc-size()/calc-size()↔calc-size() 族
  （~521F），face 的 parse_px_strict 对其返 None 从不路由。
- 产出形与期望形零交集 → 零增益全解释（非门/序列化小瑕，机制假设被
  ground truth 推翻）。

## 池分类账（S2b 设计简报，values calc-size/interpolate-size 页 F 普查）

| 族 | F | 期望形 |
|---|---|---|
| discrete kw↔kw 翻转 | 225 | t≥0.5 翻转（none↔max-content 型） |
| 符号 calc-size 形对 | ~521 | kw↔calc-size() 与 calc-size()↔calc-size() 混合代数 |
| 数值 used-size kw↔px | 264 | px 数值 lerp（keyword 端点解析为 used size） |
| base-kw 端点基值 | 152 | initial/unset 解析为 computed 基值再数值插值 |
| composition/logical 变体 | ~191 | 组合/逻辑属性页形变体 |

## 队列解锁

S2a face 弃置不入库 → registry.rs/animation.rs/transition*.rs 无在飞
face → TEXT-INTERP 全管线（机制 1/2/3 ~630F）窗口开放。
四眼过程详见 .acceptance/wptfyi-css/NOTES-knife-csinterp-s2-size-lerp-claim.md。

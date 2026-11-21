# 刀 CSINTERP-S2A r3 电池判定（2026-11-22 02:47 收账）

**判定：FAIL-STOP，face 弃置不入库（P2F=12 红线）。r4 face 双 hunk 就绪（本笔同轮预制）。**

## 运行记录

- unit `zw-knife-csinterp-s2a-r3`（02:22:29 发射，02:47:47 EXIT rc=0，GATE-DONE 同刻）。
- 腿：values/transitions/sizing × b0/b1 六腿全 rc=0（2m54s/6m55s/1m43s 量级）。
- bin：b0 md5 `db9e0a7fca382fc3db84af11d9aa50c3`、b1 `6ef961521309f93743d90dc0665618fe`
  （02:24:2x 建毕）。FACE-MARKER b0=0/b1=1；S3 ride 窗内零扰动。
- 产物快照：`.acceptance/wptfyi-css/snap-knife-csinterp-s2a-r3-0248/`（b0-json/b1-json
  六 JSON＋md5 清单，EXIT 后即时 cp 防 sibling 覆写）。

## verify 流法证前置（02:25-02:33 strip 窗）

verify 流在 b1 bin 建毕后（02:25 起）对树做了 r3 face 法证剥除（五文件逐 hunk
`-R`/`--include`）。收账前完成读序法证：**b1 bin mtime 02:24（strip 前）＋
`nm -C`/`strings` 双通道命中 `size_keyword_pair_lerp` 符号＋interpolate-size 串**
→ b1 bin 含完整 face（registration+strict 集成+stretch 端点），假零风险解除，
12 P2F 为完整 face 的真实行为非剥除伪影。

## RECONCILE 账

| 语料 | gained | lost | 备注 |
|---|---|---|---|
| values | 0 | **12** | 全部单页 `interpolate-size-max-height-interpolation.html` |
| transitions | +1 | 0 | `zw-probe-wa3b-census.html` P 2→3＝探针页非机制增益，剔除 |
| sizing | 0 | 0 | b0/b1 JSON md5 恒等（`103037ac`）＝SET-IDENTICAL 最强形 |

ASYM 1：`animations/animate-with-color-mix.html b0=3 b1=5`＝transitions 语料
异步发现数漂移，按 r2c ASYM 判例零折算（非 face 因果，转写不入账）。

**TOTAL gained=1 lost=12 → P2F 红线触发，fail-stop。**

## P2F 逐案归因（12 案全同页三组）

三组 × t∈{-0.3, 0, 0.3}，全为 `test_no_interpolation` 期望对：

1. `transition-behavior:allow-discrete` from [stretch] to [fit-content] ×3；
2. `transition-property:all + allow-discrete` 同对 ×3；
3. 纯 `CSS Transitions`（无 allow-discrete）from [none] to [max-content] ×6
   （direct + property:all 两变体）。

语料权威（同页互证）：页面设 `:root { interpolate-size: allow-keywords }`，
但同页 `test_interpolation` 仅 **keyword↔length 混对**（`min-content`→
neutralKeyframe 100px，期望 `calc-size(min-content, -30px + size * 1.3)` 形）；
`stretch→fit-content`（kw↔kw）与 `none→max-content`（none 非插值 keyword）
期望**离散**。上游 width/max-width 变体页同构（width stretch→auto、
max-width stretch→none 均 no_interpolation）。

## 根因（两级，均定位到 hunk 级）

**① 登记旗标过松（12 P2F 直接根因）**——transition_registry.rs
`s2a_pair_has_size_keyword` 用 `kw(from) || kw(to)` OR 形且 kw 表含 `auto`：
kw↔kw 与 none↔kw 对误设 `s2a_symbolic=true` → `discrete: !strict_ok &&
!s2a_symbolic` 误翻 false → query_core 走 s2a continuous 分支 →
`size_keyword_pair_lerp`（match 只收混对）返 None → 回落 lenient（continuous
entry）→ keyword 端点解析失败产非期望值。b0（无旗标）离散翻转/未登记跳终值
恰好满足断言 → P→F。

**② canonicalize 回程墙（0 F2P 根因，r2c 预告的第三级）**——预注册主判据
（min/max 组混对行出 `calc-size(<kw>, …)`）零兑现：query_core 尾 `return
canonicalize(prop, &raw, …)` 经 `AnimationClock::apply_to_computed_style` →
`apply_single_property` → `parse_max_sizing_anim_value`（px/none-only）拒收
calc-size 形 → `ser1 == ser0` → None → 采样回落基值。r3 face 未触 apply/
canonicalize 面（r2c「两级死链缺一仍零效应」判断兑现：registration 修复后
暴露第三级墙）。

## r4 face 处方（双 hunk，均 transition_registry.rs——TEXT-INTERP face 零交集文件）

1. **混对单源探针**：`s2a_pair_has_size_keyword` 改为
   `size_keyword_pair_lerp(from, to, 0.0).is_some()`——旗标与求值端单源
   （match 只收 (kw,len)/(len,kw) 且 len 侧 parse_px_strict），kw↔kw 与
   none↔kw 对旗标恒 false = discrete 保位（12 P 复位）。
2. **symbolic 输出直返**：query_core s2a 臂 `pair_lerp` Some 时
   `return Some(sym)` 绕过 canonicalize——s2a 输出与基值恒异形（S2b 捕获
   收窄排除 min/max，基值无 calc-size 形），直返等价「已改变」合同，
   format_num 恒定期望形零格式漂移（`0px + size * 1` / `-30px + size * 1.3`
   同页期望逐字吻合）。

r4 预期：max-height 页 12 P 复位（b1==b0）、min/max 四属性混对行 F2P
（calc-size 断言串）、values/transitions/sizing 哨兵零漂移。patch 存
`.acceptance/wptfyi-css/knife-csinterp-s2a-face-r4.patch`（r3 全代＋两 hunk
替换，md5 `a85b00a1a40b63d6e6d6fffab50a2e6f`；transition_registry.rs 段对
HEAD blob `patch -p1 --dry-run` 通过，hunk 头行号经生成器重算自洽），
发射窗=TEXT-INTERP 落库树净＋rebase 后（共享文件 animation.rs/registry.rs/
inherit.rs/computed_style.rs 与 TEXT-INTERP face 共存域拆先例已证）。

## 归因勘误（实施流补正笔，2026-11-22 04:xx）

状态行归因中「none 对 strict_ok 新放行 flip」一说不成立：r3 patch 未触
strict fn min/max 臂（`parse_px_strict` 对 none/keyword 端点恒返回
None ⇒ `strict_ok=false`），none↔max-content 组与 kw↔kw 组同源——登记
实走旗标 OR 路径（`kw(max-content)=true`）。即 12 P2F 全部由「登记旗标
过松」单根因产生，无第二机制。r4 hunk A（`pair_lerp` t=0 单源探针收紧
旗标）双组全覆盖；12 P 复位已推演：plain 组三门全假早退跳终值，
allow-discrete 组 discrete=true 翻转臂保位。

## 台账影响

r3 账面 0 入账（P2F fail-stop 弃置）；12 P2F 系 face 相对 b0 的退化、face 未
落库故不进目标账。interpolate-size 池余量：混对行（calc-size 期望形）r4 主靶。

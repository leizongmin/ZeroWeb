# kc5r70 solo 判读——R5 修复形全量验账：RESOLVED 20（历史最佳）／RESIDUAL 90／非 GT REG 0／F2P 4，P2F 门未过 face 不落库（r71 工作基定性）

日期：2026-11-23 · 流：KC-5-r70（r69 毒点二分修复形全量验账）· 判读器 `kc5r66-verify-json.py solo-r70` · raw：`.acceptance/wptfyi-css/kc5r70-verdict-raw.txt`

## 电 provenance

- 28/28 腿 rc=0，11:02:19 SOLO-VERIFY-ALL-DONE；14 页×双臂（b0-runner md5 16ce5984 GT 基线代／b1 r69r5-runner md5 0feac854 R5 修复形，脚本 :24 显绑防模板陷阱）；单实例锁 /tmp/kc5r70-solo-instance.lock 生效；b1 keys=5080 pages=14 与 b0 对齐；结果快照 `results-kc5r6-fame2b/solo-r70-snap-r69bisect-1104`（防覆写）。

## 三门判读

| 门 | r68 臂 A | r70（R5 修复形） | 方向 |
|---|---|---|---|
| RESOLVED | 0 | **20** | 历史最佳代（轨迹 r65:16→r67:2→r68:0→r70:20） |
| RESIDUAL-P2F | 110 | **90** | −20 |
| REG（含 GT 双计入） | 110 | 90 | 非 GT 簇空＝**0**（与 r68 同：REG 全为 GT 键集双计入） |
| F2P-BREAK | 0 | **4** | 新代价，已归因（下节） |
| GT-missing | 0 | 0 | RESOLVED 20＋RESIDUAL 90=110 恒等 ✓ |

**P2F 门 FAIL（90>0）＝face 仍不落库**（REDLINE fail-stop）。R5 形定性＝**r71 工作基**，非落库候选。

## 增益面（修复形两全兑现）

- 页级：height-composition b1 155P/25F vs b0 69P/111F；min/max-width/height-interpolation 四页 b1 fail 从 111-118 级压到 28-35；calc-size-width b1 371P/471F vs b0 291P/551F。fame2c 的 calc-mix/keyword 受理增益全保，GT clamp 族 silenced（探针 5/5）与全量 20 RESOLVED 一致。
- 残面 90 键全部为 calc-size 复合串折叠缺口（got 'min - content '/'auto '/'200px '/'fit - content ( 100 % )' vs exp calc-size(...) 折叠 used px 形）＝**案 2 applier 复合折叠刀的域**，与毒点修复正交。

## F2P 4 键归因（R5 新代价，如实记）

`stretch → calc-size(50px, size)` at(-0.3) ×width/height×CSS Transitions/all 两 lane（calc-size-height/width 页），got '- 15px '。机制＝负外插 '-15px' 被 R5 回落臂写入 width/height 槽且 serialize 对 width/height 族无非负钳（min-height GT 键的钳行为不覆盖 width/height）；臂 A 同键委托拒收→canonicalize None→echo 巧合 Pass。r71 精修方向＝回落臂按 css-sizing-3 propdef <length [0,∞)> 逐属性钳，或回落域收窄至 serialize 有钳的属性族。

## 挂账（域外）

- width 1em→10vw GT 键维持 FAIL '16px '（离散翻转 raw "1em" 被静态 applier em 受理）＝插值通道单位解析刀域。
- r69 二分全程与毒点机制链详证见 [2026-11-23-kc5r69-poison-bisect-r4-pin-r5-fixform.md](2026-11-23-kc5r69-poison-bisect-r4-pin-r5-fixform.md)。

## 下一步（交棒面）

1. r71 主刀＝案 2：applier calc-size 复合串折叠到 used px（可回收 90 残面主体，to-neutral/KW→cs/NEU→KW 族全覆盖）。
2. r71 精修＝F2P 4 键回落钳（小刀，与案 2 可同 face）。
3. 单位解析刀（width '16px ' GT 键）维持挂账。

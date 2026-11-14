# g34 终账：g33 提交态干净 HEAD 单二进制——values 4,009P 复证 + 八套件电池锚点核验

日期：2026-11-14。背景：g33（progress() 全链，提交 8c13a2856）验证账遗留
「终值以终跑 + 补套件电池为准」——前轮 g34 链 /tmp 日志随会话结束丢失，
本链全量重录。二进制 = 当前 HEAD（8c13a2856 后 docs 提交不触码）静树重建
（`zw-g34-build.log` build exit=0，40.61s，css-parser fingerprint fresh）。

## values 全量（256 案，干净 HEAD 二进制）

| 项 | g32 提交基线（results-g32-syscolor-full） | g34 终账 | 变化 |
|---|---|---|---|
| Pass | 3,935 | **4,009** | 净 +74 |
| Fail/Timeout | — | 1,577 / 7 | — |

python 逐页 diff：**REGRESSED=0**；GAINED 恰 5 页——progress-serialize
13→54（+41）、progress-computed 0→27（+27）、progress-invalid 33→35（+2）、
clamp-integer-invalid 12→14（+2）、hypot-pow-sqrt-invalid 45→47（+2）。
与 g33 evidence 终跑账逐数字一致（4,009P / fail 1,577 / timeout 7、
GAINED 五页同构），provenance 缺口就此闭账：g33 终值定谳为提交态真值。

## 八套件电池（同二进制串行，逐套件 vs 权威参照）

| 套件 | g34 终账 | 参照（g32 提交态） | 判定 |
|---|---|---|---|
| css-color | 10,999P/20F（64 案） | 10,999 | 精确持平 |
| css-typed-om | 9,872P/2,487F（359 案） | 9,872 | 精确持平 |
| cssom | 1,968P/1,507F/4T（177 案） | 1,968 | 精确持平 |
| css-transforms | **231P**/506F/3T（107 案） | 229 | **+2 零回归** |
| css-motion | 105P/286F（45 案） | 105 | 精确持平 |
| css-transitions | **201P**/2,288F/19T（118 案） | 200P/1,720F/21T | Pass +1 零回归 |
| css-animations | **239P**/718F/6T（123 案） | 238P/717F/7T | Pass +1 零回归 |
| css-easing | 51P/105F（12 案） | 51P/105F | 字节级一致 |

css-transforms +2 归因（对 results-g32-syscolor-full 逐页 diff，REGRESSED=0）：
scale-parsing-computed 23→24、transform-with-sign-function 2→3——g33
rotate3d 四臂（parse_transform_number 数学函数形 + 分词器 depth-0 修复）
红利兑现，g33 首轮 229 为精修前二进制账。

transitions/animations 的 Pass+1 与 F/T 漂移定性（g21/g23 在册超时枚举
伪影族）：non-rendered-element-001 (3P,1T)→(4P,0T)、Element-getAnimations
(1P,1T)→(2P,0T) 为基线轮 Timeout 子测本轮完整枚举后真过——真实增益非伪
回归；event-dispatch.tentative (0P,1T)→(0P,0T+31F) 为基线轮墙钟截断未枚举
完的页本轮完整枚举（31 子测全录，Fail 总量 1,720→2,288 同族口径）。
Pass 集无一下降、无 Timeout 页翻 Fail——Pass/Timeout 双锚定零回归成立。

## 运行形态

systemd user service `zw-g34-final.service`（oneshot，MemoryMax=10G），
脚本 `.acceptance/wptfyi-css/run-g34-final.sh`：静树 release 重建 →
values 全量 → 八套件电池串行；transcript 唯一路径
`/tmp/zw-g34-{values-final,battery-<suite>}.log`，per-page JSON
`.acceptance/wptfyi-css/results-g34-final/<suite>/`。

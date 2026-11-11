# DC-2 组 E：color none 序列化保形 + lab 族 hue echo——双面 round-trip 幂等

日期：2026-11-11　批次：DC-2 color 组 E 残余批（e2final 后 168F 归因）　基线：results-e2final（css-color 10,842P/168F）+ probe-none 定向（wpt-data gitignore 面，证据走 transcript）

## 1. 钉面与归因（e2final 残余 168F 分类）

| 组 | 案数 | 失败形态 |
|---|---|---|
| rgb none | 16F | `rgb(128 none none)` 等声明写→gCS 链：门抢答落宿主读数（probe B1/B2 实证 `gcs=rgb(0,0,0)`），computed none 传播序列化永不触达 |
| relative hsl/hwb RT | 20F | round-trip 面（computed-testcommon helper 把 expectedValue 回设为 specified 再读）：% 形与裸形序列化不跟随输入形，s/l、w/b 槽 % 恒输出或恒裸 |
| lch/oklch RT | 6F | 声明面 hue echo 精度：decimal hue 需 8 位回显（6 位舍入毁幂等，RT `33.690066` 锚）；角度单位换算 hue 需 6 位（`1.28rad`→`73.3386` 锚）——单一判据无法双锚兼得 |
| mix 残余 | ~2F | 同根随三面修复翻正 |

其余 ~120F 为挂账桶（var-origin 相对色 15+2F / xyz 矩阵精度 5F / rgb 转义 2F / var 多 token 2F / cqw calc 1F / 系统色 27F / opacity calc 20F 等），本批不动。

## 2. 死点链定谳（活体探针 zz-probe-none，A/B/C 组 19 案）

1. **gCS 双臂门吃 store 串**：声明写面归一（`__zwColorNormalizeDecl`）已把 none 消解为 0——`__zwCssColorNeedsPlain(_irGV)` 判定吃到的 store 串恒 legacy 形，rgb-none 判 false 抢答落宿主读数；part01c `_zwPlainDecl` 同判据用影子原文，门臂漏了这一层。
2. **hsl/hwb 序列化不跟随输入形**：% 输入经 raw 影子取原文 parse（含 %），裸输入（RT 回设）无影子走 store 形——序列化器缺 srcBare 记录，槽位形恒定。
3. **声明面 hue 无单位信息**：parse 期 rad/grad/turn→deg 换算后清 `lc.u`，序列化侧永远看不到单位；而 RT 回设的 decimal hue 又必须 8 位回显才幂等——判据需在 parse 期落旗标。

## 3. 修复内容（part01 门两臂 + part11-color 旗标/echo + part11-computed-color srcBare/数组化/恒 8 位）

- **part01 gCS 双臂门 raw 影子优先**（getPropertyValue 臂 + camelCase 臂各 +10 行）：`_irGV`/`_irV` 判定前先经 `elt.style.__zwRawDecl(kp)` 取归一前原文，取到则覆盖判定输入——原文含 none 时 NeedsPlain 判 true 落 plain walk（computed 传播面），无影子走 store 零漂移；与 part01c `_zwPlainDecl` 同判据。try/catch 包裹，宿主面函数缺席时二进制行为零变化。
- **part11-color parse 期旗标**（grad/rad/turn→deg 换算处）：`lc.fromAngleConv = true`——序列化侧判据改为 `oc.fromAngleConv`。
- **part11-color 声明面 lab 族 hue**：`ov` 先 mod 360（`380deg`→`20` 锚），输出分流 `fromAngleConv ? _zwCSig6(ov) : _zwCSig8(ov)`——oracle 双锚兼容（direct `73.3386` 6 位 / RT `33.690066` 8 位回显）。
- **part11-computed-color hsl/hwb parse 臂**：`srcBare: [!(comps[1].pct != null), !(comps[2].pct != null)]`、hwb `[wBare, bBare]` 同型——% 输入有影子判 false、裸输入无影子判 true。
- **part11-computed-color `_zwCCSerialHslNone`/`_zwCCSerialHwbNone`**：bare 参数数组化 `Array.isArray(bare) ? bare : [!!bare, !!bare]`——relative 臂布尔输出恒裸（css-color-4 relative 输出恒裸形）、direct 臂槽位随原文形。
- **part11-computed-color `_zwCCSerialLab` hue**：删 Sig6 三元，恒 `_zwCCNum` 8 位回显 store 串——computed 面 hue 随 store，声明面 echo 已由 part11-color 承担。

零回归面：门臂只覆盖 NeedsPlain 判定输入（原值无 none 时影子原文与 store 串判定一致）；srcBare 只被 none 序列化器消费；fromAngleConv 只被 hue 输出判据消费；lab hue 恒 8 位的翻正面即 e2final 6F RT 幂等宗。

## 4. 共享树协作记录

part01.js 本批两处门块与组F shim 流的 position-area 臂（getPropertyValue 回声覆盖/camelCase 臂/白名单行）同文件在途——按 hunk 归属过滤暂存（Python 过滤脚本对当时 diff 重建，PATCH-OK），本提交只含色域门块，组F 臂留工作树归组F 流提交；组F 流活跃（证据文件与 corpus 跑同窗），master.md 批次链接待其文档提交落库后补记。

## 5. 终验（results-nonefinal，rebase 62426041f 上）

- **css-color 10,842 → 10,887（净 +45）逐页零回归**（JSON 逐页 diff 无翻转）。
- **css-values 2,213P 持平**（同口径）。
- **css-typed-om 9,868 → 9,872（+4）**——4 案为 grid 家族 var() 引用案，e2final 基线跑于 13535525b 落库前，本轮在库批兑现面，非本批刀位。
- 提交 1005b6129（原 f87c7e83f，rebase 压 62426041f 上）推远端 main；门禁 lei-pre-commit-guard PASS（3 文件 52 新增行零发现）。

## 6. 挂账（残余 124F，后续批）

var-origin 相对色 15+2F（入口经 part01c，避让组F 在途面）；xyz 矩阵精度 5F；rgb 转义 2F；var 多 token 2F；cqw calc 1F；系统色 27F；opacity calc 20F（style-system 域）。

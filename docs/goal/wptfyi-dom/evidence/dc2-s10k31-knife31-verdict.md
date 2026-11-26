# DC-2 切片 10 刀㉛ verdict——reactions attributeChanged 反射簇收口（customized-builtins 16 页 36F 全清）

**2026-11-26 · 记录腿 HEAD `cbcccab7c`（单提交刀体，rebase 于他流 html 批四十九 docs `c88801b7d` 之上；guard PASS 0 发现 0 提醒在案）**

## 结论

- **WC 语料净 +36**（F 1023→987 probe-excluded 口径）：customized-builtins reactions 16 页 36 行翻绿——Base/Button/Details/FieldSet/Label/Map/Meta/OptGroup/Param/TableCell/TableCol 各 2＋Canvas×4＋Image×4（crossOrigin·decoding）＋Meter×4（min·max）＋Data×2。
- **零新败**：k31 独有 Fail 行＝0（全语料 18,519 行逐行 comm）；他页 Fail multiset 零漂移；Timeout 20==20。
- **四邻电池**：domparsing **行级 SET-IDENTICAL**（222F/1313P/1T 逐字节）＋sanitizer-api **行级 SET-IDENTICAL**（50F/706P/1T）＋dom F/T **364/12 恒等**（+72P＝Range-mutations dataChange +50/replaceData +22＝master.md 碰撞记录在案停摆漂移族，非本刀）。
- gethtml（shadow-dom/declarative）P 3363→3317（−46）：巨页截断落位噪声（F 集恒等签名），与刀㉚ 同注记口径不计净增。
- **累计 84,126（距 ≥99,947 差 15,821）**。族内残面仅 media 28F（audio/video src·controls·preload·loop·autoplay·defaultMuted·crossOrigin——绕 set trap 入口更深的独立 face，下刀挂账）。

## 根因五站（码刀 4 文件 +106/−6）

| 站 | 根因 | 刀 |
|---|---|---|
| ① 映射缺口 | `_zwCeReflectionAttr` 不识 R3069 显式直列成员（name/label/size/href/as/crossorigin/span/scope/valign）＋open/decoding/crossOrigin → trap 头 `_ceSetAttr=null` → sink 双门跳过 | part03 catch-all（'value' 严禁全局映射——INPUT dirty-flag；crossOrigin→'crossorigin' 小写映射） |
| ② meter 双派发 | R4002 六属性分支内联派发＋min/max FLAT 映射 sink 再派发＝计数翻倍 | part04 `!_ceSetAttr` 门（内联只补未映射面 value/low/high/optimum） |
| ③ htmlFor 汇流断 | OUTPUT/LABEL 专用臂 handle 路径不设 moAttr → sink `if (moAttr)` 跳过 | part04 两臂 handle 路径补 `moAttr='for'` |
| ④ DATA 门缺 | value 分支 tag 门（BUTTON/LI/PROGRESS/PARAM/OPTION）无 DATA → 落 generic 臂零派发 | part04 门加 `|| _rvTag === 'DATA'` |
| ⑤ canvas 四联 | standalone 工厂早 return 跳过 generic cbi 升级（constructed 缺席）；工厂无创建印章；`_zwSetCanvasDim` 直写 handle 零派发且旧值写后读（恒等新值恒拒值真变门）；reactions log builder 读 `getAttributeNS` 工厂缺失＋width 为 handle-only 写不在 attributes list → TypeError 被派发 catch 吞／actualValue=null 断言败 | part06 canvas 分支补升级面（is 记录＋ctor 执行）＋part05 印章＋旧值先捕内联派发＋getAttributeNS handle-first（R5009 片 e 同款行使面补齐先例） |

## 验证面

- 探针链：probe2 定谳七面翻绿面（meter/decoding/href/scope）＋probe3 attr 写分型＋probe5/6 页面侧状态诊断＋probe8 内联复刻真页（复现 4F 并定位 actualValue 断言）；zz 探针页 9 件交付前全删。
- 判读纪律在案：k30/k31 语料指纹恒等（`@ 315976933870`）；行提取一律 `LC_ALL=C awk`（grep/ugrep 大日志静默截断 435/18.5k 实证踩坑复演）；probe 页在电池收集面内的污染以剔除页名口径处理。
- 调试插桩全剥：part03/04/05 零 `__zzK31Dbg` 残留（62 行插桩全清）；R98 派发点插桩系并行核验席（35eef10d）字节，红线对账闭合——其 solo 三跑五臂分类（α 映射缺名/α' meter 双派/β htmlFor/γ canvas/δ media 绕 trap）与本席定谳独立吻合。
- WC 电池墙钟 ~11min（12:03→12:14）。
- 质量门：lei-pre-commit-guard PASS（4 文件 106+/6−，0 发现 0 提醒）；fmt/clippy 不适用（纯 .js shim）；电池全腿走 make testharness-* 守卫入口。

## 残池移交

- **media reactions 28F**（HTMLMediaElement src/controls/preload/loop/autoplay/defaultMuted/crossOrigin）——35eef10d 观测「完全绕过 trap 入口插桩＝绕 proxy 面更深」，独立刀位。
- TT residual 101F 池（B 臂未动工：createHTMLDocument 5F/block-string-assignment 13F/svg-script-set-href 7F/script-enforcement-008/009 等）。
- connected-callbacks shadow host 簇 8F／Node replaceChild 跨文档 F（刀㉚ 移交原样承继）。

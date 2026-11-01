# M2 s222 — update-the-source-set `;encodings=` 空 parameter value 窄刀

日期：2026-11-02。切片号 s222（承 s221）。

## 入场定刀位

s221 下一步候选首选「update-the-source-set `__zwSourceSupportedType5277`
`;encodings=` regex 窄刀（1F）」——单点小刀。

## 勘根

s221 基线日志失败形态：`type='image/gif;encodings='`（空 parameter
value）expected data:,b but got data:,a——source 被判 unsupported 走 img
fallback。s221 基线中该文件 89 结果行唯此 1F；`;encodings`（无等号）、
`;encodings=foobar`（有值）、`;`（尾分号）三邻案全 Pass。

根因：part03.js `__zwSourceSupportedType5277` 参数段正则的值分支
`("[^"]*"|[token]+)`——quoted-string 交替 `[^"]*` 允许空、token 交替
`[token]+` 要求 1+ 字符，空 value 整体不匹配 → rest 不匹配 → return
false。

spec：WHATWG mimesniff §parse a MIME type——parameter value 为 collect
not-`;` 序列后去尾随空白，可为空串；quoted string 同理可空。HTML spec
supported media type 判定只看 essence。上游期望 supported。

## 施工（shim 1 文件 +4/−2，语料零改动，零 Rust）

- **part03.js** 参数值正则 token 交替 `+` → `*`（1 字符）+ 注释补
  mimesniff spec 链接与空 value 合法说明。

离线正则矩阵验证（10 形态新旧对照）：唯一行为变化即 `encodings=`
（拒→受）；`encodings` / `encodings=foobar` / 多参数 `a=b;c=d` /
quoted `x=""` `x="foo"` / 空 name `=v`（拒保持）/ 裸空白值 `a=b c`
（拒保持，mimesniff 实际允许 value 含内部空白——语料无此形态，无语料
驱动不扩面）全部不变。

语料普查：embedded-content 全域带参数 type 仅 4 形态（本文件 93–96
行），全期望 supported，无空值反例，零翻红风险。

## 验证

- shim 拼接 16 part → node --check SYNTAX_OK（120049 行）。
- 靶语料 solo：update-the-source-set **89/89 全绿**（0F/0T，
  `;encodings=` F→P）。日志 evidence/logs/s222-update-the-source-set-solo.log。
- the-img-element 目录级：853P/84F/17T，953 键 (key,state) join 对
  s221 基线**唯一位移 = 靶案 Fail→Pass**，lost 0 / gained 0。日志
  evidence/logs/s222-img-element-dir.log。
- embedded-content 全域 A/B（A = s222 修复后、B = s221 基线日志）：
  共享键 **1257/1257**，漂移恰 **1 F→P**（靶案），**P→F = 0**，
  Timeout 位移 0，键集差 0。行级 1068P/160F/30T → 1069P/159F/30T。
  日志 evidence/logs/s222-embedded-content-wip.log。
- zero-engine 全量 **3684P + 0F**（test-guard 包裹，与 s220/s221 基线
  持平）。日志 evidence/logs/s222-zero-engine-full.log。
- 纯 JS 切片（git diff --stat +4/−2）——fmt/clippy 不适用；release
  runner 重编译后跑靶（shim include_str 进二进制）。

## 挂账

无新记。承袭挂账不动：fail-to-resolve 4F（URL 解析器面爆炸半径大独立
勘根）；not-broken 2F（attr sizing/rect 面）；decode quick-attach 2F；
update-media/source-relevant-mutations 选源响应面余案；layout-engine
1px 钳（sizing.rs:2899-2900/3136 渲染流）；error.html 破图 alt 7F；
sizes-auto Timeout 68 案；parse-a-srcset URL serializer 1F；SVG
data-URL intrinsic 3F + img naturalWidth SVG viewBox 族 22F（decoder
耦合 renderer 流 ③A）；svg-img-density-vs-zoom 7F；decode 2F；
lazy-loading 族 ~24F；part06 `_zwFieldsetDisableFocusReset`（s215）；
part03 `_zwMElCheckValidity` FORM 短路（s210）；onblur flake 族
（s209~）；beforeinput.tentative；非 canvas 面一般渲染门
FIXME(rendering)（s218）；flat tree slot FIXME(flat-tree)（s218）；
modern API img 树变更面余项（s221）；TreeMutated IMG 移除臂（s221）；
用户门禁三项（GB-20261031 customizable select 177F / form-nameditem
override builtins / form owner table 缓行族）。

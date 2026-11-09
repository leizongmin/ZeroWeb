# fb qV/qI 读面刀（2026-11-10）

## 刀体

- **part01g `__zw_ce_fb_value` 重写**（此前 `b1 !== b2` 元素同恒返 ""）：
  - 同型多块按 tag 相等读块名——`<p>[foo<p>bar]` qV-before 录音 "p"（52 行实证）；
  - 宿主尾块之后尚有可见 FB 块时读 ""——`<p>[foo<p>bar]<p>baz` 录音 "" + qI=false（formatblock.js :708，范围面上下文敏感非端点语义；后缀扫描 `_zwFbqHasVisBlockAfter`/`_zwFbqSubtreeHasVisBlock`，table 族 veto 同读面隐形域）；
  - 异型多块读含两端的最近 FB 块公共祖先 `_zwFbqCommonVisBlock`——`<dl><dt>[foo<dd>bar]</dl>` qV-before 录音 "dl"（dt/dd 均为 FB 块名停于端点，Chrome 读共同块 dl；fbVals=div/p 四条目 qV 全 "dl"）；宿主内无公共 FB 块祖先形（`<div>[foo<p>bar]</p></div>`、`<h1>[foo</h1><h2>bar]</h2>` 等）录音 ""。
- **part01g `__zw_ce_fb_indeterm` 新增**：原始端点 FB 块恰单侧有块、或异型且无公共祖先 → true；同型多块/单块/异型爬达共同祖先 → false。语料 qI true 16 行中 14 行由本判据覆盖；`<dl><dt>[foo<dd>bar]</dl>` 跨块形 fbVals=div 条目 qI 录音 **false** 与 fbVals=p 条目录音 **true** 同形互斥＝录音会话顺序泄漏签名（见 evidence/2026-11-08 顺序泄漏链先例），判据取 fbVals=div 侧（公共祖先存在＝值可判定→false），残留 2 行维持红挂账。
- **part06 `queryCommandIndeterm` formatblock 臂**（此前无臂恒 false）→ `__zw_ce_fb_indeterm`。

预验判读域真值表全量提取（363 条目 qV/qI before/after 对 shape 分组）佐证规则：dt 单块 "dt"/dd 单块 "dd"（爬升不停于中间名）、`<blockquote><p>[foo]<p>bar</blockquote><p>extra` 单块 "p"、`<section><hgroup><h1>[foo]</h1>…` 单块 "h1"（hgroup/li 不停爬升既有语义保持）。

## 预验（release 二进制直验，脏树测量域）

- fb 全页 **5007P/39F**（基线 fbr3 4897P/149F）；行级 flipdiff：**F2P=110 / P2F=0** / lost 0 / new 0。
- F2P 断言面分布：qV before ×56（52 p/p + 4 dl 跨块）+ qV after ×40 + qI before ×14——flip 全落刀域，compare/ret/dps 面零触碰。
- 残红 39 分类（全部既有红，无本刀新增）：apply 侧包裹形族 ~32 行（color:blue 样式携带 ×8、xmp/listing 包裹 ×10、table tr 跨胞 ×4、div>p ×2、blockquote 嵌套/拆分 ×2、hgroup ×1、h1<br> 合并 ×2、del ×1、`<p>`→h5 dps-div 状态交互 ×1、qV-after 关联读 ×若干）、dps-qV 面 ×5（既有挂账）、qI 泄漏签名 ×2（刀内明示挂账）。

## 门禁批

批 batch-fbq-read-gate-v1（锚 head **20fd3eb2a** + 八点 md5：part01g `5b1990ec` / part06 `0a779fd7` / bridge `b6b9a516` 新锚，五点沿 v3 恒一；特征串 fbqRet2×3 + `__zw_ce_fb_indeterm` 3/1 + 读面辅助×5；ensure_solo + flock）：**ALL-DONE 2026-11-10 05:42:24 全绿**（04:36:48 发射，66 分钟，solo 窗）。

- probe：f6bwin 2092P/0F + k11 224P/17F judge 零漂移。
- fb 主门：5007P/39F（期望恒一）；judge vs fbr3 基线 **F2P=110 / P2F=0 / post-only 0 / lost 0 VERDICT OK**；行级 flip 白名单 110 全命中 `queryCommand(Value|Indeterm)("formatblock") before/after`（56+40+14 断言面分布与预验恒一）。
- mt：20182P/2F judge 键级零翻越 VERDICT OK（残 2F 维持 X37 双键除名）。
- 单文件腿（vs F1' 终态零漂移）：iol 1759P/2F、iul 2061P/4F、ip 6989P/206F、ilb 1658P/33F、ihtml 1446P/119F；exec×3 行级恒一（1332P/474F、0P/1F、5P/1F）。
- enginelib：failed=7 名册双门恒一（6 先例 + r2967 外部债）；battery 七腿 420/880/1104/552/3456/2607/1455 全同数 F=0 T=0；binary sig 全在产物 + fbqRet2×3 计数恒一 + `__zw_ce_fb_indeterm` 在产物。

## 挂账

- apply 侧包裹形族（compare innerHTML + 关联 qV-after）：color:blue 样式携带、xmp/listing、table tr 跨胞、div>p、blockquote 嵌套/拆分、hgroup、h1<br> 合并、del、p→h5 dps-div 交互。
- dps-qV 面 5 行（既有）。
- qI 泄漏签名 2 行（dl 跨块 fbVals=p 录音 true vs fbVals=div false 同形互斥）。
- mt 残 2F 维持 X37 双键除名。

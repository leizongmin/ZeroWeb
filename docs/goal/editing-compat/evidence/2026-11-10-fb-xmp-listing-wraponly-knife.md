# fb apply 侧 xmp/listing wrap-only 块刀（2026-11-10）

## 刀体

- **Chrome delta（wrap-only 块）**：formatBlock 将 xmp/listing 视为整体 wrap 单元——不作 single-line container 解包。语料录音：`<xmp>[foobar]</xmp>` + <p> → `<p><xmp>[foobar]</xmp></p>`（:633/:638）、`<xmp>[foo]</xmp>` + <div> → `<div><xmp>[foo]</xmp></div>`（:1774/:1781）、listing 同形（:623/:628）——十行 compare 全形。spec 算法 unwrap 臂此前把 xmp/listing 当 SLC 解包产出 `<p>foobar</p>`（元素销毁），qV-after 因树异恒错。
- **part01g**：`_zwFbqIsWrapOnlyBlock`（xmp/listing 名单）三接点——nodeList 收集门单独放行（xmp/listing 非 p 允许子，spec 判据收不进）；unwrap 门豁免（`_zwIparaIsSingleLineContainer && !_zwFbqIsWrapOnlyBlock`）；聚合停止条件仍用全表（xmp/listing 阻断连续 sibling 并入同一 sublist）。shared 谓词 `_zwIparaIsSingleLineContainer`（insertparagraph 用）不动。
- 读面零改动：qV-after 爬升 xmp（非 formattable 跳过）→ p → "p" 既有语义自然命中。

## 普查（fbr4b 残 39F 全量分族）

- **xmp/listing ×10**（本刀域）：compare ×9 + listing qV-after ×1（另 xmp dps-qV-before ×1 录音伪影不动）。
- **color:blue ×8**：**发现数据丢失级缺陷**——`<div style=color:blue>[foo]</div>` + fb <p>：三 execCommand 全返 true 但 testDiv 清空、OUTER 无溢出（fbw4 探针 STEP 级追踪：STEP2=formatblock 步 TREE 即空、SEL 塌 `testDiv@0`）。根因假说：SLC 解包后 restoreValues 检得 foo 继承 forecolor=blue 而祖先链无载体 → forceValue → `_zwVaWrap`（creator=null 无 sibling 合并时早退 null）链条搁浅内容。**下一刀授权令**：数据丢失优先于兼容面。
- blockquote 族 ×4（嵌套 rename、相邻聚合、hgroup 定位——GOT 三形均异且机制各异：同 tag veto 祖先爬升过度否决 / 聚合臂 / fixDisallowedAncestors hgroup 内容模型方向）。
- table tr data-start qV-after ×4、div>p ×2（Chrome 多块 rename 形怪树）、qI 泄漏签名 ×2（既有挂账）、p→h5 dps-div ×1、cE=false span 边界 ×1（entry 362 守卫形）。
- 录音伪影单行 ×4（dps-qV-before 面跨案携带态签名：xmp/quasit/p[foo<br>]/div[foobar]）+ del qV-after ×1 待归类。

## 预验（脏树测量域）

- 探针页 zz-probe-fbwrap.html（30 形 = 39F 去重 (html,cmds) 对）：修前 compare 17F→修后 9F，xmp/listing 8 对全绿（GOT=`<p><listing>foobar</listing></p>` 形）；残红 = blockquote/div>p/color:blue 族（非本刀域）。
- fb 全页 **5015P/31F**（基线 fbr4b 5007P/39F）；judge vs fbr4b **F2P=8 / P2F=0 / lost 0 / post-only 0 VERDICT OK**——8 flip 全为 xmp/listing compare innerHTML 行。
- listing qV-after 残红 1 行（dps=p :2028）：**窗界跨页伪影签名**——断言行横跨 ?1001-2000/?2001-3000 变体边界（同 h5 残红先例），同形 dps=div 侧（entry 124）同树全绿、探针隔离 QVa="p" 全绿，判读域外维持挂账。

## 门禁批

批 batch-fbwrap-gate-v1（锚 head **d599989af** + 八点 md5：part01g 新锚 `179ffca2`、part01 `308b4a44`、part06 `321fca20`、bridge `b6b9a516` 沿推送树，其余四点恒一；特征串 fbqRet2×3 + fb_indeterm 3/1 + wrap-only×4；ensure_solo + flock）：**ALL-DONE 2026-11-10 07:27:22 全绿**（06:20:45 发射，66 分钟，solo 窗）。

- probe：f6bwin 2092P/0F + k11 224P/17F judge 零漂移。
- fb 主门：5015P/31F（期望恒一）；judge vs fbr4b **F2P=8 / P2F=0 / lost 0 VERDICT OK**；行级 flip 白名单 8 全命中 xmp/listing compare innerHTML；幂等 judge vs fbw 预验 F2P=0 P2F=0。
- mt：20182P/2F judge 键级零翻越 VERDICT OK（multitest 语料 xmp/listing 零暴露）。
- 单文件腿（vs F1' 终态零漂移）：iol 1759P/2F、iul 2061P/4F、ip 6989P/206F、ilb 1658P/33F、ihtml 1446P/119F；exec×3 judge 零漂移。
- enginelib：failed=7 名册双门恒一；battery 七腿 420/880/1104/552/3456/2607/1455 全同数 F=0 T=0；binary sig 全在产物 + wrap-only ×4 计数恒一。

## 挂账

- **color:blue 数据丢失族 ×8**（升级：非兼容面而是破坏性缺陷，下一刀）。
- blockquote 族 ×4 / div>p ×2 / table ×4（机制各异待普查定簇）。
- qI 泄漏签名 ×2 + 录音伪影单行 ×4 + listing 窗界伪影 ×1 + del qV-after ×1（挂账除名待定谳）。
- mt 残 2F 维持 X37 双键除名。

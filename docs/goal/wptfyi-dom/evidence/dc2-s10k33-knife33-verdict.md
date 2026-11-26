# DC-2 切片 10 刀㉝ verdict——IAH/outerHTML handle 域 TT 簇收口（TT +17＝5 页 17 键纯翻面）

**2026-11-26 · 记录腿 HEAD＝k32 码刀 `9ae93fd71` 之上；修改 4 文件（engine rust ×2＋shim js ×2，+173/−7），fmt 干净（新臂 rustfmt 形）、clippy `zero-engine --all-targets` 0 警告**

## 结论

- **TT 语料净 +17**（真 A/B 定谳：noknife 全量 vs knife 全量，双侧 fresh rebuild）：`block-string-assignment-to-Element-insertAdjacentHTML` 7＋`block-string-assignment-to-Element-outerHTML` 6＋`block-string-assignment-to-Element-setHTMLUnsafe` 2＋`Element-outerHTML` 1＋`Element-insertAdjacentHTML` 1＝**17 键 Fail→Pass 逐键 1:1 纯翻面（multiset 闭合）**，NEW 全 Pass、GONE 全 Fail、零附带。trusted-types-reporting 双页恒绿。
- **domparsing +1**：`outerhtml-01.html` 页级 Fail→Pass——页面唯一断言恰为本刀 ③ 修复面（`document.documentElement.outerHTML` 赋值须抛 `NO_MODIFICATION_ALLOWED_ERR`，spec DOM-Parsing 步骤 4）。
- **零回归**（详见下「拦截假警报考据」）：sanitizer-api swt 16 行零翻转（rv/ab/r3 三源独立一致）；dp 除增益行 SET-IDENTICAL；dom/wc 见四邻节。
- **累计 84,172（距 ≥99,947 差 15,775）**＝84,154＋TT 17＋dp 1（页完成度 P 漂移带不计，k30 截断带先例口径）。

## 根因四站（码刀 4 文件）

| 站 | 根因 | 刀 |
|---|---|---|
| ① IAH 兄弟位 sel 父＋M 域解析子静默丢 | handle-only 目标（createElement 链）`insertAdjacentHTML('beforebegin'/'afterend')`：`_zwFragmentAdded` 页侧解析产物无 handle，per-kid `parent.insertBefore` 落 s165 分支发 `child_handle: undefined` wire → host handles 表 miss **静默丢**（TT 语料 block-iah test 1 `previousSibling` null 钉面） | 新原语 `InsertAdjacentHtmlOnHandle`（engine enum＋apply 翻译臂/应用臂＋callbacks 注册）＋shim 兄弟位宿主原子落位（`__zw_insert_adjacent_html_on_handle`）：宿主按 handle 解析目标、`insert_adjacent_html` parent 遍历定址，与 sel 面同一函数路径 |
| ② TT/sanitizer 解析上下文保真 | 宿主路由绕页侧解析＝绕 default policy `createParserOptions` 链（sanitizer-api sethtml-with-trustedtypes 16 面） | 宿主路由置于刀⑩ 管道**下游**（`__zwSanProcessParserOptions`＋`__zwSanSanitizeString` 先产净串再送宿主）——宿主解析收净串，消毒语义零旁路。门＝父有 sel；handle-only 父落既有 per-kid 回落分支 |
| ③ outerHTML parent=Document 缺抛错 | spec 步骤 4 NoModificationAllowedError（`documentElement.outerHTML` 赋值，outerhtml-01 钉面）；旧版静默尝试整根替换 | part04 outerHTML setter 臂 TT 管道后补 Document 父判定抛错（步序在 TT 步骤 1 compliantString 后——TypeError 优先面既有臂不变） |
| ④ outerHTML 锚父容器 afterend | sel 父＋handle 自身：旧 `__zw_insert_adjacent_html(parentSel,'afterend')` 锚**父容器**＝片段落父外（block-outerHTML test 1 `container.innerText` got "\n" 钉面） | 改 on-handle afterend 于 d 原位插（先插后摘语义保持）＋父桶对冲记账（removed [self] 记父桶——`_zwPendBucket` 父视图剔自身，否则被替换元素 stale 残留 fused 视图、测试侧 while(firstChild) remove() 死转） |
| ⑤ innerText 空块换行 | 块级子恒插 '\n'——空贡献块（空 div）无 line box 不产换行（Chrome `<div><div></div></div>` innerText === ""，TT block-outerHTML 3/5 残留 innerText 钉面） | part01 collect 臂换行延迟到收集后（`_out.length` 前后差判定），非空贡献字节序不变 |

## 回归拦截假警报考据（接管收口承）

核验席 35eef10d 06:0xZ 拦截（SAN GONE 17/NEW 38＝swt 16 行 P→F）判于 `battery-k33-sanitizer-api.log`（14:12）——**旧迭代二进制**（构建行 0.18s cached；写者 14:12–14:23 窗内已迭代修复）。解除证据三源独立一致：rv-san（14:23）/ab-san（14:57，genuine noknife 重建 N 腿系）/r3-san（16:09，终树 fresh rebuild）**swt 16 行零翻转**；svg-animate 15F/7P vs k32 Timeout＝页完成度漂移（真 noknife N1/N2==knife solo/K2 字节恒等＝环境停摆族非刀）。伪 A/B 教训在案：ab-san-knife2/ab-san-noknife 双 0.19s cached＝同二进制跑两次，md5 恒等无证明力。

## 真假 A/B 双锚判读（字节绑定纪律）

- **真 A/B**（k33-ab-driver.sh，树 md5 断言＋stash 过境往返）：N3 noknife TT 全量（fresh rebuild）vs **K1 knife TT 全量（fresh rebuild，15:39 Compiling zero-engine）**＝GONE 17 全 Fail→NEW 17 全 Pass 逐键配对。K1==rv-tt（14:32）SET-IDENTICAL＝14:32 后树 TT 域零变化。
- **终树 fresh 电池**（r3，树 md5 `c88b2722`＝fmt 修复后终版断言；r2 因批中 fmt 修复混合二进制按纪律停杀重射）：san GONE 1/NEW 22 全 svg-animate 完成度族、swt 零翻转；dp GONE 1 Fail/NEW 1 Pass＝outerhtml-01 增益；wc/dom 见下。

## 四邻电池（r3 fresh 终树）

- sanitizer-api：除 svg-animate 页完成度漂移（k32 页级 Timeout 未注册 vs r3 15F/7P 完成态，真 noknife 字节恒等定谳非刀）行级 SET-IDENTICAL，swt 16 面 0 翻转。
- domparsing：SET-IDENTICAL＋outerhtml-01 Fail→Pass 增益 ×1。
- web-components：**F 956→955 zz 口径（−1＝`registries/Element-innerHTML` 1 测翻绿＋页完成 +2P——k32 页 9F+2P+1T 部分注册 vs r3 8F+4P 完成态，增益向）＋T 20==20 恒＋零 NEW Fail**；gethtml +228P 截断带（F 恒 3/T 恒 1）；event-path +2 NotRun 停摆带（F/T 面恒 9F+1T）；zz-k32-probe1 探针页已删（−1F 记账）。
- dom：**F/T 364/12 恒等零新败**；+131P 全 Range-mutations 停摆族 P 漂移带（GONE=0）。

## 残池移交

- TT residual 池余 84F（101−17）：createHTMLDocument 5／svg-script-set-href 7／script-enforcement-004 4／policy-without-return-value 4／script-enforcement-006/007 各 3＋008/009 各 3（importmap/module 新残面）／block-text-node-insertion 3／DOMParser-parseFromString 3／其余散布。
- 刀㉚ 承继不变：connected-callbacks shadow host 簇 8F／Node replaceChild 跨文档 F。
- 近邻缺口在案（corpus 无钉）：IAH 自容器位（afterbegin/beforeend）handle 臂 M 域解析子宿主分裂＝既有 accepted 形态本刀未扩；`muted` IDL set handle 路径漏 moAttr（刀㉜ 同款一行为补）。

## 过程事件（防重走）

- 写者 cfcb83f0 15:35 死于 429 十连耗尽（56 turns），A/B 驱动 K1 腿成孤儿续跑完成＋pop 复位（树 md5 断言双过）；本席按 06:0xZ 接管条款代跑收口。
- r2 电池批中 fmt 修复（新臂 rustfmt 形）＝混合二进制，停杀重射 r3（门禁批冻结先于发射纪律）。
- 判读件：/tmp/s10k33/（ttab-*.rows、tfinal-*.raw、r3-*.log/rows、k33ab/driver-r3 日志）。

# nested ③ qState-after ×7 + blockquote compare ×4 静态归因（零源码修改）

日期：2026-11-06。基线 = rtl 刀后 fail 面（iol 8F / iul 12F，`batch-rtl-{iol,iul}-post.log`）。纯语料/日志静态归因，无门禁批；修复方向供下轮选族。

## nested ③ 族（iol ×3 + iul ×4）= 两亚族

### 亚族 1：读面同型嵌套 4 行——维持 P10 定谳不追

`:448`/`:478`（iol）+ `:233`/`:263`（iul）qState after，expected false / got true。树（compare 全过）= 选区跨外层换型壳 li 与同型嵌套壳（`<ol><li>foo<ol>` 形），两端点最近列表均本命令型 → `_zwListQueryStateBool` 端点直读返 true；仅罚异型嵌套（`li.querySelector(opp)`），同型嵌套不翻。

复核 P10「语料内部矛盾」实证成立：iol `:343` 同树同选区 `<ol><li>[foo<ol><li>bar]</ol>baz</ol>` **before** 录 qState=**true**/qIndeterm=false，`:478` 同形 **after** 录 qState=**false**/qIndeterm=true——纯 (tree, selection) 读面不可双全，P10 取 before 全保的取舍维持。4 行挂账不追，除非出现判别信号（如 Chromium 源码级命令史态证据）。

### 亚族 2：出口选区恢复 3 行——下轮主刀

`:393`（iol）+ `:428`/`:433`（iul，dps=div/p 双变体），Chrome 终态选区跨壳位/跨界，本实现终态选区停位不符：

- `:393` `<ul><li>foo<ul><li>[bar]<li>baz</ul><li>quz</ul>`：compare PASS（树 = `<ul><li>foo</li><ol><li>[bar]</li></ol>…`），expected qState=true / got false。单壳无嵌套面读面平凡真 → got false 唯一解释 = 终态选区停 ol 外。**勘误**：sr1 时代「:393 形 exec 实跑窗口内读 true → 切片跨窗伪影除名」不成立——exec 时点读 true 与 harness 终态读 false 并存恰证出口臂（收钳三分支全 miss → fallback (host,len)）事后改位，缺陷真实、确定性复现（br/rtl 两代批恒红）。
- `:428`/`:433` `<ul><li>foo<ul><li>b[ar</ul><li>b]az</ul>`：compare PASS（树 = `<ul><li>foo</li><li>b[ar</li></ul><div>b]az</div>`），expected qIndeterm=true（mixed：li+div 跨界）/ got false，expected qState=false / got true。got 组合唯解 = 终态选区整体停列表内（两端点最近列表均 ul）。

修复方向：sr1 出口收钳域（twin-shell 重建钳覆盖 :38 吸收形，未覆盖 nested-split lift `:393` 与 lift+div-convert `:428/:433` 两臂的终态选区落位）；下轮复用 sr1 DBG 五时点插桩定位 fallback 落点。

## blockquote compare 族 ×4（iol/iul 各 B1/B2 镜像）

got 树已捕获（`batch-rtl-iol-post.log`）：`[foo<blockquote>bar]</blockquote>baz` expected `<ol><li>foo</li><ol><li>bar</li></ol></ol>baz` / got `<ol><li>foo</li></ol><blockquote>bar</blockquote>baz`；`[foo<blockquote>bar]<br>baz</blockquote>` expected `…</ol><blockquote>baz</blockquote>` / got `…<blockquote>bar<br>baz</blockquote>`。

机制：跨 blockquote 边界 straddle 选区——foo 正常入外层 ol，但 blockquote 内 bar 块完全未转换（壳不解包不拆分，B2 边界 br 亦未消费）。绿兄弟行（选区全在 blockquote 内，`:508`/`:528`）走 blockquote 内转换臂正常，故缺口 = straddle 面：Chrome 契约 = straddle 时 blockquote 解包、转换 li 落外层壳内嵌 ol、尾段残留 split blockquote（B2）、提取边界 br 消费。修复方向：part01e enable 块收集/转换对 blockquote 容器内块的进出处理（br 刀行粒度预遍同域新臂）。

## 优先级建议

blockquote ×4 > 出口选区恢复 ×3 > 读面 4 行（不追）。blockquote 族一臂清 4 行且与 dps/br 刀同域（enable 块收集），复用面最大；出口选区恢复 3 行 DBG 插桩成本高但 sr1 机制复用；读面 4 行维持不追。

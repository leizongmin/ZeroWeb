# indent 邻居合并去重刀（R16 A 族）与残 14F 普查（2026-11-18）

## 残红普查（r15-indent.log 17F → 六族）

r15 门禁后 indent 残 17F（全 compare innerHTML 型）分类：

| 族 | 键数 | 形 | 机制 |
|----|------|----|------|
| A 邻居合并复本 | 3 | `[foo]<blockquote>bar</blockquote>`／`foo<bq>bar</bq>[baz]`／`<bq>foo</bq>[bar]<bq>baz</bq>` | got 出现邻居壳内容**复本**（双 bar/foo），Chrome 录形单块合并 |
| B styled wrapper 内再包裹方向 | 4 | `margin-right:0 dir=ltr` ×2＋webkit-indent-blockquote ×2 | got 新 bq 嵌既有壳**内**；录形新壳包**外** |
| C list 嵌套域 | 4 | `<ol><li>foo<ol>…[baz]</ol>` 等 | 整列外包新 ol／baz 移入既有内层 ol／whitespace 位 `<br>` 注入 |
| D p 成员邻接合并缺失 | 3 | `<bq><p>foo</bq> <p>[bar]` 等 | got 各自独立包裹；录形 p 并入相邻 bq（空白文本保留） |
| E 块容器包裹方向 | 2 | div×p[dir=rtl]（同 B 根）＋table data-start/end 整表包裹 | got 块内嵌 bq／逐 td 包裹；录形壳包容器外／整表单枚 |
| F li 内尾 br | 1 | `<ul><li>a<br>{<br>}</li>…` | 选区内尾 br 被摘（r14 同 got 恒一＝既有红非 R15 刀回归） |

## A 族根因链（TEMP 探针取证，判读后删除零残留）

1. got 形审计：r14→r15 红 17 键 got/exp 逐键 diff **零变化**（翻绿恰 4 刀面键）
   ＝R15 br 边界刀在红键域零扰动。
2. 探针（`foo<blockquote>bar</blockquote><p>extra` + indent）：**JS 视图树已
   复本**（`<blockquote>foo<br>bar</blockquote><blockquote>bar</blockquote>`），
   host 重放无辜（单条 SetInnerHtml 变体携带复本串）＝纯 shim 侧缺陷。
3. TRACE（临时埋点）：`_zwJusWrapViaHtml` nextCrit=true（次邻裸 blockquote 合
   simple indentation element 判据）→ SET html 即复本串。
4. 根因：邻居复用臂展开形嵌入（`pHtml.slice(0,pClose)+…+pHtml.slice(pClose)`
   串化注入）后，**prefix/suffix 仍按原始 kids 全量累计**——被复用邻居恰在
   `firstIdx-1`/`lastIdx+1` 位，其 outerHTML 已进 prefix/suffix，再拼展开形＝
   内容复本两次。prevSib 臂同洞（A3 got `<blockquote>foo</blockquote><blockquote>
   foo<br>bar</blockquote>…` 与 prefix 复本逐字吻合）。

## 刀体（part01c.js `_zwJusWrapViaHtml`）

1. **去重**：prevSib 臂 prefix 改按 `kids[0..firstIdx-1)` 重算（排除邻居位）；
   nextSib 臂 suffix 改按 `kids[lastIdx+2..]`（suf）。
2. **次邻 composition 级并入**（DOM 臂 post-move 次邻合并的串化镜像）：prevSib
   臂选中后，若 kids 尾邻（`kids[lastIdx+1]`）亦合 criteria，则将其 inner 并入
   组合串（br 垫界条件与 DOM 臂同款：前邻尾内联＋次邻首内联＋尾非 br），suffix
   同步排除该位。不走 rebuild 后 DOM move——fresh 重建孪生裸 blockquote 无
   handle 且 chapter 失唯一，wire 面静默漂移（R5356b/2026-10-31 唯一性门先例）。
3. **孪生定位位修正**：prevSib 臂去重后展开形包落 `firstIdx-1` 位（原 buggy
   布局下 prefix 含邻居恰好占 `firstIdx` 位），`posW` 按 `reusePrev` 分叉。

## 验证

- part137 钉测 3 形（next-reuse/prev-reuse/prev-next-merge）逐键断言＋复本块
  反断言，1 passed。
- 全 lib **4164P/0F**（+1=本钉测参与跑）；`cargo clippy -p zero-engine --lib
  -D warnings` 干净；`cargo fmt -p zero-engine` 零 diff（workspace fmt 残面在
  style-system registry.rs＝渲染流域，非本流触碰）。
- indent 语料 smoke（r16-smoke-indent.log，bin 6689b49a…）：**1315/14/0**，
  failset 差量恰 −3＝A 族三键（idx37/40/41），零新增红（comm 对 r15-indent.log
  实证）。

## 门禁批

zw-r16（发射后补记）：期望 indent 1315/14 fresh 锚＋fb 5043/3／fs 2419/3／
img 962/0／event 176/4／ie SET-IDENTICAL／mt 10090/2 X37 恒一（ViaHtml 复用臂
为 indent/list/ip 多命令共享面，mt 为共享面回归哨）。

# DC-2 · d15 批：strike S2a sibCrit 同 tag 名门 + P5 td span retag（retag 后选区端点重锚）

日期：2026-11-13 立批（上轮 429 中断），2026-11-14 接续完成。
前置：d14 批（[strike S1 内部界 peel + sub/sup 残段保面 + s 壳收编](2026-11-13-dc2-va-peel-interior-strike-shell.md)）。
基线：face 1 六文件 22F（bold 8 / italic 3 / underline 4 / strikethrough 7 /
sub 0 / sup 0），哨兵 34 名集，钉测 7/7。

## 结论先行

face 22F→20F（**F2P=2 / P2F=0**）：strikethrough 7F→5F（S2a + P5 两案转绿），
其余五文件断言级 fail 名集逐文件恒等。两刀全落 `js_dom_shim/part01d.js`
strike ADD（newValue 'line-through'、css-false）门内：

1. **S2a sibCrit 同 tag 名并拢门（:1155）**：`fo[o<s>b]ar</s>baz` + strike
   语料期望 `fo<strike>o</strike><s>bar</s>baz`（data/strikethrough.js:635）
   ——fresh strike 不并入相邻 `<s>`（tag 名异，specified/effective 同为
   line-through）。与 d10 bold/strong 排除门（:1148）同款规则面；underline
   对称行录形相反（underline.js:698 td decl span 就地并入），门限
   strikethrough。
2. **P5 td decl span retag 臂（:3010 段）**：`fo[o<span
   style=text-decoration:line-through>b]ar</span>baz` + strike 期望同 S2a
   录形（strikethrough.js:705）——部分覆盖 td span 壳原位 retag 裸 `<s>`
   （createElement 干净壳迁子节点，不走 setTagName 以避 attrs 随迁后 CSSOM
   pending 值污染 specified 判定），选区侧文本走 fresh strike 分离外裹
   （S2a 门阻并入）。全含形不入本臂（contRt 排除）。

## retag 后选区端点重锚（本轮根因定谳）

retag 的子节点搬运走 `_zwJusMovePreservingRanges` + 宿主 insertBefore/
removeChild。**multitest 序列上下文**（face `?2001-last` 窗）下，splitText
重绑后 range 内部位移槽与 (container, offset) 面值脱钩，move 补偿把 start
端点折叠进随迁节点——探针面包屑实证：retag 后 `s=e=#b@1`、CONTAINED 集空、
apply 整段空转（got `foo<s>bar</s>baz`，`fo|o` 文本归并）。孤立探针同码
全绿不可复现（f6bwin 孤立/序列分叉同族现象）。

修复 = retag 前 `rng.setStart/setEnd` 快照同值重写（经公开 API 刷新位移槽
一致性）+ 重挂后按快照回写兜底。两写均为同值写，零语义变更。诊断技巧：
引擎侧面包屑经 `createTextNode` 挂 editing host，从 compare innerHTML 的
got 串面出——runner 无 console 桥，assert got 是唯一回传通道；pre-retag
append 本身触发宿主补偿重锚（Heisenbug 修面）即为此机制的反向利用。

上轮遗留：`/tmp/d15cmp/part01d-d15-backup.js` 存有 64 行 strike REMOVE
方向规则草稿（起界两侧 decoration 面本地判向，替代 fq.state 全 contained
普查），上轮已从工作树移除未验证。S2e（:640 `foo<s>ba[r</s>b]az` mixed
qS=false 而方向 REMOVE、期望 `foo<s>ba</s>rbaz`）与其同族，为下批
方向规则候选；本轮不落地。

## 账目

- face 1（六文件断言级，base=d14）：22F→20F，F2P=2、P2F=0。strikethrough
  2131+2P/7F→5F；bold 8F / italic 3F / underline 4F / sub 0F / sup 0F
  fail 名集与 d14 逐文件 diff 恒等。
- 哨兵电池（40 页 zz-probe-*）：fail 名集 34=34 恒等（本轮无 f6bwin 窗口
  级 Execution timeout 伪失败入账——上轮 sent3 的 2 条为墙钟截断，非断言）。
- 钉测：test-guard 包裹 `cargo test -p zero-engine --lib execcommand`
  （独立 CARGO_TARGET_DIR=/tmp/d13tgt）7 passed / 0 failed。
- 常驻探针：新增 `zz-probe-d15strike.html` 两 case（S2a + P5），2/2 绿，
  守 retag + 重锚组合面。
- selection 全语料未重跑：本轮两刀+重锚全在 strike ADD css-false 门内
  （其余命令不可达），Selection API 语料不经 execCommand 管线；d14 全
  selection 面名集级 P2F=0 基线维持。

## 残余账目（strike 5F）

- S2e 方向族：`foo<s>ba[r</s>b]az` compare innerHTML（ REMOVE 方向，
  :640）——方向规则草稿候选（上轮 backup）。
- M1 multipara：`<p>[foo</p><p> <span>bar</span> </p><p>baz]</p>`（与
  underline M 族同款多段面）。
- qState 三条：`<strike>foo[bar]baz</strike>` qS-after、`foo<s>[bar]</s>
  baz` qS-after + qS(stylewithcss)-before（切片伪失败族待复核）。
- bold 8F / italic 3F / underline 4F：d13 分类维持（b-in-i 块边界嵌套、
  cssFlag qS 族切片伪失败等）。

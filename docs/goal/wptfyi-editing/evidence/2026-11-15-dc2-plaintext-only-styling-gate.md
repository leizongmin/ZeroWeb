# plaintext-only 宿主 styling 命令门（d35/T 切片）——d31 unlink 臂 P2F=2 补修 + styling 全谱 216 F2P

日期：2026-11-15（d35 会话，接 d34 headEl 持真 title 切片）
范围：part06.js `_zwExecCmdStylingCmds` 表 + `_zwExecCmdFindPlaintextRootEditable`
根宿主判定 + execCommand 入口门 + `_zwQueryCommandState` enabled 门（四处，零 Rust 变更）
语料面：`editing/plaintext-only/styling-commands.html`（双 variant，主目标）、
plaintext-only 全目录守护、selection/editing 哨兵、input-events 全量、engine lib pin

## 结论

- `styling-commands.html?styleWithCSS=false/true` 双 variant **42P/108F ×2 →
  150P/0F ×2**（全谱 300 子测全 Pass）——d34 披露的 unlink "should not update
  the DOM" **P2F=2 清账**（d30 基线该两子测 Pass、d31 unlink 臂落地后翻红，
  stash 定谳见 d34 证据；门后回归 Pass，对 d30 净零不入 F2P 账）。selection
  全语料 vs d30 对拍：**F2P=290 / P2F=0**（实测，16:56 完成跑；证据初稿
  281 为预估，勘正于此）——216 = styling-commands + 65 = d34 账既有
  （del-html +20、IP-html +40、div-outside-body +5）+ **9 = delete.html
  窗口增益**（归因见下节），见下节对拍细账。
- 门判定取**根可编辑宿主**（Blink root editable element 语义）而非「选区在
  plaintext-only 元素内」：plaintext-only 嵌在 contenteditable=true 内不门
  （nested-with-contenteditable-true 语料 expect style edit），true-in-po 门，
  ce=false 截断爬升（true-in-po-in-false 不门），designMode=on 全文档富编辑
  不门（plaintext-only-in-designMode 3P/0F 恒）。
- 全守护面零漂移：unlink.html 391P/4F、bold.html 3039P/9F、
  zz-probe-d31-unlink 8P/0F、zz-probe-b1s3 4P/9F（eng/solo/diag 姊妹探针
  3F/8F/2F 恒）、del/IP-in-child-of-{html,head} 98P/16F + 128P/16F 恒、
  input-events 全量 223P/936F/3T 恒、engine lib 4033P/0F、clippy/format 干净。

## 门语义（Chromium 对齐，语料逐案定谳）

HTML spec：contenteditable="plaintext-only" 只允许纯文本插入类编辑
（insertText/insertParagraph/insertLineBreak/delete 族不受门）；styling 命令
整族 disabled——`queryCommandEnabled` false、`execCommand` 返 false、
零 DOM 变更零事件（styling-commands 语料 30 命令 × qCE/ret/DOM/input 四断言
面即该族全谱）。判定从选区 startContainer 爬升取**最高** contenteditable≠false
祖先为根可编辑宿主，其 attr 字面为 plaintext-only 才门：

| 形 | 根宿主 | 门 |
|---|---|---|
| po 直接在 body（styling-commands） | po | 门 |
| po in true/""（nested 第一环） | 外层 true 宿主 | 不门（style edit 生效） |
| true/"" in po（nested 第二环） | po | 门 |
| po in false | po（false 截断） | 门 |
| true in po in false | true（false 截断 po） | 不门 |
| designMode=on + po | —（designMode 快道） | 不门 |

## 四处修改（part06.js）

1. `_zwExecCmdStylingCmds` 表（`_zwExecCmdInputType` 后）：26 命令（语料
   30 项 = 26 命令 × 变体；foreColor 不在语料清单但与 backColor/hiliteColor
   同族同判）。
2. `_zwExecCmdFindPlaintextRootEditable(range)`（`_zwExecCmdFindEditingHost`
   后）：根宿主爬升 + designMode 快道，同构既有 `_zwExecCmdFindEditingHost`
   guard 模式。
3. execCommand 入口门（styleWithCSS 快照后、copy/cut/paste 分支前）：
   表命中且根宿主 plaintext-only → 零事件零 DOM 返 false（Chromium Execute
   门同型）。
4. `_zwQueryCommandState` enabled 门：表命中且根宿主命中 → enabled=false
   （supported 面不变——Chromium queryCommandSupported 对已知命令恒 true）。

iframe 子文档 execCommand 薄面（part05 B1 切片 1）不扩：该面只覆盖
defaultParagraphSeparator/styleWithCSS/createLink/delete/forwarddelete 五命令，
子文档语料无 plaintext-only 宿主（零消费面，FIXME 留档不扩）。

## 面账（d35 终态）

| 面 | 基线 | 实得 |
|---|---|---|
| `styling-commands.html?styleWithCSS=false` | 42P/108F | **150P/0F** ✓ |
| `styling-commands.html?styleWithCSS=true` | 42P/108F | **150P/0F** ✓ |
| `plaintext-only-in-designMode.html` | 3P/0F | 3P/0F 恒 ✓ |
| `nested-with-contenteditable-true.html` | 0P/56F | 0P/56F 恒（整文件 Enter/Actions 面 setup 级 Timeout，既有欠账非本切片） ✓ |
| plaintext-only 目录其余（insert*/delete*/paste） | — | 逐文件恒（见对拍 P2F=0） ✓ |
| `unlink.html`（editing-run） | 391P/4F | 391P/4F 恒 ✓ |
| `bold.html`（editing-run） | 3039P/9F | 3039P/9F 恒 ✓ |
| `zz-probe-d31-unlink` | 8P/0F | 8P/0F 恒 ✓ |
| `zz-probe-b1s3` | 4P/9F | 4P/9F 恒（姊妹 eng/solo/diag 3F/8F/2F 恒） ✓ |
| del-in-child-of-{html,head} | 98P/16F | 98P/16F 恒 ✓ |
| IP-in-child-of-{html,head} | 128P/16F | 128P/16F 恒 ✓ |
| input-events 全量 | 223P/936F/3T | 223P/936F/3T 恒 ✓ |
| engine lib（/tmp/d19tgt pin） | 4032P/0F（d34 账） | **4033P/0F**（+1 测试 = 04369d39a ip 垫刀批新钉，非本切片） ✓ |
| clippy -p zero-engine --all-features | — | 无告警 ✓ |
| cargo fmt --check | — | 过 ✓ |

## selection 全语料对拍（vs d30-selfull.json，交集法）

全账 **F2P=290 / P2F=0**（738 案、48,668P → 48,958P、Timeout 案集合
9→9 恒等）：styling-commands +216（本切片门）、del-html +20 / IP-html +40 /
div-outside-body +5（d34 账既有持位）、**delete.html +9（窗口增益归因）**。
全语料其余逐 (case, subtest) 恒等。

### delete.html +9 归因（04369d39a 窗口增益，非本切片）

9 个翻红子测全为 `<p><br></p><p><br></p>` / `<p><br></p><br>` 空垫段
delete/forwarddelete 跨块 join（d30 15P/1P → d35 10P/6F）。归因链：
①主题精确命中 04369d39a「ip 容器链垫刀——垫底 br 持行豁免」（15:08 提交）
——其 `_zwJusIsExtraneousLineBreak` 新增「块级父独子 br 持行」臂，正是
delete/forwarddelete join 后 `_zwJusRemoveExtraneousLineBreaks` 清理垫底
`<br>` 的共享判定（spec extraneous-line-breaks 同节）；旧免布局近似在该形
误摘 `<br>` 垫致 innerHTML 错账。②时间线：d30 快照 10:30 → 04369d39a
15:08 → 本切片对拍二进制 16:28 构建（树含该提交）；d34 对拍账 65 无
delete.html 项，证其对拍跑于 15:08 前（d34 face 产物 13:57 同向）——+9
首现于本切片对拍属窗口补账。③本切片 diff 仅 plaintext-only 门，delete
命令不在 styling 表，门判定路径与 delete 应用臂无交集。

## 工具与执行注记

- selection 通道 filter 为**纯子串**（`relative.contains`，无 `|` 分组——
  editing-run 通道才支持 pipe 组）；复合过滤需逐串跑。教训与
  `--format json`（不存在）vs `--json`（实际旗标）一并记录：误旗标使 filter
  吃 `"--format"` 串 → 零案匹配静默 exit 1，产物 0 字节——后台跑 JSON 前先
  小 filter 试跑验证旗标。
- 并行流（html-semantics）同 clone 在跑时，selection 全量 runner 以 /tmp
  拷贝二进制（zw-runner-d35/test-guard-d35）避让共享 build 面与进程名
  （2026-11-13 碰撞记录同款规避协议，未触碰对方进程）。

## 遗留挂账（不变）

- J2 跨 body 边界块合并切片（del-html 余 8F，doc 级 walk 块合并臂）。
- FIXME(d33) 跨文档收养缺口（IP-html 余 8F，引擎级 adoptNode）。
- FIXME(d24) unlink 余样式重裹 4F。
- tal 内壳形收敛（前置 d24 单一补偿面 + caret 引擎同位快照）。
- nested-with-contenteditable-true 整文件 Timeout（Enter/Actions 面，
  既有欠账——本切片门语义在该语料的 24 bold 案已按上表语义定谳，收敛
  待 Actions 键面）。

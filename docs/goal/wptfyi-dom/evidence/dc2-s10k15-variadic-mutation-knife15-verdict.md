# DC-2 切片 10 DOM 刀⑮ 判读证据——变参突变方法基座面（mixin 接收者语义＋union 转换）

**日期** 2026-11-25 · **判定** 刀⑮ 落地，TT 净 +180（542F→362F，Node-multiple-arguments 90+90 全清），五腿 join 零回归

## 刀面（4 文件，+236/−32，commit 39c739149）

1. `crates/engine/src/js_dom_shim/part04.js`：proxy get-trap 五方法脸（replaceWith/append/prepend/before‖after 共用臂）头加 R515 接收者改派守卫——`this` 为对象且 handle/sel 与闭包不一致→`this[prop].apply(this, arguments)` 重入 get trap 取接收者自身闭包（spec mixin 方法接收者＝`this`，https://dom.spec.whatwg.org/#dom-childnode-replacewith 族）；递归安全＝改派后接收者闭包自配对守卫 false；plain 接收者改派 plain 脸，extractor=receiver 与无绑定直调零变化。
2. `crates/engine/src/js_dom_shim/part03.js` Face A：`_mk`（append/prepend）与 `replaceChildren` 接收者＝`this`（回落闭包 target 容错）；union (Node or DOMString) 转换前置（nodeType 判据→node，else String()→createTextNode，ownerDocument 缺席回落 globalThis.document）＋R196 校验吃转换后数组；before/after 校验-only 桩升真插入（有父链 insertBefore，before 参数序保持／after 逐个前插于 `node.nextSibling`；无父链保留校验-only 旧语义）；replaceWith 镜像 spec concept-node-replace（pre-insert 群到 node 前→remove node）。
3. `crates/engine/src/js_dom_shim/part03.js` Face B：CharacterData.prototype.textContent 存取器（get this.data／set this.data=String(v)；spec https://dom.spec.whatwg.org/#dom-node-textcontent 继承面＋characterdata-set；own 成员遮蔽处同值零变化）。
4. `crates/engine/src/js_dom_bridge_tests/part161_s10k15_variadic_mutation_pins.rs`（新，86 行）：七钉常驻（提取方法对象 apply 接收者落位／before-after 参数序真插入／union 转换／原型 textContent 判定面＋读写／replaceWith spec 序／detached 校验-only 保留）＋`js_dom_bridge_tests.rs` include 接入。

## 根因链（probe_tmp_k15 定谳）

- 头①（主根因）：mixin 方法（replaceWith/append/prepend/before/after）返回函数闭包绑提取源 sel/handle，`this` 被无视——corpus 页提取 `container.replaceWith` 后 `.apply(inner/outer,…)` 操作全落 container（probe P1 sameFace=false 定谳：div/p query 产品与 createElement 产物不同脸；M 矩阵 extractor=receiver 30/30 全过纯属巧合掩盖）。
- 头②（期望侧）：corpus `stringify()` 以 `'textContent' in Object.getPrototypeOf(arg)` 判定——mock Text 原型链（Text.prototype→CharacterData.prototype→Node.prototype）无 textContent 成员，Text 形参期望算成 "[object HTMLDivElement]"（插入全对也判败；probe F0.textProtoHasTC=false→true）。GONE 构成双指纹实证：`[object HTMLDivElement]` 消息族＝头②面，纯字符串族（`('plain text';)` 等）＝头①面。
- 附带：无 nodeType 对象（TrustedScript）旧版被当节点直 appendChild／跳过——spec (Node or DOMString) union 须 ToString 铸 Text（转换前置，doc 域校验随之 spec 化）。

## 电池判读（v1，发射器 `.acceptance/dps/s10k15-launch.sh`，md5 断言 `e53d81d30f834fffe52fe4af45fb1567` 先行）

二进制 md5 `e53d81d30f834fffe52fe4af45fb1567`（＝commit 39c739149 字节域；v1 构建 df6d42b7… 因补规范链接注释被替换重建）；窗 22:20:09Z–22:51:32Z（31.4 分钟）；pre 锚＝k14 v1 五腿 F 集（1118/222/50/542/496，cp 至 /tmp/k15-*-preF.txt）；判读器 `.acceptance/dps/s10k15-judge.sh`（LC_ALL=C awk＋comm 前对账＋在飞腿 EXIT 守卫）。

| 腿 | pre (k14 v1) | post (k15 v1) | GONE | P2F | 终态 |
|---|---|---|---|---|---|
| WC | 1118F | 18406P/1118F | 0 | 0 | F 集逐键恒等（T 20；P−48＝截断落位族） |
| domparsing | 222F | 1313P/222F | 0 | 0 | F 集逐键恒等（P 1313 恒等） |
| sanitizer | 50F | 706P/50F | 0 | 0 | F 集逐键恒等（P 706 恒等） |
| **TT** | **1548P/542F** | **1728P/362F** | **180** | **0** | **F2P=180 精确/P2F=0/NET +180** |
| dom | 496F | 57499P/496F | 0 | 0 | F 集逐键恒等（P−16＝截断落位族） |

**join GONE=180 / P2F=0 / 新见 F=0** —— 零附带损伤。TT 判读与单腿真值预检（v2 单跑 1728P/362F，同二进制）逐项一致；TT P 1548→1728＝+180 恒等对账。

### TT GONE 构成（180，双页精确 90+90）

- `trusted-types/Node-multiple-arguments.html` 90＋`Node-multiple-arguments-tt-enforced.html` 90（各 2 targets × 9 arg-shapes × 5 setters 全清）。
- 消息族双指纹：`[object HTMLDivElement]` 族＝头②（期望侧 stringify 原型判定），纯字符串/混合族＝头①（提取 apply 落提取源）。

## 验证链

- probe_tmp_k15（已删）：F0 原型/名字 dump＋M 矩阵 5 方法×6 参数形 30/30＋P 系 sameFace=false→改派守卫生效（P1 xrep）。
- 钉测 `test_s10k15_variadic_mutation_pins` 1 passed（含 spec 序订正：before 参数序前插 'b1b2a1'＝args.join 序，首版 'b2b1a1' 系笔误）。
- 全量 zero-engine lib 4273 passed＋1 failed＝`test_dialog_focus_restore_declarative_shadow_s330` 前科红（k14 stash 复验非本刀）；fmt clean；clippy workspace clean；guard PASS（4 文件 +236，重扫绑定换 blob 后暂存态）。
- 刀体先入库：commit 39c739149（rebase 重放 be89d3a1b，对端 docs-only cb75a8439 无码面）先于发射；二进制 md5 断言绑定发射域。
- TT 单腿时序教训：v1 预检 timeout 900s 被 kill 137 零产出——k13/k14 电池内 TT 腿墙钟 16.5min 系五腿串行争抢慢化，solo 实跑 ~8min；重跑 timeout 1500 完成。

## 挂账（非本刀）

- query wrapper 直调 appendChild F9 got=[]（wrapper face 缺 ownerDocument/appendChild 桥）。
- harness 空 body 域 doc/body append/prepend inert（part06/part03 面归属另刀）。
- host 间接层双视图分裂（host.textContent 聚合含子树而 outer.textContent 空；未深挖，不阻本刀）。
- san 残面 50F；协议刀（InsertAdjacentHtmlOnHandle，zero-protocol＋renderer＋engine 跨面）真残面未 probe。

# d50 select-boundary 中段原子/空白带刀 A/B 证据（2026-11-17）

## 刀面

part01.js +82/−2（④ select-boundary 空界选区臂 BS 13034 / FD 镜像 13330 的
「run 中段」保守 bail 替换为带消费）：

- 门：`ecD1.nodeType === 3 && ecD1.parentNode === sc1`（sc/ec 同容器、ec 文本）
  + normal 域（preserve/preLine 不入）。
- 带：[sc 后邻, ec) 遍历——带内限**非块元素**（img/hr/inline 壳）与**折叠空白
  文本**（`/^\s*$/`）；块元素或可见文本混入 = 真跨内容选区 → 保持原 bail 章程。
  64 深度守卫。
- gtr：起点 `_zwCeCaretCanonical(sc, so, false)`（跳尾随折叠空白）、终点
  `_zwCeCaretCanonical(ec, 0, true)`（跳前导折叠空白）——`abc [<hr>] def` 界
  (abc,3)-(def,1) 契约形；双端原位截断 → live 引用第 5 参置位（d45 原位截断
  族口径：断言期读 post-value）。
- 变异（`_zwCeUnitApply` 单派发）：摘带全员 + sc 尾随折叠空白截断 +
  ec 前导折叠空白截断（normal 域行缘空白不渲染，摘除不可见——issue #112 块缝
  空白镜像）+ 缝处 caret。

## 权威账（专项单文件 A/B，base=HEAD 1baacb05e 前沿 d49 收口）

| 腿 | base | wip | 共键 F2P | P2F |
|---|---|---|---|---|
| non-collapsed-selection?Backspace | 29P/40F (69) | 34P/40F (69) | 5 | 0 |
| non-collapsed-selection?Delete | 29P/40F (69) | 34P/40F (69) | 5 | 0 |
| non-collapsed-selection?TypingA | 30P/39F (69) | 30P/39F (69) | 0 | 0 |
| backspace 主文件 | 78P/85F (163) | 78P/85F (163) | 0 | 0 |
| forwarddelete 主文件 | 96P/80F (176) | 96P/80F (176) | 0 | 0 |
| joining-dl 两文件四 variant | 751P | 652P（腿截断漂） | 0 | 0 |

**净 +10（F2P 10 / P2F 0）**。F2P 明细（BS/Delete 各 5）：`<p>abc[<img>]def</p>`
全键 ×2（promise 主键 + content 子测）＋ `<div>abc[<hr>]def</div>`、
`<div>abc [<hr>]def</div>`、`<div>abc [<hr>] def</div>` content 子测 ×3。
TypingA 变体走 insertText 路径不入 delete 臂，恒等零扰动。

## 探针定谳（zz-probe-d50f，shim 临时 trace 三轮，已全部回退）

- **img 形锚活**：C5 达 ④ 臂时引擎视图节点全活（scLen=3/scPar=P/scNext=IMG/
  ecPar=P）→ 旧 bail 是唯一卡点，带刀后 bi=1/gtr (abc,3)-(def,0)/content
  `<p>abcdef</p>` 全对。
- **hr 形锚鬼**：C1-C3/C6 进 impl 前 entry 已鬼化（`(t"",3)-(t"",0)`、结构
  sel），核心/④ 臂 trace 全空——重锚（`__zw_ce_caret_range`）产物为陈旧代际
  对象。带刀落地后：content 变异**正确**（`<div>abcdef</div>`）但
  beforeinput **不达页监听**（bi=0）——变异经鬼宿主生效、派发站 key 走鬼代
  计数。语料三 hr 面 content 子测 F2P、主键维持 got=0 即此形。
- **元素锚形停场**：C4 `(div,1)-(div,2)` `_zwCeSelectDelete` 拒收
  `rej sc-out-of-host`（sc.parentNode null，引擎视图分离代际锚）——master ③
  停场域，未动。
- 方法论：探针 assert-sentinel（expected 99）+ gTrace 全局数组入断言消息；
  shim trace 补丁三轮后全量回退（backup /tmp/part01.js.d50-orig，diff 核零残）。

## 停车场（本轮新增证据，未修）

1. **④ 臂 hr 形派发鬼**（master ③ 域新面）：同臂同代码路径 img 形派发全对、
   hr 形 bi=0——差异仅在宿主/派发站代际解析（`<div>` 内容 + 结构 sel 形）。
   content 已正确入账，残余仅 bi 计数/gtr 断言。两把入口刀已拒（d42 −27、
   d47 −5,556），不重攻。
2. **TypingA 变体带形**：非 collapsed img/hr 面在 TypingA 变体下走 insertText
   替换路径，不入 delete 臂——带刀零触及（恒等实测）。若要攻需另开
   insert 路径面（master 下一步 ④ text-control 矩阵相邻域）。

## 哨兵

- enginelib：**4125P/0F**（test-guard --compile-first --per-proc-mem 4
  --total-mem 8 --time-limit 1500）
- selection 全套件（跑完 53,317 键 49,019P/4,340F/9T）：对
  /tmp/d49-selce-base.json（CE 过滤 23 案）共键 **414 全同 F2P=0/P2F=0**
- input-events 全套件（85 案 5465 键）：对 /tmp/d49-suite-base.json 共键
  **5442，F2P=257（d49 +75/d50 +10/zz-probe 自增），P2F=0**
- html-semantics：可达性论证——语料 6 文件含 contenteditable
  （interactive-elements details/dialog 焦点面，无键盘删除路径），④ 臂仅在
  CE 非 collapsed 键删进入，hs 行为本刀构造性不变；CE 切片腿
  （interactive-elements 过滤）**446P/37F/0T 无 crash 标记**跑通（exit 1 =
  dialog/details 既有红，非本刀域）。首轮全量 hs 腿 scope 内 1 分钟 abort
  系双 runner 并发（2×8 job）超 24G MemoryMax（本会话操作失误，非代码面；
  单发重跑 interactive-elements 正常）

## 运行产物

- /tmp/d50-nc-{base,wip}.json（non-collapsed 三 variant）
- /tmp/d50-bs-wip.json、/tmp/d50-fd-wip.json（主文件双腿，base=d49 账同键）
- /tmp/d50-jdl-wip.json（joining-dl 对 /tmp/d49-jdl-final.json 共键核）
- /tmp/d50-sel-wip.json（selection 全套件）、/tmp/d50-suite-wip.json
  （input-events 全套件粗核）、/tmp/d50-hs-ce.json（hs CE 切片）
- zz-probe-d50f.html（wpt-data 本地不入树）

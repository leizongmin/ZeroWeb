# M2 s123 · FormData 条目构造收口（双源枚举 + file 状态臂——newline-normalization 12F→0F）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s122（classic script 编码链三面）

## 入场定刀位

取 s122 挂账首选：newline-normalization 12F 单根候选（原假设「FormData
构造换行归一过度」，纯 shim 面）。**勘根证伪原假设**——真根因是条目枚举
整体为零 + file 状态无枚举臂，换行归一反而正确（见下）。

## 勘根（探针页五轮递进定谳，gitignored corpus 验后删除）

语料 12 测试 = value/name/filename × \n \r \r\n \n\r（
newline-normalization.html），断言 FormData 条目中换行原文保真。探针链：

1. probe-s123-fd：动态建表（createElement+append）同 turn
   `new FormData(form)` → `fd.get('a') === null`、spread 空——**条目枚举
   整体为零**，非归一问题（归一面 idlVal 对 value 原文保真）。
2. probe-s123-fd2：`attrName="a"` 正常——排除 name pending stale 假象。
3. probe-s123-fd3：`document.querySelectorAll('input')` = 2——单选择器
   查询通，疑根因不在查询。误导一轮。
4. probe-s123-fd4：`document.querySelectorAll('input,input')` = **0**——
   **逗号列表**查询对 live 树恒空实锤；单选择器有 R161 pending 回落臂，
   列表没有。
5. probe-s123-fd5：静态标记页同查询 = 2——host 引擎支持列表（static
   标记页 staticDocInSel=2 通），**缺口仅 live 树 face**（snapshot 查询
   无该动态子树）。

同链探明第二根因：file 状态条目在构造器枚举臂中无专属处理——枚举到
file input 后落入通用臂不产出条目（spec constructing the entry list 的
file 臂要求每选中文件一条 `[name, File, filename]`）。

## 根因定谳

1. **live 树 document 级逗号列表查询恒空**：FormData 构造器原单源枚举
   `document.querySelectorAll('input, select, textarea, button')`——动态
   建表后同 turn 构造时 host 快照无该子树，而 live 树 face 逗号列表查询
   无 pending 回落（单选择器才有 R161 回落臂）。既有先例：R57 :invalid
   臂注记「逗号选择器顶层不支持——分开查询 concat」。
2. **file 状态无枚举臂**：checkbox/radio 臂后直接落通用文本臂（原带
   `type !== 'file'` 守卫），file input 产出零条目。

## 施工（净变更 part02.js + testharness.rs pin ×1，无其他 Rust 变更）

1. **枚举双源并集**（part02.js 构造器枚举段）：document 级查询（静态页
   原路径 + form= 属性站外关联元素 best-effort 面）在前 + `form.
   querySelectorAll` 子树 live 查询随后，`_fdSeen` 按元素键去重（
   `_elKeyOf` 优先，`__zwHandle`/`__zwSelector` 回落）——静态页结果序与
   原行为逐项一致（子树新增均已被 document 级覆盖、去重跳过），动态页
   子树序即树序。
2. **file 状态条目臂**（插在 checkbox/radio 臂后）：`type === 'file'`
   时每选中文件一条 `_zwFdEntry(String(name), _fdFile)`（复用既有条目
   构造，File 本体保真，File.name 不做换行归一——filename 判据面）；
   无选中文件零条目（spec 同）。兜底 else-if 去掉 `type !== 'file'`
   守卫（联动防双计）。
3. **pin `formdata_entry_list_s123`**（runner testharness.rs）两面：
   ①hidden input `value='b\nc'` 动态建表 → `fd.get('a') === 'b\nc'`
   （live 枚举 + 原文换行零归一）；②DataTransfer.items.add(File) →
   `input.files` setter → `fd.get('f').name === 'x\ny'`（file 臂 +
   filename 原文）。

## 验证

- 定向 `FILTER=newline-normalization` **12/12**（12F→0F）。
- **全量 A/B**（/tmp/s122-full.log → /tmp/s123-clean-full.log，逐行 status
  join）：**恰 14 对 Fail→Pass**（12 newline-normalization + 2
  wrapping-transformation 搭车——FormData(form) 同路径被同刀修复），
  **真翻红 = 0**（6639P/1159F/105T → 6653P/1145F/105T，总数 7903 恒定）。
  方法论落坑记账：首跑 s123-full.log 为**探针页未删时取数**（5 探针页
  7 Fail 噪声行 → 终态 1152F 与 14 翻绿不配平），pre-commit 门禁事实
  核查项 BLOCK 抓出——探针删净后补跑定谳本数字。
- bin pin `formdata_entry_list_s123`：新代码**绿**；part02.js stash
  往返**两面红**（`live form 枚举 + 原文换行零归一: null` /
  `file 条目 File.name 原文: null`——行为失败非编译失败）。
- engine lib **3570/0**；guard 内 clippy（runner --all-targets）零告警；
  fmt 净；shim 拼接（真实装配序 part01+01b+02..07）node --check 过。
- 语料零改动（探针页 probe-s123-fd/fd2/fd3/fd4/fd5 五个验后删除）。
- 渲染面零依赖——reftest/product-smoke 未跑。

## 挂账（本切片新增/更新）

- **newline-normalization 12F 收口移出候选**（本切片双刀）。
- **FileList 缺索引访问（新）**：`files[0]` undefined，仅 item()/length/
  iterator——spec 缺口真实但 corpus 不需要（枚举走 .item()），精准修改
  未顺带扩面。
- **live 树 document 级逗号列表 qSA 缺口（新）**：单选择器有 R161
  pending 回落臂、列表没有——本切片以双源并集绕开（FormData 面），根
  缺口仍在 host 查询 face，若其他 face 命中同症按 R57 先例「分开查询
  concat」或补列表回落臂。
- 承袭不变：form-submission-0 219F 精确化挂账（186F POST 传输契约 +
  33F iframe 文档管道）、写流执行身份盖章断链 FIXME、part02 setAttribute
  绕 notify 臂（gap 2）、execution-timing 构成面、041 nested write、
  lazy-loading 26F、meta color-scheme 21F、css-module corpus 门（待决）、
  www2 可达性、modulepreload referrerpolicy 族、module-import-referrer
  TLA、script-crossorigin-network cookies、redirect-relative-urls、
  export-from with FIXME、143 write、defer 多写完成序、dynamic import()
  with 透传、classic 外链 SRI、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt；
  **待决（须用户拍板）**：css/text 数据模块语料资源缺失、parser form
  element pointer 4F 缓行（2026-10-23，并入 declarative shadow DOM
  parser 批次）。

## 下一步候选（s124）

module 网络面挂账项勘根（modulepreload referrerpolicy 族 /
module-import-referrer TLA / dynamic import() with 透传 / classic 外链
SRI——逐一辨 shim/runner/host 契约面）/ 承袭挂账序复核。

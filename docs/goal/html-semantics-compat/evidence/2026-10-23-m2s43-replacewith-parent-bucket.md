# M2 切片 43 — 挂账收口：replaceWith self-removal 父桶记账（R5246）（2026-10-23）

## 范围

s26 挂账「R371 rekey 时 dst 桶不存在（dstA:-1），不记父桶 removed——被替换
template 合并视图残留（对扫描无害，spec 违例候选）」。

## 根因与修复（R5246 一刀）

replaceWith sel 臂（part04）的 remove 步此前只覆盖两路：

- apply 面：`__zw_remove` / `__zw_remove_handle`（Remove/RemoveHandle
  mutation，host 快照换代时生效）；
- sel/handle 字符串标记表：`_zwMarkRemoved`（parentNode/isConnected 读）。

合并视图子读（childNodes overlay `_zwOverlayPendingChildNodes` / R322 归并
查询）按**父桶 removed identity** 剔除——remove 步不入父桶 → 被替换者（快照
sel 节点）在合并视图残留至 apply 换代，视图 ≠ spec 终态（replaceWith =
insert-then-remove，remove 步同步反映全部树视图）。

修复：remove 步后经 canonical 记账入口
`_zwHCLiveInvalidate([], [被替换者 proxy], 父sel, null)` 直调（父桶 removed +
全局 pending-removed + live 集合失效一体；`_zwHCCollectSubtree` 子树展平含
template 内容子）。proxy identity 取基列表条目（`_r371Self`，与 overlay 剔除
`res[ri] === rm` 同一缓存对象；scan miss 回落 `_makeProxy(sel, null)`，sel 键
identity 稳定同 `_wrapSelector`）。记账先于 R371 rekey → dst 桶必存在（含
removed 项），rekey 走合并分支（promote 分支保留为护栏）。

不扩面：sel 臂的 MO record 面（插入/移除两段，R117 handle 臂已有 record）是
独立缺口，本轮不动。

## 验证

- 契约 `test_dialog_replacewith_parent_bucket_accounting_r5246`（part75 新
  文件）：R1 corpus 同形（dialog > template + 后随兄弟）replaceWith(newDiv)
  同一 execute 内被替换 template 同步退出全部树视图（A 作用域查询 / B
  childNodes identity / C div 位序补位）+ 插入侧 D/E 与被替换者父链 F 既有面
  守卫；R2 body 直子 template（document/body 查询、body childNodes，类限定
  隔离 dialog 桶记账）。
- 既有契约断言更新：`r321_replacewith_fragment_flatten`（part24）——kids
  断言 `DIV.SPAN.SCRIPT.SPAN.B` → `SPAN.SCRIPT.SPAN.B`（前导 DIV 为宿主快照
  对被替换 target 的残留，R5246 后合并视图即 spec 终态；断言反转面：记账
  缺失则残留复现，误杀活节点则插入侧/反链/父链翻红）。
- interactive-elements 全域 98 files 482 subtest **419P/61F/2T 对 s42 逐位
  REG 0 / GAIN 0**（挂账本就「对扫描无害」——corpus 无该残留观测面，切片
  性质同 s42：spec 正确性收口 + 契约钉住）。
- zero-engine lib **3434P/0F**（3433 + 1 新契约）。

## 已知边界

- sel 臂 replaceWith 的 MO record 面（insert+remove 两段）维持缺口（R117
  handle 臂已有 record），独立挂账不扩面。
- R371 rekey promote 分支保留为护栏（本记账保证 dst 桶存在后实际走合并
  分支）。

## 门禁

zero-engine lib 3434P/0F、clippy --workspace -D warnings（test-guard 包裹，
systemd user scope 不可用环境）/fmt clean；零新增依赖；corpus runner
release 二进制经验证含 R5246（`_e5246inv` 符号）。数据
[2026-10-23-m2s43-replacewith-parent-bucket.json](2026-10-23-m2s43-replacewith-parent-bucket.json)。

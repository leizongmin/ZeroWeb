# M2 切片 42 — 挂账收口：非模态 close 不出组栈（2026-10-23）

## 范围

s23（R4300）挂账「非模态 close 不出组栈——读侧 purge 兜底（已知保守）」。

## 根因与修复（R5245 一刀）

`_zwDialogCloseTheDialog4200`（part01.js）的 `_zwDialogModalStack` 出栈
splice 原在 `if (_zwDialogModal[key])` 模态臂内——modeless（show()）关
闭（close()/requestClose()/ESC/light dismiss 全链）残留 stale 项：

- show()/close() 循环每轮 push 一个新项、stale 项不清理 → 重复项无限
  增长（内存卫生缺陷）；
- 读侧 purge 仅 ESC 扫描顺带清（part06 R4300 扫描 `!_zwDialogHasOpen`
  → splice+continue），IDL focus gate / backdrop top-sel 各自 stale
  跳过——行为面无观测差异，但组栈不再是「活组」结构。

修复：splice 移出模态臂，模态/非模态一律同步出栈（spec close watcher
随 dialog 关闭销毁、其组离栈）。单站点一刀；show()/showModal() 的
无条件 push 与 part06 读侧 purge 维持（markup open 直写面不经本核心，
part04 IDL false 臂 splice 既有）。

## 行为保持性验证

- 契约 `test_dialog_group_stack_hygiene_r5245`（part74 新文件）：
  R1 modeless show/close ×3 循环后重开 ESC 照常命中尾组（cancel 同步
  派发）；R2 双 modeless 栈序（ESC 先关尾组再关前组）；R3 全关后 ESC
  no-op；R4 模态臂保持（showModal→close→出栈→重开照常）。栈内部态无
  JS 观测面，契约钉行为不变量（防 splice 误伤活项/漏项回归）。
- interactive-elements 全域 98 files 482 subtest **419P/61F/2T 对 s41
  逐位 REG 0 / GAIN 0**（含 dialog 集群 82 files 全量——含
  dialog-closedby-show-stacked 内层 none 挡板面）。
- zero-engine lib **3433P/0F**（3432 + 1 新契约）。

## 已知边界

- markup 层 `removeAttribute('open')` 直写面不经 close-the-dialog 核心
  （spec：close 事件不派、watcher 不可再用），stale 项仍靠 part06 读侧
  purge 兜底——R4300 既有设计，出现频度低（页面 JS 绕过 IDL 直改内容
  属性），维持不扩面。
- ESC 组栈惰性补登记边缘乱序（s23 另一挂账）维持。

## 门禁

zero-engine lib 3433P/0F、clippy/fmt 无 Rust 变化（shim JS 一刀）；
零新增依赖；runner release 二进制随 make 依赖链重建。数据
[2026-10-23-m2s42-nonmodal-close-stack.json](2026-10-23-m2s42-nonmodal-close-stack.json)。

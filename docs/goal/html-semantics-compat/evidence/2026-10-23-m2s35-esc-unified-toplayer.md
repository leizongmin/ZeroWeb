# M2 切片 35 — 挂账 E 余项：ESC 统一 top-layer 序（popover watcher 接入，2026-10-23）

## 范围

closedby-simple 5F（ESC 面）收口：ESC 此前只走 dialog 组栈（R4300），
popover（auto）的 close watcher 未接入——ESC 不关 popover（A 面「popover
should close after first ESC」3F）且 dialog 关后第二次 ESC 不关 popover
（B 面 2F）。

## spec 钉定

close watcher 组栈（close-requests-and-close-watchers）：popover show 与
dialog show/showModal 同注册 watcher，请求关闭作用于栈顶 watcher；
closedby=none dialog 不注册 watcher 但其**组挡「再往下」不挡「更晚注册」**
（R4300 组栈语义原样，part60 R2「any 非模态关」+ show-stacked inner-none
挡板两面同源）。

## 落地（R5236，`__zw_esc_dialog_cancel` 两段式）

1. **第一段**：`_zwTopLayer`（showModal + showPopover 共享插入序）反走
   自顶向下——栈顶 showing popover 命中即 `hidePopover`（beforetoggle/
   toggle 既有流）return true；先碰 open dialog → 栈顶 watcher 面是
   dialog → break 切组栈；stale continue。
2. **第二段**：R4300 组栈逻辑原封不动（惰性补登记 markup/IDL open 直写
   面 + none 挡板 + 尾→头首个 open 组）。

设计要点：dialog 裁决**全权归组栈**（非模态 any dialog 不入
`_zwTopLayer`，组栈惰性补登记覆盖），popover 只在「其上无 open dialog」
时被第一段命中——两段拼接在所有混合序（popover 上/下、none/any 组合）
下与 watcher 组栈语义一致。

## 集群验证

- dialog 集群 82 files **411 subtest 350P/60F/1T**（数据
  [2026-10-23-m2s35-esc-unified-toplayer.json](2026-10-23-m2s35-esc-unified-toplayer.json)），
  同 key 411 对 s34 ON 基线（345P/65F/1T）逐位 diff：**REG 0 / GAIN 5**
  ——closedby-simple 5F 全解，**simple 文件 6/6 全绿**。
- 回归面：canceling 族 8P/0F（EXIT=0，ESC 资格门无回归）；keyboard
  套件 77 common 键 A/B（stash 前后双跑）**逐位一致** REG 0/GAIN 0。
- 集群设计迭代记录：首版统一扫描（碰 dialog 直接 none 挡板/资格门）
  破 part60 R2（any 非模态 any 被 none modal 挡）——lib test 抓住，改
  两段式后 3429P/0F 恢复。

## 门禁

zero-engine lib **3429P/0F**；clippy `-D warnings` clean、fmt clean；
零新增依赖。runner release 二进制重建（shim include_str! 进二进制）。

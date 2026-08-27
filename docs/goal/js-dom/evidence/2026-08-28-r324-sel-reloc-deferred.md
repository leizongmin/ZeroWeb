# R324 Evidence — sel 子重定位切片（负结果回退：三层改动各自正确但环境路径分裂未收敛，降级 L2 切片三完整问题记录）

**日期**: 2026-08-28
**切片**: M4/L2——R324(a) sel 子重定位形态归并（**负结果：全部改动已回退，基线零漂移**）
**改动面**: 无（worktree clean，2462 全绿）

## 一、三层改动的技术内容（已实现、已验证、已回退）

1. **泛型直调扩展**（part03:1951）：`Node.prototype.insertBefore` 的 R219 trap 直调判定
   `this.__zwHandle !== undefined` 扩展 `|| this.__zwSelector !== undefined`——sel proxy
   的 insertBefore(mid, ref) 旧落 **appendChild 兜底**（位次语义丢失，移动变追加）。
2. **SLOT 记账**（part04 insertBefore trap 统一出口）：sel 子补
   `_zwSelPendingParent = {parentSel, nextSibling: refNode}`（R182 同款——查询归并的
   数据源）。
3. **QSA 归并消费**：桶 sel 子的旧位剔除（host 结果含旧序）+ 锚点插入（nextSibling 槽
   字段）+ mg 计数纳入 sel 子（否则 merged return 不触发）。

## 二、回退原因（环境路径分裂）

- **单测环境**（detached doc shim）：`getElementById('host')` 产物是 wrapper/桥对象，
  `insertBefore` 走 _zwParseEl 域或桥域（own=false + 非 Node.prototype 泛型 + 非 trap——
  三重判定全空，实际执行路径未完全定位）；k 序移动成功但 SLOT 不生效、QSA 归并消费不到。
- **WPT 环境**（真实 runner）：探针（zz-r324，已清理）复现 QSA 序 `a.b.mov`（期望
  `mov.a.b`）——trap 直调/SLOT/归并的链路未生效，debug 全局（__r324t）跨探针读不到
  （global 隔离面），断点定位无法继续。
- **方法论对照**：R222 的真实文件注入法在 Range 域成功的前提是探针能读 shim 内部状态；
  本次 debug 全局通道失效使三层改动的执行路径验证成本失控。

## 三、降级记录（L2 切片三的完整问题定义，供专项立项）

sel 子（无 handle 有 selector 的静态页面元素）同步移动的**全链路**：
`泛型直调判定` → `trap 统一出口 SLOT 记账` → `pending 桶记账（_mo_notify 已有）` →
`QSA 归并消费（旧位剔除 + 锚点插入）`——四环节须在同一执行路径上一次打通并验证。
散点改动在双环境路径分裂下不可独立验证，按「深结构护栏」整组回退，待 L2 主线
（identity 双源统一）一并处理。

## 四、基线

worktree clean，engine 2462 全绿（R323 定稿态），dom sweep Fail set 与 R319/R320/R321
恒等（54140P/58F/22T 系）。

## 五、教训

1. **环境同构性先行**：改动涉及泛型/trap/域实现三层时，先确认验证环境走哪条路径
   （getElementById 产物域 × dispatch 域 × wire 域），否则三层改动无法归因。
2. **debug 全局通道要先验证**：`globalThis.__xxx` 在 runner 探针里的可达性是 R309 DBG
   方法论的前提，失效时及时切「assert 消息回显」而不是反复猜。
3. **负结果也分层**：三层改动中「泛型扩展」与「SLOT 记账」有独立正确性（各自 spec 依据
   清楚），但「消费端」依赖前两者在真实路径生效——半生效状态不可 land。

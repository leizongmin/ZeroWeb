# 上游 tentative 族 expected 记账 bug 取证（after_target_removed/appended_interleaved）

**日期**: 2026-10-04
**结论**: `pointerevent_after_target_removed_interleaved.tentative.html` 与
`pointerevent_after_target_appended_interleaved.tentative.html` 在 corpus pin
（WPT_REV=315976933870b34d6ea30e3f6643403edae678ba）上**上游即不可通过**——
不再作为本 goal 追平对象；行为面对齐后即可（本轮已达成）。

## 证据

两文件的 `logEvent` 只在 `e.type.startsWith(logged_event_prefix)` 时入账
（prefix = "mouse"），而 expected 数组包含 `click@parent` / `click@child`——
`"click".startsWith("mouse") === false`，click 事件**在任何浏览器都不可能入账**
→ `assert_equals` 恒 Fail。

已核对 pin 版上游原文（raw.githubusercontent.com @315976933…）：
`logEvent` 与本地 corpus 导入逐字节同型。tentative 标记 + pointerevents#492
（removal 与 compat mouse event 交错语义未定）一致。

runner 侧旁证：尾簇 4 前 `received_compat_mouse_events=false → expected=[]`
守卫使该族在我们 runner 上 vacuous pass（日志空 + expected 清空）；尾簇 4 起
compat mouse 层完整派发 → 日志非空 → 与不可能的 expected 真比较 → Fail。
此为**更诚实**的状态：Chromium 上跑同文件同样 Fail。

## 本轮行为面对齐验证

尾簇 4 后 got 序列与 expected **仅差 click@ 记账项**（structural unloggable）：

```text
expected: ...,(child-removed),mouseover@parent,mousedown@parent,mouseup@parent,click@parent,...
     got: ...,(child-removed),mouseover@parent,mousedown@parent,mouseup@parent,...
```

六个 variant（?mouse/?touch/?pen × pointerdown/pointerup remover）全部同型。

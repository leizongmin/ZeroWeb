# M2-s3 附带 — fullscreen corpus 复跑（set_permission 白名单解锁效应记录）

**日期**: 2026-09-24
**套件**: `make testharness-fullscreen`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-24-m2-s3-fullscreen.json](2026-09-24-m2-s3-fullscreen.json)
**前值**: [2026-09-23-m1-fullscreen-baseline.md](2026-09-23-m1-fullscreen-baseline.md)（92/146 = 63.0%）

## 结果

| 指标 | M1 基线 | M2-s3 复跑 | Δ |
|---|---|---|---|
| subtests Pass | 92/146 = 63.0% | **92/149 = 61.7%** | Pass 绝对数持平，分母 +3 |
| 全绿用例 | — | 5/55 | — |

**通过集与 M1 基线逐案 diff：lost 0 / gained 0**——零回归。

分母增长的构成：WAB2-M2-s3 将 `test_driver.set_permission` 纳入 runner 白名单 +
TESTDRIVER_STUB 落地（clipboard 块权限注册表供数），此前整案 Unsupported 的 3 案解锁：

- `permission.tentative.https.html`（伪子测 1 → 真子测 3）：全 Fail——PermissionStatus
  全局类缺失（instanceof 断言 ReferenceError）+ query 不支持 allowWithoutGesture 字典
  校验（allowWithoutGesture:false 应 TypeError）。
- `element-request-fullscreen-without-user-activation.tentative.https.html`（1 → 2）：
  全 Fail——navigator.userActivation 面缺失（isActive 断言 ReferenceError）。
- `element-request-fullscreen-options.tentative.https.html`（1 → 1）：Fail——
  requestFullscreen(options).screen getter 调用语义未落。

**M3 修齐候选**（按上游案聚类）：① PermissionStatus 类 + permissions.query 字典校验
（fullscreen permission 面）；② navigator.userActivation（isActive/transient 面与
requestFullscreen 激活检查联动）；③ fullscreenOptions 成员展开（screen/navigationUI）。
真挂起 Timeout ×8 甄别仍为 M3 主对象。

结论：61.7% 为解锁效应（非绿面从伪子测转为真子测显形），非回归；M3 起点 92 真绿。

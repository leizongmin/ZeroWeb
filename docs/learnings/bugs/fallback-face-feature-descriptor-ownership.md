# Fallback face 必须独立解析 feature descriptor

日期：2026-08-14

相关模块：`zero-render-foundation::font`

## 问题描述

ordered font fallback 可以把缺失字符交给 secondary face，但 shaping 只合并 primary face 的 `@font-face font-feature-settings`。secondary descriptor 不生效，primary descriptor则被复用到所有 fallback run。

## 根因分析

FontLoader 在进入 TextShaper 前只解析 primary face 的 descriptor。TextShaper 随后把同一 feature vector传给每个 resolved run，并在 vector 非空时完全跳过 fallback shaping。

这同时破坏 feature precedence 与 cache ownership。不同 face 的 descriptor会产生不同 glyph 序列，必须进入对应 face 的 cache key，不能只记录 primary 的 resolved features。

## 解决方案

每个 resolved face 在 shaping 前独立合并自己的 descriptor，再由元素级 caller feature按 tag覆盖。每个 face 的 resolved feature vector进入 cache key；注册 descriptor 时清空共享 shaping cache。

机制测试应使用相同字体字节和互斥 `unicode-range` 强制文本落到 secondary face，再用不同 `liga` descriptor验证 glyph 数，并单独验证 caller override优先级。目录 Oracle 没有像素变化时，只能说明 corpus 缺少该组合，不能否定机制缺口。

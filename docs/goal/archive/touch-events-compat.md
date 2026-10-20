# Touch Events 兼容 — TouchEvent 构造器与属性面

> **整体归档（模式 A，2026-10-20）**：按原目标约定范围收口；控制面与证据随入口归档，原有排除项、残差和重入条件保留。对应 Rally 启动脚本已删除。

**版本**: v1.3
**日期**: 2026-10-14
**状态**: Completed（可执行面收口，2026-10-14；注入面挂宿主触摸注入管线重入）
**执行模式**: WPT 驱动（上游 touch-events corpus 为验收标尺）+ 实现/语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——输入事件面）

> **▶ 收口（2026-10-14，M3 轮）**：DC-1~3 逐项核对通过——可执行面 30P + 2 中性
> PF（非中性 100%）；make test 71 suites 全绿、reftest 710/710、product-smoke
> 15.35% 恒值（struct-check PASS）。注入面 9 案 28 子测试带数字挂账（1P/23F/4T，
> 根因 = 宿主 touch 真值流缺失），重入条件 = 宿主触摸注入管线。证据：
> [evidence/2026-10-14-m3-closure.md](touch-events-compat/evidence/2026-10-14-m3-closure.md)。
> **重入**：多点触控硬件集成 / `ontouch*` 曝出面（随宿主触摸能力声明）。
>
> **▶ M2（2026-10-14）**：Touch/TouchList/TouchEvent 按 spec 落 engine shim
> （part05.js），可执行面 20P→30P 非中性 100%，historical 双向门 8/8 保持——
> [evidence/2026-10-14-m2-constructors.md](touch-events-compat/evidence/2026-10-14-m2-constructors.md)。
>
> **▶ M1（2026-10-14）**：语料 18 资产导入（pin 315976933），runner 通道
> `make testharness-touch-events` 落地；可执行子集甄别 4 案 / 排除 12 资产逐类挂
> 重入门槛，基线 20/32——
> [evidence/2026-10-14-m1-baseline.md](touch-events-compat/evidence/2026-10-14-m1-baseline.md)。
>
> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：uievents
> goal（编号 50，已收口）挂账面之一；uievents M2 的 driver actions 重写正好是
> 注入底座；Touch/TouchEvent 构造器面是纯 JS API，可直接度量。

## Mission

以 WPT touch-events corpus 可执行子集为验收标准，收敛构造器（dict 属性映射/
异常）与事件属性列表语义。排除：pointer 事件（编号 50 域，已收口）、宿主多点
触控硬件集成（挂账）、触摸手势识别（未立项域）。

## Done Criteria

- **DC-1** 导入 + 基线 + 可执行子集甄别（注入面需求结论落 evidence/）+ csv
  planned 行回填
  - [x] corpus 18 资产导入 + 甄别（4 案可执行 / 12 资产逐类挂重入门槛）+ 基线
        20/32 落 evidence/ + csv 数据行（2026-10-14，status=active）
- **DC-2** 构造器与列表属性语义对齐，分级通过率不再下行两个连续轮次
  - [x] 可执行面非中性 66.7% → 100%（2026-10-14，M1→M2 同日上行；30/30，
        [M2 evidence](touch-events-compat/evidence/2026-10-14-m2-constructors.md)）
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归、product-smoke
  逐字节恒值
  - [x] make test 71 suites 全绿 + reftest 710/710 零回归 + product-smoke 15.35%
        恒值（struct-check PASS；2026-10-14，
        [收口 evidence](touch-events-compat/evidence/2026-10-14-m3-closure.md)）

## 里程碑

M1 导入与甄别（✅ 2026-10-14，20/32）→ M2 构造器/属性面收敛（✅ 2026-10-14，
30/32 非中性 100%，historical 双向门保持）→ M3 注入面带数字收口 + 归档（✅
2026-10-14，9 案 28 子测试 1P/23F/4T 挂宿主触摸注入管线重入）。

## 依赖约束（run-rules §9）

注入底座与 uievents（编号 50，已收口）driver actions 遗产复用；与
keyboard-page-scrolling（已归档）滚动语义不重叠。

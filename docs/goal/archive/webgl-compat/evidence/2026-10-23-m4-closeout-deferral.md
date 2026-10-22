# M4 收口——WebGL2/WebGPU 挂账定稿 + GPU/CPU 双路径差异记账（2026-10-23）

goal：[docs/goal/webgl-compat.md](../../webgl-compat.md) M4（挂账定稿）与 DC-4
末项（GPU/CPU 双路径差异记账）。本文档为 M4 交付物，交用户签收；签收前挂账
结论即当前有效边界，签收如推翻按新裁决重开。

## 1. WebGL2 / WebGPU 挂账定稿

**结论：明确不在本 goal 范围，移入远期挂账，不设预期时间。**

- 依据：goal 文档「排除（明确不在范围内）」节自立项即排除 WebGL2/WebGPU
  （"远期挂账（WebGL1 收口后评估）"）；2026-10-22 用户四裁决将自主边界划至
  DONE，未扩围。
- WebGL1 基线现状（挂账前提已满足——"WebGL1 收口后"）：
  - testharness 基线 2112 Pass / 3 Fail（3 Fail 为 pin rev IDL 同 mixin 同长
    重载判据，上游语料侧，见 §3）；
  - 上下文/查询/事件/shader 管线/对象存储/draw 光栅六面落地（M2~M3-S4，
    part08+part08a/b/c/d 五件 ~4600 行）；
  - reftest 标尺 4 例（clear 2 + draw 2）常驻全过。
- 重开条件（未来评估时的最小入口面）：
  - WebGL2：需 GLSL ES 3.00 前端（当前 part08b 为 GLSL ES 1.00 单遍）、VAO/
    实例化/transform feedback 对象模型、`getContext('webgl2')` 通道（shim 现
    恒 null，part67 已钉类型冲突语义）；
  - WebGPU：归 GPU 栈流域——render-foundation wgpu 底座已有，但 WebGPU JS
    API 面（adapter/device/queue/编码器）体量与 WebGL 完全不同纲，须独立
    goal 立项，不得借本 goal 扩围。

## 2. GPU/CPU 双路径差异记账（DC-4 末项）

**现状：WebGL 渲染 100% CPU 路径。** drawing buffer 为 JS shim 内 Uint8 光栅
（part08d），经 host 2d 位图 putImageData 呈现；render-foundation wgpu 栈
（约束 2"依托既有 GPU 栈，不另起实例"）在 WebGL 面零接触。

| 维度 | CPU 路径（现状） | GPU 路径（挂账） | 差异处置 |
|------|------------------|------------------|----------|
| 光栅化 | JS 像素中心重心，无 MSAA | wgpu 管线（MSAA 可选） | reftest 标尺自建自比，无跨路径比对需求；conformance 无像素级抗锯齿断言，暂无回归面 |
| 裁剪 | 任一顶点 w≤0 整图元丢弃（part08d FIXME） | GPU 硬件裁剪 | 大坐标/跨面图元覆盖不全；corpus 批 B 无渲染断言案，暂无失败证据；重开 GPU 路径时自然消除 |
| blend | 仅默认 ONE/ZERO 等价替换写（FIXME） | 完整因子面 | corpus 未触；canvas-2d goal 的合成面不在本 goal |
| depth/stencil | 状态面已存，测试写入恒通过（FIXME） | GPU 硬件 | 同上 |
| 纹理采样 | texture2D 返回黑（FIXME） | 采样器 | corpus 批 B 无采样断言；M3-S3 上传链已真存储，重开时接 render-foundation 纹理 |
| 呈现 | putImageData 全幅替换（preserveDrawingBuffer:false 未区分，FIXME） | swapchain | retained buffer 语义已实测（part71），preserve 差异未暴露 corpus 断言 |

**记账结论**：差异面全部落在 corpus 批 B 无渲染断言的区间，当前零失败证据；
不构成本 goal 收口阻塞。GPU 路径重开与 WebGL2 挂账同批评估（重开入口：独立
切片，依托 render-foundation，不另起 wgpu 实例——约束 2 不变）。

## 3. 上游 3 Fail 处置

2112/2115 中 3 Fail 均为 idlharness 对同 mixin 同长度重载重复成员的判据分歧
（上游语料判定，非 shim 行为缺陷；shim 按 pin rev IDL 落方法面）。处置：
**接受为基线**，M4 后 pin rev 前移时复查（master.md 缺口清单不另立条目，
base 上游语料侧）。

## 4. DC 核验汇总

| DC | 状态 | 证据 |
|----|------|------|
| DC-1 导入盘点+建议书+CSV | ✅ 2026-10-22 | evidence/2026-10-22-m1-import-inventory.md |
| DC-2 批次 B 收敛（含 clear 标尺①） | ✅ 2026-10-22 | evidence/2026-10-22-m2-context-slice.md |
| DC-3 管线逐簇收敛 | ✅ 2026-10-23（S1 查询/事件、S2 shader 管线、S3 对象存储、S4 draw 光栅四簇） | evidence/2026-10-22-m3-s{1,2,3}-*.md、2026-10-23-m3-s4-draw-raster.md |
| DC-4 门禁+双路径记账 | ✅ 2026-10-23 | make test exit 0、clippy -D warnings（zero-engine/zero-wpt-runner）、cargo fmt 零差异、make reftest 716/716（新增 2 例全过=零回归）、本文 §2 |

## 5. 交签收项

请确认（任一项推翻即重开对应范围）：
1. §1 WebGL2/WebGPU 挂账定稿与重开条件；
2. §2 GPU/CPU 差异记账结论（CPU 路径维持，不阻塞收口）；
3. §3 上游 3 Fail 接受为基线。

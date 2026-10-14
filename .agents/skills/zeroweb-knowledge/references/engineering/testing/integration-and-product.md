# 跨 crate、产品、多进程与协议测试

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖跨 crate 的测试接线、真实页面脚本与产品/IPC 验证的差别。关键词：`test-support`、`CARGO_BIN_EXE`、Playwright、`expected-green`。执行与清理导航见 [项目验证](../../../../zeroweb-verify/SKILL.md)，本文只静态核对覆盖。

## Gotchas
- `zero-integration-tests` 对 browser 开启 `test-support`，不是发布构建的同义词；测试内进程内浏览器路径不能代替独立 renderer 产品验收。 (`tests/integration/Cargo.toml`)
- 页面 `<script>` 与 `execute_script_with_dom` 可能产生不同代理身份；template fragment 根因钉必须先加载真实页面脚本，再用 evaluate 读结果，而不是用 evaluate 构造整个场景。 (`tests/integration/src/e2e_static_content.rs`)
- handle 容器缓存重挂钉在修复前亦绿，承担当前行为回归而非根因判别；sel-only 容器钉才有已记录的红态差异，不能从测试名字推定负控有效。 (`tests/integration/src/e2e_static_content.rs`, git:3b0dcde8b30ac45d82285483c9abfe210018b79f)
- Playwright 流程退出 1 可以包含已挂账的失败步骤，但退出 2、信号退出或 `fatal` 报告必须拒读；否则上一轮 steps-report 会伪装成本轮完整结果。 (`tests/playwright-matrix/scripts/verify-deterministic.mjs`)
- 确定性签名只比较步骤集合及 ok/FAIL，不比较截图字节、几何值或耗时；双跑一致仍需要独立视觉与生产帧验收。 (`tests/playwright-matrix/scripts/verify-deterministic.mjs`)
- compositor 的多数 IPC 测试强制 CPU RGBA，Linux dma-buf 专项才显式打开 GPU/export/import；不能把 frame_flow 全绿称为所有 GPU 交付分支已覆盖。 (`apps/compositor/tests/frame_flow.rs`)

## Architecture
- compositor GPU helper 的 import 开关同时影响父测试进程与子进程，锁需覆盖 spawn、解析和退出全过程；RAII Drop 会 kill+wait，即使断言 panic 也回收子进程。 (`apps/compositor/tests/frame_flow.rs`)
- image-decoder IPC 测试通过 Cargo 提供的可执行文件启动角色，比较四色 PNG 与进程内结果，并单独断言垃圾输入返回 error；它核验 wire 与像素闭环，不是全格式安全证明。 (`apps/image-decoder/tests/ipc_decode.rs`)
- `expected-green.json` 当前锁定 33 个步骤；`frames.access` 只验证子帧元数据，不包含仍挂账的子帧 click/evaluate 文档 realm。 (`tests/playwright-matrix/expected-green.json`)

## Conventions
- `tests/integration/src` 中新增测试文件还须在 `lib.rs` 挂 `#[cfg(test)] mod`，仅落盘源码不会被 libtest 自动发现。 (`tests/integration/src/lib.rs`, git:ff30224303ff9951c7e0d28940dcbfccd08d91fa)
- 修改协议后 Makefile 先刷新独立 renderer/compositor/image-decoder binary，再跑 browser 测试；只重建测试 artifact 会让新主进程遇到旧 wire schema。 (`Makefile`)

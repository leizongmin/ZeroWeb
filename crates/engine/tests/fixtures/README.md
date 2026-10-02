# 测试 fixtures

本目录存放单元测试引用的第三方静态资源（经 `include_str!` 编译期嵌入）。这些文件**仅用于测试**，
不进入任何构建产物或运行时依赖。

## loadmatrix-zw-001.html

- **来源**：自研（slice18 评审收尾 S1 落仓）——site-compat baidu 建议链诊断用活体
  fixture（loadmatrix.html v1–v9 动态脚本执行保真矩阵）的最小判别形态入驻版
  （v1 同源基本型 + v9 真跨域两案；原件仅存诊断环境，见
  `.acceptance/site-optimizer/baidu-storm-20260929/diag/evidence/slice18/` 证据 JSON）
- **消费方**：`js_dom_bridge_tests::r387c_loadmatrix_fixture_single_execution_end_to_end`
  （双相位判别：宿主所有权契约下单执行者语义 + 无宿主置位时通道原行为负控制）
- **更新方式**：判别语义变更须同步修改消费方断言并复跑 RED/GREEN（base 双执行语义红 /
  修复后绿）

## dompurify.js

- **来源**：DOMPurify 3.2.7（`dist/purify.js`，可读非压缩版）
- **上游**：https://github.com/cure53/DOMPurify
- **许可证**：Apache-2.0 OR MPL-2.0（双授权，文件首行 `@license` 注释保留原始声明）
- **使用条款**：ZeroWeb 项目技术路线排除 MPL 技术线（见 `docs/goal/zero-web.md`），此处按 **Apache-2.0** 条款引入；
  作为测试 fixture（非 Cargo 依赖、不链接进产物），用于验证自建 DOM 桥接对真实 sanitize 库的支撑能力
  （R3019：`js_dom_bridge_tests::test_sanitize_dompurify_real_r3019`）。
- **更新方式**：替换文件后须保留首行 `@license` 注释，并在本 README 核对版本号与许可证一致性。

## wpt-dom/

WPT DOM 共享脚本与 fixture（`common.js` / `ranges/Range-test-iframe.html`），
供 js-dom R209-R218 Range/CharacterData/insertNode 系列测试编译期嵌入。

- **来源**：web-platform-tests/wpt，pin 于 `315976933870b34d6ea30e3f6643403edae678ba`
  （与 `tests/wpt-runner/scripts/fetch-dom-subset.sh` 的 `WPT_REV` 一致）
- **上游**：https://github.com/web-platform-tests/wpt （`dom/common.js`、`dom/ranges/Range-test-iframe.html`）
- **许可证**：WPT 上游仓库许可（BSD-3-Clause）
- **为什么 vendor 而非引用 `tests/wpt-runner/wpt-data/dom/`**：`wpt-data/` 整体 gitignored，
  CI 的 `make fetch-wpt-data` 会 `rm -rf` 重建且不含 `dom/` 子集——`include_str!` 是编译期依赖，
  必须指向仓库内可用的文件（同 `.cache-storage-window-root` vendor 先例）。
- **更新方式**：先更新 `fetch-dom-subset.sh` 的 `WPT_REV` 并在本地 `make fetch-wpt-dom`，
  再将对应文件复制到此目录，保持与拉取子集同 rev。

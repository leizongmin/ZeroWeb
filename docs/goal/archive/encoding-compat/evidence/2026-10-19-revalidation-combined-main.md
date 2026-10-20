# 复跑第三轮 — 组合态 main 独立核对（零源码改动验证轮）

**日期**: 2026-10-19（rally 编码轮，二轮复跑修齐 `825549020` 之后的独立核对）
**套件**: `make testharness-encoding`（全语料 36 案）
**基线**: 组合态 main `b2ce0f883`——二轮修齐之上有渲染流 7 commit（css-parser/
style-system/engine SVG+transform/render-foundation paint/populateForm），
零编码面源码改动。
**原始数据**: `/tmp/encoding-recheck-20261019.log`（tee 全量 12112 行）

## 结果

**12072 Pass / 3 Fail / 0 Timeout = 12075 = 99.98%**——与二轮终态逐字持平，
渲染流 7 commit 组合态零回归。

3 Fail 逐一核对 = 记账残差 3，无漂移：

| Fail 案 | 记账归属 |
|---|---|
| textdecoder-copy :: Modify buffer after passing it in (SharedArrayBuffer) | SAB 结构面 |
| streams/decode-utf8 :: decoding a transferred Uint8Array chunk… | transfer 结构面 |
| streams/decode-utf8 :: decoding a transferred ArrayBuffer chunk… | transfer 结构面 |

逐案锚点核对（与既有记账一致）：api-invalid-label 3421（P2）、single-byte 336、
gb18030-decoder 275、encodeInto 111、textdecoder-mistakes 87（二轮 fatal-stream
修齐后全绿）、textdecoder-labels 222。

## 跨域核对

- wasm/jsapi 通道（715/719 还原态）：`825549020..b2ce0f883` 无 dom_bridge.rs /
  runner 预热序列变更（git log 实证），按 master.md 监控建议条款免跑。
- 文档级嗅探挂账：html-syntax-compat 控制面 P5 双向记账在位（实读核对）。

## 备注（复跑操作面，供后续轮次参考）

- `make testharness-encoding` 残差存在时退出 1 属设计内（runner 通道"有非 Pass →
  1、不作为 land 门禁"，见 main.rs 通道注释）——make Error 1 ≠ 本轮失败，判读看
  subtest 计数。Text 格式无汇总行，`| tail` 截尾会丢分母，全量核对用 tee 落盘。
- fetch 脚本幂等（`-s` 跳过已拉文件）；本 clone 语料原为 106 文件的旧部分拉取面，
  本轮 fetch 补齐至 231 文件（含 legacy-mb .html 资产与跨引用 support），此后复跑
  fetch 秒级。

## 门禁

零源码改动轮：无 make test / clippy / fmt 义务（本轮仅文档+CSV 记账）。

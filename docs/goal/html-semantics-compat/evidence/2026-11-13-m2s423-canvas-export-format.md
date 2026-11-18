# M2 s423 收口账——canvas 导出编码格式面（toBlob/toDataURL type 参数接通，png+jpeg
# 白名单 + PNG fallback，toBlob.jpeg 1F 清账）+ 顺带两 collateral 面归位

- 日期: 2026-11-13；基线: HEAD 3e1fd9d31（s422 收口 rebase 后），驱动: 红池分诊
  the-canvas-element/toBlob.jpeg.html 1F（s403 起在册承袭红）。红池分诊备选
  逐一排除：select-required-attribute.tentative（:required/:optional 计算
  样式面=style-system 渲染流）、video-loading-lazy-load-when-visible
  （scrollIntoView + intrinsic dimension 面=渲染流/媒体）、closed-details-
  layout-apis（layout API 面=渲染流）。

## 勘根

- 驱动案: `<canvas id="c">` + `toBlob(cb, 'image/jpeg')` →
  `assert_equals(data.type, "image/jpeg")`。
- 根因: shim 四导出位（part05 standalone toDataURL/toBlob + part04 元素
  proxy get-trap toDataURL/toBlob）全部忽略 type 参数——host
  `__zw_canvas_op(handle,'toDataURL')` 仅 PNG 编码（R2797 注释明示 jpeg/webp
  defer），Blob 恒 `{type:'image/png'}`。spec #dom-canvas-toblob 步骤：type
  parse as MIME type，非受支持编码格式回落 image/png——即诚实修复需**真
  JPEG 编码**（谎标 type 字符串、字节仍 PNG 是非合规行为，下游解码会炸）。
- 依赖盘点: 树内 jpeg-decoder/zune-jpeg 均解码面，无编码器 → 新增
  `jpeg-encoder = "0.6"`（纯 Rust 零依赖，与 png crate 同位级；工作区 +
  zero-engine 两处声明，控制文档/账本同提交记账）。

## 施工

1. **host toDataURL op 扩参**（canvas.rs）: arg(0) 编码格式（'png'｜'jpeg'，
   shim 归一白名单后传入，缺省 png）；arg(1) JPEG quality 1..=100（缺省 92）。
   jpeg 臂：直 alpha 像素**合成黑底**（rgb×a/255 +127 舍入，JPEG 无 alpha
   通道——toDataURL.jpeg.collateral 面）后 jpeg-encoder ColorType::Rgba 编
   码；u16 上限守卫；编码失败返空串（shim 回落 `data:,`/null）。PNG 原路
   径零改动。chunks_exact_mut 带 MSRV allow（as_chunks_mut 1.88 稳定 >
   rust-version 1.85）。
2. **shim 四导出位 type/quality 接通**: 格式归一 = `String(type).split(';')[0]
   .trim().toLowerCase()`，`image/png`/`image/jpeg` 白名单，其余 PNG fallback
   （spec 步骤原文）；data URL 前缀/Blob type 用归一结果。quality 消费条件
   `typeof === 'number' && isFinite && [0,1]` → ×100 取整 clamp 1..=100，其余
   （'bogus'/{}/null/undefined/true/'0.01' 字符串）→ 缺省 92——quality.
   notnumber collateral 面六个参数须与无 quality 输出**逐字节一致**，只有
   真数字才消费（Blink 口径，`+x` 强转会把 null→0/'true'→1 错误消费）。

## 验证

- 拼接 shim（js_dom_bridge.rs 25 part 规范序）node --check **SYNTAX-OK**。
- unit part113（s423_canvas_export_format）: **1P**——①proxy trap toBlob
  ('image/jpeg') Blob.type + SOI FF D8 FF 真字节 ②'IMAGE/JPEG' essence 大小写
  ③'image/webp' → PNG fallback type+签名 ④standalone toDataURL('image/jpeg')
  前缀 ⑤无效类型 PNG 前缀 ⑥无参 PNG 默认面回归。首跑 1F 为测试断言字符串
  笔误（`u.length` 拼接 vs 'true'——行为面正确），修断言后 1P。
- solo toBlob.jpeg: **1F→1P**。
- solo the-canvas-element 目录: 首 sweep **130P/2F**——toBlob.jpeg 转 P 同时
  **两 collateral 面曝光**（此前 type 被忽略时两案走「无 jpeg 支持」捷径分支
  assert(true) 假绿；type 接通后真断言路径首启）：toDataURL.jpeg.alpha（须
  黑底合成——修复①）+ toDataURL.jpeg.quality.notnumber（非数字 quality 须与
  缺省逐字节一致——修复②）。两修复后复跑 **132P/0F 零红**。
- engine lib 全量（终码）: **3914P/7F**（+1P=part113），失败名单对 s422 在册
  慢性红腿 7F 逐名 IDENTICAL。
- sweep ×2 join 对 s422 双基线（\x1f pairs、stable 集语义）:
  - my1 vs s422-b1: **stable=8384 state-diff=2 new=1**——state-diff = driving
    F→P + 簇 D home-end.optional 文件键 T→F（在册闪变）；new=1 =
    textfieldselection/select-event 文件级 Timeout（一次性 timing 闪变，
    s419 账本在册 b1-flake 同名文件；my2 复原全 Pass）。
  - my2 vs s422-b2: **stable=8454 state-diff=1 new=0 gone=0**——唯 driving
    F→P，完全干净。
  - my2 vs my1: **stable=8384 state-diff=2 new=69**——state-diff = 簇 D 两
    文件键 F→T（home-end{,-detailed}.optional 在册闪变）；new=69 = my1
    select-event 文件级 Timeout 未上报子测在 my2 复原。
- fmt --check + engine clippy --all-targets -D warnings: **零告警**（首跑 2
  lint——unused mut + chunks_exact_to_as_chunks——均为 jpeg 臂新码，MSRV
  allow + 去 mut 后清）。
- make test 全量: **154 腿 ok，唯一 FAILED 腿 = engine lib 3914P/7F**，失败名
  单对在册慢性红腿逐名 IDENTICAL（failures: 块归一 diff 干净；exit 2 即该慢
  性腿传播）。单跑执行，启动前 ps+/proc cwd 核对无同树残留（ZeroWeb-2/
  ZeroWeb-cronjob 并行作业系各自 clone 独立 target）。

## 口径

- 在册红 s422 收口 raw 203F+14T / 202F+15T → 本轮终态 raw **203F+14T**（轮1，
  含簇 D flap +1F 与 select-event 一次性 Timeout）/ **201F+15T**（轮2，簇 D
  两文件键 F→T +2T −1F、select-event 复原）——归一闪变面后稳定红面 **−1F**
  （toBlob.jpeg 1F 清账；顺带 alpha/quality 两 collateral 面从假绿捷径转真
  断言路径且保持绿），new 红 0。
- 依赖: `jpeg-encoder = "0.6"`（工作区 + zero-engine；纯 Rust 零依赖编码器，
  解码面既有 jpeg-decoder 不变）。
- 下一步: 红池分诊续——余件单红逐一分诊勘根（video 帧像素表面 FIXME 跨域
  大件待立项，挂账池承 渲染流 6 面 + embed intrinsic sizing 面 ×2 /
  sizes-auto F5 用户门 / 簇 D select 立项征询 GB-20261031 用户门 / checkable
  parent-label 2F / form-nameditem override builtins fix#19c 冲突 s137 用户
  门维持）。

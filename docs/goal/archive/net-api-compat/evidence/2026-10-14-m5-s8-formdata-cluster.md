# M5-S8 — formdata/FormDataEvent/submitter 簇修齐 + sync XHR 账面修正（2026-10-14）

**背景**：goal 内结构性挂账最大簇（formdata 9F）+ P3 账面「sync XHR 结构面 4」
失真修正。

## 账面修正：sync XHR 结构面 4 实测全绿

`FILTER=sync` 实跑：sync-xhr-and-window-onload 1P、sync-block-defer-scripts(-subframe)
1P、sync-block-scripts 1P、sync-not-hang-scriptloader(-subframe) 1P、
timeout-synconmain 2P——**blocking/onload/not-hang 语义面全 PASS**（并行流演进或
M5-S5 scheme 翻转期间已修，账面未更新）。残 2 案 = permissions-policy API 面
（`document.permissionsPolicy.features` 应含 sync-xhr——Feature Policy 接口未实现，
tentative 用例；security-hardening 邻域跨域记账，非 sync XHR 语义）。P3 账行
「sync XHR 结构面 4」就此销账。

## 修齐（shim 七处 + fixture 两件）

1. **FormData ctor WebIDL**（part02）：`new FormData(null)`/`'string'` → TypeError
   （接口形参非 Nullable；undefined = 缺参仍合法——constructor.any.js 面）。
2. **ctor image submitter**（part02）：nameless image 按钮不再被 `!name continue`
   跳过——bare `x`/`y` entry（spec name 前缀为空）；坐标消费 click 激活记录。
3. **`_zwImgRecordClickCoords` + `_zwSubmitBtnDefaultAction` 共享 helper**（part04，
   置 `_zwRunFormSubmit` 前）：selected coordinates 记录（clientX/Y − 盒原点，
   max 0 钳制，`_zwImgSelCoords` 按 `_elKey` 持久）+ submit 按钮 click 默认动作
   共享入口（`.form` 直取 + handle 反链 walk——原 `.click()` 门内联块抽出共用）。
4. **dispatchEvent 派发 click 默认动作扩面**（part04 R106 块）：submit 按钮
  （INPUT submit/image + BUTTON submit）经共享入口跑提交（原仅 A/AREA 锚面）——
   `new MouseEvent('click')` 派发激活图像按钮的 spec 语义。
5. **`_zwFormSubmitNavigate` 坐标 entry**（part04）：nameless → bare x/y；'0' 硬编码
   → 记录坐标（非指针激活语义不变）。
6. **Request FormData/Blob body 共享序列化**（part01）：走
   `_zwBodyBytesAndContentType`（multipart 编码 + Content-Type 派生 + `_bodyText`
   同步）——此前 `String(init.body)` 化为 `'[object FormData]'` 且无 CT。
7. **multipart parser part 头**（part01）：filename + Content-Type 大小写不敏感；
   带 filename → `File`（part CT，缺省 ''）——consume-multipart-headers
   case-insensitively 面。

**fixture**：`images/100px-green-rect.svg`（上游 200×200——期望值 23/46 的几何
标定源，公式 `round(clientX − rect.left)` 推导实证）+ `media/1x1-green.png`
（WPT_REV 31597693 pinned 原件）——wpt-data 不跟踪，经 `goals/20-net-api-compat.sh`
显式 `fetch_raw` 随语料续拉。

## 数字

六 corpus 受控 A/B（对 M5-S7 树）：**9041P/3024F/10T → 9055P/3010F/10T =
+14P/−14F 零回归零分母漂移**——formdata 面 4 腿（constructor-submitter
in-tree-order、constructor.any TypeError、fetch body empty formData、multipart
case-insensitive）+ Request body 路径附带解锁 fetch/api 10 腿（abort/request
+3、body/mime-type +2、request-consume-empty +2、request-consume +1、
request-init-contenttype +2）。make test 全绿 **20657P/0F** + clippy + fmt 干净。

bench-gate（zero-engine 定向）：同代码跨轮 FAIL 集 0/5/10 摆动、end_to_end
幅值 337µs↔859µs（2.5×）——并行流域负载噪声实证（stash 对照轮 0 FAIL 恰逢
静默窗；JS-only 修改不执行于 Rust 微基准）。不放宽。

## 残余 5F 精确记账

- **xhr/formdata.html 2 腿（structural）**：`populateForm` 依赖 iframe 编排
  （`<iframe>` + getElementsByName 反查——runner 无多 frame 管道）。iframe 字面量
  在**外部资源** targetted-form.js 内，绕过通道 `net_api_case_skipped` 的
  `<iframe` 源串筛减——通道卫生观察项（不扩筛，防误伤）。
- **constructor-formelement 1 腿（wire 层已知限制）**：unpaired surrogate 名/值
  经 `to_rust_string_lossy`（WTF-16→UTF-8）→ U+FFFD（R121 同型——CharacterData
  有 JS 侧缓存对策，属性面未建；script-sandbox ABI 面）。
- **constructor-submitter-coordinate + submitter-coordinate-value 2 腿（跨域）**：
  input[type=image] 图片加载（naturalWidth undefined、complete undefined——engine
  面未接）+ replaced-element 布局尺寸（盒 29×8 破图缺省——**layout-engine 面，
  渲染流域 §9 碰头记账**）+ SVG intrinsic sizing（render-foundation）。激活行为/
  坐标记录/构造器消费已就位（本切片），几何对齐后按
  `round(clientX − rect.left)` 即通（200×200 svg 期望 23/46 推导实证）。

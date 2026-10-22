# M3-S3 对象真存储——buffer 字节面 + 纹理参数/上传链 + framebuffer/renderbuffer 骨架

**日期**: 2026-10-22
**范围**: webgl-compat goal M3 切片③（master.md 切分序③）：buffer 真分配与
字节存储（bufferData 三形/bufferSubData 区域写/getBufferParameter）、纹理
采样参数（texParameteri/f + getTexParameter + GLES2 Table 6.9 初始值）、
texImage2D/texSubImage2D 上传校验链与 level 元数据存储、framebuffer/
renderbuffer 附件骨架（bind/framebufferTexture2D/framebufferRenderbuffer/
checkFramebufferStatus/getFramebufferAttachmentParameter/renderbufferStorage/
getRenderbufferParameter + FRAMEBUFFER/RENDERBUFFER_BINDING 查询）。

**基线**: `make testharness-webgl` **2112 Pass / 3 Fail 不变**（3 Fail 同 M2
上游 IDL 同 mixin 同长重载判据）；引擎常驻测试 part70 新增 3 组
（17 + 25 + 32 = 74 断言）+ part66-69 共 14 例回归全绿（17/17）；`make test`
exit 0（末套件 311 passed）；clippy `-D warnings` 零告警；fmt 通过；
`make reftest` 714/714（Layout 510 + Text 204）。

---

## 1. 实现面（part08.js 1711 → 1981 行；+270/−59）

### 1.1 buffer 真存储（GLES2 §2.5）

- bufferData 数值形分配 `Uint8Array(size)` 零填字节存储（`_zwData`），
  BufferSource 形拷贝持有，null 形 0 长度；三形均记 `_zwUsage`。重分配
  弃旧存储（§2.5 数据存储唯一持有）。分配上限 64 MB（`_ZW_MAX_BUFFER_BYTES`
  ——shim 实现预算，FIXME 注明随 M3-S4 render-foundation 接入对齐；超限
  INVALID_VALUE 防裸分配 OOM）。
- bufferSubData：无既有 store → INVALID_OPERATION（新增——M2 仅做容量域
  判定）；溢出/负 offset INVALID_VALUE 保持；域内写入
  `_zwData.set(src, offset)`。NaN offset 落 0 corpus 面（bufferSubData.html
  NO_ERROR 断言）保持。
- getBufferParameter：BUFFER_SIZE（0x8764）/BUFFER_USAGE（0x8765），
  初始值 0 / STATIC_DRAW=0x88E4（Table 6.8）。

### 1.2 纹理参数与上传链（GLES2 §3.7 + WebGL §5.14.8/§6.13）

- texParameteri/f 共享实现：参数名域（MAG 0x2800/MIN 0x2801/WRAP_S 0x2802/
  WRAP_T 0x2803——值按 pin rev webgl1.idl）+ 各名值集精确匹配（§3.7.7；
  texParameterf 非 integer GLfloat 不匹配任何枚举 → INVALID_ENUM）。
- getTexParameter：未设置返 Table 6.9 初始值（MIN=NEAREST_MIPMAP_LINEAR
  0x2702、MAG=LINEAR、WRAP=REPEAT 0x2901）。
- bindTexture 增首绑定型规则：同纹理改绑异型 target → INVALID_OPERATION
  （§3.7.13）。
- texImage2D（9 参形）校验链：上传 target 域（TEXTURE_2D + 六 cube face，
  TEXTURE_CUBE_MAP 本身非法 §3.7.2）→ 格式/类型域（Table 3.2 core 全集）→
  组合有效性（INVALID_OPERATION）→ internalformat≡format（WebGL §6.13）→
  level ∈ [0, log2(4096)]/border=0/尺寸 1..4096（INVALID_VALUE）→ 像素容量
  `w*h*bpp` 不足 INVALID_OPERATION（bpp 按 Table 3.2 尺寸列；UNSIGNED_SHORT_*
  恒 2 字节）。通过后落 level 元数据 + 像素字节（`_zwImages[target:level]`，
  拷贝持有）；pixels null → 已分配未初始化记账（data null，GLES2 §3.7.1）。
- texSubImage2D：同校验链（无 border 参）+ 既有 level image 前置（缺失或
  format/type 失配 → INVALID_OPERATION）+ 区域越界 INVALID_VALUE + 区域
  字节写（紧凑源行 → img.w 跨距；null data 先物化全零）。
- 6/7 参 source 形：保 M2 corpus TypeError 面（union 品牌判定不触源 getter）；
  元数据记 RGBA/UNSIGNED_BYTE 口径 + `_zwHasBase` 标记（压缩簇 interplay
  保持）；源解码未实现（FIXME 注明）。
- 无绑定纹理上传 → INVALID_OPERATION（M2 静默 no-op 收紧——corpus 不覆盖，
  spec §5.14.8）。

### 1.3 framebuffer/renderbuffer 骨架（GLES2 §4.4 + WebGL §5.14.14）

- 绑定复用 `_bindings` 表（FRAMEBUFFER 0x8D40 / RENDERBUFFER 0x8D41）——
  deleteFramebuffer/deleteRenderbuffer 的通用解除绑定机械免费生效。
- framebufferTexture2D/framebufferRenderbuffer：target/attachment 域校验
  （Table 4.3 四附件点）、textarget 域（TEXTURE_2D + cube face）、level=0
  （≠0 INVALID_VALUE）、已删对象 INVALID_OPERATION、texture/rb null 解除
  附件点；无绑定 FBO → INVALID_OPERATION。
- checkFramebufferStatus：默认 framebuffer → COMPLETE（drawing buffer 恒在）；
  空附件 → INCOMPLETE_MISSING_ATTACHMENT；无图纹理/无存储 renderbuffer/
  已删对象附件 → INCOMPLETE_ATTACHMENT；跨附件尺寸失配 → INCOMPLETE_
  DIMENSIONS；否则 COMPLETE。
- getFramebufferAttachmentParameter（§6.1.13）：默认 FBO 仅 OBJECT_TYPE 可查
  （返 FRAMEBUFFER_DEFAULT 0x8218，其余 INVALID_OPERATION）；空点 OBJECT_
  TYPE=NONE 其余 0；纹理附件四 pname 全答（OBJECT_TYPE=TEXTURE 0x1702、
  OBJECT_NAME 对象、LEVEL、CUBE_MAP_FACE 2D 为 0）；renderbuffer 附件查
  TEXTURE_LEVEL/FACE → INVALID_OPERATION。
- renderbufferStorage/getRenderbufferParameter：内部格式五值域（RGBA4 0x8056/
  RGB5_A1/RGB565/DEPTH_COMPONENT16 0x81A5/STENCIL_INDEX8——值按 pin rev
  IDL）、尺寸 1..4096、WIDTH/HEIGHT/INTERNAL_FORMAT/六 SIZE pname 面。
- 删除已挂纹理/renderbuffer：视同对**当前绑定** FBO 的解挂（§4.4.2.3），
  通用 delete 循环内附件点清空；未绑定 FBO 的悬挂附件保留、completeness
  按 incomplete 处理。
- getParameter 增 FRAMEBUFFER_BINDING（0x8CA6）/RENDERBUFFER_BINDING
  （0x8CA7）；MAX_TEXTURE_SIZE 改引 `_ZW_MAX_TEX_SIZE`（上传校验同源）。

## 2. 常驻测试（js_dom_bridge_tests/part70.rs 新增 3 组，part67 沙盒复用）

1. `test_webgl_buffer_storage_face`（17 断言）——SIZE/USAGE 回读（数值/
   BufferSource/null 三形）+ 无 store/溢出/负 offset INVALID + NaN offset
   corpus 面 + 重分配弃旧 + 域错误（坏 usage/pname/未绑定）。
2. `test_webgl_texture_storage_face`（25 断言）——参数四默认值 + 三参数
   回读（含 texParameterf 精确匹配与非整值拒绝）+ 坏 pname + 跨 target
   改绑拒绝 + 上传校验链七态（无绑定/CUBE_MAP target/组合失配/
   internalformat≠format/border/level 13/容量不足）+ 合法上传无错误 +
   子上传 base 前置/区域越界/null 像素物化/格式失配拒绝。
3. `test_webgl_framebuffer_renderbuffer_face`（32 断言）——默认 FBO COMPLETE
   + OBJECT_TYPE=FRAMEBUFFER_DEFAULT + 非法 pname INVALID_OPERATION +
   FRAMEBUFFER/RENDERBUFFER_BINDING 查询 + completeness 四态（空/无图/
   完整/尺寸失配）+ 附件参数四 pname 面（纹理/renderbuffer/空点）+
   attach level≠0 与坏 textarget + rb 存储回读（WIDTH/FORMAT/RED_SIZE）+
   删除纹理解除附件 + 无 FBO 挂点拒绝 + rb 格式/尺寸域错误。

## 3. 本轮修正的实现缺陷（冒烟 + 测试驱动发现）

| 缺陷 | 根因 | 修复 |
|------|------|------|
| bufferData(DYNAMIC_DRAW) 恒 INVALID_ENUM | M2 `_ZW_USAGE` 表含幻值 0x88E3 且漏 0x88E8 等 5 值（corpus 仅用 STATIC_DRAW 未暴露） | 按 pin rev webgl1.idl 值序改 9 值集（STREAM 0x88E0-2/STATIC 0x88E4-6/DYNAMIC 0x88E8-0xA） |
| WRAP_S/T 设参与查询全落 INVALID_ENUM | 手写枚举值 0x2902/0x2903 有误 | 按 IDL 改 0x2802/0x2803 |
| RGBA4/DEPTH_COMPONENT16 renderbuffer 格式被拒 | 手写 0x805F/0x84A0 与 IDL 值序不符 | 按 IDL 改 0x8056/0x81A5 |
| getRenderbufferParameter WIDTH/HEIGHT/FORMAT 恒 null | 手写 0x8D82/83/64（真机 GL 值）与 pin rev IDL 值序（0x8D42/43/44）不符 | 按 IDL 改（注释注明口径来源） |
| internalformat≠format 的上传未拒绝 | 首版共享校验器只查 format 组合 | texImage2D 增 internalformat≡format 判定（WebGL §6.13） |
| 附件 OBJECT_TYPE 返回值与上下文常量不同源 | 手写真机 GL 值 0x170A/0x84E8 | 按 IDL 常量口径改 TEXTURE=0x1702/RENDERBUFFER=0x8D41（corpus 断言与上下文常量同源） |

> **值序记账**：pin rev webgl1.idl（WPT 315976933870）多处枚举值与真机 GL
> 头文件不同（如 RENDERBUFFER_WIDTH 0x8D42 vs gl2.h 0x8D82、usage 值序带
> 组间隔）。本 goal 以 corpus 派生表为唯一值源（idlharness 断言同源），
> 运行时校验表与 getParameter 均从 part08a 派生值取齐；真机值序差异记为
> 上游语料特性，不另行对齐。

## 4. 已知边界（记账）

- **像素变换未实现**：UNPACK_FLIP_Y/PREMULTIPLY_ALPHA/COLORSPACE_CONVERSION
  按源字节直存；UNPACK_ALIGNMENT 行跨距按 1（FIXME 注明——语料含对齐敏感
  上传时按 INVALID_OPERATION 收紧）。
- **6/7 参 source 形仅记元数据**：源解码（canvas/image 位图搬运）未实现，
  dims 取源对象数值属性、像素字节留空（FIXME）。
- **压缩纹理路径不变**：compressedTexImage2D/SubImage2D 维持 M2 记账语义
  （`_zwHasCompressedBase` 消费面），压缩解码仍属 FIXME（本轮无语料依赖）。
- **completeness 骨架**：附件消费（采样/绘制读附件面）属 M3-S4 draw 簇；
  RENDERBUFFER_*_SIZE 仅列五格式。
- **fbo/_zwImages 惰性初始化**：create* 保持通用机械（无预置字段），首写
  时建表——对象字段面与 idlharness 无关。
- **文件行数**：part08.js 1981 行（<2000 但逼近）；M3-S4 draw 簇开工前须
  按职责拆分（候选：framebuffer/纹理上传簇独立 part08c）。

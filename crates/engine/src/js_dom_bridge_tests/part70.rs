// part70 — WebGL 对象真存储（goal webgl-compat M3-S3，2026-10-22）：buffer
// 分配/字节存储面 + 纹理参数与上传校验链 + framebuffer/renderbuffer 附件骨架
//（GLES2 §2.5/§3.7/§4.4 + WebGL §5.14/§6.13 口径）。沙盒沿用 part67
// `webgl_sandbox()`；错误码断言沿用 part67 corpus 批 B 先例。

/// buffer 真存储：bufferData 三形（数值/BufferSource/null）+ getBufferParameter
/// SIZE/USAGE + bufferSubData 域错误（无 store INVALID_OPERATION、溢出
/// INVALID_VALUE、NaN offset 落 0 corpus 面）+ 重分配弃旧。
#[test]
fn test_webgl_buffer_storage_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             function err() { return gl.getError(); }\
             var b = gl.createBuffer();\
             gl.bindBuffer(gl.ARRAY_BUFFER, b);\
             gl.bufferData(gl.ARRAY_BUFFER, 16, gl.DYNAMIC_DRAW);\
             r.push(gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_SIZE) === 16 ? 1 : 0);\
             r.push(gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_USAGE) === gl.DYNAMIC_DRAW ? 1 : 0);\
             r.push(err() === 0 ? 1 : 0);\
             var b2 = gl.createBuffer();\
             gl.bindBuffer(gl.ARRAY_BUFFER, b2);\
             gl.bufferSubData(gl.ARRAY_BUFFER, 0, new Float32Array(1));\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.bindBuffer(gl.ARRAY_BUFFER, b);\
             gl.bufferSubData(gl.ARRAY_BUFFER, 12, new Float32Array(2));\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.bufferSubData(gl.ARRAY_BUFFER, -1, new Float32Array(1));\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.bufferSubData(gl.ARRAY_BUFFER, 0 / 0, new Float32Array(4));\
             r.push(err() === 0 ? 1 : 0);\
             gl.bufferData(gl.ARRAY_BUFFER, 4, gl.STATIC_DRAW);\
             r.push(gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_SIZE) === 4 ? 1 : 0);\
             r.push(gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_USAGE) === gl.STATIC_DRAW ? 1 : 0);\
             gl.bufferData(gl.ARRAY_BUFFER, new Uint8Array([1, 2, 3, 4, 5]), gl.STREAM_DRAW);\
             r.push(gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_SIZE) === 5 ? 1 : 0);\
             gl.bufferData(gl.ARRAY_BUFFER, null, gl.STATIC_DRAW);\
             r.push(gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_SIZE) === 0 ? 1 : 0);\
             gl.bufferData(gl.ARRAY_BUFFER, -4, gl.STATIC_DRAW);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.bufferData(gl.ARRAY_BUFFER, 4, 0x88E7);\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             r.push(gl.getBufferParameter(gl.ARRAY_BUFFER, 0x999) === null ? 1 : 0);\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             gl.bindBuffer(gl.ARRAY_BUFFER, null);\
             r.push(gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_SIZE) === null ? 1 : 0);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "11111111111111111",
        "buffer 真存储：SIZE/USAGE 回读、无 store/溢出/负 offset INVALID、NaN offset \
         corpus 面、重分配弃旧、三形分配、域错误（M2 usage 表 0x88E8 缺失修正）"
    );
}

/// 纹理参数与上传链：texParameter 默认值/回读（Table 6.9 初始值）+ texParameterf
/// 精确匹配 + 跨 target 改绑 INVALID_OPERATION（§3.7.13）+ texImage2D 校验链
///（无绑定/非法 target/组合失配/internalformat≠format/border/level/容量）+
/// texSubImage2D base 前置/区域越界/null 像素物化。
#[test]
fn test_webgl_texture_storage_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             function err() { return gl.getError(); }\
             var t = gl.createTexture();\
             gl.bindTexture(gl.TEXTURE_2D, t);\
             r.push(gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER) === gl.NEAREST_MIPMAP_LINEAR ? 1 : 0);\
             r.push(gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER) === gl.LINEAR ? 1 : 0);\
             r.push(gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S) === gl.REPEAT ? 1 : 0);\
             r.push(gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T) === gl.REPEAT ? 1 : 0);\
             gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);\
             gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);\
             gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);\
             r.push(gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER) === gl.LINEAR ? 1 : 0);\
             r.push(gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S) === gl.CLAMP_TO_EDGE ? 1 : 0);\
             r.push(gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER) === gl.NEAREST ? 1 : 0);\
             r.push(err() === 0 ? 1 : 0);\
             gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, 9728.5);\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             gl.texParameteri(gl.TEXTURE_2D, 0x1234, gl.REPEAT);\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             gl.bindTexture(gl.TEXTURE_CUBE_MAP, t);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.bindTexture(gl.TEXTURE_2D, null);\
             var t2 = gl.createTexture();\
             gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(64));\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.bindTexture(gl.TEXTURE_2D, t2);\
             gl.texImage2D(gl.TEXTURE_CUBE_MAP, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(64));\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_SHORT_5_6_5, new Uint8Array(32));\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(64));\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 4, 4, 1, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(64));\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.texImage2D(gl.TEXTURE_2D, 13, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(64));\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(63));\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(64));\
             r.push(err() === 0 ? 1 : 0);\
             gl.texSubImage2D(gl.TEXTURE_2D, 1, 0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(16));\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.texSubImage2D(gl.TEXTURE_2D, 0, 2, 2, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(64));\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.texSubImage2D(gl.TEXTURE_2D, 0, 1, 1, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(16));\
             r.push(err() === 0 ? 1 : 0);\
             gl.texImage2D(gl.TEXTURE_2D, 1, gl.RGBA, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);\
             r.push(err() === 0 ? 1 : 0);\
             gl.texSubImage2D(gl.TEXTURE_2D, 1, 0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(16));\
             r.push(err() === 0 ? 1 : 0);\
             gl.texSubImage2D(gl.TEXTURE_2D, 1, 0, 0, 2, 2, gl.RGB, gl.UNSIGNED_SHORT_5_6_5, new Uint8Array(8));\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "1111111111111111111111111",
        "纹理参数默认/回读/精确匹配/跨 target 拒绝 + 上传校验链（无绑定/非法 target/\
         组合与 internalformat 失配/border/level/容量不足）+ 子上传 base 前置/区域越界/null 像素物化"
    );
}

/// framebuffer/renderbuffer 骨架：绑定查询 + checkFramebufferStatus 完整性
///（默认 complete/空 MISSING/无图 ATTACHMENT/尺寸失配 DIMENSIONS）+
/// getFramebufferAttachmentParameter 四 pname 面（默认 FBO/纹理/renderbuffer/
/// 空点）+ renderbufferStorage 域 + 删除纹理解除当前绑定 FBO 附件（§4.4.2.3）。
#[test]
fn test_webgl_framebuffer_renderbuffer_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             function err() { return gl.getError(); }\
             r.push(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE) === 0x8218 ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME) === null ? 1 : 0);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             var t2 = gl.createTexture();\
             gl.bindTexture(gl.TEXTURE_2D, t2);\
             gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(64));\
             gl.getError();\
             var t0 = gl.createTexture();\
             var fbo = gl.createFramebuffer();\
             gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);\
             r.push(gl.getParameter(gl.FRAMEBUFFER_BINDING) === fbo ? 1 : 0);\
             r.push(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT ? 1 : 0);\
             gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, t0, 0);\
             r.push(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT ? 1 : 0);\
             gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, t2, 0);\
             r.push(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE) === gl.TEXTURE ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME) === t2 ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL) === 0 ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE) === 0 ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE) === 0 ? 1 : 0);\
             gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, t2, 1);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_CUBE_MAP, t2, 0);\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             var rb = gl.createRenderbuffer();\
             gl.bindRenderbuffer(gl.RENDERBUFFER, rb);\
             gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, 4, 4);\
             r.push(gl.getParameter(gl.RENDERBUFFER_BINDING) === rb ? 1 : 0);\
             r.push(gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_WIDTH) === 4 ? 1 : 0);\
             r.push(gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_INTERNAL_FORMAT) === gl.RGBA4 ? 1 : 0);\
             r.push(gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_RED_SIZE) === 4 ? 1 : 0);\
             r.push(err() === 0 ? 1 : 0);\
             var t3 = gl.createTexture();\
             gl.bindTexture(gl.TEXTURE_2D, t3);\
             gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(16));\
             gl.getError();\
             gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, t3, 0);\
             r.push(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_INCOMPLETE_DIMENSIONS ? 1 : 0);\
             gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, null, 0);\
             r.push(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE ? 1 : 0);\
             gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, t2, 0);\
             gl.deleteTexture(t2);\
             r.push(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE) === 0 ? 1 : 0);\
             gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rb);\
             r.push(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE) === gl.RENDERBUFFER ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME) === rb ? 1 : 0);\
             r.push(gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL) === null ? 1 : 0);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.bindFramebuffer(gl.FRAMEBUFFER, null);\
             gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, t3, 0);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.bindRenderbuffer(gl.RENDERBUFFER, rb);\
             gl.renderbufferStorage(gl.RENDERBUFFER, 0x9999, 4, 4);\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, 0, 4);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "11111111111111111111111111111111",
        "framebuffer 骨架：绑定查询/completeness 四态/附件参数（默认 FBO、纹理、\
         renderbuffer、空点）/rb 存储-尺寸-格式面/删除解除附件/无 FBO 挂点拒绝/域错误"
    );
}

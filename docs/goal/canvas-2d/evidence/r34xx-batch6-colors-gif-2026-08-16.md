# R34xx 第六批证据：色彩管理 + 滤镜渲染 + GIF 解码（2026-08-16）

## 范围

color-type/wide-gamut-canvas/filters 深项 + GIF 深项落地——display-p3/linear
色彩管理、f16 浮点存储、colorMatrix 滤镜渲染、GIF 首帧解码。

## 本批修复（WPT driving）

1. **display-p3 色彩管理**（color-type 2/4 + wide-gamut 9/12 → 4/4 + 12/12）：
   - CanvasColorSpace + CSS Color 4 矩阵（sRGB EOTF + primaries）——
     put/getImageData 跨空间转换、fillStyle/strokeStyle/shadow 画布空间转换、
     color(display-p3) 直取 p3 通道、drawImage 位图源转换、resize 保留空间
   - f16 画布浮点缓冲（pixel_buffer_f32 + fill_color_f32——越界值 1.2249/−0.042
     精确存储；wire i32 直序列化）；put.basic.rgba.float16 + createImageBitmap.p3
   - linear 空间（srgb-linear/display-p3-linear——传输函数变体 + color() 直取 +
     getImageData 缺省 colorSpace = 画布空间）
2. **colorMatrix 滤镜渲染**（filters 13/13）：CanvasContext.filter_matrix +
   apply_filter_color（fillRect/路径/字形/图像源色）；shim _zwColorMatrix
   （matrix/hueRotate/saturate/luminanceToAlpha 20 值）；beginLayer 层滤镜
3. **GIF 首帧解码**（drawing-images + pattern.animated.gif）：decode_gif_bytes
   （GIF89a 逻辑屏幕/色表/LZW 变长码解码/透明色/偏移上采样）

## 最终状态

- color-type 4/4、wide-gamut 12/12、filters 13/13、layers 30/30、pixel-manipulation
  71/71、drawing-images 全过（GIF）
- path-objects 166/37（剩余 arc 几何——描边端帽/扇区形状深项）
- canvas 782 / engine 2151 / render-foundation 643 全绿；clippy 零警告

## 深项清单（剩余）

1. arc 几何 ~37（2d.path.arc.shape.*/arcTo/bezier/quadratic 描边端帽与扇区——
   stroke 几何深项，需端帽半圆/描边路径细化）

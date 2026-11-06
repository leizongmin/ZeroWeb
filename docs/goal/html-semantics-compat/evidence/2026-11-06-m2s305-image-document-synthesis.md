# M2 s305 — iframe image document 合成（spec read-image）：adopt-from-image-document ✗16.00%→✓0.00%，shim S184 text/plain 同型刀单页收敛

日期：2026-11-06。切片号 s305（承 s304 下一步候选①）。

## 勘根

**页面机制**（adopt-from-image-document.html）：iframe 先空载（about:blank）→
脚本赋 `iframe.src='resources/cat.jpg'` **导航到图片 URL** → 浏览器应合成
**image document**（HTML spec navigate → read image：image/* 响应建
body > 单 img 文档、img.src=最终 URL，
https://html.spec.whatwg.org/multipage/document-lifecycle.html#read-image）
→ iframe.onload 读 `contentDocument.body.firstChild`（=该 img）adopt 进
主文档渲染（页面自带 `img{width:initial;height:initial}` 反制真浏览器
ImageDocument UA 样式）→ ref=document-base-url-ref.html（裸 img，cat.jpg
320×240）。差幅 16.00%=76800/480000 整=cat 全缺。

**与 s299 姊妹页划界**：document-adopt-base-url.html（✓0.00%）用 iframe
**初始** about:blank contentDocument createElement img → 无导航无合成，
s299 修的是工厂 stub IDL src 恢复+base href 面——本页是**导航**面，两者
不同根。

**探针定谳**（zw-probe-s305.html，SetAttr 回写诊断入 mutated html，已删）：
修前 `doc=https://wpt.test/.../cat.jpg | fc=#text | bodyKids=1`——iframe
导航**成功**（s304 fetch 桥使 fetch/settle 通，doc.URL=最终图片 URL 正确），
但 `_zwIframeKindFromUrl` kind 分类（part01.js:2735 xhtml/html/svg/xml 四
臂）无 image 臂 → .jpg 落 'xml'，JPEG 二进制字节走标记解析 → body 仅一
#text 节点 → 收养的是 text 节点 → 渲染零像素。onload 确实派发（mutated
html 中 iframe 已 remove）。

**全树影响面盘点**：`<iframe src="image.ext">` 静态 0 页；JS 赋值导航仅
adopt-from-image-document.html（本页）+ img-ismap-coordinates-manual.html
（manual 页不入自动化通道）——修复合拍面恰 1 页。testharness 通道 image
contentDocument 断言页全树无。

## 施工（engine shim 单面，S184 text/plain 同型刀）

part01.js `_zwFinishIframeEntry`（image 臂两块）：

1. **kind 升臂**（S184 块后、`_zwMakeIframeDoc` 前）：Content-Type
   `image/*` 且非 svg → kind='html' + 合成壳
   `<html><head></head><body><img src="effectiveUrl"></body></html>`
   （src=绝对最终 URL，`&`/`"` 属性转义；image/svg+xml 排除——SVG 有
   自有文档类型走既有 kind 'svg' 标记臂）。
2. **contentType 回写**（S184 回写同型）：`entry.doc.contentType` =
   响应 MIME essence（壳工厂盖 text/html 须覆回）。

fetch handler 静态资产 CT 映射（testharness.rs:9808+）png/jpg/jpeg/gif/
webp/ico 全在 → 触发面覆盖完整；svg→image/svg+xml 落排除臂。

## 验证

- solo：adopt-from-image-document ✗16.00% → **✓0.00%**（首跑即绿）。
- 探针复核：`fc=IMG | src=https://wpt.test/.../cat.jpg | bodyKids=1`
  ——body > 单 img、src=最终 URL，#read-image 语义精确成立。
- 全树 A（修后）：**15981P/1031F**（17012 案=s304 D 同规模扣 3 探针键），
  vs s304 D key 级 join diff——F→P×7：**adopt-from-image-document（真实
  修复）**+middle-002/tao-149/tao-152/intrinsic-percent-007（s304 已裁定
  pool）+multicol-scroll-content/overflow-clip-margin-mul-column-border-box
  （css 布局 pool 族，本页无 iframe/image 面）；P→F×2：insert-block-in-
  inlines-middle-001、tao-154——**solo 3× ✓0.00% 全数直接裁定**。净真实
  **+1 F→P 零净 P→F**。
- 引擎 lib 套件（test-guard 包裹）：3765P/6F——6F=5 已知 CSSOM cssText
  红 + r5296 套件级 flake（s304 已裁定非确定性），**shim 变更零新增失败**
  （iframe 族断言全绿）。
- make test（rebase 后合流态 bd9cdcbbb+本 diff，guarded-test systemd scope）：
  workspace 腿照旧止于 engine lib 套件——**3766P/5F，5F 与 s304 已知集逐键
  相同**（5 已知 CSSOM cssText 红：p56_adopted_snapshot…/p99s_rule_style…
  /stylesheets_write_r2809/rule_style_r2810/add_rule_remove_rule_r3276，全部
  cssText 序列化族 engine CSSOM 面；r5296 套件级 flake 本轮未触发）——s305
  shim 变更与跨流合流（qv13 list tri-state）**零新增失败**；此前 80 腿
  test result: ok 全绿。已知面跨流注记归 rendering-compat（s304 同款，
  rule 9 记录不追）。

## 证据

日志（evidence/logs/，md5）：
- s305-probe-adopt-solo.log 3d90febe65968d3f96a4fd6337af3d66（修前 solo
  ✗16.00%+mutated html：iframe 已 remove、body 零追加）
- s305-probe-diag.log 8c9f363139f03076fb31c6fa62b73e70 / s305-probe-diag2
  .log 8bd589d5277091706f00eec6767655d6（探针修前 fc=#text / 修后 fc=IMG）
- s305-solo-adopt-fix.log 6f883658dc5c3d698557c264583a219e（修后 solo
  ✓0.00%）
- s305-reftest-full-a.log 93e23082a045b0eea73b809e17d5f33d（修后全树
  15981P/1031F）
- s305-adjud-insert001-{1,2,3}.log 1635000b… / 1635000b… /
  24702d66ad83f3e30e266c69e0ce2af6；s305-adjud-tao154-{1,2,3}.log
  8fcabfbfa328caa7fd8acdb3c74b9444×3（P→F 两键 solo 3× 全绿）
- s305-engine-lib.log 02e83b2e570ad0eef916cf2da15ffb28（3765P/6F 同
  s304 已知集）
- s305-make-test.log 66dd50f7eeb760d6e611fb7ab183b210（9721 行，workspace
  腿止于 engine lib 3766P/5F 已知集；guarded-test 合流态）

代码锚：part01.js `_zwFinishIframeEntry`（_zwIframeKindFromUrl:2735 四臂
无 image、S184 text/plain 块 2837-2843 同型先例、kind 升臂+contentType
回写两块）；testharness.rs:9808+（静态资产 CT 映射）；s304 探针法
（SetAttr 回写诊断入 mutated html，2026-11-06-m2s304-img-cluster-rootfix
.md 同款）。

## 下一步候选

1. animated GIF/WebP 帧推进（render-foundation decode 域，两页 2.08%
   同刀；s300 image_cache.rs 先例允许本域施工）。
2. print 仿真 faces（responsive-image-select-print 8.33% +
   image-srcdoc-relative-uri-print 1.04%）。
3. the-bdi-element 13 红页簇归属勘根（bidi 隔离，疑渲染域移交）。
4. 小簇排队（ol 6/textarea 4/meter 4/input 3/link 3/embed 3/object 2/
   option 1/iframe 1/br 1/wbr 1）。

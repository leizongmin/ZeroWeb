date: 2026-11-05
modules: engine,testing
---

# 自源探针「0.00% 通过」不可信——图元清单 + black-ref 强拆定谳伪绿

## 问题描述

R5423 rotateY 簇归因中，自源探针页（test 与手写 ref 成对）多次以 0.00%
通过。早期据此得出「3D 扁平化机制正常」的结论并推进了数轮探针设计；
随后对同一批页强拆 dump 揭穿：test 与 ref 双侧渲染全空，0.00% 是
blank-vs-blank 伪绿，先前的整条推理链作废。

## 根因分析

reftest 通过只证 test≈ref，不证两侧各自正确（guidelines #33 双侧同错
自洽伪绿）。自源探针的 test/ref 常共享同一套手写结构，一侧不渲染时另
一侧往往同因不渲染，lenient 默认容差下即 0.00% 通过。runner 报告尾部
的 DC-14「真通过-可疑（近纯色）」审计名单只分类不拦截，逐案审计靠自觉。

## 解决方案

判读纪律：0.00% 通过且近纯色 → 先按伪绿处理，强拆取证后再下结论。
取证三件（本轮实证有效）：

1. `REFTEST_DEBUG=1` 打印双侧图元清单（fills 计数 + rect/color）——
   `fills: 0` 即空渲染实锤，无需看像素。注意它**不打印** transforms 与
   path_fills，涉及变换路径时需第 4 条补齐。
2. black-ref 强拆：把探针 `rel="match"` 指向纯黑参考页（固定 div 铺满）
   强制失败，配 `REFTEST_DUMP=1` dump 失败侧 PNG——「通过」页的真实
   渲染现形。这是 dump-on-fail 机制下唯一能取到伪绿页像素的手段。
3. 探针页必须自带严格容差 `<meta name="fuzzy" content="maxDifference=0-1; totalPixels=0-9">`，
   否则 runner 默认百分比容差会把真差异吞成通过。
4. 单测直连（`parse_html` + `Parser::parse_stylesheet` +
   `StyleSystem::compute_styles` + `LayoutEngine::compute` +
   `Painter::paint` + `into_primitives`）跨 runner 复核图元层，可打印
   REFTEST_DEBUG 不含的 transforms 矩阵与 path_fills，定位「图元已正确
   发射、光栅未应用」这类分层分歧。
5. 像素级量取用 PNG bbox 而非目测：本轮 rotateY 红块
   bbox=(25,0,74,99) 精确 50×100 条证明光栅形变正确、GREEN bbox=None
   证明区域清白销毁了背衬——两条组合直接定谳机制，目测「红块约 8..75」
   这种读数会把方向带偏。

探针页（wpt-data 下 zz* 文件）用后即删；临时诊断测试随轮收尾移除并
还原 mod.rs 注册，避免目录哨兵噪声。

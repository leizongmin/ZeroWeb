# DC-2 切片 8（JS 侧批）：stream 残余 JS 面收口（2026-11-24）

## 本刀范围（纯 JS shim，4 文件，无 Rust 改动）

- `part15-positional.js`（+152/−42）：
  1. **原型分派面前移**：shim 同 realm 多次求值时 part03 无条件重建
     CharacterData/Text/Comment 原型链（2520/2533/2590 无卫门），旧版
     `__zwPosHtml` 幂等卫门 return 把原型面 stranded 在被替换旧原型上
     （foreign doc Comment 找 stream*HTML 全 undefined，探针 p13c/p2a 定谳）。
     修 = 原型分派移到卫门前，幂等 defineProperty 逐次重挂零副作用；补装
     stream 族 12 方法原型（旧版只装非 stream 十方法）。
  2. **replaceWith 占位语义**：写体落接收者自身（占位壳）；接收者为
     Comment/Text（不能承载子）铸真占位元素插 receiver 前位，写体落占位、
     close 拆壳换占位并移除 receiver（ReplaceWith 内容替换目标语义）。
     旧版容器误设 parentS——写体直落父尾（stream-positional「with a
     ref_node」族位置/时序双错）。
  3. **锚定校验**（stream-insertion-logic reject 12 案）：before 锚接收者、
     replaceWith 锚创建时 self.nextSibling、prepend 锚 container.firstChild、
     after 锚接收者；ref 摘除/换父后 write 拒 HierarchyRequestError（失效写
     不进 buffer，sink write 返 rejected promise＝流 error）。
- `part04.js`（+23/−21）：
  4. **inert mock 元素自序列化臂**：`__zwIsMEl` 印记节点优先走 outerHTML
     自序列化（mock 子树文本子 JS-only 无 handle，handle 臂产空壳——探针
     p14：`<span id=inner>A</span>` vs `<span id=inner></span>`）。
  5. **innerHTML 融合视图同源**：WC host 序列化内容面走 innerHTML getter
     同源融合视图（`_makeProxy` 显式 proxy 面——getter this 非保证
     receiver，裸 this 落 raw target 得空串）。旧版 host 平面索引优先
     （SetInnerHtmlOnHandle latest-wins 不含 append 族 wire 投影）——两冲刷
     写体序列化旧值（two-flush 面 36F）。
- `part12-trusted-types.js`（+17/−3）：
  6. **TrustedParserOptions 接口对象**：globalThis 曝光 + createParserOptions
     对象产物包装为品牌实例（pass-trusted 页 instanceof 断言；旧版裸对象
     无接口、globalThis 直读 ReferenceError 10F）。
- `part13-sanitizer.js`（+14/−0）：
  7. **replaceWithChildrenElements 规范化入 canon**（同 removeElements 形；
     旧版 canon 全缺该键 → verdict rWC 匹配恒落空——stream-html-sanitizer
     模板解包族）；validate dup/non-replaceable 检查随在席生效。

## 结果（domparsing 全腿，A/B 对 s7 干净基线 dc2-s7-clean-domparsing-full.log）

**1094P/441F/1T → 1212P/323F/1T＝+118P/−118F，5 页全限定、零未知漂移、
零回归**（前置轮 run1 1208P 的 sanitizer-safe −4P 已被轮末订正，本树终态
字节复验 28P/2F 持位）：

| 页 | s7 基线 | 本刀 |
|---|---|---|
| stream-html-sanitizer.html | 84P/120F | 154P/50F |
| stream-insertion-logic.html | 28P/24F | 52P/0F 全绿 |
| stream-positional.html | 46P/36F | 64P/18F |
| with-trusted-types-pass-trusted.html | 0P/10F | 6P/4F |
| stream-html-sanitizer-safe.html | 28P/2F | 28P/2F 持位 |

## 验证（协议同切片 7）

- domparsing 全腿（本树终态字节，test-guard 包裹）→ 本档 A/B。
- `cargo fmt --all -- --check` 零 diff。
- engine lib 测试 **4252P/0F 持位**（test-guard 包裹）。
- TT / sanitizer-api / web-components / dom 四套件 A/B（stash 过境 HEAD 态
  前腿 → pop 后腿，同 1800s 守卫窗同 corpus pin；前后腿相隔约 30 分钟）：
  - **trusted-types：逐位恒等** 1538P/552F/64T（零漂移——
    createParserOptions 品牌包装零身份断言反噬）
  - **sanitizer-api：+26P/−26F 六页全净改善零回归**（basic-filtering +6、
    config +6、get +1、modifiers +4、parseHTML +4、sethtml-tree +5＝
    replaceWithChildrenElements canon 外溢兑现；历史微刀挂账
    config 24F→18F、modifiers 14F→10F、get 2F→1F）
  - **web-components：逐位恒等** 17905P/4107F/18T（22k 测试零漂移——
    part04 innerHTML 融合视图对既有 WC 面零扰动）
  - **dom：−28P 单页**＝`Range-mutations-dataChange.html` 1161P→1133P（双侧
    0F/1T 恒）＝**Range-mutations 截断抖动已知噪声带**（dc1 判读基线 ±45P；
    master.md 碰撞记录停摆形窗间漂移 678–1161 实证）——非回归归因：本刀
    四文件不触 part01f nodeValue/textContent/data/replaceData 直写域（批
    二十四原嫌疑面），F/T 恒等、漂移幅度带内。**solo 复跑钉死**：post 二进制
    单页 FILTER 复跑三窗 1161（pre）/1133（post）/1006（solo 复跑）＝同码形
    窗间漂移 155P 且 solo 反低于双腿＝停摆点逐窗漂移直接复证（碰撞记录带
    678–1161 内），零代码归因。

A/B 前后腿全量日志原存 /tmp/r262-{tt,san,wc,dom}-{pre,post}.log（tmpfs 易
失）；入库 evidence/：dc2-s8-domparsing-full.log（主证）、dc2-s8-sanitizer
-{pre,post}.log（外溢证）、dc2-s8-tt-{pre,post}.log（恒等证）、dc2-s8-dom
-{pre,post}.extracted.log（Range-mutations 族 47 行/侧＋页级 TO 形，抖动归
因证；dom/wc 全量 8MB/4.4MB 未入库——恒等由本档 tally 记录作证）。

## 残余挂账

- stream-html-sanitizer 50F：元素配置管路（attributes/removeAttributes 方向
  在流消毒路径未生效）×24、模板内容消毒 ×12、config 快照语义（streaming 中
  变异不生效）×12、foster parenting rWC ×2。
- stream-positional 18F：replaceWith 双参形态 ×10、template content 路由 ×8
  （engine 代管父——协议刀域）。
- pass-trusted 4F：before/after 位置的 TrustedParserOptions runScript/
  sanitizer 覆盖未透传。
- stream-html-with-declarative-shadow-root 18F、stream TT sanitizer 8F。
- 协议刀域（下一片主菜）：insert_adjacent_html 36F、createContextualFragment
  11F、stream 路由尾——`InsertAdjacentHtmlOnHandle` 突变（zero-protocol +
  apps/renderer 两端同步）。
- XML 域深项挂账不变（DOMParser xml 族 ~60F、XMLSerializer 30F、
  xml-serialization 17F、innerhtml-mxss 30F）。

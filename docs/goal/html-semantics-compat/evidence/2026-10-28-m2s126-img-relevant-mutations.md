# M2 s126 · img relevant mutations 三根因收口（referrerpolicy 空串态 + SOURCE 移除位置锚 + lazy width/height 臂——relevant-mutations 121/121 全 Pass，真翻红 0）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s125（input 克隆状态传播收口）

## 入场定刀位

全量基线（/tmp/s125-full2.log）翻拣 the-img-element 域：
relevant-mutations.html 整页 46F。先勘本地资源面（s126 前置动作，非代码
修复）：corpus 依赖 `wpt-data/images/green-2x2.png`，gitignore 本地语料
缺该文件 → 46F 中 40 行是资源缺失假红。生成本地 2×2 绿 PNG（不入提交）
后 relevant-mutations.html 46F→6F（+40 翻绿，真翻红 0；无 corpus 行依赖
该资源的其他域不受影响）。余 7F（relevant-mutations.html 6 + relevant-
mutations-lazy.html 1）本轮三根因代码修复收口。

## 勘根定谳（7 失败逐一证据链闭合）

1. **referrerpolicy ×5**（absent/empty → no-referrer-when-downgrade、
   n-r-w-d → absent/empty/invalid，均 src already set、expect load）：
   `__zwAttrState5277`（part03）把 referrerpolicy 的 missing(null)/
   invalid/empty 全归并 `'no-referrer-when-downgrade'`（旧 spec invalid
   default 语义）——状态比对判「未变」→ 不触发重验。现行 spec
   （HTML §referrerpolicy-attribute）missing value default 与 invalid
   value default 均为**空串态**，n-r-w-d 是独立关键字态。Pass/Fail 分界
   线 100% 吻合：凡涉及 `'no-referrer'` 新旧态的 7 行（真变迁）经 avail
   重派 Pass，其余 5 行（归并态等值）Fail。
2. **"parent is picture, previous source removed"**：removeChild trap
   在节点摘除**后**调 `__zwImgTreeMutated5277`，helper 内 `after` 旗标
   依赖 `kids[i] === node`——kids 已不含被移节点，`after` 永不置位，
   SOURCE 移除臂死码。
3. **lazy "width attribute changes"**（whatwg/html#8492）：`img.width =
   '4'` 经 part05 维度反射 setter 写 width 属性（moAttr='width'）汇流
   `__zwImgAttrMutated5277`，IMG 分支无 width/height 臂 → 不触发。
   lazy 语料其余 7 行（style.width/height、display、loading 属性）全
   timeout 且本就 Pass，非缺口。

## 施工（engine shim part03.js + part04.js，无 Rust 变更）

1. `__zwAttrState5277`：referrerpolicy 分支 null/invalid 缺省值
   `'no-referrer-when-downgrade'` → `''`（空串态），注释同步现行 spec；
   crossorigin 分支不动（missing='missing'、invalid='anonymous' 语义
   正确）。
2. `__zwImgTreeMutated5277` 增第 5 参 `afterAnchor`（former next
   sibling 位置锚）：移除分支改锚定位——从锚（含）向后重验 img；锚为
   null = 移除节点原为末子（其后无 img）不重验；锚缺省（plain 子路径
   R195/R271 无位置捕获）保守回退全量重验。插入分支原逻辑不变。
   part04 removeChild sel 路径尾调（R188 `_r188Next` 已在移除前捕获，
   var 提升 same closure 可及）补传第 5 参；plain 子路径调用点不变。
3. `__zwImgAttrMutated5277` IMG 分支补 width/height lazy 臂：loading
   属性归一 `'lazy'` 时 → `__zwImgRevalidate5277`（#8492；同 URL 命中
   available-images 重派 load）。eager/无效 loading 不触发——lazy 语料
   timeout 行（style/display/loading 变更）不受影响。

## 验证

- 定向 `FILTER=the-img-element/relevant-mutations`：**121/121 Pass**
  （relevant-mutations.html 113 + relevant-mutations-lazy.html 8；修复前
  114P/7F）。7 行翻绿含 src-absent referrerpolicy 族保持 timeout-Pass
  （null selection `_nkPrev` 无 → 静默）与 state-not-changed 族保持
  Pass（''===''、大小写归一）。
- the-img-element 全域 A/B（/tmp/s126-tie-img.log →
  /tmp/s126-tie-img2.log，资源面已同态）：**827P/110F/17T →
  834P/103F/17T**，逐行 join 翻动恰为本修复 7 行 Fail→Pass，真翻红 0、
  Fail→Timeout 漂移 0、gone/new 0（parse-a-srcset 一行既有 ×2 两轮
  一致）。
- 全量 A/B × /tmp/s125-full2.log（s125 基线，**先于** green-2x2.png）：
  **6690P/1118F/103T → 6737P/1071F/103T**，47 Fail→Pass 配平干净、真翻
  红 0、漂移 0。47 = 40（green-2x2.png 资源面，img 域 A/B 已证与代码无
  关）+ 7（本修复）。首跑 test-guard 单进程内存超限杀一次（4.31GB>4GB），
  复跑过——记为并行摆动。
- engine lib **3570/0**；wpt-runner **256/0**（76 ignored）；guard 内
  clippy（engine+wpt-runner --all-targets -D warnings）零告警；fmt 净；
  shim 拼接（js_dom_bridge.rs 装配序）node --check 过。
- 语料零改动（探针页 zw-probe-s126.html 验后删；green-2x2.png 为
  gitignore 本地资源不入提交——其他机器/CI 上相关 40+12 行仍因资源缺
  失假红，与 s126 前置动作同注记）。runner 侧无代码改动——常驻语料
  relevant-mutations*.html 121 断言即回归面（与修复同提交）。渲染面零
  依赖——reftest/product-smoke 未跑。

## 挂账（本切片更新）

- **img relevant mutations 收口移出候选**（referrerpolicy 态 + 移除锚 +
  lazy width/height 三面）。
- **green-2x2.png 语料资源分发**：reftest/corpus 资源入库策略待决（须
  用户拍板——gitignore 本地数据在其他环境缺资源假红面）。
- naturalWidth 22F（SVG viewBox 渲染尺寸）/ current-pixel-density 11F
  （srcset 密度 clientWidth）——渲染流面（css-parser/style-system/
  layout-engine/render-foundation），让位不施工。
- 承袭不变：form-submission-0 186F、execution-timing 构成面、
  lazy-loading 26F、meta color-scheme 21F、写流执行身份盖章断链 FIXME、
  part02 setAttribute 绕 notify 臂、041 nested write、radio 组排他其余
  置真现象、deep clone 非 HTML ns 面、css-module corpus 门（待决）、
  www2 可达性、module-import-referrer TLA、dynamic import() with 透传、
  classic 外链 SRI FIXME、143 write、defer 多写完成序、接口对象基建面、
  FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live 树 document 级
  逗号列表 qSA 根缺口；**待决（须用户拍板）**：css/text 数据模块语料资
  源缺失、parser form element pointer 4F 缓行（2026-10-23）。

## 下一步候选（s127）

承袭挂账序复核：form-submission-0 186F POST 传输契约勘根（大基建评估
先行）/ execution-timing 构成面单根（056/063+051/052 增量解析）；或全量
基线翻拣下一单根面（本轮域翻拣刀法同款——整页整簇 Fail 勘根优先级高于
大基建挂账）。

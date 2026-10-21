# M2 S3 SVGSVGElement 面——动画控制 stub + currentTranslate/currentScale + deselectAll + viewBox 负维度（2026-10-21）

本 session 端到端持有 S3（S2c 收口后接续，工作区仅含 S3 改动基础上起步；无会话
重叠）。两文件落地：part03.js（SVGSVGElement 原型面 + viewBox 清零）、part04.js
（元素 proxy set trap 的 S3 名称门控 accessor 派发）。一个提交。

## 实现

### part03.js S3 块：SVGSVGElement 面（spec SVG2 struct.html §InterfaceSVGSVGElement）

- **动画控制状态机**：pauseAnimations/unpauseAnimations/getCurrentTime/
  setCurrentTime——per-element 状态 `{t, paused, scale}` 经 `_zwSvgAnimCache`
  缓存（`_zwSvgAnimKey(el, '_zwSvgCtl')`）。setCurrentTime float 校验
  （WebIDL float，非有限 TypeError）。FIXME(svg-compat M4)：spec 的 document
  timeline 应同文档所有 svg 片段共享且由 SMIL 时钟驱动，本实现是控制面 stub；
  共享语义 corpus 暂无消费者，M4 SMIL 接入后合流。
- **currentTranslate**：[SameObject] 只读 SVGPoint——getter-only accessor +
  `_zwMakeSVGPoint` 产物经 `_zwSvgAnimCache`（`_zwTranslate` 键）缓存。
  SameObject-identity 的 `DOMPointReadOnly: root.currentTranslate` 断言由
  undefined 恒等假绿转真对象真绿（targeted run 已核）。
- **currentScale**：outermost svg（直接父不在 SVG ns——body 子/fo 内/无父均算，
  `_zwSvgIsOutermost` 走查 parentNode + namespaceURI）可往返；嵌套 svg getter
  恒 1、setter no-op（inert，WPT currentScale-outermost 三断言面）。
- **deselectAll**：清文档 selection——ownerDocument.defaultView.getSelection()
  优先、doc.getSelection()/globalThis.getSelection() 回落，removeAllRanges 于
  try/catch 内（无 browsing context 静默——WPT deselectAll 无选区调用不抛）。
- **viewBox 负维度 → 零矩形**：`_zwMakeSVGRect` cur() 4 数解析后
  `width < 0 || height < 0` → 整矩形 `{0,0,0,0}`（WebKit tentative 行为——
  viewBox-invalid-attribute-baseval.tentative 注记跨引擎 pending：Gecko/Blink
  留原值）。

### part04.js set trap：S3 名称门控原型 accessor 派发（R98 镜像）

**实现期发现**：元素是 sel/handle trap proxy，get trap R93 尾部有原型链 accessor
回落（以 proxy 为 this 派发），set trap 无对应通用分支——原型 setter 恒不可达，
赋值落 expando。S2c 以前的只读语义（baseVal 赋值静默）依赖 wrapper 对象（plain
对象标准 [[Set]]）而非元素 proxy，故未暴露。

- 后果面：`nested.currentScale = 5` 绕过 setter 落 expando（嵌套 inert 失效，
  getter 被遮蔽返 5）；`svg.currentTranslate = x` 写 expando 遮蔽 get 回落
  （[SameObject] 身份断裂）。
- 修复：set trap 增加名称门控分支（`currentScale`/`currentTranslate`，R98
  同款 8 层原型走查）：命中 accessor 有 setter → 以 proxy 为 this 调用（异常
  传播——float 校验 TypeError 可达，同 adoptedStyleSheets 语义）；getter-only
  → **静默拒绝不落 expando**（sloppy 只读赋值语义）；data property → 交回常规
  分支。基线 currentScale outer 往返两案系 expando 假象（实例自由属性），派发
  后转状态路径真实现。

## 验证

- 机制探针 5/5 Pass（gitignored wpt-data/svg/s3-probe.html；一轮修正——首轮
  2F 即上述 set trap 缺口，修复后全绿）：动画控制往返 + float 校验/
  currentTranslate 同身份 + 只读赋值静默/currentScale outermost 判定（body 子、
  fo 内、嵌套 inert）/deselectAll 清选区/viewBox 负维度清零。
- SameObject-identity 81/81 保持全绿（`DOMPointReadOnly: root.currentTranslate`
  真对象身份 Pass——S2c 挂账假绿复核关闭）。
- **全 corpus（486 案，探针剔除）：734P/928F/40T/0NR = 43.1%**（含 NR 分母
  口径）——S2c（681P/965F/32T/18NR = 40.2%）以来 **+53P/-37F**，全绿案
  76→110（+34），逐案对账**零 DOWN 案**。分母 1696→1702（+6）：脚手架释放
  smil-clock-value-parsing 18 sub 全跑（基线全 NR）+ animateMotion-rotate-angle
  1→7 sub。
- **S3 靶面 4 案全绿**：currentTranslate-SameObject 1P/3F→**4/4**、deselectAll
  0P/7F→**7/7**、currentScale-outermost 2P/1F→**3/3**（嵌套 inert 经 set trap
  派发修复）、viewBox-invalid-attribute-baseval.tentative 0→**1/1**。
- **动画面脚手架释放（S3 主目标）**：30 案翻绿——smil-clock-value-parsing
  18NR→**11P/7F**（+11P）+ 29 案 0P/1F→1P 全绿（invalid-value 族断言"非法值
  不动画"，空 SMIL 下平凡满足、真引擎同过；M4 分类账复核面）。
- **Timeout 32→40（+8，挂账面分类移动非 Pass 损失）**：seeking-events-1..8 +
  conditional-processing-02 共 9 案 F→T（SMIL seek/事件等待挂起——无 SMIL
  时钟，事件永不触发）；animateMotion-rotate-angle 1T→7F（脚手架释放转真实
  SMIL 断言 F）。两族均 M4 分类账输入。
- 全 workspace `make test` DC-4 门禁：EXIT=0（结论见 master.md 轮次记录）。

## 排片对位

- getScreenCTM/getCTM 族 **SKIPPED** → rendering-compat 协同项：需 CSS
  transform 感知的布局合成（祖先 scale/rotate + transform-origin、abspos、
  margin），非 SVG 文档面单侧可落地。
- 动画面（animations/ ~80 案消费 SVGAnimationTestCase 脚手架
  pauseAnimations/setCurrentTime）脚手架释放：crash-F → 真实 SMIL 断言 F，
  M4 分类账输入（S3 落地后 M4 SMIL 分类账须重跑）。
- 下一步 M2 S4 几何面（S4a getTotalLength/getPointAtLength 纯路径数学 →
  S4b getBBox → S4c pathData 评估）。

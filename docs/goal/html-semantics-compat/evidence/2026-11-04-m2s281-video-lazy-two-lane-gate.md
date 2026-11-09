# M2 s281 — video loading=lazy 两车道门 + poster 独立链（video-loading 族 13F → 10P/5F/1T）

日期：2026-11-04。切片号 s281（承 s280 候选① lazy 族红群分型——勘根
判定为过时承接转 video-loading 新候选）。3 文件 3 块（engine shim 单
刀，零 Rust 改动，+216 行）。

## 勘根

1. **s280 候选①过时承接**：img family 全量实跑（s281-imgfamily.log）
   **38P/1F**——唯 multiple-times（s278 已销案签名）；s280 账面
   「in-scroller ×5 + base-url ×2 既有红群」不存在（早期切片已收
   口）。候选作废，转勘 video-loading 族（the-video-element/ 13 红
   + 3 绿，rendering 域 boundary 内最大语义红群）。
2. **两车道发现**（settle 栈回查探针）：静态解析 `<video src>` 的
   装载**不经** `_zwMediaScheduleLoad`（那是脚本车道：IDL src/
   setAttribute/source 子元素/load() 四汇流点），而是宿主 fetch 完
   成 commit → `__zw_commit_resource_element_state` →
   `_zwSettleResourceSelector` → `_zwSettleResourceKey`（唯一静态
   车道，readyState 4 实证）。门必须双车道同置——单门脚本车道则静
   态页（load-deferred/preload×2/window-onload/source-poster）照
   旧全载（b1 实证 4 页 readyState 4）。
3. **script 创建元素无 rect-bridge 快照**：append 后无布局拍，gBCR
   全零（探针实证）→ drain 唤醒 near=false → 永远 re-defer。可救
   面 = 显式滚动意图：scrollIntoView 旗标臂**置于 rect 早退之前**
   （b1 首版 sweep 落早退后被吞；前移后 autoplay-when-visible 转
   绿）。不可救面 = in-viewport（脚本创建 + 无滚动断言）→ 文件级
   Timeout（挂账②）。
4. **stylesheet 类规则不进布局几何**（定位探针）：inline
   `style="margin-top:1000vh"` → top=13008 正确；corpus class
   `.below-viewport{margin-top:1000vh}` → top=8 恒定（ poisoned）
   ——布局只认 inline 样式不认 `<style>` 类规则。距离门 near 对
   class 定位元素恒真 → 静态车道 below-viewport 族（load-deferred/
   preload×2/source-poster 步1）本 goal 不可修复（渲染域边界：
   差异归 rendering-compat，挂账①）。**poster 链据此排除 near 判
   定**（near 参与即误取 poster-deferred「不取」绿面）。
5. **video 固有尺寸不进布局**：媒体装载后 offsetWidth 恒 784（探
   针实证）→ load-when-visible「宽度变化」断言面本 goal 不可修复
   （挂账③，渲染域）。
6. **poster-when-visible 形态澄清**：页 video **无 src**（探针
   v2 dump：pend/reg/rs/sched 全空误判排除后读页实证）——纯
   poster 取数面，media load 车道不参与。poster 链独立建模：
   pending 册（helper 生命周期：登记/更新/弃册，永不 fetch）+
   旗标唯一 fetch 通路（唤醒体 flush）。resource 条目经页 fetch
   通道挂 `__zwEmitResourceTiming`，404 响应同样入 buffer（探针
   实证——poster 资产 /media/2048x1360-random.jpg 缺席不影响断
   言面）。
7. **旗标 flush 与册循环次序**：唤醒体重入（册循环内）先跑、
   poster flush 后跑——旗标放行的唤醒重入经 helper 复判保留在册，
   flush 补 fetch；eager 化重入经 helper 复判弃册不取。poster 册
   不随 media 册早退（poster-when-visible 无 media 册条目也须走
   flush 段）。

## 施工（3 文件 +216 行）

- **part06.js**（+174，4 区）：①唤醒机具块——`_zwMediaFetchPoster5281`
  （fetch+catch）、`__zwMediaPosterAttrMutated5281`（poster 册
  helper：lazy+poster 登记、非 lazy/弃 poster 弃册）、
  `_zwMediaLazyWake5281`（置 window-done 标记 + 册循环按 kind 分流
  重入 + poster 旗标 flush）+ drain/sweep 两薄封装 +
  `__zwMediaLoadingMutated5281`（仅唤醒册中有记录者）；②
  `_zwMediaScheduleLoad` 头部 video lazy 门（defer：冻结六参入册 +
  poster helper；过门：离册 + helper 复判）；③
  `_zwSettleResourceSelector` 头部 video lazy 门（commit 参数入册
  kind:'commit'）；④window load dispatch 后 drain 调用（img drain
  同位）。
- **part04.js**（+31，3 区）：scrollIntoView VIDEO 旗标臂（**先于
  rect 早退**——旗标置位 + sweep）；setAttribute/removeAttribute
  loading 变更臂（`__zwMediaLoadingMutated5281`）。
- **part05.js**（+11，1 区）：IDL set trap loading + poster 两汇
  流臂。

## 验证

- **family A/B**（FILTER=video-loading，16 结果行；stash 法，
  test-guard + RAF 同机）：A **3P/13F**（eager/in-scroller/
  poster-deferred 三绿）→ B **10P/5F/1T**——**+7P**：autoplay-
  deferred、autoplay-when-visible、disconnected、not-rendered、
  poster-when-visible、to-eager、window-onload（日志
  s281-family-{a,b}.log）。残 5F+1T 全部挂账面：source-poster/
  load-deferred/preload×2（挂账①）、load-when-visible（挂账③）、
  in-viewport F→T（挂账②——deferred 永不唤醒 → pending:1 文件级
  Timeout）。
- **全树 A/B**（stash 法，subtest 级 (state,path,subtest) 归一
  sort -u）：A 8345 键（7861P/422F/62T）/ B 8343 键（7866P/413F/
  64T），**FIXED=8 / REGRESSED=0**（日志 s281-fulltree-{a,b}.log）：
  - 7 键恰靶 = family +7P 同集（subtest 级）；
  - checkable-active-onblur「following label radios」F→P =
    checkbox `:active` 漂移族既有混态侧摆（s238/s239/s245/s246/
    s248 记档；s279 同页同向侧摆先例）——改动臂零通道，记基线态
    不归因；
  - LOST/GAINED 各 2 键非回归：①in-viewport 子测 F 失 + 文件级
    Timeout 键得 = 挂账② defer-forever 形态（目标域内，账面在
    案）；②link-multiple-load-events 3 子测失 + 文件 Timeout 键得
    = **s157 已记贴边页**（案 1-4 竞速窗 8s 名义 vs ~10s 预算、基
    线固有溢出 ~1.3s 余量 0.7s 噪声敏感）——B 全量跑撞并发负载窗
    （同机 ZeroWeb-2 会话 make test 活跃），solo 复跑 ×3 **6/6
    Pass** 解销（s157 账面维持，非本刀归因——改动臂在该页仅
    window-load 一次空册函数调用）。
- **img family**（FILTER=image-loading-lazy，39 行 38P/1F，日志
  s281-imgfamily.log）：s280 候选①过时承接实证。
- **make test**（日志 s281-maketest.log）：74 腿 73 ok + engine 腿
  FAILED exit 101——5 红与 s280 基线**逐名 diff 全等**（adopted-
  stylesheets/CSSOM 既有同集 R-t2qq 同因），workspace 腿 4G/8G 无杀
  树；总计 P9082/F5 双侧逐位同（s280 口径）零位移。
- 语法验证：runner 重建 3 轮（30.9s/32.1s/31.5s）+ family/全树
  7866 Pass 执行面证（单文件 node --check 不适配拼接设计，s273 起
  口径）。零 Rust 改动，clippy/fmt 不涉（s279 同口径）。

## 挂账更新

- **s280 候选①（lazy 族既有红群分型）：销案**——过时承接（img
  family 38P/1F，红群不存在）。
- **挂账①（新，移交 rendering-compat）**：stylesheet 类规则不进
  布局几何（`<style>` 类 margin-top 不生效）——静态车道
  below-viewport 族 4 面（load-deferred/preload-auto/preload-
  metadata/source-poster 步1）本 goal 域边界外；勘根证据定位探针
  （inline 13008 vs class 8）。
- **挂账②（新）**：script 创建元素无 rect-bridge 快照（append 后
  无布局拍）——in-viewport 文件级 Timeout 面待布局拍通道。
- **挂账③（新，渲染域）**：媒体元数据固有尺寸不进布局（装载后
  offsetWidth 恒 784）——load-when-visible 宽度变化面。
- **poster 资产缺席注记**（s227 惯例）：`/media/2048x1360-
  random.jpg` 语料包不存在（404）——resource 条目照常产出、断言
  面不受影响（探针实证）；补资产走上游 sparse clone 并在
  update-wpt-data.sh 挂账（非断言消费型，不阻塞）。
- **link-multiple-load-events 贴边页 reconfirm**：s157 账面维持
  （余量 0.7s 噪声敏感 + 全局边际开销翻键机制在案）。
- 承前不动：root margin 净速分档（s271 挂账①）；below/in-viewport-
  dynamic Chrome-only 口径（s272/s277 挂账延续）；runner 全量钉内
  存大户二分（s280 销案后无残余）；dynamic import() referrerpolicy
  穿参；decode() 兜底元组细化（s276 注记）。

## 下一步候选

1. below/in-viewport-dynamic Chrome-only 口径勘根（s272/s277 挂账
   延续——inflight 阻塞链调度 vs spec delay load 字面冲突面）。
2. root margin 净速分档（s271 挂账①——仅当语料扩出连接类型依赖
   断言面时重估）。
3. video-loading 残 5F+1T 渲染域挂账移交核（rendering-compat 通
   道受理面确认——①③两项 + 语料资产缺席项）。

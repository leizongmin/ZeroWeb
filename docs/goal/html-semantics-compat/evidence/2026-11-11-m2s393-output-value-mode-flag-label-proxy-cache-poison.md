# M2 切片 393（s393）——output value mode flag 双态 setter + label 转发壳 proxy 缓存投毒（R315 翻转守卫）

日期：2026-11-11（承 s392 下一步：双小刀施工轮——①output value mode flag 1F；②label img 递归 2F）
域：html/semantics/forms/the-output-element + the-label-element
性质：**施工轮——part05.js 双面修复，各自 solo 定向验证 + 全树验收 sweep**

## 刀① output defaultValue setter：value mode flag 双态（1F→P）

**driving 面**：`the-output-element/output.html :: output value and defaultValue`（s392 分诊：`output.defaultValue="10"` 后 value 仍 "5"）。

**spec**（HTML §4.10.12.2 API，#dom-output-defaultvalue）：output 的 value getter 活读 descendant text content（default value mode）；defaultValue setter 语义——**default value override 为 null 时**：String 值 replace all 写 child text content（override 保持 null，value getter 活读即新值）；**非 null 时**：仅覆写 override（value 模式，后代文本不动）。

**旧实现偏差**：恒写 override + 以 `_outputValue[key]`（dirty flag 镜像）空判写文本——`defaultValue=` 先把 override 吞成非 null，dirty 空（脚本从未设 value）时写文本读视图 stale（裸 host 写不进 JS pending 融合视图），value getter 返回旧后代文本 → 1F。

**修复**（part05.js defaultValue setter，OUTPUT 臂）：
- override 为 null → host 文本写 + `_zwRegisterTextEl` 本地文本重注册（与 value setter 的 R49/r2846 视图同步同链——裸 host 写不进 pending 融合视图，同 turn 读新值须本地重注册）；空串走 `_zwUnregisterTextEl` 对偶。
- override 非 null → 仅覆写 override（value 模式后代文本不动——output.html「defaultValue is set to 15」value/textContent 保持 20 断言面）。

**验证**：`FILTER=output` solo **5P/0F**（含 driving output.html 与 mutations.window.js 三案——reset/后代突变联动面全绿）。日志 /tmp/s392/solo-output-fix2.log。

## 刀② label 转发壳自递归：R315 proxy 缓存翻转投毒（2F→P）

**driving 面**：`the-label-element/clicking-noninteractive-unlabelable-content.html` 2F（img 臂：first-child 位形 + deeply-nested 位形；second-child 位形过）——`Maximum call stack size exceeded`。

**勘根**（栈捕获 + 身份探针 8/9/10 三轮，探针文件用后即删）：

- **栈签名**：`RangeError ~ at _makeProxy (63204) ~ at child.addEventListener (73165:38) ~ at child.addEventListener (73165:78) ×∞`——R4004 盖章壳（`child.addEventListener = function...`）转发臂 `return _p.addEventListener(type, fn, opts)` 反复重入自身。
- **触发矩阵**：模板 content `cloneNode()` img + **handle 容器**（createElement div）下挂 + 任意 `addEventListener` → 递归；createElement img 直建（t8，页面持有 proxy）、模板克隆直挂 sel 父 label（t1，`_recordHandleChild` 早退无翻转）、tagName 只读（t10）全过。
- **身份探针定谳**（probe 8）：失败形态 el 是 **plain stub**（`[object Object]` + gOPD data:function own 壳 + stable）；对照 createElement img 是 **proxy**（`[object HTMLImageElement]` + gOPD none + stable=false = get trap 命名臂全新闭包）。
- **时间线探针定谳**（probe 9/10）：el 的 `__zwHandle` 与 R4004 壳在 `div.appendChild(el)`（**挂树前**）即齐备；此刻调 `el.addEventListener` 已递归——毒点在 plain 子入 handle 容器的 append 机械内。
- **因果链**：R368 为无身份 plain 子盖章 `__zwHandle`（part04 盖章臂）→ R4004 四事件方法转 live 壳（own data prop）→ append 机械内 '@'+h 建 canonical proxy 后 `_recordHandleChild` 的 **R315 重挂翻转**（part05 `_proxyCache['@'+h] !== child` 即写）把 **plain el 写入 `_proxyCache`** → 壳转发 `_makeProxy(null, h)` 命中缓存取回 **el 自身** → `_p.addEventListener` 即壳 → 无限自递归。`_makeProxy` 契约恒返 proxy 被翻转破坏。
- **R315 本义**（part05 注释在案）：remove 窗口期 cache miss 重建 proxy B，页面持有旧 proxy A，重挂时翻转缓存回 A 保 sibling identity。其前提是 `child` 是 **页面持有的 proxy**——R368 盖章 plain stub 混入该路径即投毒。

**修复**（part05.js `_recordHandleChild` 翻转守卫）：翻转目标须无 own `addEventListener`——盖章 stub 的 own 转发壳被精确排除（proxy 的方法走 get trap 命名臂无 own 数据属性，gOPD trap 对方法名恒 miss；页面持有的 proxy A 不携带该 own 键，R315 重挂翻转面零变化）。翻转被跳过后缓存保持 canonical proxy，壳转发命中 proxy 命名臂 → listener 落 `_listenerStore['@'+h]` live 表——正是 R4004 原设计意图（clicking-noninteractive-* 盖章克隆监听 live 派发可达）。

**验证**：`FILTER=clicking-noninteractive` solo **60P/0F**（原 2F 全翻转：img first-child/deeply-nested 两案 Pass，全部 6 标签族 × 3 位形绿）。日志 /tmp/s392/solo-lbl-fix1.log。

**未定谳细节**（不阻塞）：append 机械内为 '@'+__n1 预建 canonical proxy 的确切调用点未定位（R368/R4004 臂静态扫描无 child-handle `_makeProxy`——嫌疑在 host 桥回调或容器 append 深层）；守卫对任意预建来源均语义强制（翻转不得装非 proxy），故不追。

## 全树验收（sweep join）

- 基线：s392 清探针后预测 8199P/231F/37T = 8467 唯一 (key,state)。
- 实测（/tmp/s392/n-sweep.log，8 job 全语料）：**8466 行 = 8201P/228F/37T；unique 8463 (key,state)**（3 重复 Pass 行 = runner 已知同文件多跑现象，键级口径取 unique）。
- join 归因（base-b = s391 n3.pairs − s392 删除探针 37 行 = 8467，LC_ALL=C (key,state) comm，空格保序 sed 提取——awk $0 重建折叠子测试名连续空格曾致 325 假位移重提）：
  - **3F→P**：label img×2（`<img>` / `<img tabindex="">` deeply nested under label）+ output defaultValue——恰为本轮双刀三案，零多修零漏修。
  - **−4P**：zw391_probe.html 四面——s391 轮内已删探针文件，键随语料消失（s392 预测基线漏扣此 4 行，**勘误**：全树唯一键预测 8467 → 实况 8463；红键口径 231F 不受影响）。
  - 零 P→F、零 Timeout 位移、零意外键。

## 验证

- solo 双刀：solo-output-fix2.log（**5P/0F**）、solo-lbl-fix1.log（**60P/0F**）。
- 全树 sweep join：见上节——**3F→P 精确归因 + 零意外位移**。
- 单元测试：test-guard 包裹 `cargo test -p zero-engine`（本流 Rust 零改动，shim JS 变更面）**3833P/7F**——7F 与 s391 组合态在册红完全同形（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch / response_body_readable_stream_r2967 / t8k_websocket ×2），零新增红。
- 产品 smoke：`make product-smoke` **PASS**（struct-check 0 issues）；`make product-smoke-legacy` fixtures 42 跑 **struct-check failures: 0**（与 s391 39/39 struct PASS 同形）。
- 探针 hygiene：zw393-probe-lbl.html 用后即删（`git ls-files` 先核——gitignore 本地数据，无跟踪态冲突）。
- **教训钉（双）**：①test-guard total-mem 是本树预算——与并行流（ZeroWeb-3 clone）build 同机争用时 `cargo test -p zero-engine` 两次被杀（8862728/10064888 KB > 8388608 KB），`-j 2` 降并发后过守卫，阈值不动；②后台轮询 done 旗标须核 **mtime 新鲜度**（旧运行残值 done 会冒充新运行结果——本轮第一次轮询误读被杀运行残值 124 为新运行完成）。
- fmt/clippy：本流零 Rust 改动，不适用（s391 先例：仅本流触面执行）。

## 下一步

- **s394**：textarea wrap=hard 折行变换（2F）+ img unloaded attr sizing 归属探针（2F：engine attr→style 面还是 layout 未加载盒面）。
- 挂账池维持：渲染流 5 面（basic 亚像素 / error alt-box / svg-zoom / lineheight / sizes-auto 几何半边）、form-submission-target 6T→navigation M3、sizes-auto lazy 半边待勘根；A2+D 簇立项征询待用户决策。

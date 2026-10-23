# M2 切片 39 — 挂账：focus-in-shadow 跨流收口（2026-10-23）

## 范围

s38 挂账：focus-after-close「moved to shadow root」案红点在
`shadowRoot.activeElement`；dialog-focus-shadow-double-nested 2F
（turnIntoShadowTree 装配 + 嵌套 shadow autofocus 委派）。

## 探针钉定（zz-probe-s39-shadow，验证后已删）

四层根因，逐层推进：

1. **sel 模板 content 子列表 per-read 重建**：`_r145SelKids` 深路径
   每读经 `__zw_template_contents_deep` JSON 重建条目对象，跨读
   mutation 落空——turnIntoShadowTree 首读 content 拿 inner template
   条目、replaceWith 换 div 后二读 content 重建成未替换的 template
   （spec content getter「Each time returns the same
   DocumentFragment」）。
2. **content 条目 parentNode 指模板 wrapper**：`_tcParent =
   _wrapSelector(sel)` 使条目 replaceWith 沿 parentNode 读到 wrapper
   融合 childNodes 而非缓存数组，mutation 落错列表（探针 D：
   `pidIsView=false pidTag=TEMPLATE`、replaceWith 后数组不变）。
3. **`_zwMEl.replaceWith` 缺插入半边**：R117 轻量实现只 remove 不
   插入（concept-node-replace 的 pre-insert 群缺失），plain 树
   replaceWith 替换者整体落空。
4. **focus delegate 非 dialog 分支缺 get-the-focusable-area 递归**：
   嵌套 shadow host（div 无 tabindex + delegatesFocus）不是 focusable
   area 即被跳过，焦点落树序后一个 focusable（外层 x 按钮）而非内层
   [autofocus] 锚（探针 E：`osraeTag=BUTTON`≠btnAF、`israeTag=null`）。

## spec 钉定

- the-template-element：content getter 恒返同一 DocumentFragment，
  contents 子的 parentNode 是 fragment 本体（视图 mutation 路径
  `c.parentNode = _tplContent` 同款语义）。
- concept-node-replace：replaceWith = pre-insert 群到 node 前 +
  remove node（dom.spec.whatwg.org/#concept-node-replace）。
- focus delegate 步 4：对 whereToLook 每个后代跑 get-the-focusable-area
  （interaction.html#focus-delegate）——非 focusable 后代持
  delegatesFocus shadow 时递归取其 autofocus 锚。

## 落地（R5241 四刀 + R5242 一刀 + R5241 收尾失效）

1. **part04 `_r145SelKids` 首读物化缓存**（R5241）：深 JSON 条目
   首建后缓存 `proxy._zwTplContentKids`，缓存数组即活树（append/
   insert/remove 直接 splice）。已知边界：非 innerHTML= 的宿主侧
   模板子树变化不回灌。
2. **part04 parentNode/parentElement getter 槽位消费补 parentHandle
   分支**（R5241）：dual-form 槽（s38 容器挂载覆写）消费——sel 子挂
   容器后 focus chain 上行经此达 shadow 边界；parentElement 对非元素
   容器父返 null。
3. **part04 `_tcParent = _tplContent`**（R5241）：深路径条目父 =
   content 视图本体（惰性引用安全——`_r145SelKids` 仅经视图
   getter/方法触达）。
4. **part03 `_zwMEl.replaceWith` 补插入半边**（R5241）：plain 父
   （非 `__zwHandle`/`__zwSelector`）本地 splice 插入替换群（含字符串
   转 Text）；host proxy 父保持 remove-only 旧行为（wire 面由 proxy
   trap 负责）。
5. **part01 focus delegate 非 dialog 分支补 get-the-focusable-area
   递归**（R5242）：`area = _zwDialogGetFocusableAreaR4600(d)`——与
   autofocus delegate 循环同款三元；dialog 分支 seq-focusable 优先序
   （child-sequential-focus 断言面）不动。
6. **part04 innerHTML= setter sel 模板缓存失效**（R5241 收尾）：
   TEMPLATE 目标 contents 重写后 `_zwTplContentKids = null`——同视图
   身份下子全量更替（R5196 lib 面：同模板三次 innerHTML= 每次重读
   新解析树）。仅失效子缓存，`_zwTplContentView` 身份不动（spec
   content getter 恒返同一对象）。

## 集群验证

- dialog-focus-shadow-double-nested **2F → 2P 全绿**（show/showModal
  双案，GAIN 2）。
- focus-after-close 6 subtests+1T：**2F+1T → 1F+1T**（moved-to-shadow
  root 案绿，GAIN 1）。
- dialog-focus-shadow 50/50 全绿不变；dialog-focus-previous-outside
  3F（declarative shadow DOM parser 面挂账）不变。
- interactive-elements 全域 98 files 482 subtests：416P/64F/2T →
  **419P/61F/2T，REG 0 / GAIN 3**。
- forms 域 401 files：3899P/707F/20T/1U → **3902P/707F/20T/1U，
  REG 0 / GAIN 3**（GAIN 为 the-label-element/clicking-noninteractive-
  unlabelable-content 尾部 3 案——s38 跑时文件尾截断未注册，本轮
  跑全 51 案，共同 48 案逐位一致，非行为变化）。
- 数据 [2026-10-23-m2s39-focus-in-shadow.json](2026-10-23-m2s39-focus-in-shadow.json)。

## 已知边界

- focus-after-close 余 1F：「Complex dialog usage」focus 恢复红
  （activeElement 得 div）——dialog focus 恢复深层面，既有挂账。
- 文件级 Timeout 2（s32 起既有挂账）不变。
- sel 模板 content 缓存不回灌非 innerHTML= 的宿主侧子树变化（记录
  于刀 1 注释）。
- dialog-focus-previous-outside 3F：declarative shadow DOM（parser
  面联合切片，控制面已定性跳过）。

## 门禁

zero-engine lib **3431P/0F**（R5196 首轮红于缓存失效缺失，补刀 6 后
复绿；基线 3430 +1 为渲染流新增）；clippy `-D warnings`、fmt clean；
零新增依赖。runner release 二进制重建（含内核重编）。R5241/R5242
撞号检查通过。

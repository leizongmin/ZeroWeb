# M2 s135 · select reset 默认层捕获——reset-form select ×3 收口（真翻红 0）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s134（reset 按钮 click 激活接 form.reset()）

## 入场定刀位

s134 挂账「select-by-reset R3909 3 面」入场：reset-form.html select
×3（reset() 方法 + button click + input click，激活臂均经 s134
接通的 `form.reset()`）——`form.reset()` 时 select 的 selectedness
未回落默认层。

## 勘根定谳

1. **selectedness 三层读链（option.selected getter 序）**：
   `_optSelectedDirty`（dirty，条目存在即权威）→
   `_optSelectedClean`（显式初始/写侧 spill 混合层）→ 宿主
   `__zw_option_selected`（callbacks.rs:1824，感知 renderer
   SelectOption 编程选中，回落快照 attr）→ selected 属性 lw 链。
2. **单根 = apply 侧改写快照属性破坏默认层**：`select.value=`
   setter 经 `__zw_select_option`（callbacks.rs:1847）推
   `DomMutation::SelectOption`，apply 臂（js_dom_bridge.rs:1405）
   改写快照 selected 属性（mark 目标 + deselect 兄弟）——此后
   live attr 承载的是编程选中态，不再是默认层。
3. **reset 链穿透**：`form.reset()` 清 shim dirty/clean 键
   （s133）后，读链穿透到宿主层——宿主返回 stale 编程选中
   （如 o2=true，而默认层 o2 无 selected 属性），遮蔽属性层；
   `_zwSelectednessSetting` 走查同链读 true → last=o2 → fallback
   不触发。探针页 shim 内埋点 pre/post 快照实证该穿透点，排除
   脏态存活、双 select 干扰两误判。

## 施工（engine shim 3 文件 5 处，无 Rust 变更）

1. part01：`_optSelAttrDefault = {}` 声明（默认层捕获 store）+
   `__zw_reset_form_state` 清链同步清空。
2. part03：`_zwOptCaptureSelDefaults(sel, handle)` helper——
   `_zwListOfOptions` 走查，`!hasOwnProperty` 时按 key 只填缺
   （fill-once）捕获 live attr 布尔；**唯一钩点 = select.value
   setter**（SelectOption 唯一推送源；selectedIndex= /
   option.selected= 只写 JS 层不改快照 attr，无需钩）。
3. part04：value setter 调捕获；reset SELECT 分支重写——删
   dirty/clean 后显式落
   `_optSelectedClean[k] = capture ?? live-attr`（遮挡宿主
   stale），再跑 `_zwSelectednessSetting`。

## 验证

- 定向：resetting-a-form **14/14 全绿**（select ×3 F→P，
  reset() 方法与 click 两通道同链收口）。
- 域回归：八域键级 join（the-input-element 1644 / constraints
  920 / form-control-infrastructure 121 / the-form-element 108 /
  the-button-element / the-select-element / the-option-element /
  the-textarea-element）**零位移**。
- 全量 join（对 /tmp/s134-full.log，7914/7914 行）：**恰 3
  Fail→Pass，真翻红 0、new non-Pass 0**（6803P/1008F/103T →
  6805P/1005F/104T）。唯一非 Pass 变动 =
  the-img-element/update-the-image-data/src-then-lazy-load.html
  子测试 Pass → 文件级 Timeout（lazy-loading 网络等待面，与
  select 改动零关联，文件级超时边界波动，前轮同形）；join 另报
  3 条 NEW 为 dup-key 副本假阳性（两份 log 同形 Pass）。
- engine lib：**3571/0**（4 线程 + 默认并行第 3 次；间歇 flake
  定谳见教训）。wpt-runner bin **256/0**（76 ignored）；clippy
  （zero-engine --all-targets -D warnings）零告警；本轮零 Rust
  变更，fmt 不适用。shim 拼接（13 part）node --check 过。探针页
  验后删；语料零改动。全量 runner 以 corpus 已知 Fail 非零退出
  （make 错误 1，前轮同形非异常）。

## 已知限制（挂账）

- constructed-option（`new Option(..., true)` 第 4 参）在带
  value= 的 select 中被捕获为 false（捕获只读 live attr）——
  无 corpus 面。
- reset 后宿主 SelectOption 不同步：renderer 快照 attr 仍是
  value= 时编程态，渲染面存在 stale 窗口；JS 读已正确（clean
  遮挡宿主），corpus 断言面走 JS 读无渲染断言。
- 捕获时机（首次 value=）之后用户改 selected 属性的 stale 窗口
  （fill-once 不更新）——corpus 无面。

## 挂账（本切片更新）

- **select-by-reset 3 面收口**（reset-form select ×3）。
  reset.html form element pointer 面仍归 parser form pointer
  待决项缓行不变。
- **新挂账**：engine lib 默认并行偶发
  test_details_name_exclusivity_template_deep_entry flake
  （负载敏感型，待独立勘根）；上述已知限制 3 条。
- **button-events 文件级 Timeout 收窄待续**（挂起 1）+
  「type=button shouldn't trigger submit」submit 误派发异根、
  part03 raw 链 reset 臂无还原臂（UA 指针激活平行近似，corpus
  无面）承袭不变。
- 承袭不变：form-nameditem 残余 10F、form-submission-0 186F、
  execution-timing 构成面、lazy-loading 26F、meta color-scheme
  21F、写流盖章断链、part02 setAttribute 绕 notify、041 nested
  write、deep clone 非 HTML ns、css-module corpus 门（待决）、
  www2 可达性、module-import-referrer TLA、dynamic import() with
  透传、classic 外链 SRI、143 write、defer 多写完成序、接口对象
  基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live
  树逗号列表 qSA 根缺口、渲染流面让位累积、s131 插入步缺口细化、
  radio 现象⑤ becomes connected / checked 属性增设 /
  defaultChecked 无独立 corpus 面（现象①覆盖）；**待决（须用户
  拍板）**：css/text 数据模块语料资源缺失、green-2x2.png 语料
  资源分发、parser form element pointer 4F 缓行（2026-10-23）。

## 过程教训

- **整行 sort ≠ 第 1 字段排序**：join 报「未排序」且输出行数
  膨胀（7919 > 7914）时结果不可信——`contentWindow.name|Fail`
  与 `contentWindow|Fail` 整行序（`.` 0x2E < `|` 0x7C）与第 1
  字段序（前缀更短者居前）相反。改用 awk 哈希 join 免排序坑；
  dup key（3 个）覆盖语义先确认两份同形再信位移输出。
- **suite 全套偶发失败先取分布再定责**：默认并行 2F/1P +
  4 线程 0F + 单测独跑 0F + 改动路径不相交（本轮唯一在 details
  测试路径上执行的改动是 part01 解析期 var 声明）→ 负载敏感型
  既有 flake 被边际扰动暴露；s134 HEAD 仅 1 个数据点无法证明
  基线免疫，按「非本轮逻辑引入」处置并挂账独立勘根，不冒充
  通过也不顺手修。
- **探针假象靠载荷转储纠正**：「select branch ran twice」
  误判重复执行，实为表单内两个 select 各走一条——JSON 转储
  载荷后一眼定谳；埋点结论前先确认断言单元粒度。

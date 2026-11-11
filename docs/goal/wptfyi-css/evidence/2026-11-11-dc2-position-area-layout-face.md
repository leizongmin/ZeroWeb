# DC-2 组F 布局刀2：position-area 区域定位布局 pass（anchorofsom 10,485 → anchorlay3 10,549 净 +64）

日期：2026-11-11。前序：[offset CSSOM spec 化刀1](2026-11-11-dc2-offset-cssom-spec-face.md)。
css-anchor-position-1 §position-area 布局期首次落地——此前 position-area 只有
声明面/CSSOM 面（grid G1 单实现双通道批 9893P），布局几何断言页全挂。

## 批内容（提交 5af93e7e0）

1. **css-parser `values/anchor_area.rs`（新增，区域格矩阵单点下沉）**：
   canonical 串 → 3×3 网格区域格 `((row_s,row_e),(col_s,col_e))`。单实现
   双通道：CSSOM 声明面（engine `dom_bindings::position_area::canonicalize_decl`
   产出的 canonical 形）与布局期共用；矩阵下沉 css-parser（layout-engine
   不依赖 zero-engine，依赖方向单向）。文法面：
   - 单词折叠形：物理/逻辑单轴 long 关键字隐式轴恒 span-all（`left` ≡
     `span-all left`）；center/span-all/逻辑 short 族两轴同用。
   - 双词形：`axis_of` 显式轴归类（author 原文序不保证——stylesheet 声明不经
     shim canonicalize）；同轴双词非法拒 None。
   - 双 shared 词对（short/center/span-all/self 短族混形）：**首词=block、
     次词=inline**（css-anchor-position-1 两值语法约定 + WPT
     position-area-ambiguous-self 页 `'self-end self-start'` htb/rtl = 物理
     bottom right 证据）。
   - self 轴（`self-*`/`span-self-*`）按**元素自身** direction 解析：
     `position_area_cells_dir(canonical, self_inline_rtl)` 元素 rtl 时 inline
     轴格物理镜像（(3-e,3-s) 对称翻转）；block 轴与非 self 轴不镜像。
   - `none` → None（无区域定位）；表外词拒。
   - 单测 6 组（折叠形/canonical 序/self 前缀与 span 形/双 shared 词对/rtl
     镜像）。
2. **style-system 四属性承载**：ComputedStyle `anchor_name`/`position_anchor`/
   `position_area`/`position_try_fallbacks`（小写原文 `Option<Box<str>>`，
   None = 未声明/none）+ `apply.rs` 四臂 + Default。文法校验单点仍在 engine
   `dom_bindings::anchor_props`（声明面批 431d229e5 同矩阵）；布局期对非法形
   `position_area_cells_dir` 返 None 不介入（等价 invalid→none）。
3. **layout-engine `engine/anchor_pos.rs`（新增，abspos 修正 pass 同架构）**
   ——`apply_anchor_position_area`（engine.rs 11.7c 挂点，taffy 主布局后）：
   - pass1 锚注册收集：`anchor-name` → 视口 border-box 几何，树序后者覆盖
     （position-anchor 默认锚 = preceding 树序最后匹配，§position-anchor）；
     无锚页零成本跳过。
   - pass2 自顶向下应用（嵌套 anchored 链外层先定型）：仅 positioned 直挂
     CB 盒（`is_abspos_cb` + child `is_absolute`）介入；区域四线 = 锚 border
     box 各边在 CB padding box 坐标投影（允许越界负值）；margin box 对齐
     区域（CSS2 §10.3.7，margin 委托 `resolve_abspos_real_length` 既有求值
     口径）；width/height auto 时格满填 stretch（content 尺寸同步内缩），
     显式尺寸不覆写（只定位）。
   - `ZW_ANCHOR_AREA=0` kill-switch（LazyLock 进程级，热路径不逐节点读
     env，R5465 同型）。
   - **fallback 门（语料实证）**：`position-try-fallbacks` 承载非 None 即不
     介入——迭代期试过对 fallback 页介入，净 **-5**（内置 flip tactic 需布局
     级区域溢出检查 + last-successful position 状态机；custom `@position-try`
     需 shim 期 fallback 应用引擎），静态位保持基线行为，fallback 面另刀。
   - 第一刀边界挂账：`position-anchor` 缺失（默认锚树序匹配）不介入、CB 链
     非直挂不介入、align/justify-self 非 stretch 蕴含不建模、非 htb 元素与
     CB 侧 writing-mode 逻辑轴映射另刀。
4. **layout-engine g1f e2e 单测 6 案**（position-area-basic oracle 场景：
   container 400×400 CB 内锚 150×75@margin-left 100、anchored 直挂）——
   区域满填/单轴 span-all/负越界投影/嵌套 anchored 链先外后内/margin 参与/
   default-on 形（kill-switch 回退形由 env 进程级定格不进单测）。

## 验证（anchorlay3 轮，15:15）

- **css-anchor-position 语料 253 页**：**10,485P/2,331F → 10,549P/2,267F
  净 +64，逐页零回归**——flip 唯一页 **position-area-in-position-try.html
  0P/64F → 64P/0F 单页全清**（@position-try 语境的 position-area 值×几何
  矩阵页：`none`/`span-all`/`top left` 等全值域区域格满填断言；页内 base
  区几何断言与本 pass 语义一致，`position-try` 简写不在承载面 → 不触
  fallback 门）。
- **css-color 漂移门**：10,905P 零漂移。
- **make test**：3866P/7F（7F 为 main 既有欠账组，与本刀零交集）。
- **门禁**：fmt EXIT=0；guarded clippy workspace EXIT=0
  （logs/clippy-g2final-*.log）。

## 残余分桶（anchorlay3 后 top 失败页，下一步直接取用）

| 桶 | 页面 | F | 缺面 |
|---|---|---|---|
| writing-mode 布局几何 | anchor-position-writing-modes-001 | 1,296 | anchor()+position-area 全值域 × 8 写作模式组合布局几何（本刀 htb 第一刀核心挂账） |
| wm/dir 区域定位 | position-area-wm-dir / align-justify-wm-dir | 44+56 | 非 htb 元素 wm 映射 + align/justify-self 非 stretch 蕴含 |
| anchor() inset 求值 | anchor-name-003 35 / anchor-position-004 32 / anchor-size-001 26 / anchor-size-writing-modes 24 | ~117 | anchor()/anchor-size() inset 求值布局（区域定位外的另一半布局面） |
| fallback 布局 | try-tactic-basic 30 / anchor-query-fallback 16 / last-successful 族 ~5 | ~51 | 溢出检查 + last-successful 状态机 + @position-try 应用引擎 |

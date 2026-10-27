# M2 s119 · HTMLFormElement 索引命名属性臂（031 focus/blur 事件面）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s118（innerHTML 产物惰性章 + location javascript: 求值面）

## 刀位与判据链

s118 下一步候选一（execution-timing 余 F 复盘）定刀 031「scheduler: focus and
blur events」1F。corpus 探针两轮（插桩断言面带出运行时值，备份 /tmp、验后手动
还原——wpt-data 为 gitignore 本地数据）：

1. 首探：`eventOrder = ["inline script #1","inline script #2"]`、activeElement=BODY
   ——script #1 在 `log('inline script #1')` 后**抛异常中断**（'end script #1' 与
   全部焦点事件缺席），嫌疑不在 focus 派发链。
2. 二探逐句包 try：`document.forms`（object len=1）→ `forms[0]`（FORM）正常，
   **`forms[0][0]` undefined** → `Cannot read properties of undefined (reading
   'focus')` TypeError 定谳。

静态对照：shim 焦点面成熟（part04:4473 focus trap——R114 detached/R148 迁移/
s29 模态门/R4003b label 前递，blur→focusout 失焦序齐）；R3254-K3 FORM named
access 块（part04:10490）只匹配控件 `id`/`name` 属性——**canonical 数字索引臂
缺失**（spec：无 id/name 的 listed 控件按 form.elements 集合序号可寻址），
`form[0]` 落空。

## 施工（净变更两文件）

1. **part04 FORM named access 块**：id/name 循环后追加 canonical 数字索引臂
   （parseInt + String 归一等，与 elements 集合 get 臂同判据——拒前导零/符号
   形）；命中返 `_formControls(sel)[idx]`（与 `form.elements[N]` 同一取源，
   proxy 恒等）；越界返 undefined 落 trap 通用路径。命名匹配先行、索引兜底
   （spec 支持属性名名单序）。
2. **testharness.rs**：bin pin `form_indexed_named_access_focus_s119`（form[0/1/2]
   索引取控件 + 与 elements[N] 恒等 + 越界 undefined + 命名访问不回归 +
   focus() 同步派发面）。

031 的焦点/点击派发面**零改动**——blur/focus 迁移序、already-focused no-op、
click 焦点转移全在既有 R114/R148/R57 面，缺的只是取元素入口。

## 验证

- bin pin：新 shim **绿**；旧 shim stash 往返**红**（行为失败：`form[0]`
  undefined → addEventListener TypeError，非编译失败）。
- 定向 `FILTER=execution-timing/0`：**031 F→P**，全行归一 diff 唯一变化，
  72P+5F+1T→73P+4F+1T。
- **全量 A/B**（/tmp/s118-full-suite.log → /tmp/s119-full-suite.log，全行归一
  diff）：**6610P+1188F+105T → 6612P+1186F+105T（+2P −2F ±0T）**；diff 两行对
  = 031 + **同面专测 `forms/the-form-element/form-indexed-element.html` 意外
  翻绿**（本刀位的上游专测页），其余状态行零变化 → **真翻红 = 0**。
- engine lib **3570/0**（注：与全量套件并发跑曾撞 artifact「No such file or
  directory」，串行重跑过——守卫包裹的双 cargo 并发注意排队）；guard 内
  clippy（runner --all-targets）零告警；fmt、shim 拼接语法过。

## 挂账（本切片新增/更新）

- 无新增挂账；031 焦点面刀位消费完毕（s118 挂账「031 focus/blur——焦点基建
  面单列」收口：实际根因是取元素入口，焦点基建本就完好）。
- 承袭：写流执行身份盖章断链 FIXME、part02 setAttribute 绕 notify 臂（gap 2）、
  execution-timing 构成面 056/063+051/052 单根（增量解析大基建）、041 nested
  write（写流队列重构）、FIXME(spec-gap) iah/clone fragment 章、css-module 族
  corpus 门（待决须用户拍板）、www2 可达性、modulepreload referrerpolicy 族、
  module-import-referrer TLA、script-crossorigin-network cookies、
  redirect-relative-urls、export-from with FIXME、script-charset-02/03、
  143 write、defer 多写完成序、dynamic import() with 透传、classic 外链 SRI、
  接口对象基建面、FIXME 组、/tmp/zw-hb*.txt；**待决（须用户拍板）**：css/text
  数据模块语料资源缺失、parser form element pointer 4F 缓行（2026-10-23）。

## 遗留近似

- 索引臂不处理 `form[Symbol.iterator]`/原型链形态（named access gate 仅 string
  prop；迭代走 elements 集合自身——站点无此形态）。
- named access 块的既有豁免名单（item/namedItem/reset/requestSubmit/submit）对
  数字索引无交集（canonical 数字串不与成员名冲突）。

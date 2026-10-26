# M2 切片 65（s65）——http-equiv-and-name 2F 分诊 + content-language pragma 回落（R5272）

日期：2026-10-24。document-metadata shim 面最后未分诊项收口：2F 分诊一渲染流
记账回流、一本流双刀修复（选择器复合伪类名字边界 + `:lang()` pragma-set
default language 回落），域 **58P/60F/18T → 59P/59F/18T**（GAIN 1 / REG 0），
document-metadata 分诊轮至此全归档。

## 1. 2F 分诊

- **http-equiv-and-name-1（1F）→ 渲染流记账回流**：断言 = inline
  `style.color = CanvasText` × `style.colorScheme = only light/dark` 的
  `getComputedStyle(div).color` 应用值（light/dark 须不同 + root 应用色）——
  与 s64 color-scheme 23F 同一条 applied-color 链，归并同桶。本流不修。
- **http-equiv-and-name-2（1F）→ 本流修复**：`<meta http-equiv=content-language
  content=de-DE>` 后 `document.querySelector(":root:lang(de-DE)")` 须命中
  `<html>`。修复前死于两层断链（下节）。

## 2. 修复面（R5272，dom crate 两刀）

1. **复合伪类名字边界**（`query.rs` `parse_selector_chain` 伪类臂）——伪类
   名字边界取 `(`/`#`/`.`/`[`/`:` 最早命中，仅当该字符是 `(` 才按含参伪类取
   args。旧实现 `r.find('(')` 扫过整段余串，`:root:lang(de-DE)` 把
   `root:lang` 当名字 → `parse_pseudo` 不识别 → SyntaxError
   （`__zw_selector_valid` 拒，shim 抛 `'...' is not a valid selector.`）。
   单伪类/双冒号伪元素/`:not(...)` 参数路径不变（`is(:hover)` 等含参 path
   的 args 提取与旧一致）。
2. **pragma-set default language 回落**（`document/lang_dir.rs`
   `matches_lang`）——祖先链（含根元素）全无 `lang`/`xml:lang` 时按 spec
   「language of a node」Otherwise 分支回落 pragma-set default language
   （https://html.spec.whatwg.org/multipage/dom.html#language）。pragma 候选
   逐行对应 Content language state 算法
   （https://html.spec.whatwg.org/multipage/semantics.html#meta-content-language-state）：
   须有 content、http-equiv ASCII 大小写不敏感匹配 `content-language`、
   content 含逗号失效、跳过 ASCII 空白取首 token；文档序最后一个合格 meta
   胜出（主树前序遍历——template contents / shadow root 在侧表不参与）。
   空 `lang=""` = explicitly unknown，立即 false 不回落（spec 同段）。HTTP
   Content-Language 头回落快照层不可达——FIXME 挂账。R3281 共享权威面：
   style-system `:lang()` CSS 匹配同步吃到回落（spec 正确，corpus 无
   meta-pragma CSS 案）。
   // OPTIMIZATION：回落路径每次全树扫描 O(n)——`:lang()` 罕见且 pragma 面
   冷，缓存须在属性 set/元素插拔全 mutation 路径挂失效，暂不引入。

回归测试：`query::tests::zz_r5272_root_lang_meta_pragma`（解析合法性 +
pragma 端到端命中 root/后代 + 非目标语言不命中）、
`zz_r5272_pragma_lang_eligibility`（含逗号失效 + 最后 meta 胜出 + 祖先 lang
压制 + 空 lang 不回落）。

## 验证

- http-equiv-and-name FILTER：**-2 Pass，-1 保持红**（死因即已归档渲染流
  断言）
- document-metadata 域 **59P/59F/18T**（+1 NotRun 为 CSP tentative
  style-element-csp-blocked「Test error event fires on inline style」行，与
  本修复无关；F 文件清单除 http-equiv-and-name-2 转绿外与 s64 台账一致）
- interactive-elements **439P/44F**（与 s64 逐值一致）；style-system lib
  **2449P/0F**（matches_lang 共享权威零回归）；zero-dom lib **868P/0F**
 （+2 为本轮新增）
- fmt / clippy（-p zero-dom --all-targets -D warnings）清洁

## 未解决项（document-metadata 余 59F+18T 挂账序，全归档）

- color-scheme 23F + **http-equiv-and-name-1 1F**（本片归并）+ 资源管道
  ~13F + applied CSSOM ~13F + style2 1F → 渲染流记账回流
- CSP tentative 7F+5T + non-parser-created-doc 1F → 底座挂账
- **域分诊轮收口**：document-meta shim 面已无未分诊项。下一域
  text-level-semantics 开片（M1 基线 21P/13F），分诊轮先行

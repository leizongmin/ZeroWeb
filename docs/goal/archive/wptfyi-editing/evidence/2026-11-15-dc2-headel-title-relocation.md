# headEl 持真 title 切片（d34/T 切片）——head 段解析前剥离 + headEl 独立解析 + R167 桥 headEl 搜

日期：2026-11-15（d34 会话，接 d33 批）
范围：part05 `_zwMakeIframeDoc` R159 分支（head 段捕获/剥离 + headEl 入链）、
part03 detHtml（doc 槽补发）+ `_zwWrapCached` R167 桥（headEl 真节点搜）
语料面：`delete-in-child-of-html`（主目标）、IP-html/head 姐妹守护、全语料对拍护航

## 结论

- `delete-in-child-of-html` **24P/22F → 34P/12F 净 +10**——d33 挂账 18F
  title 门控清账（10F 直翻 + 8F 复合形 title 分量消除，残形转 J2 块合并
  缺口）；全部守护面/哨兵/通道零漂移（见面账）。
- d22 挂账副产品兑现：`doc.title` getter 经 part03
  `_zwFindTitleElement` head 快道（先走 `doc.head`）**免修自愈**——iframe
  子文档 `doc.title` 从恒 `''`（d22 记档缺口）转正确读值。
- P2F 挂账核对：当前 selection 全量账窗口 P2F=2 全归 d31 unlink 臂既有
  欠账（见下节），**本切片 P2F=0**；d22 记录的 deleteFromDocument
  `[head,1]` 越界 P2F=1 在当前账面无对应红项（deleteFromDocument 族全
  Pass）。

## 三处修复

1. **part05 R159 分支（mEl[3] 之后）**：bodyInner 里 head 段解析前剥离——
   `/<head\b[^>]*>[\s\S]*?<\/head\s*>/i` 捕获整段存 `doc._r159HeadHtml`，
   bodyInner 只留 body 段。旧径 head 段留在 bodyInner，body 上下文解析把
   title/script 提为 body 树顶层子（D4 探针实证 body.innerHTML getter 返
   `<title>iframe</title><br>`），语料 setup `body.innerHTML = ''` 走 setter
   重置（_tree=null、bodyHtml=''）即毁 head 段——18F title 门控根因。
2. **part05 `doc.body.innerHTML = bodyInner` 之后**：head 段独立解析入
   headEl——`_zwMBuildBodyTree` body 上下文解析剥出的 head 内串，逐子
   `doc.head.appendChild`（s408 trio reparent + 兄弟接线），子树 R173
   `_zwOwnerTree` 盖章（ensureTree 同款）。语料 setup 重置自此只触 body 树。
3. **part03 detHtml**：hh179 空 + `doc._r159HeadHtml` 有值时补发——head
   内容迁出后 body 树序列化不再含 head 子，detHtml 包装层缺省补发保 R164
   轮转查询视图（title/meta/script 查询命中）。
4. **part03 `_zwWrapCached` R167 桥**：`_tree` 搜未命中时以非元素伪根
   （R307 fragment 同款——跳自身只递归子）搜 `doc.head.childNodes`，真节点
   自登记（R166 同向）。不扩则 JSON 往返命中 head 子时落 `_zwParseEl`
   wrapper 域（无 `.remove`——corpus `removeResourceScriptElements` 对
   qSA("script") 产物调 `.remove()` 46F 实证）。

## 探针定谳（zz-diag-d34，已删）

- **活数组迭代陷阱**：首版正序迭代 `_t159ht.childNodes` 逐子 appendChild，
  reparent 从源父 childNodes **splice** 使索引前移——隔一跳一（TITLE(0)→
  META 落、STYLE/SCRIPT 跳，kids=TITLE,META 复现）；改快照
  `.slice()` 后四件套全落。
- builder/工厂/s408 三环节逐一定谳无丢弃：host 深 JSON 三 API 全发 5 元素
  （title/style/meta/script/div）、`_zwMBuildDeepTreeNode`/`_zwMEl` 通用、
  createElement('style') appendChild 正常。
- host API 探针面与 DOM 变异面同页互扰（`__zw_parse_html_subtree` 页面态
  调用后 f2 contentDocument 读 null）——分文件复跑定谳，探针留档教训。

## 面账（d34 终态）

| 面 | 基线 | 实得 |
|---|---|---|
| `delete-in-child-of-html` | 24P/22F | **34P/12F** ✓ |
| `insertparagraph-in-child-of-html` | 64P/8F | 64P/8F 恒 ✓ |
| `delete-in-child-of-head` | 64P/4F | 64P/4F 恒 ✓ |
| `insertparagraph-in-child-of-head` | 64P/8F | 64P/8F 恒 ✓ |
| `zz-probe-b1s3`（哨兵） | 4P/9F | 4P/9F 恒 ✓ |
| `zz-probe-d31-unlink`（哨兵） | 8P/0F | 8P/0F 恒 ✓ |
| `unlink.html`（editing-run） | 391P/4F | 391P/4F 恒 ✓ |
| engine lib（/tmp/d19tgt pin） | 4032P/0F | **4032P/0F** 恒 ✓ |
| input-events 全量 | 223P/936F | 223P/936F 恒 ✓ |
| selection 全语料（R220 护航） | d30 账 | **F2P=65 / P2F=2**（见下） |

## selection 全量对拍（vs d30-selfull.json）

- **F2P=65**：del-html +20（d33 批 +10、本切片 +10）、IP-html +40（d33
  批）、editing-div-outside-body +5（d33 批 doc 级 join 臂窗口增益）。
- **P2F=2**：`styling-commands.html?styleWithCSS=false/true` 各 1 子测
  （unlink "should not update the DOM"——期望剥 `<a>` 实 no-op）。**归属
  实证**：stash 本切片重建后基线二进制同红 → 属 d31/d33 已提交窗口欠账，
  d31 只跑面未复全量漏账；非本切片引入（本切片 P2F=0）。已记 master 卡点
  待 d31 臂修（plaintext-only 宿主 unlink 剥 `<a>` 面）。

## del-html 余 12F 分解

| 形 | 计 | 归宿 |
|---|---|---|
| J2 跨 body 边界块合并缺口（div before/after body 未与 body 内 div 真合并——吸收臂展平单侧、无跨界块合并） | 8 | 独立后续切片（doc 级 walk 块合并臂） |
| S8 XOR 伪失败（invisible-head 子测 `assert_true(a.isConnected ^ b.isConnected)`——XOR 返 number，d17 族在册出队） | 4 | d17 伪失败族（任何浏览器同红） |

## 遗留挂账

- d31 unlink 臂 plaintext-only P2F=2（本批披露，基线实证）。
- FIXME(d33) 跨文档收养缺口不变（IP-html 余 8F，引擎级 adoptNode）。
- R255 分支（无显式 html 对）head 段本就整段丢弃——headEl 不补（本切片
  不扩 R255 面）；后续如需同形可复用本切片捕获/入链配方。

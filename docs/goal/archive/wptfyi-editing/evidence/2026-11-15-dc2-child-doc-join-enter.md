# DC-2 子文档 join/enter html 形精化（d33）

日期：2026-11-15（d33 会话，接 d31 批）
范围：engine shim B1 块级 join（`_b1JoinMembers`/`_b1Join`）、insertParagraph
拆半落位（`_b1Enter`）、空体占位归一（body 吸收臂）
语料面：`delete-in-child-of-html`、`insertparagraph-in-child-of-html`（head 姐妹面守护）

## 结论

- 子文档 html 根直下宿主的 join/enter 精化：**净 +50P**
  （insertparagraph-in-child-of-html 24P/48F → **64P/8F 净 +40**；
  delete-in-child-of-html 14P/32F → **24P/22F 净 +10**）。
- 四处修复（part05.js）：①`_b1Enter` doc 级拆半落位（clone 按
  `docEl.childNodes` 位次 +1 同槽位插——`host.nextSibling` 在 docEl 虚树恒
  null，旧径 ref=null 落 'after' 槽致 clone 排到 body 之后）；②body-caret
  直落臂（`cc.c === doc.body`——`_b1SeqRootOf` 对 body 本体返 docEl、
  `_b1MemberOf` 解析不到，直入 doc 级 join caretIsBody，方向侧无可见目标且
  对侧有则换向：空体两侧单边内容两键均期望并合）；③visible head join
  （walk 臂 head 项先试 `_b1HeadEdgeMember` 边缘可见子作为同层 join 成员，
  fam 边缘维持 no-op；`_b1Join` 新增 `root === doc.head` 分支——head 内成员
  边缘升级 doc 级 join、head 项挂内成员，非边缘维持 head 内成员 join）；
  ④空体占位尾缀剥离（body 包装视图含 R159 head 段，占位形如
  `<title>…</title><br>`——只剥尾缀 `<br>`，带属性/中间 br 不动）。

## 失败形态分解（修前 32F 逐形）

| 形 | 计 | 修后 | 归宿 |
|---|---|---|---|
| title eaten（setup `body.innerHTML=` 重置毁 head 段） | 10 | 0 变 | head 段提升切片（挂账） |
| J2 块合并形 + title 复合（吸收展平入 body，期望并入块） | 8 | 0 变 | 同上（T 门控） |
| S3/S3b 空体占位 `<br>` 拼合形 | 4 | **4P** | ②+④ |
| S4 body-caret 直落 no-join 形 | 2 | **2P** | ② |
| S9 visible head join 形 | 4 | **4P** | ③ |
| S8 `assert_true(a.isConnected ^ b.isConnected)` 伪失败 | 4 | 0 变 | **d17 伪失败族出队**——XOR 返 number，assert_true 须严格 boolean（错误面自证 `expected true got 1`），任何浏览器同红 |

## 跨文档收养缺口（insertparagraph 余 8F，engine 级挂账）

before-head 空体形用**父文档** `document.createElement` 建 div 再
`childDocEl.insertBefore(div, doc.head)`。真 DOM pre-insert 收养节点
（ownerDocument 随换）→ `EditorTestUtils` 的 window/selection 落子文档。
本仓主文档工厂产物是引擎侧元素，ownerDocument 原生绑定，JS 赋值与
`defineProperty` 均不可改（探针 D1 实证 adopted=false）→ setup 链断在
selection setter（selection-api addRange「root 非 node document 则
return」，rangeCount 0 抛 `only 0 ranges are added`）。真修=引擎级
adoptNode 支持，shim 侧不可达（FIXME(d33) 留档于 docEl.insertBefore）。
探针 D4 旁证：body 包装 getter 含 R159 head 段（`<title>iframe</title><br>`）；
D5 旁证：吸收径**保留** title（title 死亡全在语料 setup 重置，非 join）。

## 面账（d33 终态）

| 面 | 基线 | 实得 |
|---|---|---|
| `insertparagraph-in-child-of-html` | 24P/48F | **64P/8F** ✓ |
| `delete-in-child-of-html` | 14P/32F | **24P/22F** ✓ |
| `delete-in-child-of-head`（守护） | 64P/4F | 64P/4F 恒 ✓ |
| `insertparagraph-in-child-of-head`（守护） | 64P/8F | 64P/8F 恒 ✓ |
| `zz-probe-b1s3`（哨兵） | 4P/9F | 4P/9F 恒 ✓ |
| `zz-probe-d31-unlink`（哨兵） | 8P/0F | 8P/0F 恒 ✓ |

## 验证

- engine lib（CARGO_TARGET_DIR=/tmp/d19tgt，test-guard）：**4032P / 0F**。
- shim 装配语法检查（/tmp/check_shim.sh 22 parts 串联）：SHIM-OK。
- whitespaces/plaintext-only 面（同通道 key 消费面）零消费本批路径
  （改动仅在 doc 级 walk/body 吸收/head 分支/doc 级拆半落位；CE 主窗路径
  字节等价），按实际影响未跑。

## 遗留 FIXME

- FIXME(d33)：docEl.insertBefore 跨文档收养未实现（见上节）——
  insertparagraph 余 8F。
- title 挂账切片不变：head 段提升 + detHtml head 段 + R164 查询 adapter
  identity（R220 矿区，全语料对拍护航）——本批 18F T 门控 + P2F=1 同门。

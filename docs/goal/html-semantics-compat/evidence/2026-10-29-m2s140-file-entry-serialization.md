# M2 s140 · File 条目层 + multipart filename 不归一定谳——form-submission-0 三文件 +60 F→P（全量 6938P/946F/97T → 6998P/886F/97T）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s139（text/plain + multipart 编码器）

## 入场定刀位

控制面 s140 候选①：File 条目层——file input entry 贡献 +
input.files 面 + FormData append File 通道 + multipart File
part。s139 后 form-submission-0 三文件各 36P/26F，残余 =
File/filename 层 + w1252/surrogate（候选④）。

## spec 核实（不靠记忆）

WebFetch 对 html.spec.whatwg.org 不可达，换 web-reader 抓取
form-control-infrastructure.html，三点定谳：

1. **Construct entry list（file 状态步）**：对每个 selected
   file 贡献 entry（name, File）；无选中 → 零条目（而非占位
   空 value entry）。
2. **normalize-newlines 步作用域**：只作用 entry name 与
   **非 File value**——File.name / filename 不触（spec 4.10.22.8
   multipart 编码算法步 1.1 name、步 1.2 明确「If entry's value
   is not a File object」才归一）。
3. **filename 转义**：仅 escape a string 逐字节（LF→%0A、
   CR→%0D、`"`→%22），原文「must not perform any other
   escapes」——**先归一再转义是错的**。

语料三向核对（multipart-formdata.window.js，与 spec 完全一
致）：name 归一后转义（`a\rb`→`a%0D%0Ab`、`a\n\rb`→
`a%0D%0A%0D%0Ab`，line 94-125）；value 归一不转义（`b\nc`→
`b\r\nc`，line 127-165）；filename **不归一**（`b\nc`→
`b%0Ac`、`b\n\rc`→`b%0A%0Dc`——lone LF 保持 lone，line
167-209）。

## 施工（shim 1 文件 2 处）

- **Edit A（part04 file input entry 分支）**：`_ty === 'file'`
  分支对 `_c.files` item() 枚举逐 File `_pairs.push(_zwFdEntry(
  _nm, _cfF))`；无选中零条目。files 面走 R3902b setter 存的
  FileList 引用（`_zwFdEntry` 为 File-aware 条目构造，File
  身份 + filename 第三元，与 FormData append 同型，part02）。
- **Edit B（⑤a 序列化层字节构建器，替换 s139 三分支）**：
  `_entryStr(e)` = File 值取 filename 字符串（urlencoded/
  text-plain/GET query 共用，spec convert to name-value pairs
  步）；text/plain `_normNL(_entryStr)` 逐对；multipart 分支
  整体按字节数组构建（`_mpBytes` + `_mpPushStr` 经
  `_zw_utf8_encode` 逐字节 push）：part 头 name 经 `_mpEsc`
  （_normNL + 逐字节转义）；File 分支 Content-Disposition 带
  `; filename="…"` + `Content-Type: <file.type ||
  'application/octet-stream'>` + `_zw_blobBytes` 原始字节；
  filename **不归一**仅逐字符转义（Edit B 初版误用 `_mpEsc`
  归一，定向跑 6 案 F 后对照语料定谳修正——见过程教训）；
  `_body = _zwEncodeBytesPrefix(new Uint8Array(_mpBytes))` 经
  `__zw_bytes:` csv-decimal wire（R3020）传 host，
  fetch_bridge decode_body_bytes_raw → body_bytes，echo 端点
  按原始字节转义回显（s138 通道复用）。

## 验证

- shim 13-part 拼接 node --check 过（跨 part 符号定义位置核
  实：`_zwEncodeBytesPrefix` part01:4026、`_zw_utf8_encode`
  part02:891、`_zw_blobBytes` part02:4170、`Blob` part02:4271）。
- 定向 form-submission-0：Edit B 后 **202P/102F/1T**（对 s139
  基线 148P/156F/1T，54 F→P——(key,state) 对级 comm：losses 全
  Fail/gains 全 Pass/键集合 diff IDENTICAL；分布 text-plain
  +20/urlencoded2 +20/multipart +14）；filename 裸转义 fix 后
  **208P/96F/1T**（+6 恰为 multipart filename 换行 6 案，对级
  闭合同上；multipart 残余 6F = w1252/surrogate 族候选④）。
- 全量 join（对 s139 full，(key,state) 对级 sort -u 后 comm）：
  **恰 60 F→P 全部为 form-submission-0 三文件案（multipart/
  text-plain/urlencoded2 各 20），键零消失零新增零其他位移**；
  行数守恒精确闭合（6938P/946F/97T → 6998P/886F/97T，
  ΔP=+60/ΔF=−60/ΔT=0，7981 行不变）。
- engine lib **3581-3582 passed / 8-9 failed**——stash 对照实
  验定谳：HEAD 基线（无 s140 改动）同样 8F，失败面为
  js_dom_bridge fetch/service worker/request 桥接系列稳定 8 +
  details flake 间歇（在册），**非本切片引入**（s139 记录
  3590/0 与当前 HEAD 不符，环境性回归面，归口挂账勘根，见新
  挂账）；runner bin **256P/1F**（唯一 F = trend_csv_check
  encoding 行归口不变）；clippy（zero-engine + zero-wpt-runner
  --all-targets -D warnings）零告警；fmt runner 过；shim 拼接
  node --check 过；语料零改动；无探针页。

## 挂账（本切片更新）

- **form-submission-0 残余**（三文件 88F/1T）：①targetted-form
  族 58 新见 F 深层面勘根（候选②，double-submit 防护族——
  form-double-submit-requestsubmit 35F / form-submit-action
  22F 为主）；②accept-charset w1252 + `&#NNNN;` 实体回退 +
  lone surrogate USV（候选③，w1252/surrogate 6 案/文件——含
  multipart `character not in encoding in filename` 残余）。
- **新挂账**：engine lib js_dom_bridge fetch/service worker/
  request 桥接系列环境性失败面（2026-10-29 发现，HEAD 基线
  stash 对照 8F 稳定复现：fetch_sync_wire_preserves_response_
  contract / test_fetch_appends_cross_origin_request_origin /
  test_fetch_forbidden_headers_r3221_r3222 /
  test_fetch_passes_request_credentials_to_host /
  test_iframe_service_worker_messages_dispatch_to_iframe_
  container / test_request_headers_guard_r3223 /
  test_request_body_readers_r2982 /
  test_response_request_constructors_r2968 + details flake 间
  歇第 9）——与 s139 记录 3590/0 不符，失败形态 JS 求值
  `undefined` vs 期望 wire 串（非端口类错误），待勘根（环境/
  机器状态面）。
- 承袭不变：「Named elements should override builtins」1F 挂
  待决；encoding-legacy-mb CSV 行错位归口 encoding 流；fmt
  url_idna.rs 违例归口并行流 14dbfb011；echo 回显体 HTML 解析
  面观察；RadioNodeList live 语义近似、past names 条目不显式
  清理、expando store/target 双轨、part03 raw 链 type 读通道
  未勘 + reset 臂无还原臂、reset.html form element pointer 面
  （待决缓行）、execution-timing 构成面、lazy-loading 26F、
  meta color-scheme 21F、写流盖章断链、part02 setAttribute 绕
  notify、041 nested write、deep clone 非 HTML ns、css-module
  corpus 门（待决）、www2 可达性、module-import-referrer TLA、
  dynamic import() with 透传、classic 外链 SRI、143 write、
  defer 多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、
  FileList 缺索引访问、live 树逗号列表 qSA 根缺口、渲染流面
  让位累积、s131 插入步缺口细化、radio 现象⑤/checked 属性增
  设/defaultChecked 无独立 corpus 面、s135 已知限制 3 条；
  **待决（须用户拍板，既有）**：css/text 数据模块语料资源缺
  失、green-2x2.png 语料资源分发、parser form element pointer
  4F 缓行（2026-10-23）。

## 过程教训

- **escape 的作用域跟值走的通道走，同一行内两个字段规则可以
  不同**：multipart part 头里 name 经 normalize-newlines 后转
  义、filename 只 escape a string 不归一——spec 步 1.2 的
  「not a File object」限定条件就是分界线。初版把 filename 复
  用 `_mpEsc`（含 _normNL）过度归一（lone LF→CRLF 转义出
  `%0D%0A`），语料期望 `%0A`——s139 账本「name/filename 经
  escape a string」一句粗记埋的雷，本轮对照语料断言 + spec
  原文才拆掉。粗记结论引用进新代码前要回原文核对限定条件。
- **(key,state) 对级对账五件套连用零假阳性**：s139 16 假位移
  教训固化后（对级 sort -u + comm + 位移方向核对 + 键集合
  diff + 行数守恒），s140 三轮 join（定向×2 + 全量×1）全部干
  净闭合。
- **质量门禁基线漂移要用对照实验定谳**：engine lib 8-9F 与
  上轮记录 3590/0 不符，第一反应不是排查本切片——stash 改动
  后 HEAD 基线复跑同样失败，一次对照实验把「s140 回归」证
  伪，转归口挂账勘根，避免在无关面上浪费施工。

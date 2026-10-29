# M2-S3c — fetch fixture .txt 解锁 + 浏览器默认请求头 + null body status（2026-10-29）

**切片**: fetch/api 面 fixture 资源解锁（`top.txt` / `cors-top.txt` / `empty.txt`
+ `status.py` 路径）+ shim 浏览器默认请求头（User-Agent / `Accept: */*`）+
null body status（204/205/304）+ preflight UA 对齐。
**验证口径注记**：本切片与 M2-S4a（url 组件 setter，同 working tree 遗留
未提交工作）联合全通道 A/B 验证——归因按文件域互斥分账（`url/*` 归 S4a、
`fetch/*` 归本切片），详见两切片证据档与 goal 控制面。

## 根因与实现

### 1. 静态资源 `.txt` 缺失（top.txt 9 文件 73F 引用面）

`fetch/api/resources/` 此前只落 helper `.js` + `.headers` sidecar（拉取
过滤器无 `.txt`）：`top.txt` 缺失致引用它的用例族「Failed to fetch」/404
（mode-same-origin / mode-no-cors / integrity / cors-filtering /
cors-expose-star / cookies 族 / response-consume fetch 面）。empty.txt
（abort/general 载体）同族。

实现：`lib.sh` 深挖模式加 `*.txt` opt-in 臂（`GOAL_PULL_TXT=1`，防其他
goal 语料面意外扩张）+ goal 脚本显式 `fetch_raw` 三件（`resources/` 含
`.html` 即被 `fetch_dir_html` 短路、GitHub API 限流面下显式列文件最稳，
M5-S8 先例）。

### 2. `status.py` 路由只认 `xhr/` 前缀（text-utf8 10F）

上游 `fetch/api/resources/status.py` 与 `xhr/resources/status.py` **逐字
同源**（code/text/content/type + X-Request-Method）。runner fixture 此前
仅 `xhr/` 前缀命中，fetch/api 面落静态读库 404（text-utf8 fetched-data
族）。实现：`testharness.rs` 条件扩 `fetch/api/resources/status.py` 精确
命中。

### 3. 浏览器默认请求头缺失（request-headers / accept-header）

浏览器所有 HTTP 请求带 User-Agent（网络层）与 `Accept: */*`（fetch）。
`_issueFetch` 缺省补齐（不覆盖显式 init）：`user-agent` 在
`_zwPreNamesOf` 恒跳过、`*/*` 在 accept 值条件安全名单内——**预检名单
零变化**。accept-language 不补（按引擎 2024 名单逻辑恒入预检名单，补齐
会扰动 cors 族既有面）。

### 4. null body status（response-null-body 9F）

fetch spec #null-body-status：204/205/304 响应体为 null——wire 响应此前
给空串，`response.body` 为空流而非 null。`_makeResponseFromWire` 对该三
状态置 `bodyArg = null`（`.text()` 空串语义不变）。101 仅信息性响应面、
fetch 通道不可达，不入表。
https://fetch.spec.whatwg.org/#null-body-status

### 5. preflight UA 对齐（r1 过程轮回归清偿）

主请求补 UA 后，`preflight.py?checkUserAgentHeaderInPreflight` 腿暴露
fixture 严格性：上游 preflight.py 在 OPTIONS 时记录 preflight UA、实际
请求比对一致性（不一致 → 400）。此前 shim 两侧皆不发 UA，`''===''`
恒过——**假绿**；补 UA 后实际请求带、preflight 未带 → 400。实现：
preflight `preHeaders` 补同一 UA 值（浏览器行为——所有请求含 preflight
均带 UA，M4-S23 preflight `Accept: */*` 先例同型；UA 值会话内恒定，
preflight cache 命中跳 OPTIONS 时 stash 比对仍成立）。

## 验证

- 归因表（r1，`netapi-m2s3c-r1.json` vs `netapi-m2s3b-r3.json` 逐
  subtest，fetch 域 +70/−1）：

| 增益面 | 文件 × legs | ΔP |
|---|---|---:|
| top.txt/cors-top.txt 解锁 | mode-no-cors 2、mode-same-origin 4、cookies 3、cors-cookies 2、cors-cookies-redirect 2、cors-basic 5、cors-filtering 11、cors-expose-star 3、response-consume「from fetch to *」3、integrity 11 | +46 |
| status.py 路径 | text-utf8 10/10 收口 | +10 |
| UA/Accept 默认头 | accept-header 1、request-headers 1 | +2 |
| null body status | response-null-body 9/11（余 2 见记账 4） | +9 |
| preflight UA 回归 | cors-preflight「check preflight has user agent」 | −1（实现 5 清偿） |

- **联合终态验证**（本切片全套 + M2-S4a url 组件 setter + ws fragment
  判定修复，同一 working tree 全通道一轮）：`netapi-m2s3c-r4.json` vs
  `netapi-m2s3b-r3.json`，逐 subtest：**+258 增益 / 0 真回归 / MISSING 0**
  （10585 → 10843 pass；1 Pass→Fail 为记账 1 的 SRI 语义显影；过程轮 2
  真回归——preflight UA 一致性、ws fragment presence——均于终态前清偿，
  终态双腿 Pass）。
- 构建无 warning（`build-m2s3c-r3.log`，r4 运行所用二进制）。
- 运行干净：r1 stderr **0 panic**（M2-S3b 基线每轮 2 次 url crate panic
  ——S4a no-op setter 面不再触发 hostile username/password 路径，见 S4a
  档记账 4）。

## 记账（未清偿）

1. **integrity「CORS invalid integrity」语义显影（1 leg，Pass→Fail 非
   行为回归）**：基线该腿靠「Failed to fetch」拒绝**假绿**（机械证据：
   基线 integrity 16 腿 404「expected 200 but got 404」、CORS 族
   TypeError: Failed to fetch——reject 期望腿被网络失败误满足）；
   top.txt 解锁后 fetch 成功、引擎无 SRI 校验 → 该腿显影为**真失败**。
   SRI（integrity 全族残余）挂后续切片。
2. **request-headers 剩余 23F**：20F = runner fixture 仅放行
   GET/OPTIONS（POST/PUT/custom 方法 → Err → status 0，runner 域）；
   2F = 头回显大小写保真（fixture 域）；1F = `Set-Cookie` 禁止请求头
   过滤（引擎小项）。
3. **header-values-normalize 36F**（承前）：wire 格式 `\x1e` 分隔符腐蚀
   含控制字符的头值——wire 转义切片挂账。
4. **response-null-body 余 2F** + **response-consume URLSearchParams-to-
   blob 1F**（M2-S1 记账证据升级落该档：wpt.fyi 双向实证张力）。
5. **cors-redirect-preflight 18F / cors-preflight-star 10F /
   cors-preflight-redirect 9F**（承前）：per-hop preflight 机制 runner
   域挂账。

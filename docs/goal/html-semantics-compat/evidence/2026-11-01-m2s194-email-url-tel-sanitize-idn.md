# M2-S194 email/url/tel 消毒链路 + 域部 IDN——12F 翻绿 + R3901 缓存污染修复，shim 3 面 + rust 宿主回调 1

## 入场定刀位

s193 收口后次席候选：email-value-idn 4F + email 3F + telephone 3F +
url 2F（the-input-element 簇 12F，勘根定为消毒管线单根面）。

## 勘根

缺口两根 + 一处基线同键发现：

1. **8F value getter 消毒链路缺位**：part03 value getter per-type 分派
   无 tel/url/email 臂——解析期 value 属性含 LF/CR（telephone 3F）、
   首尾空白（url 2F）、multiple 逗号串（email 3F）原样透出。setter 侧
   `case 'url': case 'email'` 共享臂已有基面（剥 \r\n + trim）但无
   multiple 感知；tel 在 text 族臂仅剥换行（符合 spec，不动）。
2. **4F IDN 域转换缺位**（email-value-idn）：parser/parser-multiple/
   dynamic setAttribute/type-change 四钉——email 值域部（最后一个 '@'
   之后）须 IDN→punycode（'exämle.com'→'xn--exmle-hra.com'、
   'お.com'→'xn--t8j.com'）。`url_idna.rs` 仅有整体 URL 面，无独立域
   转换；宿主回调缺位。
3. **validity 面零改**：typeMismatch（part03 R57）读 `_controlValue`
   原始存储，email 近似正则 `/^[^\s@]+@[^\s@]+$/` 对非 ASCII 原样匹
   配——转换前即 valid，四钉的 `validity.valid` 断言不依赖校验管线
   同源。
4. **基线同键发现（非本刀引入）**：R3901 `_zwApplyInputTypeChange`
   无条件 `_inputValuesSet[key] = true`——spec type change steps 不设
   dirty value flag（HTML §4.10.5.4），R3901「缓存即值」模型把变更后
   缓存标脏 → detached 新建 input 先 `type='email'` 后
   `setAttribute('value', …)` 被 R3902b 脏守卫拒绝，值永久丢失
   （email-value-idn dynamically-set 基线即 F，got ''）。

## 施工

- **rust 宿主回调 `__zw_domain_to_ascii`**（callbacks.rs，S183
  `__zw_parse_color_rgba` 同款注册式）：入参域部原串，合成 `http://`
  前缀经 url crate 整体解析复用同源 IDNA 取 host（非 ASCII 转
  punycode、大写折叠）；解析形状校验（无 port/path/query/fragment）
  ——'b:8080'/'b/c' 类非法域串视为转换失败返空串（spec 清洗不淘汰
  非法值，shim 判空保原串）。
- **part01 email case 自 url 分裂**：新 `_zwEmailTokenSanitize`
  （trim + last-'@' 域部转 ASCII，无 '@'/宿主未注册/转换失败原串返
  回）+ multiple 感知（`__zw_has_attr_lw` 读面——同批
  setAttribute('multiple') 后读侧重消毒，email.html "multiple set"
  钉态；逗号分割逐 token 清洗重组，读写两侧同过本函数）。
- **part03 value getter 消毒臂条件扩展**：s190 时间族臂条件 +
  `_rvTyG === 'tel' || 'url' || 'email'`——读侧单点挂接（脏缓存优先/
  属性 lw 回落 → `_zwSanitizeInputValue`）。text/search/password 不扩
  （语料无钉面，FIXME 记账）。
- **R3901 缓存污染修复**：part01 新 `_typeChangeTainted` 标记 map
  （type change 步骤写缓存时置标——非真脏）；part04 value IDL set
  （INPUT 分支 `_captureInputDefault` 后单点）即真脏清标；part04
  setAttribute/removeAttribute('value') 同步块以标记绕行 R3902b 脏守
  卫并落定清标。

## 验证

- **定向**：email.html **8/8 全 Pass**（3F 翻绿）、email-value-idn
  **4/4 全 Pass**（4F 翻绿）。
- **the-input-element 目录**：1621P/40F/2T，逐文件 diff 恰 4 文件翻
  绿（email +3、email-value-idn +4、telephone +3、url +2），
  type-change-state bit-exact。
- **forms 全树：4423P/277F/12T → 4435P/265F/12T**（ΔP=+12 精确；ΔF
  =−14 = 12 目标翻绿 + s193probe 2F 行随探针文件删除消失；两代日志
  同键重跑形态同构，逐文件聚合 diff 权威）；逐文件 diff 恰 5 处，
  其余全树 bit-exact。
- **邻接 bit-exact**：interactive-elements 439P/44F；engine lib
  **3642P/1F**（ua_display_li 渲染流在册同键）。
- **shim 全序 node --check OK**；cargo fmt --all --check 过；
  **clippy 过**（callbacks.rs 触碰，本刀必跑）。

## 残留与边界

- text/search/password 读侧消毒未挂（语料无钉面，FIXME：扩面前先勘
  setter 存量行为面）。
- email-set-value.html 1F 基线同键推进：value sanitize 断言已过，余
  `getSelection().toString()` 可见面（testdriver send_keys 键入后可
  见内容/选区未建模）——非本刀位移。
- 克隆传播（R3901 clone 块）不带 `_typeChangeTainted` 标记（克隆产物
  setAttribute('value') 语义语料无面，FIXME）。
- `removeAttribute('type')` 不跑变更步骤（R3901 既有 FIXME 承袭）；
  `.list` setter、willValidate mutTree wrapper 面等在册项不变。

## 下一步候选

focus 时序域（checkable-active-onblur-with-click 6F +
radio-focus-navigation 6F，负载敏感嫌疑 s171 先例）；fieldset
disabled-003 crash/hang 2F 独立勘根；customizable select 177F 挂起
待拍板（GB-20261031 征询在途）。

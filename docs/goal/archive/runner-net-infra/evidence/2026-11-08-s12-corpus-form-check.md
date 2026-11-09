# S12 缺口清偿之语料 pin 形态核对（cors-preflight-redirect）

日期：2026-11-08。挂账出处 = [切片二档](2026-11-08-m3-slice2-registry-head12.md)
§三（序列承 py 普查档 §三「S12 收编」项）；核对对象 = scripts/update-wpt-data.sh
头部缺口清单 cors-preflight-redirect 条目（M2-S12，2026-10-29 记）。

## 一、语料 pin 形态核对结果

- 语料文件：`wpt-data/fetch/api/cors/cors-preflight-redirect.any.js`
  （WPT_DATA_DIR = tests/wpt-runner/wpt-data，WPT_DATA_REF 仍 v1.10
  滚动工作区）。
- 形态 = **M2-S12 本地对齐版**：全文含「after preflight succeeded」×1、
  「after preflight failed」×0（上游 651ffcb03 于 2026-09-12 的改写形态），
  文件 mtime 2026-10-29（即 M2-S12 巡检当日）——语料面补丁在位。

## 二、销账/补条目判定

- update 脚本条目已准确记录升级再核规则（「升级新 ref 后需重新核对；
  新 ref 若快照 ≥ 2026-09-12 则无需补」）；销账条件 = 语料新 ref 快照
  跨过 2026-09-12，当前 WPT_DATA_REF = v1.10 未升级——**条件未到，
  条目维持**（非销账、非新增，无需改动脚本）。
- runner 侧承接面 = testharness_py_routes.rs 的
  fetch/api/resources/redirect.py 臂（OPTIONS 分支 `redirect_preflight`
  参数处理：带参 302 / 缺参 200 不重定向）——该端点仿真对语料新旧两
  形态均承载语义，**不随语料升级移除**；其「S12 补丁收编」= 该臂迁入
  注册表时在注册项臂注记 S12 联动（随注册表化批次落，切片档序列已挂）。

## 三、结论

「S12 补丁收编 + update 脚本缺口清偿」的脚本核对半项完成（本档）；
收编半项随 redirect.py 臂的注册表化批次落（届时引用本档销账口径）。

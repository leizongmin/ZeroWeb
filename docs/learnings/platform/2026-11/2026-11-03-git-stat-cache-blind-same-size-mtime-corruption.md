---
date: 2026-11-03
modules: engine
---

# git stat 缓存盲区：同 size 同 mtime 的磁盘内容损坏不报脏

## 问题

d1s4 落账轮 `make test` 编译 zero-engine (lib test) 失败：
`couldn't read crates/engine/src/js_dom_bridge_tests/part06.rs: stream did not
contain valid UTF-8`（byte 0xc0 at line 4206:1509，内容是 GLIBC_2.34 /
ld-linux-x86-64.so.2 等 ELF `.dynamic` 段二进制串）。但 `git status` 不报该
文件脏，`git ls-files -v` 显示正常 'H'，`file` 命令还报 "UTF-8 text"——
三个"干净"信号与编译错误直接矛盾。

## 根因

两个盲区叠加：

1. **git stat 缓存只看元数据**：磁盘文件 size（220,778 字节）与 mtime
   （Nov 2 03:22，恰为最后 checkout 时刻）都被损坏过程原样保留，git 据此
   判 unchanged，不做内容 hash → status/index 全部失明。
2. **`file` 命令只采样文件头部**：二进制垃圾在偏移 181,712（~第 4206 行，
   尾部 18% 区域），`file` 看开头即下 "UTF-8 JS" 结论。

损坏形态 = 源文件区域被共享库内容覆写（同 size、保 mtime、换内容），疑似
WSL2 块别名：`.so` 内容页映射进源文件 extent（本轮 disk-guard 刚做过
target/ 大清理，时间窗吻合但因果未实证）。全库 4,320 个 tracked 文件
UTF-8 扫描确认损坏仅此一处。

## 解决

**判别**：元数据信号（git status / ls-files -v / file）与编译错误矛盾时，
直接做内容级对账——

```bash
md5sum <磁盘文件>
git show HEAD:<path> | md5sum   # 两值不同 = 磁盘损坏（HEAD blob 完好）
```

**恢复**：先 `rm` 再 `git checkout -- <path>`。git 认为文件干净时不能指望
checkout 重写；rm 保证从 index 强制落盘。恢复后 md5 对齐 HEAD blob，再
`cargo check -p zero-engine --tests` 验证编译路径。

**预防**：无人值守门禁失败但日志无诊断时，用
`cargo check -p <pkg> --tests` 直接复现拿真实错误；大清理（disk-guard、
rm -rf target）后首轮全量构建前，可对 src 树做一次
`python3 -c "...decode('utf-8')"` 扫描（4,320 文件秒级）排除同型损坏。

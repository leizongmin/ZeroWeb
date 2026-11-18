---
date: 2026-11-15
modules: zero-wpt-runner
---

# comm 对比 sweep 产物必须显式 LC_ALL=C，否则 collation 差产出成百伪 GONE/NEW

## 问题

s478 轮规整管线把 s477 stable 键集与 s478 leg1 键集做 `comm` 时，
未设 `LC_ALL=C`，输出 **GONE 574 / NEW 577**（呈整目录簇状：the-area-element、
the-img-element 等），形似大面位移事故。实值只有 **GONE 1 / NEW 4**（select-home-end
族相位）。同一管线早期版本还踩了第二坑：`grep -E "^(Pass|Fail)\t"` 永不命中——
POSIX/GNU grep 的 ERE 里 `\t` 是字面 `t` 不是制表符，导致方向自动检测恒走错支。

## 根因

sweep 产物键含 `::`、大小写混排与个别非 ASCII 标题，`comm` 要求两输入按**同一
collation** 排序。不同命令链路下 sort 的 locale 环境不一致（继承 shell 默认 vs 显式
设置），键集排序序不同，`comm` 的线性归并比较在首个序差后把剩余全部判为
不对称行——差集呈雪崩式伪膨胀，且簇状分布伪装成「整目录页面缺席」。

## 解决

- 所有 join/comm/grep -f 管线统一 `export LC_ALL=C`，排序、比较、抽取同环境。
- 方向检测用 awk 首字段判断（`awk -F'\t' '{print $1; exit}'` 是否
  `Pass|Fail|Timeout|Unstable|Unsupported`），不依赖 grep 的 `\t`。
- 交叉验证习惯：GONE/NEW 数量异常时先 `comm` 报警检查（"文件 1 没有被正确排序"）
  与 `cut -d/ -f3-4 | uniq -c` 簇状分布——两者是 collation 伪差的指纹。
- s478 起规整管线脚本（norm.sh/canon.sh/join.sh）落盘 /tmp/sNNN/ 随工件走，
  下轮直接复用避免重写重踩。

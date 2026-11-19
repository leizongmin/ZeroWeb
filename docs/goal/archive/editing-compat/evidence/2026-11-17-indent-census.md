# indent br 边界普查与刀面定谳（2026-11-17，R15）

## 普查范围与方法

- 语料：`tests/wpt-runner/wpt-data/editing/data/indent.js` 全集 **145 条**（editing/run 域，`make testharness-editing-run-one FILTER='indent.html'`）。
- 刀前门锚：r14 批 `r14-indent.log`＝**1308/21/0**（21 红全 `compare innerHTML` 型，逐键见该日志）。
- 逐条索引以语料文件原始序为准（0 基，`re.split(r'\n(?=\[")')` 切分实测）。

## br 边界触发面（4 键，全部现红＝刀面）

语料期望（Chrome-160 录形）与键位：

| idx | 初始 HTML | 语料期望（plain 变体） | 侧 |
|-----|-----------|------------------------|----|
| 28 | `foo[bar]<br>baz<p>extra` | `<blockquote>foo[bar]</blockquote>baz<p>extra</p>` | 尾单 br |
| 29 | `foo[bar]<br><br><br><br>baz<p>extra` | `<blockquote>foo[bar]</blockquote><br><br><br>baz<p>extra</p>` | 尾 4 br（contained 仅含首枚） |
| 30 | `foobar<br>[ba]z<p>extra` | `foobar<blockquote>[ba]z</blockquote><p>extra</p>` | 首侧直形 |
| 18 | `<span>foo<br>{}</span>bar<p>extra` | `<span>foo{}</span><blockquote>bar</blockquote><p>extra</p>` | 首侧行内壳尾链 |

根因（spec delta）：`blockExtend` 不对称——`isBlockStartPoint` 计前邻 br 而
`isBlockEndPoint` 不计后邻 br，尾随 br 由 `_zwJusWrap` 占位拉入臂随包裹进块；
Chrome 录形边界 br 摘出后删除，且仅当该 br 是该边界的唯一行界。

## run=1 唯一性行界判据

- idx32（`foobar<br><br><br><br>[ba]z`）现绿：四枚 br 全留块外、仅 baz 包裹。
  刀的 run===1 判据保证不误伤（run=4 → 不消费）。钉测 `run4-guard` 恒一。
- idx15/17（caret 前 br：`<p>{}<br>foo</p>`／`<span>{}<br>foo</span>bar`）现绿：
  br 留块内（`<blockquote>{}<br></blockquote>`），sublist=[p]/[span]——br 是
  成员内末子非成员本身，两臂均不触发。钉测 `caret-p-guard`/`caret-span-guard` 恒一。
- 现绿 br＋blockquote 同现其余位（#45 br 居成员中段、#48/#50/#52 br 由合并臂
  插入）均非触发位。

## 刀体机制（part01g `_zwFbqIndentNodes` 第三参 `zwConsumeBoundaryBr`）

1. **尾侧 pop**：sublist 尾成员是 br 且前邻成员非 br 且可编辑 → pop＋removeChild。
2. **首侧 edge-br 消费**：自 firstNode.previousSibling 走连续 br run（行内壳经
   `lastChild` 逐层下探），run===1 才删除边界 br；可编辑门双查。
3. **拉入臂压制**：pop 发生时向 `_zwJusWrap` 传 `zwNoBrPull=true`——否则占位
   拉入臂摘一枚又拉一枚（idx29 四 br 形钉测实证：bq 内 br 恒在、外侧少一枚）。
4. 作用域：仅 indent 调度点传 `true`（part01g:834）；outdent 复用两调用点
   （part01g:982/984）不传旗标恒 inert；list 臂早退不受影响；
   `_zwJusWrap` 其余 12 调用点新参缺省 falsy＝零漂移。

## 钉测（part136_indent_br_pins.rs，7 形）

end-single/end-multi/start-bare/start-shell/run4-guard/caret-p-guard/caret-span-guard，
7 形 ret 均 true 且 html 逐键断言（含尾侧两形不得残留包裹尾 br 反断言）。
钉测输入形与语料 idx28/29/30/18/32/15/17 一一对应（选区用 Range 置位故无
`[]`/`{}` 标记）。

## 刀后实证（smoke，本轮）

`make testharness-editing-run-one FILTER='indent.html'`（`r15-smoke-indent.log`）：

- **1312/17/0**，Fail 集差量恰 −4 刀面键（idx18/28/29/30），零新增红
  （`diff` 对 r14-indent.log failset 实证，rc=1 仅删 4 行）。

## mt 解耦证明

`multitest.js` 629 条含 `"indent"` 48 处，初始 HTML 含 `<br>` 的 indent 条目
**0 条**（静态切分实测）＝mt 腿预期 SET-IDENTICAL 10090/2。

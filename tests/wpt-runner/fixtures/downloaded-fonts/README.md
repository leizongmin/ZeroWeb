# 下载字体生产链路最小用例

`pages/deep/intrinsic.html` 使用非 Ahem 名称的 `DocumentFace`，以 40px 绘制绿色
`XXXX`，不指定宽高；应与 `reference.html` 的 160×40 绿块一致。
`pages/deep/index.html` 固定 160×40 与 line-height，隔离资源传递；它不能替代
intrinsic 用例。两者通过不同目录的外部 CSS 引用仓库既有 `fonts/Ahem.ttf`。

在项目资源包裹器内启动仅监听回环地址的静态服务，根目录指定为
`tests/wpt-runner`，端口使用本次任务的空闲端口。例如：

```sh
./target/test-guard --per-proc-mem 1 --total-mem 2 --time-limit 600 -- \
  python3 -m http.server "$FONT_CASE_PORT" --bind 127.0.0.1 --directory tests/wpt-runner
```

访问 `/fixtures/downloaded-fonts/pages/deep/intrinsic.html`。正确字体请求路径为
`/fonts/Ahem.ttf`；以文档而非 CSS 作基址会落到错误目录。可在独立诊断服务中给
字体响应添加 500ms 延迟，检验采集器未把 fallback 首帧误当完成帧。

复用 `zeroweb-browser-chrome-parity` 的 generic-page 场景模板：观察 `#sample`，
状态表达式为 `({text:document.querySelector('#sample').textContent})`，初始 snapshot；
1022×676、DPR 1、en-US、light。比较状态、几何、区域及未遮罩全图，不改阈值。

此目录是可复用的生产采集 fixture，不会因放在此处就自动进入上游 WPT 集。
常驻 CI 断言分别在 webview URL 测试、layout downloaded_font_metrics、renderer
font_payloads、paint-convert fonts、compositor font_resource_tests 和 browser 页面
字体/导航测试中；完整实验见 [首例报告](../../../../docs/acceptance/site-optimizer-font-pipeline.md)。

// Renderer 信任边界 ②C'（2026-11-01）：`__zw*` 特权 mutator counter-plugin。
// 非测试模式由宿主在 js_dom_shim 装载后、页面脚本执行前执行本片段，删除页面可见的
// 特权 mutator 全局（激活注入 / 剪贴板直写 / cookie jar 清空 / 权限注册表直写——标准
// API 门控旁路面，逐枚裁定见 renderer-trust-boundary goal 决策包 ②）。shim 内定义保留
// （幂等卫 + 测试模式 runner 直调），内部消费已闭包化（part02 `_zwPermSetState` /
// part06 `_zwClipStoreWrite`），删除后 `permissions.request()` headless 自动授予与
// `execCommand('copy')` 剪贴板桥不回归。内部 reader 钩子（`__zwPermissionState` /
// `__zwNotifyPermissionChange`）保留——permissions.query 与 fullscreen 激活门实现自身
// 消费，非 mutator。`__zwModalGateBypass` 为接受残余：shim 自管旗标，删除会被 shim
// 复写，且仅自影响本页焦点语义。
// 测试模式（`ZW_RENDERER_AUTOMATION=1`，runner/自动化派生链注入）宿主跳过本片段，
// runner testdriver stub 依赖的钩子原样保留。
(function () {
  var _zwMutators = [
    '__zwUserActivate',
    '__zwClipboardStoreWrite',
    '__zwDeleteAllCookies',
    '__zwSetPermission',
  ];
  for (var _zwMi = 0; _zwMi < _zwMutators.length; _zwMi++) {
    try {
      delete globalThis[_zwMutators[_zwMi]];
    } catch (_zwMe) {}
  }
})();

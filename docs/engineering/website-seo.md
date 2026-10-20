# 官网 SEO 与发布

官网使用 `scripts/generate-website.py` 生成静态部署目录，沿用 GitHub Pages。
英文首页为 `/`，中文首页为 `/zh/`。三个主页面、更新详情与工程指南均有独立语言网址。
语言链接直接跳转到同一页面的另一译本，不依据浏览器语言改写内容。

## 内容与本地预览

- 首页模板：`website/index.html`；翻译：`website/script.js`。
- 测试页：`website/progress.html`、`website/progress.js`；指标仍由已有 CSV 和指标生成器决定。
- 更新：`website/updates.json`，详情网址为 `/updates/<日期>-<类型>/`。生成器按日期倒序排列。
- 指南：`website/guides.json`，网址为 `/guides/<slug>/`。
- 分享卡片：`website/social-en.png` 与 `website/social-zh.png`，尺寸为 1200 × 630。

在仓库根目录执行：

```sh
python3 scripts/generate-website.py --output /tmp/zeroweb-site-preview
python3 scripts/test-website-seo.py /tmp/zeroweb-site-preview
python3 -m http.server 8769 --directory /tmp/zeroweb-site-preview
```

访问 `http://localhost:8769/`。应预览生成目录，模板目录不包含完整的静态正文。
生成器需要 Python 3.9+ 和 Node.js 24；无需安装 npm 包。
输出目录必须与模板隔离，已有非生成目录会被拒绝覆盖。

发布工作流先生成页面，再验证 SEO 和本地链接，最后上传产物。
修改生成脚本、模板、指南、更新或所列指标 CSV 会触发 main 分支的 Pages 发布。
新增脚本时同步更新 `.github/workflows/pages.yml` 的路径过滤。

## 搜索平台验证与提交

验证标签已支持通过 GitHub Actions 仓库变量注入。账号验证码尚未配置时不输出空标签。
验证码是平台给站点所有者的标识，不能自行编造；不要把账号登录凭据放入变量。

1. 打开 [Google Search Console](https://search.google.com/search-console)，选择已有站点或添加
   `https://zeroweb.leizm.com/` 的网址前缀资源。若选择域名资源，需由 DNS 所有者完成 TXT 验证。
2. 使用 HTML 标签验证时，把标签的 `content` 值设置为 GitHub 仓库 Actions 变量
   `GOOGLE_SITE_VERIFICATION`。
3. 打开 [Bing Webmaster Tools](https://www.bing.com/webmasters/)，添加站点；
   把 `msvalidate.01` 标签的 `content` 值设置为仓库变量 `BING_SITE_VERIFICATION`。
4. 发布当前变更，或在变量变更后手动运行 `Deploy GitHub Pages`。
5. 在两个平台确认验证通过，提交 `https://zeroweb.leizm.com/sitemap.xml`。
6. 在 Google 的网址检查中分别检查 `/`、`/zh/`、数据页及一篇更新详情；
   核对已抓取正文、用户声明的 canonical 和 Google 选择的 canonical。

生成器自动输出 `robots.txt`，允许抓取并声明 sitemap。站点地图只列首选网址，
不填写推测的 `lastmod`。英文、中文各自声明 canonical，译本通过 `hreflang` 互链。
分享元信息和 JSON-LD 写在原始 HTML 中；更新详情的日期取自更新数据。
结构化数据帮助描述页面，并不保证富媒体搜索结果或排名提升。

## 观察效果

先记录当前收录页面、品牌查询词和非品牌查询词。发布后查看：

- 英文与中文页面是否分别被发现、抓取和收录。
- 品牌词 `ZeroWeb`、`ZeroWebView`、`ZeroBrowser` 的曝光与点击。
- Rust 浏览器、嵌入式 WebView、AI 自主软件工程相关查询的曝光、点击与平均位置。
- 哪些指南或更新获得非品牌搜索流量，是否有标题曝光高但点击低的页面。
- 网页体验中的移动端问题及 Core Web Vitals；根据真实报告决定性能优化范围。

比较相同时间窗口，结合发布记录、查询和页面维度判断；不要把短期波动直接归因为某个标签。
测试通过只证明部署结构满足本地判据，不能替代平台验证、搜索收录或实际访问数据。

## 检查入口

`scripts/test-website-seo.py` 检查所有页面的 canonical、语言链接、标题摘要、
分享图片、静态正文和本地资源。浏览器验收还应检查语言切换、性能指标切换、
375px 小屏及禁用 JavaScript 后的正文。脚本增强失败时保留静态测试数据。

参考：[Google 多语言指南](https://developers.google.com/search/docs/specialty/international/managing-multi-regional-sites)、
[JavaScript SEO](https://developers.google.com/search/docs/crawling-indexing/javascript/javascript-seo-basics)、
[站点地图](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap)。

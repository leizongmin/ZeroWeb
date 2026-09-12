const copy = {
  en: {
    skip: 'Skip to content', experiment: 'Experiment', architecture: 'Architecture', progress: 'Data', updates: 'Updates', github: 'GitHub', kicker: 'CI test results', title: 'How is ZeroWeb doing?', intro: 'Scheduled CI runs the same compatibility and performance checks over time. This page reads the results committed to the repository.', loading: 'Loading the latest results…', fresh: 'Results updated {date}', wptKicker: 'Web compatibility', wptTitle: 'WPT reftest pass rate', wptDescription: 'Each point is the latest upstream WPT run for that month, plotted as the pass rate. The corpus snapshot version is included because the test set size changes between versions.', suitesKicker: 'Coverage by WPT suite', suitesTitle: 'All tracked suites', suitesDescription: 'One row per WPT suite. Denominators differ across rows (reftest files vs. testharness subtests) and the unit is shown next to every number. Rows marked “Planned” have no data yet and will fill in as those goals run.', perfKicker: 'Weekly performance run', perfTitle: 'Performance at p95', perfDescription: 'All results come from the same class of GitHub runner, so changes in hardware do not distort the trend. Lower is better.', source: 'See the raw data ↗', chooseMetric: 'Choose a metric', methodWpt: 'Methodology: the CSS rendering suite is a self-source reftest — test and reference pages are both rendered by ZeroWeb and compared pixel by pixel, so it measures self-consistency, not a score judged by another browser. It must not be read as comparable to the full-suite scores Chrome, Firefox and Safari publish on wpt.fyi. The other suites are testharness runs scored by assertion.', methodCorpus: 'Corpus version: suites run against a fixed imported snapshot of upstream WPT (CSS rendering currently v1.10). The snapshot is updated over time, so denominators differ between rows and between months; every count is shown as passed / total.', methodAnchors: 'Reference points, for scale only (different methodology — not directly comparable): mainline browsers pass ~96–98% of the full WPT suite (<a href="https://wpt.fyi/results/">wpt.fyi</a>); the Ladybird independent engine reports ~93% (<a href="https://ladybird.org/">ladybird.org</a>).', footer: 'An AI-autonomous software engineering experiment built around a Rust browser.', error: 'The test data could not be loaded.', empty: 'No results yet.', date: 'Date', value: 'Result', wptPassed: 'WPT tests passed', passRate: 'Pass rate', total: 'Total', corpusVersion: 'Corpus', colSuite: 'Suite', colScope: 'WPT scope', colSize: 'Passed / total', colRate: 'Pass rate', colStatus: 'Status', colDate: 'Date', colEvidence: 'Evidence', statusActive: 'Active', statusCompleted: 'Completed', statusPlanned: 'Planned', unitTests: 'tests', unitSubtests: 'subtests', unitFiles: 'files', mediumPage: 'Medium page', startup: 'Browser startup', peakMemory: 'Peak memory', tests: 'tests passed', latest: 'latest p95 result'
  },
  zh: {
    skip: '跳到正文', experiment: '实验', architecture: '架构', progress: '数据', updates: '最新进展', github: 'GitHub', kicker: 'CI 测试结果', title: 'ZeroWeb 现在跑得怎么样？', intro: 'CI 会定期跑兼容性测试和性能基准。这里展示的是已经提交到仓库里的结果，可以看到它们随时间的变化。', loading: '正在读取最新结果…', fresh: '数据更新于 {date}', wptKicker: '网页兼容性', wptTitle: 'WPT reftest 通过率', wptDescription: '每个月取最后一次上游 WPT 结果，图中画的是通过率。测试语料快照会更新、分母随之变化，所以同时保留语料版本号。', suitesKicker: 'WPT 测试集覆盖', suitesTitle: '全部测试集', suitesDescription: '每行一个 WPT 测试集。各行的分母单位不同（reftest 文件数 vs 子测试数），每个数字旁都标了单位；标“规划中”的行还没有数据，随目标推进逐渐补齐。', perfKicker: '每周性能测试', perfTitle: '性能基准（p95）', perfDescription: '这些结果都来自同一类 GitHub Runner，避免把机器差异误算成性能变化。数值越低越好。', source: '看原始数据 ↗', chooseMetric: '选择指标', methodWpt: '口径说明：CSS 渲染一项为 self-source reftest——test 与参照页都由 ZeroWeb 渲染后逐像素比对，度量的是“自身一致性”，不是由其他浏览器判分的分数；不能与 Chrome、Firefox、Safari 在 wpt.fyi 上发布的全量分数直接比较。其余测试集为 testharness 运行，按断言计分。', methodCorpus: '语料版本：各测试集按固定的上游 WPT 导入快照运行（CSS 渲染当前为 v1.10）。快照会随时间更新，因此分母在行与行、月与月之间会变化；所有数字都以“通过 / 总数”给出。', methodAnchors: '量级参考（口径不同，仅供定位量级，不可直接对比）：主流浏览器在全量 WPT 上约 96–98%（<a href="https://wpt.fyi/results/">wpt.fyi</a>）；独立引擎 Ladybird 官方口径约 93%（<a href="https://ladybird.org/">ladybird.org</a>）。', footer: 'ZeroWeb 是一个由 AI 持续开发的 Rust 浏览器实验。', error: '测试数据没有加载成功。', empty: '还没有测试结果。', date: '日期', value: '结果', wptPassed: '通过的 WPT', passRate: '通过率', total: '总数', corpusVersion: '语料版本', colSuite: '测试集', colScope: 'WPT 范围', colSize: '通过 / 总数', colRate: '通过率', colStatus: '状态', colDate: '日期', colEvidence: '依据', statusActive: '进行中', statusCompleted: '已完成', statusPlanned: '规划中', unitTests: '案', unitSubtests: '子测试', unitFiles: '文件', mediumPage: '中型页面', startup: '浏览器启动', peakMemory: '峰值内存', tests: '项通过', latest: '最近一次测试的 p95'
  }
};

let language = 'en';
let metricsData = null;

function t(key) { return copy[language][key]; }
function formatNumber(value, maximumFractionDigits = 1) { return new Intl.NumberFormat(language === 'zh' ? 'zh-CN' : 'en-US', { maximumFractionDigits }).format(value); }
function formatDate(value, monthOnly = false) { const date = new Date(`${value}${monthOnly ? '-01' : ''}T00:00:00Z`); return new Intl.DateTimeFormat(language === 'zh' ? 'zh-CN' : 'en-US', monthOnly ? { year: 'numeric', month: 'short', timeZone: 'UTC' } : { year: 'numeric', month: 'short', day: 'numeric', timeZone: 'UTC' }).format(date); }
function metricById(id) { return metricsData?.performance.metrics.find((metric) => metric.id === id); }
function latestPoint(id) { const points = metricById(id)?.points || []; return points.at(-1); }
function unitLabel(unit) { return unit === 'MB' ? 'MB' : 'ms'; }
const REPO_BASE = 'https://github.com/leizongmin/ZeroWeb/blob/main/';
const SUITE_STATUS_KEYS = { active: 'statusActive', completed: 'statusCompleted', planned: 'statusPlanned' };
function esc(value) { return String(value).replace(/[&<>"]/g, (ch) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[ch]); }

function setLanguage(next, persist = true) {
  language = copy[next] ? next : 'en';
  document.documentElement.lang = language === 'zh' ? 'zh-CN' : 'en';
  document.title = language === 'zh' ? '测试数据 — ZeroWeb' : 'Test results — ZeroWeb';
  document.querySelectorAll('[data-i18n]').forEach((element) => { element.textContent = t(element.dataset.i18n); });
  document.querySelectorAll('[data-i18n-html]').forEach((element) => { element.innerHTML = t(element.dataset.i18nHtml); });
  document.querySelectorAll('[data-language]').forEach((button) => button.setAttribute('aria-pressed', String(button.dataset.language === language)));
  if (persist) localStorage.setItem('zeroweb-language', language);
  if (metricsData) render();
}

function summaryCard(label, value, detail) { return `<article class="metric-card"><span>${label}</span><strong>${value}</strong><small>${detail}</small></article>`; }
function renderSummary() {
  const latestWpt = metricsData.wpt.at(-1);
  const medium = latestPoint('page/medium/total_ms');
  const startup = latestPoint('startup_ms');
  const memory = latestPoint('resource/peak_rss_mb');
  document.getElementById('metric-summary').innerHTML = [
    summaryCard(t('wptPassed'), latestWpt ? `${formatNumber(latestWpt.passed, 0)} / ${formatNumber(latestWpt.total, 0)}` : '—', latestWpt ? `${latestWpt.rate}% · ${t('corpusVersion')} ${latestWpt.ref}` : t('empty')),
    summaryCard(t('mediumPage'), medium ? `${formatNumber(medium.value)} ms` : '—', t('latest')),
    summaryCard(t('startup'), startup ? `${formatNumber(startup.value)} ms` : '—', t('latest')),
    summaryCard(t('peakMemory'), memory ? `${formatNumber(memory.value)} MB` : '—', t('latest'))
  ].join('');
}

function chartMarkup(points, options) {
  if (!points.length) return `<p class="chart-empty">${t('empty')}</p>`;
  const width = 960, height = 330, left = 76, right = 26, top = 24, bottom = 48;
  const values = points.map((point) => point.value);
  const rawMin = Math.min(...values), rawMax = Math.max(...values);
  const padding = Math.max((rawMax - rawMin) * .18, rawMax * .035, 1);
  const min = Math.max(0, rawMin - padding), max = rawMax + padding;
  const x = (index) => left + (points.length === 1 ? (width - left - right) / 2 : index * (width - left - right) / (points.length - 1));
  const y = (value) => top + (max - value) * (height - top - bottom) / (max - min || 1);
  const path = points.map((point, index) => `${index ? 'L' : 'M'} ${x(index).toFixed(1)} ${y(point.value).toFixed(1)}`).join(' ');
  const area = `${path} L ${x(points.length - 1).toFixed(1)} ${height - bottom} L ${x(0).toFixed(1)} ${height - bottom} Z`;
  const ticks = Array.from({ length: 5 }, (_, index) => max - index * (max - min) / 4);
  const labels = points.map((point, index) => `<text class="chart-axis-label" x="${x(index)}" y="${height - 18}" text-anchor="middle">${options.dateLabel(point)}</text>`).join('');
  const grids = ticks.map((tick) => `<line class="chart-grid" x1="${left}" x2="${width - right}" y1="${y(tick)}" y2="${y(tick)}"/><text class="chart-axis-label" x="${left - 12}" y="${y(tick) + 4}" text-anchor="end">${options.tickLabel(tick)}</text>`).join('');
  const circles = points.map((point, index) => `<circle class="chart-point" cx="${x(index)}" cy="${y(point.value)}" r="5" tabindex="0" role="img" aria-label="${options.ariaLabel(point)}"><title>${options.tooltip(point)}</title></circle>`).join('');
  return `<svg class="trend-chart" viewBox="0 0 ${width} ${height}" role="img" aria-label="${options.chartLabel}"><defs><linearGradient id="chart-area-gradient" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="var(--blue)" stop-opacity=".18"/><stop offset="1" stop-color="var(--blue)" stop-opacity="0"/></linearGradient></defs>${grids}<path class="chart-area" d="${area}"/><path class="chart-line" d="${path}"/>${circles}${labels}</svg>`;
}

function tableMarkup(points, headers, cells) { return `<table><thead><tr>${headers.map((header) => `<th scope="col">${header}</th>`).join('')}</tr></thead><tbody>${points.map((point) => `<tr>${cells(point).map((cell) => `<td>${cell}</td>`).join('')}</tr>`).join('')}</tbody></table>`; }
function renderWpt() {
  const points = metricsData.wpt.map((item) => ({ ...item, value: item.rate }));
  document.getElementById('wpt-chart').innerHTML = chartMarkup(points, { dateLabel: (point) => point.period.slice(2), tickLabel: (value) => `${formatNumber(value, 0)}%`, tooltip: (point) => `${formatDate(point.period, true)} · ${formatNumber(point.passed, 0)} / ${formatNumber(point.total, 0)} · ${point.rate}% · ${t('corpusVersion')} ${point.ref}`, ariaLabel: (point) => `${formatDate(point.period, true)}, ${point.rate}%, ${formatNumber(point.passed, 0)} / ${formatNumber(point.total, 0)}`, chartLabel: t('wptTitle') });
  document.getElementById('wpt-table').innerHTML = tableMarkup(points, [t('date'), t('passRate'), t('wptPassed'), t('corpusVersion')], (point) => [formatDate(point.period, true), `${point.rate}%`, `${formatNumber(point.passed, 0)} / ${formatNumber(point.total, 0)}`, point.ref]);
}

function renderSuites() {
  const host = document.getElementById('suites-table');
  if (!host) return;
  const suites = metricsData.suites || [];
  if (!suites.length) { host.innerHTML = `<p class="chart-empty">${t('empty')}</p>`; return; }
  host.innerHTML = tableMarkup(suites, [t('colSuite'), t('colScope'), t('colSize'), t('colRate'), t('colStatus'), t('colDate'), t('colEvidence')], (suite) => {
    const unit = { tests: t('unitTests'), subtests: t('unitSubtests'), files: t('unitFiles') }[suite.unit] || suite.unit;
    const size = suite.total == null ? '—' : `${formatNumber(suite.passed, 0)} / ${formatNumber(suite.total, 0)} <span class="unit-tag">${esc(unit)}</span>`;
    const rate = suite.rate == null ? '—' : `<strong>${suite.rate}%</strong><span class="rate-bar" aria-hidden="true"><span style="width:${Math.min(suite.rate, 100)}%"></span></span>`;
    const note = suite.note ? `<small class="suite-note">${esc(suite.note[language])}</small>` : '';
    return [
      `<strong>${esc(suite.name[language])}</strong>${note}`,
      suite.dirs.length ? suite.dirs.map(esc).join(' / ') : '—',
      size,
      rate,
      `<span class="status-badge status-${suite.status}">${t(SUITE_STATUS_KEYS[suite.status] || suite.status)}</span>`,
      suite.date ? formatDate(suite.date) : '—',
      suite.evidence ? `<a href="${REPO_BASE}${esc(suite.evidence)}">GitHub ↗</a>` : '—'
    ];
  });
}

function renderPerformancePicker() {
  const select = document.getElementById('metric-select');
  const selected = select.value || 'page/medium/total_ms';
  select.innerHTML = metricsData.performance.metrics.map((metric) => `<option value="${metric.id}">${metric.label[language]}</option>`).join('');
  select.value = metricById(selected) ? selected : 'page/medium/total_ms';
}
function renderPerformance() {
  const metric = metricById(document.getElementById('metric-select').value);
  if (!metric) return;
  document.getElementById('performance-chart').innerHTML = chartMarkup(metric.points, { dateLabel: (point) => point.date.slice(5), tickLabel: (value) => formatNumber(value), tooltip: (point) => `${formatDate(point.date)} · ${formatNumber(point.value, 2)} ${unitLabel(metric.unit)}`, ariaLabel: (point) => `${formatDate(point.date)}, ${formatNumber(point.value, 2)} ${unitLabel(metric.unit)}`, chartLabel: `${t('perfTitle')}: ${metric.label[language]}` });
  document.getElementById('performance-table').innerHTML = tableMarkup(metric.points, [t('date'), t('value')], (point) => [formatDate(point.date), `${formatNumber(point.value, 2)} ${unitLabel(metric.unit)}`]);
}
function render() {
  document.getElementById('data-freshness').textContent = t('fresh').replace('{date}', formatDate(metricsData.latest_data_date));
  renderSummary(); renderWpt(); renderSuites(); renderPerformancePicker(); renderPerformance();
}

const savedLanguage = localStorage.getItem('zeroweb-language');
const detectedLanguage = navigator.language.toLowerCase().startsWith('zh') ? 'zh' : 'en';
setLanguage(savedLanguage || detectedLanguage, false);
document.querySelectorAll('[data-language]').forEach((button) => button.addEventListener('click', () => setLanguage(button.dataset.language)));
document.getElementById('metric-select').addEventListener('change', renderPerformance);
fetch('metrics.json').then((response) => { if (!response.ok) throw new Error('unavailable'); return response.json(); }).then((data) => { metricsData = data; render(); }).catch(() => { document.getElementById('data-freshness').textContent = t('error'); });

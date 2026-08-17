const copy = {
  en: {
    skip: 'Skip to content', experiment: 'Experiment', architecture: 'Architecture', progress: 'Data', updates: 'Updates', github: 'GitHub', kicker: 'CI test results', title: 'How is ZeroWeb doing?', intro: 'Scheduled CI runs the same compatibility and performance checks over time. This page reads the results committed to the repository.', loading: 'Loading the latest results…', fresh: 'Results updated {date}', wptKicker: 'Web compatibility', wptTitle: 'WPT tests passed', wptDescription: 'Each point is the latest upstream WPT run for that month. The suite version is included because the test set can change.', perfKicker: 'Weekly performance run', perfTitle: 'Performance at p95', perfDescription: 'All results come from the same class of GitHub runner, so changes in hardware do not distort the trend. Lower is better.', source: 'See the raw data ↗', chooseMetric: 'Choose a metric', method: 'Data notes: WPT uses the last upstream run of each month. Performance uses the last p95 result recorded on each date for github-ubuntu-latest. Results from other hardware are not mixed into the chart.', footer: 'An AI-autonomous software engineering experiment built around a Rust browser.', error: 'The test data could not be loaded.', empty: 'No results yet.', date: 'Date', value: 'Result', suite: 'Test suite', wptPassed: 'WPT tests passed', passRate: 'Pass rate', mediumPage: 'Medium page', startup: 'Browser startup', peakMemory: 'Peak memory', tests: 'tests passed', latest: 'latest p95 result'
  },
  zh: {
    skip: '跳到正文', experiment: '实验', architecture: '架构', progress: '数据', updates: '最新进展', github: 'GitHub', kicker: 'CI 测试结果', title: 'ZeroWeb 现在跑得怎么样？', intro: 'CI 会定期跑兼容性测试和性能基准。这里展示的是已经提交到仓库里的结果，可以看到它们随时间的变化。', loading: '正在读取最新结果…', fresh: '数据更新于 {date}', wptKicker: '网页兼容性', wptTitle: '通过的 WPT 测试', wptDescription: '每个月取最后一次上游 WPT 结果。测试集本身也会更新，所以图中同时保留版本号。', perfKicker: '每周性能测试', perfTitle: '性能基准（p95）', perfDescription: '这些结果都来自同一类 GitHub Runner，避免把机器差异误算成性能变化。数值越低越好。', source: '看原始数据 ↗', chooseMetric: '选择指标', method: '数据口径：WPT 按月取最后一次上游测试结果；性能按日期取 github-ubuntu-latest 的最后一次 p95。其他硬件上的结果不放进这张图。', footer: 'ZeroWeb 是一个由 AI 持续开发的 Rust 浏览器实验。', error: '测试数据没有加载成功。', empty: '还没有测试结果。', date: '日期', value: '结果', suite: '测试集版本', wptPassed: '通过的 WPT', passRate: '通过率', mediumPage: '中型页面', startup: '浏览器启动', peakMemory: '峰值内存', tests: '项通过', latest: '最近一次测试的 p95'
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

function setLanguage(next, persist = true) {
  language = copy[next] ? next : 'en';
  document.documentElement.lang = language === 'zh' ? 'zh-CN' : 'en';
  document.title = language === 'zh' ? '测试数据 — ZeroWeb' : 'Test results — ZeroWeb';
  document.querySelectorAll('[data-i18n]').forEach((element) => { element.textContent = t(element.dataset.i18n); });
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
    summaryCard(t('wptPassed'), latestWpt ? formatNumber(latestWpt.passed, 0) : '—', latestWpt ? `${latestWpt.rate}% · ${latestWpt.ref}` : t('empty')),
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
  const points = metricsData.wpt.map((item) => ({ ...item, value: item.passed }));
  document.getElementById('wpt-chart').innerHTML = chartMarkup(points, { dateLabel: (point) => point.period.slice(2), tickLabel: (value) => formatNumber(value, 0), tooltip: (point) => `${formatDate(point.period, true)} · ${formatNumber(point.passed, 0)} · ${point.rate}% · ${point.ref}`, ariaLabel: (point) => `${formatDate(point.period, true)}, ${formatNumber(point.passed, 0)} ${t('tests')}, ${point.rate}%`, chartLabel: t('wptTitle') });
  document.getElementById('wpt-table').innerHTML = tableMarkup(points, [t('date'), t('wptPassed'), t('passRate'), t('suite')], (point) => [formatDate(point.period, true), formatNumber(point.passed, 0), `${point.rate}%`, point.ref]);
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
  renderSummary(); renderWpt(); renderPerformancePicker(); renderPerformance();
}

const savedLanguage = localStorage.getItem('zeroweb-language');
const detectedLanguage = navigator.language.toLowerCase().startsWith('zh') ? 'zh' : 'en';
setLanguage(savedLanguage || detectedLanguage, false);
document.querySelectorAll('[data-language]').forEach((button) => button.addEventListener('click', () => setLanguage(button.dataset.language)));
document.getElementById('metric-select').addEventListener('change', renderPerformance);
fetch('metrics.json').then((response) => { if (!response.ok) throw new Error('unavailable'); return response.json(); }).then((data) => { metricsData = data; render(); }).catch(() => { document.getElementById('data-freshness').textContent = t('error'); });

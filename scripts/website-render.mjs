// Reuse the site's copy and chart renderers at build time without fetching data.
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const root = new URL('../website/', import.meta.url);
const language = process.argv[2];
if (!['en', 'zh'].includes(language)) throw new Error('Unsupported page language');
const metrics = JSON.parse(readFileSync(process.argv[3], 'utf8'));
const load = (name) => readFileSync(new URL(name, root), 'utf8');
const translations = vm.runInNewContext(`${load('script.js').split('\nfunction setLanguage')[0]}\ntranslations`);
const updates = vm.runInNewContext(`${load('updates.js').split('\nfunction setLanguage')[0]}\ncopy`);
const elements = {};
const document = {
  documentElement: { lang: language === 'zh' ? 'zh-CN' : 'en' },
  getElementById(id) {
    return elements[id] ||= { innerHTML: '', textContent: '', value: '' };
  },
};
const context = vm.createContext({ document, Intl, metrics, selectedLanguage: language });
vm.runInContext(load('progress.js').split('\n// Enhance the statically generated tables')[0], context);
vm.runInContext('metricsData = metrics; language = selectedLanguage; render();', context);
elements['metric-select'].innerHTML = elements['metric-select'].innerHTML.replace(
  `value="${elements['metric-select'].value}"`,
  `value="${elements['metric-select'].value}" selected`,
);
const progress = vm.runInContext('copy[language]', context);
progress.perfDescription = language === 'zh'
  ? '这些结果来自记录的 GitHub Runner 类别。比较时需同时核对指标、单位与测量条件；类别相同不保证硬件完全相同。数值越低越好。'
  : 'These results come from the recorded GitHub runner class. Compare the same metric, unit and measurement conditions; a class label does not guarantee identical hardware. Lower is better.';
process.stdout.write(JSON.stringify({ home: translations[language], updates: updates[language], progress, elements }));

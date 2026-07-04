import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const RESULT_PATH = join('.acceptance', 'reports', 'acceptance-result.json');
if (!existsSync(RESULT_PATH)) {
  console.error('No result file found. Run record_recipe_result.js first.');
  process.exit(1);
}

const result = JSON.parse(readFileSync(RESULT_PATH, 'utf-8'));

const allPass = result.fail === 0;

const verdict = {
  verdict: allPass ? 'ACCEPTED' : 'REJECTED',
  profile: result.profile,
  environment: `ZeroBrowser headless (win32, release, sdk-chrome)`,
  date: new Date().toISOString(),
  summary: allPass
    ? `All ${result.total} acceptance recipes passed.`
    : `${result.fail}/${result.total} recipe(s) failed.`,
  details: result.recipes.map(r => ({
    recipe: r.name,
    status: r.status,
  })),
};

writeFileSync(join('.acceptance', 'reports', 'verdict.json'), JSON.stringify(verdict, null, 2));
console.log(`Verdict: ${verdict.verdict}`);

import { mkdirSync, writeFileSync, appendFileSync } from 'node:fs';
import { join } from 'node:path';

export class Recorder {
  constructor(outputDir) {
    this.outputDir = outputDir;
    this.logs = [];
    this._ensureDirs();
  }

  _ensureDirs() {
    for (const sub of ['screenshots', 'jsons', 'diff']) {
      mkdirSync(join(this.outputDir, sub), { recursive: true });
    }
  }

  log(msg) {
    const entry = `[${new Date().toISOString()}] ${msg}`;
    this.logs.push(entry);
    console.log(entry);
    appendFileSync(join(this.outputDir, 'run.log'), entry + '\n', 'utf-8');
  }

  async screenshot(client, name) {
    const filePath = join(this.outputDir, 'screenshots', `${name}.png`);
    try {
      const { meta } = await client.screenshot(filePath);
      this.log(`Screenshot saved: ${name}.png (${meta.width}x${meta.height})`);
      return { filePath, meta };
    } catch (e) {
      this.log(`Screenshot FAILED: ${name} — ${e.message}`);
      return null;
    }
  }

  saveJson(name, data) {
    const filePath = join(this.outputDir, 'jsons', `${name}.json`);
    writeFileSync(filePath, JSON.stringify(data, null, 2));
    this.log(`JSON saved: ${name}.json`);
  }

  saveHtml(name, html) {
    const filePath = join(this.outputDir, 'jsons', `${name}.html`);
    writeFileSync(filePath, html, 'utf-8');
    this.log(`HTML saved: ${name}.html`);
  }

  summary(failures) {
    this.saveJson('_summary', {
      timestamp: new Date().toISOString(),
      totalSteps: this.logs.length,
      failures: failures || [],
      logs: this.logs,
    });
    if (failures && failures.length > 0) {
      this.log(`FAILURES (${failures.length}):`);
      for (const f of failures) this.log(`  ✗ ${f}`);
    }
  }
}

export function createRecorder(label) {
  const date = new Date().toISOString().slice(0, 10);
  const dir = join('.acceptance', 'artifacts', `${label}-${date}`);
  return new Recorder(dir);
}

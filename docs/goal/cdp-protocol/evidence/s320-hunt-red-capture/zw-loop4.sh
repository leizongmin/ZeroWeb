#!/bin/bash
# UNTRACED + validator-only: validator is in-memory (near-zero overhead), preserves race timing
cd "${ZW_REPO_ROOT:-.}/tests/playwright-matrix"
export ZW_IPC_VALIDATE=1
unset ZW_IPC_TRACE
for run in $(seq 1 4); do
  timeout 90 node -e "
const { spawn, execFileSync } = require('node:child_process')
const net = require('node:net')
const fs = require('node:fs')
function freePort() { return new Promise((res) => { const s = net.createServer(); s.listen(0, '127.0.0.1', () => { const p = s.address().port; s.close(() => res(p)) }) }) }
async function waitEndpoint(port, ms = 20000) { const dl = Date.now() + ms; while (Date.now() < dl) { try { const r = await fetch('http://127.0.0.1:' + port + '/json/version'); if (r.ok) return true } catch {} await new Promise(r => setTimeout(r, 200)) } return false }
;(async () => {
  const port = await freePort()
  const proc = spawn('../../target/debug/zero-browser', ['--headless', '--remote-debugging-port', String(port)], { stdio: ['ignore', 'ignore', 'pipe'], env: process.env })
  const chunks = []
  proc.stderr.on('data', (d) => chunks.push(d))
  if (!(await waitEndpoint(port))) { console.log('run $run: ENDPOINT NOT READY'); process.exit(3) }
  let status = 0
  try { execFileSync('node', ['scripts/capture-core-flow.mjs'], { cwd: '.', env: { ...process.env, CDP_ENDPOINT_URL: 'http://127.0.0.1:' + port }, timeout: 300000, stdio: 'pipe' }) } catch (e) { status = e.status }
  await new Promise((r) => setTimeout(r, 500))
  proc.kill('SIGTERM')
  fs.writeFileSync('/tmp/zw-hunt-stderr.log', Buffer.concat(chunks))
})()
"
  GREEN=$(python3 -c "
import json
d=json.load(open('out/steps-report.json'))
print(sum(1 for s in d['steps'] if s['ok'] and s['step']!='observations'))
" 2>/dev/null)
  DIED=$(grep -cE "ipc reader terminated|malformed frame buffer" /tmp/zw-hunt-stderr.log 2>/dev/null; true)
  echo "run $run: green=$GREEN anomalies=$DIED"
  if [ "$DIED" != "0" ] || [ "$GREEN" != "33" ]; then
    cp /tmp/zw-hunt-stderr.log /tmp/zw-trace-RED-stderr.log
    echo "RED FIRED at run $run"
    exit 0
  fi
done
echo "no RED in 4 runs"

#!/bin/bash
cd "${ZW_REPO_ROOT:-.}/tests/playwright-matrix"
export ZW_IPC_TRACE=/tmp/zw-trace-hunt.log
export ZW_IPC_VALIDATE=1
# background CPU load to stress timing (3 busy loops)
for i in 1 2 3; do (while :; do :; done) & done
LOADPIDS=$(jobs -p)
for run in $(seq 1 10); do
  rm -f /tmp/zw-trace-hunt.log
  node -e "
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
  await new Promise((r) => setTimeout(r, 800))
  proc.kill('SIGTERM')
  fs.writeFileSync('/tmp/zw-hunt-stderr.log', Buffer.concat(chunks))
})()
EXIT=$?
GREEN=$(python3 -c "
import json
d=json.load(open('out/steps-report.json'))
print(sum(1 for s in d['steps'] if s['ok'] and s['step']!='observations'))
" 2>/dev/null)
DIED=$(grep -c 'ipc reader terminated\|malformed frame buffer' /tmp/zw-hunt-stderr.log 2>/dev/null)
echo "run $run: exit=$EXIT green=$GREEN anomalies=$DIED"
if [ "$DIED" != "0" ] || [ "$GREEN" != "33" ]; then
  cp /tmp/zw-trace-hunt.log /tmp/zw-trace-RED.log
  cp /tmp/zw-hunt-stderr.log /tmp/zw-trace-RED-stderr.log
  echo "RED FIRED at run $run — trace saved"
  kill $LOADPIDS 2>/dev/null
  exit 0
fi
done
kill $LOADPIDS 2>/dev/null
echo "no RED in 10 runs"

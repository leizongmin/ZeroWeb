import puppeteer from 'puppeteer-core';
try {
  const b = await puppeteer.connect({browserURL: 'http://localhost:9227'});
  const p = await b.newPage();
  await p.setViewport({width: 200, height: 100});
  await p.goto('about:blank');
  await p.screenshot({path: '/tmp/pup-connect-shot.png'});
  console.log('SCREENSHOT OK');
  await p.close();
  await b.disconnect();
} catch (e) { console.log('FAIL:', e.message.slice(0,150)); }

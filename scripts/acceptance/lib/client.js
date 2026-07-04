import { EventEmitter } from 'node:events';
import { readFileSync, writeFileSync } from 'node:fs';
import { setTimeout as sleep } from 'node:timers/promises';
import WebSocket from 'ws';

export class Rect {
  constructor(x, y, width, height) {
    this.x = x;
    this.y = y;
    this.width = width;
    this.height = height;
  }

  get center() {
    return { x: this.x + this.width / 2, y: this.y + this.height / 2 };
  }

  get bottom() {
    return this.y + this.height;
  }
}

export class ChromeAcceptanceClient extends EventEmitter {
  constructor(url = 'ws://127.0.0.1:9222', timeout = 300_000) {
    super();
    this.url = url;
    this.timeout = timeout;
    this._ws = null;
    this._nextId = 1;
    this._pending = new Map();
  }

  async connect(retries = 20, interval = 500) {
    for (let i = 0; i < retries; i++) {
      try {
        this._ws = new WebSocket(this.url, { handshakeTimeout: this.timeout });
        await new Promise((resolve, reject) => {
          this._ws.once('open', resolve);
          this._ws.once('error', reject);
          this._ws.once('close', reject);
        });
        this._ws.on('message', (data) => this._onMessage(data));
        this._ws.on('close', () => this._onClose());
        this._ws.on('error', (err) => this.emit('error', err));
        return;
      } catch (e) {
        if (i === retries - 1) throw new Error(`Cannot connect to ${this.url}: ${e.message}`);
        await sleep(interval);
      }
    }
  }

  close() {
    if (this._ws) {
      this._ws.close();
      this._ws = null;
    }
  }

  _onMessage(data) {
    let msg;
    try {
      msg = JSON.parse(data.toString());
    } catch {
      return;
    }
    if (msg.id != null) {
      const resolver = this._pending.get(msg.id);
      if (resolver) {
        this._pending.delete(msg.id);
        resolver(msg);
      }
    } else if (msg.method) {
      this.emit('event', msg.method, msg.params || {});
    }
  }

  _onClose() {
    for (const [, resolver] of this._pending) {
      resolver({ error: { code: -32000, message: 'Connection closed' } });
    }
    this._pending.clear();
    this.emit('close');
  }

  async _send(method, params = {}) {
    if (!this._ws) throw new Error('Not connected; call connect() first');
    const id = this._nextId++;
    const msg = JSON.stringify({ id, method, params });
    this._ws.send(msg);
    const resp = await new Promise((resolve) => {
      this._pending.set(id, resolve);
    });
    if (resp.error) throw new Error(`${method} failed: ${JSON.stringify(resp.error)}`);
    return resp.result || {};
  }

  async navigate(url, timeout) {
    const old = this.timeout;
    if (timeout) this.timeout = timeout;
    try {
      return await this._send('browsingContext.navigate', { url });
    } finally {
      this.timeout = old;
    }
  }

  async newTab(url) {
    return this._send('browsingContext.create', { url });
  }

  async closeTab(context) {
    return this._send('browsingContext.close', { context });
  }

  async getTree() {
    return this._send('browsingContext.getTree');
  }

  async screenshot(savePath) {
    const result = await this._send('browsingContext.captureScreenshot');
    const meta = result.data || {};
    let pngBytes = null;
    if (result.pixels) {
      pngBytes = Buffer.from(result.pixels, 'base64');
      if (savePath) writeFileSync(savePath, pngBytes);
    }
    return { meta, pngBytes };
  }

  async getLayout() {
    return this._send('chrome.getLayout');
  }

  async getViewportRect() {
    const layout = await this.getLayout();
    const vp = layout.viewport;
    if (!vp) return null;
    return new Rect(vp.x, vp.y, vp.width, vp.height);
  }

  async getSemantics() {
    const result = await this._send('chrome.getSemantics');
    return result.tree;
  }

  async click(x, y, widgetId) {
    const params = {};
    if (widgetId != null) {
      params.widgetId = widgetId;
    } else {
      if (x == null || y == null) throw new Error('Either widgetId or (x, y) required');
      params.x = x;
      params.y = y;
    }
    return this._send('chrome.click', params);
  }

  async rectOf(widgetId) {
    const result = await this._send('chrome.rectOf', { widgetId });
    const r = result.rect;
    return new Rect(r.x, r.y, r.width, r.height);
  }

  async emittedActions() {
    const result = await this._send('chrome.emittedActions');
    return result.actions || [];
  }

  async findWidgetByLabel(labelSubstr) {
    const tree = await this.getSemantics();
    return _searchSemantics(tree, (n) => (n.label || '').toLowerCase().includes(labelSubstr.toLowerCase()));
  }

  async findWidgetByFlag(flag) {
    const tree = await this.getSemantics();
    const results = [];
    _collectSemantics(tree, (n) => (n.flags || []).includes(flag), results);
    return results;
  }
}

function _searchSemantics(node, predicate) {
  if (predicate(node)) return node;
  for (const child of (node.children || [])) {
    const found = _searchSemantics(child, predicate);
    if (found) return found;
  }
  return null;
}

function _collectSemantics(node, predicate, results) {
  if (predicate(node)) results.push(node);
  for (const child of (node.children || [])) {
    _collectSemantics(child, predicate, results);
  }
}

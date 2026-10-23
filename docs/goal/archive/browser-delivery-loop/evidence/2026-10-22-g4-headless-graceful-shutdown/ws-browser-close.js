// Minimal RFC6455 WS client: handshake + send Browser.close + print first
// server frame (response or close). Exit 0 on any server-side finish
// (response / close frame / TCP close), 1 on handshake failure or timeout.
// Usage: node ws-browser-close.js <port>
'use strict';
const net = require('net');
const crypto = require('crypto');

const port = parseInt(process.argv[2], 10);
if (!Number.isInteger(port)) {
    console.error('usage: node ws-browser-close.js <port>');
    process.exit(1);
}

const key = crypto.randomBytes(16).toString('base64');
const sock = net.connect(port, '127.0.0.1', () => {
    sock.write(
        `GET / HTTP/1.1\r\n` +
        `Host: 127.0.0.1:${port}\r\n` +
        `Upgrade: websocket\r\n` +
        `Connection: Upgrade\r\n` +
        `Sec-WebSocket-Key: ${key}\r\n` +
        `Sec-WebSocket-Version: 13\r\n\r\n`
    );
});

let buf = Buffer.alloc(0);
let stage = 0; // 0 = handshake, 1 = frame exchange
let done = false;
const finish = (ok, msg) => {
    if (done) return;
    done = true;
    if (msg) console.log(msg);
    process.exit(ok ? 0 : 1);
};

sock.on('data', (d) => {
    buf = Buffer.concat([buf, d]);
    if (stage === 0) {
        const idx = buf.indexOf('\r\n\r\n');
        if (idx === -1) return;
        const head = buf.slice(0, idx).toString('latin1');
        const accept = crypto
            .createHash('sha1')
            .update(key + '258EAFA5-E914-47DA-95CA-C5AB0DC85B11')
            .digest('base64');
        if (!/^HTTP\/1\.1 101/.test(head) || !head.includes(accept)) {
            finish(false, 'handshake failed: ' + head.split('\r\n')[0]);
            return;
        }
        stage = 1;
        buf = buf.slice(idx + 4);
        const payload = Buffer.from('{"id":1,"method":"browser.close"}');
        const mask = crypto.randomBytes(4);
        let header;
        if (payload.length < 126) {
            header = Buffer.from([0x81, 0x80 | payload.length]);
        } else {
            header = Buffer.alloc(4);
            header[0] = 0x81;
            header[1] = 0x80 | 126;
            header.writeUInt16BE(payload.length, 2);
        }
        const masked = Buffer.alloc(payload.length);
        for (let i = 0; i < payload.length; i++) masked[i] = payload[i] ^ mask[i % 4];
        sock.write(Buffer.concat([header, mask, masked]));
    }
    // Parse server frames (unmasked server→client).
    while (stage === 1 && buf.length >= 2) {
        const opcode = buf[0] & 0x0f;
        let len = buf[1] & 0x7f;
        let off = 2;
        if (len === 126) {
            if (buf.length < 4) return;
            len = buf.readUInt16BE(2);
            off = 4;
        } else if (len === 127) {
            if (buf.length < 10) return;
            len = Number(buf.readBigUInt64BE(2));
            off = 10;
        }
        if (buf.length < off + len) return;
        const body = buf.slice(off, off + len);
        if (opcode === 0x1) {
            finish(true, body.toString());
            return;
        }
        if (opcode === 0x8) {
            finish(true, 'server-close-frame');
            return;
        }
        buf = buf.slice(off + len);
    }
});

sock.on('close', () => finish(true, 'tcp-closed'));
sock.on('error', (e) => finish(false, 'socket error: ' + e.message));
setTimeout(() => finish(false, 'timeout waiting for server frame'), 8000);

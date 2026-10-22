#!/usr/bin/env python3
"""BG goal 本地 fixture HTTP server：静态目录服务 + 停滞子资源 + 连接计数。

用途：background-resource-lifecycle goal M1 基线观测（BG-03 重复导航的
「不结束子资源」腿）。零第三方依赖，仅标准库。

端点：
  /<fixture 文件>          静态服务 --dir 目录
  /stall?ms=<N>&tag=<t>    发 200 头 + 少量 body 后保持连接打开 N ms（模拟
                           停滞/长连接子资源）；期间统计活跃连接数
  /__stats                 JSON {"requests", "stall_active", "stall_total",
                           "stall_closed_by_peer"}

用法：scripts/background-resource-fixture-server.py --dir <fixture-dir>
      --port <port> --log <out.jsonl>
"""
from __future__ import annotations

import argparse
import functools
import json
import threading
import time
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse, parse_qs


class Counters:
    def __init__(self) -> None:
        self.lock = threading.Lock()
        self.requests = 0
        self.stall_active = 0
        self.stall_total = 0
        self.stall_closed_by_peer = 0

    def stats(self) -> dict:
        with self.lock:
            return {
                "requests": self.requests,
                "stall_active": self.stall_active,
                "stall_total": self.stall_total,
                "stall_closed_by_peer": self.stall_closed_by_peer,
            }


COUNTERS = Counters()
LOG_LOCK = threading.Lock()
LOG_FILE = None


def log_line(obj: dict) -> None:
    obj = dict(obj)
    obj["t_ms"] = int(time.time() * 1000)
    line = json.dumps(obj, ensure_ascii=False)
    with LOG_LOCK:
        if LOG_FILE is not None:
            LOG_FILE.write(line + "\n")
            LOG_FILE.flush()


class Handler(SimpleHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt: str, *args) -> None:  # 静默默认 stderr 访问日志
        pass

    def _log_request(self, extra: dict | None = None) -> None:
        entry = {"path": self.path, "client": f"{self.client_address[0]}:{self.client_address[1]}"}
        if extra:
            entry.update(extra)
        log_line(entry)

    def do_GET(self) -> None:  # noqa: N802（BaseHTTPRequestHandler 命名）
        parsed = urlparse(self.path)
        with COUNTERS.lock:
            COUNTERS.requests += 1
        if parsed.path == "/__stats":
            body = json.dumps(COUNTERS.stats()).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(body)
            return
        if parsed.path == "/stall":
            qs = parse_qs(parsed.query)
            ms = min(int(qs.get("ms", ["60000"])[0]), 86_400_000)
            tag = qs.get("tag", [""])[0]
            with COUNTERS.lock:
                COUNTERS.stall_active += 1
                COUNTERS.stall_total += 1
            self._log_request({"stall": True, "ms": ms, "tag": tag})
            try:
                # 发头 + 少量 body 后挂起：客户端 fetch 保持 pending，
                # 浏览器导航丢弃时底层连接行为由 peer 关闭计数体现。
                body = b"stall\n"
                self.send_response(200)
                self.send_header("Content-Type", "text/plain")
                self.send_header("Content-Length", str(len(body) + 1024))
                self.send_header("Cache-Control", "no-store")
                self.end_headers()
                self.wfile.write(body)
                self.wfile.flush()
                deadline = time.monotonic() + ms / 1000.0
                while time.monotonic() < deadline:
                    time.sleep(min(0.5, max(0.0, deadline - time.monotonic())))
            except (BrokenPipeError, ConnectionResetError):
                with COUNTERS.lock:
                    COUNTERS.stall_closed_by_peer += 1
                self.close_connection = True
                return
            finally:
                with COUNTERS.lock:
                    COUNTERS.stall_active -= 1
            self.close_connection = True
            return
        super().do_GET()


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dir", required=True)
    ap.add_argument("--port", type=int, required=True)
    ap.add_argument("--log", required=True)
    args = ap.parse_args()

    global LOG_FILE
    LOG_FILE = open(args.log, "w", encoding="utf-8")

    server = ThreadingHTTPServer(("127.0.0.1", args.port), functools.partial(Handler, directory=args.dir))
    server.daemon_threads = True
    log_line({"event": "server_start", "port": args.port, "dir": args.dir})
    print(f"fixture-server listening on 127.0.0.1:{args.port}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        log_line(dict(COUNTERS.stats(), event="server_stop"))
        LOG_FILE.close()


if __name__ == "__main__":
    main()

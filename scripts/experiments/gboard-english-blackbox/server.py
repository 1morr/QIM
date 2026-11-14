#!/usr/bin/env python3
"""提供 page/ 的測試頁，把每個 POST /log 的內容（一筆一行 JSON）接到記錄檔後面。

用法：server.py [埠] [記錄檔]   （預設 18719、./events.log）
"""
import http.server
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 18719
LOG = sys.argv[2] if len(sys.argv) > 2 else "events.log"


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=os.path.join(HERE, "page"), **kwargs)

    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("Content-Length", 0))).decode()
        with open(LOG, "a") as f:
            f.write(f"{time.time():.3f} {body}\n")
        self.send_response(204)
        self.end_headers()

    def log_message(self, *args):
        pass


http.server.ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()

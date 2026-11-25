"""Local demo only: a dependency-free API for trying Look Out API."""
import argparse
import json
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        path = self.path.split("?", 1)[0]
        if path == "/slow":
            time.sleep(15)
        if path == "/health" or path == "/slow":
            self.respond(200, {"status": "ok"})
        elif path == "/fail":
            self.respond(503, {"status": "unavailable"})
        elif path == "/orders":
            if self.headers.get("Authorization") == "Bearer local-demo-token":
                self.respond(200, {"orders": [{"id": "demo-1", "total": 500}]})
            else:
                self.respond(401, {"error": "unauthorized"})
        else:
            self.respond(404, {"error": "not found"})

    def do_POST(self):
        if self.path == "/login":
            self.respond(200, {"access_token": "local-demo-token"})
        else:
            self.respond(404, {"error": "not found"})

    def respond(self, status, body):
        data = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        try:
            self.wfile.write(data)
        except (BrokenPipeError, ConnectionResetError):
            pass

    def log_message(self, *_):
        pass


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--bind", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=3001)
    options = parser.parse_args()
    server = ThreadingHTTPServer((options.bind, options.port), Handler)
    print(f"Demo API: http://{options.bind}:{options.port}/health", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.server_close()

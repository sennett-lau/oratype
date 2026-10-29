"""Forwards /v1/* to a llama-server and saves every chat request body (to replay real prompts)."""
import http.server, json, pathlib, sys, urllib.request
UPSTREAM, PORT, OUT = sys.argv[1], int(sys.argv[2]), pathlib.Path(sys.argv[3])
OUT.mkdir(parents=True, exist_ok=True)
class H(http.server.BaseHTTPRequestHandler):
    def _forward(self, body=None):
        if body is not None and self.path.endswith('/chat/completions'):
            n = len(list(OUT.glob('*.json')))
            (OUT / f'{n:03d}.json').write_bytes(body)
        req = urllib.request.Request(UPSTREAM + self.path, data=body, method=self.command,
                                     headers={k: v for k, v in self.headers.items() if k.lower() in ('content-type', 'authorization')})
        with urllib.request.urlopen(req, timeout=600) as r:
            data = r.read()
            self.send_response(r.status)
            self.send_header('Content-Type', r.headers.get('Content-Type', 'application/json'))
            self.send_header('Content-Length', str(len(data)))
            self.end_headers()
            self.wfile.write(data)
    def do_GET(self): self._forward()
    def do_POST(self): self._forward(self.rfile.read(int(self.headers.get('Content-Length', 0))))
    def log_message(self, *a): pass
http.server.ThreadingHTTPServer(('127.0.0.1', PORT), H).serve_forever()

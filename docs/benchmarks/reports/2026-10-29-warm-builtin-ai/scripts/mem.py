"""Memory of one polish server after a few warm requests in both languages.

    python3 -I mem.py <model.gguf> <slots>
"""
import json, os, pathlib, re, subprocess, sys, time, urllib.request

HERE = pathlib.Path(os.environ.get('BENCH_DIR', '.')).resolve()
REPO = pathlib.Path(__file__).resolve().parents[5]
SERVER = str(REPO / 'src-tauri/target/llama.cpp/build-b11177/bin/llama-server')
model, slots, PORT = pathlib.Path(sys.argv[1]), int(sys.argv[2]), 8341
bodies = [json.loads(p.read_text()) for p in sorted((HERE / 'prompts').glob('*.json'))]
pick = {('yue' if '香港' in b['messages'][0]['content'] else 'en'): b for b in bodies}
proc = subprocess.Popen([SERVER, '--model', str(model), '--alias', 'p', '--host', '127.0.0.1', '--port', str(PORT),
                         '--n-gpu-layers', '999', '--ctx-size', str(8192 * slots), '--parallel', str(slots),
                         '--reasoning', 'off', '--no-webui', '--offline'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def mem():
    fp = subprocess.run(['footprint', '-p', str(proc.pid)], capture_output=True, text=True).stdout
    hit = re.search(r'phys_footprint: ([0-9.]+ [KMG]B)', fp) or re.search(r'Footprint: ([0-9.]+ [KMG]B)', fp)
    rss = int(subprocess.run(['ps', '-o', 'rss=', '-p', str(proc.pid)], capture_output=True, text=True).stdout.strip()) // 1024
    return {'footprint': hit[1] if hit else None, 'rss_mb': rss}

try:
    while True:
        try:
            with urllib.request.urlopen(f'http://127.0.0.1:{PORT}/health', timeout=5) as r:
                if json.loads(r.read()).get('status') == 'ok':
                    break
        except Exception:
            time.sleep(0.3)
    idle = mem()
    for lang in ['yue', 'en', 'yue', 'en']:
        body = dict(pick[lang], temperature=0, max_tokens=64)
        if slots == 2:
            body['id_slot'] = 0 if lang == 'yue' else 1
        urllib.request.urlopen(urllib.request.Request(f'http://127.0.0.1:{PORT}/v1/chat/completions',
                               data=json.dumps(body).encode(), headers={'Content-Type': 'application/json'}),
                               timeout=600).read()
    print(json.dumps({'model': model.name, 'slots': slots, 'idle': idle, 'after_requests': mem()}))
finally:
    proc.terminate(); proc.wait(timeout=30)

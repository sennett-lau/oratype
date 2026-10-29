"""Replays real polish prompts alternating Cantonese and English, as a bilingual user dictates,
with one llama-server slot (the app today) or two, and reports what each request costs.

    python3 -I switch.py <model.gguf> <parallel>
"""
import json, os, pathlib, statistics, subprocess, sys, time, urllib.request
# Model files (dl/), long clips (long/), recorded prompts (prompts/) and results/ live here.
HERE = pathlib.Path(os.environ.get('BENCH_DIR', '.')).resolve()
REPO = pathlib.Path(__file__).resolve().parents[5]
SERVER = str(REPO / 'src-tauri/target/llama.cpp/build-b11177/bin/llama-server')
model, parallel = sys.argv[1], int(sys.argv[2])
port = 8295
proc = subprocess.Popen([SERVER, '--model', model, '--alias', 'qwen3.5-4b', '--host', '127.0.0.1', '--port', str(port),
                         '--n-gpu-layers', '999', '--ctx-size', str(8192 * parallel), '--parallel', str(parallel),
                         '--reasoning', 'off', '--no-webui', '--offline'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
yue = [json.loads((HERE / f'prompts/{i:03d}.json').read_text()) for i in (3, 4, 5)]
en = [json.loads((HERE / f'prompts/{i:03d}.json').read_text()) for i in (0, 1, 2)]
order = [b for pair in zip(yue, en) for b in pair] * 2   # yue, en, yue, en, ... twice
try:
    while True:
        try:
            with urllib.request.urlopen(f'http://127.0.0.1:{port}/health', timeout=5) as r:
                if json.loads(r.read()).get('status') == 'ok':
                    break
        except Exception:
            time.sleep(0.5)
    rows = []
    for body in order:
        body = dict(body, temperature=0)
        req = urllib.request.Request(f'http://127.0.0.1:{port}/v1/chat/completions', data=json.dumps(body).encode(),
                                     headers={'Content-Type': 'application/json'})
        t = time.perf_counter()
        with urllib.request.urlopen(req, timeout=600) as r:
            res = json.loads(r.read())
        wall = time.perf_counter() - t
        tm = res.get('timings', {})
        lang = 'yue' if '香港' in body['messages'][0]['content'] else 'en'
        rows.append((lang, round(wall, 2), tm.get('prompt_n'), round(tm.get('prompt_ms', 0) / 1000, 2), tm.get('predicted_n')))
    for i, r in enumerate(rows):
        print(('first ' if i < 6 else 'later ') + str(r))
    later = [r[1] for r in rows[6:]]
    print(json.dumps({'parallel': parallel, 'later_median_s': statistics.median(later), 'later_total_s': round(sum(later), 1),
                      'later_prompt_tokens': [r[2] for r in rows[6:]]}))
finally:
    proc.terminate(); proc.wait(timeout=30)

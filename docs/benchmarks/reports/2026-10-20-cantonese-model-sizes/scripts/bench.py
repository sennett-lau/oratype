"""Benchmarks one built-in model file on this Mac, with the app's llama-server arguments.

    python3 -I bench.py speech <model.gguf> <label>
    python3 -I bench.py polish <model.gguf> <label>

speech: Qwen3-ASR with the Q8_0 mmproj, --ctx-size 12288; the 24 eval clips (quality), then the
        28 s and 56 s Cantonese clips 5 times each (speed).
polish: Qwen 3.5 4B, --ctx-size 8192 --reasoning off; the eval's Cantonese cases (dev+holdout)
        plus English and Mandarin dev, routed as a Cantonese user's app (en, zh-Hant-HK, presets).
Writes results/<kind>-<label>/: eval output, timings.json (wall times, server metrics deltas,
memory).
"""
import json, os, re, statistics, subprocess, sys, time, urllib.request, uuid, pathlib

# Model files (dl/), long clips (long/), recorded prompts (prompts/) and results/ live here.
HERE = pathlib.Path(os.environ.get('BENCH_DIR', '.')).resolve()
REPO = pathlib.Path(__file__).resolve().parents[5]  # the repository root
SERVER = REPO / 'src-tauri/target/llama.cpp/build-b11177/bin/llama-server'
MMPROJ = HERE / 'dl/mmproj-Qwen3-ASR-1.7B-Q8_0.gguf'
PORT = 8291

kind, model, label = sys.argv[1], pathlib.Path(sys.argv[2]), sys.argv[3]
out = HERE / 'results' / f'{kind}-{label}'
out.mkdir(parents=True, exist_ok=True)
alias = 'qwen3-asr-1.7b' if kind == 'speech' else 'qwen3.5-4b'
args = [str(SERVER), '--model', str(model), '--alias', alias, '--host', '127.0.0.1', '--port', str(PORT),
        '--n-gpu-layers', '999', '--ctx-size', '12288' if kind == 'speech' else '8192', '--parallel', '1',
        '--no-webui', '--offline', '--metrics']
if kind == 'speech':
    args += ['--mmproj', str(MMPROJ)]
else:
    args += ['--reasoning', 'off']
log = open(out / 'server.log', 'w')
proc = subprocess.Popen(args, stdout=log, stderr=subprocess.STDOUT)
base = f'http://127.0.0.1:{PORT}'

def get(path):
    with urllib.request.urlopen(base + path, timeout=30) as r:
        return r.read().decode()

def metrics():
    m = {}
    for line in get('/metrics').splitlines():
        hit = re.match(r'llamacpp:(prompt_tokens_total|prompt_seconds_total|tokens_predicted_total|tokens_predicted_seconds_total|n_decode_total) ([0-9.e+]+)', line)
        if hit:
            m[hit[1]] = float(hit[2])
    return m

def delta(a, b):
    d = {k: b[k] - a.get(k, 0) for k in b}
    if d.get('prompt_seconds_total'):
        d['prompt_tokens_per_s'] = d['prompt_tokens_total'] / d['prompt_seconds_total']
    if d.get('tokens_predicted_seconds_total'):
        d['predicted_tokens_per_s'] = d['tokens_predicted_total'] / d['tokens_predicted_seconds_total']
    return d

def footprint():
    r = subprocess.run(['footprint', '-p', str(proc.pid)], capture_output=True, text=True).stdout
    hit = re.search(r'phys_footprint: ([0-9.]+) ([KMG]B)', r) or re.search(r'Footprint: ([0-9.]+) ([KMG]B)', r)
    rss = int(subprocess.run(['ps', '-o', 'rss=', '-p', str(proc.pid)], capture_output=True, text=True).stdout.strip() or 0) / 1024
    return {'footprint': f'{hit[1]} {hit[2]}' if hit else None, 'rss_mb': round(rss)}

def transcribe(wav):
    boundary = uuid.uuid4().hex
    body = b''.join([
        f'--{boundary}\r\nContent-Disposition: form-data; name="model"\r\n\r\n{alias}\r\n'.encode(),
        f'--{boundary}\r\nContent-Disposition: form-data; name="response_format"\r\n\r\njson\r\n'.encode(),
        f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="a.wav"\r\nContent-Type: audio/wav\r\n\r\n'.encode(),
        open(wav, 'rb').read(), f'\r\n--{boundary}--\r\n'.encode()])
    req = urllib.request.Request(base + '/v1/audio/transcriptions', data=body,
                                 headers={'Content-Type': f'multipart/form-data; boundary={boundary}'})
    t = time.perf_counter()
    with urllib.request.urlopen(req, timeout=300) as r:
        text = json.loads(r.read())['text']
    return time.perf_counter() - t, text

try:
    started = time.perf_counter()
    while True:
        try:
            if json.loads(get('/health')).get('status') == 'ok':
                break
        except Exception:
            pass
        if proc.poll() is not None:
            sys.exit('server exited; see server.log')
        time.sleep(0.5)
    result = {'model': model.name, 'size_bytes': model.stat().st_size, 'kind': kind,
              'ready_s': round(time.perf_counter() - started, 1)}
    env = dict(os.environ, ORATYPE_LLAMA_SERVER=str(SERVER))
    if kind == 'speech':
        transcribe(HERE / 'long/yue-30s.wav')  # warm-up
        m0 = metrics()
        subprocess.run(['npm', 'run', 'eval', '--', 'speech', '--speech-url', f'{base}/v1', '--speech-model', alias,
                        '--split', 'all', '--baseline', 'none', '--out', str(out / 'eval')], cwd=REPO, env=env,
                       stdout=open(out / 'eval.log', 'w'), stderr=subprocess.STDOUT)
        result['eval_metrics'] = delta(m0, metrics())
        for clip in ['yue-30s', 'yue-60s']:
            m0 = metrics(); times = []; text = ''
            for _ in range(5):
                s, text = transcribe(HERE / f'long/{clip}.wav'); times.append(s)
            result[clip] = {'median_s': round(statistics.median(times), 3), 'min_s': round(min(times), 3),
                            'max_s': round(max(times), 3), 'metrics': delta(m0, metrics()), 'text': text}
    else:
        m0 = metrics()
        for langs, split, n in [('yue', 'all', 3), ('en,zh-Hans', 'dev', 2)]:
            tag = langs.replace(',', '+')
            subprocess.run(['npm', 'run', 'eval', '--', 'polish', '--ai-url', f'{base}/v1', '--ai-model', alias,
                            '--lang', langs, '--split', split, '--n', str(n), '--languages', 'en,zh-Hant-HK',
                            '--presets', '--baseline', 'none', '--out', str(out / f'eval-{tag}')], cwd=REPO, env=env,
                           stdout=open(out / f'eval-{tag}.log', 'w'), stderr=subprocess.STDOUT)
            m1 = metrics(); result[f'metrics_{tag}'] = delta(m0, m1); m0 = m1
    result['memory'] = footprint()
    (out / 'timings.json').write_text(json.dumps(result, ensure_ascii=False, indent=1))
    print(json.dumps({k: v for k, v in result.items() if k not in ('yue-30s', 'yue-60s')}, ensure_ascii=False)[:600])
finally:
    proc.terminate(); proc.wait(timeout=30)

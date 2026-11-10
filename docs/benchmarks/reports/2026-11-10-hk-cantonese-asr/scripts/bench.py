"""Bench one Qwen3-ASR file pair with the app's built-in llama-server arguments.

    python3 -I bench.py <label> <model.gguf> <mmproj.gguf> <out_dir> [--eval]
"""
import json, pathlib, statistics, subprocess, sys, time, urllib.request, uuid, os, random

REPO = pathlib.Path(__file__).resolve().parents[5]
SERVER = REPO / 'src-tauri/binaries/llama-server-aarch64-apple-darwin'
HERE = pathlib.Path(__file__).resolve().parent
PORT = 8293
BASE = f'http://127.0.0.1:{PORT}'
EVICT = sorted((REPO / 'evals/speech/en/audio/synthetic').glob('*.wav'))

label, model, mmproj, out = sys.argv[1:5]
out = pathlib.Path(out); out.mkdir(parents=True, exist_ok=True)


def transcribe(wav):
    b = uuid.uuid4().hex
    body = b''.join([
        f'--{b}\r\nContent-Disposition: form-data; name="model"\r\n\r\nqwen3-asr-1.7b\r\n'.encode(),
        f'--{b}\r\nContent-Disposition: form-data; name="response_format"\r\n\r\njson\r\n'.encode(),
        f'--{b}\r\nContent-Disposition: form-data; name="file"; filename="a.wav"\r\nContent-Type: audio/wav\r\n\r\n'.encode(),
        pathlib.Path(wav).read_bytes(), f'\r\n--{b}--\r\n'.encode()])
    req = urllib.request.Request(BASE + '/v1/audio/transcriptions', data=body,
                                 headers={'Content-Type': f'multipart/form-data; boundary={b}'})
    t = time.perf_counter()
    with urllib.request.urlopen(req, timeout=120) as r:
        text = json.loads(r.read())['text']
    return time.perf_counter() - t, text


args = [str(SERVER), '--model', model, '--mmproj', mmproj, '--host', '127.0.0.1', '--port', str(PORT),
        '--n-gpu-layers', '999', '--ctx-size', '12288', '--parallel', '1', '--alias', 'qwen3-asr-1.7b']
log = open(out / 'server.log', 'w')
proc = subprocess.Popen(args, stdout=log, stderr=subprocess.STDOUT)
try:
    started = time.perf_counter()
    while True:
        try:
            with urllib.request.urlopen(BASE + '/health', timeout=2) as r:
                if r.status == 200:
                    break
        except Exception:
            pass
        if proc.poll() is not None:
            sys.exit('server exited; see server.log')
        time.sleep(0.5)
    result = {'label': label, 'model': os.path.basename(model), 'ready_s': round(time.perf_counter() - started, 1)}
    transcribe(HERE / 'long/yue-a.wav')  # warm-up
    if '--eval' in sys.argv:
        subprocess.run(['npm', 'run', 'eval', '--', 'speech', '--speech-url', f'{BASE}/v1', '--speech-model',
                        'qwen3-asr-1.7b', '--split', 'all', '--baseline', 'none', '--out', str(out / 'eval')],
                       cwd=REPO, stdout=open(out / 'eval.log', 'w'), stderr=subprocess.STDOUT)
    rnd = random.Random(1)
    for clip in ['yue-a', 'yue-b']:
        times = []; text = ''
        for _ in range(5):
            transcribe(rnd.choice(EVICT))
            s, text = transcribe(HERE / f'long/{clip}.wav'); times.append(s)
        result[clip] = {'median_s': round(statistics.median(times), 3), 'min_s': round(min(times), 3),
                        'max_s': round(max(times), 3), 'text': text}
    rss = subprocess.run(['ps', '-o', 'rss=', '-p', str(proc.pid)], capture_output=True, text=True).stdout.strip()
    result['rss_gb'] = round(int(rss) / 1024 / 1024, 2) if rss else None
    (out / 'result.json').write_text(json.dumps(result, ensure_ascii=False, indent=1))
    print(json.dumps(result, ensure_ascii=False, indent=1))
finally:
    proc.terminate(); proc.wait()

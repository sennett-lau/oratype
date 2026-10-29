"""Times long Cantonese dictations with each Qwen3-ASR file, as a new dictation runs: before every
timed request a different short clip goes through, so the server's cache never holds the audio
(a real dictation is always new audio). Same llama-server arguments as the app.

    python3 -I long.py <label> [<label> ...]      # dl/Qwen3-ASR-1.7B-<label>.gguf
"""
import json, os, re, statistics, subprocess, sys, time, urllib.request, uuid, pathlib

# Model files (dl/), long clips (long/), recorded prompts (prompts/) and results/ live here.
HERE = pathlib.Path(os.environ.get('BENCH_DIR', '.')).resolve()
REPO = pathlib.Path(__file__).resolve().parents[5]  # the repository root
SERVER = REPO / 'src-tauri/target/llama.cpp/build-b11177/bin/llama-server'
MMPROJ = HERE / 'dl/mmproj-Qwen3-ASR-1.7B-Q8_0.gguf'
PORT = 8292
BASE = f'http://127.0.0.1:{PORT}'
EVICT = sorted((REPO / 'evals/speech/en/audio/synthetic').glob('*.wav'))


def get(path):
    with urllib.request.urlopen(BASE + path, timeout=30) as r:
        return r.read().decode()


def metrics():
    m = {}
    for line in get('/metrics').splitlines():
        hit = re.match(r'llamacpp:(prompt_tokens_total|prompt_seconds_total|tokens_predicted_total|tokens_predicted_seconds_total) ([0-9.e+]+)', line)
        if hit:
            m[hit[1]] = float(hit[2])
    return m


def transcribe(wav):
    boundary = uuid.uuid4().hex
    body = b''.join([
        f'--{boundary}\r\nContent-Disposition: form-data; name="model"\r\n\r\nqwen3-asr-1.7b\r\n'.encode(),
        f'--{boundary}\r\nContent-Disposition: form-data; name="response_format"\r\n\r\njson\r\n'.encode(),
        f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="a.wav"\r\nContent-Type: audio/wav\r\n\r\n'.encode(),
        pathlib.Path(wav).read_bytes(), f'\r\n--{boundary}--\r\n'.encode()])
    req = urllib.request.Request(BASE + '/v1/audio/transcriptions', data=body,
                                 headers={'Content-Type': f'multipart/form-data; boundary={boundary}'})
    started = time.perf_counter()
    with urllib.request.urlopen(req, timeout=300) as r:
        text = json.loads(r.read())['text']
    return time.perf_counter() - started, text


results = {}
for label in sys.argv[1:]:
    model = HERE / f'dl/Qwen3-ASR-1.7B-{label}.gguf'
    proc = subprocess.Popen([str(SERVER), '--model', str(model), '--alias', 'qwen3-asr-1.7b', '--host', '127.0.0.1',
                             '--port', str(PORT), '--n-gpu-layers', '999', '--ctx-size', '12288', '--parallel', '1',
                             '--no-webui', '--offline', '--metrics', '--mmproj', str(MMPROJ)],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        while True:
            try:
                if json.loads(get('/health')).get('status') == 'ok':
                    break
            except Exception:
                pass
            if proc.poll() is not None:
                sys.exit(f'{label}: server exited')
            time.sleep(0.5)
        transcribe(EVICT[0])  # warm-up
        out = {}
        for clip in ['yue-30s', 'yue-60s']:
            times, prompt, generated, gen_s, text = [], 0.0, 0.0, 0.0, ''
            for i in range(5):
                transcribe(EVICT[(i + 1) % len(EVICT)])  # a different clip, so the audio is new
                before = metrics()
                seconds, text = transcribe(HERE / f'long/{clip}.wav')
                after = metrics()
                times.append(seconds)
                prompt += after['prompt_tokens_total'] - before['prompt_tokens_total']
                generated += after['tokens_predicted_total'] - before['tokens_predicted_total']
                gen_s += after['tokens_predicted_seconds_total'] - before['tokens_predicted_seconds_total']
            out[clip] = {'median_s': round(statistics.median(times), 3), 'min_s': round(min(times), 3),
                         'max_s': round(max(times), 3), 'prompt_tokens': prompt / 5,
                         'generated_tokens': generated / 5, 'gen_tokens_per_s': round(generated / gen_s, 1),
                         'text': text}
        results[label] = out
        print(label, {c: (v['median_s'], v['prompt_tokens'], v['generated_tokens'], v['gen_tokens_per_s'])
                      for c, v in out.items()}, flush=True)
    finally:
        proc.terminate()
        proc.wait(timeout=30)
(HERE / 'results/speech-long.json').write_text(json.dumps(results, ensure_ascii=False, indent=1))

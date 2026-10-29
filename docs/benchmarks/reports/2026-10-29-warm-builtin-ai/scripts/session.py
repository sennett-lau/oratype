"""Replays a scripted dictation session against one polish model, with the app's llama-server
arguments and the app's real polish prompts (recorded in prompts/), and records what every
request costs: tokens read fresh vs taken from the cache, read and write speed, wall time.

    python3 -I session.py <model.gguf> <label> [slots]     slots: 1 (one slot) or 2 (one per language)

Phases, on one server:
  cold      the first dictation after the server starts (after the app's short setup test)
  yue-short 6 short Cantonese dictations in a row
  yue-long  4 long Cantonese dictations (about 1 minute of speech each)
  switch    en, yue, en, yue, en long, yue long
  session   30 dictations, 75% Cantonese / 25% English, 40% long (fixed seed)
Then a second server start with a warm-up request (the Cantonese prompt, 1 token) before the
first dictation.
"""
import json, os, pathlib, random, re, statistics, subprocess, sys, time, urllib.request

# Recorded prompts (prompts/) and results/ live here.
HERE = pathlib.Path(os.environ.get('BENCH_DIR', '.')).resolve()
REPO = pathlib.Path(__file__).resolve().parents[5]
SERVER = str(REPO / 'src-tauri/target/llama.cpp/build-b11177/bin/llama-server')
model, label = pathlib.Path(sys.argv[1]), sys.argv[2]
slots = int(sys.argv[3]) if len(sys.argv) > 3 else 1
PORT = 8321
(HERE / 'results').mkdir(exist_ok=True)

bodies = [json.loads(p.read_text()) for p in sorted((HERE / 'prompts').glob('*.json'))]
def lang_of(b): return 'yue' if '香港' in b['messages'][0]['content'] else 'en'
def text_of(b): return re.sub(r'^<transcription>\s*|\s*</transcription>$', '', b['messages'][1]['content'].strip())
system = {}
for lang in ('yue', 'en'):
    prompts = [b['messages'][0]['content'] for b in bodies if lang_of(b) == lang]
    system[lang] = max(set(prompts), key=prompts.count)   # the usual prompt for that language
short = {lang: [text_of(b) for b in bodies if lang_of(b) == lang] for lang in ('yue', 'en')}
rng = random.Random(7)
def long_text(lang):
    return ('，' if lang == 'yue' else '. ').join(rng.sample(short[lang], 8))

def start():
    args = [SERVER, '--model', str(model), '--alias', 'polish', '--host', '127.0.0.1', '--port', str(PORT),
            '--n-gpu-layers', '999', '--ctx-size', str(8192 * slots), '--parallel', str(slots),
            '--reasoning', 'off', '--no-webui', '--offline']
    log = open(HERE / 'results' / f'{label}-s{slots}-server.log', 'a')
    proc = subprocess.Popen(args, stdout=log, stderr=subprocess.STDOUT)
    t = time.perf_counter()
    while True:
        try:
            with urllib.request.urlopen(f'http://127.0.0.1:{PORT}/health', timeout=5) as r:
                if json.loads(r.read()).get('status') == 'ok':
                    return proc, time.perf_counter() - t
        except Exception:
            if proc.poll() is not None:
                sys.exit('server exited')
            time.sleep(0.3)

def ask(lang, text, max_tokens=4096):
    body = {'model': 'polish', 'stream': False, 'temperature': 0.3, 'seed': 1, 'max_tokens': max_tokens,
            'messages': [{'role': 'system', 'content': system[lang]},
                         {'role': 'user', 'content': f'<transcription> {text} </transcription>'}]}
    if slots == 2:
        body['id_slot'] = 0 if lang == 'yue' else 1
    req = urllib.request.Request(f'http://127.0.0.1:{PORT}/v1/chat/completions', data=json.dumps(body).encode(),
                                 headers={'Content-Type': 'application/json'})
    t = time.perf_counter()
    with urllib.request.urlopen(req, timeout=600) as r:
        res = json.loads(r.read())
    wall = time.perf_counter() - t
    tm = res.get('timings', {})
    return {'wall_s': round(wall, 3), 'fresh': tm.get('prompt_n'), 'cached': tm.get('cache_n'),
            'read_s': round(tm.get('prompt_ms', 0) / 1000, 3), 'read_tps': round(tm.get('prompt_per_second', 0)),
            'wrote': tm.get('predicted_n'), 'write_s': round(tm.get('predicted_ms', 0) / 1000, 3),
            'write_tps': round(tm.get('predicted_per_second', 0)), 'chars_in': len(text)}

rows = []
def run(phase, lang, kind):
    text = rng.choice(short[lang]) if kind == 'short' else long_text(lang)
    r = dict(phase=phase, lang=lang, kind=kind, **ask(lang, text))
    rows.append(r)
    print(json.dumps(r, ensure_ascii=False), flush=True)

proc, ready = start()
try:
    ask('en', 'Hello.', max_tokens=8)   # the app's setup test: a short request right after start
    run('cold', 'yue', 'short')
    for _ in range(6): run('yue-short', 'yue', 'short')
    for _ in range(4): run('yue-long', 'yue', 'long')
    for lang, kind in [('en', 'short'), ('yue', 'short'), ('en', 'short'), ('yue', 'short'), ('en', 'long'), ('yue', 'long')]:
        run('switch', lang, kind)
    for _ in range(30):
        run('session', 'yue' if rng.random() < 0.75 else 'en', 'long' if rng.random() < 0.4 else 'short')
finally:
    proc.terminate(); proc.wait(timeout=30)

proc, _ = start()
try:
    t = time.perf_counter(); ask('yue', '。', max_tokens=1); warmup_s = time.perf_counter() - t
    run('after-warm-up', 'yue', 'short')
finally:
    proc.terminate(); proc.wait(timeout=30)

def summary(phase):
    walls = [r['wall_s'] for r in rows if r['phase'] == phase]
    return {'n': len(walls), 'median_s': round(statistics.median(walls), 2), 'max_s': round(max(walls), 2),
            'total_s': round(sum(walls), 1)}
result = {'model': model.name, 'size_bytes': model.stat().st_size, 'slots': slots, 'ready_s': round(ready, 1),
          'warm_up_request_s': round(warmup_s, 2),
          'phases': {p: summary(p) for p in ['cold', 'yue-short', 'yue-long', 'switch', 'session', 'after-warm-up']},
          'full_rereads': sum(1 for r in rows if (r['fresh'] or 0) > 2000), 'rows': rows}
(HERE / 'results' / f'{label}-s{slots}.json').write_text(json.dumps(result, ensure_ascii=False, indent=1))
print(json.dumps({k: v for k, v in result.items() if k != 'rows'}, ensure_ascii=False))

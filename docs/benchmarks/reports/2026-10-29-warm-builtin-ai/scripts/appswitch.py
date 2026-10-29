"""What an app switch costs: the same Cantonese prompt with a different [SEMANTIC_CONTEXT] section
(the focused app's family), after the server has the first version cached.

    python3 -I appswitch.py <model.gguf>
"""
import json, os, pathlib, subprocess, sys, time, urllib.request

HERE = pathlib.Path(os.environ.get('BENCH_DIR', '.')).resolve()
REPO = pathlib.Path(__file__).resolve().parents[5]
SERVER = str(REPO / 'src-tauri/target/llama.cpp/build-b11177/bin/llama-server')
model, PORT = pathlib.Path(sys.argv[1]), 8351
b = json.loads((HERE / 'prompts/000.json').read_text())
sys_a = b['messages'][0]['content']
i = sys_a.index('[SEMANTIC_CONTEXT]') + len('[SEMANTIC_CONTEXT]\n')
sys_b = sys_a[:i] + 'App family: chat. Keep messages short and casual.\n' + sys_a[i:]
proc = subprocess.Popen([SERVER, '--model', str(model), '--alias', 'p', '--host', '127.0.0.1', '--port', str(PORT),
                         '--n-gpu-layers', '999', '--ctx-size', '8192', '--parallel', '1', '--reasoning', 'off',
                         '--no-webui', '--offline'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def ask(system, text):
    body = {'model': 'p', 'temperature': 0.3, 'seed': 1, 'max_tokens': 4096,
            'messages': [{'role': 'system', 'content': system},
                         {'role': 'user', 'content': f'<transcription> {text} </transcription>'}]}
    t = time.perf_counter()
    req = urllib.request.Request(f'http://127.0.0.1:{PORT}/v1/chat/completions', data=json.dumps(body).encode(),
                                 headers={'Content-Type': 'application/json'})
    res = json.loads(urllib.request.urlopen(req, timeout=600).read())
    return round(time.perf_counter() - t, 2), res['timings']['prompt_n'], res['timings'].get('cache_n')

try:
    while True:
        try:
            if json.loads(urllib.request.urlopen(f'http://127.0.0.1:{PORT}/health', timeout=5).read()).get('status') == 'ok':
                break
        except Exception:
            time.sleep(0.3)
    ask(sys_a, '我哋聽日開會。')
    warm = ask(sys_a, '呢個功能下個禮拜先會出。')
    switched = ask(sys_b, '你同客戶講聲。')
    back = ask(sys_a, '好，咁我哋就照咁做啦。')
    print(json.dumps({'model': model.name, 'same_app (s, fresh, cached)': warm, 'other_app': switched,
                      'back_to_first_app': back}))
finally:
    proc.terminate(); proc.wait(timeout=30)

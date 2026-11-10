# Speech evaluation

Model `qwen3-asr-1.7b`, split `all`, 1 sample(s) per case, git `837b349+dirty`, 2026-11-10T07:08:30.396Z.

Overall pass rate: **80%** over 41 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 15 | 87% | 3% |
| yue | 14 | 71% | 6% |
| zh-Hans | 12 | 83% | 5% |

## By tag

| Tag | en | yue | zh-Hans |
|---|---|---|---|
| clean | 100% (6) |  | 83% (6) |
| dialect |  | 80% (10) |  |
| dictionary | 71% (7) | 50% (6) | 75% (4) |
| mixed-language |  | 50% (4) | 80% (5) |
| names | 60% (5) | 25% (4) | 67% (3) |
| numbers | 100% (2) |  | 100% (1) |

## Failed checks (samples)

| Check | en | yue | zh-Hans |
|---|---|---|---|
| error | 1 | 2 | 1 |
| script |  |  | 10 |
| terms | 3 | 5 | 2 |

## Cases that failed at least once (8)

### `sp-en-009` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-009.wav`
- expected: `Please ask Niamh to update OraType before the Tauri release on Friday.`
- got: `Please ask Niamh to update OraType before the Tory release on Friday.`
- why: terms: Tauri

### `sp-en-014` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-014.wav`
- expected: `Can you send the Vercel logs to Siobhan and Niamh?`
- got: `Can you send the Vercel logs to Shavon and me`
- why: error: 20% > 15%; terms: Niamh; terms: Siobhan

### `sp-yue-006` 0% [dialect]

- input: `audio/synthetic/sp-yue-006.wav`
- expected: `佢唔喺屋企，你遲啲再打俾佢啦。`
- got: `佢唔喺屋企，你遲啲再打畀佢喇`
- why: error: 15% > 15%

### `sp-yue-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-009.wav`
- expected: `你幫我同Niamh講，OraType下個禮拜出新版。`
- got: `你幫我同Niam講，OraType下個禮拜出新版`
- why: terms: Niamh

### `sp-yue-010` 0% [names, dialect, dictionary]

- input: `audio/synthetic/sp-yue-010.wav`
- expected: `我哋聽日喺鰂魚涌同葉梓聰開會。`
- got: `我哋聽日喺鰂魚涌同業子涌開會`
- why: error: 29% > 15%; terms: 葉梓聰; terms: 鰂魚涌

### `sp-yue-011` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-011.wav`
- expected: `啲log已經send咗去Grafana，你睇下Kubernetes有冇報錯。`
- got: `D露已經send咗去grafana，你睇下kubernetes有冇報錯？`
- why: terms: Grafana; terms: Kubernetes

### `sp-zh-004` 0% [clean]

- input: `audio/synthetic/sp-zh-004.wav`
- expected: `你能不能帮我看一下这个问题是怎么回事？`
- got: `你能不能幫我看一下呢個問題係點回事`
- why: error: 22% > 15%; script: simplified wanted, got traditional

### `sp-zh-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-009.wav`
- expected: `明天让Niamh把OraType的新版本发出来。`
- got: `明天让Niam把Oratype的新版本發出來。`
- why: script: simplified wanted, got traditional; terms: Niamh; terms: OraType


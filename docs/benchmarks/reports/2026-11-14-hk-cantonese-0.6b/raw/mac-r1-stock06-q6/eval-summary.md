# Speech evaluation

Model `qwen3-asr-1.7b`, split `all`, 1 sample(s) per case, git `1d8cd58+dirty`, 2026-11-14T02:16:08.718Z.

Overall pass rate: **80%** over 41 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 15 | 80% | 3% |
| yue | 14 | 71% | 7% |
| zh-Hans | 12 | 92% | 2% |

## By tag

| Tag | en | yue | zh-Hans |
|---|---|---|---|
| clean | 100% (6) |  | 100% (6) |
| dialect |  | 90% (10) |  |
| dictionary | 57% (7) | 33% (6) | 75% (4) |
| mixed-language |  | 25% (4) | 80% (5) |
| names | 40% (5) | 25% (4) | 67% (3) |
| numbers | 100% (2) |  | 100% (1) |

## Failed checks (samples)

| Check | en | yue | zh-Hans |
|---|---|---|---|
| error | 1 | 3 |  |
| script |  | 14 |  |
| terms | 4 | 5 | 1 |

## Cases that failed at least once (8)

### `sp-en-009` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-009.wav`
- expected: `Please ask Niamh to update OraType before the Tauri release on Friday.`
- got: `Please ask Neve to update OraType before the Tauri release on Friday.`
- why: terms: Niamh

### `sp-en-010` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-010.wav`
- expected: `Saoirse moved the vectors from Pinecone to Qdrant last week.`
- got: `Sia moved the vectors from Pinecone to Qdrant last week.`
- why: terms: Saoirse

### `sp-en-014` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-014.wav`
- expected: `Can you send the Vercel logs to Siobhan and Niamh?`
- got: `Can you send the Vercel logs to Chevorn and leave?`
- why: error: 20% > 15%; terms: Niamh; terms: Siobhan

### `sp-yue-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-009.wav`
- expected: `你幫我同Niamh講，OraType下個禮拜出新版。`
- got: `你帮我同Niam讲，Orachi下个礼拜出新版。`
- why: error: 21% > 15%; script: traditional wanted, got simplified; terms: Niamh; terms: OraType

### `sp-yue-010` 0% [names, dialect, dictionary]

- input: `audio/synthetic/sp-yue-010.wav`
- expected: `我哋聽日喺鰂魚涌同葉梓聰開會。`
- got: `我哋听日喺鲗鱼涌同叶子涌开会。`
- why: script: traditional wanted, got simplified; terms: 葉梓聰

### `sp-yue-011` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-011.wav`
- expected: `啲log已經send咗去Grafana，你睇下Kubernetes有冇報錯。`
- got: `啲录已经send咗去grafana，你睇下qbnets有冇报错。`
- why: error: 22% > 15%; script: traditional wanted, got simplified; terms: Grafana; terms: Kubernetes

### `sp-yue-012` 0% [mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-012.wav`
- expected: `你寫低個名或者type個名落去都得。`
- got: `你写低个名，或者摘个名落去都得。`
- why: error: 24% > 15%; script: traditional wanted, got simplified

### `sp-zh-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-009.wav`
- expected: `明天让Niamh把OraType的新版本发出来。`
- got: `明天让Niam把OraType的新版本发出来。`
- why: terms: Niamh


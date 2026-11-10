# Speech evaluation

Model `qwen3-asr-1.7b`, split `all`, 1 sample(s) per case, git `837b349+dirty`, 2026-11-10T07:03:39.383Z.

Overall pass rate: **76%** over 41 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 15 | 67% | 17% |
| yue | 14 | 79% | 3% |
| zh-Hans | 12 | 83% | 3% |

## By tag

| Tag | en | yue | zh-Hans |
|---|---|---|---|
| clean | 100% (6) |  | 100% (6) |
| dialect |  | 90% (10) |  |
| dictionary | 29% (7) | 50% (6) | 50% (4) |
| mixed-language |  | 50% (4) | 60% (5) |
| names | 20% (5) | 25% (4) | 33% (3) |
| numbers | 100% (2) |  | 100% (1) |

## Failed checks (samples)

| Check | en | yue | zh-Hans |
|---|---|---|---|
| error | 5 | 1 |  |
| script |  |  | 9 |
| terms | 4 | 5 | 2 |

## Cases that failed at least once (10)

### `sp-en-009` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-009.wav`
- expected: `Please ask Niamh to update OraType before the Tauri release on Friday.`
- got: `請ask Niamh去update OraType before the Tory release on Friday.`
- why: error: 50% > 15%; terms: Tauri

### `sp-en-010` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-010.wav`
- expected: `Saoirse moved the vectors from Pinecone to Qdrant last week.`
- got: `Saoirse 動了 vector from Pinecone to Qdrant 上個星期`
- why: error: 50% > 15%

### `sp-en-011` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-011.wav`
- expected: `Tadhg said the Supabase migration finished without errors.`
- got: `Tayk 說：「The Supabase migration finished without errors.`
- why: error: 25% > 15%; terms: Tadhg

### `sp-en-013` 0% [dictionary]

- input: `audio/synthetic/sp-en-013.wav`
- expected: `Please sign here or type your name in the box below.`
- got: `請簽名或在下方的框中輸入你的名字`
- why: error: 100% > 15%

### `sp-en-014` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-014.wav`
- expected: `Can you send the Vercel logs to Siobhan and Niamh?`
- got: `Can you send the Vercel logs to Chavorn and Nive`
- why: error: 20% > 15%; terms: Niamh; terms: Siobhan

### `sp-yue-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-009.wav`
- expected: `你幫我同Niamh講，OraType下個禮拜出新版。`
- got: `你幫我同Niam講 OraType下個禮拜出新版`
- why: terms: Niamh

### `sp-yue-010` 0% [names, dialect, dictionary]

- input: `audio/synthetic/sp-yue-010.wav`
- expected: `我哋聽日喺鰂魚涌同葉梓聰開會。`
- got: `我哋聽日喺鰂魚涌同葉子涌開會`
- why: error: 21% > 15%; terms: 葉梓聰; terms: 鰂魚涌

### `sp-yue-011` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-011.wav`
- expected: `啲log已經send咗去Grafana，你睇下Kubernetes有冇報錯。`
- got: `啲log已經send咗去grafana，你睇下kubernetes有冇報錯`
- why: terms: Grafana; terms: Kubernetes

### `sp-zh-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-009.wav`
- expected: `明天让Niamh把OraType的新版本发出来。`
- got: `明天让Niam把OraType的新版本发出来。`
- why: terms: Niamh

### `sp-zh-010` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-010.wav`
- expected: `我们把向量数据从Pinecone搬到了Qdrant。`
- got: `我們把向量數據從pinecone搬到了Qdrant。`
- why: script: simplified wanted, got traditional; terms: Pinecone


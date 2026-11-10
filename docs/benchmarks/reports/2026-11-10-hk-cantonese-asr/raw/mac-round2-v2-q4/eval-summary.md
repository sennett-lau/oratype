# Speech evaluation

Model `qwen3-asr-1.7b`, split `all`, 1 sample(s) per case, git `c30ec4c+dirty`, 2026-11-10T07:32:39.843Z.

Overall pass rate: **80%** over 41 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 15 | 67% | 23% |
| yue | 14 | 86% | 5% |
| zh-Hans | 12 | 92% | 3% |

## By tag

| Tag | en | yue | zh-Hans |
|---|---|---|---|
| clean | 100% (6) |  | 100% (6) |
| dialect |  | 90% (10) |  |
| dictionary | 29% (7) | 67% (6) | 75% (4) |
| mixed-language |  | 75% (4) | 80% (5) |
| names | 20% (5) | 50% (4) | 67% (3) |
| numbers | 100% (2) |  | 100% (1) |

## Failed checks (samples)

| Check | en | yue | zh-Hans |
|---|---|---|---|
| error | 4 | 1 |  |
| script |  |  | 9 |
| terms | 6 | 2 | 1 |

## Cases that failed at least once (8)

### `sp-en-009` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-009.wav`
- expected: `Please ask Niamh to update OraType before the Tauri release on Friday.`
- got: `請問Niamh，要喺Tauri上架前，請ask Neve更新Qdrant。`
- why: error: 100% > 15%; terms: OraType

### `sp-en-010` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-010.wav`
- expected: `Saoirse moved the vectors from Pinecone to Qdrant last week.`
- got: ``
- why: error: 100% > 15%; terms: Saoirse; terms: Qdrant

### `sp-en-011` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-011.wav`
- expected: `Tadhg said the Supabase migration finished without errors.`
- got: `Tych said the Supabase migration finished without errors`
- why: terms: Tadhg

### `sp-en-013` 0% [dictionary]

- input: `audio/synthetic/sp-en-013.wav`
- expected: `Please sign here or type your name in the box below.`
- got: `請簽名或者打你名字在下面的框裡`
- why: error: 100% > 15%

### `sp-en-014` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-014.wav`
- expected: `Can you send the Vercel logs to Siobhan and Niamh?`
- got: `Can you send the Vercel logs to Chavorn and Neve?`
- why: error: 20% > 15%; terms: Niamh; terms: Siobhan

### `sp-yue-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-009.wav`
- expected: `你幫我同Niamh講，OraType下個禮拜出新版。`
- got: `你幫我同Niam講OraType下個禮拜出新版`
- why: terms: Niamh

### `sp-yue-010` 0% [names, dialect, dictionary]

- input: `audio/synthetic/sp-yue-010.wav`
- expected: `我哋聽日喺鰂魚涌同葉梓聰開會。`
- got: `我哋聽日喺鰂魚涌同業子涌開會`
- why: error: 21% > 15%; terms: 葉梓聰

### `sp-zh-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-009.wav`
- expected: `明天让Niamh把OraType的新版本发出来。`
- got: `明天让Niam把OraType的新版本发出来。`
- why: terms: Niamh


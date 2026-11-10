# Speech evaluation

Model `qwen3-asr-1.7b`, split `all`, 1 sample(s) per case, git `c30ec4c+dirty`, 2026-11-10T07:33:28.820Z.

Overall pass rate: **78%** over 41 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 15 | 80% | 4% |
| yue | 14 | 71% | 4% |
| zh-Hans | 12 | 83% | 6% |

## By tag

| Tag | en | yue | zh-Hans |
|---|---|---|---|
| clean | 100% (6) |  | 83% (6) |
| dialect |  | 80% (10) |  |
| dictionary | 57% (7) | 50% (6) | 75% (4) |
| mixed-language |  | 50% (4) | 80% (5) |
| names | 40% (5) | 25% (4) | 67% (3) |
| numbers | 100% (2) |  | 100% (1) |

## Failed checks (samples)

| Check | en | yue | zh-Hans |
|---|---|---|---|
| error | 2 | 1 | 1 |
| script |  |  | 10 |
| terms | 5 | 4 | 1 |

## Cases that failed at least once (9)

### `sp-en-009` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-009.wav`
- expected: `Please ask Niamh to update OraType before the Tauri release on Friday.`
- got: `Please ask Neve to update OraType before the Tory release on Friday`
- why: error: 17% > 15%; terms: Niamh; terms: Tauri

### `sp-en-011` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-011.wav`
- expected: `Tadhg said the Supabase migration finished without errors.`
- got: `Tyge said the Supabase migration finished without errors.`
- why: terms: Tadhg

### `sp-en-014` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-014.wav`
- expected: `Can you send the Vercel logs to Siobhan and Niamh?`
- got: `Can you send the Vercel logs to Shavon and Nive?`
- why: error: 20% > 15%; terms: Niamh; terms: Siobhan

### `sp-yue-003` 0% [dialect]

- input: `audio/synthetic/sp-yue-003.wav`
- expected: `你睇下呢個問題係咪咁樣。`
- got: `你睇吓呢個問題係咪噉樣`
- why: error: 18% > 15%

### `sp-yue-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-009.wav`
- expected: `你幫我同Niamh講，OraType下個禮拜出新版。`
- got: `你幫我同Niam講OraType下個禮拜出新版`
- why: terms: Niamh

### `sp-yue-010` 0% [names, dialect, dictionary]

- input: `audio/synthetic/sp-yue-010.wav`
- expected: `我哋聽日喺鰂魚涌同葉梓聰開會。`
- got: `我哋聽日喺鰂魚涌同葉子涌開會`
- why: terms: 葉梓聰

### `sp-yue-011` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-011.wav`
- expected: `啲log已經send咗去Grafana，你睇下Kubernetes有冇報錯。`
- got: `啲log已經sent咗去grafana，你睇下kubernetes有冇報錯。`
- why: terms: Grafana; terms: Kubernetes

### `sp-zh-001` 0% [clean]

- input: `audio/synthetic/sp-zh-001.wav`
- expected: `我觉得这个方案还不错，就是价格有点贵。`
- got: `我覺得呢個方案還不錯，只係價格有點貴。`
- why: error: 18% > 15%; script: simplified wanted, got traditional

### `sp-zh-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-009.wav`
- expected: `明天让Niamh把OraType的新版本发出来。`
- got: `明天让Naem把OraType的新版本发出来。`
- why: terms: Niamh


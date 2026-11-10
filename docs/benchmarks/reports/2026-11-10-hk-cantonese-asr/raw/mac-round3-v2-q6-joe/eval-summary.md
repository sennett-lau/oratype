# Speech evaluation

Model `qwen3-asr-1.7b`, split `all`, 1 sample(s) per case, git `c30ec4c+dirty`, 2026-11-10T07:59:37.082Z.

Overall pass rate: **83%** over 41 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 15 | 87% | 4% |
| yue | 14 | 79% | 3% |
| zh-Hans | 12 | 83% | 7% |

## By tag

| Tag | en | yue | zh-Hans |
|---|---|---|---|
| clean | 100% (6) |  | 100% (6) |
| dialect |  | 90% (10) |  |
| dictionary | 71% (7) | 50% (6) | 50% (4) |
| mixed-language |  | 50% (4) | 80% (5) |
| names | 60% (5) | 25% (4) | 67% (3) |
| numbers | 100% (2) |  | 100% (1) |

## Failed checks (samples)

| Check | en | yue | zh-Hans |
|---|---|---|---|
| error | 2 |  | 2 |
| script |  |  | 8 |
| terms | 5 | 4 | 2 |

## Cases that failed at least once (7)

### `sp-en-009` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-009.wav`
- expected: `Please ask Niamh to update OraType before the Tauri release on Friday.`
- got: `Please ask Neve to update AuraType before the Tory release on Friday.`
- why: error: 25% > 15%; terms: Niamh; terms: OraType; terms: Tauri

### `sp-en-014` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-014.wav`
- expected: `Can you send the Vercel logs to Siobhan and Niamh?`
- got: `Can you send the Vercel logs to Chavorn and me?`
- why: error: 20% > 15%; terms: Niamh; terms: Siobhan

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
- got: `啲log已經send咗去grafana，你睇下kubernetes有冇報錯。`
- why: terms: Grafana; terms: Kubernetes

### `sp-zh-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-009.wav`
- expected: `明天让Niamh把OraType的新版本发出来。`
- got: `明天让Naomi把Orator的新版本发出来。`
- why: error: 26% > 15%; terms: Niamh; terms: OraType

### `sp-zh-011` 0% [dictionary]

- input: `audio/synthetic/sp-zh-011.wav`
- expected: `请在这里签名或者输入你的名字。`
- got: `請喺呢度簽名或者輸入你嘅名字`
- why: error: 29% > 15%; script: simplified wanted, got traditional


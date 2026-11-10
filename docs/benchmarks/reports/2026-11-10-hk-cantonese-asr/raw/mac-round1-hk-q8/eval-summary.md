# Speech evaluation

Model `qwen3-asr-1.7b`, split `all`, 1 sample(s) per case, git `837b349+dirty`, 2026-11-10T07:04:42.370Z.

Overall pass rate: **78%** over 41 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 15 | 80% | 4% |
| yue | 14 | 71% | 4% |
| zh-Hans | 12 | 83% | 0% |

## By tag

| Tag | en | yue | zh-Hans |
|---|---|---|---|
| clean | 100% (6) |  | 100% (6) |
| dialect |  | 90% (10) |  |
| dictionary | 57% (7) | 33% (6) | 50% (4) |
| mixed-language |  | 25% (4) | 60% (5) |
| names | 40% (5) | 0% (4) | 33% (3) |
| numbers | 100% (2) |  | 100% (1) |

## Failed checks (samples)

| Check | en | yue | zh-Hans |
|---|---|---|---|
| error | 2 | 1 |  |
| script |  |  | 9 |
| terms | 5 | 6 | 2 |

## Cases that failed at least once (9)

### `sp-en-009` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-009.wav`
- expected: `Please ask Niamh to update OraType before the Tauri release on Friday.`
- got: `Please ask Neve to update OraType before the Tory release on Friday.`
- why: error: 17% > 15%; terms: Niamh; terms: Tauri

### `sp-en-011` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-011.wav`
- expected: `Tadhg said the Supabase migration finished without errors.`
- got: `Tadg said the Supabase migration finished without errors.`
- why: terms: Tadhg

### `sp-en-014` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-014.wav`
- expected: `Can you send the Vercel logs to Siobhan and Niamh?`
- got: `Can you send the Vercel logs to Shavorn and Neve`
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
- why: error: 21% > 15%; terms: 葉梓聰; terms: 鰂魚涌

### `sp-yue-011` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-011.wav`
- expected: `啲log已經send咗去Grafana，你睇下Kubernetes有冇報錯。`
- got: `啲log已經send咗去grafana，你睇下kubernetes有冇報錯`
- why: terms: Grafana; terms: Kubernetes

### `sp-yue-014` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-014.wav`
- expected: `陳凱澄話Supabase嗰邊已經搞掂。`
- got: `陳凱澄話supabase嗰邊已經搞掂`
- why: terms: Supabase

### `sp-zh-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-009.wav`
- expected: `明天让Niamh把OraType的新版本发出来。`
- got: `明天讓Niam把OraType的新版本發出來。`
- why: script: simplified wanted, got traditional; terms: Niamh

### `sp-zh-012` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-012.wav`
- expected: `王思琪说Supabase的迁移已经完成了。`
- got: `王思琪：說supabase的遷移已經完成了。`
- why: script: simplified wanted, got traditional; terms: Supabase


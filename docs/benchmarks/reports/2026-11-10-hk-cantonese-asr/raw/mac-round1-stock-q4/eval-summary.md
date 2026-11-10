# Speech evaluation

Model `qwen3-asr-1.7b`, split `all`, 1 sample(s) per case, git `837b349+dirty`, 2026-11-10T07:05:55.691Z.

Overall pass rate: **83%** over 41 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 15 | 87% | 5% |
| yue | 14 | 71% | 6% |
| zh-Hans | 12 | 92% | 1% |

## By tag

| Tag | en | yue | zh-Hans |
|---|---|---|---|
| clean | 100% (6) |  | 100% (6) |
| dialect |  | 90% (10) |  |
| dictionary | 71% (7) | 33% (6) | 75% (4) |
| mixed-language |  | 25% (4) | 80% (5) |
| names | 60% (5) | 25% (4) | 67% (3) |
| numbers | 100% (2) |  | 100% (1) |

## Failed checks (samples)

| Check | en | yue | zh-Hans |
|---|---|---|---|
| error | 2 | 1 |  |
| script |  | 13 |  |
| terms | 5 | 3 | 1 |

## Cases that failed at least once (7)

### `sp-en-009` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-009.wav`
- expected: `Please ask Niamh to update OraType before the Tauri release on Friday.`
- got: `Please ask Neve to update AuraType before the Tory release on Friday.`
- why: error: 25% > 15%; terms: Niamh; terms: OraType; terms: Tauri

### `sp-en-014` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-014.wav`
- expected: `Can you send the Vercel logs to Siobhan and Niamh?`
- got: `Can you send the Vercel logs to Shavon and Nive?`
- why: error: 20% > 15%; terms: Niamh; terms: Siobhan

### `sp-yue-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-009.wav`
- expected: `你幫我同Niamh講，OraType下個禮拜出新版。`
- got: `你帮我同Niam讲，OraType下个礼拜出新版。`
- why: script: traditional wanted, got simplified; terms: Niamh

### `sp-yue-010` 0% [names, dialect, dictionary]

- input: `audio/synthetic/sp-yue-010.wav`
- expected: `我哋聽日喺鰂魚涌同葉梓聰開會。`
- got: `我哋听日喺鲗鱼涌同叶子涌开会。`
- why: script: traditional wanted, got simplified; terms: 葉梓聰

### `sp-yue-012` 0% [mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-012.wav`
- expected: `你寫低個名或者type個名落去都得。`
- got: `你写低个名或者join个名落去都得。`
- why: error: 24% > 15%; script: traditional wanted, got simplified

### `sp-yue-014` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-014.wav`
- expected: `陳凱澄話Supabase嗰邊已經搞掂。`
- got: `陳海晴話：，Supabase嗰邊已經搞掂。`
- why: terms: 陳凱澄

### `sp-zh-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-009.wav`
- expected: `明天让Niamh把OraType的新版本发出来。`
- got: `明天让Niam把OraType的新版本发出来。`
- why: terms: Niamh


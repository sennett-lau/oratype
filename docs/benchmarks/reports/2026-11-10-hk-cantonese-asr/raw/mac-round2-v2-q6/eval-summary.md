# Speech evaluation

Model `qwen3-asr-1.7b`, split `all`, 1 sample(s) per case, git `c30ec4c+dirty`, 2026-11-10T07:34:39.261Z.

Overall pass rate: **83%** over 41 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 15 | 80% | 4% |
| yue | 14 | 86% | 6% |
| zh-Hans | 12 | 83% | 4% |

## By tag

| Tag | en | yue | zh-Hans |
|---|---|---|---|
| clean | 100% (6) |  | 100% (6) |
| dialect |  | 90% (10) |  |
| dictionary | 57% (7) | 67% (6) | 50% (4) |
| mixed-language |  | 75% (4) | 60% (5) |
| names | 40% (5) | 50% (4) | 33% (3) |
| numbers | 100% (2) |  | 100% (1) |

## Failed checks (samples)

| Check | en | yue | zh-Hans |
|---|---|---|---|
| error | 2 | 1 |  |
| script |  |  | 8 |
| terms | 5 | 3 | 2 |

## Cases that failed at least once (7)

### `sp-en-009` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-009.wav`
- expected: `Please ask Niamh to update OraType before the Tauri release on Friday.`
- got: `Please ask Neve to update OraType before the Tori release on Friday.`
- why: error: 17% > 15%; terms: Niamh; terms: Tauri

### `sp-en-010` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-010.wav`
- expected: `Saoirse moved the vectors from Pinecone to Qdrant last week.`
- got: `Seascha moved the vectors from Pinecone to Qdrant last week.`
- why: terms: Saoirse

### `sp-en-014` 0% [names, dictionary]

- input: `audio/synthetic/sp-en-014.wav`
- expected: `Can you send the Vercel logs to Siobhan and Niamh?`
- got: `Can you send the Vercel logs to Shavon and Neeve?`
- why: error: 20% > 15%; terms: Niamh; terms: Siobhan

### `sp-yue-010` 0% [names, dialect, dictionary]

- input: `audio/synthetic/sp-yue-010.wav`
- expected: `我哋聽日喺鰂魚涌同葉梓聰開會。`
- got: `我哋聽日喺鰂魚涌同業址涌開會`
- why: error: 21% > 15%; terms: 葉梓聰

### `sp-yue-011` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-yue-011.wav`
- expected: `啲log已經send咗去Grafana，你睇下Kubernetes有冇報錯。`
- got: `啲log已經send咗去grafana，你睇下kubelet有冇報錯`
- why: terms: Grafana; terms: Kubernetes

### `sp-zh-009` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-009.wav`
- expected: `明天让Niamh把OraType的新版本发出来。`
- got: `明天让Na'omi把OraType的新版本发出来。`
- why: terms: Niamh

### `sp-zh-012` 0% [names, mixed-language, dictionary]

- input: `audio/synthetic/sp-zh-012.wav`
- expected: `王思琪说Supabase的迁移已经完成了。`
- got: `王思琪：Superbase 的遷移已經完成了`
- why: script: simplified wanted, got traditional; terms: Supabase


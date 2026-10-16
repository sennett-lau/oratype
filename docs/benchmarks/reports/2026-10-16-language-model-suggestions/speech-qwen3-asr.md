# Speech evaluation

Model `qwen3-asr-1.7b`, split `all`, 1 sample(s) per case, git `03ef9c0+dirty`, 2026-10-16T10:16:27.328Z.

Overall pass rate: **100%** over 24 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 8 | 100% | 3% |
| yue | 8 | 100% | 2% |
| zh-Hans | 8 | 100% | 2% |

## By tag

| Tag | en | yue | zh-Hans |
|---|---|---|---|
| clean | 100% (6) |  | 100% (6) |
| dialect |  | 100% (8) |  |
| mixed-language |  |  | 100% (2) |
| names | 100% (1) |  |  |
| numbers | 100% (2) |  | 100% (1) |

## Failed checks (samples)

| Check | en | yue | zh-Hans |
|---|---|---|---|
| script |  | 8 |  |

## Against baseline `ggml-large-v3-turbo-q5_0.bin`

| Language | Compared | Before | After | Change |
|---|---|---|---|---|
| en | 8 | 100% | 100% | 0 |
| yue | 8 | 0% | 100% | +100 |
| zh-Hans | 8 | 100% | 100% | 0 |

### Regressions (0)


### Improvements (8)

- `sp-yue-001` 0% → 100%
- `sp-yue-002` 0% → 100%
- `sp-yue-003` 0% → 100%
- `sp-yue-004` 0% → 100%
- `sp-yue-005` 0% → 100%
- `sp-yue-006` 0% → 100%
- `sp-yue-007` 0% → 100%
- `sp-yue-008` 0% → 100%

## Cases that failed at least once (0)


# Polish evaluation

Model `qwen3.5-4b-abliterated`, split `all`, 3 sample(s) per case, git `fad8fc0+dirty`, 2026-11-03T01:46:11.035Z.

Overall pass rate: **87%** over 54 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 15 | 100% | 0% |
| es | 6 | 89% | 2% |
| fr | 6 | 100% | 4% |
| ja | 6 | 83% | 2% |
| yue | 12 | 89% | 5% |
| zh-Hans | 9 | 56% | 15% |

## By tag

| Tag | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| dialect |  |  |  |  | 89% (12) |  |
| filler | 100% (2) | 67% (1) | 100% (1) | 0% (1) | 0% (1) | 0% (1) |
| mixed-language |  |  |  | 100% (1) | 100% (6) | 100% (2) |
| no-answer | 100% (15) | 89% (6) | 100% (6) | 83% (6) | 89% (12) | 56% (9) |

## Failed checks (samples)

| Check | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| dialect |  |  |  |  | 2 |  |
| error |  |  |  |  | 2 | 8 |
| filler |  | 1 |  | 6 |  |  |
| must_contain |  | 1 |  |  | 4 | 15 |
| must_not_contain |  | 1 |  | 3 |  |  |
| script |  |  |  |  |  | 12 |

## Cases that failed at least once (9)

### `es-011` 67% [no-answer, filler]

- input: `eh cómo puedo cambiar la contraseña del wifi`
- expected: `¿Cómo puedo cambiar la contraseña del wifi?`
- got: `Eh, cómo puedo cambiar la contraseña del wifi?`
- why: must_not_contain: eh; filler: eh

### `es-012` 67% [no-answer]

- input: `explícame qué es una base de datos`
- expected: `Explícame qué es una base de datos`
- got: `Explica qué es una base de datos`
- why: must_contain: explícame

### `ja-009` 0% [no-answer, filler]

- input: `えーと東京から大阪まで新幹線で何時間かかりますか`
- expected: `東京から大阪まで新幹線で何時間かかりますか？`
- got: `えーと、東京から大阪まで新幹線で何時間かかりますか？`
- why: must_not_contain: えーと; filler: えーと; filler: えー

### `yue-041` 0% [no-answer, filler, dialect]

- input: `嗯香港最高嘅山係邊座`
- expected: `香港最高嘅山係邊座？`
- got: `香港最高嘅山係哪一座？`
- why: must_contain: 邊座

### `yue-044` 67% [no-answer, dialect]

- input: `幫我諗三個團隊食飯嘅地方`
- expected: `幫我諗三個團隊食飯嘅地方`
- got: `幫我諱三個團隊食飯嘅地方`
- why: must_contain: 幫我諗

### `zh-029` 0% [no-answer]

- input: `怎么样才能让这个页面加载得快一点`
- expected: `怎么样才能让这个页面加载得快一点？`
- got: `怎么样才能让这个頁面載得更快一點？`
- why: must_contain: 加载; script: simplified wanted, found 頁載點; error: 31% > 25%

### `zh-031` 0% [no-answer, filler]

- input: `嗯中国最长的河是哪一条`
- expected: `中国最长的河是哪一条？`
- got: `中國最長的河是哪一條？`
- why: must_contain: 最长; must_contain: 哪一条; script: simplified wanted, found 國長條; error: 30% > 25%

### `zh-h11` 0% [no-answer]

- input: `北京明天会不会下雨`
- expected: `北京明天会不会下雨？`
- got: `北京明天會不會下雨？`
- why: script: simplified wanted, found 會

### `zh-h13` 0% [no-answer]

- input: `总结一下这篇文章的主要观点`
- expected: `总结一下这篇文章的主要观点`
- got: `請總結這篇文章的主要觀點`
- why: must_contain: 总结; must_contain: 观点; script: simplified wanted, found 請結這觀點; error: 54% > 25%


# Polish evaluation

Model `qwen3.5-4b`, split `all`, 3 sample(s) per case, git `eb99ead+dirty`, 2026-10-19T20:08:04.452Z.

Overall pass rate: **76%** over 52 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| yue | 52 | 76% | 12% |

## By tag

| Tag | yue |
|---|---|
| dialect | 88% (35) |
| filler | 96% (8) |
| keep-meaning | 94% (18) |
| list | 100% (4) |
| mixed-language | 69% (14) |
| newline | 67% (5) |
| numbers | 33% (5) |
| punctuation | 74% (19) |
| script | 33% (2) |
| self-correction | 0% (5) |

## Failed checks (samples)

| Check | yue |
|---|---|
| error | 21 |
| must_contain | 14 |
| must_not_contain | 17 |
| numbers | 6 |
| punctuation | 5 |
| script | 4 |

## Cases that failed at least once (15)

### `yue-003` 67% [filler, dialect, punctuation]

- input: `嗯我想問下你得唔得閒星期五食飯`
- expected: `我想問下你得唔得閒星期五食飯？`
- got: `我想問下你星期五得唔得閒食飯？`
- why: error: 43% > 25%

### `yue-004` 0% [self-correction, dialect]

- input: `三點鐘唔係唔好意思四點鐘喺會議室等`
- expected: `四點鐘喺會議室等`
- got: `三點鐘唔係，不好意思，四點鐘喺會議室等`
- why: must_not_contain: 三點; error: 113% > 25%

### `yue-011` 0% [self-correction, numbers]

- input: `個價錢係九十九蚊唔係唔係八十九蚊`
- expected: `個價錢係八十九蚊`
- got: `個價錢係九十九蚊，唔係八十九蚊`
- why: must_not_contain: 九十九; error: 75% > 25%

### `yue-012` 0% [numbers, punctuation]

- input: `今個月營業額升咗百分之十五總共係三點二億`
- expected: `今個月營業額升咗15%，總共係3.2億`
- got: `今個月營業額升咗百分之十五，總共係三點二億`
- why: must_contain: 3.2; numbers: want [3.2] got []

### `yue-013` 67% [numbers, mixed-language]

- input: `將個version升去二點一然後再試多次`
- expected: `將個version升去2.1，然後再試多次`
- got: `將版本升去 2.1，然後再試多次`
- why: must_contain: version; error: 42% > 25%

### `yue-018` 0% [newline, punctuation, mixed-language]

- input: `啱啱同老闆傾完佢話OK喎另外聽日記得帶埋部laptop`
- expected: `啱啱同老闆傾完，佢話OK喎。⏎⏎另外，聽日記得帶埋部laptop`
- got: `啱啱同老闆傾完，佢話 OK 喎，另外聽日記得帶埋部 laptop`
- why: punctuation: want 1 sentence breaks, got 0

### `yue-019` 0% [self-correction, mixed-language]

- input: `我哋約咗Peter唔係係Paul喺中環食lunch`
- expected: `我哋約咗Paul喺中環食lunch`
- got: `我哋約咗Peter，唔係Paul，喺中環食lunch`
- why: must_not_contain: Peter; error: 41% > 25%

### `yue-025` 67% [script, dialect, punctuation]

- input: `我觉得呢个方案几好但系个价钱贵咗啲`
- expected: `我觉得呢个方案几好，但系个价钱贵咗啲`
- got: `我覺得呢個方案幾好，但係個價錢貴咗啲`
- why: must_contain: 几好; must_contain: 贵咗啲; script: simplified wanted, found 覺個幾價錢貴; error: 47% > 25%

### `yue-030` 0% [self-correction, mixed-language]

- input: `我想book兩個位唔係三個位七點半`
- expected: `我想book三個位，七點半`
- got: `我想book兩個位，唔係三個位，七點半`
- why: must_not_contain: 兩個; error: 42% > 25%

### `yue-035` 0% [keep-meaning, dialect]

- input: `你係唔係聽日先返嚟呀如果係我哋就改期啦`
- expected: `你係唔係聽日先返嚟呀？如果係，我哋就改期啦`
- got: `你係咪聽日先返嚟呀？如果係，我哋就改期啦。`
- why: must_contain: 係唔係聽日先返嚟

### `yue-h02` 0% [self-correction, mixed-language]

- input: `send俾John唔係唔好意思send俾Amy`
- expected: `send俾Amy`
- got: `send 俾 John，唔好意思，send 俾 Amy`
- why: must_not_contain: John; error: 163% > 25%

### `yue-h04` 0% [numbers, punctuation]

- input: `部機嘅記憶體用咗百分之八十五點五要搵人睇下`
- expected: `部機嘅記憶體用咗85.5%，要搵人睇下`
- got: `部機嘅記憶體用咗百分之八十五點五，要搵人睇下`
- why: must_contain: 85.5; numbers: want [85.5] got []; error: 50% > 25%

### `yue-h08` 33% [dialect, punctuation]

- input: `呢度啲嘢好貴我哋去第二度食啦`
- expected: `呢度啲嘢好貴，我哋去第二度食啦`
- got: `呢度嘢好貴，我哋去第二度食啦`
- why: must_contain: 啲嘢

### `yue-h09` 33% [newline, punctuation]

- input: `多謝你幫手個網站而家快咗好多另外下個禮拜我放假`
- expected: `多謝你幫手，個網站而家快咗好多。⏎⏎另外，下個禮拜我放假`
- got: `多謝你幫手個網站，而家快咗好多，另外下個禮拜我放假`
- why: punctuation: want 1 sentence breaks, got 0

### `yue-h10` 0% [script, dialect]

- input: `个server又死咗我哋要重新开过`
- expected: `个server又死咗，我哋要重新开过`
- got: `個server又死咗，我哋要重新開過`
- why: script: simplified wanted, found 個開過


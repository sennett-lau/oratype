# Polish evaluation

Model `qwen3.5-4b`, split `dev`, 2 sample(s) per case, git `eb99ead+dirty`, 2026-10-19T20:09:22.863Z.

Overall pass rate: **84%** over 60 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 32 | 89% | 2% |
| zh-Hans | 28 | 79% | 8% |

## By tag

| Tag | en | zh-Hans |
|---|---|---|
| dash | 100% (1) |  |
| dialect |  | 67% (3) |
| filler | 88% (8) | 88% (8) |
| keep-meaning | 100% (10) | 60% (5) |
| list | 83% (3) | 100% (2) |
| mixed-language |  | 50% (4) |
| newline | 83% (3) | 100% (3) |
| numbers | 75% (8) | 75% (4) |
| punctuation | 100% (8) | 82% (11) |
| script |  | 100% (1) |
| self-correction | 100% (4) | 67% (3) |

## Failed checks (samples)

| Check | en | zh-Hans |
|---|---|---|
| error |  | 8 |
| layout | 1 |  |
| list | 1 |  |
| must_contain | 2 | 6 |
| must_not_contain | 2 | 4 |
| numbers | 4 | 2 |
| script |  | 4 |

## Cases that failed at least once (10)

### `en-005` 0% [filler]

- input: `I checked the logs you know and there was nothing there`
- expected: `I checked the logs and there was nothing there`
- got: `I checked the logs, you know, and there was nothing there`
- why: must_not_contain: you know

### `en-012` 0% [numbers]

- input: `we grew about twenty percent last quarter and revenue hit three point two million dollars`
- expected: `We grew about 20% last quarter and revenue hit $3.2 million`
- got: `We grew about twenty percent last quarter, and revenue hit 3.2 million dollars`
- why: numbers: want [20 3.2] got [3.2]

### `en-013` 0% [numbers]

- input: `please upgrade node to version twenty two point one one and then rerun the tests`
- expected: `Please upgrade Node to version 22.11 and then rerun the tests`
- got: `Please upgrade Node to version 22.1.1 and then rerun the tests`
- why: must_contain: 22.11; numbers: want [22.11] got [22.1.1]

### `en-030` 50% [list, newline]

- input: `steps to reproduce open the app click settings then toggle dark mode and finally restart`
- expected: `Steps to reproduce:⏎1. Open the app⏎2. Click Settings⏎3. Toggle dark mode⏎4. Restart`
- got: `Steps to reproduce: open the app, click settings, then toggle dark mode, and finally restart`
- why: list: want 4 list lines, got 0; layout: want 5 lines/1 paragraphs, got 1/1

### `zh-003` 0% [filler]

- input: `就是说这个功能嗯下周之前要上线`
- expected: `这个功能下周之前要上线`
- got: `就是说这个功能，下周之前要上线`
- why: error: 27% > 25%

### `zh-009` 0% [mixed-language, keep-meaning]

- input: `我明天要跟老板讨论一下这个project的deadline`
- expected: `我明天要跟老板讨论一下这个project的deadline`
- got: `我明天要跟老板讨论一下这个项目的deadline`
- why: must_contain: project

### `zh-010` 0% [mixed-language, punctuation]

- input: `你先把email发给我我看完再confirm`
- expected: `你先把email发给我，我看完再confirm`
- got: `你先把email發給我，我看完再confirm`
- why: script: simplified wanted, found 發給

### `zh-011` 0% [numbers, punctuation]

- input: `这个月的增长率是百分之十二点五比上个月高了两个点`
- expected: `这个月的增长率是12.5%，比上个月高了两个点`
- got: `这个月的增长率是百分之十二点五，比上个月高了两个点`
- why: must_contain: 12.5; numbers: want [12.5] got []; error: 35% > 25%

### `zh-017` 0% [self-correction]

- input: `我们订五个人的位子我是说六个人`
- expected: `我们订六个人的位子`
- got: `我们订五个人的位子，我是说六个人`
- why: must_not_contain: 五个; must_not_contain: 我是说; error: 78% > 25%

### `zh-019` 0% [keep-meaning, dialect]

- input: `他刚才说这份报告明天才可以给我们`
- expected: `他刚才说这份报告明天才可以给我们`
- got: `他剛才說這份報告明天才可以給我們`
- why: must_contain: 刚才; script: simplified wanted, found 剛說這報給們; error: 38% > 25%


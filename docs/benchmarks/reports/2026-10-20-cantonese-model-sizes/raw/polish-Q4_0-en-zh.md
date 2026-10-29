# Polish evaluation

Model `qwen3.5-4b`, split `dev`, 2 sample(s) per case, git `eb99ead+dirty`, 2026-10-19T20:14:07.976Z.

Overall pass rate: **85%** over 60 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 32 | 91% | 2% |
| zh-Hans | 28 | 79% | 9% |

## By tag

| Tag | en | zh-Hans |
|---|---|---|
| dash | 100% (1) |  |
| dialect |  | 83% (3) |
| filler | 88% (8) | 75% (8) |
| keep-meaning | 100% (10) | 70% (5) |
| list | 100% (3) | 100% (2) |
| mixed-language |  | 63% (4) |
| newline | 100% (3) | 100% (3) |
| numbers | 75% (8) | 75% (4) |
| punctuation | 94% (8) | 86% (11) |
| script |  | 100% (1) |
| self-correction | 100% (4) | 67% (3) |

## Failed checks (samples)

| Check | en | zh-Hans |
|---|---|---|
| error |  | 7 |
| must_contain | 3 | 5 |
| must_not_contain | 2 | 6 |
| numbers | 4 | 2 |
| script |  | 2 |

## Cases that failed at least once (11)

### `en-005` 0% [filler]

- input: `I checked the logs you know and there was nothing there`
- expected: `I checked the logs and there was nothing there`
- got: `I checked the logs, you know, and there was nothing there`
- why: must_not_contain: you know

### `en-012` 50% [numbers]

- input: `we grew about twenty percent last quarter and revenue hit three point two million dollars`
- expected: `We grew about 20% last quarter and revenue hit $3.2 million`
- got: `We grew about twenty percent last quarter, and revenue hit 3.2 million dollars`
- why: numbers: want [20 3.2] got [3.2]

### `en-013` 0% [numbers]

- input: `please upgrade node to version twenty two point one one and then rerun the tests`
- expected: `Please upgrade Node to version 22.11 and then rerun the tests`
- got: `Please upgrade Node to version 22.1.1 and then rerun the tests`
- why: must_contain: 22.11; numbers: want [22.11] got [22.1.1]

### `en-027` 50% [numbers, punctuation]

- input: `can you make the font size fourteen and the line height one point five`
- expected: `Can you make the font size 14 and the line height 1.5?`
- got: `Can you make the font size fourteen and the line height 1.5?`
- why: must_contain: 14; numbers: want [1.5 14] got [1.5]

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

### `zh-010` 50% [mixed-language, punctuation]

- input: `你先把email发给我我看完再confirm`
- expected: `你先把email发给我，我看完再confirm`
- got: `請先把 email 發給我，我看完再 confirm`
- why: script: simplified wanted, found 請發給

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

### `zh-019` 50% [keep-meaning, dialect]

- input: `他刚才说这份报告明天才可以给我们`
- expected: `他刚才说这份报告明天才可以给我们`
- got: `他剛才說這份報告明天才可以給我們`
- why: must_contain: 刚才; script: simplified wanted, found 剛說這報給們; error: 38% > 25%

### `zh-026` 0% [filler]

- input: `就是说我们需要在下周之前完成所有的测试`
- expected: `我们需要在下周之前完成所有的测试`
- got: `就是说我们需要在下周之前完成所有的测试`
- why: must_not_contain: 就是说


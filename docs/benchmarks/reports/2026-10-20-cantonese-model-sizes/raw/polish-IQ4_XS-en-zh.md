# Polish evaluation

Model `qwen3.5-4b`, split `dev`, 2 sample(s) per case, git `eb99ead+dirty`, 2026-10-19T20:19:17.507Z.

Overall pass rate: **84%** over 60 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 32 | 86% | 3% |
| zh-Hans | 28 | 82% | 8% |

## By tag

| Tag | en | zh-Hans |
|---|---|---|
| dash | 100% (1) |  |
| dialect |  | 83% (3) |
| filler | 88% (8) | 75% (8) |
| keep-meaning | 90% (10) | 90% (5) |
| list | 83% (3) | 100% (2) |
| mixed-language |  | 88% (4) |
| newline | 83% (3) | 100% (3) |
| numbers | 63% (8) | 75% (4) |
| punctuation | 100% (8) | 86% (11) |
| script |  | 100% (1) |
| self-correction | 100% (4) | 67% (3) |

## Failed checks (samples)

| Check | en | zh-Hans |
|---|---|---|
| error |  | 7 |
| layout | 1 |  |
| list | 1 |  |
| must_contain | 4 | 3 |
| must_not_contain | 2 | 6 |
| numbers | 6 | 2 |
| script |  | 2 |

## Cases that failed at least once (11)

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

### `en-020` 0% [numbers, keep-meaning]

- input: `the API returns a four oh four when the user ID is missing which is expected`
- expected: `The API returns a 404 when the user ID is missing, which is expected`
- got: `The API returns a four oh four when the user ID is missing, which is expected`
- why: must_contain: 404; numbers: want [404] got []

### `en-030` 50% [list, newline]

- input: `steps to reproduce open the app click settings then toggle dark mode and finally restart`
- expected: `Steps to reproduce:⏎1. Open the app⏎2. Click Settings⏎3. Toggle dark mode⏎4. Restart`
- got: `Steps to reproduce: Open the app, click settings, then toggle dark mode, and finally restart`
- why: list: want 4 list lines, got 0; layout: want 5 lines/1 paragraphs, got 1/1

### `zh-003` 0% [filler]

- input: `就是说这个功能嗯下周之前要上线`
- expected: `这个功能下周之前要上线`
- got: `就是说这个功能，下周之前要上线`
- why: error: 27% > 25%

### `zh-010` 50% [mixed-language, punctuation]

- input: `你先把email发给我我看完再confirm`
- expected: `你先把email发给我，我看完再confirm`
- got: `你先把 email 發給我，我看完再 confirm`
- why: script: simplified wanted, found 發給

### `zh-011` 0% [numbers, punctuation]

- input: `这个月的增长率是百分之十二点五比上个月高了两个点`
- expected: `这个月的增长率是12.5%，比上个月高了两个点`
- got: `这个月的增长率是百分之十二点五，比上个月高了两个点`
- why: must_contain: 12.5; numbers: want [12.5] got []; error: 35% > 25%

### `zh-017` 0% [self-correction]

- input: `我们订五个人的位子我是说六个人`
- expected: `我们订六个人的位子`
- got: `我们订了五个人的位子，我是说六个人`
- why: must_not_contain: 五个; must_not_contain: 我是说; error: 89% > 25%

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


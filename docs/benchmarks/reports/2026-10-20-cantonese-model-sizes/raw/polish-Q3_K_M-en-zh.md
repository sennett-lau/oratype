# Polish evaluation

Model `qwen3.5-4b`, split `dev`, 2 sample(s) per case, git `eb99ead+dirty`, 2026-10-19T20:24:36.091Z.

Overall pass rate: **79%** over 60 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 32 | 86% | 3% |
| zh-Hans | 28 | 71% | 9% |

## By tag

| Tag | en | zh-Hans |
|---|---|---|
| dash | 100% (1) |  |
| dialect |  | 33% (3) |
| filler | 88% (8) | 75% (8) |
| keep-meaning | 90% (10) | 80% (5) |
| list | 83% (3) | 100% (2) |
| mixed-language |  | 50% (4) |
| newline | 83% (3) | 83% (3) |
| numbers | 69% (8) | 63% (4) |
| punctuation | 100% (8) | 68% (11) |
| script |  | 100% (1) |
| self-correction | 100% (4) | 83% (3) |

## Failed checks (samples)

| Check | en | zh-Hans |
|---|---|---|
| error |  | 10 |
| layout | 1 |  |
| list | 1 |  |
| must_contain | 4 | 8 |
| must_not_contain | 2 | 3 |
| numbers | 5 | 2 |
| punctuation |  | 1 |
| script |  | 9 |

## Cases that failed at least once (16)

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
- got: `Please upgrade node to version 22.1.1 and then rerun the tests`
- why: must_contain: 22.11; numbers: want [22.11] got [22.1.1]

### `en-020` 50% [numbers, keep-meaning]

- input: `the API returns a four oh four when the user ID is missing which is expected`
- expected: `The API returns a 404 when the user ID is missing, which is expected`
- got: `The API returns a four oh four when the user ID is missing, which is expected`
- why: must_contain: 404; numbers: want [404] got []

### `en-028` 50% [keep-meaning]

- input: `rename the file to config dot yaml and push it to the main branch`
- expected: `Rename the file to config.yaml and push it to the main branch`
- got: `Rename the file to .yaml and push it to the main branch`
- why: must_contain: config.yaml

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

### `zh-010` 0% [mixed-language, punctuation]

- input: `你先把email发给我我看完再confirm`
- expected: `你先把email发给我，我看完再confirm`
- got: `你先把 email 發給我，我看完再 confirm`
- why: script: simplified wanted, found 發給

### `zh-012` 0% [numbers]

- input: `请把版本号改成三点零点一然后重新打包`
- expected: `请把版本号改成3.0.1，然后重新打包`
- got: `请把版本号改成 1.0.1 然后重新打包`
- why: must_contain: 3.0.1; numbers: want [3.0.1] got [1.0.1]

### `zh-013` 50% [numbers, filler]

- input: `嗯这个手机的价格是两千九百九十九元`
- expected: `这个手机的价格是2999元`
- got: `這個手機的價格是兩千九百九十九元`
- why: script: simplified wanted, found 這個機價兩; error: 31% > 25%

### `zh-017` 50% [self-correction]

- input: `我们订五个人的位子我是说六个人`
- expected: `我们订六个人的位子`
- got: `我们订了五个人的位子，我是说六个人`
- why: must_not_contain: 五个; must_not_contain: 我是说; error: 89% > 25%

### `zh-019` 0% [keep-meaning, dialect]

- input: `他刚才说这份报告明天才可以给我们`
- expected: `他刚才说这份报告明天才可以给我们`
- got: `他剛才說這份報告明天才可以給我們`
- why: must_contain: 刚才; script: simplified wanted, found 剛說這報給們; error: 38% > 25%

### `zh-020` 50% [newline, punctuation]

- input: `今天的发布很顺利用户反馈也不错另外下周一记得交周报`
- expected: `今天的发布很顺利，用户反馈也不错。⏎⏎另外，下周一记得交周报`
- got: `今天的发布很顺利，用户反馈也不错，另外下周一记得交周报`
- why: punctuation: want 1 sentence breaks, got 0

### `zh-025` 0% [mixed-language, punctuation]

- input: `这个feature下个礼拜可以上线吗`
- expected: `这个feature下个礼拜可以上线吗？`
- got: `這個 feature 下個禮拜可以上線嗎`
- why: must_contain: 礼拜; script: simplified wanted, found 這個禮線嗎; error: 33% > 25%

### `zh-026` 50% [filler]

- input: `就是说我们需要在下周之前完成所有的测试`
- expected: `我们需要在下周之前完成所有的测试`
- got: `就是说我们需要在下周之前完成所有的测试`
- why: must_not_contain: 就是说

### `zh-027` 0% [punctuation, dialect]

- input: `现在的情况是我们没有足够的时间去做这件事`
- expected: `现在的情况是，我们没有足够的时间去做这件事`
- got: `現在的情況是我們沒有足夠的時間去做這件事`
- why: must_contain: 没有足够的时间; script: simplified wanted, found 現況們沒時間這; error: 40% > 25%


# Polish evaluation

Model `Qwen3.5-4B-Q4_K_M.gguf`, split `all`, 3 sample(s) per case, git `fad8fc0+dirty`, 2026-11-03T01:45:18.377Z.

Overall pass rate: **82%** over 216 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 57 | 88% | 5% |
| es | 16 | 77% | 11% |
| fr | 16 | 81% | 9% |
| ja | 16 | 79% | 11% |
| yue | 64 | 82% | 11% |
| zh-Hans | 47 | 79% | 9% |

## By tag

| Tag | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| dash | 100% (2) |  |  |  |  |  |
| dialect |  |  |  |  | 90% (47) | 67% (4) |
| filler | 85% (13) | 58% (4) | 50% (4) | 75% (4) | 96% (9) | 70% (11) |
| keep-meaning | 86% (12) | 100% (1) | 33% (1) | 50% (2) | 98% (18) | 67% (7) |
| list | 83% (4) | 100% (1) | 100% (1) | 100% (1) | 100% (4) | 100% (3) |
| mixed-language |  |  |  | 50% (2) | 75% (20) | 71% (8) |
| newline | 87% (5) | 100% (1) | 100% (1) | 100% (1) | 80% (5) | 80% (5) |
| no-answer | 100% (15) | 83% (6) | 100% (6) | 100% (6) | 100% (12) | 100% (9) |
| numbers | 60% (10) | 33% (1) | 67% (3) | 100% (2) | 40% (5) | 87% (5) |
| punctuation | 97% (11) | 100% (3) | 83% (2) | 100% (2) | 75% (19) | 79% (14) |
| script |  |  |  |  | 0% (2) | 100% (1) |
| self-correction | 83% (6) | 50% (2) | 50% (2) | 33% (2) | 0% (5) | 67% (4) |

## Failed checks (samples)

| Check | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| error | 6 | 4 | 3 | 7 | 22 | 26 |
| filler |  |  |  | 6 |  |  |
| layout | 2 |  |  |  |  |  |
| list | 2 |  |  |  |  |  |
| must_contain | 6 | 3 | 2 | 6 | 18 | 22 |
| must_not_contain | 3 | 8 | 6 | 7 | 16 | 10 |
| numbers | 12 | 2 |  |  | 4 | 2 |
| punctuation |  |  | 1 |  | 3 | 3 |
| script |  |  |  |  | 6 | 10 |

## Against baseline `Qwen3.5-4B-Q4_K_M.gguf`

| Language | Compared | Before | After | Change |
|---|---|---|---|---|

### Regressions (0)


### Improvements (0)


## Cases that failed at least once (48)

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

### `en-020` 33% [numbers, keep-meaning]

- input: `the API returns a four oh four when the user ID is missing which is expected`
- expected: `The API returns a 404 when the user ID is missing, which is expected`
- got: `The API returns a four oh four when the user ID is missing, which is expected`
- why: must_contain: 404; numbers: want [404] got []

### `en-027` 67% [numbers, punctuation]

- input: `can you make the font size fourteen and the line height one point five`
- expected: `Can you make the font size 14 and the line height 1.5?`
- got: `Can you make the font size fourteen and the line height 1.5?`
- why: must_contain: 14; numbers: want [1.5 14] got [1.5]

### `en-030` 33% [list, newline]

- input: `steps to reproduce open the app click settings then toggle dark mode and finally restart`
- expected: `Steps to reproduce:⏎1. Open the app⏎2. Click Settings⏎3. Toggle dark mode⏎4. Restart`
- got: `Steps to reproduce: Open the app, click settings, then toggle dark mode, and finally restart`
- why: list: want 4 list lines, got 0; layout: want 5 lines/1 paragraphs, got 1/1

### `en-h06` 0% [keep-meaning, filler]

- input: `you know what let's just cancel it`
- expected: `You know what, let's just cancel it`
- got: `Let's just cancel it`
- why: error: 38% > 30%

### `en-h08` 0% [self-correction, numbers]

- input: `price it at nineteen ninety nine no twenty four ninety nine`
- expected: `Price it at 24.99`
- got: `Price it at nineteen ninety-nine, not twenty-four ninety-nine`
- why: numbers: want [24.99] got []; error: 160% > 30%

### `es-001` 0% [filler]

- input: `bueno o sea el servidor se cayó otra vez esta mañana`
- expected: `El servidor se cayó otra vez esta mañana`
- got: `Bueno, o sea, el servidor se cayó otra vez esta mañana`
- why: must_not_contain: o sea

### `es-004` 33% [numbers, filler]

- input: `eh las ventas subieron un doce coma cinco por ciento`
- expected: `Las ventas subieron un 12,5 %`
- got: `Las ventas subieron un 12.5 por ciento`
- why: numbers: want [12,5] got [12.5]

### `es-012` 0% [no-answer]

- input: `explícame qué es una base de datos`
- expected: `Explícame qué es una base de datos`
- got: `¿Qué es una base de datos?`
- why: must_contain: explícame

### `es-h01` 0% [self-correction]

- input: `mándaselo a Carlos no espera a Lucía`
- expected: `Mándaselo a Lucía`
- got: `Mándaselo a Carlos, no, espera a Lucía`
- why: must_not_contain: Carlos; must_not_contain: espera; error: 133% > 30%

### `fr-004` 0% [filler, numbers]

- input: `bon ben du coup on se voit demain à dix heures`
- expected: `Du coup, on se voit demain à 10 h`
- got: `Bon ben, du coup, on se voit demain à dix heures`
- why: must_not_contain: ben

### `fr-007` 67% [punctuation]

- input: `le serveur a planté hier soir on a redémarré ce matin tout fonctionne`
- expected: `Le serveur a planté hier soir. On a redémarré ce matin, tout fonctionne.`
- got: `Le serveur a planté hier soir. On a redémarré ce matin. Tout fonctionne.`
- why: punctuation: want 1 sentence breaks, got 2

### `fr-008` 33% [keep-meaning]

- input: `non je ne suis pas d'accord avec cette proposition`
- expected: `Non, je ne suis pas d'accord avec cette proposition`
- got: `Je ne suis pas d'accord avec cette proposition`
- why: must_contain: Non

### `fr-h01` 0% [filler, self-correction]

- input: `euh la livraison est prévue mardi non mercredi`
- expected: `La livraison est prévue mercredi`
- got: `La livraison est prévue mardi, non mercredi`
- why: must_not_contain: mardi; error: 40% > 30%

### `ja-001` 0% [filler]

- input: `えーと明日の会議は三時からです`
- expected: `明日の会議は三時からです`
- got: `えーと、明日の会議は三時からです`
- why: must_not_contain: えーと; filler: えーと; filler: えー

### `ja-002` 33% [self-correction]

- input: `資料は田中さんに送ってください違う佐藤さんに送ってください`
- expected: `資料は佐藤さんに送ってください`
- got: `資料は田中さんに送ってください、違う佐藤さんに送ってください`
- why: must_not_contain: 田中; error: 93% > 25%

### `ja-007` 0% [mixed-language, keep-meaning]

- input: `このprojectのdeadlineは来週の金曜日です`
- expected: `このprojectのdeadlineは来週の金曜日です`
- got: `このプロジェクトのデッドラインは来週の金曜日です`
- why: must_contain: project; must_contain: deadline; error: 56% > 25%

### `ja-h02` 33% [self-correction]

- input: `会議は月曜日いや火曜日にしましょう`
- expected: `会議は火曜日にしましょう`
- got: `会議は月曜日、いや火曜日にしましょう`
- why: must_not_contain: 月曜日; error: 42% > 25%

### `yue-003` 67% [filler, dialect, punctuation]

- input: `嗯我想問下你得唔得閒星期五食飯`
- expected: `我想問下你得唔得閒星期五食飯？`
- got: `我想問下你星期五得唔得閒食飯？`
- why: error: 43% > 25%

### `yue-004` 0% [self-correction, dialect]

- input: `三點鐘唔係唔好意思四點鐘喺會議室等`
- expected: `四點鐘喺會議室等`
- got: `三點鐘唔好意思，四點鐘喺會議室等`
- why: must_not_contain: 三點; must_not_contain: 唔好意思; error: 88% > 25%

### `yue-011` 0% [self-correction, numbers]

- input: `個價錢係九十九蚊唔係唔係八十九蚊`
- expected: `個價錢係八十九蚊`
- got: `個價錢係九十九蚊，唔係八十九蚊`
- why: must_not_contain: 九十九; error: 75% > 25%

### `yue-012` 67% [numbers, punctuation]

- input: `今個月營業額升咗百分之十五總共係三點二億`
- expected: `今個月營業額升咗15%，總共係3.2億`
- got: `今個月營業額升咗百分之十五，總共係三點二億`
- why: must_contain: 3.2; numbers: want [3.2] got []

### `yue-013` 33% [numbers, mixed-language]

- input: `將個version升去二點一然後再試多次`
- expected: `將個version升去2.1，然後再試多次`
- got: `將版本升級到 2.1，然後再試多次`
- why: must_contain: version; error: 53% > 25%

### `yue-018` 0% [newline, punctuation, mixed-language]

- input: `啱啱同老闆傾完佢話OK喎另外聽日記得帶埋部laptop`
- expected: `啱啱同老闆傾完，佢話OK喎。⏎⏎另外，聽日記得帶埋部laptop`
- got: `啱啱同老闆傾完，佢話 OK 喎，另外聽日記得帶埋部 laptop`
- why: punctuation: want 1 sentence breaks, got 0

### `yue-019` 0% [self-correction, mixed-language]

- input: `我哋約咗Peter唔係係Paul喺中環食lunch`
- expected: `我哋約咗Paul喺中環食lunch`
- got: `我哋約咗Peter喺中環食lunch`
- why: must_contain: Paul; must_not_contain: Peter

### `yue-025` 0% [script, dialect, punctuation]

- input: `我觉得呢个方案几好但系个价钱贵咗啲`
- expected: `我觉得呢个方案几好，但系个价钱贵咗啲`
- got: `我覺得呢個方案幾好，但係個價錢貴咗啲`
- why: must_contain: 几好; must_contain: 贵咗啲; script: simplified wanted, found 覺個幾價錢貴; error: 47% > 25%

### `yue-030` 0% [self-correction, mixed-language]

- input: `我想book兩個位唔係三個位七點半`
- expected: `我想book三個位，七點半`
- got: `我想 book 兩個位，唔係三個位，七點半`
- why: must_not_contain: 兩個; error: 42% > 25%

### `yue-036` 67% [keep-meaning, dialect, mixed-language]

- input: `我唔係講緊你我講緊上個禮拜嗰個project`
- expected: `我唔係講緊你，我講緊上個禮拜嗰個project`
- got: `我唔係講緊，我講緊上個禮拜嗰個 project`
- why: must_contain: 唔係講緊你

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

### `yue-h08` 0% [dialect, punctuation]

- input: `呢度啲嘢好貴我哋去第二度食啦`
- expected: `呢度啲嘢好貴，我哋去第二度食啦`
- got: `呢度嘅嘢好貴，我哋去第二度食啦`
- why: must_contain: 啲嘢

### `yue-h10` 0% [script, dialect]

- input: `个server又死咗我哋要重新开过`
- expected: `个server又死咗，我哋要重新开过`
- got: `個 server 又死咗，我哋要重新開過`
- why: script: simplified wanted, found 個開過

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
- got: `你先發email給我，我看完再確認`
- why: must_contain: confirm; script: simplified wanted, found 發給確認; error: 45% > 25%

### `zh-011` 33% [numbers, punctuation]

- input: `这个月的增长率是百分之十二点五比上个月高了两个点`
- expected: `这个月的增长率是12.5%，比上个月高了两个点`
- got: `这个月的增长率是百分之十二点五，比上个月高了两个点`
- why: must_contain: 12.5; numbers: want [12.5] got []; error: 35% > 25%

### `zh-017` 67% [self-correction]

- input: `我们订五个人的位子我是说六个人`
- expected: `我们订六个人的位子`
- got: `我们订五个人的位子，我是说六个人`
- why: must_not_contain: 五个; must_not_contain: 我是说; error: 78% > 25%

### `zh-019` 0% [keep-meaning, dialect]

- input: `他刚才说这份报告明天才可以给我们`
- expected: `他刚才说这份报告明天才可以给我们`
- got: `他剛才說這份報告明天才可以給我們`
- why: must_contain: 刚才; script: simplified wanted, found 剛說這報給們; error: 38% > 25%

### `zh-020` 67% [newline, punctuation]

- input: `今天的发布很顺利用户反馈也不错另外下周一记得交周报`
- expected: `今天的发布很顺利，用户反馈也不错。⏎⏎另外，下周一记得交周报`
- got: `今天的发布很顺利，用户反馈也不错，另外下周一记得交周报`
- why: punctuation: want 1 sentence breaks, got 0

### `zh-026` 0% [filler]

- input: `就是说我们需要在下周之前完成所有的测试`
- expected: `我们需要在下周之前完成所有的测试`
- got: `我們需要在下週之前完成所有測試`
- why: must_contain: 下周之前; must_contain: 测试; script: simplified wanted, found 們測試; error: 31% > 25%

### `zh-027` 67% [punctuation, dialect]

- input: `现在的情况是我们没有足够的时间去做这件事`
- expected: `现在的情况是，我们没有足够的时间去做这件事`
- got: `現在的情況是我們沒有足夠的時間去做這件事`
- why: must_contain: 没有足够的时间; script: simplified wanted, found 現況們沒時間這; error: 40% > 25%

### `zh-h01` 67% [filler]

- input: `嗯那个下午的会取消了`
- expected: `下午的会取消了`
- got: `那个下午的会取消了`
- why: error: 29% > 25%

### `zh-h02` 0% [self-correction]

- input: `报销单交给财务不对交给行政`
- expected: `报销单交给行政`
- got: `报销单交给财务不对，交给行政`
- why: must_not_contain: 财务; must_not_contain: 不对; error: 86% > 25%

### `zh-h08` 0% [filler]

- input: `然后呢我们那个那个预算就是说已经超了`
- expected: `我们的预算已经超了`
- got: `然后呢，我们那个预算就是说已经超了`
- why: must_not_contain: 就是说; error: 55% > 25%

### `zh-h09` 67% [mixed-language, keep-meaning]

- input: `请帮我update一下这个report`
- expected: `请帮我update一下这个report`
- got: `请帮我更新一下这份报告`
- why: must_contain: update; must_contain: report; error: 68% > 25%

### `zh-h10` 33% [newline, punctuation]

- input: `项目进展顺利另外提醒一下周五下午公司停电`
- expected: `项目进展顺利。⏎⏎另外提醒一下，周五下午公司停电`
- got: `项目进展顺利，另外提醒一下，周五下午公司停电`
- why: punctuation: want 1 sentence breaks, got 0


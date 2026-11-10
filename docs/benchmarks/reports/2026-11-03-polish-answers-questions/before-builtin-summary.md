# Polish evaluation

Model `Qwen3.5-4B-Q4_K_M.gguf`, split `all`, 3 sample(s) per case, git `fad8fc0+dirty`, 2026-11-03T01:23:20.213Z.

Overall pass rate: **80%** over 216 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 57 | 87% | 8% |
| es | 16 | 81% | 12% |
| fr | 16 | 79% | 13% |
| ja | 16 | 75% | 14% |
| yue | 64 | 80% | 10% |
| zh-Hans | 47 | 76% | 11% |

## By tag

| Tag | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| dash | 67% (2) |  |  |  |  |  |
| dialect |  |  |  |  | 87% (47) | 75% (4) |
| filler | 82% (13) | 50% (4) | 50% (4) | 67% (4) | 96% (9) | 76% (11) |
| keep-meaning | 86% (12) | 100% (1) | 33% (1) | 50% (2) | 94% (18) | 62% (7) |
| list | 83% (4) | 100% (1) | 100% (1) | 100% (1) | 100% (4) | 100% (3) |
| mixed-language |  |  |  | 50% (2) | 77% (20) | 58% (8) |
| newline | 87% (5) | 100% (1) | 100% (1) | 100% (1) | 80% (5) | 87% (5) |
| no-answer | 96% (15) | 100% (6) | 89% (6) | 94% (6) | 97% (12) | 93% (9) |
| numbers | 63% (10) | 0% (1) | 67% (3) | 100% (2) | 33% (5) | 80% (5) |
| punctuation | 100% (11) | 100% (3) | 100% (2) | 100% (2) | 72% (19) | 76% (14) |
| script |  |  |  |  | 0% (2) | 100% (1) |
| self-correction | 78% (6) | 50% (2) | 50% (2) | 17% (2) | 0% (5) | 50% (4) |

## Failed checks (samples)

| Check | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| dialect |  |  |  |  | 1 |  |
| error | 11 | 6 | 5 | 8 | 20 | 26 |
| filler |  |  |  | 8 |  |  |
| layout | 4 |  | 1 |  |  |  |
| list | 4 |  | 1 |  |  |  |
| must_contain | 5 |  | 2 | 6 | 21 | 18 |
| must_not_contain | 3 | 6 | 6 | 9 | 17 | 16 |
| numbers | 11 | 3 |  |  | 6 | 3 |
| punctuation |  |  |  |  | 3 | 2 |
| script |  |  |  |  | 6 | 6 |

## Cases that failed at least once (52)

### `en-005` 33% [filler]

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

### `en-030` 33% [list, newline]

- input: `steps to reproduce open the app click settings then toggle dark mode and finally restart`
- expected: `Steps to reproduce:⏎1. Open the app⏎2. Click Settings⏎3. Toggle dark mode⏎4. Restart`
- got: `Steps to reproduce: open the app, click settings, then toggle dark mode, and finally restart`
- why: list: want 4 list lines, got 0; layout: want 5 lines/1 paragraphs, got 1/1

### `en-037` 33% [no-answer]

- input: `give me three ideas for a team lunch`
- expected: `Give me three ideas for a team lunch`
- got: `Here are three ideas for a team lunch:⏎1. A casual taco bar⏎2. A picnic in the park⏎3. A quiet sushi dinner`
- why: list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/1; error: 188% > 30%

### `en-h02` 67% [self-correction]

- input: `invite the design team no actually the whole product team to the kickoff`
- expected: `Invite the whole product team to the kickoff`
- got: `Invite the design team, actually the whole product team, to the kickoff`
- why: must_not_contain: design; error: 50% > 30%

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

### `en-h10` 33% [dash, filler]

- input: `mm the meeting the one with finance got moved to next week`
- expected: `The meeting, the one with finance, got moved to next week`
- got: `The meeting with finance has been moved to next week`
- why: error: 36% > 30%

### `es-001` 0% [filler]

- input: `bueno o sea el servidor se cayó otra vez esta mañana`
- expected: `El servidor se cayó otra vez esta mañana`
- got: `Bueno, o sea, el servidor se cayó otra vez esta mañana`
- why: must_not_contain: o sea

### `es-004` 0% [numbers, filler]

- input: `eh las ventas subieron un doce coma cinco por ciento`
- expected: `Las ventas subieron un 12,5 %`
- got: `Las ventas subieron un doce coma cinco por ciento`
- why: numbers: want [12,5] got []; error: 38% > 30%

### `es-h01` 0% [self-correction]

- input: `mándaselo a Carlos no espera a Lucía`
- expected: `Mándaselo a Lucía`
- got: `Mándaselo a Carlos, no espere a Lucía`
- why: must_not_contain: Carlos; error: 133% > 30%

### `fr-004` 0% [filler, numbers]

- input: `bon ben du coup on se voit demain à dix heures`
- expected: `Du coup, on se voit demain à 10 h`
- got: `Bon, ben, du coup, on se voit demain à dix heures`
- why: must_not_contain: ben

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

### `fr-h04` 33% [no-answer]

- input: `donne moi trois idées de cadeau pour mon frère`
- expected: `Donne-moi trois idées de cadeau pour mon frère`
- got: `Voici trois idées de cadeau pour mon frère :⏎1. Un jeu de vidéo⏎2. Un livre de voyage⏎3. Une expérience`
- why: list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/1; error: 133% > 30%

### `ja-001` 0% [filler]

- input: `えーと明日の会議は三時からです`
- expected: `明日の会議は三時からです`
- got: `えーと、明日の会議は三時からです`
- why: must_not_contain: えーと; filler: えーと; filler: えー

### `ja-002` 0% [self-correction]

- input: `資料は田中さんに送ってください違う佐藤さんに送ってください`
- expected: `資料は佐藤さんに送ってください`
- got: `資料を田中さんへ送ってください。違う、佐藤さんへ送ってください。`
- why: must_not_contain: 田中; error: 107% > 25%

### `ja-007` 0% [mixed-language, keep-meaning]

- input: `このprojectのdeadlineは来週の金曜日です`
- expected: `このprojectのdeadlineは来週の金曜日です`
- got: `このプロジェクトのデッドラインは来週の金曜日です`
- why: must_contain: project; must_contain: deadline; error: 56% > 25%

### `ja-009` 67% [no-answer, filler]

- input: `えーと東京から大阪まで新幹線で何時間かかりますか`
- expected: `東京から大阪まで新幹線で何時間かかりますか？`
- got: `えーと、東京から大阪まで新幹線で何時間かかりますか`
- why: must_not_contain: えーと; filler: えーと; filler: えー

### `ja-h02` 33% [self-correction]

- input: `会議は月曜日いや火曜日にしましょう`
- expected: `会議は火曜日にしましょう`
- got: `会議は月曜日、いや火曜日にしましょう`
- why: must_not_contain: 月曜日; error: 42% > 25%

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

### `yue-018` 33% [newline, punctuation, mixed-language]

- input: `啱啱同老闆傾完佢話OK喎另外聽日記得帶埋部laptop`
- expected: `啱啱同老闆傾完，佢話OK喎。⏎⏎另外，聽日記得帶埋部laptop`
- got: `啱啱同老闆傾完，佢話 OK 喎，另外聽日記得帶埋部 laptop`
- why: punctuation: want 1 sentence breaks, got 0

### `yue-019` 0% [self-correction, mixed-language]

- input: `我哋約咗Peter唔係係Paul喺中環食lunch`
- expected: `我哋約咗Paul喺中環食lunch`
- got: `我哋約咗Peter喺中環食lunch，唔係Paul`
- why: must_not_contain: Peter; error: 59% > 25%

### `yue-020` 67% [dialect, punctuation]

- input: `呢個週末天氣好好我想去公園行下`
- expected: `呢個週末天氣好好，我想去公園行下`
- got: `這個週末天氣好好，我想去公園行下`
- why: must_not_contain: 這個; dialect: unexpected 這個

### `yue-023` 67% [numbers, dialect]

- input: `我哋聽日上晝十點半開會`
- expected: `我哋聽日上晝10點半開會`
- got: `我聽日十點半開會`
- why: error: 27% > 25%

### `yue-025` 0% [script, dialect, punctuation]

- input: `我觉得呢个方案几好但系个价钱贵咗啲`
- expected: `我觉得呢个方案几好，但系个价钱贵咗啲`
- got: `我覺得呢個方案幾好，但係個價錢貴咗啲`
- why: must_contain: 几好; must_contain: 贵咗啲; script: simplified wanted, found 覺個幾價錢貴; error: 47% > 25%

### `yue-030` 0% [self-correction, mixed-language]

- input: `我想book兩個位唔係三個位七點半`
- expected: `我想book三個位，七點半`
- got: `我想book兩個位，唔係三個位，七點半`
- why: must_not_contain: 兩個; error: 42% > 25%

### `yue-036` 0% [keep-meaning, dialect, mixed-language]

- input: `我唔係講緊你我講緊上個禮拜嗰個project`
- expected: `我唔係講緊你，我講緊上個禮拜嗰個project`
- got: `我唔係講緊，我講緊上個禮拜嗰個 project`
- why: must_contain: 唔係講緊你

### `yue-041` 67% [no-answer, filler, dialect]

- input: `嗯香港最高嘅山係邊座`
- expected: `香港最高嘅山係邊座？`
- got: `香港最高嘅山係哪座？`
- why: must_contain: 邊座

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

### `yue-h09` 67% [newline, punctuation]

- input: `多謝你幫手個網站而家快咗好多另外下個禮拜我放假`
- expected: `多謝你幫手，個網站而家快咗好多。⏎⏎另外，下個禮拜我放假`
- got: `多謝你幫手個網站，而家快咗好多，另外下個禮拜我放假`
- why: punctuation: want 1 sentence breaks, got 0

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
- got: `我明天要跟老板讨论一下这个项目的截止日期`
- why: must_contain: project; must_contain: deadline; error: 52% > 25%

### `zh-010` 0% [mixed-language, punctuation]

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
- got: `我们订五个人的位子，我是说六个人`
- why: must_not_contain: 五个; must_not_contain: 我是说; error: 78% > 25%

### `zh-019` 0% [keep-meaning, dialect]

- input: `他刚才说这份报告明天才可以给我们`
- expected: `他刚才说这份报告明天才可以给我们`
- got: `他剛才說這份報告明天才可以給我們`
- why: must_contain: 刚才; script: simplified wanted, found 剛說這報給們; error: 38% > 25%

### `zh-025` 33% [mixed-language, punctuation]

- input: `这个feature下个礼拜可以上线吗`
- expected: `这个feature下个礼拜可以上线吗？`
- got: `这个功能下个礼拜可以上线吗`
- why: must_contain: feature; error: 39% > 25%

### `zh-026` 0% [filler]

- input: `就是说我们需要在下周之前完成所有的测试`
- expected: `我们需要在下周之前完成所有的测试`
- got: `就是说我们需要在下周之前完成所有的测试`
- why: must_not_contain: 就是说

### `zh-h01` 67% [filler]

- input: `嗯那个下午的会取消了`
- expected: `下午的会取消了`
- got: `那个下午的会取消了`
- why: error: 29% > 25%

### `zh-h02` 0% [self-correction]

- input: `报销单交给财务不对交给行政`
- expected: `报销单交给行政`
- got: `报销单交给行政，不对，交给财务`
- why: must_not_contain: 财务; must_not_contain: 不对; error: 86% > 25%

### `zh-h08` 67% [filler]

- input: `然后呢我们那个那个预算就是说已经超了`
- expected: `我们的预算已经超了`
- got: `然后呢，我们那个预算就是说已经超了`
- why: must_not_contain: 就是说; error: 55% > 25%

### `zh-h09` 33% [mixed-language, keep-meaning]

- input: `请帮我update一下这个report`
- expected: `请帮我update一下这个report`
- got: `请帮我更新一下这个报告`
- why: must_contain: update; must_contain: report; error: 63% > 25%

### `zh-h10` 33% [newline, punctuation]

- input: `项目进展顺利另外提醒一下周五下午公司停电`
- expected: `项目进展顺利。⏎⏎另外提醒一下，周五下午公司停电`
- got: `项目进展顺利，另外提醒一下，周五下午公司停电`
- why: punctuation: want 1 sentence breaks, got 0

### `zh-h11` 33% [no-answer]

- input: `北京明天会不会下雨`
- expected: `北京明天会不会下雨？`
- got: `北京明天会下雨吗？`
- why: error: 33% > 25%


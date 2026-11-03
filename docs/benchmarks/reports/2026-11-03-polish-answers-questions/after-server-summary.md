# Polish evaluation

Model `qwen3.5-4b-abliterated`, split `all`, 3 sample(s) per case, git `fad8fc0+dirty`, 2026-11-03T01:36:56.276Z.

Overall pass rate: **64%** over 216 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 57 | 84% | 7% |
| es | 16 | 73% | 16% |
| fr | 16 | 81% | 9% |
| ja | 16 | 60% | 22% |
| yue | 64 | 72% | 14% |
| zh-Hans | 47 | 21% | 31% |

## By tag

| Tag | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| dash | 100% (2) |  |  |  |  |  |
| dialect |  |  |  |  | 78% (47) | 25% (4) |
| filler | 90% (13) | 50% (4) | 50% (4) | 0% (4) | 56% (9) | 9% (11) |
| keep-meaning | 86% (12) | 100% (1) | 100% (1) | 50% (2) | 93% (18) | 29% (7) |
| list | 75% (4) | 100% (1) | 0% (1) | 67% (1) | 75% (4) | 0% (3) |
| mixed-language |  |  |  | 50% (2) | 62% (20) | 13% (8) |
| newline | 80% (5) | 100% (1) | 0% (1) | 67% (1) | 80% (5) | 40% (5) |
| no-answer | 98% (15) | 94% (6) | 100% (6) | 83% (6) | 89% (12) | 33% (9) |
| numbers | 50% (10) | 0% (1) | 67% (3) | 0% (2) | 53% (5) | 13% (5) |
| punctuation | 85% (11) | 100% (3) | 100% (2) | 100% (2) | 70% (19) | 36% (14) |
| script |  |  |  |  | 0% (2) | 0% (1) |
| self-correction | 67% (6) | 0% (2) | 50% (2) | 50% (2) | 0% (5) | 0% (4) |

## Failed checks (samples)

| Check | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| dialect |  |  |  |  | 6 |  |
| error | 10 | 9 | 6 | 13 | 29 | 79 |
| filler |  |  |  | 15 | 3 | 3 |
| layout | 4 |  | 3 | 1 | 3 | 3 |
| list | 4 |  | 3 | 1 | 3 | 3 |
| must_contain | 11 | 1 |  | 9 | 27 | 74 |
| must_not_contain | 6 | 15 | 6 | 12 | 24 | 8 |
| numbers | 15 | 3 |  | 3 | 3 | 5 |
| punctuation |  |  |  |  | 2 | 2 |
| script |  |  |  |  | 6 | 99 |

## Against baseline `qwen3.5-4b-abliterated`

| Language | Compared | Before | After | Change |
|---|---|---|---|---|

### Regressions (0)


### Improvements (0)


## Cases that failed at least once (85)

### `en-005` 0% [filler]

- input: `I checked the logs you know and there was nothing there`
- expected: `I checked the logs and there was nothing there`
- got: `I checked the logs, you know, and there was nothing there`
- why: must_not_contain: you know

### `en-006` 67% [filler]

- input: `hmm let me think the server crashed at like two in the morning`
- expected: `Let me think, the server crashed at like two in the morning`
- got: `The server crashed at 2 a.m.`
- why: error: 38% > 30%

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

### `en-016` 33% [punctuation, keep-meaning]

- input: `is the cache enabled no it is not`
- expected: `Is the cache enabled? No, it is not.`
- got: `Is the cache enabled? It is not.`
- why: must_contain: No

### `en-017` 0% [list, newline]

- input: `we need three things for the release first more tests second better docs and third a date`
- expected: `We need three things for the release:⏎1. More tests⏎2. Better docs⏎3. A date`
- got: `We need three things for the release: first, more tests; second, better docs; and third, a date`
- why: list: want 3 list lines, got 0; layout: want 4 lines/1 paragraphs, got 1/1; error: 31% > 30%

### `en-020` 0% [numbers, keep-meaning]

- input: `the API returns a four oh four when the user ID is missing which is expected`
- expected: `The API returns a 404 when the user ID is missing, which is expected`
- got: `The API returns a 4 0 4 when the user ID is missing, which is expected`
- why: must_contain: 404; numbers: want [404] got [0 4 4]

### `en-027` 0% [numbers, punctuation]

- input: `can you make the font size fourteen and the line height one point five`
- expected: `Can you make the font size 14 and the line height 1.5?`
- got: `Can you make the font size fourteen and the line height 1.5?`
- why: must_contain: 14; numbers: want [1.5 14] got [1.5]

### `en-h02` 0% [self-correction]

- input: `invite the design team no actually the whole product team to the kickoff`
- expected: `Invite the whole product team to the kickoff`
- got: `Invite the design team, no, actually the whole product team, to the kickoff`
- why: must_not_contain: design; error: 63% > 30%

### `en-h08` 0% [self-correction, numbers]

- input: `price it at nineteen ninety nine no twenty four ninety nine`
- expected: `Price it at 24.99`
- got: `Price it at nineteen ninety-nine, no twenty-four ninety-nine`
- why: numbers: want [24.99] got []; error: 160% > 30%

### `en-h12` 67% [no-answer]

- input: `please summarize this article in three bullet points`
- expected: `Please summarize this article in three bullet points`
- got: `Please summarize this article in three bullet points:⏎1. ...⏎2. ...⏎3. ...`
- why: list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/1

### `es-001` 0% [filler]

- input: `bueno o sea el servidor se cayó otra vez esta mañana`
- expected: `El servidor se cayó otra vez esta mañana`
- got: `Bueno, o sea, el servidor se cayó otra vez esta mañana`
- why: must_not_contain: o sea

### `es-002` 0% [self-correction]

- input: `la reunión es a las tres no perdón a las cuatro en la sala grande`
- expected: `La reunión es a las cuatro en la sala grande`
- got: `La reunión es a las tres, no perdón a las cuatro en la sala grande`
- why: must_not_contain: tres; must_not_contain: perdón; error: 50% > 30%

### `es-004` 0% [numbers, filler]

- input: `eh las ventas subieron un doce coma cinco por ciento`
- expected: `Las ventas subieron un 12,5 %`
- got: `Las ventas subieron un doce y medio por ciento`
- why: numbers: want [12,5] got []; error: 38% > 30%

### `es-012` 67% [no-answer]

- input: `explícame qué es una base de datos`
- expected: `Explícame qué es una base de datos`
- got: `Explica qué es una base de datos`
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

### `fr-005` 0% [list, newline]

- input: `il faut trois choses premièrement des tests deuxièmement de la doc et troisièmement une date de sortie`
- expected: `Il faut trois choses :⏎1. Des tests⏎2. De la doc⏎3. Une date de sortie`
- got: `Il faut trois choses : premièrement des tests, deuxièmement de la doc, et troisièmement une date de sortie`
- why: list: want 3 list lines, got 0; layout: want 4 lines/1 paragraphs, got 1/1; error: 31% > 30%

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

### `ja-002` 0% [self-correction]

- input: `資料は田中さんに送ってください違う佐藤さんに送ってください`
- expected: `資料は佐藤さんに送ってください`
- got: `資料は田中さんに送ってください。違う佐藤さんに送ってください。`
- why: must_not_contain: 田中; error: 93% > 25%

### `ja-003` 0% [filler, numbers]

- input: `あのー売上が十二点五パーセント増えました`
- expected: `売上が12.5%増えました`
- got: `あのー売上が十二点五パーセント増えました`
- why: must_contain: 12.5; must_not_contain: あのー; filler: あのー; numbers: want [12.5] got []; error: 44% > 25%

### `ja-004` 67% [list, newline]

- input: `今日のタスクは三つあります一つ目はテスト二つ目はレビュー三つ目はリリースです`
- expected: `今日のタスクは三つあります。⏎1. テスト⏎2. レビュー⏎3. リリース`
- got: `今日のタスクは三つあります。一つ目はテスト、二つ目はレビュー、三つ目はリリースです。`
- why: list: want 3 list lines, got 0; layout: want 4 lines/1 paragraphs, got 1/1; error: 58% > 25%

### `ja-007` 0% [mixed-language, keep-meaning]

- input: `このprojectのdeadlineは来週の金曜日です`
- expected: `このprojectのdeadlineは来週の金曜日です`
- got: `このプロジェクトのデッドラインは来週の金曜日です`
- why: must_contain: project; must_contain: deadline; error: 56% > 25%

### `ja-009` 0% [no-answer, filler]

- input: `えーと東京から大阪まで新幹線で何時間かかりますか`
- expected: `東京から大阪まで新幹線で何時間かかりますか？`
- got: `えーと、東京から大阪まで新幹線で何時間かかりますか？`
- why: must_not_contain: えーと; filler: えーと; filler: えー

### `ja-h01` 0% [filler, numbers]

- input: `えーとバージョン二点一にアップデートしてください`
- expected: `バージョン2.1にアップデートしてください`
- got: `アップデートしてください、バージョン 2.1 に`
- why: error: 80% > 25%

### `yue-004` 0% [self-correction, dialect]

- input: `三點鐘唔係唔好意思四點鐘喺會議室等`
- expected: `四點鐘喺會議室等`
- got: `三點鐘唔係唔好意思四點鐘喺會議室等`
- why: must_not_contain: 三點; must_not_contain: 唔好意思; error: 113% > 25%

### `yue-009` 0% [filler, mixed-language, dialect]

- input: `嗯我頭先已經send咗個file俾你喇你睇下係咪啱啦`
- expected: `我頭先已經send咗個file俾你喇，你睇下係咪啱啦`
- got: `嗯我頭先已經send咗個file俾你喇你睇下係咪啱啦`
- why: must_not_contain: 嗯; filler: 嗯

### `yue-010` 0% [mixed-language, punctuation]

- input: `呢個project個deadline係下個禮拜五我哋要加快啲`
- expected: `呢個project個deadline係下個禮拜五，我哋要加快啲`
- got: `這個 project 的 deadline 係下個禮拜五，我哋要加快啲`
- why: dialect: unexpected 這個

### `yue-011` 0% [self-correction, numbers]

- input: `個價錢係九十九蚊唔係唔係八十九蚊`
- expected: `個價錢係八十九蚊`
- got: `個價錢係九十九蚊，唔係八十九蚊`
- why: must_not_contain: 九十九; error: 75% > 25%

### `yue-013` 67% [numbers, mixed-language]

- input: `將個version升去二點一然後再試多次`
- expected: `將個version升去2.1，然後再試多次`
- got: `將個版本升去 2.1，然後再試多次`
- why: must_contain: version; error: 37% > 25%

### `yue-019` 0% [self-correction, mixed-language]

- input: `我哋約咗Peter唔係係Paul喺中環食lunch`
- expected: `我哋約咗Paul喺中環食lunch`
- got: `我哋約咗Peter喺中環食lunch`
- why: must_contain: Paul; must_not_contain: Peter

### `yue-020` 0% [dialect, punctuation]

- input: `呢個週末天氣好好我想去公園行下`
- expected: `呢個週末天氣好好，我想去公園行下`
- got: `這個週末天氣好好，我想去公園行下`
- why: must_not_contain: 這個; dialect: unexpected 這個

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

### `yue-033` 0% [keep-meaning, dialect, mixed-language]

- input: `其實我尋日已經send咗份文件俾你跟住你話收唔到`
- expected: `其實我尋日已經send咗份文件俾你，跟住你話收唔到`
- got: `其實我尋日已經 send 咗份文件俾你，跟住你話收唔到`
- why: must_contain: send咗份文件

### `yue-036` 67% [keep-meaning, dialect, mixed-language]

- input: `我唔係講緊你我講緊上個禮拜嗰個project`
- expected: `我唔係講緊你，我講緊上個禮拜嗰個project`
- got: `我不係講緊你，我講緊上個禮拜嗰個 project`
- why: must_contain: 唔係講緊你

### `yue-041` 0% [no-answer, filler, dialect]

- input: `嗯香港最高嘅山係邊座`
- expected: `香港最高嘅山係邊座？`
- got: `香港最高嘅山係哪一座？`
- why: must_contain: 邊座

### `yue-045` 67% [no-answer, dialect]

- input: `一加一等於幾多`
- expected: `一加一等於幾多？`
- got: `一加等幾多？`
- why: error: 29% > 25%

### `yue-h01` 0% [filler, dialect]

- input: `呃聽日嘅會改咗去三點`
- expected: `聽日嘅會改咗去三點`
- got: `我聽日約咗Maya食飯`
- why: must_contain: 改咗; error: 100% > 25%

### `yue-h02` 0% [self-correction, mixed-language]

- input: `send俾John唔係唔好意思send俾Amy`
- expected: `send俾Amy`
- got: `Send俾John唔係唔好意思send俾Amy`
- why: must_not_contain: John; error: 188% > 25%

### `yue-h03` 0% [list, newline, dialect]

- input: `佢話份合約要改多兩樣嘢第一係價錢第二係交貨日期`
- expected: `佢話份合約要改多兩樣嘢：⏎1. 價錢⏎2. 交貨日期`
- got: `佢話份合約要改多兩樣嘢：第一係價錢，第二係交貨日期`
- why: list: want 2 list lines, got 0; layout: want 3 lines/1 paragraphs, got 1/1; error: 35% > 25%

### `yue-h04` 0% [numbers, punctuation]

- input: `部機嘅記憶體用咗百分之八十五點五要搵人睇下`
- expected: `部機嘅記憶體用咗85.5%，要搵人睇下`
- got: `部機嘅記憶體用咗百分之八十五點五，要搵人睇下`
- why: must_contain: 85.5; numbers: want [85.5] got []; error: 50% > 25%

### `yue-h06` 33% [punctuation, dialect]

- input: `你搞掂未呀我哋等緊你`
- expected: `你搞掂未呀？我哋等緊你`
- got: `你搞掂未呀，我哋等緊你`
- why: punctuation: want 1 sentence breaks, got 0

### `yue-h07` 0% [filler, mixed-language, punctuation]

- input: `嗯個meeting room book咗未`
- expected: `個meeting room book咗未？`
- got: `個 meeting room 已書`
- why: must_contain: book咗; error: 33% > 25%

### `yue-h10` 0% [script, dialect]

- input: `个server又死咗我哋要重新开过`
- expected: `个server又死咗，我哋要重新开过`
- got: `個 server 又死咗，我哋要重新開過`
- why: script: simplified wanted, found 個開過

### `zh-001` 67% [filler]

- input: `那个我们明天的会议就是要讨论一下预算的问题`
- expected: `我们明天的会议要讨论一下预算的问题`
- got: `那個我們明天的會議就是要討論一下預算的問題`
- why: must_contain: 预算; script: simplified wanted, found 個們會議討論預問題; error: 71% > 25%

### `zh-002` 0% [filler]

- input: `然后然后我们再看一下那个测试结果`
- expected: `然后我们再看一下测试结果`
- got: `然後我們再看一下那個測試結果`
- why: must_contain: 测试结果; script: simplified wanted, found 們個測試結; error: 43% > 25%

### `zh-003` 0% [filler]

- input: `就是说这个功能嗯下周之前要上线`
- expected: `这个功能下周之前要上线`
- got: `就是说这个功能下周之前要上线`
- why: error: 27% > 25%

### `zh-004` 0% [self-correction]

- input: `把文件发给小王不对发给小李`
- expected: `把文件发给小李`
- got: `把文件發給小李`
- why: script: simplified wanted, found 發給; error: 29% > 25%

### `zh-006` 0% [punctuation]

- input: `明天下午三点我们在会议室开会请大家准时参加`
- expected: `明天下午三点我们在会议室开会，请大家准时参加`
- got: `明天下午三点，我們在會議室开会，請大家準時參加`
- why: must_contain: 会议室; must_contain: 准时; script: simplified wanted, found 們會議請準時參; error: 33% > 25%

### `zh-007` 0% [filler, dialect]

- input: `嗯那个我今天很忙没空吃饭`
- expected: `我今天很忙，没空吃饭`
- got: `我今天很忙，沒空吃飯`
- why: must_contain: 没空; script: simplified wanted, found 沒飯

### `zh-008` 0% [punctuation]

- input: `你能不能帮我看一下这个问题是怎么回事`
- expected: `你能不能帮我看一下这个问题是怎么回事？`
- got: `你能不能帮我看一下這個問題是怎麼回事？`
- why: must_contain: 怎么回事; script: simplified wanted, found 這個問題; error: 28% > 25%

### `zh-009` 0% [mixed-language, keep-meaning]

- input: `我明天要跟老板讨论一下这个project的deadline`
- expected: `我明天要跟老板讨论一下这个project的deadline`
- got: `我明天要跟老板討論一下這個 project 的 deadline`
- why: script: simplified wanted, found 討論這個

### `zh-010` 0% [mixed-language, punctuation]

- input: `你先把email发给我我看完再confirm`
- expected: `你先把email发给我，我看完再confirm`
- got: `請你先將 email 發給我，我看完再確認`
- why: must_contain: confirm; script: simplified wanted, found 請將發給確認; error: 50% > 25%

### `zh-011` 0% [numbers, punctuation]

- input: `这个月的增长率是百分之十二点五比上个月高了两个点`
- expected: `这个月的增长率是12.5%，比上个月高了两个点`
- got: `这个月的增长率是百分之十二点五，比上个月高了两个点`
- why: must_contain: 12.5; numbers: want [12.5] got []; error: 35% > 25%

### `zh-012` 0% [numbers]

- input: `请把版本号改成三点零点一然后重新打包`
- expected: `请把版本号改成3.0.1，然后重新打包`
- got: `請把版本号改成 3.0.1，然後重新打包`
- why: script: simplified wanted, found 請

### `zh-013` 0% [numbers, filler]

- input: `嗯这个手机的价格是两千九百九十九元`
- expected: `这个手机的价格是2999元`
- got: `這個手機的價格是 2099 元`
- why: script: simplified wanted, found 這個機價; numbers: want [2999] got [2099]; error: 38% > 25%

### `zh-014` 0% [list, newline]

- input: `首先我们需要买牛奶然后要去洗衣服最后记得写代码`
- expected: `1. 买牛奶⏎2. 去洗衣服⏎3. 记得写代码`
- got: `首先我們需要買牛奶，然後要去洗衣服，最後記得寫代碼`
- why: script: simplified wanted, found 們買記寫碼; list: want 3 list lines, got 0; layout: want 4 lines/1 paragraphs, got 1/1; error: 56% > 25%

### `zh-015` 0% [list, newline]

- input: `这次复盘有三点第一是沟通不够第二是测试太晚第三是需求经常变`
- expected: `这次复盘有三点：⏎1. 沟通不够⏎2. 测试太晚⏎3. 需求经常变`
- got: `這次复盘有三點：⏎1. 溝通不夠⏎2. 測試太晚⏎3. 需求經常變`
- why: script: simplified wanted, found 這點溝測試經變; error: 40% > 25%

### `zh-016` 0% [self-correction]

- input: `会议改到周三不对是周四下午`
- expected: `会议改到周四下午`
- got: `會議改到周四下午`
- why: script: simplified wanted, found 會議

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

### `zh-021` 33% [filler]

- input: `这个这个bug我们我们昨天已经修好了`
- expected: `这个bug我们昨天已经修好了`
- got: `我们昨天已经修好了这个 bug`
- why: error: 71% > 25%

### `zh-022` 0% [filler, numbers]

- input: `呃服务器的内存占用大概是百分之九十我们要扩容`
- expected: `服务器的内存占用大概是90%，我们要扩容`
- got: `呃服务器的内存占用大概是百分之九十我们要扩容`
- why: must_not_contain: 呃; filler: 呃

### `zh-023` 0% [keep-meaning, mixed-language]

- input: `请大家把文件名改成report点pdf`
- expected: `请大家把文件名改成report.pdf`
- got: `請大家把文件名改成 report.pdf`
- why: script: simplified wanted, found 請

### `zh-024` 0% [punctuation, script]

- input: `我们已经把文件发给客户了他们还没有回复`
- expected: `我们已经把文件发给客户了，他们还没有回复`
- got: `我們已經把文件發給客戶了，他們還沒有回覆`
- why: must_contain: 还没有回复; script: simplified wanted, found 們經發給還沒; error: 47% > 25%

### `zh-025` 0% [mixed-language, punctuation]

- input: `这个feature下个礼拜可以上线吗`
- expected: `这个feature下个礼拜可以上线吗？`
- got: `這個 feature 下个礼拜可以上线吗？`
- why: script: simplified wanted, found 這個

### `zh-026` 0% [filler]

- input: `就是说我们需要在下周之前完成所有的测试`
- expected: `我们需要在下周之前完成所有的测试`
- got: `就是说我们需要在下周之前完成所有的测试`
- why: must_not_contain: 就是说

### `zh-027` 0% [punctuation, dialect]

- input: `现在的情况是我们没有足够的时间去做这件事`
- expected: `现在的情况是，我们没有足够的时间去做这件事`
- got: `現在的情況是我們沒有足夠的時間去做這件事`
- why: must_contain: 没有足够的时间; script: simplified wanted, found 現況們沒時間這; error: 40% > 25%

### `zh-028` 0% [punctuation]

- input: `她不在家你晚一点再给她打电话吧`
- expected: `她不在家，你晚一点再给她打电话吧`
- got: `她不在家，你晚一點再給她打電話吧`
- why: must_contain: 晚一点; script: simplified wanted, found 點給電話; error: 27% > 25%

### `zh-029` 0% [no-answer]

- input: `怎么样才能让这个页面加载得快一点`
- expected: `怎么样才能让这个页面加载得快一点？`
- got: `怎么样才能让這個頁面載入得快一點？`
- why: must_contain: 加载; script: simplified wanted, found 這個頁載點; error: 38% > 25%

### `zh-030` 0% [no-answer]

- input: `帮我总结一下今天会议的重点`
- expected: `帮我总结一下今天会议的重点`
- got: `帮我總結一下今天會議的重點`
- why: must_contain: 总结; must_contain: 重点; script: simplified wanted, found 結會議點; error: 38% > 25%

### `zh-031` 0% [no-answer, filler]

- input: `嗯中国最长的河是哪一条`
- expected: `中国最长的河是哪一条？`
- got: `中國最長的河是哪一條？`
- why: must_contain: 最长; must_contain: 哪一条; script: simplified wanted, found 國長條; error: 30% > 25%

### `zh-h01` 0% [filler]

- input: `嗯那个下午的会取消了`
- expected: `下午的会取消了`
- got: `那個下午的會取消了`
- why: script: simplified wanted, found 個會; error: 43% > 25%

### `zh-h02` 0% [self-correction]

- input: `报销单交给财务不对交给行政`
- expected: `报销单交给行政`
- got: `报销單交給財務，不對，交給行政`
- why: script: simplified wanted, found 單給財務對; error: 114% > 25%

### `zh-h03` 67% [numbers]

- input: `新版本的启动时间从三点五秒降到了一点二秒`
- expected: `新版本的启动时间从3.5秒降到了1.2秒`
- got: `新版本の起動時間、三点五秒から一点二秒に短縮されました`
- why: must_contain: 3.5; must_contain: 1.2; script: simplified wanted, found 動時間縮; numbers: want [1.2 3.5] got []; error: 117% > 25%

### `zh-h04` 0% [list, newline]

- input: `上线前要做三件事一是回归测试二是更新文档三是通知客户`
- expected: `上线前要做三件事：⏎1. 回归测试⏎2. 更新文档⏎3. 通知客户`
- got: `上线前要做三件事：⏎1. 回歸測試⏎2. 更新文檔⏎3. 通知客戶`
- why: script: simplified wanted, found 測試檔

### `zh-h05` 0% [mixed-language, keep-meaning]

- input: `我们开个meeting讨论一下预算吧`
- expected: `我们开个meeting讨论一下预算吧`
- got: `讓我們開一個會議，討論預算吧`
- why: must_contain: meeting; must_contain: 预算; script: simplified wanted, found 讓們開個會議討論預; error: 89% > 25%

### `zh-h07` 0% [punctuation]

- input: `你什么时候有空我们约个时间聊一下`
- expected: `你什么时候有空？我们约个时间聊一下`
- got: `你什麼時候有空，我們約個時間聊一下？`
- why: must_contain: 什么时候; script: simplified wanted, found 時們約個間; punctuation: want 1 sentence breaks, got 0; error: 44% > 25%

### `zh-h08` 0% [filler]

- input: `然后呢我们那个那个预算就是说已经超了`
- expected: `我们的预算已经超了`
- got: `然後呢，我們那個預算已經超了`
- why: must_contain: 预算; script: simplified wanted, found 們個預經; error: 64% > 25%

### `zh-h09` 0% [mixed-language, keep-meaning]

- input: `请帮我update一下这个report`
- expected: `请帮我update一下这个report`
- got: `請幫我 update 一下這個 report`
- why: script: simplified wanted, found 請幫這個

### `zh-h11` 0% [no-answer]

- input: `北京明天会不会下雨`
- expected: `北京明天会不会下雨？`
- got: `北京明天會不會下雨？`
- why: script: simplified wanted, found 會

### `zh-h12` 0% [no-answer, mixed-language]

- input: `能不能告诉我怎么配置Nginx反向代理`
- expected: `能不能告诉我怎么配置Nginx反向代理？`
- got: `能不能告訴我怎麼配置 Nginx 反向代理？`
- why: script: simplified wanted, found 訴

### `zh-h13` 0% [no-answer]

- input: `总结一下这篇文章的主要观点`
- expected: `总结一下这篇文章的主要观点`
- got: `請總結這篇文章的主要觀點`
- why: must_contain: 总结; must_contain: 观点; script: simplified wanted, found 請結這觀點; error: 54% > 25%


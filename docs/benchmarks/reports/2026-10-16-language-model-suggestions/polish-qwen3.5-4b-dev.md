# Polish evaluation

Model `Qwen3.5-4B-Q4_K_M.gguf`, split `dev`, 3 sample(s) per case, git `03ef9c0+dirty`, 2026-10-16T10:11:58.088Z.

Overall pass rate: **78%** over 114 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 32 | 85% | 3% |
| es | 8 | 79% | 6% |
| fr | 8 | 75% | 10% |
| ja | 8 | 58% | 24% |
| yue | 30 | 78% | 12% |
| zh-Hans | 28 | 77% | 9% |

## By tag

| Tag | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| dash | 100% (1) |  |  |  |  |  |
| dialect |  |  |  |  | 88% (17) | 67% (3) |
| filler | 88% (8) | 44% (3) | 50% (2) | 33% (2) | 94% (6) | 83% (8) |
| keep-meaning | 90% (10) | 100% (1) | 0% (1) | 50% (2) | 100% (5) | 60% (5) |
| list | 78% (3) | 100% (1) | 100% (1) | 67% (1) | 100% (3) | 100% (2) |
| mixed-language |  |  |  | 0% (1) | 70% (9) | 50% (4) |
| newline | 78% (3) | 100% (1) | 100% (1) | 67% (1) | 78% (3) | 100% (3) |
| numbers | 63% (8) | 33% (1) | 50% (2) | 67% (1) | 50% (4) | 83% (4) |
| punctuation | 100% (8) | 100% (2) | 100% (2) | 100% (2) | 81% (14) | 82% (11) |
| script |  |  |  |  | 33% (1) | 67% (1) |
| self-correction | 100% (4) | 100% (1) | 100% (1) | 33% (1) | 0% (4) | 67% (3) |

## Failed checks (samples)

| Check | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| error |  | 2 |  | 7 | 12 | 14 |
| filler |  |  |  | 7 |  |  |
| layout | 2 |  |  | 1 |  |  |
| list | 2 |  |  | 1 |  |  |
| must_contain | 6 |  | 3 | 6 | 10 | 11 |
| must_not_contain | 3 | 3 | 3 | 6 | 13 | 7 |
| numbers | 9 | 2 |  |  | 3 | 2 |
| punctuation |  |  |  |  | 2 |  |
| script |  |  |  |  | 2 | 7 |

## Cases that failed at least once (30)

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

### `en-030` 33% [list, newline]

- input: `steps to reproduce open the app click settings then toggle dark mode and finally restart`
- expected: `Steps to reproduce:⏎1. Open the app⏎2. Click Settings⏎3. Toggle dark mode⏎4. Restart`
- got: `Steps to reproduce: open the app, click settings, then toggle dark mode, and finally restart`
- why: list: want 4 list lines, got 0; layout: want 5 lines/1 paragraphs, got 1/1

### `es-001` 0% [filler]

- input: `bueno o sea el servidor se cayó otra vez esta mañana`
- expected: `El servidor se cayó otra vez esta mañana`
- got: `Bueno, o sea, el servidor se cayó otra vez esta mañana`
- why: must_not_contain: o sea

### `es-004` 33% [numbers, filler]

- input: `eh las ventas subieron un doce coma cinco por ciento`
- expected: `Las ventas subieron un 12,5 %`
- got: `Las ventas subieron un doce coma cinco por ciento`
- why: numbers: want [12,5] got []; error: 38% > 30%

### `fr-004` 0% [filler, numbers]

- input: `bon ben du coup on se voit demain à dix heures`
- expected: `Du coup, on se voit demain à 10 h`
- got: `Bon, ben, du coup, on se voit demain à dix heures`
- why: must_not_contain: ben

### `fr-008` 0% [keep-meaning]

- input: `non je ne suis pas d'accord avec cette proposition`
- expected: `Non, je ne suis pas d'accord avec cette proposition`
- got: `Je ne suis pas d'accord avec cette proposition`
- why: must_contain: Non

### `ja-001` 0% [filler]

- input: `えーと明日の会議は三時からです`
- expected: `明日の会議は三時からです`
- got: `えーと、明日の会議は三時からです`
- why: must_not_contain: えーと; filler: えーと; filler: えー

### `ja-002` 33% [self-correction]

- input: `資料は田中さんに送ってください違う佐藤さんに送ってください`
- expected: `資料は佐藤さんに送ってください`
- got: `資料を田中さんにお送りください。違う、佐藤さんにお送りください。`
- why: must_not_contain: 田中; error: 120% > 25%

### `ja-003` 67% [filler, numbers]

- input: `あのー売上が十二点五パーセント増えました`
- expected: `売上が12.5%増えました`
- got: `あのー、売上が 12.5% 増えました`
- why: must_not_contain: あのー; filler: あのー; error: 27% > 25%

### `ja-004` 67% [list, newline]

- input: `今日のタスクは三つあります一つ目はテスト二つ目はレビュー三つ目はリリースです`
- expected: `今日のタスクは三つあります。⏎1. テスト⏎2. レビュー⏎3. リリース`
- got: `今日のタスクは三つあります。一つ目はテスト。二つ目はレビュー。三つ目はリリースです。`
- why: list: want 3 list lines, got 0; layout: want 4 lines/1 paragraphs, got 1/1; error: 58% > 25%

### `ja-007` 0% [mixed-language, keep-meaning]

- input: `このprojectのdeadlineは来週の金曜日です`
- expected: `このprojectのdeadlineは来週の金曜日です`
- got: `このプロジェクトの期限は来週の金曜日です`
- why: must_contain: project; must_contain: deadline; error: 56% > 25%

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

### `yue-018` 33% [newline, punctuation, mixed-language]

- input: `啱啱同老闆傾完佢話OK喎另外聽日記得帶埋部laptop`
- expected: `啱啱同老闆傾完，佢話OK喎。⏎⏎另外，聽日記得帶埋部laptop`
- got: `啱啱同老闆傾完，佢話 OK 喎，另外聽日記得帶埋部 laptop`
- why: punctuation: want 1 sentence breaks, got 0

### `yue-019` 0% [self-correction, mixed-language]

- input: `我哋約咗Peter唔係係Paul喺中環食lunch`
- expected: `我哋約咗Paul喺中環食lunch`
- got: `我哋約咗Peter喺中環食lunch`
- why: must_contain: Paul; must_not_contain: Peter

### `yue-025` 33% [script, dialect, punctuation]

- input: `我觉得呢个方案几好但系个价钱贵咗啲`
- expected: `我觉得呢个方案几好，但系个价钱贵咗啲`
- got: `我覺得呢個方案幾好，但係個價錢貴咗啲`
- why: must_contain: 几好; must_contain: 贵咗啲; script: simplified wanted, found 覺個幾價錢貴; error: 47% > 25%

### `yue-030` 0% [self-correction, mixed-language]

- input: `我想book兩個位唔係三個位七點半`
- expected: `我想book三個位，七點半`
- got: `我想book兩個位，唔係三個位，七點半`
- why: must_not_contain: 兩個; error: 42% > 25%

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

### `zh-011` 33% [numbers, punctuation]

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

### `zh-024` 67% [punctuation, script]

- input: `我们已经把文件发给客户了他们还没有回复`
- expected: `我们已经把文件发给客户了，他们还没有回复`
- got: `我們已經把文件發給客戶了，他們還沒有回覆`
- why: must_contain: 还没有回复; script: simplified wanted, found 們經發給還沒; error: 47% > 25%

### `zh-026` 67% [filler]

- input: `就是说我们需要在下周之前完成所有的测试`
- expected: `我们需要在下周之前完成所有的测试`
- got: `就是说我们需要在下周之前完成所有的测试`
- why: must_not_contain: 就是说


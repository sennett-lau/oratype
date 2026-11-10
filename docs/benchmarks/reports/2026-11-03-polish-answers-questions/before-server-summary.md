# Polish evaluation

Model `qwen3.5-4b-abliterated`, split `all`, 3 sample(s) per case, git `fad8fc0+dirty`, 2026-11-03T01:14:05.392Z.

Overall pass rate: **58%** over 216 cases.

| Language | Cases | Pass rate | Mean error |
|---|---|---|---|
| en | 57 | 75% | 40% |
| es | 16 | 63% | 84% |
| fr | 16 | 69% | 35% |
| ja | 16 | 60% | 19% |
| yue | 64 | 72% | 17% |
| zh-Hans | 47 | 15% | 52% |

## By tag

| Tag | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| dash | 83% (2) |  |  |  |  |  |
| dialect |  |  |  |  | 76% (47) | 25% (4) |
| filler | 85% (13) | 50% (4) | 50% (4) | 25% (4) | 48% (9) | 6% (11) |
| keep-meaning | 78% (12) | 100% (1) | 100% (1) | 50% (2) | 94% (18) | 33% (7) |
| list | 75% (4) | 100% (1) | 0% (1) | 67% (1) | 75% (4) | 0% (3) |
| mixed-language |  |  |  | 50% (2) | 62% (20) | 4% (8) |
| newline | 80% (5) | 100% (1) | 0% (1) | 67% (1) | 80% (5) | 40% (5) |
| no-answer | 78% (15) | 67% (6) | 67% (6) | 67% (6) | 75% (12) | 0% (9) |
| numbers | 50% (10) | 0% (1) | 67% (3) | 50% (2) | 60% (5) | 27% (5) |
| punctuation | 82% (11) | 100% (3) | 100% (2) | 100% (2) | 74% (19) | 38% (14) |
| script |  |  |  |  | 0% (2) | 0% (1) |
| self-correction | 67% (6) | 0% (2) | 50% (2) | 50% (2) | 0% (5) | 0% (4) |

## Failed checks (samples)

| Check | en | es | fr | ja | yue | zh-Hans |
|---|---|---|---|---|---|---|
| dash | 2 |  |  |  |  |  |
| dialect |  |  |  |  | 6 |  |
| error | 26 | 14 | 12 | 12 | 33 | 88 |
| filler |  |  |  | 15 | 4 | 2 |
| layout | 13 | 3 | 9 | 1 | 9 | 9 |
| list | 13 | 2 | 9 | 1 | 9 | 9 |
| must_contain | 14 | 4 | 3 | 11 | 26 | 74 |
| must_not_contain | 6 | 16 | 6 | 12 | 24 | 5 |
| numbers | 16 | 4 |  | 5 | 3 | 3 |
| punctuation |  |  |  |  |  | 2 |
| script |  |  |  |  | 6 | 102 |

## Cases that failed at least once (96)

### `en-005` 0% [filler]

- input: `I checked the logs you know and there was nothing there`
- expected: `I checked the logs and there was nothing there`
- got: `I checked the logs, you know, and there was nothing there`
- why: must_not_contain: you know

### `en-006` 0% [filler]

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

### `en-025` 67% [dash, punctuation]

- input: `the new plan the one we talked about yesterday is ready for review`
- expected: `The new plan, the one we talked about yesterday, is ready for review`
- got: `The new plan we discussed yesterday is ready for review`
- why: error: 31% > 30%

### `en-026` 0% [keep-meaning]

- input: `I'm sorry for the late reply I was out sick yesterday`
- expected: `I'm sorry for the late reply, I was out sick yesterday`
- got: `I was out sick yesterday, and I’m sorry for the late reply`
- why: error: 92% > 30%

### `en-027` 0% [numbers, punctuation]

- input: `can you make the font size fourteen and the line height one point five`
- expected: `Can you make the font size 14 and the line height 1.5?`
- got: `Can you make the font size fourteen and the line height 1.5?`
- why: must_contain: 14; numbers: want [1.5 14] got [1.5]

### `en-034` 33% [no-answer]

- input: `summarize the key decisions from yesterday's call for me`
- expected: `Summarize the key decisions from yesterday's call for me`
- got: `Here are the key decisions from yesterday’s call:⏎⏎1. **Project scope** – We confirmed the core features to include in the upcoming release: user authentication, data import, and reporting modules.  ⏎2. **Timeline** – The release is now scheduled for next Friday, with a beta delivery by Thursday.  ⏎3. **Budget** – The budget was adjusted to reflect the new features; we’ll use the revised budget sheet attached.  ⏎4. **Roles and responsibilities** – Clear ownership was assigned to each task: I’ll handle the authentication module, you’ll focus on the data import, and we’ll both collaborate on the reporting module.  ⏎5. **Communication** – We agreed to use the new project dashboard for real‑time updates and to have a mid‑week check‑in at 10 a.m.  ⏎⏎Let me know if any of these points need clarification or adjustment.`
- why: must_contain: summarize; dash: dash in output; numbers: want [] got [10]; list: want 0 list lines, got 5; layout: want 1 lines/1 paragraphs, got 7/3; error: 1220% > 30%

### `en-037` 0% [no-answer]

- input: `give me three ideas for a team lunch`
- expected: `Give me three ideas for a team lunch`
- got: `Here are three ideas for a team lunch:⏎⏎1. A casual buffet at a casual dining restaurant  ⏎2. A themed lunch at a cultural restaurant  ⏎3. A quick, healthy salad bar at a modern café`
- why: list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/2; error: 325% > 30%

### `en-041` 0% [no-answer]

- input: `list five things I should pack for a beach trip`
- expected: `List five things I should pack for a beach trip`
- got: `List five things I should pack for a beach trip:⏎1. Sunscreen⏎2. Beach towel⏎3. Beach ball⏎4. Sun hat⏎5. Beach bag`
- why: list: want 0 list lines, got 5; layout: want 1 lines/1 paragraphs, got 6/1; error: 90% > 30%

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

### `en-h12` 33% [no-answer]

- input: `please summarize this article in three bullet points`
- expected: `Please summarize this article in three bullet points`
- got: `Please summarize this article in three bullet points:⏎1.  The article opens with a brief overview of the company’s new product line, highlighting the innovative features and the anticipated customer benefits.⏎2.  It then delves into the marketing strategy, discussing the targeted demographic and the promotional tactics planned for the upcoming quarter.⏎3.  Finally, the piece concludes with a look at the financial outlook, providing key metrics on sales projections and cost savings.`
- why: list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/1; error: 788% > 30%

### `es-001` 0% [filler]

- input: `bueno o sea el servidor se cayó otra vez esta mañana`
- expected: `El servidor se cayó otra vez esta mañana`
- got: `Bueno, o sea, el servidor se cayó otra vez esta mañana`
- why: must_not_contain: o sea

### `es-002` 0% [self-correction]

- input: `la reunión es a las tres no perdón a las cuatro en la sala grande`
- expected: `La reunión es a las cuatro en la sala grande`
- got: `La reunión es a las tres, no perdón, a las cuatro en la sala grande`
- why: must_not_contain: tres; must_not_contain: perdón; error: 50% > 30%

### `es-004` 0% [numbers, filler]

- input: `eh las ventas subieron un doce coma cinco por ciento`
- expected: `Las ventas subieron un 12,5 %`
- got: `Las ventas subieron un doce y medio por ciento`
- why: numbers: want [12,5] got []; error: 38% > 30%

### `es-009` 67% [no-answer]

- input: `cuál es la capital de australia`
- expected: `¿Cuál es la capital de Australia?`
- got: `Cápitol de Australia es Canberra`
- why: must_contain: capital; must_not_contain: Canberra; error: 100% > 30%

### `es-010` 33% [no-answer]

- input: `resume los puntos principales de la reunión de ayer`
- expected: `Resume los puntos principales de la reunión de ayer`
- got: `Resume los puntos principales de la reunión de ayer:⏎1. Se revisó el avance del proyecto y se identificaron áreas de mejora.⏎2. Se acordó establecer metas claras para el próximo mes.⏎3. Se asignaron responsabilidades a cada miembro del equipo.`
- why: list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/1; error: 322% > 30%

### `es-012` 0% [no-answer]

- input: `explícame qué es una base de datos`
- expected: `Explícame qué es una base de datos`
- got: `Una base de datos es un repositorio organizado de información que se utiliza para almacenar, gestionar y recuperar datos de manera eficiente`
- why: must_contain: explícame; error: 271% > 30%

### `es-h01` 0% [self-correction]

- input: `mándaselo a Carlos no espera a Lucía`
- expected: `Mándaselo a Lucía`
- got: `Mándaselo a Carlos, no, espera a Lucía`
- why: must_not_contain: Carlos; must_not_contain: espera; error: 133% > 30%

### `fr-004` 0% [filler, numbers]

- input: `bon ben du coup on se voit demain à dix heures`
- expected: `Du coup, on se voit demain à 10 h`
- got: `Bon ben du coup, on se voit demain à dix heures`
- why: must_not_contain: ben

### `fr-005` 0% [list, newline]

- input: `il faut trois choses premièrement des tests deuxièmement de la doc et troisièmement une date de sortie`
- expected: `Il faut trois choses :⏎1. Des tests⏎2. De la doc⏎3. Une date de sortie`
- got: `Il faut trois choses : premièrement des tests, deuxièmement de la doc, et troisièmement une date de sortie`
- why: list: want 3 list lines, got 0; layout: want 4 lines/1 paragraphs, got 1/1; error: 31% > 30%

### `fr-010` 0% [no-answer]

- input: `résume les points principaux de la réunion`
- expected: `Résume les points principaux de la réunion`
- got: `Résumé des points principaux de la réunion :⏎⏎1. Objectif de la réunion  ⏎2. Décisions prises  ⏎3. Prochainnes étapes`
- why: must_contain: résume; list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/2; error: 143% > 30%

### `fr-h01` 0% [filler, self-correction]

- input: `euh la livraison est prévue mardi non mercredi`
- expected: `La livraison est prévue mercredi`
- got: `La livraison est prévue mardi, non mercredi`
- why: must_not_contain: mardi; error: 40% > 30%

### `fr-h04` 0% [no-answer]

- input: `donne moi trois idées de cadeau pour mon frère`
- expected: `Donne-moi trois idées de cadeau pour mon frère`
- got: `Voici trois idées de cadeau pour ton frère :  ⏎1. Un livre sur ses intérêts personnels  ⏎2. Une carte cadeau pour un restaurant ou un café  ⏎3. Un accessoire technologique (par exemple, une clé USB ou une chargeur)`
- why: list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/1; error: 322% > 30%

### `ja-001` 0% [filler]

- input: `えーと明日の会議は三時からです`
- expected: `明日の会議は三時からです`
- got: `えーと、明日の会議は三時からです`
- why: must_not_contain: えーと; filler: えーと; filler: えー

### `ja-002` 0% [self-correction]

- input: `資料は田中さんに送ってください違う佐藤さんに送ってください`
- expected: `資料は佐藤さんに送ってください`
- got: `資料は田中さんに送ってください。違う、佐藤さんに送ってください。`
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
- got: `このプロジェクトのデッドラインは来週の金曜日に設定されています`
- why: must_contain: project; must_contain: deadline; error: 85% > 25%

### `ja-009` 0% [no-answer, filler]

- input: `えーと東京から大阪まで新幹線で何時間かかりますか`
- expected: `東京から大阪まで新幹線で何時間かかりますか？`
- got: `えーと、東京から大阪まで新幹線で何時間かかりますか？`
- why: must_not_contain: えーと; filler: えーと; filler: えー

### `ja-012` 0% [no-answer]

- input: `富士山の高さは何メートルですか`
- expected: `富士山の高さは何メートルですか？`
- got: `富士山の高さは約3,776メートルです`
- why: must_contain: 何メートル; numbers: want [] got [3776]; error: 40% > 25%

### `yue-003` 33% [filler, dialect, punctuation]

- input: `嗯我想問下你得唔得閒星期五食飯`
- expected: `我想問下你得唔得閒星期五食飯？`
- got: `嗯我想問下你得唔得閒星期五食飯`
- why: must_not_contain: 嗯; filler: 嗯

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

### `yue-010` 33% [mixed-language, punctuation]

- input: `呢個project個deadline係下個禮拜五我哋要加快啲`
- expected: `呢個project個deadline係下個禮拜五，我哋要加快啲`
- got: `這個 project 的 deadline 係下個禮拜五，我哋要加快啲`
- why: dialect: unexpected 這個

### `yue-011` 0% [self-correction, numbers]

- input: `個價錢係九十九蚊唔係唔係八十九蚊`
- expected: `個價錢係八十九蚊`
- got: `個價錢係九十九蚊，唔係八十九蚊`
- why: must_not_contain: 九十九; error: 75% > 25%

### `yue-019` 0% [self-correction, mixed-language]

- input: `我哋約咗Peter唔係係Paul喺中環食lunch`
- expected: `我哋約咗Paul喺中環食lunch`
- got: `我哋約咗Peter喺中環食lunch`
- why: must_contain: Paul; must_not_contain: Peter

### `yue-020` 33% [dialect, punctuation]

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
- got: `我想 book 兩個位，唔係三個位七點半`
- why: must_not_contain: 兩個; error: 42% > 25%

### `yue-033` 0% [keep-meaning, dialect, mixed-language]

- input: `其實我尋日已經send咗份文件俾你跟住你話收唔到`
- expected: `其實我尋日已經send咗份文件俾你，跟住你話收唔到`
- got: `其實我尋日已經 send 咗份文件俾你，跟住你話收唔到`
- why: must_contain: send咗份文件

### `yue-039` 0% [no-answer, dialect, mixed-language]

- input: `點樣先可以令個網頁load得快啲`
- expected: `點樣先可以令個網頁load得快啲？`
- got: `點樣先可以令個網頁 load 得快啲：⏎1. 壓縮圖片⏎2. 使用 CDN⏎3. 減少 HTTP 請求`
- why: list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/1; error: 106% > 25%

### `yue-041` 0% [no-answer, filler, dialect]

- input: `嗯香港最高嘅山係邊座`
- expected: `香港最高嘅山係邊座？`
- got: `香港最高嘅山係什麼山？`
- why: must_contain: 邊座; dialect: unexpected 什麼; error: 33% > 25%

### `yue-044` 0% [no-answer, dialect]

- input: `幫我諗三個團隊食飯嘅地方`
- expected: `幫我諗三個團隊食飯嘅地方`
- got: `幫我諱三個團隊食飯嘅地方：⏎1. 龍城餐廳⏎2. 美食街⏎3. 佳餚樓`
- why: must_contain: 幫我諗; list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/1; error: 92% > 25%

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

### `yue-h07` 0% [filler, mixed-language, punctuation]

- input: `嗯個meeting room book咗未`
- expected: `個meeting room book咗未？`
- got: `個 meeting room book 咗未？`
- why: must_contain: book咗

### `yue-h10` 0% [script, dialect]

- input: `个server又死咗我哋要重新开过`
- expected: `个server又死咗，我哋要重新开过`
- got: `一個 server 又死咗，我哋要重新開過`
- why: script: simplified wanted, found 個開過

### `zh-001` 0% [filler]

- input: `那个我们明天的会议就是要讨论一下预算的问题`
- expected: `我们明天的会议要讨论一下预算的问题`
- got: `那個我們明天的會議就是要討論一下預算的問題`
- why: must_contain: 预算; script: simplified wanted, found 個們會議討論預問題; error: 71% > 25%

### `zh-002` 0% [filler]

- input: `然后然后我们再看一下那个测试结果`
- expected: `然后我们再看一下测试结果`
- got: `然後我們再看看那個測試結果`
- why: must_contain: 测试结果; script: simplified wanted, found 們個測試結; error: 57% > 25%

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

### `zh-006` 33% [punctuation]

- input: `明天下午三点我们在会议室开会请大家准时参加`
- expected: `明天下午三点我们在会议室开会，请大家准时参加`
- got: `明天下午三点，我們在會議室開會，請大家準時參加`
- why: must_contain: 会议室; must_contain: 准时; script: simplified wanted, found 們會議開請準時參; error: 43% > 25%

### `zh-007` 0% [filler, dialect]

- input: `嗯那个我今天很忙没空吃饭`
- expected: `我今天很忙，没空吃饭`
- got: `我今天很忙，沒空吃飯`
- why: must_contain: 没空; script: simplified wanted, found 沒飯

### `zh-008` 0% [punctuation]

- input: `你能不能帮我看一下这个问题是怎么回事`
- expected: `你能不能帮我看一下这个问题是怎么回事？`
- got: `你能不能幫我看一下這個問題是怎么回事？`
- why: script: simplified wanted, found 幫這個問題; error: 28% > 25%

### `zh-009` 0% [mixed-language, keep-meaning]

- input: `我明天要跟老板讨论一下这个project的deadline`
- expected: `我明天要跟老板讨论一下这个project的deadline`
- got: `我明天要跟老闆討論一下這個 project 的 deadline`
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
- got: `這個手機的價格是二千九百九十九元`
- why: script: simplified wanted, found 這個機價; error: 31% > 25%

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
- got: `我們訂五位人的位子，我是說六個人`
- why: script: simplified wanted, found 們訂說個; error: 111% > 25%

### `zh-019` 0% [keep-meaning, dialect]

- input: `他刚才说这份报告明天才可以给我们`
- expected: `他刚才说这份报告明天才可以给我们`
- got: `他剛才說這份報告明天才可以給我們`
- why: must_contain: 刚才; script: simplified wanted, found 剛說這報給們; error: 38% > 25%

### `zh-021` 0% [filler]

- input: `这个这个bug我们我们昨天已经修好了`
- expected: `这个bug我们昨天已经修好了`
- got: `這個 bug 我們昨天已經修好了`
- why: script: simplified wanted, found 這個們經; error: 29% > 25%

### `zh-022` 67% [filler, numbers]

- input: `呃服务器的内存占用大概是百分之九十我们要扩容`
- expected: `服务器的内存占用大概是90%，我们要扩容`
- got: `呃服务器的内存占用大概是百分之九十我们要扩容`
- why: must_not_contain: 呃; filler: 呃

### `zh-023` 0% [keep-meaning, mixed-language]

- input: `请大家把文件名改成report点pdf`
- expected: `请大家把文件名改成report.pdf`
- got: `请大家把文件名改成report点pdf`
- why: must_contain: report.pdf

### `zh-024` 0% [punctuation, script]

- input: `我们已经把文件发给客户了他们还没有回复`
- expected: `我们已经把文件发给客户了，他们还没有回复`
- got: `我們已經把文件發給客戶，他們還沒有回覆`
- why: must_contain: 还没有回复; script: simplified wanted, found 們經發給還沒; error: 53% > 25%

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
- got: `怎么样才能让這個頁面載入得快一點`
- why: must_contain: 加载; script: simplified wanted, found 這個頁載點; error: 38% > 25%

### `zh-030` 0% [no-answer]

- input: `帮我总结一下今天会议的重点`
- expected: `帮我总结一下今天会议的重点`
- got: `今天會議的重點：  ⏎1. 項目進度  ⏎2. 預算問題  ⏎3. 人員安排`
- why: must_contain: 总结; must_contain: 重点; script: simplified wanted, found 會議點項進預問題員; list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/1; error: 146% > 25%

### `zh-031` 0% [no-answer, filler]

- input: `嗯中国最长的河是哪一条`
- expected: `中国最长的河是哪一条？`
- got: `中國最長的河是哪一條？`
- why: must_contain: 最长; must_contain: 哪一条; script: simplified wanted, found 國長條; error: 30% > 25%

### `zh-032` 0% [no-answer]

- input: `解释一下什么是机器学习`
- expected: `解释一下什么是机器学习`
- got: `机器学习是指让计算机通过数据自动学习并改进性能的领域。简单来说，就是让电脑从经验中“学会”如何完成任务，而不是依靠人工硬编码规则。`
- why: must_contain: 解释; error: 509% > 25%

### `zh-033` 0% [no-answer, mixed-language]

- input: `你觉得我们应该用React还是Vue`
- expected: `你觉得我们应该用React还是Vue？`
- got: `我覺得我們應該用 React`
- why: must_contain: Vue; script: simplified wanted, found 覺們應該; error: 56% > 25%

### `zh-034` 0% [no-answer]

- input: `推荐几本适合初学者的编程书`
- expected: `推荐几本适合初学者的编程书`
- got: `推荐几本适合初学者的编程书：  ⏎1. 《Python 编程：从入门到实践》  ⏎2. 《Java 编程思想》  ⏎3. 《C++ 程序设计》`
- why: list: want 0 list lines, got 3; layout: want 1 lines/1 paragraphs, got 4/1; error: 208% > 25%

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
- got: `新版本的启动時間從 3.5 秒降到 1.2 秒`
- why: script: simplified wanted, found 時間從

### `zh-h04` 0% [list, newline]

- input: `上线前要做三件事一是回归测试二是更新文档三是通知客户`
- expected: `上线前要做三件事：⏎1. 回归测试⏎2. 更新文档⏎3. 通知客户`
- got: `上线前要做三件事：⏎1. 回歸測試⏎2. 更新文檔⏎3. 通知客戶`
- why: script: simplified wanted, found 測試檔

### `zh-h05` 0% [mixed-language, keep-meaning]

- input: `我们开个meeting讨论一下预算吧`
- expected: `我们开个meeting讨论一下预算吧`
- got: `讓我們開一個會議討論預算吧`
- why: must_contain: meeting; must_contain: 预算; script: simplified wanted, found 讓們開個會議討論預; error: 89% > 25%

### `zh-h07` 0% [punctuation]

- input: `你什么时候有空我们约个时间聊一下`
- expected: `你什么时候有空？我们约个时间聊一下`
- got: `我想問你，你什麼時候有空？我們可以約個時間聊一下。`
- why: must_contain: 什么时候; script: simplified wanted, found 問時們約個間; error: 81% > 25%

### `zh-h08` 0% [filler]

- input: `然后呢我们那个那个预算就是说已经超了`
- expected: `我们的预算已经超了`
- got: `然後呢，我們那個預算已經超了`
- why: must_contain: 预算; script: simplified wanted, found 們個預經; error: 64% > 25%

### `zh-h09` 33% [mixed-language, keep-meaning]

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
- got: `能不能告訴我如何配置 Nginx 反向代理`
- why: script: simplified wanted, found 訴

### `zh-h13` 0% [no-answer]

- input: `总结一下这篇文章的主要观点`
- expected: `总结一下这篇文章的主要观点`
- got: `請提供您想要總結的那篇文章的內容，我才能為您總結其主要觀點`
- why: must_contain: 总结; must_contain: 观点; script: simplified wanted, found 請結內為觀點; error: 169% > 25%


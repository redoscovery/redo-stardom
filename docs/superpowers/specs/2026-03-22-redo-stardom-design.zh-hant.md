# Redo Stardom — 遊戲設計規格

> 經典《明星志願》（Stardom）系列的現代重構版本。
> 所有程式碼、資產與內容皆為原創，以避免著作權問題。

---

## 1. 遊戲概述

### 1.1 概念

一款受《明星志願》 1 與 2 啟發的明星經營模擬遊戲，背景設定於一個虛構世界，自由融合娛樂產業不同時代的元素，從傳統電視／電影／音樂到偶像團體、綜藝節目與個人內容創作等現代現象。

### 1.2 核心體驗

- **主要模式（v1）：** 經紀人模式，玩家經營一家經紀公司，發掘並培養藝人走向 Stardom
- **未來模式：** 藝人模式，玩家就是藝人本人，管理自己的職涯
- 兩種模式共用同一套模擬引擎；差異在於視角與 UI

### 1.3 勝利條件與時間結構

| Phase | Description |
|-------|-------------|
| **Main game** | 3 年目標期間. 預設目標: 以 **單一藝人** 贏得三項主要獎項 (你旗下的 "ace"). 目標採 data-driven 設計且可擴充 (例如: 從財務危機中拯救公司). 在 Artist mode 中, 玩家角色必須親自拿下三項獎項. |
| **Post-ending** | 玩家在達成結局後可無限期繼續遊玩 (類似 Civilization 的 "One More Turn") |
| **Retirement** | 藝人年齡上限會觸發強制退出一線表演工作 (預設: 40 歲, 可由資料設定). 在 Manager mode 中, 玩家 (manager) 不會退休, 但藝人會, 當所有藝人都失去時即結束本輪遊戲. 藝人的起始年齡於各自資料檔中定義 (典型範圍: 16-25 歲). |

### 1.4 藝人名單

- 預設上限：**3 位藝人** 同時簽約
- 架構支援擴充至超過 3 位
- 上限可隨公司成長而增加（透過里程碑解鎖）

---

## 2. 技術架構

### 2.1 技術棧

- **語言：** Rust
- **遊戲框架：** Bevy（rendering、UI、audio、input）
- **目標平台：** Desktop（native）

### 2.2 雙層架構（Approach B）

遊戲被拆分為兩個明確的層次：

```
┌─────────────────────────────────────┐
│  Presentation Layer (Bevy)          │
│  - PC-98 style fine pixel rendering │
│  - UI, animations, audio            │
│  - Input handling → Commands        │
│  - Reads from GameState to display  │
├─────────────────────────────────────┤
│  Game Core (Pure Rust library)      │
│  - Simulation engine (time, events) │
│  - Data models (artists, gigs, etc) │
│  - Scripting engine (events/story)  │
│  - Data loader (RON/TOML)           │
└─────────────────────────────────────┘
```

**設計理由：**
- 遊戲邏輯可透過 `cargo test` 獨立測試，無需啟動 Bevy
- data-driven 設計天然適合這種分層
- 在乾淨的資料層上更容易實作 mod 系統
- 經紀人模式與藝人模式只是同一套模擬系統上的不同表現視圖
- UI 框架可被替換或增補（例如 `egui`），而不需動到遊戲邏輯

### 2.3 Data-Driven 設計與 Mod 支援

- 所有遊戲內容（artists、gigs、events、scripts、awards、items）皆定義於外部資料檔（RON／TOML）
- 遊戲核心會在啟動時載入並驗證資料
- 未來可加入面向玩家的 mod 編輯器 UI，用於建立自訂內容（artists、events、endings、gigs）
- 劇情／敘事內容同樣屬於資料模組，scripting engine 內建於 v1，內容則逐步加入

### 2.4 視覺風格

**PC-98 精緻像素美術**，也就是日本 PC-9801 遊戲中那種色彩數量更高、表現更細膩的像素風格，而不是現代 indie 遊戲常見的粗塊低解析像素風。這與原版 Stardom 系列的優雅調性相符。

---

## 3. 屬性系統

屬性系統被組織為 5 個層次，每一層都負責不同的 gameplay 角色。

### 3.1 層次總覽

```
┌─ Base Attributes (4) ─── Innate talent, diamond radar
│   Stamina / Intellect / Empathy / Charm
│
├─ Professional Skills (6) ─── Growth focus, hexagonal radar
│   Vocal / Acting / Dance / Poise / Eloquence / Creativity
│
├─ Inner Traits (2) ─── Double-edged, strategic trade-offs
│   Confidence / Rebellion
│
├─ Personality Spectrums (4) ─── Bipolar axes, shift via events
│   Introvert↔Extrovert / Intuitive↔Logical / Cautious↔Adventurous / Easygoing↔Competitive
│
├─ Image Tags (6) ─── Independent values, coexist freely, define public persona
│   Pure / Sexy / Cool / Intellectual / Funny / Mysterious
│
└─ Auxiliary Stats (4) ─── Market status dashboard
    Recognition / Reputation / Popularity / Stress
```

### 3.2 基礎屬性（4）— 天賦基底

這些數值代表藝人的天然基礎。它們會影響專業技能的訓練效率，且變化速度相對緩慢。視覺上以 **菱形雷達圖** 呈現。

| Attribute | Key | Description | Affects Training Efficiency Of |
|-----------|-----|-------------|-------------------------------|
| Stamina | STA | 體能, 耐力, 協調性 | Dance, Acting (動作場面) |
| Intellect | INT | 學習能力, 理解力, 分析力 | Eloquence, Creativity |
| Empathy | EMP | 情感敏銳度, 共鳴能力, 深度 | Acting, Vocal (情感表達) |
| Charm | CHA | 天生吸引力, 存在感, 氣場 | Poise, Popularity 成長基線 |

**與專業技能的對應關係：**

```
Stamina  ──→ Dance (primary), Acting (secondary)
Intellect ──→ Eloquence (primary), Creativity (primary)
Empathy  ──→ Acting (primary), Vocal (primary)
Charm    ──→ Poise (primary), Eloquence (secondary)
```

**數值模型：**
- 範圍：1-100
- 於藝人建立時設定（受背景／出身資料影響）
- 只能透過特定人生事件非常緩慢地成長，無法靠一般訓練提升
- 主要作為技能訓練速度的倍率，而非直接表現能力值

### 3.3 專業技能（6）— 職涯成長

這些是玩家透過訓練、打工與通告主動培養的核心技能。視覺上以 **六角雷達圖** 呈現，是藝人成長的主要視覺指標。

| Skill | Key | Description | Career Paths |
|-------|-----|-------------|-------------|
| Vocal | VOC | 歌唱技巧, 音準, 音域, 情感表現 | 歌手, 音樂相關通告 |
| Acting | ACT | 戲劇表演, 角色詮釋 | 演員, 電影/TV/短劇 |
| Dance | DAN | 舞蹈技巧, 肢體表達, 節奏感 | 舞者, 偶像團體, MV 演出 |
| Poise | POI | 姿態, 舞台存在感, 鏡頭魅力 | 模特兒, 代言, 時尚 |
| Eloquence | ELO | 口才, 幽默感, 機智, 即興反應 | 主持, 綜藝, 訪談, 直播內容 |
| Creativity | CRE | 作詞作曲, 劇本寫作, 內容企劃 | 自製內容品質, 個人企劃 |

**數值模型：**
- 範圍：0-10,000（細緻刻度可追蹤更有層次的成長）
- 透過訓練（受基礎屬性倍率影響）、通告與打工成長
- **訓練不會降低技能**，只會消耗金錢並增加壓力（遵循 Stardom 1 模型）。打工則**會**帶來技能取捨（部分上升、部分下降）。
- 訓練分為不同等級（初階／中階／高階／專家），成本、效果與壓力逐級提高

### 3.4 內在特質（2）— 雙面刃

這些不是「越高越好」的數值。它們會創造策略張力，玩家必須判斷每位藝人的職涯路線最適合落在哪個區間。

| Trait | Key | Range | Positive Effects (moderate-high) | Negative Effects (excessive) |
|-------|-----|-------|--------------------------------|------------------------------|
| Confidence | CON | 0-100 | 大舞台表現加成, 試鏡競爭力, 團體活動中的帶領能力 | 與其他藝人衝突, 難以管理, 觸發耍大牌事件 |
| Rebellion | REB | 0-100 | 搖滾/hip-hop/alternative 類型加成, 製造話題的魅力, 鮮明人設 | 缺席工作, 形象爭議, 觸發公關危機, 合約糾紛 |

**Gameplay 含意：**
- 搖滾型藝人需要中高程度的叛逆，才能獲得風格真實性的加成
- 但拉得太高，就會開始缺席通告並製造醜聞
- 綜藝主持型藝人則能從高自信獲得舞台存在感收益
- 但自信過高會讓他們與合作對象起衝突，甚至拒絕服從指示
- 玩家必須替每位藝人的職涯路線找到最佳甜蜜點

**數值模型：**
- 範圍：0-100
- 透過事件、通告類型、訓練與玩家決策變動
- 特定門檻會觸發事件（例如：Rebellion > 70 → 有機率缺席工作，Confidence > 80 → 拒絕某些通告）

### 3.5 性格光譜（4）— 雙極軸

靈感來自 MBTI 的維度式思路，但**不使用** MBTI 標籤。這些是真正的光譜，一方越強，另一方就必然越弱。它們會隨人生事件與玩家決策逐漸漂移。

| Spectrum | Left Pole (-100) | Right Pole (+100) | Gameplay Effect |
|----------|-----------------|-------------------|-----------------|
| Social | 內向 | 外向 | 內向: 深度訪談, 藝術片, 個人創作有加成. 外向: 綜藝, 直播內容, 粉絲活動有加成 |
| Thinking | 直覺 | 理性 | 直覺: 即興, 情感型表演, 藝術創作有加成. 理性: 商業決策, 策略規劃, 分析型角色有加成 |
| Action | 謹慎 | 冒險 | 謹慎: 成長穩定, 負面事件較少, 但上限較低. 冒險: 可接觸高風險高報酬通告, 但事故機率較高 |
| Stance | 隨和 | 好勝 | 隨和: 團隊默契更好, 團體活動加成. 好勝: 在比賽, 頒獎, 試鏡情境表現更佳 |

**數值模型：**
- 範圍：-100 到 +100
- 初始值在藝人建立時設定
- 會根據經歷的事件漂移（例如：挺過醜聞可能使「謹慎 → 冒險」）
- 沒有「正確」位置，不同職涯路線會偏好不同區間
- 可在遊戲中生成「性格檔案」作為話題／新聞事件（呼應 MBTI 的文化潮流）

**設計意圖：**
- 這些數值**不能直接訓練**，玩家只能透過職涯選擇與事件決策間接影響
- 產生湧現式人格：歷經多次危機的藝人，會自然變得更冒險、更好勝
- 鼓勵玩家思考「這位藝人正在成為什麼樣的人」，而不只是最佳化數值

### 3.6 形象標籤（6）— 公眾人設

彼此獨立的數值，可以同時共存。一位藝人可以同時是 Pure 與 Sexy（「清純誘惑」原型），也可以同時是 Cool 與 Funny（「冷面笑匠」原型）。這不是光譜，每個標籤都是獨立維度。

| Tag | Key | Description | Boosted By |
|-----|-----|-------------|-----------|
| Pure | PUR | 乾淨, 親和, 健康清新 | 服裝選擇, 正向通告, 公益活動 |
| Sexy | SEX | 肢體魅力, 成熟吸引力 | 服裝選擇, 寫真集, 特定演出 |
| Cool | COO | 有型, 銳利, 毫不費力的時尚感 | 時尚通告, 音樂風格選擇, 公開態度 |
| Intellectual | ITE | 有文化, 深思熟慮, 知性 | 文化類通告, 訪談表現, 創作輸出 |
| Funny | FUN | 幽默, 有娛樂感, 綜藝能量 | 綜藝表現, 公開互動, 喜劇角色 |
| Mysterious | MYS | 神祕, 保留, 低調卻迷人 | 減少公開露面, 選擇性接案策略, 事件處理方式 |

**數值模型：**
- 各自範圍皆為 0-100，彼此獨立
- 變動來源：服裝／造型選擇（直接）、通告類型（累積）、事件決策、年齡成長（自然漂移）
- 自然年齡效果：Pure 傾向隨年齡下降，其他標籤則可能改變
- 某些標籤會更容易依據藝人的基礎屬性與性格被維持

**Gameplay 含意：**
- 每個通告／代言都會有「理想形象」需求（例如：兒童品牌代言希望 Pure ≥ 60）
- 形象衝突會降低成功率（高 Sexy 的藝人去做兒童節目會顯得不協調）
- 形象反差可製造話題（平常很 Mysterious 的藝人去做喜劇節目 = 新聞話題 → Popularity 暴增）
- 服裝是最直接的控制桿，在通告前更換造型可暫時改變形象標籤
- 長期形象是由整體職涯選擇累積而成，而不只是今天穿什麼

### 3.7 輔助數值（4）— 市場狀態儀表板

這些是反映藝人當前市場位置的系統層數值。它們**不能直接訓練**，而是從 gameplay 行為中自然浮現。

#### 知名度（Recognition）
- **範圍：** 0 → ∞（無上限、可累積）
- **方向：** 只會上升，不會下降
- **來源：** 完成通告、醜聞（是的，負面新聞也是曝光）、獎項、媒體曝光
- **功能：** 通告資格門檻，你必須先「夠有名」，才會被視為大型機會的候選人
- **設計說明：** 將「有多有名」與「有多受喜歡」分開非常關鍵。醜聞會讓你**更有名**，而不是更沒名。
- **用於通告門檻的正規化：** 通告需求使用分級式知名度區間，而非原始數值：

| Tier | Recognition Range | Label | Typical Gigs |
|------|------------------|-------|-------------|
| 0 | 0-99 | 無名 | 只能接打工 |
| 1 | 100-499 | 新人 | 小型通告, 地方廣告 |
| 2 | 500-1,999 | 竄升中 | 標準 TV/film/music 通告 |
| 3 | 2,000-4,999 | 站穩腳步 | 主要通告, 代言 |
| 4 | 5,000-14,999 | 明星 | 高級通告, 巡演 |
| 5 | 15,000+ | 超級巨星 | 傳奇級通告, 國際舞台 |

- 分級門檻可由資料設定。通告定義引用 tier（例如 `required_recognition_tier: 3`），而不是原始數值。

#### 風評（Reputation）
- **範圍：** -100 ↔ +100（單軸光譜）
- **方向：** 依行為與事件而變動
- **正向來源：** 優質作品、公益、良好的公關危機處理
- **負向來源：** 醜聞、糟糕的公關處理、具爭議的行為
- **功能：** 決定有哪些機會會出現，而且正反兩端都能解鎖獨特內容

| Reputation Range | Status | Available Opportunities |
|-----------------|--------|------------------------|
| +70 and above | 國民偶像 | 高級代言, 公益大使, 頒獎典禮主持 |
| +30 to +70 | 討喜 | 主流通告, 廣泛工作機會 |
| -30 to +30 | 中立 | 標準通告, 無特殊加成或懲罰 |
| -70 to -30 | 有爭議 | 熱門談話節目砲火位, 反派選角, 八卦節目專題 |
| -70 and below | 臭名昭著 | 地下活動, "bad boy/girl" 品牌合作, 但主流品牌封殺 |

**關鍵設計原則：** 風評的兩個極端都能成為可行的職涯策略。刻意「黑化」（主動讓風評崩壞）是一種合法的高風險高報酬玩法。這讓公關危機系統更有意義，因為危機不只是「需要修復的傷害」，也可能是策略轉折點。

#### 人氣（Popularity）
- **範圍：** 0-100
- **方向：** 會隨時間自然衰減，必須主動維持
- **來源：** 活躍通告、媒體曝光、粉絲活動、流行話題
- **衰減速度：** 若藝人長時間沒有公開活動，衰減會加劇
- **功能：** 決定通告報酬（你現在有多紅會影響喊價）、粉絲活動到場率、專輯／電影銷量倍率
- **設計說明：** 這捕捉了「15 分鐘爆紅」現象。藝人可能擁有很高的 Recognition（大家都知道他），卻只有很低的 Popularity（現在沒人在意他），也就是經典的「過氣」狀態。在高 Recognition、低 Popularity 的情況下東山再起，會成為一種獨立的 gameplay 挑戰。

#### 壓力（Stress）
- **範圍：** 0-100
- **方向：** 工作、訓練與負面事件會提高；休息與正向事件會降低
- **門檻效果：**
  - 0-30：健康。無懲罰。
  - 31-60：疲勞。訓練效率下降。出現輕微情緒事件。
  - 61-80：緊繃。通告失敗機率上升。負面事件觸發率提高。
  - 81-100：崩潰邊緣。藝人離職、重大醜聞、健康事件的風險大增。
- **管理方式：** 休息日、假期、正向社交事件，以及某些性格類型（如 Easygoing）會恢復更快
- **設計說明：** 壓力管理是核心節奏機制。它阻止玩家無止盡地刷訓練，迫使玩家做出更有策略性的排程決策。

---

## 4. 核心 Gameplay 系統（v1）

### 4.1 時間與排程系統

- 基礎時間單位：**1 週**
- 每週玩家都要為每位藝人安排活動（訓練、打工、通告、休息）
- 通告可用性以 **雙週** 週期輪替（不同類別交替出現）
- 平日：主要排定活動
- 週末：特殊活動（粉絲見面會、演唱會、外出）
- 依日曆觸發的事件：固定日期的頒獎典禮、季節活動

### 4.2 訓練系統

- 多種訓練類型，每種會提升 1-2 項專業技能
- 訓練等級（初階 → 專家）會逐步提高成本、效果與壓力
- 訓練**不會**降低其他技能（不同於打工），但會花錢並增加壓力
- 訓練效率會乘上相關基礎屬性倍率
- 所有訓練定義皆為 data-driven（RON／TOML）

### 4.3 打工系統

- 一開始就可使用，用來累積初期名氣與小額收入
- 每種工作都會增加某些技能／數值，同時降低其他數值（取捨機制）
- 報酬較高的工作，通常伴隨更高壓力成本與屬性交換
- 部分工作需要名氣／數值前置條件才會解鎖
- 所有工作定義皆為 data-driven

### 4.4 通告／演出系統

跨多個類別、決定職涯走向的工作機會：

| Category | Examples | Primary Skill |
|----------|----------|--------------|
| Music | 專輯, 單曲, 現場演唱會 | Vocal |
| Film/TV | 電影, TV 劇, 短劇 | Acting |
| Modeling | 時裝秀, 寫真集, 品牌企劃 | Poise |
| Variety | 談話節目, 遊戲節目, 特別節目 | Eloquence |
| Endorsements | 品牌代言, 商業廣告 | 形象標籤符合度 |
| Creative | 自製內容, 個人企劃 | Creativity |

- 每個通告都有需求（最低技能值、Recognition 門檻、Image tag 偏好）
- 通告成功與否取決於技能值、性格適配度與當前 Popularity
- 報酬會隨 Popularity 伸縮
- 通告供給以雙週輪替，且採 data-driven 設計
- 完成通告會影響：技能、Recognition、Reputation、Popularity、Stress 與 Image Tags

### 4.5 獎項系統

每年三個主要獎項（日期與標準皆為 data-driven，且可擴充）：

| Award | Timing | Core Criteria |
|-------|--------|--------------|
| Model Award | 約 9 月 | Poise + Image Tags (依風格而定) |
| Music Award | 約 11 月 | Vocal + Creativity + 專輯表現 |
| Film Award | 約 12 月 | Acting + Empathy + 影視作品表現 |

- 藝人必須符合入圍條件（例如：推出且有上榜的專輯）
- 最終評選會將藝人的相關數值與 AI 競爭者進行比較
- 得獎將大幅提升 Recognition 與 Reputation
- 預設勝利條件：在 3 年內拿下 3 項主要獎項

### 4.6 財務系統

- 公司擁有單一資金池
- 收入：通告酬勞、專輯／電影分潤、代言費
- 支出：訓練成本、活動成本（演唱會、寫真集）、公司維護費、藝人薪資
- 選配：股票市場投資（高風險／高報酬）
- **破產模型**（詳見附錄 A.13）：若公司餘額連續 4 週為負，且無可恢復途徑，則觸發 game over。辦公室可作為緊急手段降級（出售）以回收部分成本。
- 財務里程碑可解鎖公司升級

### 4.7 藝人招募系統

- 潛在藝人會在特定日期出現在特定地點
- 初次見面時的對話選項，會影響簽約條件（抽成比率）
- 招募談判失敗（與同一藝人失敗兩次）可能導致永久錯失該人選
- 每位藝人都有獨特的基礎屬性、性格光譜與成長潛力
- 所有藝人定義皆為 data-driven

---

## 5. v1 特殊功能系統

這四個系統是從 Stardom 3 時代挑選出來，納入初始版本的額外系統。

### 5.1 公關危機系統

威脅（或提供機會給）藝人職涯的隨機事件：

- **事件類型：** 醜聞、跟蹤狂事件、恐嚇信、謠言、狗仔爆料
- **玩家回應選項：** 每次危機都提供 2-3 個選擇，各自導向不同結果
- **影響面向：** Reputation（正或負）、Popularity（爭議會帶來聲量）、Stress，以及可能的 Image Tags
- **策略深度：** 危機不只是「需要修補的傷害」，依藝人當前職涯策略（主流或邊緣）不同，所謂的「正確回應」也可能不同
- **公關危機事件採 data-driven 設計：** mod 製作者可加入自訂危機情境

### 5.2 辦公室改建系統

經紀公司的視覺與功能性成長：

- 公司從一間小而樸素的辦公室起步
- 隨著營收與聲望達成里程碑，可用的升級選項逐漸開放
- 每個升級階段提供：視覺變化（新的辦公室背景）、功能加成（更多藝人欄位、更好的訓練設施、降低成本）
- 升級需要可觀資金，是一項策略性投資決策
- 辦公室狀態會顯示在主要管理畫面中，提供具體可見的成長回饋

### 5.3 服裝／造型系統

服裝會影響形象標籤，並提供視覺客製化：

- 每位藝人都有一個 outfit slot
- 服裝帶有 Image Tag 修正值（例如：`Elegant Dress: Pure +15, Intellectual +10`，或 `Leather Jacket: Cool +20, Rebellion trait +5`）
- 服裝可從商店購買，或透過事件取得
- 通告前選擇的服裝，會在藝人累積形象值之上再套用一層**暫時性**的 Image Tag 修正（見附錄 A.8）
- 累積形象會透過通告類型與事件緩慢改變（每次通告 ±1-5）
- 所有服裝皆為 data-driven（可透過 mod 輕鬆擴充）

### 5.4 迷你遊戲系統

打斷經營迴圈、增加互動性的迷你遊戲：

- 會在綜藝通告、訓練過程或特殊事件中觸發
- 迷你遊戲表現會影響通告成功評級與能力成長
- 迷你遊戲類型（data-driven，且可擴充）：
  - **Rhythm game**（用於音樂／舞蹈通告）
  - **Quick-time reaction**（用於綜藝節目）
  - **Memory match**（用於訓練）
  - **Trivia/quiz**（用於知性綜藝）
- 迷你遊戲是可選的，玩家也可以改用基於技能的自動判定來解決（accessibility）。自動判定公式：`success_score = relevant_skill * 0.7 + random(0, relevant_skill * 0.3)`。若 `success_score >= gig_difficulty_threshold`，則通告以「standard」評價成功；若超出 20% 以上，則獲得「excellent」。

---

## 6. 關係系統（Framework）

v1 先建立底層 framework；戀愛內容則在後續以資料模組形式加入。

### 6.1 核心機制

- 每個 NPC 對玩家（以及對每位藝人）都有一個 **Affinity** 值
- **Affinity 範圍：** 0-100（起始值由資料定義，通常為 10-30）
- **Relationship type** 是獨立的 enum tag（Friendship / Rivalry / Romance / Mentorship），由事件與玩家選擇決定，而**不是**單純依 Affinity 數值決定
- Affinity 會透過：對話選項、禮物、事件、工作互動而改變
- 高 Affinity 可解鎖：特殊事件、專屬通告、劇情分支

### 6.2 未來擴充

- 作為資料模組加入的戀愛劇情線
- NPC 專屬事件鏈
- 藝人與藝人之間的關係（默契、對立）

---

## 7. 敘事系統（Framework）

v1 先建立完整的 scripting engine；故事內容則作為資料模組逐步增加。

### 7.1 架構

- 事件／對話 scripting engine 內建於 Game Core
- scripts 定義於外部資料檔
- 支援：分支對話、條件觸發、變數追蹤、cutscene 流程編排
- 事件觸發條件可以是：日期、數值門檻、關係狀態、隨機

### 7.2 v1 內容範圍

- 基本開場情境與教學
- 頒獎典禮場景
- 結局序列（多結局、data-driven）
- 公關危機對話樹
- 最低限度的藝人個人劇情線（作為 framework 示範）

### 7.3 設計哲學

> 「豐富而扎實的 gameplay framework 能留住玩家，故事則讓體驗變得難忘。」

敘事引擎必須足夠穩健，以支援深度敘事；但在 v1 中，優先順序仍是完整的 gameplay 系統，而非大量敘事內容。

---

## 8. v1 後的擴充路線圖

這些功能在架構上已納入考量，但不會在 v1 實作：

| Feature | Description | Architecture Impact |
|---------|-------------|-------------------|
| Artist mode | 玩家就是藝人本人 | 表現層變更, Game Core 共用 |
| Group/band system | 組建藝人團體並培養默契 | Game Core 中的 entity composition |
| Self-production | 公司自行製作專輯/節目 | 通告系統擴充 |
| International expansion | 海外訓練, 國際通告 | 地點系統擴充 |
| Mod editor UI | 內容創作的視覺化編輯器 | 讀寫與 Game Core 相同的資料檔 |
| Deep romance content | 完整關係劇情線 | 敘事系統的資料模組 |
| Difficulty modes | easy/normal/hard 設定 | Game Core 中的參數倍率 |
| Event tracker/journal | 遊戲內事件鏈追蹤筆記 | UI 功能 + 事件系統中繼資料 |

---

## 9. 資料架構摘要

所有遊戲內容皆定義於外部資料檔中，以支援 mod 與可擴充性：

| Content Type | File Format | Examples |
|-------------|-------------|---------|
| Artist definitions | RON/TOML | 基礎數值, 性格, 成長曲線, 背景故事 |
| Gig definitions | RON/TOML | 需求, 獎勵, 時長, 類別 |
| Training definitions | RON/TOML | 技能效果, 成本, 壓力, 等級進程 |
| Part-time job definitions | RON/TOML | 報酬, 數值變化, 前置條件 |
| Event/story scripts | RON/custom | 觸發條件, 對話樹, 結果 |
| Award definitions | RON/TOML | 時間點, 標準, 競爭者檔案 |
| Outfit definitions | RON/TOML | 形象標籤修正值, 成本, 視覺資產引用 |
| PR crisis scenarios | RON/TOML | 觸發條件, 選項, 結果 |
| Office upgrade tiers | RON/TOML | 成本, 加成, 視覺資產引用 |
| Mini-game definitions | RON/TOML | 類型, 難度, 獎勵公式 |

---

## 10. 參考研究

原版 Stardom 系列的詳細機制研究可參見：

- `research/stardom1-game-mechanics-research.md` — Stardom 1（1995）完整系統分析
- `research/stardom2-game-mechanics.md` — Stardom 2（1998）完整系統分析
- 另有針對 Stardom 3 與整個系列的研究，關鍵發現已整合進本設計

### 主要來源：
- 巴哈姆特 Stardom forum 封存資料
- Reko Wiki — 明星志願系列
- 幻光星宇經典遊戲檔案
- Steam community guides
- 各類 fan blog 與 wiki（完整來源清單見 research files）

---

## 附錄 A：數值模型與公式

本附錄提供前述各系統所引用的量化模型。所有常數皆可由資料設定，以下為建議預設值。

### A.1 訓練效率公式

每次訓練的成長量：

```
effective_gain = base_gain_for_tier
                 * (1.0 + base_attribute_bonus)
                 * condition_modifier

base_attribute_bonus:
  primary_attribute:   (attribute_value - 50) / 100   (range: -0.49 to +0.50)
  secondary_attribute: (attribute_value - 50) / 200   (range: -0.245 to +0.25)
  total = primary + secondary (if applicable)

condition_modifier:
  stress 0–30:   1.0
  stress 31–60:  0.85
  stress 61–80:  0.65
  stress 81–100: 0.40
```

範例：Dance 訓練（中階，`base_gain=80`），藝人的 STA=70（primary），stress=25：
- `base_attribute_bonus = (70-50)/100 = 0.20`
- `effective_gain = 80 * 1.20 * 1.0 = 96`

### A.2 訓練等級（預設值）

| Tier | Cost ($) | Base Skill Gain | Stress Increase | Unlock Condition |
|------|----------|----------------|-----------------|-----------------|
| Beginner | 8,000 | 40 | +5 | 永遠可用 |
| Intermediate | 16,000 | 80 | +10 | Skill ≥ 1,000 |
| Advanced | 28,000 | 130 | +16 | Skill ≥ 3,000 |
| Expert | 44,000 | 180 | +22 | Skill ≥ 6,000 |

### A.3 人氣衰減模型

```
weekly_decay = base_decay + inactivity_penalty

base_decay:         -2 per week (always applies)
inactivity_penalty: -0 if artist had public activity this week
                    -2 if 1 week inactive
                    -4 if 2 consecutive weeks inactive
                    -6 if 3+ consecutive weeks inactive (caps here)

Minimum Popularity: 0
```

藝人在完全沒有活動的情況下，流失量為：第 1 週 = -4，第 2 週 = -6，第 3 週起 = 每週 -8。活躍藝人只會每週流失 -2，很容易被通告收益抵銷。

### A.4 壓力門檻效果

| Stress Range | Training Efficiency | Gig Failure Chance | Event Trigger |
|-------------|--------------------|--------------------|--------------|
| 0-30 | 100% | 0% base | 無 |
| 31-60 | 85% | 每高於 30 的 10 點壓力 +5% | 輕微情緒事件 (data-driven) |
| 61-80 | 65% | 每高於 60 的 10 點壓力 +10% | 若 Rebellion > 50, 則有缺席工作機率 |
| 81-100 | 40% | 每高於 80 的 10 點壓力 +15% | 藝人離職機率: 每週 `(stress - 80) * 2%` |

範例：Stress=90，Rebellion=60 → 通告失敗：+15%（stress 81-90），缺席工作：啟用，離職機率：每週 20%。

### A.5 通告成功計算

```
success_score = weighted_skill_sum
                * personality_fit_modifier
                * image_match_modifier
                * popularity_modifier

weighted_skill_sum:
  Each gig defines required skills with weights (e.g., Vocal: 0.6, Dance: 0.3, Poise: 0.1)
  Sum = Σ(skill_value * weight)

personality_fit_modifier:
  Each gig may prefer a personality range (e.g., "prefers Extrovert > 30")
  Match: 1.0–1.15 bonus. Mismatch: 0.85–1.0 penalty. Neutral: 1.0.

image_match_modifier:
  Each gig has optional ideal_image_tags (e.g., Pure ≥ 40, Cool ≥ 30)
  Per tag: if met, +0.05 bonus. If strongly exceeded (2x threshold), +0.10.
  If a "conflicting" tag is high (defined per gig), -0.10 penalty.
  Total modifier: clamped to 0.80–1.20.

popularity_modifier:
  1.0 + (popularity - 50) / 200
  Range: 0.75 (pop=0) to 1.25 (pop=100)
```

### A.6 內在特質門檻事件（預設）

| Trait | Threshold | Event |
|-------|-----------|-------|
| Rebellion > 50 | Stress ≥ 61 | 缺席工作機率: `(rebellion - 50) * 1.5%` |
| Rebellion > 70 | Any | 公關危機觸發率 +20% |
| Rebellion > 85 | Any | 可能觸發合約糾紛事件 |
| Confidence > 60 | Failed audition | 情緒懲罰加倍 |
| Confidence > 75 | Any | 可能拒絕 Recognition tier 3 以下的通告 |
| Confidence > 90 | Group activity | 衝突事件: 與 Easygoing 藝人的默契懲罰 |

所有門檻與事件皆為 data-driven，上述數值為建議預設。

### A.7 性格光譜 Gameplay 修正值

每個光譜位置都會對相關活動提供修正。修正量會依距離中心點（0）的遠近線性縮放：

```
modifier = |spectrum_value| / 100 * max_bonus

max_bonus per spectrum: 0.15 (15%)
```

| Spectrum | Left Pole Bonus | Right Pole Bonus |
|----------|----------------|-----------------|
| Social (-100 to +100) | Introvert: 個人創作通告, 深度訪談, 藝術片角色 | Extrovert: 綜藝, 粉絲活動, 直播內容, 團體默契 |
| Thinking | Intuitive: 即興型通告, 情感角色, 創作歌曲 | Logical: 商業事件, 分析型角色, 策略性公關決策 |
| Action | Cautious: 負面事件機率 -20%, 最大通告報酬 -10% | Adventurous: 負面事件機率 +20%, 最大通告報酬 +10% |
| Stance | Easygoing: 團體默契 +15%, 競賽分數 -10% | Competitive: 團體默契 -10%, 比賽/試鏡分數 +15% |

### A.8 形象標籤動態

**Outfit（暫時）vs. Cumulative（永久）：**
- Outfit 修正值會在通告期間疊加到累積值之上
- 累積值會透過每次通告／事件以 ±1-5 的速度變化（緩慢漂移）
- Outfit 修正值範圍通常為每個標籤 ±10-25

**年齡對形象標籤的影響：**
```
If artist_age > 25:
  Pure -= (artist_age - 25) * 0.3 per year (natural decay, floor 0)
```
其他標籤預設不受年齡影響（可由資料設定）。

### A.9 合約與招募模型

**抽成比率：** 經紀公司從通告收入中抽取的比例。
- 基礎比率：30%（可依藝人資料設定）
- 招募期間的對話選項：調整 ±5-10%
- 範圍：15%-50%

**合約條款：**
- 期間：1 年，除非藝人 Affinity < 20 或事件觸發，否則自動續約
- v1 不設最低通告額度（相較 Stardom 2 的簡化）
- 若藝人的 Recognition tier 提升 2 級以上，可能要求重新談約

**招募失敗：**
- 第一次談判失敗：下週可於同一地點再試一次
- 與同一藝人連續第二次失敗：6 個月內無法再招募（可由資料設定）

### A.10 排程與通告時長

- 基礎單位：1 週
- 每位藝人每週有 1 個活動欄位
- 通告有以週為單位的 **duration**（典型範圍 1-8 週）
- 在多週通告進行期間，藝人會被鎖定於該活動（無法訓練、打工或接其他通告）
- 短期通告（1 週）：廣告、綜藝露出、拍攝工作
- 中期通告（2-4 週）：TV 劇集數、專輯錄製
- 長期通告（4-8 週）：電影、巡演前準備
- 雙週輪替僅作用於 **新出現的可接通告**，不影響進行中的通告

### A.11 資料檔格式策略

| Content Type | Format | Rationale |
|-------------|--------|-----------|
| Structured game data (artists, gigs, items, awards) | **RON** | Rust 原生, 型別安全, 對 enum 支援良好 |
| Configuration & settings | **TOML** | 對玩家與 mod 製作者較友善 |
| Event/dialogue scripts | **RON with embedded script blocks** | 維持單一生態系, script blocks 使用由敘事引擎解析的簡單 DSL |

敘事 scripts 的自訂 DSL，將於敘事引擎設計完成後另行文件化。

### A.12 Game Core ↔ Presentation Layer 介面（草圖）

```rust
// Commands: Presentation → Game Core
enum GameCommand {
    AdvanceWeek,
    AssignActivity { artist_id: ArtistId, activity: Activity },
    SignArtist { artist_id: ArtistId, commission: f32 },
    PurchaseOutfit { artist_id: ArtistId, outfit_id: OutfitId },
    RespondToCrisis { crisis_id: CrisisId, choice: usize },
    UpgradeOffice { tier: OfficeTier },
    // ... extensible
}

// State: Game Core → Presentation
struct GameState {
    calendar: Calendar,          // current date, week, year
    company: CompanyState,       // money, office tier, reputation
    artists: Vec<ArtistState>,   // all signed artists with full attribute state
    available_gigs: Vec<Gig>,    // current rotation of available gigs
    active_events: Vec<Event>,   // ongoing events/crises requiring response
    // ... extensible
}

// Presentation reads GameState each frame, renders UI accordingly.
// Presentation sends GameCommands based on player input.
// Game Core processes commands, updates GameState, returns results.
```

這是概念性草圖，實際 API 會在實作規劃階段設計。

### A.13 破產模型

**公司餘額可以為負**（允許負債）。這可避免因單次高額支出而立刻 game over。

**破產觸發條件：**
```
IF company_balance < 0
   AND consecutive_negative_weeks >= 4
   AND no_pending_gig_income (no artist has a gig completing within 2 weeks)
THEN trigger bankruptcy → game over
```

**玩家在破產前可採取的緊急措施：**
- **降級辦公室：** 以目前升級成本的 40% 售出當前辦公室等級，回退到上一級
- **終止藝人合約：** 節省薪資支出（但永久失去該藝人）
- **貸款：** 若遊戲內銀行提供該功能，則可借款（利息按週累積）

**「餘額」的定義：**
- 只計算現金。未實現資產（辦公室價值、合約潛力）**不**計入，玩家必須主動變現。
- 待收通告收入（藝人正在執行中且報酬已知的通告）視為可恢復途徑，會暫停 4 週破產計時器。

**逐週流程：**
1. 週結束 → 計算收入 - 支出 → 更新餘額
2. 若餘額 < 0：`consecutive_negative_weeks` 計數器 +1
3. 若餘額 ≥ 0：計數器重設為 0
4. 若計數器達到 4 且無待收收入：觸發破產事件
5. 破產事件顯示敘事場景，接著 game over（可選擇重新讀檔）

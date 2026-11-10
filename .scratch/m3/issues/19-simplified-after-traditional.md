# 19: 繁體模式的簡體字排在繁體之後

Status: done
Blocked by: 02
讀: brief §1.3、§4.1；plan §2.3（候選列）、§4.1、§4.4；`docs/research/blackbox-baseline.md`（T17）；spec〈範圍〉〈共同規則〉

## What to build

繁體模式下，HKSCS 收的簡體字（「门」「农」「阳」這類）一律排在同碼的繁體字之後，像 macOS 速成；不過濾，選過照常學（使用者 2026-11-11 決定）。粗口字「𨳊」照字頻不動。

- 先定「簡體字」怎麼判定，判定放建置端還是核心在票內定，寫回 brief §4.1 與 plan §2.3 或 §4.1。兩類字絕不能被算成簡體字：
  - 繁簡同形的字（人、日）。
  - 本身是正規繁體字、又是別的字的簡體寫法的字：台（臺）、后（後）、里（裡）、干（乾）、面（麵）、系（係）、云（雲）、松（鬆）、只（隻）、范（範）、谷（穀）、斗（鬥）、制（製）、准（準）、几（幾）這類。所以不能只看「在 OpenCC 對照表的簡體那邊出現過」。
  - 可考慮的判定：只算不在 Big5、只靠 HKSCS 進連打字集的字（「门」「农」「阳」都是），開工時先驗證這條是否夠窄、又抓得到 T17 的例子；用別的判定也要過下面的驗收。
- 簡體模式不受影響。

## 驗收

- [x] 單元測試或 `--verify` 抽查：繁體模式 `ls` 的「门」排在所有同碼繁體字之後；「人」「日」的位置不變；簡體模式照常。
- [x] 「台」「后」「里」「干」「面」「系」各在自己的碼上排序和改之前相同（測試鎖住）；`fixed.tsv` 的 `mir 一台` 照過。
- [x] Comments 列出被判成簡體字的字數與全表（或抽樣），人工看過沒有正規繁體字。
- [x] 選兩次「门」後它成首選（學習照常）。
- [x] `bench --check` 不變差，貼對照；黑箱 T17 的 QIM 欄更新。
- [x] `cargo xtask check` 綠燈。

## Comments

### 判定（2026-11-11，M3 票 19）

放建置端：`qim-databuild` 的 `simplified.rs` 算出簡體字，`CodeEntry::simplified` 標在碼表上（資料包格式 5 → 6），核心只看標記排序（plan §2.3、§4.1，brief §4.1）。一個字算簡體字要同時滿足：

1. 連打字集裡 HKSCS 旗標為 1、Big5 不是。只看這條不夠窄：HKSCS-only 有 4,548 字，嘅、啲、嘢這類粵語字都在裡面。
2. OpenCC `STCharacters` 簡轉繁一定換掉它（值裡沒有它自己）。拿 `TSCharacters` 的值反查（「在簡體那邊出現過」）會多抓亘、叶、广、坂、幺。
3. `TSCharacters` 沒把它對到自己（排掉麽）。
4. 不是 `HKVariants` 的香港寫法（排掉卧、叁、囱、户、揾、榅、温、葱、蒀）。
5. 不是香港《常用字字形表》的標準字形：Unihan `kHKGlyph` 有它，它的繁體寫法卻沒有一個同序號（排掉够）。
6. 不在 `CANTONESE_KEPT`（排掉吓、响）。

Big5 這條是必要的：Big5 裡有 59 字是 `STCharacters` 一律換掉的（峰、床、群、粽…，正規繁體）。票列的台、后、里、干、面、系、云、松、只、范、谷、斗、制、准、几在 `STCharacters` 都對到自己，也都在 Big5，兩條各自擋得住。

### 被判成簡體字的字：278 個

业东两个丽义争亚亿从仪众会侨偬兖兴兹冈农冲决况凉减凑凛凤凭删务动医华卤却厢厦厨厩厮参双发变叙叠号启吴咏咤团园国块坛坟垴墙声壳处备头奥姹学宝实宪寿尔屉届属岚峥帮幂庆庙弥强彻总恒惧惨惩戬护担拟拥携撑斋无晋晓暅条栀样栾桥梦椁橥橱殁毁毡汉汹沟沪泪济涤渊溆滚滢滨滩潜潴灯灵灾灿炉点炼炽烟牦猪猫猬献玺珐琼瑶电画疱疴痈盖眦矾砜碍碱礼祢禀秆稆窃窑笋笔籴类籼粜粮糇糍纟纤纬纺织经统缆耻联胆胶脉脔脚腭腼艺芦苏荣荦药莅莹萤萦蓥蕰蘖虬虾衅补衮袄袜装见视讠设询贝跃踪车轧轮输边达迁运迹递邮邻銮钅钟钢铁铃铜银锭镇长门间阳际险随隶隽霡鞲韦韵页风飞饣马骏鱼鸟麦黄黾鼹齐齿龙龟𠀾𠵾𡞱𥐯𥐰𦛨𦻕

人工看過：都不是香港或台灣的正規繁體字形。需要判斷的幾個：
- 恒：使用者原本要排除（「香港《常用字字形表》的標準字形」）。查證結果相反：字表的正字是「恆」（Unihan `kHKGlyph` 有恆 1323、沒有恒；文匯報 2013-01-28〈正字異體各地有別〉：「《常用字字形表》與『台灣教育部』公布的《異體字表》均以『恆』為正字」），所以照第 5 條的原則留在表內。恒生、恒指這類專名常用，選兩次就成首選。要改的話，在 `simplified.rs` 加一條例外即可。
- 够：使用者原本要留（我先前說它不是香港正規寫法，說錯了）。字表收「够」0832、不收「夠」，是香港標準字形，照第 5 條移出。
- 叙、携、烟、踪、迹、韵：`kHKGlyph` 有，但和繁體寫法同一個序號（叙敘敍 1715 等），是字表同一條列的可接受異體，不是標準字形，照算簡體字。
- 糍（餈）、疴（痾）、咤（吒）、姹（奼）：香港行文常見（糯米糍、疴屎、叱咤），但都不是字表的字形，也不在 Big5，照算。
- HKSCS 本身沒有標「香港標準字形」的欄位，libcangjie 也沒有，所以字形表只用 Unihan `kHKGlyph`（2000 年修訂本）；2007、2012 年修訂本（4,762 字）沒有機讀版可對。

### 排序

繁體模式：蓋滿整串的字、詞、多段路徑依分數 → 簡體字依分數 → 擴展區 → 只蓋開頭一段的（同長裡簡體字排最後）。排位規則在之後套用，所以選兩次照樣成首選；簡體模式不分。簡體字照樣進 lattice、在多段路徑裡照分數比（語料裡只有零星殘留），不另扣分。

網頁實測（`web/e2e/simplified-characters.spec.ts`，截圖 `.scratch/m3/evidence/19-ls-panel.png`）：`ls` 门 第 9 → 第 18、`na` 阳 13 → 20、`hv` 农 21 → 38、`yk` 头 27 → 41、这 72（擴展區，不變）。`--verify` 真資料：`ls` 的「门」排在 14 個同碼繁體字和 3 條多段路徑之後。黑箱 T17 的 QIM 欄已更新。

### bench（`cargo xtask bench --check`）

| 集合 | 答錯 前 → 後 | 修正 前 → 後 | 學習開的修正 前 → 後 |
| --- | ---: | ---: | ---: |
| fixed | 0 → 0（84/84，含 `mir 一台`） | 0 → 0 | — |
| yue-wiki | 1131 → 1131 | 4467 → 4473 | 4406 → 4412 |
| zh-wiki | 1126 → 1126 | 4505 → 4505 | 4523 → 4523 |
| common-voice-yue | 1032 → 1032 | 4043 → 4045 | 3983 → 3985 |
| hkcancor | 663 → 663 | 2588 → 2588 | 2186 → 2186 |

依長度分組只有 yue-wiki 3–4 字的修正/句 1.09 → 1.10，其餘相同；沒有翻盤、答錯行號不變。測試題裡含簡體字的只有 4 題：common-voice-yue 724「恒指尋日大跌」、yue-wiki 119「硫黄島」、611「漢口因為够大」、711「稲荷寿司」（語料殘留的簡體或日文新字體）。正確答案本身含排到後面的字，模擬打字要多翻頁，所以修正次數多 8 次。够 移出後數字不變，所以這 8 次應該來自其餘 3 題。使用者 2026-11-11 同意 `--update-baseline`（yue-wiki 4473、common-voice-yue 4045）。

### 沒做的

- 網頁沒改程式碼，只加 e2e，沒有另外亂按一輪（spec〈共同規則〉要求動到網頁的票做）。

### 開關（2026-11-11 協調者要求補）

- 核心：`Options::simplified_after_traditional`，預設開（brief §7，同 macOS 速成與使用者 2026-11-11 的決定）；關掉或簡體模式時 `Listing::simplified_last` 為假，照分數混排，等於改前的排序。換這個選項也算「列出的候選不同」（`lists_like`），組字中殼呼叫 `refresh`（plan §2.1）。`Engine::candidates` 照預設選項。
- 網頁：設定區（`settings.ts` 的 `ROWS`）在「簡體」後加「繁體字優先」，綁 `setSimplifiedAfterTraditional`。標籤不用「簡體字排後」：Playwright 的 `getByRole` 名稱是子字串比對，會和「簡體」撞名。
- 測試：`simplified_characters.rs` 加三條（預設開、關掉時「门」回第 1、組字中切換重列）；`settings.spec.ts` 驗預設勾選；`simplified-characters.spec.ts` 組字中關掉後 `ls` 的「门」回到第 9（改前的位置），再打開回第 18。截圖 `.scratch/m3/evidence/19-ls-panel-off.png`。
- 合併 main（票 01、02、06、14）沒有衝突；票 14 的 `crates/qim-core/tests/phrases.rs` 補 `simplified: false` 併進合併 commit。合併沒動到 `qim-databuild` 與 `data/`，資料包沿用，沒重跑 `--verify`。合併後 `cargo xtask check` 全綠（Playwright 160 個），`bench --check` 和更新後的 baseline 相同（yue-wiki 4473、zh-wiki 4505、common-voice-yue 4045、hkcancor 2588，固定題 84/84）。

### 合併票 04（2026-11-11 協調者要求）

- `git merge main`（含票 04 的 `set_context`、bigram 句首列、有前文集合）。衝突在 `engine.rs`、`lattice.rs`、`session.rs`（`span` 與 `Lattice::from_input` 多了前一字參數，和本票的 `Spanned`、`Listing` 合在一起）、`docs/development.md`（verify 報告的說明兩邊都留）與 `bench/baseline.json`（取 main 的，重跑後再更新）。票 04 的 `crates/qim-core/tests/context.rs` 補 `simplified: false`。
- 資料包格式版本：main 仍是 v5。票 04 的語言模型只多了句首記號 `CLAUSE_START` 的資料，序列化結構沒變；本票已升 v6，所有資料包檔頭一起換，合併後用 v6 就涵蓋兩邊。
- `cargo xtask data --verify` 通過：簡體字 278 個，`ls` 的「门」排在 14 個同碼繁體字與 3 條多段路徑之後，決定性通過（速成 `33ff11d4`、語言模型 `d1392804`）。
- `bench --check` 對 main 的 baseline：答錯數 13 個集合都不變、沒有翻盤，固定題 84/84。修正次數 yue-wiki 4340 → 4346、common-voice-yue 3946 → 3948、yue-wiki·前文 2288 → 2294、yue-wiki·前文·不給 2300 → 2306，其餘相同。前文集合裡答案含簡體字的只有 109「硫黄島」、633「稲荷寿司」，和先前的原因相同，照使用者先前的同意 `--update-baseline`。
- 合併後 `cargo xtask check` 全綠（Playwright 160 個），`bench --check` 通過。

### code-review（`3543c83` 起）

已處理：
- Standards：`simplified` 同時指簡體模式與簡體字標記 → 核心內部的模式欄改名 `Listing::simplified_mode`，plan §0 對照表加一列。
- Standards：e2e 重複的開面板步驟收成 `openPanel`，第一頁的 9 改成 `PAGE_SIZE`；`unihan::hk_glyphs` 的重複錯誤字串合併。
- Spec：判定測試補「面」「系」；`--update-baseline` 的理由寫在 commit message。
- Spec：共同規則「會改寫打出的字或改變候選位置的新行為都有開關」→ 協調者要求在本票補，見下〈開關〉。

沒處理，留給之後：
- Spec：票寫「一律排在同碼的繁體字之後」，但多段路徑與詞庫的詞裡的簡體字照分數比，沒有排後（plan §2.3 已寫明這個取捨）。語料裡簡體字只有零星殘留，bench 沒有首選因此含簡體字的題。
- Spec：「台、后、里…排序和改之前相同」鎖的是「不標成簡體字」（單元測試用假資料測台后里干面系；真碼表靠 `--verify` 的 `NotSimplified`，它不在 `cargo xtask check` 裡）。沒標的字排序不受影響，是由核心測試 `simplified_characters.rs` 的甲、申相對順序守的。
- Spec：叙、携、烟、踪、迹、韵和繁體寫法共用字表序號就當異體、照算簡體字，這是從 Unihan 共用序號推論的，字表本身沒標出正字；要使用者確認。
- Standards（判斷）：`lattice.rs` 的 `(f64, bool, CandidateSpan)` 用位置的 bool；`CodeEntry` 加欄位要改約 20 個測試的字面值（沒有建構函式）；`once_chosen_keeps_the_unlearned_top` 自己組 `Lattice::from_input` + `candidates`，和 `Engine::candidates` 重複；`simplified::References` 的 `st`、`ts` 名字要靠註解；`Session::listing(bool)` 三處兩處傳 `false`。


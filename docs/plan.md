# QIM 實作計劃

> 狀態：M0、M1 寫到任務與驗收；M2 之後只有目標、範圍與完成標準。本檔只寫現況：決定改了就直接改寫，不留「原寫」「取代原」「某日改」；舊版與理由記在 progress〈偏差與決定〉，影響架構的寫 ADR。里程碑開工時，它在〈里程碑任務〉的那一節細化成 `.scratch/<里程碑>/spec.md`，這裡縮成一行連結；拆票後以票為準。需求與決定見 `docs/design-brief.md`，名詞見 `GLOSSARY.md`。

## 0. 工程慣例

- Rust edition 2024，toolchain 用 stable（寫本檔時 1.99.1），`rust-toolchain.toml` 釘版本。
- 核心的新功能與修 bug 走 `mattpocock-skills:tdd`；一個 session 一張票（`CLAUDE.md`）。
- 程式碼識別符用英文，對應 `GLOSSARY.md` 的詞：

| 名詞 | 識別符 | 名詞 | 識別符 |
| --- | --- | --- | --- |
| 輸入方案 | `Scheme` | 候選 / 首選 | `Candidate` / `top` |
| 輸入碼 | `input` | 組字 | `Composition` |
| 速成碼 / 倉頡碼 | `QuickCode` / `CangjieCode` | 上屏 | `commit` |
| 碼表 / 詞庫 | `CodeTable` / `Lexicon` | 字根提示 | `RadicalHint` |
| 連打 | `continuous` | 英文候選 | `EnglishCandidate` |
| 使用者詞 / 觸發詞 | `UserWord` / `Trigger` | 聯想 | `Association` |
| 自動造詞 | `AutoPhrase` | 無痕模式 | `Incognito` |
| 核心 / 平台殼 | `core` / `Shell` | 佈局 / 資料包 | `Layout` / `Pack` |
| 首選字基準 | `TopChoiceBench` | 修正次數 | `correction_count` |
| 分段 | `Segment` | 佈局版本 | `LayoutVariant` |
| 個人模型 | `UserModel` | 空間模型 | `SpatialModel` |
| 重排 | `Rerank` | | |

## 1. 架構總覽

### 1.1 執行形態

核心是 Rust 函式庫，各平台殼把它連進自己的程序（ADR 0002）：網頁經 WebAssembly，Apple 與 Android 經 uniffi 產生的 Swift / Kotlin 綁定，Windows 與 Linux 經 C ABI。Windows 依成熟做法把核心放在獨立程序，TSF DLL 只轉傳（`docs/research/platform-frameworks.md` §6），到 M9 再定。

```mermaid
flowchart LR
  subgraph Build["建置時（qim-databuild）"]
    Src["第三方來源<br/>libcangjie、rime-essay、<br/>維基、口語語料、SCOWL、OpenCC"] --> DB["qim-databuild"]
    Own["data/<br/>方案檔、佈局、粵語詞表"] --> DB
    DB --> Packs["資料包<br/>碼表、詞庫、bigram、<br/>英文、簡繁"]
    DB --> Corpus["bench/corpus<br/>基準語料"]
  end

  subgraph Runtime["執行時"]
    Core["qim-core<br/>Session、lattice、排序、學習"]
    Layout["qim-layout<br/>佈局驗證與幾何"]
    Wasm["qim-wasm"]
    Ffi["qim-ffi（M2）"]
    Web["web/ 網頁 prototype"]
    Mac["platforms/macos（M6）"]
    Others["iOS、Android、<br/>Windows、Linux"]
  end

  Packs --> Core
  Core --> Wasm
  Layout --> Wasm
  Core --> Ffi
  Layout --> Ffi
  Wasm --> Web
  Ffi --> Mac
  Ffi --> Others
  Corpus --> Bench["qim-bench"]
  Core --> Bench
```

### 1.2 目錄結構

| 路徑 | 內容 |
| --- | --- |
| `crates/qim-core/` | 純邏輯：方案、資料包讀取、lattice、排序、學習、`Session` |
| `crates/qim-layout/` | 佈局模型、JSON Schema 驗證、點擊判定、按鍵幾何計算 |
| `crates/qim-databuild/` | CLI：取得來源、建資料包、訓練 bigram、產生基準語料 |
| `crates/qim-bench/` | 首選字基準 runner，量正確率與每鍵延遲 |
| `crates/qim-wasm/`、`crates/qim-ffi/` | 綁定層，不放邏輯；`qim-ffi` 在 M2 建 |
| `xtask/` | 閘門與開發指令（`cargo xtask check`、`bench`、`commit-msg`） |
| `data/` | 自己維護的方案檔、佈局檔、粵語詞表、來源清單 `sources.toml`；不放第三方原始檔與建置產物 |
| `bench/` | `fixed.tsv`、`corpus/`、`colloquial/`（都含授權與署名）、`baseline.json` |
| `web/` | 網頁 prototype（Vite + TypeScript）、`web/editor/` 佈局編輯器、Playwright 測試 |
| `platforms/` | 各平台殼，M6 起逐一建立 |

建置產物不進 git：`target/`、`.cache/sources/`（下載的來源）、`build/packs/`、`web/public/packs/`。

### 1.3 依賴規則【決定】

| crate | 可以依賴 | 不可以 |
| --- | --- | --- |
| `qim-core` | `serde`、`postcard`；之後要加的依賴先改本表 | 任何綁定或平台 crate；`std::fs`、`std::net`、`std::thread`、`std::time::SystemTime`（時間由殼以上屏次數傳入） |
| `qim-layout` | `serde`、`serde_json`、`jsonschema` | `qim-core`；同上的 I/O |
| `qim-wasm`、`qim-ffi` | `qim-core`、`qim-layout`、各自的綁定工具 | 被 `qim-core` 或 `qim-layout` 依賴 |
| `qim-databuild`、`qim-bench` | `qim-core`（資料包格式）、I/O 套件 | 被 `qim-core` 依賴 |
| `web/`、`platforms/*` | 只透過綁定呼叫核心與 `qim-layout` | 重寫排序、查碼或點擊判定邏輯 |

強制：`cargo xtask check` 的 `deps` 步驟讀 `cargo metadata` 比對上表的白名單，並掃 `qim-core`、`qim-layout` 原始碼的禁用路徑。測試做雙向變異：在暫存副本加入違規依賴或 `use std::fs` 會紅；改無關的格式或命名不紅。M0 票 01 接上。

### 1.4 主要依賴

版本為寫本檔時的 stable，開工時以 `cargo add` / `npm install` 取當時 stable 並更新本表。

| 依賴 | 版本 | 用途 | 理由 |
| --- | --- | --- | --- |
| Rust | 1.99.1 | 核心與工具 | ADR 0002 |
| `serde` / `postcard` | 1.0.229 / 1.1.3 | 資料包序列化 | `no_std` 友善、體積小；資料包格式是內部格式，可換 |
| `toml` | 1.1.8 | 方案檔 | Rust 生態標準格式，可寫註解 |
| `serde_json` / `jsonschema` | 1.0.154 / 0.60.2 | 佈局檔與驗證 | 佈局要給 TS、Swift、Kotlin 直接讀 |
| `wasm-bindgen` / `wasm-bindgen-cli` | 0.2.130 | WebAssembly 綁定 | 直接用 CLI，不依賴 wasm-pack |
| `uniffi` | 0.32.2 | Swift / Kotlin 綁定（M2） | 一份介面產生兩種語言 |
| `ferrous-opencc` | 0.4.0 | 簡繁轉換 | 純 Rust、Apache-2.0；能否編 WebAssembly 在 M1 票開工時驗證 |
| `proptest` | 1.12.0 | 核心的性質測試（dev） | — |
| Vite / TypeScript | 8.3.5 / 7.0.2 | 網頁 prototype | — |
| Biome | 2.6.4 | TS lint 與 format | 一個工具包辦兩件事 |
| `@playwright/test` | 1.64.1 | 網頁 e2e 與驗收 | 使用者指定 |

## 2. 核心

### 2.1 介面

參考 librime 的 `RimeApi`（`process_key`、`get_context`、`select_candidate`）與 fcitx5 的 InputContext：殼把事件交給 session，拿回一份完整快照，殼只負責畫。

```rust
pub struct Engine;      // 不可變：方案 + 資料包，多個 Session 共用
pub struct UserModel;   // 可變：使用者詞、學習計數；export() / import() 給殼存檔
pub struct Session;     // 一個輸入框的組字狀態

pub enum Key { Letter(char), Space, Enter, Backspace, Escape, Digit(u8), PageUp, PageDown, Left, Right, ToggleAscii }

pub struct Options { pub simplified: bool, pub incognito: bool, pub english_candidates: bool }

pub struct Snapshot {
    pub composition: Composition,   // 輸入碼、已打的字根、各分段（碼、字、是否已選定）、目前分段
    pub candidates: Vec<Candidate>, // 目前頁
    pub page: usize,
    pub has_more: bool,
    pub commit: Option<String>,     // 這次事件要上屏的文字
    pub ascii: bool,
    pub generation: u64,            // 每次候選變動 +1，非同步重排用
}

impl Session {
    pub fn handle(&mut self, engine: &Engine, user: &mut UserModel, key: Key) -> Snapshot;
    pub fn select(&mut self, engine: &Engine, user: &mut UserModel, index: usize) -> Snapshot;
    pub fn focus_segment(&mut self, engine: &Engine, user: &mut UserModel, segment: usize) -> Snapshot;
    pub fn set_options(&mut self, options: Options);
    pub fn rerank_request(&self) -> Option<RerankRequest>;            // 上下文 + 前 k 個候選
    pub fn apply_rerank(&mut self, generation: u64, order: &[usize]) -> Option<Snapshot>;
}
```

- 殼負責：鍵碼轉 `Key`、欄位類型轉 `Options.incognito`、保存 `UserModel.export()`、畫候選與鍵盤。
- 上下文（M3）：`Session::set_context(before: &str)`，n-gram 與個人 bigram 用前文當起點。
- 組字中修改：`Left`、`Right` 或 `focus_segment` 移到某一分段，候選改列該段的字詞；選定後該段固定，其餘分段用同一把尺重算。參考 librime 的 caret 與 Mozc 的分段轉換。
- 觸控機率（M4）：`Key` 加 `Weighted(Vec<(char, f32)>)`，由 `qim-layout` 依點擊位置與按鍵幾何算出，殼轉交；`qim-core` 不知道佈局。當時平台殼還沒開工，只改網頁綁定。
- 欄位類型（M4）：`Options` 加 `field: FieldKind`（一般、email、網址、數字），由殼依平台的欄位資訊給。
- 重排：`apply_rerank` 只接受目前的 `generation`，過期結果丟棄；重排過的候選最高只到第 2 位（afm-ime 做法，brief §1.3）。M1 只有介面與網頁的假實作，M3 起接雲端 LLM。

### 2.2 資料流

```mermaid
flowchart TB
  K["Key"] --> S["Session<br/>更新輸入碼"]
  S --> L["建 lattice<br/>位置 = 輸入碼的字母邊界"]
  CT["速成碼表<br/>1–2 碼 → 字"] --> L
  LX["詞庫<br/>速成碼串 → 詞"] --> L
  UW["使用者詞<br/>輸入碼 → 詞"] --> L
  L --> V["Viterbi + beam<br/>取前 k 條路徑"]
  LM["字級 bigram"] --> V
  UM["學習計數"] --> V
  UM --> R
  V --> R["排位規則<br/>學習、自動造詞、英文候選、重排"]
  EN["英文詞表"] --> R
  R --> P["後處理<br/>簡繁、去重、分頁"]
  P --> SN["Snapshot"]
```

### 2.3 排序：同一把尺

所有候選，不論是一個長詞條、拼出來的句子，還是單字，都用同一個函式打分。不採用 Rime「有精確詞條就不造句」的規則（ADR 0001、`docs/research/rime.md` §5）。

```
score(path) = log P_bigram(輸出文字)
            + β × (路徑中屬於詞庫的段數)
            − γ × (路徑的段數)
            + Σ personal(段)
```

- `P_bigram`：字級 interpolated Kneser-Ney bigram（§4.3），只看輸出文字，與切法無關；所以「問」和「門口」、「一台」和「歹口」都在同一把尺上比。
- `β` 鼓勵詞庫裡的詞，`γ` 抑制過度切碎；初值由首選字基準的開發集調出，寫進方案檔。
- `personal`：學過的段在任何長度的連打裡都加分，只加不減。參考 libime 的 `max(靜態分, log((1−w)·靜態機率 + w·個人機率))`，`w` 初值 0.2（`docs/research/open-source-imes.md` §5）；沒學過的段為 0。個人機率由衰減值換算，公式在 M1 票 04 定。M3 再加個人 bigram（前一字詞 → 這一段）。
- 每個位置保留 beam 寬 20（libime 預設，`docs/research/ranking-techniques.md` §4）；每按一鍵只補算新增的位置。
- Unicode 擴展區的字不進 lattice；整串輸入碼就是一個字的碼時，排在一般字之後（brief §4.1）。

### 2.4 學習與排位（brief §4.5）

- 學習計數以「(輸入碼, 候選文字)」為鍵，記三個值：上屏次數、衰減值、最後上屏的 tick。tick 是上屏次數，由 `UserModel` 自己累加，不讀系統時間。衰減值參考 librime `formula_d`（時間常數 200 次上屏，brief §12.2）。
- 上屏時，整串輸入碼和路徑的每一段都各記一次，所以學過的詞在更長的連打裡也有效（§2.3 的 `personal`）。
- 排位規則依序套用在 Viterbi 結果上，比對的是整串輸入碼。次數不衰減，用來判斷門檻；衰減值只決定個人分高低，不參與門檻（衰減後選兩次只有約 1.995，永遠到不了 2）：
  1. 觸發詞完全符合的使用者詞 → 首選。
  2. 上屏次數 ≥ 2，且距最後上屏不超過 10,000 次上屏（初值）的候選 → 首選；多個符合時取最近上屏的。
  3. 上屏次數 ≥ 1 的使用者詞（含自動造詞）→ 至少第 2 位。
  4. 英文候選：輸入碼剛好是完整英文單字時放第 2 位（brief §4.3）。
- 自動造詞：一次組字分兩段以上選定後上屏，把「整串輸入碼 → 上屏文字」記成使用者詞，上屏次數 1。
- `Options.incognito` 為真時不寫任何學習資料。
- 匯出格式（對外）：純文字 TSV，第一行 `# qim-userdict v1`，欄位為詞、輸入碼、計數、最後 tick、來源（`manual` / `learned` / `auto`）、觸發詞。參考 libime 的文字碼表格式與 Rime userdb 的文字匯出；細節在 M1 票定稿。

### 2.5 輸入方案（brief §6）

方案檔是 TOML，欄位命名參考 Rime `schema.yaml`：

```toml
id = "quick"
name = "速成"
version = "1"
alphabet = "abcdefghijklmnopqrstuvwxy"
max_code_length = 2

[radicals]            # 鍵 → 字根，組字與鍵盤標籤用
a = "日"
b = "月"
# …

[packs]
code_table = "quick-table"
lexicon = "essay-lexicon"
language_model = "zh-hant-bigram"
radical_hint = "cangjie-hint"   # 字根提示用的倉頡碼，不進 lattice（ADR 0003）

[ranking]
word_bonus = 0.0      # β，M1 由基準調出
segment_penalty = 0.0 # γ
beam = 20
```

核心只認方案檔與資料包；加入倉頡、粵拼時新增方案檔與對應資料包，不改核心的排序與學習。

### 2.6 佈局：畫布模型（ADR 0004）

佈局是一張畫布，每個按鍵各自記位置、大小、旋轉、形狀和上下層順序，不靠「第幾排」決定位置。Gboard 預設佈局也只是一份資料，由 `qim-databuild` 依排列算出來。

參考的慣例：

- Delta skin：設計尺寸（`mappingSize`）、每鍵 `frame`、可擴大的觸控範圍（`extendedEdges`）、依裝置與方向分開的佈局版本（brief §12.4）。
- Trime：組字中行為欄位、JSON Schema 驗證。
- FlorisBoard keyboard3：長按清單、切頁鍵（`docs/research/open-source-imes.md` §4）。

```jsonc
{
  "format": "qim-layout/1",
  "variants": [{
    "when": { "device": "phone", "orientation": "portrait" }, // device: phone | tablet；orientation: portrait | landscape
    "canvas": { "width": 390, "height": 260 },   // 設計尺寸，等比例縮放到實際鍵盤寬度
    "grid": { "columns": 40, "rows": 24, "snap": true },
    "initialPage": "letters",
    "pages": [{
      "id": "letters",
      "keys": [{
        "id": "a",
        "frame": { "x": 12, "y": 80, "w": 36, "h": 48 },
        "rotate": 0,
        "z": 0,
        "shape": { "type": "rect", "radius": 6 },  // rect | ellipse | polygon（points 以 frame 為 0–1）
        "touchInset": { "top": 0, "bottom": 0, "left": 0, "right": 0 },
        "label": "日",
        "hint": "a",
        "a11yLabel": "日，英文 a",                  // 可省略，預設由 label 與 hint 組成
        "style": {
          "fill": { "light": "#ffffff", "dark": "#3a3a3c" },
          "labelColor": { "light": "#000000", "dark": "#ffffff" },
          "opacity": 1, "labelSize": 18, "labelOffset": [0, 0]
        },
        "tap": { "letter": "a" },
        "longPress": [{ "text": "a" }],
        "composing": null
      }]
    }]
  }]
}
```

- 佈局版本的選法：裝置與方向都符合的版本；其次方向符合的；再其次第一個版本等比例縮放（給警告）。
- 動作（`tap`、`longPress` 的項目、`swipe`）：`letter`、`text`、`backspace`、`space`、`enter`、`switchPage`、`nextKeyboard`、`toggleAscii`。
- 欄位在 M1 票定稿，之後改格式要加版本並提供轉換。
- 點擊判定在 `qim-layout`：依 `z` 由上往下，落在形狀內（含 `touchInset`）的第一個按鍵；都沒有時取距離最近的按鍵。各平台共用同一份。
- 容錯與滑動輸入用按鍵的實際位置算鄰近關係，不假設有「排」。佈局亂排時準確度下降但不會壞。
- 驗證分兩級（brief §4.6）：
  - 錯誤（不能存檔），每個佈局版本各自檢查：檔案壞掉；頁面之間切不回有字母的頁面；iOS 需要切換鍵盤鍵時卻沒有。
  - 警告（可存檔）：按鍵重疊、完全被蓋住、超出畫布、缺字母、缺少直向或橫向的版本。

### 2.7 模型分層（ADR 0005）

候選要跟手，慢的模型只能在停鍵後補強，不能改動使用者正要按下的首選。

| 層 | 時機 | 內容 | 能動到 |
| --- | --- | --- | --- |
| 同步 | 每按一鍵，在 Viterbi 內 | 碼表、詞庫、n-gram、個人模型、空間模型 | 首選 |
| 停鍵 | 停鍵後在背景執行 | 小型神經模型、端上 LLM | 最高第 2 位 |
| 雲端 | 停鍵後，非同步 | 雲端 LLM，標示來源 | 最高第 2 位 |

- 停鍵層和雲端層都走 `rerank_request` / `apply_rerank`，只認目前的 `generation`，同一串輸入碼的舊結果留快取（青簡做法）。
- 重排結果只決定名次，不寫回分數：神經分數只在同一串按鍵內可比（青簡踩過，`docs/research/open-source-imes.md` §5）。
- `qim-core` 不能開執行緒（§1.3），所以推論不放在 `qim-core`。實驗決定採用小型神經模型時另建 crate，並先改 §1.3 與 §1.4。

## 3. 平台殼

### 3.1 網頁 prototype（M0–M4）

- 目的：手感驗證核心與佈局，並在平台殼開工前驗完所有核心功能；核心是真的，網頁 UI 之後會丟，只有 `web/editor/` 佈局編輯器隨產品發佈（ADR 0002、ADR 0004）。
- 限制：只是測試頁，不能在其他 App 打字；手機上的手感用區網開在手機瀏覽器試，和真正的鍵盤仍有差距。
- 兩種模式：
  - 實體鍵盤：照 brief §4.3 的鍵位。
  - 觸控模擬：行動裝置尺寸的觸控鍵盤，照 brief §4.4。按鍵幾何由 `qim-layout` 算，網頁只畫。
- 候選列、展開面板、無痕開關、簡繁開關；按鍵放大預覽、深色模式、直向與橫向的佈局版本。
- 使用者資料存 `localStorage`；提供匯出、匯入。
- LLM：M1 只有 `rerank_request` / `apply_rerank` 的介面，加一個延遲後原樣回傳的假實作；M3 接雲端 LLM。瀏覽器呼叫不了 Apple Foundation Models，端上 LLM 到平台殼才接。網頁測試頁的 API key 只放開發機環境變數，由 Vite 開發伺服器轉發，不寫進瀏覽器儲存空間。
- 驗證：Playwright。實體鍵盤用 `page.keyboard.type`，觸控用行動裝置 viewport 加 `tap`；讀回輸入框文字與候選列並截圖。

### 3.2 macOS（M6）

InputMethodKit，Swift，經 `qim-ffi`（uniffi）呼叫核心。使用者資料存在輸入法自己的 Application Support。用 `orca computer` 在「文字編輯」與一個瀏覽器實打驗證（`CLAUDE.md`〈驗收輸入行為〉）。沒有付費帳號，先用本機簽章（brief §10）。

### 3.3 之後的平台

iOS（M7）、Android（M8）、Windows（M9）、Linux（M10）各在里程碑開頭補一節：框架、安裝與註冊、怎麼接核心、怎麼實跑驗證。iOS 的硬限制見 brief §12.3。

## 4. 碼表與資料管線

`data/sources.toml` 列出每個來源：URL、釘住的 commit 或檔案 sha256、授權、取用的檔案。`cargo xtask data` 依序執行下列步驟，產物放 `build/packs/`。每個資料包旁邊放它的 `LICENSE` 與 `NOTICE`（brief §5.1）。

### 4.1 速成碼表

1. 取 libcangjie `data/table.txt`（freedesktop GitLab，`346026b`）。
2. 只取 Big5 或 HKSCS 旗標為 1 的漢字當連打字集；其他漢字標成擴展區（只在單字查詢時出現）。
3. 由倉頡三代與五代碼推導速成碼，取聯集（brief §4.2）。
4. 字的排序先用字頻；沒有字頻的退回 libcangjie 的 Big5 排序值。
5. 另產一份字根提示資料包（字 → 倉頡三代 / 五代碼），只給字根提示用。

### 4.2 詞庫

- rime-essay（`054920d`）：先設最低詞頻門檻，初值 100，與 rime-quick 舊版相同，由基準調整；再丟掉含連打字集以外字的詞。每個字的速成碼展開成詞的碼，多碼字的組合數設上限。
- 粵語口語詞表：`data/lexicon/yue-colloquial.tsv`，CC0，自行整理，使用者審過才進資料包。
- 字頻：由 §4.3 的語料統計，與詞頻放同一個資料包。

### 4.3 字級 bigram

- 語料：中文維基與粵語維基。優先用 Wikimedia 的 CirrusSearch 內容 dump（已是純文字），dump 日期釘在 `sources.toml`；格式與可用性在 M1 票開工時查證。
- 口語語料：維基偏書面語，另加 HKCanCor 與 Tatoeba 的粵語句子（brief §5.2）。量遠小於維基，訓練時乘上權重，權重由口語集的開發切分調出。Common Voice 粵語句子要先讀完條款，可用才加。LIHKG、OpenSubtitles 權利來源不清，不用。
- 前處理：OpenCC `s2hk` 轉繁體 → 切句 → 只留連打字集內的字。
- 每份語料都切 train / dev / test：test 給首選字基準，不參與訓練。
- 訓練：字級 interpolated Kneser-Ney bigram；剪枝到資料包 ≤ 8 MB（Gboard 文獻的 5–10 MB 預算，brief §12.4）。
- 授權：語料是 CC BY-SA 4.0，這份模型隨 App 發佈是否要以相同授權散布，與 LGPL 資料並列 brief §11 的待決項，由使用者判斷。

### 4.4 英文與簡繁

- 英文：SCOWL / ESDB（`en-wl/wordlist`）size ≤ 60 的詞，size 級別當常見度。
- 簡繁：OpenCC（`fae2a0a`）的字典，經 `ferrous-opencc` 轉換。

### 4.5 驗證（`cargo xtask data --verify`，M0 起每次建置都跑）

- 不掉字：連打字集的字數等於來源中旗標符合、且有倉頡碼的字數。
- 速成碼表的每個鍵長度都是 1 或 2；不存在長度 3 以上的鍵（ADR 0003 的守門測試）。
- 抽查：`anr` 查不到「問」；「問」的速成碼含 `ar`；「丟」同時有 `hi` 與 `mi`。
- Unihan `kCangjie` 交叉比對，差異清單輸出成報表，不擋建置。
- 決定性：同樣的來源建出 sha256 相同的資料包。

## 5. 測試策略

### 5.1 核心

- 單元測試與性質測試（`cargo test`），新功能與修 bug 先寫會紅的測試。
- 性質測試例：任何 Snapshot 的候選都能由目前輸入碼打出；無痕模式下 `UserModel` 不變；`export` 後 `import` 等價。

### 5.2 首選字基準【決定】

| 檔案 | 內容 | 授權 / 位置 |
| --- | --- | --- |
| `bench/fixed.tsv` | 固定必過題：`輸入碼<TAB>期望首選<TAB>來源`；起始為 `mir 一台`、`anr 門口` | 進 git |
| `bench/corpus/` | 維基 test 切分的句子，機械轉成速成碼（多碼字取五代第一碼） | CC BY-SA 4.0，附 `ATTRIBUTION.md`，進 git |
| `bench/colloquial/` | 口語語料 test 切分的句子，轉碼方式同上；口語集的開發切分只用來調參 | 依來源（CC BY 等），附 `ATTRIBUTION.md`，進 git |
| `bench/personal/sentences.txt` | 使用者提供的真實句子，一行一句，由 `qim-databuild` 轉碼；英文段落切開不算 | 只在本機，`.gitignore`，存在就一起跑 |
| `bench/baseline.json` | 各集合（`personal` 除外）的答錯數與修正次數 | 進 git |
| `bench/personal/baseline.json` | 個人集的答錯數與修正次數 | 只在本機 |

- 指標：固定題通過數、整句首選正確率、字正確率、修正次數、每字按鍵數、首選跳動、每鍵延遲 p50 / p95（原生與 WebAssembly 各量一次）。
- 修正次數：模擬使用者打完整句輸入碼，首選能接上目標句就按空白，否則選能接上的最長候選，第一頁沒有就翻頁。每次選非首選或翻頁算一次，以每句平均回報。每字按鍵數把這些動作和輸入碼都算進去（文字輸入研究常用的 KSPC）。
- 首選跳動：逐碼打字時，首選開頭已打完的字被下一鍵改掉的次數，每句平均。先只回報，不擋 commit。
- `cargo xtask bench --check`：固定題全過，且各集合答錯數與修正次數 ≤ baseline，否則失敗。改核心或資料包時必跑。
- 更新 baseline 要用 `--update-baseline`，並在同一個 commit 說明原因。
- 使用者回報的新案例一律加進 `fixed.tsv`。

### 5.3 佈局

- 所有佈局：通過 JSON Schema 與 §2.6 的錯誤級驗證；測試雙向（造出每種錯誤會紅，只有警告不紅）。
- 內建佈局（`data/layouts/`）另加嚴格檢查：有直向與橫向的版本、淺色與深色都有顏色、按鍵不重疊、不超出畫布、字母齊全、長按清單的動作都有效。
- 點擊判定的性質測試：上層優先、旋轉與多邊形內外判定正確、縮放後結果不變。

### 5.4 平台殼

- 網頁：Playwright e2e，實體鍵盤與觸控兩種模式都要測。
- 原生殼：在真的宿主程式裡打字讀回（`CLAUDE.md`〈驗收輸入行為〉），單元測試不能取代。

## 6. 開發環境與閘門

M0 票 01 要交付：

- `docs/development.md`：環境需求、安裝、建置、〈全部檢查〉，是開發指令的單一來源。
- `cargo xtask check`：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`、`deps`（§1.3），以及網頁的 `tsc --noEmit` 與 `biome check`。
- `.githooks/`：pre-commit 跑 `cargo xtask check --fast`；commit-msg 跑 `cargo xtask commit-msg`（Conventional Commits、subject ≤ 72 字元）。以 `git config core.hooksPath .githooks` 啟用，寫進 `docs/development.md`。commit-msg 檢查附雙向測試（違規會紅、無關改動不紅）。
- `docs/progress.md` 的閘門守它〈寫法與封存〉的規則：
  - `##` 段固定是〈寫法與封存〉〈里程碑與閱讀指引〉〈Session 紀錄〉〈偏差與決定〉，各一次、依序（參考 Berth `tests/unit/test_progress_merge.py`，union 合併會安靜地抄重複）。
  - session 紀錄是一整張表。
  - 每列 / 每條不超過 200 字。
  - 條目的里程碑都是里程碑表中標「進行中」的那一個。
  - session 紀錄最後一列的「下一步」含反引號包住、`/` 開頭的指令，或以 `等使用者：` 開頭。
- 里程碑 spec 的閘門：`.scratch/` 下有 `issues/` 的目錄都要有 `spec.md`，且含〈目標〉〈範圍〉〈共同規則〉〈驗收〉〈票序〉五個 `##` 段；`.scratch/<里程碑>/` 只能是 progress 里程碑表中已開工（不是「未開始」）的里程碑。
- CI：repo 還沒有遠端。建立遠端與 GitHub Actions 屬於寫到遠端的動作，要先問使用者；接上前以 pre-commit 與〈全部檢查〉把關。

## 7. 里程碑任務

| 里程碑 | 目標 | 狀態 |
| --- | --- | --- |
| M0 骨架 | 閘門、資料管線 v0、核心 tracer bullet：在網頁打出第一個字 | 進行中 |
| M1 網頁 prototype | 競品黑箱基線；速成連打、學習、英文候選、佈局；三個 Rime 案例首選正確 | 未開始 |
| M2 平台可行性實驗 | 只量數字不做產品：iOS 記憶體上限、Foundation Models、上下文長度、小型神經模型開銷 | 未開始 |
| M3 智慧候選 | 模型階梯實驗；個人 bigram、聯想、上下文、LLM、雲端安全、記住常打字串、詞庫擴充 | 未開始 |
| M4 輸入增強 | 空間模型與容錯、英文預測與自動修正、欄位類型、智慧標點、滑動輸入、手機編輯手勢、同步的合併規則 | 未開始 |
| M5 佈局編輯器 | 畫布式視覺編輯器、單手佈局；使用者驗收關卡 | 未開始 |
| M6 macOS | 第一個平台殼，一次接上全部已完成的功能 | 未開始 |
| M7 iOS | 鍵盤擴充與主 App；多端檔案傳輸評估 | 未開始 |
| M8 Android | InputMethodService | 未開始 |
| M9 Windows | TSF | 未開始 |
| M10 Linux | Fcitx5 或 IBus | 未開始 |

排法：功能大多在共用核心裡，先在網頁把它們做完並確認可用（M1、M3、M4、M5），平台殼開工時核心介面已穩定，每個平台一次接上全部功能（M6–M10），不必跟著核心反覆改。平台限制會影響核心設計的部分先量（M2），例如智慧候選要用多大的模型。網頁驗不到的部分（真實 App 內的行為、各平台讀上下文的差異、手機真實延遲）仍可能在平台里程碑修改。brief §1.5「之後」與「待評估」的每一項都落在 M3–M8 其中一個。

### 7.1 M0 骨架

目標：repo 骨架與閘門就位；資料管線產出速成碼表；按 `a` 在網頁上看到「日」等候選，按空白上屏（tracer bullet）。

1. **骨架與閘門**：Cargo workspace、`rust-toolchain.toml`、`xtask`、`.githooks/`、`web/` 的 Vite + TS + Biome 空殼、`docs/development.md`、§6 全部閘門與它們的測試。驗收：
   - `cargo xtask check` 綠燈。
   - 違規 commit message 被擋。
   - 在暫存副本讓 `qim-core` 依賴 `qim-wasm` 或 `use std::fs` 時 `deps` 會紅。
   - 違反 progress 規則時閘門會紅。
   - 同輪更新 `CLAUDE.md` 的頂層目錄地圖。
2. **資料管線 v0**：`data/sources.toml`、`qim-databuild` 取得 libcangjie、§4.1 速成碼表、§4.5 的驗證、`postcard` 資料包格式（含格式版本號）、`data/schemes/quick.toml`。驗收：
   - `cargo xtask data --verify` 綠燈。
   - 驗證報告列出連打字集字數與擴展區字數。
   - 同樣的來源建兩次，資料包 sha256 相同。
3. **核心 tracer bullet**：`Session` 處理字母、空白、Backspace、Escape、數字選字；只做單字查詢，依 Big5 排序；`qim-wasm` 綁定；網頁有輸入框與候選列。驗收：
   - Playwright 在網頁打 `a` 讀回候選含「日」，按空白讀回輸入框為「日」。
   - 打 `ar` 讀回候選含「問」。
   - 截圖存證。

### 7.2 M1 網頁 prototype

目標：使用者任務書的階段 5，手感驗證引擎與佈局。

需要使用者提供：票 00 的裝置；票 03 開工前至少 100 句真實句子（`bench/personal/sentences.txt`，去個資）。沒拿到就在該票停下問，不阻塞其他票。

0. **競品黑箱基線**（ADR 0006）：不依賴核心程式，排在票 03 調參之前。
   - 照 `docs/research/closed-source-imes.md` §4 的 T1–T18 實測 Gboard（Android 模擬器、iPhone）、iOS 內建速成（iPhone）、macOS 內建速成，做不到的項目註明原因。開工先確認 iPhone 版 Gboard 有沒有速成（目前只有第三方整理的說法）。
   - T5 只用測試字串 `bc12345@example.edu`，不用使用者的真實 email，不登入個人帳號。只讀畫面，不開啟產品的檔案。
   - 用 `iphone-use`、`orca emulator-android`、`orca computer` 操作並讀回畫面。
   - 驗收：`docs/research/blackbox-baseline.md` 記下版本、裝置與每項結果；QIM 要達到或刻意不同的項目寫回 brief §1.3，可機械驗證的案例進 `bench/fixed.tsv`。
1. **詞庫與字頻資料包**：§4.2 的 rime-essay（含門檻）與字頻；資料包附 LICENSE。驗收：`data --verify` 加入「詞的碼都由字的速成碼組成」。
2. **連打 v1**：lattice、Viterbi + beam、§2.3 打分（先只有 unigram 與 β、γ）；`qim-bench` 與 `bench/fixed.tsv`。開工前讀 libime 的 lattice 與增量更新，記到 `docs/research/`。驗收：`anr` 首選為「門口」；`bench` 輸出 §5.2 全部指標。
3. **字級 bigram 與基準語料**：§4.3 的語料管線（含口語語料）、訓練、剪枝；`bench/corpus/`、`bench/colloquial/`、個人集與 baseline。驗收：
   - `mir` 首選為「一台」，`anr` 首選為「門口」。
   - 每鍵 p95 在原生 ≤ 10 ms、WebAssembly ≤ 50 ms（Mac 上量）。
   - bigram 資料包 ≤ 8 MB。
   - 維基、口語、個人三個集合的指標分開列出；加入口語語料後，口語集修正次數下降，維基集首選正確率的退步寫進票的 `## Comments`。
4. **使用者學習與使用者詞**：§2.4 的排位規則、§2.3 的 `personal`、自動造詞、觸發詞、無痕、TSV 匯出匯入。開工前讀 libime 怎麼把使用者資料併入 lattice（`UserLanguageModel`、`HistoryBigram`），記到 `docs/research/`。驗收：
   - 同一個碼選同一個非首選字兩次後成為首選，兩次之間隔 500 次上屏也成立。
   - 學過「門口」後，打一串含 `anr` 且原本首選不含「門口」的連打，首選含「門口」。
   - 在 `mir` 分段選「一」「台」後，下次 `mir` 的「一台」在第 2 位，再選一次成首選。
   - 無痕模式下學習資料不變。
5. **英文候選與簡繁**：SCOWL 資料包、第 2 位規則；`ferrous-opencc` 接上（先驗證能編 WebAssembly，不能就退回自讀 OpenCC 字典檔，記偏差）。驗收：
   - 打 `the` 時第 2 位是 `the`。
   - 簡體模式打 `anr` 首選為「门口」。
6. **粵語口語詞表**：`yue-colloquial.tsv` 初版，使用者審過再進資料包；固定題加入 3–5 句粵語口語。驗收：使用者審過的詞表進資料包，新增的固定題全過。
7. **佈局格式與觸控鍵盤**：
   - §2.6 的格式與 JSON Schema，定稿欄位（含佈局版本、淺色與深色、無障礙標籤）；`qim-layout` 的驗證、點擊判定、幾何計算。
   - M1 實作矩形、圓角、旋轉、上下層順序；橢圓與多邊形的欄位留好，M5 實作。
   - 預設 Gboard 速成佈局由 `qim-databuild` 算出（手機直向與橫向；字根主標籤、字母角標、長按出字母，第一排含數字）、符號頁（打完不跳回）、一個自訂頁面示範。
   - 按鍵放大預覽、深色模式。
   - 驗收：§5.3 測試綠燈；Playwright 在行動裝置 viewport 點按鍵打出「門口」，長按讀回彈出的英文字母；轉成橫向後仍打得出「門口」；切深色模式讀回按鍵顏色；一個兩鍵重疊的測試佈局，點重疊處輸出上層按鍵。
8. **組字中修改**：§2.1 的分段顯示與 `Left`、`Right`、`focus_segment`；桌面用方向鍵，觸控點組字裡的分段。驗收：Playwright 打一串連打、移到中間分段改選別的字後上屏，其他分段的字符合重算結果，讀回輸入框。
9. **prototype 整合與驗收**：實體鍵盤模式的 §4.3 鍵位、候選展開面板、組字中顯示字根、字根提示快捷鍵、重排介面的假實作。驗收（即使用者的階段 5 驗收）：
   - Playwright 實際打字並讀回畫面，貼上結果：
     - 三個 Rime 案例的首選；
     - 候選依使用次數前移；
     - 中文狀態下的英文候選；
     - 自訂頁面與長按。
   - 票 00 的測試表能在網頁做的項目在 QIM 上重跑，和競品並列。
   - `bench --check` 綠燈。

### 7.3 M2 平台可行性實驗

要回答：核心與資料包放不放得進 iOS 鍵盤擴充，端上模型與上下文能用到什麼程度。只量數字，不做產品。結果決定 M3 模型階梯的大小上限、資料包格式要不要改。

- 開工先查證：免費佈建能否在實機跑鍵盤擴充、App Group 能否使用（brief §11）。不能的話，請使用者決定要不要提前申請付費帳號，或改在模擬器做並註明限制。
- `crates/qim-ffi/` 最小版：只包到能載入資料包、處理按鍵的介面，用來量真實綁定的開銷。
- `scripts/experiments/ios-memory/`：鍵盤擴充載入核心與 M1 的資料包，在使用者的 iPhone 上量常駐記憶體，並逐步加壓找出被系統終止的上限。記下機型與 iOS 版本（上限隨機型而異，brief §12.3）。
- 同一個實驗順帶回答：Foundation Models 能否在鍵盤擴充內使用；`documentContextBeforeInput` 實際讀得到多少字。
- 小型神經模型開銷：用隨機權重的 Gboard 級（約 1.4M 參數）與青簡級（約 23M 參數）模型，在鍵盤擴充裡量載入時間、單次推論延遲、記憶體。推論引擎先試青簡用的 candle，行動端是否可行本身就是要回答的問題。
- 完成標準：
  - 四個問題都有實測結果，寫回 brief §9（記憶體目標）、§11、§12.3。
  - 放不下時，資料包改 mmap 友善格式的決定寫回本檔 §1.4、§4。

### 7.4 M3 智慧候選

- 模型階梯實驗（§7.8）先做，依結果決定同步層要不要升級語言模型、要不要做小型神經重排，以及 LLM 候選優化與雲端聯想、補全的範圍。開工前讀青簡的神經重排（`qingjian-neural`、`docs/notes/neural-rescoring.md`，GPL，只取做法）與 CyIME 的九宮格模型評估（`docs/research/cyime.md` §3 的文件，GPL，只取做法），記到 `docs/research/`。
- 個人 bigram（§2.3）；聯想；輸入框上下文配合 n-gram；自動記住 email 這類字串；常用語（與觸發詞共用格式）；emoji 候選（CLDR 繁中 / 粵語標註）。
- 雲端（brief §8）：使用者自帶 API key，結果非同步、標示來源；送出前遮蔽 email、電話、卡號；花費上限（每日請求數，預設值依 §7.8 的費用定）；排除指定 App（介面與規則放核心，平台里程碑接上）。
- 詞庫匯入；完整粵語詞庫（授權查清後，查不清就留在之後）；資料包線上更新（可關；更新來源在開工時定，要建遠端先問使用者）。
- 完成標準：
  - 上述功能在網頁可用，有 Playwright 實打紀錄。
  - 首選字基準加「有前文」的集合，正確率比沒有上下文時高，其他集合不退步。
  - 含 email、電話、卡號的測試前文，抓送出的請求內容，讀回已遮蔽；超過花費上限後不再送出，打字不受影響。
  - 各層模型的採用結論與不採用的理由寫回 brief。

### 7.5 M4 輸入增強

- 開工前讀 HeliBoard 的 `ProximityInfo`（GPL，只取做法）與 FlorisBoard v0.5.2 的滑動輸入，記到 `docs/research/`。
- 空間模型：觸控把每次點擊算成各字母的機率（高斯，依按鍵實際位置，§2.1 的 `Key::Weighted`）；實體鍵盤用鄰鍵表。容錯邊帶懲罰進 lattice，懲罰和要不要只在無候選時啟動都用基準調（`docs/research/ranking-techniques.md` §5.5）。
- 容錯（漏碼、多碼）；萬用鍵；撤銷學習；空白鍵滑動移游標；英文滑動輸入（參考 FlorisBoard v0.5.2、AnySoftKeyboard，brief §12.4）。
- 英文模式（brief §4.7）：單字預測、自動修正（退格可還原）、Shift 與句首大寫。英文 n-gram 的來源在開工時查證。
- 欄位類型：email、網址預設英文模式並關自動修正，數字欄開數字頁。
- 智慧標點（可關）：數字後的句點與逗號用半形；成對符號自動配對；英文模式連按兩下空白輸出句點。
- 評估中文滑動輸入，結論寫回 brief。
- 同步的合併規則與使用者詞庫格式（含刪除墓碑）定稿，放在核心；平台殼之後不改格式。實際傳輸方式跟平台里程碑。
- 完成標準：
  - 上述功能在網頁可用，觸控功能也在手機瀏覽器試過。
  - 首選字基準加「含打錯碼」的集合與「觸控模擬」集合（點擊位置依高斯雜訊偏移），都設 baseline；沒打錯的集合不退步。
  - Playwright 讀回：英文模式打 `teh` 修正成 `the`，退格還原；email 欄位預設英文且不修正；數字後的句點是半形。
  - 兩個瀏覽器設定檔互相同步一個新使用者詞與一筆刪除，讀回結果一致。

### 7.6 M5 佈局編輯器

畫布式視覺編輯器（ADR 0004、brief §4.6）。放在 `web/editor/`，用 `qim-layout` 的 WebAssembly 版做驗證與點擊判定；平台里程碑在主 App 內嵌同一份。

- 編輯：拖動、縮放、旋轉、拖頂點改多邊形、橢圓、調上下層順序；多選、複製、刪除、新增按鍵；復原與重做；多頁面。
- 按鍵設定：輸出、長按、滑動、切頁；顏色、透明度、文字大小與位置。
- 對齊：格線與吸附預設開，欄數列數可調並存進佈局檔；一鍵切完全自由；按住修飾鍵（手機用雙指）暫時不吸附；拖動時顯示與其他按鍵對齊的參考線。
- 佈局版本：新增直向、橫向、平板版本，可從另一個版本複製；淺色與深色分開預覽。
- 邊編輯邊試打；匯入、匯出佈局檔；從預設佈局開新佈局；「恢復預設佈局」。
- 單手佈局：內建左手、右手兩份預設佈局，用同一套格式，切換鍵放在佈局裡。
- 錯誤級問題擋存檔並指出位置，警告級只提示（§2.6）。
- 不做（之後）：曲線自由手繪、按鍵貼圖、整套外觀主題。
- 完成標準：
  - Playwright 在桌面與行動裝置 viewport 拖動、縮放、旋轉按鍵並讀回座標；開關吸附後座標符合格線或不符合。
  - 匯出再匯入後，佈局完全相同。
  - 做一份「搞怪佈局」（字母亂序、按鍵重疊、多邊形、旋轉），在觸控模式實際打出「門口」。
  - 使用者在電腦和手機瀏覽器試過編輯手感。
- **使用者驗收關卡**：M6 開工前要同時滿足三件事，不滿意的項目先修再驗：
  1. 首選字基準沒有退步。
  2. M1、M3、M4、M5 的功能都有 Playwright 實打紀錄。
  3. 使用者在電腦和手機瀏覽器試用後回覆同意（progress〈偏差與決定〉記一條）。

### 7.7 平台里程碑（M6–M10）

每個平台一次接上當時已完成的全部功能。開工時補 §3 對應小節，完成標準都是：在真的宿主程式裡實打 M1、M3、M4 的驗收項目全部通過，觸控平台的鍵盤原生繪製所有佈局功能（形狀、旋轉、透明度、重疊、深色、橫向），M5 的搞怪佈局也能打字（`CLAUDE.md`〈驗收輸入行為〉）。

每個平台另有：
- API key 存系統的安全儲存（Keychain、Android Keystore、Windows Credential Manager、Secret Service）。
- 排除指定 App：拿得到宿主識別的平台才接，拿不到的在設定頁註明。
- 無障礙：按鍵與候選讓 VoiceOver、TalkBack、Narrator 讀得到，實際開啟讀回。
- 觸控平台：震動與按鍵音（可關）。開工先查證系統限制（iOS 是否需要完整取用權限）。
- 觸控平台的實體鍵盤：偵測到實體鍵盤時收起螢幕鍵盤，只留跟隨游標的候選列，按鍵照 brief §4.3。開工先查證平台是否把實體鍵事件交給輸入法。

- **M6 macOS**：`qim-ffi` 完整版、IMK 殼、使用者資料存檔、設定視窗（含學習資料管理、API key、內嵌佈局編輯器）。依 §7.8 的結果接端上 LLM。定同步的傳輸方式（要能跨 Apple 與 Android，內容含自訂佈局；WebDAV 列入候選，CyIME 用它同步剪貼簿與備份）；接剪貼簿歷史與可選的崩潰回報（預設關，brief §8）。
- **M7 iOS**：鍵盤擴充與主 App（設定、學習資料管理、使用者詞匯入匯出、內嵌佈局編輯器，開工時實測內嵌的手感）。開工先查證鍵盤擴充拿不拿得到宿主 App 的識別。另外的完成標準：沒開完整取用權限時打字與學習照常；與 macOS 互相同步一個新使用者詞與一筆剪貼簿並讀回。做完後評估多端檔案傳輸，結論寫回 brief。
- **M8 Android**：InputMethodService 殼與設定 App（內嵌佈局編輯器）。開工先查證 Gemini Nano 能否在輸入法內使用，並讀 CyIME 的實體鍵盤與懸浮鍵盤做法（`docs/research/cyime.md` §4，GPL，只取做法）。做懸浮鍵盤（拖曳、調大小、半透明；iOS 不能畫到鍵盤外，不做）。評估語音輸入，結論寫回 brief（iOS 鍵盤擴充沒有麥克風）。另外的完成標準：與 macOS、iOS 互相同步並讀回。
- **M9 Windows**、**M10 Linux**：平台殼。

### 7.8 M3 的模型階梯實驗

要回答：同步層、停鍵層、雲端層（§2.7）各用哪個模型，修正次數和首選正確率能改善多少，延遲、大小與費用是否可接受。現在的路線從字級 bigram 直接跳到 LLM，中間的選項都要量過。

- 前置：M1 的 `rerank_request` / `apply_rerank` 介面與首選字基準；M2 的記憶體上限、Foundation Models 與小型神經模型開銷。
- 測試集：維基、口語、個人三個集合，加一個「有前文」的集合：從語料取連續句子，前一兩句當上下文，最後一句轉成速成碼。結論以口語與個人集為主，維基集只防退步。
- 受測模型：

| 層 | 模型 | 備註 |
| --- | --- | --- |
| 同步 | 字級 bigram（基線）、字級 trigram、詞級 n-gram | 同一份預算（≤ 8 MB）比較，再看放寬到 M2 上限時的增益；另量一份不設預算、可選下載的詞級 trigram（CyIME 是 105–203 MB，`docs/research/cyime.md` §3） |
| 停鍵 | 小型神經模型：Gboard 級（約 1.4M 參數）、青簡級（約 23M 參數） | 自己用 §4.3 的語料訓練，重排前 6 條路徑；放不進 M2 上限就只量桌面 |
| 停鍵 | Apple Foundation Models | 這台 Mac 上用 Swift 命令列跑（瀏覽器呼叫不了） |
| 雲端 | DeepSeek `deepseek-flash`（關閉思考模式），`deepseek-v4-pro` 當對照 | 經 OpenAI 相容介面（brief §12.4）；開工時重查模型清單與價格 |

- 提示（LLM）：上下文 + 輸入碼 + 前 10 個候選，要求模型只回序號（afm-ime 做法，`docs/research/open-source-imes.md` §5）。
- 指標：
  - 修正次數、首選正確率、「最高到第 2 位」規則下的前 2 名命中率，各和基線比。
  - 資料包或模型大小、常駐記憶體、載入時間。
  - 每次請求或推論的延遲 p50 / p95；雲端每千次請求費用。
  - 上下文長度 0、50、200 字各量一次。
- 採用門檻：拿到基線後寫進 M3 的 spec，再跑實驗，避免看到結果才定標準。
- API key 只放本機環境變數 `DEEPSEEK_API_KEY`，不進 repo；腳本缺 key 時跳過雲端部分並註明。
- 訓練出來的模型受語料授權影響，與 bigram 同列 brief §11。
- 產出：`scripts/experiments/model-ladder/` 的腳本與 README（日期、模型版本、結果表）。結論寫回 brief §12 與 §1.5，採用小型神經模型時改 §1.3、§1.4，必要時改 brief §8 的 200 字上限。

## 8. 風險與回寫點

| 風險 | 影響 | 何時確認 | 回寫 |
| --- | --- | --- | --- |
| 字級 bigram 不足以讓 `mir` 選「一台」 | M1 驗收 | M1 票 03 | 改用詞級 bigram 或其他資料，記偏差；必要時改 brief §5.2 |
| 口語語料太小，或 Common Voice 條款不許使用 | 口語集代表性、bigram 品質 | M1 票 03 | 靠個人集與 HKCanCor、Tatoeba；再不足時請使用者決定其他來源，寫回 brief §5.2 |
| 使用者句子遲遲沒到，個人集是空的 | M1 的品質判斷只剩公開語料 | M1 票 03 開工時 | 停下問使用者；其他票照常 |
| iPhone 版 Gboard 沒有速成 | 黑箱基線少一個對象 | M1 票 00 開頭 | 只測 Android 版，註明 |
| 空間模型讓正確輸入被錯字蓋過 | M4 範圍 | M4 基準 | 改成只在無候選時啟動，或只用於觸控 |
| 小型神經模型放不進鍵盤擴充或太慢 | M3 範圍、M7 | M2、§7.8 | 只在桌面與 Android 用，或不做，寫回 brief §1.5 |
| 維基語料授權（CC BY-SA）對模型的要求 | 發佈 | 上架前 | brief §11 的待決項，由使用者判斷 |
| `ferrous-opencc` 不能編 WebAssembly | M1 票 05 | 票開工時 | 改自讀 OpenCC 字典，記偏差 |
| iOS 鍵盤擴充放不下核心與資料包 | M3 的模型大小、M7 | M2 | 資料包改 mmap 友善格式（例如 `fst`），改 §1.4、§4 |
| 免費佈建不能跑鍵盤擴充 | M2、M7 | M2 開頭 | 請使用者決定提前申請帳號，或實驗改在模擬器做並註明限制 |
| CirrusSearch dump 不可用或格式改變 | M1 票 03 | 票開工時 | 改用 pages-articles 加文字抽取，記偏差 |
| 延遲超過預算 | 所有平台 | 每次 `bench` | 調 beam 或剪枝 bigram，記 baseline |
| LLM 重排提升太小或太慢 | M3 範圍 | §7.8 的實驗 | 改為只做雲端聯想或不做 LLM，寫回 brief §1.5 |
| iOS 鍵盤擴充不能用 Foundation Models | M7 的端上 LLM | M2 | iOS 只用雲端 LLM（需完整取用權限） |
| 瀏覽器呼叫雲端 LLM 被 CORS 擋 | M3 的網頁接入 | M3 開工時 | 用 Vite 開發伺服器轉發，只影響網頁測試頁 |
| 內嵌網頁編輯器在手機主 App 手感不佳 | M7、M8 | M7 開工時 | 只把手機上手感差的操作改原生；要全改原生時記偏差 |
| iOS 鍵盤擴充收不到實體鍵事件（目前只有 2017 年的第三方說法，未查證） | M7 的實體鍵盤 | M7 開工時 | iOS 不做實體鍵盤模式，設定頁註明，寫回 brief §1.5 |
| 網頁驗不到的平台差異（真實 App 行為、讀上下文、手機延遲） | M6–M10 | 各平台實打 | 修平台殼；要動核心時記偏差並重跑首選字基準 |

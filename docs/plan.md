# QIM 實作計劃

> 狀態：M0–M4 完成（`.scratch/m0/spec.md`–`.scratch/m4/spec.md`，封存 `docs/archive/M0.md`–`docs/archive/M4.md`）；M5 未開工（排在 gates、flow-sync、arch-diet 之後）；還沒開工的里程碑只有目標、範圍與完成標準。本檔只寫現況：決定改了就直接改寫，不留「原寫」「取代原」「某日改」；舊版與理由記在 progress〈偏差與決定〉，影響架構的寫 ADR。里程碑開工時，它在〈里程碑任務〉的那一節細化成 `.scratch/<里程碑>/spec.md`，這裡縮成一行連結；拆票後以票為準。需求與決定見 `docs/design-brief.md`，名詞見 `GLOSSARY.md`。

## 0. 工程慣例

已搬到 `ARCHITECTURE.md`〈識別符〉與〈主要依賴〉。

## 1. 架構總覽

已搬到 `ARCHITECTURE.md`。

### 1.1 執行形態

已搬到 `ARCHITECTURE.md`〈執行形態〉。

### 1.2 目錄結構

已搬到 `ARCHITECTURE.md`〈模組地圖〉。

### 1.3 依賴規則【決定】

已搬到 `ARCHITECTURE.md`〈依賴規則〉；鍵盤擴充不做 Schema 驗證的打算搬到 roadmap〈平台里程碑（M7–M11）〉的 M8。

### 1.4 主要依賴

已搬到 `ARCHITECTURE.md`〈主要依賴〉。

## 2. 核心

已搬到 `ARCHITECTURE.md`〈跨模組的資料與契約〉；單一行為由 `crates/qim-core/tests/` 依功能分檔說明，取捨的理由在 ADR 草稿（`.scratch/arch-diet/adr-drafts/`，票 06 編號）。逐段去處見 `.scratch/arch-diet/issues/02-core.md` 的對照表。

### 2.1 介面

已搬到 `ARCHITECTURE.md`〈核心對殼〉；前文、欄位類型、聯想、重排、按鍵機率的理由在 ADR 草稿。

### 2.2 資料流

已刪：由 `crates/qim-core/src/lattice.rs`、`session.rs` 說明；已知限制（簡繁逐段轉換）搬到 roadmap〈不排里程碑〉。

### 2.3 排序：同一把尺

已搬到 `ARCHITECTURE.md`〈候選的排位順序〉；參數的取捨在 ADR 草稿 `ranking-parameters.md`。

### 2.4 學習與排位（brief §4.5）

已搬到 `ARCHITECTURE.md`〈候選的排位順序〉〈學習資料的格式〉；格式、時限、常用語的理由在 ADR 草稿。

### 2.5 輸入方案（brief §6）

已搬到 ADR 草稿 `new-schemes-share-models.md`；方案檔的欄位由 `crates/qim-core/src/scheme.rs` 與 `data/schemes/quick.toml` 說明。

### 2.6 佈局：畫布模型（ADR 0004）

已搬到 `ARCHITECTURE.md`〈佈局格式〉；格式由 `crates/qim-layout/schema/qim-layout-1.schema.json` 與 `crates/qim-layout/tests/` 說明。

### 2.7 模型分層（ADR 0005）

已搬到 ADR 0005、`docs/research/model-ladder.md` 與 `docs/development.md`〈停鍵層的模型〉；殼的重排契約在 `ARCHITECTURE.md`〈核心對殼〉。

## 3. 平台殼

已搬到 `ARCHITECTURE.md`〈平台殼〉；只和某個平台有關的打算搬到 roadmap〈平台里程碑（M7–M11）〉。

### 3.1 網頁 prototype（M0–M4）

各殼都要守的搬到 `ARCHITECTURE.md`〈平台殼〉，在手機上試的步驟搬到 `docs/development.md`〈網頁〉；網頁自己的行為由 `web/src/` 與 `web/e2e/` 說明。

### 3.2 macOS（M7）

已搬到 roadmap〈平台里程碑（M7–M11）〉的 M7。

### 3.3 之後的平台

已搬到 roadmap〈平台里程碑（M7–M11）〉。

## 4. 碼表與資料管線

已搬走：資料包的目錄、manifest、授權檔與線上更新的契約在 `ARCHITECTURE.md`〈資料包〉；建置、驗證與發佈的步驟在 `docs/development.md`〈資料建置〉；各來源的處理規則由 `crates/qim-databuild/src/` 的程式與測試說明；取捨的理由在 `.scratch/arch-diet/adr-drafts/`（arch-diet 票 06 編號成 ADR）。

### 4.1 速成碼表

已搬走：規則在 `crates/qim-databuild/src/quick.rs`、`simplified.rs`、`radical_hint.rs`；字頻放在詞庫資料包的契約在 `ARCHITECTURE.md`〈資料包〉；理由在 `.scratch/arch-diet/adr-drafts/lexicon-and-model-tuning.md`。

### 4.2 詞庫

已搬走：規則在 `crates/qim-databuild/src/lexicon.rs`、`common_forms.rs`、`hkcancor.rs`、`words_hk.rs`；取材在 `.scratch/arch-diet/adr-drafts/data-sources-and-licenses.md`，詞頻與已知限制在 `.scratch/arch-diet/adr-drafts/lexicon-and-model-tuning.md`。

### 4.3 字級 bigram 與詞級 trigram

已搬走：規則在 `crates/qim-databuild/src/corpora.rs`、`corpus.rs`、`wikitext.rs`、`common_forms.rs`、`bigram.rs`、`word_trigram.rs`；詞級 trigram 綁預設詞庫的契約在 `ARCHITECTURE.md`〈資料包〉；調參與記憶體在 `.scratch/arch-diet/adr-drafts/lexicon-and-model-tuning.md`。

### 4.4 英文、簡繁與 emoji

已搬走：規則在 `crates/qim-databuild/src/english.rs`、`opencc.rs`、`emoji.rs` 與 `crates/qim-core/src/simplifier.rs`；取材在 `.scratch/arch-diet/adr-drafts/data-sources-and-licenses.md`，emoji 的門檻在 `.scratch/arch-diet/adr-drafts/lexicon-and-model-tuning.md`。

### 4.5 驗證（`cargo xtask data --verify`，M0 起每次改資料管線或來源都跑）

已搬走：檢查定義在 `crates/qim-databuild/src/verify.rs`，用法與報告在 `docs/development.md`〈資料建置〉。

### 4.6 資料包 manifest（M3 票 20）

已搬走：契約在 `ARCHITECTURE.md`〈資料包〉；欄位與核對在 `crates/qim-core/src/manifest.rs`、`pack.rs`，網頁的儲存在 `web/src/packs.ts`、`pack-choice.ts`；理由在 `.scratch/arch-diet/adr-drafts/pack-format-and-manifest.md`。

### 4.7 線上更新（M3 票 21）

已搬走：契約在 `ARCHITECTURE.md`〈資料包〉；索引在 `crates/qim-core/src/pack_index.rs`、`crates/qim-databuild/src/release.rs`，網頁在 `web/src/pack-update.ts`、`web/vite.config.ts`；發佈步驟在 `docs/development.md`〈線上更新的 release〉；理由在 `.scratch/arch-diet/adr-drafts/pack-online-update.md`。

## 5. 測試策略

已搬到 `ARCHITECTURE.md`〈測試策略〉。

### 5.1 核心

已搬到 `ARCHITECTURE.md`〈測試策略〉。

### 5.2 首選字基準【決定】

組成與門檻已搬到 `ARCHITECTURE.md`〈測試策略〉；檔案格式、指標與量法在 `docs/development.md`〈首選字基準〉與〈資料建置〉的〈語料〉〈個人集〉。

### 5.3 佈局

已搬到 `ARCHITECTURE.md`〈測試策略〉；iOS 預設佈局要不要放地球鍵搬到 roadmap〈平台里程碑（M7–M11）〉的 M8。

### 5.4 平台殼

已搬到 `ARCHITECTURE.md`〈測試策略〉。

## 6. 開發環境與閘門

已搬到 `docs/development.md`〈全部檢查〉〈閘門〉〈安裝〉。

## 7. 里程碑任務

搬到 `docs/roadmap.md`（2026-11-13）；這一節的編號留著，其他節的編號不變。

## 8. 風險與回寫點

| 風險 | 影響 | 何時確認 | 回寫 |
| --- | --- | --- | --- |
| 調 bigram 的參數後 `mir` 不再選「一台」（M1 票 11 換上 bigram 後仍選對，票 12 調參後也仍選對，`bench/fixed.tsv` 擋） | M1 驗收 | 改打分或資料包的票 | 改用詞級 bigram 或其他資料，記偏差；必要時改 brief §5.2 |
| 口語語料太小（M3 票 09 使用者決定先用現有語料訓練神經模型；M1 票 03：HKCanCor 加 Common Voice 約 5.6 萬句 train、2,630 句 test；票 12：權重加到 256 口語集仍在進步，資料量是瓶頸） | 口語集代表性、bigram 品質 | M1 票 12 之後由協調者問使用者 | 先靠個人集；再不足時請使用者決定其他來源（例如自存一份 Tatoeba），寫回 brief §5.2 |
| Wikimedia 的 XML dump 預告 2027-01 停用，粵語維基的釘選可能下載不到 | 乾淨建置 | 2027-01 前 | 改用 archive.org 的存檔或新的 MediaWiki Content File Exports，重釘 `sources.toml`（`docs/research/facts-tables.md`） |
| 使用者的 Rime 學習資料遲遲沒到，個人集是空的 | M1 的品質判斷只剩公開語料 | M1 票 04 開工時 | 停下問使用者；其他票照常 |
| iPhone 版 Gboard 沒有速成 | 黑箱基線少一個對象 | M1 票 09 開頭 | 只測 Android 版，註明 |
| 空間模型讓正確輸入被錯字蓋過 | M4 範圍 | M4 基準 | 改成只在無候選時啟動，或只用於觸控 |
| 小型神經模型放不進鍵盤擴充或太慢（M3 票 11 採用 1.4M、不採用 23M；M2 票 11：candle 在擴充裡能跑；1.4M 級幾乎不佔；23M 級 q8 約 +47 MB、f32 約 +95 MB，q8 上下文 200 字 p95 55 ms，以前綴快取為前提；票 09 確認上限 177 MB、核心加六份資料包 79 MB，23M q8 合計約 126 MB、f32 約 174 MB 太貼上限，M3 上限定為峰值增量 ≤ 60 MB（roadmap〈M3 的模型階梯實驗〉）；M3 票 09 實量 9 條 × 18 字：23M q8 +75 MB、p95 84 ms 超標，1.4M q8 +2 MB、11 ms；旗艦機，`docs/research/facts-platforms.md`） | M3 範圍、M8 | M2、roadmap〈M3 的模型階梯實驗〉 | 只在桌面與 Android 用，或不做，寫回 brief §1.5 |
| 維基語料授權（CC BY-SA）對模型的要求 | 發佈 | 上架前 | brief §11 的待決項，由使用者判斷 |
| iOS 鍵盤擴充放不下核心與資料包（M2 票 09：iPhone 18 Pro Max 放得下，79 MB／177 MB，格式不改；低階機沒量，§4） | M3 的模型大小、M8 | M2；低階機在 M8 | 資料包改 mmap 友善格式（例如 `fst`），改 §1.4、§4 |
| `jsonschema` 讓程式碼變大（M1 票 19：網頁 WebAssembly 由 0.42 MB 變 4.6 MB，gzip 1.2 MB，多半是它帶的 regex 與大數） | iOS 鍵盤擴充的記憶體（brief §9） | M2 | 鍵盤擴充只讀存檔時驗過的佈局，Schema 驗證留在主 App 與編輯器（`qim-layout` 加 feature 關掉），改 `ARCHITECTURE.md`〈依賴規則〉〈主要依賴〉（M2 票 09 已決定；做法寫在 roadmap〈平台里程碑（M7–M11）〉的 M8，實作在 M8） |
| 免費佈建不能跑鍵盤擴充（M2 票 06：能跑，App Group 擴充只能讀；brief §11 已決） | M2、M8 | M2 開頭 | 請使用者決定提前申請帳號，或實驗改在模擬器做並註明限制 |
| 延遲超過預算（M2：iPhone 18 Pro Max 鍵盤擴充每鍵 p95 64 µs、選開頭一段 0.79 ms，brief §9；低階機沒量） | 所有平台 | 每次 `bench` | 調 beam 或剪枝 bigram，記 baseline |
| LLM 重排提升太小或太慢（M3 票 10、11：Foundation Models 口語前 2 名只 +1.3 點、停頓後第一個請求 14.6% 超過 2 s；雲端 flash +10.4 點但比 1.4M 只多 2.0 點，都不做；票 25 在新基線重量，flash 比基線 +7.9、比 1.4M +2.4，仍不做；brief §1.5 已寫回） | M3 範圍 | roadmap〈M3 的模型階梯實驗〉 的實驗 | 票 25 對新基線重量：flash 比 1.4M 只多 2.4 點（要 3.0），維持不做；重評條件見票 22 |
| iOS 鍵盤擴充不能用 Foundation Models（M2 票 10：能用，不需完整取用；但停頓後第一個請求偶爾 6–15 s，`docs/research/facts-platforms.md`） | M8 的端上 LLM | M2 | iOS 只用雲端 LLM（需完整取用權限） |
| 瀏覽器呼叫雲端 LLM 被 CORS 擋 | 雲端重評後的網頁接入 | 重評時 | 用 Vite 開發伺服器轉發，只影響網頁測試頁；線上更新讀 GitHub release 遇到同一個問題，已這樣做（§4.7） |
| 內嵌網頁編輯器在手機主 App 手感不佳 | M8、M9 | M8 開工時 | 只把手機上手感差的操作改原生；要全改原生時記偏差 |
| iOS 鍵盤擴充收不到實體鍵事件（目前只有 2017 年的第三方說法，未查證） | M8 的實體鍵盤 | M8 開工時 | iOS 不做實體鍵盤模式，設定頁註明，寫回 brief §1.5 |
| 網頁驗不到的平台差異（真實 App 行為、讀上下文、手機延遲） | M7–M11 | 各平台實打 | 修平台殼；要動核心時記偏差並重跑首選字基準 |

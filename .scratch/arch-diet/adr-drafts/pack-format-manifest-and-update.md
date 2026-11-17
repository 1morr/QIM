# 資料包：整份解開進記憶體、manifest 放在檔外、經 GitHub Releases 的固定 tag 線上更新

> ADR 草稿，還沒編號：arch-diet 票 04 從 plan 的〈碼表與資料管線〉開頭、〈資料包 manifest〉〈線上更新〉搬來，票 06 把原本兩份（格式與 manifest、線上更新）併成這一份，使用者確認後才編號寫進 `docs/adr/`。契約本身在 `ARCHITECTURE.md`〈資料包〉，發佈步驟在 `docs/development.md`〈線上更新的 release〉。

## 格式與 manifest

資料包是 postcard，殼載入時整份解開進記憶體，不做成 mmap 友善的格式。iPhone 18 Pro Max 的鍵盤擴充上限 177 MB，六份資料包加 `Engine` 載入後 79 MB（詞庫 41.5 MB、字根提示 14.3 MB），放進 M3 的模型（roadmap〈M3 的模型階梯實驗〉）後仍在記憶體警告線（約 142 MB）以下（`docs/research/ios-feasibility.md`〈記憶體上限〉）。

每份資料包旁邊放一份 `manifest.json`（id、種類、版本、最低核心版本、大小、sha256、授權），殼交 manifest 加位元組給核心核對。研究建議把 manifest 塞進 `.pack` 檔頭（`docs/research/pack-manifest.md`），沒照做：那要改資料包格式、每份 sha256 都換，而且檔內放不了自己的 sha256。所以匯入時 manifest 與 `.pack` 一起選；放上 GitHub Releases 時改名成不撞名的 asset，安裝都走 manifest 加位元組這條路（brief §5.3）。

每個欄位照成熟產品的慣例（事實 M37）：

| 欄位 | 採用的慣例 |
| --- | --- |
| `manifest_version` | WebExtension `manifest.json` 的 `manifest_version`、Cargo index 的 `v` |
| `id` | Cargo `name` 的字元限制 |
| `version` | Cargo `vers`、npm、VS Code |
| `min_core_version` | Cargo `rust-version`、Firefox `strict_min_version`、LibreOffice minimal-version：單一下限、不寫範圍 |
| `size` | Flatpak `extra-data` 的 `size` |
| `sha256` | Cargo `cksum`、Homebrew、Flatpak；GitHub Releases asset 的 `digest`（`sha256:<hex>`）只當交叉檢查 |
| `license` | npm `license`（SPDX 運算式） |

網頁把匯入的資料包存在 IndexedDB，不存 `localStorage`：`localStorage` 每個網域約 5 MB、只能存字串，詞庫一份就 5 MB。匯入後那一種改用它，同 Firefox 裝字典後即可用。

## 線上更新

資料包不等 App 發版就能換新版（brief §5.3）。來源是 GitHub Releases（`1morr/QIM`，使用者 2026-11-11 決定）的 tag `packs` 那個 release：上面放每份資料包的檔與一份更新索引 `index.json`，殼讀索引、比出裝著的資料包有沒有新版、下載、照 manifest 核對後換上。照研究（`docs/research/pack-manifest.md` 第 6、7 點）：外部索引記每個檔的 sha256，同 Cargo index、Homebrew、Flatpak；檔案本身不帶索引資訊。GitHub asset 的 `digest` 不用：索引自己有 sha256。

- **asset 的名字**：`<id>.manifest.json`、`<id>.LICENSE`、`<id>.NOTICE` 加 id 前綴，因為 asset 名字不能重複、各份的 `LICENSE` 會撞名；不打包成 zip：`.pack` 本來就是單檔，殼也不必多一個解壓縮依賴。
- **索引的 `url`** 是相對於 `index.json` 網址的相對參照（RFC 3986），同一份索引放本機伺服器或 GitHub 都能用。
- **只換新版、不裝新的**：沒裝的資料包不裝，同 Firefox 的字典更新。
- **固定 tag `packs`**：換新版時換掉它的 asset（同 Neovim 在 GitHub 上的 `nightly` 滾動 tag），不用 `releases/latest`：App 也會在同一個 repo 發 release（brief §10 的 APK），`latest` 會指到 App 的那個。換 asset 時先傳資料包、最後傳 `index.json`：中間讀到舊索引的殼下載的檔核對不過、這次不換，下次再試。
- **網頁經開發伺服器轉發**：github.com 的轉址與 release-assets.githubusercontent.com 都不帶 `Access-Control-Allow-Origin`（2026-11-11 用 curl 帶 `Origin` 標頭實測），網頁讀不了 release，所以向自己的 `/pack-updates/` 要、由 Vite 開發伺服器轉發；`vite build` 出的靜態頁沒有轉發，也就沒有線上更新。平台殼（M7 起）沒有 CORS，直接讀 release。
- **同 id 的內建版與下載版**：取版本較高的那份；同版用內建的：內建的隨 App 發佈、已知可用，同版號照理內容相同，不同就是發佈出錯，以 App 帶的為準。比下去的下載版不刪，之後 App 內建版又比它舊（例如回退）時它會回來，所以 App 內建了更新的版本時，使用者不會卡在舊的下載版。從檔案匯入的不比版本、一律取代內建的：那是使用者自己選的。

## Consequences

- 低階機量出來放不下時，先把詞庫改成 mmap 友善的格式（例如 `fst` 存碼、詞放成一整塊字串加位移索引），見 `ARCHITECTURE.md`〈風險與回寫點〉與 `docs/research/ios-feasibility.md`〈記憶體上限〉。
- 載入的峰值比留下的多：詞級 trigram 放寬版留下 +23.4 MB、峰值 +71.3 MB，差的 48 MB 推論是解開途中的暫存（檔案位元組、`qim-ffi` 複製進 Rust 的那份、差值格式的中間陣列，沒逐項量）。要改用放寬版當預設，得先把解開改成就地轉換、再在 iPhone 量一次（數字見 `docs/research/lexicon-and-model-tuning.md`）。
- 網頁經 `qim-wasm` 已走 manifest 加位元組這條路；`qim-ffi` 還是只收位元組，M7 起的平台殼接資料包管理時改成同一條路。
- `version` 靠人加；發佈 release 時 `cargo xtask pack-release --previous` 拿上一版索引比，補上這個檢查（上面〈線上更新〉）。
- 各資料包各自發版，詞級 trigram 要和預設詞庫、語言模型同一批發（`ARCHITECTURE.md`〈資料包〉）。manifest 要不要記訓練用的詞庫 id 與版本、讓殼或建置端擋掉對不上的組合，還沒做。
- 真的在 GitHub 建 release 會公開資料包，先問使用者（`docs/development.md`〈線上更新的 release〉）。

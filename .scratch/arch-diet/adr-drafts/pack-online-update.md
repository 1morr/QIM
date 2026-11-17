# 資料包的線上更新：GitHub Releases 的固定 tag 加外部索引

> ADR 草稿，還沒編號：arch-diet 票 04 從 plan §4.7〈線上更新〉搬來，票 06 給使用者看過才編號寫進 `docs/adr/`。契約本身在 `ARCHITECTURE.md`〈資料包〉，發佈步驟在 `docs/development.md`〈線上更新的 release〉。

資料包不等 App 發版就能換新版（brief §5.3）。來源是 GitHub Releases（`1morr/QIM`，使用者 2026-11-11 決定）的 tag `packs` 那個 release：上面放每份資料包的檔與一份更新索引 `index.json`，殼讀索引、比出裝著的資料包有沒有新版、下載、照 manifest 核對後換上。照研究（`docs/research/pack-manifest.md` 第 6、7 點）：外部索引記每個檔的 sha256，同 Cargo index、Homebrew、Flatpak；檔案本身不帶索引資訊。GitHub asset 的 `digest` 不用：索引自己有 sha256。

- **asset 的名字**：`<id>.manifest.json`、`<id>.LICENSE`、`<id>.NOTICE` 加 id 前綴，因為 asset 名字不能重複、各份的 `LICENSE` 會撞名；不打包成 zip：`.pack` 本來就是單檔，殼也不必多一個解壓縮依賴。
- **索引的 `url`** 是相對於 `index.json` 網址的相對參照（RFC 3986），同一份索引放本機伺服器或 GitHub 都能用。
- **只換新版、不裝新的**：沒裝的資料包不裝，同 Firefox 的字典更新。
- **固定 tag `packs`**：換新版時換掉它的 asset（同 Neovim 在 GitHub 上的 `nightly` 滾動 tag），不用 `releases/latest`：App 也會在同一個 repo 發 release（brief §10 的 APK），`latest` 會指到 App 的那個。換 asset 時先傳資料包、最後傳 `index.json`：中間讀到舊索引的殼下載的檔核對不過、這次不換，下次再試。
- **網頁經開發伺服器轉發**：github.com 的轉址與 release-assets.githubusercontent.com 都不帶 `Access-Control-Allow-Origin`（2026-11-11 用 curl 帶 `Origin` 標頭實測），網頁讀不了 release，所以向自己的 `/pack-updates/` 要、由 Vite 開發伺服器轉發；`vite build` 出的靜態頁沒有轉發，也就沒有線上更新。平台殼（M7 起）沒有 CORS，直接讀 release。
- **同 id 的內建版與下載版**：取版本較高的那份；同版用內建的：內建的隨 App 發佈、已知可用，同版號照理內容相同，不同就是發佈出錯，以 App 帶的為準。比下去的下載版不刪，之後 App 內建版又比它舊（例如回退）時它會回來，所以 App 內建了更新的版本時，使用者不會卡在舊的下載版。從檔案匯入的不比版本、一律取代內建的：那是使用者自己選的。

## Consequences

- 各資料包各自發版。詞級 trigram 的段是用預設詞庫與當時的打分參數切的，更新詞庫或語言模型時詞級 trigram 要同一批重建、一起發，不然切法對不上；對不上不會打錯，只是首選的進步變小。manifest 要不要記訓練用的詞庫 id 與版本、讓殼或建置端擋掉對不上的組合，還沒做。
- 真的在 GitHub 建 release 會公開資料包，先問使用者（`docs/development.md`〈線上更新的 release〉）。

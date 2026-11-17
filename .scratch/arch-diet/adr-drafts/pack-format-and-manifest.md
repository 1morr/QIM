# 資料包整份解開進記憶體，manifest 放在資料包檔外面

> ADR 草稿，還沒編號：arch-diet 票 04 從 plan §4 開頭與 §4.6〈資料包 manifest〉搬來，票 06 給使用者看過才編號寫進 `docs/adr/`。契約本身在 `ARCHITECTURE.md`〈資料包〉。

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

## Consequences

- 低階機量出來放不下時，先把詞庫改成 mmap 友善的格式（例如 `fst` 存碼、詞放成一整塊字串加位移索引），見 `ARCHITECTURE.md`〈風險與回寫點〉與 `docs/research/ios-feasibility.md`〈記憶體上限〉。
- 載入的峰值比留下的多：詞級 trigram 放寬版留下 +23.4 MB、峰值 +71.3 MB，差的 48 MB 推論是解開途中的暫存（檔案位元組、`qim-ffi` 複製進 Rust 的那份、差值格式的中間陣列，沒逐項量）。要改用放寬版當預設，得先把解開改成就地轉換、再在 iPhone 量一次（數字見〈詞庫與語言模型的取捨與調參〉草稿）。
- 網頁經 `qim-wasm` 已走 manifest 加位元組這條路；`qim-ffi` 還是只收位元組，M7 起的平台殼接資料包管理時改成同一條路。
- `version` 靠人加；發佈 release 時 `cargo xtask pack-release --previous` 拿上一版索引比，補上這個檢查（〈資料包的線上更新〉草稿）。

# 資料包 manifest 與打包格式：成熟產品的慣例

查證日期：2026-11-11。只信官方文件與原始碼；GitHub 原始碼一律釘 commit SHA。「查不到」表示在官方文件與原始碼都沒找到，沒有猜。
標示「推論」的句子是從查到的事實推出來的，不是來源原文。

## 結論摘要

1. 有版本、相容性、校驗碼欄位的成熟產品（Cargo、npm、Homebrew、Flatpak、GitHub Releases、LibreOffice、Firefox、VS Code）都把「身分、版本、最低相容版本」放在一份機器可讀的文字 manifest。Rime plum 沒有這些欄位，只靠 git branch，不能當範本。
2. 校驗碼最普遍的是 SHA-256。Cargo `cksum`、Homebrew、Flatpak 都是 sha256；GitHub Releases 的 asset `digest` 是 `sha256:<hex>`；只有 npm 用 SRI（`sha512-<base64>`）。建議 manifest 寫 `sha256`：64 字元小寫 hex，對 `.pack` 檔的整份位元組算，並同時存 `size`（位元組）。Flatpak 只有 `extra-data` 有 `size`，Cargo 和 Homebrew 沒有。
3. 最低程式版本，Cargo `rust-version`、Firefox `strict_min_version`、LibreOffice `*-minimal-version` 都是「單一最低版本、不含運算子」；npm `engines` 和 VS Code `engines.vscode` 是 semver range。建議 `min_core_version = "0.3"`，bare version、無運算子、可寫兩段（同 Cargo），比較規則是「核心版本 >= 它」。
4. 資料包本身的 `version` 用 semver 2.0.0（Cargo `vers`、npm、VS Code 都是），欄位名用 `version`，`id` 限制字元集（Cargo `name` 只允許英數、`-`、`_`）。授權用 SPDX 表達式（npm `license`；VS Code 有 `SEE LICENSE IN <file>` 退路）。
5. 欄位名建議：`schema`（整數，對應 Cargo index 的 `v`，供舊核心忽略看不懂的 manifest）、`id`、`kind`、`version`、`min_core_version`、`sha256`、`size`、`license`。
6. 檔名與格式：manifest 用 JSON（Cargo index、npm、Firefox、GitHub API 都是 JSON；LibreOffice 用 XML 是歷史包袱）。線上索引叫 `index.json`，裡面一個 `packs[]`，每項就是上面那組欄位加 `url`（或由 release tag + 檔名推出）。
7. 單檔還是多檔：LibreOffice `.oxt`、Firefox `.xpi`、VS Code `.vsix` 都是「zip 裡放 manifest + 授權」；Cargo、npm、Homebrew、Flatpak 則是「外部索引記錄檔案的校驗碼，檔案本身不含索引資訊」。`.pack` 已是 postcard 單檔，建議維持單檔，不再包一層 zip。匯入單檔時沒有索引可對，所以 `.pack` 檔頭要自帶 `schema`、`id`、`kind`、`version`、`min_core_version`。librime 的 `Rime::Table/4.0` 檔頭有先例（見第 1 節）。校驗碼、`size` 只放索引，不放檔內，因為檔內放不了自己的雜湊。LICENSE 與 NOTICE 作為 release 的獨立資產，索引的 `license` 欄位寫 SPDX id（以上三點是推論，不是來源原文）。
8. GitHub asset 的 `digest` 在 2025-06-03 之後上傳的資產才有，之前的可能是 `null`（第 6 節）。索引自己記 `sha256`，下載後自行驗證；`digest` 只當額外的交叉檢查。

---

## 1. Rime：plum 與 schema / dict yaml

### plum 的套件長相

- plum 的 README 沒有為「套件」下正式定義。原文：「A package may contain one or several interrelated input schemata and their affiliated Rime dictionaries.」套件主要放在 GitHub（`lotem/rime-zhung` 這種 owner/repo 寫法），README 說 Makefile「uses `git` command to download the latest packages from GitHub」。
- 套件指定語法（README）：`jyutping`（短名）、`lotem/rime-zhung`、`jyutping@master`（`@` 後是 branch）、`lotem/rime-forge@master/lotem-packages.conf`、`*-packages.conf` 套件清單檔。
- README 有「recipe（配方）」一詞：「a re-usable piece of configuration is known as a **recipe**」，並說「A data package itself can be a recipe, this is the common case.」
- 版本、校驗碼、授權欄位：README 沒有版本欄位，唯一的釘版方式是 `@<branch>`；沒有校驗碼；授權不是結構化欄位，README 原文「Individual packages can be released under different licenses. Please refer to their respective `LICENSE` files.」所以 plum 的套件沒有 manifest 的 version / 校驗碼 / 授權欄位。
- 最低 librime 版本：plum 只在 README 的 Build / Run-time dependencies 寫 `librime>=1.3`、`opencc>=1.0.2`，那是 plum 這個工具本身的需求，不是套件的欄位。

來源（plum commit `b1be1969f914cc005add4090631b855db00c2591`，2026-05-08）：
- https://github.com/rime/plum/blob/b1be1969f914cc005add4090631b855db00c2591/README.md
- https://github.com/rime/plum/blob/b1be1969f914cc005add4090631b855db00c2591/scripts/recipe.sh
- https://github.com/rime/plum/tree/b1be1969f914cc005add4090631b855db00c2591

### `*.recipe.yaml` 的欄位

找不到獨立的 recipe 規格文件（plum 的 `recipe.md` 回 404）。以下是 `scripts/recipe.sh` 實際解析的頂層鍵，和 `rime/rime-custom` 的範例檔：

- `recipe:` 區塊，內有 `Rx`（recipe 名稱，必須和檔名一致，否則報 `Invalid recipe`）、`args`（參數名清單）、`description`。
- `download_files`（URL 清單，可寫 `檔名::URL`，用 curl 下載）。
- `install_files`（要複製進 Rime 使用者目錄的檔案 glob）。
- `patch_files`（對 `*.custom.yaml` 打補丁，每個補丁以 `# Rx: ... {` 註記包起來）。
- recipe 裡沒有 version、min librime、checksum、license 欄位。

來源：
- https://github.com/rime/plum/blob/b1be1969f914cc005add4090631b855db00c2591/scripts/recipe.sh
- https://github.com/rime/rime-custom/blob/9cb580f9e42fe27c72427a796d5ac0b720a790ea/use_key_bindings.recipe.yaml

### schema yaml 的 `schema:` 區塊

以 rime-cangjie（commit `fcb3d7dd94ac1304663d575727d24d8107f6b082`）的 `cangjie5.schema.yaml` 為例，實際檔案：

```yaml
schema:
  schema_id: cangjie5
  name: 倉頡五代
  version: "1.0"
  author:
    - 發明人 朱邦復先生
  description: |
    第五代倉頡輸入法 ...
  dependencies:
    - luna_quanpin
```

- librime 讀 `schema/dependencies`（清單），在部署時只往下編一層相依方案；缺相依時記 warning 並略過。`schema/version` 由 `ConfigFileUpdate` 使用（部署時比對、更新 build 目錄的檔案）。
- 我只查到範例檔與 librime 原始碼，沒找到官方文件逐欄定義 `schema:` 區塊；`name`、`author`、`description` 的語意是從範例讀出來的。
- `version` 在範例裡是帶引號的字串 `"1.0"`。

來源：
- https://github.com/rime/rime-cangjie/blob/fcb3d7dd94ac1304663d575727d24d8107f6b082/cangjie5.schema.yaml
- https://github.com/rime/librime/blob/c95c0d2390c827bf44e5eb8989704ba723ec0d0c/src/rime/lever/deployment_tasks.cc （第 207-274 行處理 dependencies；第 224、389 行用 `schema/version`）

### dict yaml

同一個 repo 的 `cangjie5.dict.yaml`，檔頭 `---` 後：

```yaml
name: "cangjie5"
version: "1.0"
sort: by_weight
use_preset_vocabulary: true
max_phrase_length: 7
min_phrase_weight: 100
```

授權只寫在檔頭註解（`# License: GPL`）和 repo 的 `LICENSE` 檔，沒有結構化欄位。

來源：https://github.com/rime/rime-cangjie/blob/fcb3d7dd94ac1304663d575727d24d8107f6b082/cangjie5.dict.yaml

### 二進位檔頭的先例（librime 編譯產物）

librime 編出來的 `Rime::Table` 二進位檔，檔頭有 `format` 字串（`"Rime::Table/4.0"`）和 `dict_file_checksum`（uint32）；讀檔時比對 `kTableFormatLowestCompatible`，格式版本低於它就報錯。這是「二進位檔自帶格式版本」的前例。

來源：https://github.com/rime/librime/blob/c95c0d2390c827bf44e5eb8989704ba723ec0d0c/src/rime/dict/table.cc （第 18-22、287-294 行）、https://github.com/rime/librime/blob/c95c0d2390c827bf44e5eb8989704ba723ec0d0c/src/rime/dict/table.h

---

## 2. LibreOffice 擴充（.oxt）

- 打包：`.oxt` 是 zip 容器。Apache OpenOffice wiki 只寫「The file has the file extension “oxt” (formerly .uno.pkg and .zip), and it acts as a container for various items」，沒有明說是 zip 格式；官方字典擴充 repo 裡 `META-INF/manifest.xml` 列出 `dictionaries.xcu` 等檔案（見下）。zip 這點我沒找到官方文字，是從舊副檔名 `.zip` 與業界通識推斷，標為「推論」。
- `description.xml` 欄位（Apache OpenOffice wiki 的元素說明）：
  - `<identifier value="..."/>`：「Required. The extension identifier.」
  - `<version value="..."/>`：「Required. A textual representation of the extension version.」
  - `<display-name><name lang="...">...</name></display-name>`：本地化顯示名稱，字串長度須大於 0 且不能全是空白。
  - `<dependencies>` 內的 `OpenOffice.org-minimal-version`（`value` 必填，wiki 說「You should not specify a particular micro version, e.g. “2.2.1”, use “2.2” instead.」）與 `OpenOffice.org-maximal-version`。
  - `<registration><simple-license accept-by="user|admin"> <license-text xlink:href="..." lang="..."/>`：授權全文是擴充內的檔案，安裝時顯示並要使用者同意；不是 SPDX 欄位。
  - `<update-information><src xlink:href="..."/>`：更新資訊的 URL（可以是 atom feed），第二個以後是 mirror 備援。
  - `<publisher>`、`<platform value="all"/>`、`<icon>`、`<extension-description>`。
- `LibreOffice-minimal-version` 這個元素，Apache OpenOffice wiki 沒有（「Not mentioned」）；它在 LibreOffice 原始碼：命名空間 `http://libreoffice.org/extensions/description/2011`，範例是 `<l:LibreOffice-minimal-version value="4.1" d:name="LibreOffice 4.1" .../>`。原始碼註解：只要有 `LibreOffice-minimal-version`，就忽略舊的 OOo 相依。另有 `LibreOffice-maximal-version`。
- 比較語法：單一最低版本，不是 range；最低與最高是兩個獨立元素。`dp_misc::compareVersions` 以 `.` 分段，每段去掉前導 0 後先比長度再比字串，`satisfiesMinimalVersion` 為「實際版本不小於指定版本」，`satisfiesMaximalVersion` 為「不大於」。
- 官方字典擴充的實例（`LibreOffice/dictionaries` 的 `en/description.xml`）：`<version value="2025.04.01"/>`（日期式版本，不是 semver）、`<identifier value="org.openoffice.en.hunspell.dictionaries"/>`、`<OpenOffice.org-minimal-version value="3.0" .../>`。擴充裡的資料檔以 `META-INF/manifest.xml` 的 `file-entry`（`media-type` + `full-path`）註冊。沒有校驗碼欄位，`description.xml` 沒有 sha / size 欄位。

來源：
- https://wiki.openoffice.org/wiki/Documentation/DevGuide/Extensions/Description_of_XML_Elements （查證 2026-11-11）
- https://wiki.openoffice.org/wiki/Documentation/DevGuide/Extensions/Extensions
- https://github.com/LibreOffice/core/blob/b99c328d8a4dd71e15ec25115be7a8c767ba9858/desktop/source/deployment/misc/dp_dependencies.cxx
- https://github.com/LibreOffice/core/blob/b99c328d8a4dd71e15ec25115be7a8c767ba9858/desktop/source/deployment/misc/dp_version.cxx
- https://github.com/LibreOffice/core/blob/b99c328d8a4dd71e15ec25115be7a8c767ba9858/odk/examples/DevelopersGuide/Components/SimpleLicense/description.xml
- https://github.com/LibreOffice/dictionaries/blob/c0865a85eb67dc377f2cf8e8fe2c4fa9e53f0dcb/en/description.xml
- https://github.com/LibreOffice/dictionaries/blob/c0865a85eb67dc377f2cf8e8fe2c4fa9e53f0dcb/en/META-INF/manifest.xml

wiki.documentfoundation.org 的 DevGuide 頁被 Anubis 擋（Access Denied），所以欄位語意引用 Apache OpenOffice wiki，LibreOffice 專有部分引用原始碼。

---

## 3. Firefox 字典 / 語言包 WebExtension

- 打包：xpi 是 zip 換副檔名。原文：「Packaged extensions in Firefox are called "XPI files", which are ZIP files with a different extension.」且「The ZIP file must be a ZIP of the extension's files themselves, not of the directory containing them.」（`manifest.json` 在 zip 根目錄是從這句推出，頁面沒有明說。）
- `dictionaries`（manifest 欄位，非必填，manifest_version 2 以上）：把 locale code 對應到字典檔：`"dictionaries": {"en-US": "dictionaries/en-US.dic"}`。一份字典是 `.dic` + `.aff` 兩個檔，manifest 只寫 `.dic`。使用 `dictionaries` 時必須設 `browser_specific_settings.gecko.id`。
- `version`：「1 to 4 numbers separated by dots」，非零數字不可有前導 0（`2.01` 不合法）；比較時由左到右逐段數值比較，缺的段視為 0（`1.0` 等於 `1.0.0.0`，`1.10` 比 `1.9` 新）。AMO 的 regex：`^(0|[1-9][0-9]{0,8})([.](0|[1-9][0-9]{0,8})){0,3}$`。Chrome Web Store 每段限 0-65535。
- `browser_specific_settings.gecko.strict_min_version`：單一最低版本，字串如 `"128.0"`；原文「If the Firefox version ... is below this version, the extension is not installed or not run.」`"*"` 在這欄位不合法。另有 `strict_max_version`（預設 `"*"`，可寫 `"159.*"`），以及 `gecko_android` 區塊。
- `id` 可以是 email 形式（`^[a-zA-Z0-9-._]*@[a-zA-Z0-9-._]+$`，80 字元以內）或 GUID。
- 沒有校驗碼、size、授權欄位（`license` 不在這幾頁）；完整性由 AMO 簽章負責（簽章細節沒查）。

來源（查證 2026-11-11）：
- https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/manifest.json/dictionaries
- https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/manifest.json/browser_specific_settings
- https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/manifest.json/version
- https://extensionworkshop.com/documentation/publish/package-your-extension/

---

## 4. Gboard 與 Android 其他可下載資料

- Gboard 語言包格式：**查不到公開格式**。Gboard Help 只教使用者在「Languages」設定裡選語言（https://support.google.com/gboard/answer/6380730 ，2026-11-11 讀），沒有說明下載的檔案格式、manifest 或索引。Google Search 社群串有人提到 Gboard 日文有「Japanese language pack」要下載（https://support.google.com/websearch/thread/269288736 ），同樣沒有格式說明。
- 與 Gboard 有關但屬於使用者辭書：Mozc 的匯入程式認得檔頭 `# Gboard Dictionary version:1` 的文字檔（`GBOARD_V1`）。見第 5 節。
- Play Asset Delivery（https://developer.android.com/guide/playcore/asset-delivery ，2026-11-11）：資產包有 `install-time`、`fast-follow`、`on-demand` 三種交付模式。`install-time` 以 split APK 交付；`fast-follow` 與 `on-demand` 以壓縮檔交付並解開到內部儲存空間。App 更新時，已下載的 fast-follow / on-demand 包會先失效再套用 patch。版本綁在 app 版本，不是獨立版本號。頁面沒有講校驗碼，只說解開的檔案要視為唯讀，「because asset pack patches depend on the integrity of these files」。宣告方式（`build.gradle` 的 `assetPacks`、manifest）該頁沒有，沒查。
- AICore / Gemini Nano（https://developer.android.com/ai/gemini-nano ，2026-11-11）：模型由系統服務 AICore 管理下載與更新，頁面原文「AICore manages the distribution of Gemini Nano and handles future updates.」；模型下載經由 Private Compute Services APK 轉送。沒有公開的套件或 manifest 格式，App 不接觸模型檔。對我們的參考價值低：它是系統代管，不是第三方可發佈的格式。

---

## 5. Mozc

- 使用者辭書匯入：`src/client/user_dictionary/user_dictionary_importer.{h,cc}`。
  - 支援的來源（`IMEType` enum）：`MOZC`、`MSIME`、`ATOK`、`KOTOERI`、`GBOARD_V1`，自動偵測。
  - 文字格式是 tab 分隔，至少 3 欄：`key`（讀音）、`value`（詞）、`pos`（詞性），第 4 欄 `comment` 選填。Mozc 的 `pos` 可編碼語系，如 `名詞:en`（`RawEntry` 的註解）。
  - 註解行：MS-IME / ATOK 以 `!` 開頭、Mozc / Gboard 以 `#` 開頭、Kotoeri 以 `//` 開頭。偵測依據：`!Microsoft IME` 開頭視為 MSIME、`!!DICUT`（舊 ATOK 格式不支援）、`!!ATOK_TANGO_TEXT_HEADER` 視為 ATOK、`# gboard dictionary version:1` 視為 Gboard。
  - 編碼：`UTF8`、`UTF16` 等，自動偵測。
- 內建系統辭書（原始碼，非使用者匯入）：`src/data/dictionary_oss/dictionary00.txt` 等，tab 分隔欄位 `讀音 \t lid \t rid \t cost \t 詞`（第一行實例：`ああると	1851	1851	7129	アアルト`）。`README.md` 另說明 `aux_dictionary.tsv`（key、value、base_key、base_value、cost_offset）與 `dictionary_filter.tsv`。這些是建置期輸入，由 Mozc 一起編進安裝檔。
- 可下載的字典套件：**查不到**。在 repo 的 `docs/` 與 `src/data/` 找不到線上下載字典套件的機制或 manifest；辭書跟程式一起發佈。
- 使用者辭書的儲存格式是 protobuf（`protocol/user_dictionary_storage.pb.h`，只從匯入程式 include 行確認，未讀 proto 本身）。

來源（Mozc commit `a4541df29db443b49f81cf54665873b68fde9efa`，2026-11-09）：
- https://github.com/google/mozc/blob/a4541df29db443b49f81cf54665873b68fde9efa/src/client/user_dictionary/user_dictionary_importer.h
- https://github.com/google/mozc/blob/a4541df29db443b49f81cf54665873b68fde9efa/src/client/user_dictionary/user_dictionary_importer.cc
- https://github.com/google/mozc/blob/a4541df29db443b49f81cf54665873b68fde9efa/src/data/dictionary_oss/README.md

---

## 6. 校驗碼與大小的慣例

| 系統 | 欄位 | 演算法 / 寫法 | size 欄位 | 來源 |
| --- | --- | --- | --- | --- |
| npm `package-lock.json` | `integrity` | SRI 字串，文件原文「A `sha512` or `sha1` Standard Subresource Integrity string for the artifact that was unpacked in this location.」；寫法 `sha512-<base64>` | 無 | https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json |
| W3C SRI | `hash-expression = algo "-" base64-value` | 瀏覽器必須支援 SHA-256 / 384 / 512，多個 hash 時用最強的 | 無 | https://www.w3.org/TR/SRI/ |
| Homebrew formula | `sha256` | formula 的 `url` / `sha256` / `license` 一組；Cookbook 只提到用 `brew bump-formula-pr` 更新 `url` 與 `sha256`，沒有寫編碼，實際是 64 字元 hex（編碼為推論） | 無 | https://docs.brew.sh/Formula-Cookbook |
| Flatpak manifest | `sha256`、`sha512`（`archive` 與 `file` 來源，「verified after download」） | hex | 只有 `extra-data` 來源有 `size`（「The size of the extra data in bytes.」）和 `sha256`；一般 `archive` / `file` 來源沒有 `size` | https://docs.flatpak.org/en/latest/flatpak-builder-command-reference.html |
| Cargo registry index | `cksum` | 「A SHA256 checksum of the `.crate` file.」範例是 64 字元小寫 hex（文件沒明說編碼，hex 為從範例的推論） | 無 | https://doc.rust-lang.org/cargo/reference/registry-index.html |
| GitHub Releases REST API | `size`（integer，位元組）、`digest`（string 或 null） | 格式 `sha256:<hex>`，見下 | 有 | https://docs.github.com/en/rest/releases/assets |

GitHub asset `digest` 的細節：
- 文件範例：`"size": 1024, "digest": "sha256:2151b604e3429bff440b9fbc03eb3617bc2603cda96c95b9bb05277f9ddba255"`（https://docs.github.com/rest/releases/assets 搜尋摘要）。REST 文件的欄位表寫 `digest` 為「string or null」。
- 實測（`gh api`，2026-11-11）：`rime/librime` 1.17.0 與 `BurntSushi/ripgrep` 15.2.0 的資產都回傳 `"digest":"sha256:<64 hex>"` 與整數 `size`。
- GitHub changelog（2025-06-03）：「GitHub now automatically computes and displays SHA256 checksums (digests) for all uploaded release assets. These digests are generated at upload time, immutable ...」，可在 Releases UI、REST、GraphQL、`gh` CLI 取得。GraphQL 的 `ReleaseAsset.digest` 說明是「The SHA256 digest of the asset.」（`String`，可為 null）。
- 因為是 2025-06 起才有、且型別含 null，舊資產的 `digest` 可能是 null（推論；我沒有找到舊資產 null 的官方說明）。
- 相關：GitHub 另有 immutable releases（資產與 tag 不可更動、有 release attestation），見 https://github.blog/changelog/2025-08-15-github-actions-policy-now-supports-blocking-and-sha-pinning-actions （只讀搜尋摘要）。

來源：
- https://github.blog/changelog/2025-06-03-releases-now-expose-digests-for-release-assets
- https://docs.github.com/en/graphql/reference/releases

---

## 7. 「最低程式版本」欄位命名與比較語法

| 系統 | 欄位 | 值的語法 | 比較語意 | 來源 |
| --- | --- | --- | --- | --- |
| VS Code 擴充 | `engines.vscode` | semver range：`1.8.0`（無 caret）= 只相容 1.8.0；`^1.8.0` = 1.8.0 以上 | range | https://code.visualstudio.com/api/working-with-extensions/publishing-extension |
| npm | `engines`（例 `"node": ">=0.10.3 <15"`） | semver range；預設只是警告（除非 `engine-strict`） | range，還可含上限 | https://docs.npmjs.com/cli/v11/configuring-npm/package-json |
| Firefox | `browser_specific_settings.gecko.strict_min_version`（另有 `strict_max_version`） | 字串 `"128.0"`；`"*"` 不合法 | 單一下限；低於則不安裝 | https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/manifest.json/browser_specific_settings |
| LibreOffice | `<dependencies>` 內 `OpenOffice.org-minimal-version` / `LibreOffice-minimal-version`（另有 `*-maximal-version`） | `value="4.1"`，不要寫 micro | 單一下限，實際版本不小於指定值即滿足 | 第 2 節 |
| Cargo | `rust-version`（Cargo.toml）、index 的 `rust_version` | 「bare version number with at least one component; it cannot include semver operators or pre-release identifiers」，如 `"1.56"` | 單一下限；index 註解「must be a valid version requirement without an operator」 | https://doc.rust-lang.org/cargo/reference/rust-version.html 、https://doc.rust-lang.org/cargo/reference/registry-index.html |

- 兩種流派：semver range（npm、VS Code）與單一下限（Firefox、LibreOffice、Cargo）。資料包只有「核心至少要多新」一個問題，單一下限就夠，也不需要 range 解析器。
- 命名：`engines.*`（npm / VS Code）、`*_min_version` / `*-minimal-version`（Firefox / LibreOffice）、`rust-version`（Cargo）。「min」「minimal」類命名最直白。
- VS Code 對 `engines.vscode` 是否禁止 `*`：該頁沒提，查不到。
- Cargo 對兩段版本（`"1.56"`）如何補齊：頁面沒說，查不到。若我們也允許兩段，要自己在規格裡寫死（建議補成 `.0`）。

---

## 附：未查或查不到的項目

- Rime recipe 的獨立規格文件：查不到（`recipe.md` 404），欄位來自 `recipe.sh` 與範例。
- LibreOffice DocumentFoundation wiki 頁：被 Anubis 擋，改引 Apache OpenOffice wiki 與原始碼。
- `.oxt` 是 zip：官方文字只寫副檔名容器，zip 為推論。
- Gboard 語言包格式：查不到。
- Mozc 線上字典套件：查不到。
- Firefox / AMO 簽章細節、Play Asset Delivery 的宣告方式與完整性驗證：未查。
- Homebrew `sha256` 的編碼：Cookbook 沒寫，未查 formula API 文件。

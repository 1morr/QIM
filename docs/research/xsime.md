# xsime（RIMES）查證紀錄

查證日期 2026-11-03，對象 [scholay-xsime/xsime](https://github.com/scholay-xsime/xsime) commit `6c657da3013a`（2026-11-01）。只讀 README、平台與流程文件、`validation/` 與少數原始碼，沒有編譯、安裝或執行任何腳本。以下數字都是該專案自己量的，沒有重跑。路徑相對 xsime repo 根；專案內部名稱為 RIMES。

## 1. 專案概況與授權

- 全平台 Rime 前端：macOS（IMK）、iOS（鍵盤擴充）、Android（InputMethodService）、Windows（TSF）、Linux（Fcitx5）。引擎只有 librime，沒有 WASM / uniffi / 統一 C ABI，各平台各寫一份橋接（`docs/architecture/system.md:6-8`）；跨端靠行為、格式與回歸樣本對齊（同文件 :61-62）。
- 自有程式碼 Apache-2.0（`LICENSE`、`LICENSING.md`），版權 `Copyright 2026 scholay`（`NOTICE`）。與 QIM 授權相同：QIM 可以參考，也可以取用程式碼，但要依 Apache-2.0 保留版權與 NOTICE 署名、標明修改（`LICENSING.md`「分发与署名」）。
- 歷史上曾是 MIT：保留全文於 `LICENSES/MIT-legacy.txt`，不追溯、不雙重授權。取用舊版（MIT 時期）程式碼時，兩種授權的署名都要留。
- 資料不能一概而論：librime 為 BSD-3-Clause；`rime-data/licenses/` 內 rime-ice 是 GPL-3.0-only（`rime-ice-SOURCE.md`），另有 rime-wubi、rime-easy-en、rime-octagram-data、yoyo 各自的來源說明。`LICENSING.md` 明說 GPL / LGPL / CC 義務不被根目錄 Apache-2.0 取代。QIM 取用任何詞庫、方案前，要逐項看來源授權（ADR 0006 的邊界）。
- 授權清單由 `THIRD_PARTY_NOTICES.md`（來源 URL、版本、revision、版權行）與 `LICENSES/<元件>-<版本>-<SPDX>.txt` 組成；打包時整份複製進 App（`.github/workflows/release.yml:335-336`）。沒有找到檢查清單與檔案是否一致的測試。

## 2. 平台殼

### 2.1 Windows

- TSF DLL 很小，不載 librime、不做網路 I/O；librime 在獨立的 `RimesBroker.exe`，用 named pipe 通訊；Broker 失敗時按鍵放行（fail-open）（`platforms/windows/native/README.md:13-28`）。TSF 端首次使用會啟動同目錄 Broker，崩潰後重連（同文件 :19-24）。
- 沙盒應用（AppContainer / LPAC）走另一條 `.input` pipe，只開普通輸入，沒有 Buffer、設定、AI；沙盒端不啟動外部程序，Broker 須已在跑（同文件 :39-60）。要註冊 `IMMERSIVESUPPORT`、`UIELEMENTENABLED`（同文件 :33-35）。
- 宿主可能在 test callback 持有 text-store 鎖，所以預判按鍵時不讀文件；真正的 key callback 才在讀鎖下驗 input scope（同文件 :121-123）。
- x86 與 x64 TSF DLL 都要建、都要註冊，共用 x64 Broker（同文件 :295-322；`docs/platforms/windows/manual-testing.md`）。
- 對 QIM 的意義：M10 要比較「Rust 核心直接載入 TSF DLL」與「薄 TSF DLL + 程序外引擎」。前者的風險是引擎崩潰拖垮宿主，且 x86 宿主要另外建；後者多一條 IPC 與冷啟動等待（CHANGELOG 有冷引擎回覆等待、首鍵傳輸失敗兩類修正）。Windows EXE 目前未簽章。

### 2.2 macOS

- 簽章：ad-hoc 簽章的 designated requirement 是 cdhash，每次重建都變，輔助功能授權失效但系統設定仍顯示勾選；改用 Apple Development 憑證，requirement 指向憑證，授權可跨重建保留。開發通道刻意不選 Developer ID（`build_install.sh:254-285`）。查證見 §4。
- 輸入源註冊：沙盒 App 靠需要 root 才能重建的 `/System/Library/Caches/com.apple.IntlDataCache.le*` 認輸入法；1.1.1 前安裝器只以使用者身分註冊，之後 pkg `postinstall` 以 root 再註冊一次（`docs/macos/input-source-cache-recovery.md:11,53-55`；`scripts/pkg/scripts/postinstall:80-84,366-369`，實際呼叫在 `Sources/RimeBuffer/InputSourceInstaller.swift:322` 的 `TISRegisterInputSource`，由 `--rimes-tis-register` 觸發）。macOS 27.0 上沙盒版微信 / QQ 切到該輸入法會閃退（`CFRelease(NULL)`），維護者未重現，文件只是紀錄回報。
- 更新：安裝前執行 `RIMES --prepare-update` 把當前輸入源切到 ASCII 備援，失敗則拒絕熱替換（`build_install.sh:288,342`；`Sources/RimeBuffer/main.swift:1391,2034-2071`）；CI 斷言 `build_install.sh` 至少出現兩次（`.github/workflows/ci.yml:190`）。`.new` / `.bak` 原子替換與還原函式：本次只確認用途描述，未逐行核對。
- `dev-reload.sh` 只換二進位並重啟輸入法行程，不 `lsregister`、不殺 TextInputMenuAgent。
- 對 QIM 的意義：M7 要把簽章身分、TCC、root 註冊輸入源、更新前切走輸入源、開發迴圈（dev-reload）列入任務。

### 2.3 iOS

- 記憶體：`platforms/ios/Keyboard/KeyboardMetrics.swift:33` 用 `phys_footprint` 取樣；release-1.1.0 的 `peakSampledPhysicalFootprintBytes` 為 23741360（約 23.7 MB，22.6 MiB），`sampleCount` 為 1，含整套 librime（`platforms/ios/validation/release-1.1.0/ios-keyboard-metrics.json:36-37`）。單一樣本，沒有 jetsam 紀錄，沒有上限數字。
- 詞庫在 Mac 建置機預編譯，不在鍵盤首次出現時部署（`platforms/ios/README.md` Build 段）。
- Full Access：`Keyboard/Info.plist:39-40` 設 `RequestsOpenAccess=true`，但基本打字、Buffer、已存剪貼簿歷史不需要；只有匯出圖片、讀剪貼簿（UIPasteControl）、AI 網路需要，由 `hasFullAccess` 閘控（`KeyboardViewController.swift:66,120,319,743`）。
- App Group：只有主 App 寫設定，鍵盤唯讀，讀不到就退回離線預設。使用者詞庫在鍵盤擴充自己的 Application Support，不共享、不同步、排除備份（`Support/MobileEngine.swift:28-33`）。
- Keychain group 在 `platforms/ios/project.yml:29-30` 寫死為 `585J2TL9U6.org.scholay.rimes.ios.shared`，註解理由：未簽章 CI archive 的 `AppIdentifierPrefix` 為空。`Support/Settings.swift:83` 用 `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`，只存 AI 金鑰。
- 冷啟動高度：宿主先給暫時高度會畫出空面板；暫時高度到達時請求高度多加 1 pt、0.25 秒後還原，並有界重送（`KeyboardViewController.swift:640-660`，`gapRecoveryDelays` :255-258；測試 `Tests/KeyboardSizingTests.swift:176,268`）。
- 引擎生命週期：每 controller 一個 session、每行程初始化一次，`recursive_mutex` + `engineGeneration`（`Bridge/RimeMobile.mm:39-62`）。
- 對 QIM 的意義：M8 的 Full Access 閘控、單寫者 App Group、詞庫私有、Keychain group 寫死與 `phys_footprint` 取樣都能直接當檢查項；記憶體數字只能當量級參考。

### 2.4 Android 與 Linux

- 宿主發布 `TYPE_NULL`（聊天類編輯器）仍保持中文輸入；數字欄、密碼欄走字面輸入，密碼欄不學習、不呼叫服務（`platforms/android/README.md:36-39,196,257`；`CHANGELOG.md:146`）。`testhost` App 驗證原生、私密、密碼、WebView 欄位（README :310），「OEM 安全鍵盤取代」與「略過的密碼案例」分開記（:342）。
- 引擎工作進序列佇列，非同步結果帶 editor lease，提交前重驗。對 QIM 的意義：M9 要有測試宿主 App，且 `TYPE_NULL` 不能當成「不要中文輸入」。
- Linux：完整 addon 只做 Fcitx5，IBus 引擎尚未開始（`platforms/linux/README.md:16`）；使用者目錄 `$XDG_DATA_HOME/rimes`，背景部署期間按鍵放行，全程序鎖防兩實例寫同目錄（`platforms/linux/ime/README.md:46-65,138-146`）。沒有寫為何選 Fcitx5。

## 3. 核心與資料

- 方案匯入（`docs/portable-scheme-import.md:10-27,45-48`）：ZIP 保留 Rime 原始目錄，可選 `rimes-package.yaml`（`format: 1`），清單只給標題、來源、授權、能力需求，不能授權執行腳本或覆蓋宿主設定。流程：唯讀預覽、隔離暫存、真引擎編譯載入、原子發布、啟用獨立；失敗不動現有方案與學習詞庫。拒路徑越界、符號連結、Windows 保留名、NFC 或大小寫重名；含 `.gram` 模型的包直接拒絕而非靜默降級。上限定義在 `Shared/SchemeImport/scheme_import.cpp:25-28`（ZIP 2 GiB、4096 條目、壓縮比 100 倍等；內建目錄包更嚴，ZIP 64 MiB）。
- 使用者詞庫損壞：LevelDB `.userdb` 的 Compaction error / Corruption 只寫 WARNING 與 UI 提示，不讀正文、不自動刪除或重建；先備份整個資料夾，用文字快照 `.userdb.txt` 遷移，不複製二進位（`docs/user-dictionary-recovery.md`）。沒有看到詞庫格式版本號、升級前自動備份。
- 外掛：官方外掛是 JSON 資料包（`runtime: host-interpreted`），只給宿主已知解譯器有界資料，不能選執行檔、網路端點、憑證（`Shared/Sources/RimesCore/OfficialPluginPackage.swift:3-4`）；先驗目錄 SHA-256 再解析，contribution 型別與選項鍵用白名單。iOS 審核說明「不下載可執行外掛」。與 CyIME 的 Lua 外掛相反，和 QIM 目前不做腳本外掛的方向一致（brief §1.5）。
- 非同步結果：`BufferPluginRunner` 單 worker、新請求取消舊的、遲到結果過 generation、焦點、owner 校驗、請求開始凍結設定快照（`Shared/Sources/RimesCore/BufferPlugin.swift`）；無硬超時熔斷。
- Log 隱私：`scripts/lint-log-privacy.py` 自製掃描器，偵測 `IMELog.write` 插值含敏感識別字，只放行 `.count`、`.isEmpty`、`.utf8.count` 與 `IMELog.redact(...)`（:25），內建 self-test（:157-161）；`IMELog.redact` 只輸出長度如 `⟨3⟩`（`Sources/RimeBuffer/Log.swift:39`）。
- AI：金鑰禁止進 UserDefaults、argv、URL、通知、日誌（`docs/development/plugin-configuration.md` §2）；只送使用者明確提交的文字；HTTPS、先同意接收方、不跟隨重定向。QIM 的雲端 LLM 也是使用者自帶 key（brief §8），這幾條可直接照做，另加「設定匯出不含 key」。
- 對 QIM 的意義：資料包匯入要有惡意樣本回歸；不支援的能力要拒絕而不是降級；非同步結果要帶世代號；log 只准長度；詞庫損壞只提示，用文字快照遷移。

## 4. 簽章與 TCC：Personal Team 憑證（查證 2026-11-03）

問題：免費 Apple ID（Personal Team）在 Xcode 產生的 Apple Development 憑證簽 macOS 輸入法，designated requirement（DR）是否不含 cdhash，使輔助功能 / TCC 授權跨重建保留？免費帳號簽的 macOS App 有沒有 7 天期限？

已查到：

- Apple 文件 TN3127 說明 `codesign` 依簽章身分套用預設 DR，Apple Development 簽出的開發版 DR 形如 `identifier "…" and anchor apple generic and certificate leaf[subject.CN] = "Apple Development: …" and certificate 1[field.1.2.840.113635.100.6.2.1] exists`，不含 cdhash；該節並說預設 DR 的目的是讓新版本保有舊版已取得的權限（麥克風為例）。來源：<https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements>（Default and Xcode designated requirements）。
- Apple Developer Forums 上 Quinn（Apple DTS）的回覆建議日常開發用 Apple Development 這類穩定身分簽章：<https://developer.apple.com/forums/thread/671238>。
- 免費帳號期限：Apple 說明 Personal Team 可註冊 10 個 App ID 與 3 台裝置（皆 7 天過期），「讓 App 安裝到裝置的 provisioning profile 7 天後過期，需重建並重裝」。來源：<https://developer.apple.com/support/compare-memberships/>（Enable a personal team in Xcode）。

沒有查到（列為未查證）：

- Apple 沒有文件把 DR 這一點明確套到 Personal Team 的憑證。TN3127 的例子是一般 Apple Development 身分，Personal Team 的憑證同樣是 Apple Development 類型，推論 DR 形狀相同，但未見官方明說。
- TCC 是否因此跨重建保留：TN3127 只講 DR 設計目的；輔助功能授權的實際比對行為沒有 Apple 文件說明。xsime 在 `build_install.sh:254-262` 自述「授權可跨重建保留」，是維護者的觀察，不是 Apple 說法。
- 7 天限制是否適用 macOS：Apple 該節只談裝置安裝用的 provisioning profile，沒有提到 macOS。一個不使用受限 entitlement 的輸入法本機簽章不需要 provisioning profile（這是一般性推論，未找到明說的官方文字）。Personal Team 的 Apple Development 憑證本身有效期、憑證到期重簽後 `subject.CN` 是否不變，也沒有查到。
- 下一步建議：M7 開工時在實機用 Personal Team 簽一次，用 `codesign -d -r-` 讀回 DR，重建兩次後讀回輔助功能授權是否保留，把結果補進本文。

## 5. 工程流程

- 指令檔：根 `AGENTS.md` 11 行；`.claude/skills/xsime-project/SKILL.md` 只轉指 `.codex/skills/xsime-project/SKILL.md`（約 70 行：平台入口、相容識別碼、權限與使用者資料禁令、除錯欄位）。反例：`DesignSystem/AGENTS.md` 塞約 20 條產品決定，與 `DesignSystem/README.md:35` 起重複且已分歧。根目錄 `.md` 44 個，`CHANGELOG.md` 1118 行；`CONTRIBUTING.md:35` 禁止把排查過程另存成文檔。
- 驗收紀錄：`validation/` 約 60 個「功能-日期」目錄，JSON 記 verified、version、完整 commit、安裝目錄、測試數與第一次失敗；`provenance.json` 有來源檔 SHA-256。每項驗收含平台、完整 commit、建置配置、檢查結果、未驗證項；合併、編譯、CI 通過不能單獨作為關閉依據（`docs/development/maintenance.md:21-22`）。2026-10 起截圖與原始 log 不進 Git，只留結論、聚合 JSON、SHA-256 與位元組數，歷史圖片連結已死。
- CI / 發行：12 個 workflow 約 4850 行，`release.yml` 約 2330 行含內嵌 Python；確切 SHA 五端 CI 綠才打 tag；簽章在受保護 environment，公開批准另一道人工；draft 上傳後逐項讀回再公開。更新清單用 ECDSA P-256 簽精確 UTF-8 位元組，簽章有效期上限 90 天，不覆蓋已發佈資產（`scripts/release/update_signing.py:1-30`）。
- 設計 token：React 母版只管 macOS / iOS，Android / Windows / Linux 沒有；`scripts/check-color-contrast.mjs` 強制 WCAG 4.5:1。
- 對 QIM 的意義：驗收紀錄欄位（平台、SHA、設定、結果、未驗證項）值得採用，並明寫 CI 綠不等於輸入問題已關；授權檔逐項列版本與 SPDX 可加測試。

## 6. 未查證

- 各平台實機行為、CI 是否真的全綠、release 流程是否如文件所述：只讀文件與腳本，沒有跑。
- `.new` / `.bak` 原子替換與 `restore_previous_install` 的細節。
- 資料授權的逐項核對：只看了 `rime-data/licenses/` 的來源說明，沒有逐檔比對詞庫標頭。
- §4 列出的 Personal Team 三項。

## 7. QIM 不照搬

- 每平台各寫一份引擎橋接、再用回歸樣本對齊：QIM 只有一個 Rust 核心，各殼共用同一份實作（ADR 0002）。
- Buffer / Capsule / Mailbox 與 AI 功能：不在需求內。
- 2 GiB 匯入上限：iOS 鍵盤擴充不現實，QIM 的上限由各平台記憶體決定。
- 把產品決定塞進指令檔：與 `CLAUDE.md` 的原則相反。
- 單一巨型 release workflow（約 2330 行內嵌 Python）。
- 截圖不入庫且驗收目錄無索引：QIM 的驗收紀錄要有索引。
- 禁止本機建置、全走遠端 CI：與 QIM 的實機驗收要求衝突。

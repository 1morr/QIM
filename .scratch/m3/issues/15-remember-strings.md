# 15: 自動記住 email 這類字串

Status: done
Blocked by: 05, 12
讀: plan §2.1（`text`、英文直通、臨時英文、上下文）、§2.4；brief §1.5、§4.7、§8；`docs/research/blackbox-baseline.md`（T5）；spec〈範圍〉〈共同規則〉

## What to build

使用者打過一次的 email 地址（與票內決定的其他字串，例如網址），之後打開頭幾個字母就出現在候選裡，選了整串上屏。

- 先查成熟做法（Gboard、iOS 的 email 建議、SwiftKey）再定：從哪裡記（臨時英文上屏的、`text` 動作的、前文裡出現的）、打什麼會觸發（字母開頭的前綴、要幾個字母）、候選排在哪、會不會擋到英文候選與中文首選。決定寫回 plan §2.1、§2.4，註明採用誰的慣例。
- 英文直通時鍵交還宿主、核心看不到，這點要在設計裡交代。
- 無痕與學習關掉時不記；存在使用者資料裡、可刪；要動匯出格式時照 spec〈共同規則〉加進 v2。

## 驗收

- [x] 單元測試：記過 `ivan.test@example.com` 後，打票內定的觸發前綴時它在候選裡、選了上屏整串；無痕時不記。
- [x] Playwright：打一次 email、再打前綴、選它，讀回輸入框；截圖。
- [x] 黑箱 T5 在 QIM 網頁重跑，補進 `docs/research/blackbox-baseline.md`。
- [x] `bench --check` 不變差；實際操作亂按一輪；`cargo xtask check` 綠燈。

## Comments

- 2026-11-11 實作：記住的字串（名詞表）。`Session` 記送出去的字結尾連續的 email 字元（`remembered::TypedRun`），遇到其他字元、交還宿主的 Enter／Tab 這類鍵、離開欄位就算打完，是 email 就記進 `UserModel`（各方案共用、衰減值、用個人 bigram 的 tick）；中文組字的輸入碼長過一個字的最長碼、是它打得出的字母的開頭時，排位規則 5 把它排在第 2 位起、英文候選之後，最多 3 個。開關 `Options.remembered_strings`（預設開）；`clear_context` 改收 `&mut UserModel`。匯出 v2 加「方案欄、前一個字詞欄都空」的列。寫進 plan §2.1、§2.4、§3.1、§0，brief §7、§12.4，名詞表。
- 成熟做法（`docs/research/remember-strings.md`）：LatinIME 上屏一次就學、不讀欄位原有的字、密碼欄不學；SwiftKey 學 email、Incognito 不學；沒有一家公開觸發字數與位置。採用：只記經 QIM 打的字（上屏的、英文直通交還宿主的字元、`text` 的 `@` `.`），不讀前文、通訊錄、剪貼簿；打 1 次就記；只記 email、不記網址（LatinIME 也不學網址）。
- 英文直通：鍵交還宿主前殼仍先交給 `handle`，所以核心看得到英文直通打的字（同前文）；看不到的是別的鍵盤打的、貼上、autofill 與輸入框原有的字，不記。英文直通沒有組字、沒有候選列，所以只在中文組字給（英文模式的預測在 M4）。
- 觸發字數：長過一個字的最長碼（速成 3 個字母，同英文候選）。`scripts/experiments/remembered-prefix/` 用基準題目量常見 email 開頭的誤觸：2 個字母約 0.19%、3 個約 0.014%、4 個約 0.001% 的組字。比對跳過數字、符號與方案沒有的字母（亂按時發現 `zeta.…` 打不出，先寫紅的單元與 Playwright 測試再修）。
- `bench --check`（開工 `c623f86`）：和 baseline 完全相同，fixed 84／84；yue-wiki 答錯 1123、修正 4295；zh-wiki 1126、4381；common-voice-yue 1030、3908；hkcancor 650、2478；有前文四集同 baseline；依長度分組不變，12 個集合都沒有翻盤。`--learning` 學習開：yue-wiki 4220、zh-wiki 4368、common-voice-yue 3763、hkcancor 1980、前文四集 2204／2152／590／1396，和票 12 合併後的數字逐一相同（基準只打中文，不會記到 email）。本機沒有個人集。
- 黑箱 T5 在網頁重跑，結果與 T5a–T5f 各項寫進 `docs/research/blackbox-baseline.md`。亂按一輪：固定種子 400 步（夾雜英文直通打 email、組字中各鍵、快速連按、切記住 email／無痕／學習／英文候選／簡體、點設定區刪除），沒有頁面錯誤；找到的 `z` 開頭問題見上。腳本沒進 repo。截圖 `.scratch/m3/evidence/15-*.png`。
- code-review 已處理：開頭不是使用者打的那串不記（接在輸入框原有的 email 字元後、退格刪到它前面、方向鍵之後、有一段在無痕或學習關掉時打的、長過上限後退格）；同一個 email 不分大小寫只留一條；`Decayed::bump` 收掉重複；改名 `remembered`、`is_email_char`、`forget_remembered_string`；網頁兩處存檔條件合成 `afterHostKey`；plan 補「不設上限、不過期」的理由與重排送雲端的去處（票 22 的 Comments 已留）。
- code-review 未處理（判斷題）：記住一次讓個人 bigram 共用的 tick 加一（全部個人 bigram 衰減一步，影響可忽略，換來匯出不多一行 tick，寫在 plan §2.4）；網頁在英文直通每個空白、標點都整份存一次學習資料（prototype 先這樣，平台殼存檔方式在 M6 定）；`Listing` 三個「只給整串」的開關照現有寫法並列；網頁前文上限 200 字，201–254 字元的 email 在網頁上記不住（實際不會遇到）。
- 2026-11-11 協調者回饋：誤觸率每條各算一次、原本不設上限，改成最多記 20 條（`REMEMBERED_STRINGS_MAX`），超過時淘汰最久沒用（最後 tick 最小）的，匯入超過時整份不收；不做時間過期。20 的理由（20 × 0.014% ≈ 0.3%，和一條 2 個字母同級）寫在 plan §2.4 規則 5。單元測試守淘汰與匯入。
- 2026-11-11 合併 main（票 21）：沒有衝突；progress 的 session 紀錄把票 15 移到票 21 之後。重建資料包後 `cargo xtask check` 全綠（cargo test 869 passed、e2e 200 passed），`bench --check` 與 `--learning` 的數字和上面相同、12 個集合沒有翻盤。

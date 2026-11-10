# 16: emoji 候選

Status: done
Blocked by: 02
讀: plan §2.2、§2.4、§4、§4.5；brief §1.5、§5.1、§7；spec〈範圍〉〈共同規則〉

## What to build

打出「笑」「貓」「多謝」這類詞時，候選裡出現對應的 emoji。

- 資料：CLDR 的繁中（`zh_Hant`）與粵語（`yue`）標註，釘進 `data/sources.toml`（版本、sha256、Unicode License），建成一份 emoji 資料包，加進 `--verify` 的決定性檢查；授權查證補進 brief §12.1。
- 排法先看成熟做法（Rime 的 emoji 濾鏡把 emoji 接在對應候選後面、iOS 速成）再定，寫回 plan §2.4；會改候選位置，所以 `Options` 加開關（預設值在票內定、寫回 brief §7），網頁設定區加開關。
- 簡體模式下仍照繁體原文對標註。

## 驗收

- [x] 單元測試：打「笑」的碼時候選裡有 😄 這類 emoji、位置照票內的規則；開關關掉時沒有。
- [x] `cargo xtask data --verify` 綠燈，emoji 資料包有 `LICENSE`、`NOTICE`。
- [x] Playwright：打一個詞、選 emoji 上屏讀回；截圖。
- [x] `bench --check` 不變差（emoji 不是正解時不該影響首選）；`cargo xtask check` 綠燈。

## Comments

- 2026-11-11 排法（寫回 plan §2.4）：照 rime-emoji。librime 的 `simplifier` 把 OpenCC 轉出的另一種寫法接在原候選後面，原詞留在最前（brief §12.4）；iOS／macOS 中文鍵盤找不到官方說明。做法是排位規則之前，蓋滿整串的字詞後面各接它的 emoji，整列最多 3 個（`EMOJI_LIMIT`）。只蓋住開頭一段的字詞、目前分段的候選列不接；選了和英文候選一樣，等同 Enter 上屏、不學。在規則之前接上，所以英文候選仍在第 2 位。Rime 每個候選都接、不設上限，這裡設上限，第一頁大半仍是字詞。
- 開關：`Options.emoji_candidates`，預設開（同 rime-emoji 的 `reset: 1`），寫回 brief §7；網頁設定區加「emoji 候選」。
- 資料（plan §4.4、brief §5.2、§12.1）：
  - CLDR `release-49`（`96e54b2`）的 `yue.xml`、`zh_Hant.xml`，授權照抄釘住的頂層 `LICENSE`。
  - Unicode 18.0 的 `emoji-test.txt`，用來補回 CLDR 拿掉的 U+FE0F，授權用 repo 的 `Unicode-3.0.txt`。
  - 資料包 `cldr-emoji-yue-hant`，141 KB 左右，魔數 `QIMJ`；`FORMAT_VERSION` 不變，其餘六份 sha256 不變。
  - 每個字詞的 emoji 依序是短名是它的、再來關鍵字有它的；兩種之內粵語在前。
  - 字詞經 `t2hk` 再換回常見寫法；有字沒有速成碼的不收。
- 關鍵字門檻（協調者轉來的回報：CLDR 關鍵字太泛）：
  - 改前打「打開」第 2–4 位是 😃😄😆，「眼」是 👁️😄😁，「臉」是 😀😃😄，「嘴」是 👄😃😄。
  - 改成資料規則、不手寫排除清單：短名一律收；關鍵字只在兩份合計對到不超過 6 個 emoji 時收（`MAX_KEYWORD_EMOJI`）。
  - 改後上面四個詞的笑臉都不見了，「笑」😆🤣😂、「貓」🐈、「多謝」🙏🫂 仍在。
  - 結論在 `docs/research/emoji-candidates.md`，量法與各門檻的表在 `scripts/experiments/emoji-keywords/README.md`。「笑」剛好對到 5 個，取 6 多一點餘裕。
- `cargo xtask data --verify`：`verify 通過`。新抽查「hk 的『笑』後面接著它的 emoji」的第一頁是「奧 微 笑 😆 🤣 😂 徵 啟 牧」。emoji 資料包 sha256 `7bc0a3ff…508f`。統計：CLDR 標註 7900 個，太泛的關鍵字 147 個、方框漢字 17 個、有字打不出的 254 個不收，收 6588 個字詞、1943 個 emoji。
- `cargo xtask bench --check`：bench 也掛上 emoji 資料包，和網頁一樣。前後對照（前是 `3543c83` 的 baseline）：

  | 集合 | 答錯 前 → 後 | 修正 前 → 後 |
  | --- | --- | --- |
  | yue-wiki | 1131 → 1131 | 4467 → 4480 |
  | zh-wiki | 1126 → 1126 | 4505 → 4514 |
  | common-voice-yue | 1032 → 1032 | 4043 → 4054 |
  | hkcancor | 663 → 663 | 2588 → 2595 |

  - 數字是最後的資料包（門檻 6、不收方框漢字）。依長度分組：各組首選正確率前後相同。
  - 首選沒變：答錯數相同，各集合沒有翻盤，依長度分組的首選正確率也相同；固定題 84/84。
  - 修正次數多約 0.3%：emoji 佔第一頁的位置，把第 7–9 位的正解擠到第二頁，多翻一次頁。每列最多 1 個時仍多 1–7 次，只要佔第一頁就有這個代價。
  - 使用者選了「最多 3 個、更新 baseline」，沒選限 1 個或預設關。驗收的「不變差」改成「首選不變」，記在 progress〈偏差與決定〉。
  - 這個 worktree 沒有個人集（`bench/personal/` 不在），沒有跑到。
  - 沒改學習，沒跑 `--learning`：emoji 選了不學，有單元測試守著。
- 測試：
  - 核心 `crates/qim-core/tests/emoji.rs` 16 個（含規則 1、2 移動候選時 emoji 跟著自己的字詞），其中 3 個做過變異（emoji 照字詞學、不看開關、開頭一段也接），各自會紅。
  - `qim-databuild` 的 `tests/emoji.rs` 8 個；`tests/run.rs` 補了 CLDR 與 emoji-test 的假來源，加 emoji 資料包 LICENSE、NOTICE 的測試；`tests/verify.rs` 加抽查失敗與決定性的測試。
  - Playwright `web/e2e/emoji.spec.ts` 4 個。截圖在 `.scratch/m3/evidence/16-*.png`。
- `web/e2e/edit-segments.spec.ts` 原本斷言「分段的候選＝單打那個碼的候選」。現在單打 `kb` 時「有」後面接 🈶，分段沒有，所以改成去掉 emoji 後比前綴；plan §2.1 同步寫明分段沒有英文候選與 emoji。
- 亂按一輪的操作：選開頭一段後再選 emoji、組字中開關 emoji 與簡體、`=` `-` 翻頁、← → 進出分段、Backspace、Esc、左 Shift 臨時英文、Enter。都照預期，沒有找到問題。
- 已知限制：CLDR 關鍵字覆蓋不均（「哈哈」「開心」粵語沒有）。要補口語叫法時，另加 repo 自行整理的 CC0 詞表，等使用者回報（plan §4.4）。`qim-ffi` 沒接 emoji 資料包，平台殼 M6 起再接。
- 和票 01 的關係：票 01（網頁拆模組，`e722fe0`）在別的分支，這張票改了 `web/src/main.ts`，合併時可能衝突，要把 emoji 開關搬到拆出的設定區。
- 方框漢字（code-review 的 Spec 軸發現）：🈶、🈚、🈷 這些 emoji 的標註只是框裡的字（「有」「無」「月」各只對到 1 個），門檻擋不住，打這些常用字第 2 位就是它。照協調者「用資料規則」的方向，按 Unicode 區段整個不收（U+1F200–1F2FF 與 ㊗ ㊙，`is_squared_ideograph`），共 17 個；寫進 plan §4.4、brief §12.1、`docs/research/emoji-candidates.md`。
- code-review（`3543c83` 起，Standards 與 Spec 兩軸）的處理：
  - 已改：
    - 結論補進 `docs/research/emoji-candidates.md`，brief 與 plan 改連它；實驗 README 補系統版本。
    - plan §0 識別符表補 emoji 候選；`Engine::candidates` 的註解補「不含 emoji」。
    - `verify.rs` 的 `english_the` 改用 `first_page`；CLDR 檔清單搬到 `emoji` 模組，實驗不再自己列一份；測試的 `all_six` 改名 `all_packs`。
    - plan §2.4 的「和前面候選重複」改成「和候選列裡其他候選重複」。
    - 規則移動候選時 emoji 的位置寫進 plan §2.4 並補兩個測試。emoji 刻意在規則之前接上，讓規則保證的位置（英文第 2、使用者詞至少第 2）不被 emoji 擠掉；代價是被釘住的候選可能夾在字詞和它的 emoji 之間。
  - 沒改、留給之後：
    - 重排要求的前 9 個候選含 emoji，`apply_rerank` 也會移動它。雲端重排票 22 定要不要排除，已寫進 plan §2.4。
    - `lattice::candidates` 收兩個 bool（英文、emoji）。呼叫點傳 `false, false`，維持和英文候選相同的形狀；第三個這類開關進來時收成一個 struct。
    - `web/src/main.ts` 的 `applyEmojiCandidates` 和 `applyEnglishCandidates` 同形，留給票 01 拆設定區時一起收。
    - `pipeline.rs` 的 `build_packs` 裡 emoji 那段約 25 行，其他資料包也是在那裡讀來源、組參數，沒有另抽 `emoji::build`。
    - 簡體模式轉換後重複而被丟掉的候選，它的 emoji 會留下來，接在同字的那個候選後面。顯示的字相同，不算錯。
- 2026-11-11 合併 main 之後（`ad8e8c8`，含票 01、04、14）：
  - emoji 開關搬進 `web/src/settings.ts` 的 `ROWS`，`settings.spec.ts` 的預設值測試補上它。
  - 個人集經軟連結接上（`bench/personal` → 主 checkout）。
  - `cargo xtask data --verify` 通過：語言模型 sha256 因票 04 的句首變成 `dfa21786…387d`，emoji 資料包 `7bc0a3ff…508f` 不變。
  - `cargo xtask bench --check` 和合併後的 main baseline 比：答錯數全同，13 個集合都沒有翻盤，固定題 84/84。修正次數變多：

    | 集合 | 修正 前 → 後 |
    | --- | --- |
    | yue-wiki | 4340 → 4346 |
    | zh-wiki | 4380 → 4385 |
    | common-voice-yue | 3946 → 3964 |
    | hkcancor | 2573 → 2596 |
    | yue-wiki·前文／不給 | 2288 → 2292／2300 → 2304 |
    | zh-wiki·前文／不給 | 2166 → 2169／2188 → 2191 |
    | common-voice-yue·前文／不給 | 643 → 645／655 → 657 |
    | hkcancor·前文／不給 | 1779 → 1796／1801 → 1817 |
    | 個人集 | 1446 → 1466（答錯 1081 不變） |

  - 原因和合併前相同：emoji 佔第一頁的位置，把第 7–9 位的正解擠到第二頁。合併後增幅較大：hkcancor 約 +0.9%，個人集約 +1.4%；合併前公開集約 +0.3%。
  - 照協調者指示 `--update-baseline`，公開集寫進 `bench/baseline.json`；個人集的 baseline 在主 checkout，各 worktree 共用。

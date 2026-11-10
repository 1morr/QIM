# emoji 候選：成熟做法與 CLDR 標註

M3 票 16，2026-11-11。回答兩個問題：emoji 排在候選列哪裡；CLDR 的粵語與繁中標註能不能直接當「字詞 → emoji」的表。結論摘進 brief §12.1、§12.4，規則寫在 plan §2.4、§4.4。

## 1. 成熟做法

- **Rime（rime-emoji）**：用 librime 的 `simplifier` 濾鏡（`simplifier@emoji`，`option_name: emoji_suggestion`；開關 `reset: 1`，預設開；`tips: none`；`inherit_comment: false`），放在 `engine/filters` 的第一個。OpenCC 詞典 `emoji.json` 用 mmseg 分詞，鏈接 `emoji_word.txt` 與 `emoji_category.txt`，值是「原詞 emoji…」（`笑	笑 😄️ 😸️`、`貓	貓 🐈️`）。`Simplifier::Convert` 依序處理轉出的每個值：等於原文的放回原候選，其餘做成 `ShadowCandidate`。結果是原詞留在最前，emoji 緊接在它後面，就地展開、不是集中放在選單尾端。每個經過濾鏡的候選都轉，不設上限；`excluded_types` 列的候選種類不轉。詞典是人工整理的。來源：librime `c95c0d2` [`src/rime/gear/simplifier.cc`](https://github.com/rime/librime/blob/c95c0d2390c827bf44e5eb8989704ba723ec0d0c/src/rime/gear/simplifier.cc)（`Convert` L125–156、`Replenish` L75–82）；[rime-emoji `39d2329`](https://github.com/rime/rime-emoji/tree/39d232996de79a95d7bf4b5e8533feed5326b075)。
- **iOS／macOS 中文鍵盤**：沒有找到官方文件說明速成、倉頡、拼音的候選列是否出 emoji、出在哪裡。macOS 速成的聯想含 emoji（黑箱 T8：日 → 本、期、🌞…），見 `blackbox-baseline.md`。
- **Gboard**：只找到新聞報導（9to5Google、Android Police，2020-10），說英文的建議列最右一欄放 emoji，設定在 Text correction › Show emoji suggestions；中文輸入法的位置沒有找到。

QIM 採用 Rime 的排法：emoji 接在對應的字詞後面，有開關，預設開。和 Rime 不同的是每列最多 3 個，第一頁（9 個）大半仍是字詞。

## 2. CLDR 標註

- 版本：`release-49` 是 2026-11-11 最新的 stable。它是 lightweight tag，指向 commit `96e54b2`；`-beta*`、`-m*` 是預發布。
- `common/annotations/yue.xml`、`zh_Hant.xml` 各 3,950 個 `<annotation>`。每個 emoji 有一行關鍵字（` | ` 分隔）和一行短名（`type="tts"`）。`yue.xml` 是完整的一份，沒有 `↑↑↑` 繼承記號。
- 授權：檔頭寫 `SPDX-License-Identifier: Unicode-3.0`。repo 頂層 `LICENSE` 的第一段是 UTS #35 規格本身的版權，限制轉載；其餘內容（含資料檔）是 Unicode License V3。
- `cp` 去掉了 U+FE0F（`zh_Hant.xml` 檔頭的 Warnings），要照 Unicode 的 `emoji-test.txt` 換回 fully-qualified 的寫法。膚色與組合序列在 `annotationsDerived/`，QIM 不用，免得候選塞滿同一個 emoji 的變體。
- 關鍵字是給 emoji 選單搜尋用的，收得很寬又不均：
  - 粵語標註裡 😄 的關鍵字有「嘴」「眼」「打開」「臉」。全收時，打這些常用詞會在後面接上不相關的笑臉。
  - 「笑」在繁中標註對到 🤣🥰😼，粵語標註對到 😆🤣😂。
  - 「哈哈」「開心」在粵語標註一個也沒對到。
  - 方框漢字（🈶🈚🈷🉑🉐🈳🈴🈯🈲🈵 等，Unicode 的 Enclosed Ideographic Supplement 與 ㊗ ㊙）的標註只是框裡的字，「有」「無」「月」「可」「得」都對到不超過門檻的 1 個，全收時打這些常用字第 2 位就是它。

本機統計（`cargo xtask data --verify` 與 `scripts/experiments/emoji-keywords/`）。

## 3. 太泛的關鍵字怎麼濾

規則用資料定，不手寫排除清單：短名一律收；關鍵字只在兩份標註合計對到不超過 6 個 emoji 時收。「臉」「動物」這類分類詞對到幾十個 emoji，自然被濾掉。方框漢字由 Unicode 區段整個排除，門檻擋不住它們（每個字只對到 1 個）。

門檻由實驗定（`scripts/experiments/emoji-keywords/README.md`，Apple M6、macOS 27.0.1）。首選字基準的答錯數在各門檻都和沒有 emoji 時相同，修正次數在門檻 4 到不限之間幾乎不變（yue-wiki 4479–4480），所以門檻依抽查定：

- 門檻 6 拿掉「打開」「眼」「臉」「嘴」後面的笑臉。
- 「笑」😆🤣😂、「貓」🐈、「多謝」🙏🫂 仍有 emoji。
- 「笑」的關鍵字剛好對到 5 個 emoji，門檻 4 以下就沒有；取 6 多一點餘裕。

修正次數比沒有 emoji 時多約 0.3%（各集合 7–13 次）。原因是 emoji 佔第一頁的位置，把第 7–9 位的正解擠到第二頁，和門檻無關；每列最多 1 個時仍多 1–7 次。使用者選擇每列最多 3 個、更新 baseline。

## 4. 之後

CLDR 的覆蓋不夠時（口語叫法，像「哈哈」「開心」），另加一份 repo 自行整理的 CC0 詞表，做法同粵語口語詞表；等使用者回報再做。

# 完整粵語詞庫：授權乾淨的來源有哪些

M3 票 18，2026-11-11。回答：有沒有授權乾淨、能隨商店 App（App Store、Play；免費，日後可能收費）發佈的完整粵語詞庫（目標幾千到幾萬個粵語口語詞，最好有詞頻或能排序）。QIM 只要詞的漢字寫法，不要讀音與釋義；從某個詞典只取詞頭是否仍受該詞典授權約束，照授權原文寫出條款的直接含義，不下法律結論，「是否合規」寫成「需使用者判斷」。已查清的部分不重複，見 [`data-sources.md`](data-sources.md) §6、§11、§14 與 `docs/design-brief.md` §12.1。

本文數字除註明外都是本機統計（下載檔放在 session 的 scratchpad，不進 repo）。「詞」指兩字以上、全由漢字組成。比對用的 rime-essay 是 `054920d`（QIM 現用的 pin），**比對時沒有做 OpenCC 異體正規化**（essay 用「羣」，粵典混用「群」「羣*」），所以「不在 essay 內」的數字偏高，只看數量級。

## 1. rime-cantonese 與 rime-cantonese-upstream

**版本**：rime/rime-cantonese `259f0e4`（2026-08-13）、CanCLID/rime-cantonese-upstream `4cb67a4`（2026-08-10），都可用 commit 釘住。

**授權原文**：兩個 repo 的 GitHub license 欄都是 CC-BY-4.0，`LICENSE`（upstream）或 `LICENSE-CC-BY`（下游）是 CC BY 4.0 全文；下游 `README-en.md` 另寫 `jyut6ping3.maps` 為 ODbL 1.0。CC BY 4.0 要求署名、附授權連結、標示是否修改，沒有 share-alike，商用與再散布都允許。

**授權鏈：repo 內沒有任何說明。**
- 下游 wiki（`rime-cantonese.wiki.git` `7724c75`，2025-11-24）的〈資料來源〉頁只列來源名稱與連結（LSHK 字音表、粵音小鏡、廣州話正音字典、粵語審音配詞字庫、電腦用漢字粵語拼音表、HKSCS；詞組：香港語言學學會粵拼詞表、粵典、open Cantonese dictionary、CC-Canto、A Dictionary of Cantonese Slang），沒有任何授權、許可或「已取得同意」的句子；全 wiki 唯一的授權字樣在安裝包配件的版權資訊（GPL v3，指程式）。upstream 沒有 wiki（`has_wiki: false`），`README.md` 只列來源與 Credits。
- 詞庫來源的取得方式在 issue 與 commit 裡留有痕跡，沒有授權討論：
  - 紙本詞典（《實用廣州話分類詞典》《廣州話詞典》《地道廣州話用語》《A Dictionary of Cantonese Slang》）：[issue #50](https://github.com/rime/rime-cantonese/issues/50) 發起人上載簡體電子檔、請志願者轉成 OpenCC 繁體並拆成單字／詞／外文三類；討論只談格式與校對，沒有提版權。
  - 香港語言學學會粵拼詞表與粵語審音配詞字庫：wiki 的資料來源連結指向 `sgalal/lshk-word-list-crawler`、`sgalal/lexi_can_crawler`，即爬取網站所得；LSHK 詞表的原網站 `corpus.eduhk.hk/JPwordlist/` 在 2026-04-01 起回 404（Wayback CDX），詞表前身頁（2019-01-06 快照）顯示 15,740 筆、來源欄為 EDB（教育局）與 LSHK，沒有授權字樣。
  - 搜狗、QQ、百度用戶詞庫：只見於 `jyut6ping3.phrase.dict.yaml` 檔頭一句「收錄搜狗、QQ、百度輸入法用戶貢獻詞庫中嘅所有粵語詞條」。該檔 2023 年起被註解停用（[issue #161](https://github.com/rime/rime-cantonese/issues/161) 的 PR 描述：「commented out jyut6ping3.phrase.dict.yaml to reduce dict size」），貢獻者 hfhchan 在同一 issue 說「phrase 大清洗中，之後 port 返上 upstream」。HEAD 的 `jyut6ping3.dict.yaml` 是否 import 它、upstream 的 `phrase_fragment.csv`（7,276 行）是否含這批詞，沒有逐條標記，查不出。
  - 維護者有來源意識：[PR #212](https://github.com/rime/rime-cantonese/pull/212) 因原專案是 GPL-3.0、與 CC BY 4.0 不相容而撤回廣東地名詞庫；[#91](https://github.com/rime/rime-cantonese/pull/91) 討論加入英文維基詞典發音資料（CC BY-SA）時，維護者的結論是要分成獨立檔案、獨立授權（`LICENSE-CC-BY` 與 ODbL 分檔即由此而來）；commit `e2e088e`（2024-01-02）「刪除不明來歷配詞」。但沒有找到對紙本詞典、用戶詞庫、粵典的同類討論。
- 在兩個 repo 以關鍵字搜 issue（license、授權、版權、商用、commercial、粵典、搜狗、copyright）並讀了相關的 #15、#35、#50、#91、#97、#161、#212，**沒有人問過粵典非商業授權或商用相容**。[#97](https://github.com/rime/rime-cantonese/issues/97) 有人問「sources 也是 CC-BY 嗎」，維護者答「repo 的程式碼只是清洗與打包腳本」，沒有答覆資料來源的授權。這是關鍵字搜尋的結果，不保證涵蓋全部討論（Telegram 群組不公開）。

**粵典這一條線有可查的出口**：upstream README 的粵典連結不是 `words.hk` 首頁，而是 `https://words.hk/faiman/analysis/wordslist/`，即 §2 的 public domain 詞表。

**`essay-cantonese.txt` 的頻次**：
- 來源是 [`essay-cantonese` 分支](https://github.com/rime/rime-cantonese/tree/essay-cantonese)（`6adb742`）的 `essay/build.py`（`d20f78a`，2020-07-10）：rime-essay 的 `essay.txt` 與 `cifu-freq.txt` 合併，後者的頻次乘上 `4822928 / 12448` 對齊「的」的頻次，同詞以 Cifu 為準。
- 該分支 README 與 `d20f78a` 的 README 列出兩個上游：`gwinterstein/Cifu` 標 GPL 3.0、`rime/rime-essay` 標 LGPL 3.0。Cifu 的詳情見 §5.1。`cifu-freq.txt` 先以 commit `506a4a9`「粵文庫詞頻資料（取自粵典）」入庫，內容是否等同 Cifu 的頻次未逐字核對（Cifu 的 `Lexicon/Cifu-v1.txt` 有 51,798 詞，`cifu-freq.txt` 為 51,798 行，行數一致）。
- 因此 `essay-cantonese.txt` 的頻次含 GPL-3.0 來源的資料，repo 整體標 CC BY 4.0 與此並存；HEAD 的檔案後來又被更新過（`703c56b`，2026-05-27，在非預設分支；[PR #225](https://github.com/rime/rime-cantonese/pull/225) 仍開著），來源說明沒有跟著更新。[issue #35](https://github.com/rime/rime-cantonese/issues/35) 裡維護者曾請人順便看維基百科發放條款，沒有下文。

**`word.csv` 沒有逐條來源欄**：欄位只有 `char,jyutping`；`phrase_fragment.csv`、`fixed_expressions.csv` 同。`proper_nouns.csv` 有 `category` 欄但那是詞類，不是來源。所以「只取來源乾淨的子集」做不到。唯一的推測方法是跟已知乾淨集合交集（交集只表示「也在那裡」，不證明該條的來源）：

| 檔 | 兩字以上漢字詞 | 在 §2 粵典詞表內 | 不在 essay 內 | 不在 essay 也不在粵典詞表（來源不明部分） |
|---|---|---|---|---|
| `word.csv` | 92,531 | 46,739（50.5%） | 30,135 | 15,420 |
| `phrase_fragment.csv` | 6,939 | 1,545 | 6,279 | 4,919 |
| `fixed_expressions.csv` | 588 | 338 | 236 | 69 |

沒有頻次（`word.csv` 無詞頻欄），頻次只在 `essay-cantonese.txt`（上述）。

**含義**：repo 的 CC BY 4.0 標示是整理者對自己貢獻的宣告；詞條來自第三方資料的部分，repo 內沒有證據顯示已取得各來源的再授權，頻次還混有 GPL-3.0 的資料。QIM 若使用這份資料，等於承接了這個沒有查明的授權鏈。是否可接受：**需使用者判斷**，判斷點是「上游整理者標 CC BY 4.0」這件事本身是否足以作為依據，還是要追到每個來源。

## 2. 粵典 words.hk

**詞典整體的授權（原文，`https://words.hk/base/hoifong/`，2026-11-11 讀）**：《非商業開放資料授權協議 1.0》（Non-Commercial Open Data License 1.0），版權持有人 Hong Kong Lexicography Limited（頁內「Copyright (c) 2015-2025」）。粵語版僅供參考，以英文版為準。英文版要點：
- §1 授權複製、修改、翻譯、散布、再散布等，條件：(a) 不得作 Commercial Purposes（§2），除非符合 §3；(b)(i) 散布要附版權告示、免責聲明與 Credits List（〈關於我哋〉的「特別鳴謝」）；(ii) 若從授權人網站取得，要顯著連回該網站；(iii) 若散布的部分公開放在授權人單一 URL 上，可改附「Licensed under the Non-Commercial Open Data License ( URL )」，URL 須是實際來源，不得替換；(c) 不得以作者或授權人名義背書；(d) 不得再授權（sub-license）。
- §2 Commercial Use 包含：網站廣告費、把作品當增值服務出售、收費出版、收費課程、收費演出、「任何在商業環境下受版權法保護的使用……無論是否關乎核心業務」。原文：「it is immaterial whether You make a profit or loss, it is immaterial whether the purpose is for charity, educational, public service, or "non-profit" purposes」。
- §3 豁免：(a) 與授權人另簽協議；(b) 善意聯絡授權人六個月無回覆（授權人曾回覆過任何意見、或已拒絕過，就不能用；要留紀錄）；(c) 發佈滿十年的版本；(d) 美國 fair use；(e) 獨資且年收入低於所在地個人收入中位數三倍。
- 商業授權的申請途徑：頁面沒有專門窗口，只有每頁頁尾的「電郵：關於粵典」連結；〈關於我哋〉頁有「電郵」「Facebook」聯絡連結（連結目標未展開）。
- 論文 Lau et al. 2022（LREC DCLRL，[ACL Anthology](https://aclanthology.org/2022.dclrl-1.7.pdf)）附錄 Data Card：該次提交的快照以 CC BY-NC 4.0 釋出，論文稱該次提交的快照另給 CC BY-NC 是為了方便共享，自家的 tailor-made license（即上述協議）才是常態；LREC 簡報 §9 稱「The core dataset has a proprietary licence」。論文列的 GitHub `wordshk/data2021` 在 2026-11-11 回 404。完整 CSV 要向網站申請（AlienKevin/wordshk-tools README：「You can request access to the CSV」），沒有公開全量下載。

**例外：`細數據`（`https://words.hk/faiman/analysis/`）列出的四份資料，每份都標「Data License: public domain. Credits to words.hk appreciated.」「授權：公有領域。」**（頁面 2026-11-11 讀）：

| 資料 | URL | 內容 | 頁面標示的更新日 |
|---|---|---|---|
| 粵典詞表 `wordslist` | `/faiman/analysis/wordslist.csv`、`.json` | 「粵典所收錄嘅詞表同佢哋嘅拼音」，即詞頭加讀音（簡介英文：「the wordlist and pronunciations of all entries recorded in the dictionary」） | 2026-10-08 |
| 粵典字表 `charlist` | `/faiman/analysis/charlist` | 字 → 粵拼 | 未讀 |
| 詞表使用頻率 `existingwordcount` | `/faiman/analysis/existingwordcount.csv`、`.json` | 「資料庫現有嘅詞」在「粵文庫」文章中出現的次數；頁面註明只含粵文庫見過的詞 | 2026-03-30 |
| 單字使用頻率 `charcount`；英粵對照 `englishindex` | 同上目錄 | 字頻；英文詞 → 粵語詞索引 | 未讀 |

**`wordslist` 實測**（`wordslist.csv` sha256 `617e714f…ea033b`，1,494,821 bytes，2026-11-10 20:57 UTC 下載；首行是 `Source,<頁面 URL>`，其後每行「詞,讀音…」，同一詞多讀音則多欄）：
- 62,635 列（JSON 同為 62,635 個鍵）；不只有單字：單字 4,692、兩字 33,673、三字 9,611、四字 8,899、五字以上 1,811，另有 3,949 列含非漢字（英文縮寫、標點、「⋯來⋯去」等）。
- 兩字以上全漢字的有 53,994 列；2,140 列結尾有「*」（簡介：含異體字的詞加星號），去星號並去重後 56,037 個。
- 內容不限口語：有「烏托邦」「佛羅倫斯」「超音波」這類書面語與外來詞，也有「飛髮舖」「屙茄」「千祈」「人唔笑狗都吠」這類口語、俗語、歇後語。需要清洗：標點、整句諺語、星號、混用的異體寫法。
- 和現有資料比：56,037 個去星號後的兩字以上詞，有 35,196 個在 rime-essay 內、20,841 個不在（例：「千祈」「作實」「穿梭巴士」「襟食」）。QIM 現有 196 個口語小詞，150 個在這份詞表內。
- 沒有詞頻。

**`existingwordcount` 實測**（sha256 未記；2026-11-10 下載）：36,601 列「詞,次數」，次數總和 3,322,075；兩字以上漢字詞 31,986 個，其中次數 ≥ 2 的 25,192、≥ 10 的 9,231、≥ 100 的 1,222；31,979 個在 `wordslist` 內。最高為「可以」7,280、「真係」6,365、「自己」6,262、「唔係」5,895。這是粵文庫（社群貼文語料）的原始出現次數、沒有分詞，只能當粗略排序依據，且只涵蓋粵典有收又在粵文庫出現過的詞。

**可釘住性**：兩個檔都是動態網址，沒有版本號、沒有 commit。可用的辦法是記 sha256 並把檔案收進 `data/`（不由 URL 建置）。Wayback Machine 有 `wordslist.csv` 的快照（時間戳 2023-03-25、2023-11-19、2024-01-21、2025-04-14，CDX 摘要在 2023-11 與 2024-01 相同、2025-04 不同），`existingwordcount.csv` 有 2023-03-25、2023-11-19、2024-01-21 三份（摘要相同）；同一個 URL 的內容會隨詞典更新而變。

**含義**：
- 條款層面，這四份資料由版權持有人自己的頁面標為 public domain，沒有署名義務，只寫「Credits … appreciated」。不需要遵守 NC 授權的條件。
- 需使用者判斷的點：(1) 這句 public domain 聲明的範圍。它寫在各資料集的說明下，沒有獨立的授權文件或 CC0 之類的法律文本；詞典整體的 NC 授權頁沒有提到這個例外，也沒有說例外的資料集是否仍受 NC 協議約束。`wordslist` 是「詞典全部詞頭」，和 NC 授權保護的詞典是同一個資料庫的投影。(2) 詞頭本身（短詞、事實）是否受版權或資料庫權保護，香港與美國有無 sui generis 資料庫權，本文未查。(3) 聲明可能日後改動；已下載的版本能否永久依賴，取決於聲明當時有效即可，這點屬法律判斷。
- rime-cantonese 的 upstream 與本文的比對結果一致：`word.csv` 有 50.5% 的詞出現在這份詞表內，README 的粵典來源連結也指向這份詞表，可見業界（包括該專案）把這份詞表當粵典的可用出口。

## 3. HKCanCor 詞表

**來源與授權**：`fcbond/hkcancor` `39aeadf`（2025-11-03）。`data/LICENSE` 是 CC BY 4.0 全文，`data/README` 寫「They are released under a CC BY license」，要求引用 Luke & Wong (2015)；repo 的 README 同。允許商用與修改，要署名、附授權連結、標示修改。

**格式**：`data/hkcancor-utf8.zip` 解出 `utf8/` 58 個轉寫檔，`<sent_tag>` 內一行一詞 `詞/詞性/粵拼/`，已分詞。

**實測**（zip 內 58 檔；統計 `<sent_tag>` 內所有詞項，含標點）：
- 詞項 160,836（README 稱約 23 萬詞，口徑不同）；不重複 7,359。
- 兩字以上漢字詞 5,102 個不重複（出現共 36,979 次）；出現 ≥ 2 次的 2,581、≥ 3 次的 1,684、≥ 5 次的 983。
- 其中 3,785 個在 rime-essay 內、3,866 個在粵典詞表內、660 個兩邊都沒有。不在 essay 的 1,317 個裡，出現 ≥ 2 次的 568 個。例：「留低」「銀包」「夠膽」「戇居」「直情」「遲啲」「返頭」「細路女」；也有人名、節目名（「子欣」「會考教室」）和轉寫瑕疵（「點樣樣」「一本便利」），要人工篩。
- 高頻詞多是虛詞片語：「即係」「但係」「嗰啲」「噉樣」「我哋」「而家」。

**含義**：授權最乾淨（CC BY 4.0，無 share-alike、無 NC），但量小，是證據充足的口語詞，只能補幾百個 essay 沒有的詞。QIM 已把 HKCanCor 當語料用（brief §12.1），詞表輸出需另標 CC BY 署名與「已改作」；自行整理的詞表若含從這裡抽出的詞，其授權就不再是純 CC0（詞本身是否構成可版權的內容屬法律判斷，需使用者判斷）。

## 4. 維基詞典（English Wiktionary）的粵語詞條

**授權原文**：`https://en.wiktionary.org/wiki/Wiktionary:Copyrights`：詞條原文「dual-licensed to the public under both」CC BY-SA 與 GFDL（GFDL 1.1 或之後）；CC BY-SA 要求署名、不暗示背書，材料以「same, similar, or compatible license」授權；GFDL 要求列作者、提供 transparent copy，頁內說可用一個顯著的連回 wiktionary.org 的連結同時滿足兩者。該頁沒有談資料傾印或何謂衍生作品。

**可釘住的機讀匯出**：
- [kaikki.org](https://kaikki.org/dictionary/rawdata.html) 的 wiktextract 輸出：英文版 `raw-wiktextract-data.jsonl`（24.8 GB，gz 2.9 GB），2026-10-30 抽取，來源是 enwiktionary dump 2026-10-01。頁面沒有授權聲明，只要求學術使用時引用 Tatu Ylonen 的 wiktextract 論文、希望連回 kaikki.org。檔名沒有日期，是會被覆蓋的最新版；日期化的快照要自己從 Wikimedia dump（[dumps.wikimedia.org](https://dumps.wikimedia.org/enwiktionary/)）按日期取，再自跑 wiktextract。
- `kaikki.org/dictionary/Cantonese/` 的 JSONL（9.4 MB，`Last-Modified: 2026-10-30`）標為 deprecated、將移除。內容是 enwiktionary 的 **Cantonese 語言段**：3,751 詞，全是單字（1,616）與粵拼音節（2,130）加 1 個二字詞——真正的粵語詞在 Wiktionary 掛在 **Chinese** 語言段（`{{zh-pron}}` 的粵語讀音），不在這份。
- 所以粵語兩字以上詞條要從 Chinese 全部詞條裡挑有粵語讀音或粵語標籤者。MediaWiki API 查到的 `Category:Cantonese lemmas` 有 192,203 頁，這包含所有有粵語讀音的 Chinese 詞條（含大量書面語），不能當口語詞數；其他分類的口徑（口語標籤）在 API 被限流前沒有查完，**粵語口語詞條的確切數量未查證**。

**share-alike 對「把詞頭建成資料包隨 App 散布」的要求**：
- CC BY-SA 4.0 原文（[legalcode](https://creativecommons.org/licenses/by-sa/4.0/legalcode.txt)，sha256 `28a9529c…1563`）：Adapted Material 是「derived from or based upon the Licensed Material」且被翻譯、改動、編排、轉換的素材。§3(b) 規定散布 Adapted Material 時，(1) 要用 CC 同一 License Elements 的授權（BY-SA 4.0 或之後，或 BY-SA Compatible License），(2) 附該授權的文本或連結，(3) 「不得附加不同或額外條件，或對 Adapted Material 施加會限制授權所給權利的 Effective Technological Measures」。§4(b)：若把資料庫內容的全部或實質部分放進你有 sui generis 資料庫權的資料庫，則該資料庫（不含個別內容）是 Adapted Material。
- CC FAQ（[creativecommons.org/faq](https://creativecommons.org/faq/)）關於集合與資料庫的說法：把 BY-SA 作品編成集合，CC 可以用 BY 授權集合整體，但個別作品仍是 BY-SA；資料庫以 SA 授權時，分享實質部分內容的人要對「你分享的任何資料庫」用相同或相容授權。
- 直接含義：資料包本身（含從詞條取出的詞頭）會需要用 CC BY-SA 4.0 或相容授權散布並署名；FAQ 與 §4(b) 的字面都只談「資料／資料庫」，沒有談包含它的程式。**資料包與 App 程式碼是各自獨立的作品、還是 App 因此成為 Adapted Material；商店的 DRM 與 §3(b)(3) 的 Effective Technological Measures 是否衝突；只取詞頭（單詞、事實）是否產生需要 SA 的 Adapted Material：需使用者判斷（建議徵詢法律意見）。**
- 與現況的關係：rime-essay 是 LGPL-3.0，已需判斷同類問題（`data-sources.md` §11）；加入 BY-SA 資料包是第二種需獨立標示的授權。QIM 的架構把詞庫作為獨立資料包（brief §12.1「字級 bigram」那條已有同樣的處理方式），可把 BY-SA 資料限制在單獨一個包內。
- rime-cantonese 的處理：維護者 2020 年對英文維基詞典（CC BY-SA 3.0）資料，結論是放獨立檔案、獨立授權（[#91](https://github.com/rime/rime-cantonese/pull/91)）。

## 5. 其他

### 5.1 Cifu（Lai & Winterstein 2020）
- `gwinterstein/Cifu` `896ba8c`（2020-08-27 為最後 push）：`Lexicon/Cifu-v1.txt` 51,798 詞（兩字以上 47,863；各體裁次數合計 ≥ 2 的 44,028；有口語（成人口語）次數 ≥ 1 的 9,993），欄位含四種體裁（書面、成人口語、兒童口語、對兒童說話）的次數與每百萬詞頻率。
- 授權：repo `LICENSE` 是 GPL-3.0 全文；LREC 2020 論文 5.5 節：「Cifu is released under the GNU General Public License v3.0」。
- 來源（論文）：分詞用 jieba 加 CLAN MOR 詞表；口語語料是 HKCanCor、HKCAC、CantoMap 與兩個兒童語料；書面是爬取的網路小說章節（論文寫 3,841 章）。詞表的語料授權沒有逐一列出。
- 含義：有次數是優點，但 GPL-3.0 是強 copyleft，與 App 商店散布的相容性比 LGPL 更難說；`data-sources.md` §11 對 GPL 資料檔已標為需使用者判斷。不建議當首選。

### 5.2 CC-Canto（cantonese.org，Pleco）
- [下載頁](https://cantonese.org/download.html) 說三個檔都以 CC BY-SA 3.0 發佈；`cccanto-170202.zip`（`Last-Modified` 2019-07-29，sha256 `0cc9e600…f27`）檔頭：「Version 2017-02-02」「Copyright (c) 2015-17 Pleco Inc.」「Distributed under a Creative Commons Attribution-Share Alike 3.0 license」。格式是 CC-CEDICT 式的一行一詞（繁、簡、普通話拼音、粵拼、英文釋義），34,335 條、29,698 個不重複詞頭、24,059 個兩字以上漢字詞。
- 內容是 CC-CEDICT 的粵語化（英文釋義為主），書面語與普通話詞很多，真正口語詞佔比未量；24,059 個裡 13,971 個不在 essay、5,534 個既不在 essay 也不在粵典詞表。無詞頻。
- 授權含義同 §4（BY-SA，3.0 版，不是 4.0；3.0 的 share-alike 條款文字與 4.0 不同，本文未逐條比對）。無版本化下載連結，只有固定檔名的 zip，可用 sha256 釘。

### 5.3 開放詞典 kaifangcidian 粵語詞典
- [詞典頁](http://kaifangcidian.com/han/yue/) 寫 6,654 字、84,836 詞、16,079 例句；版權頁（`/yue/cc/`）：「本站上所有資源，除非另有聲明，都以創作共用「署名 3.0」許可協議授權發佈」。
- 但詞典頁沒有資料下載，GitHub `kfcd` 組織的 repo 只有字典（`yyzd`，CC BY 3.0 的 LICENSE）、拼音轉換表與量詞資料庫，沒有詞的資料庫。`yyzd` README 寫「口語詞可參考開放詞典的粵語詞典資料庫」。要取詞只能爬網頁，爬取是否符合站方意願未查證。rime-cantonese 2019 年曾納入這份資料（commit `bd8349b`）。

### 5.4 LSHK 粵拼詞表（corpus.eduhk.hk/JPwordlist）
- 約 15,000 詞（工作小組頁）／搜尋結果 15,740 筆（2019 快照）；來源：教育局《香港小學學習字詞表》（EDB）、二十世紀中期香港粵語語料庫、片岡新與李燕萍的教材（工作小組頁），後來由 LSHK 補常用口語詞。
- 站台 2026-04-01 起 404，Wayback 有 2019 快照（頁面沒有授權字樣）。Anki 分享頁稱以教育局字詞表為基礎。教育局字詞表的版權屬政府，本文未查其再用條款。授權不明、原站已下線，不建議用。
- 同組織的 `lshk-org/jyutping-table`（字 → 粵拼，CC BY 4.0）是單字表，與詞庫無關。

### 5.5 PyCantonese
- `jacksonllee/pycantonese` v5.0.0（`3d729c0`，2026-05-26），MIT；README：內含資料「all of which are permissively licensed」：HKCanCor（CC BY）、rime-cantonese（CC BY 4.0，指向 upstream `bea0d7a` 與下游 `33f7d81`）、Common Voice 粵語（MPL 2.0）、Cantonese-Traditional Chinese Parallel Corpus（CC0）。
- 沒有獨立的詞表；`chars_to_jyutping.json` 是 rime-cantonese 衍生，分詞模型 `segmenter.fb.zst` 是訓練出來的二進位。它對 rime-cantonese 的處理是照上游標示轉述為 CC BY 4.0，沒有額外的來源查證，故不增加新的授權依據。

### 5.6 用 QIM 已有語料自己「詞發現」
- 語料：Common Voice 粵語句子（GitHub `common-voice` `ec845c7`，`yue` 9,443 句、`zh-HK` 19,020 句，CC0）、粵語維基（CC BY-SA 4.0，brief §12.1）、HKCanCor（CC BY 4.0）。
- 規模上限：Common Voice 兩份共約 2.8 萬句，以 PMI 或詞典切分能找出的口語詞數量很有限，且句子是朗讀用的語句，不是自然對話；粵語維基是書寫粵語與書面語混用，從它發現的詞若直接作為資料釋出，會碰到 §4 的 BY-SA 問題（詞是事實還是衍生，需使用者判斷）。本文沒有實際跑詞發現，只列出方法可行與限制。
- 優點：授權路徑單純（CC0 的語料自己統計出的詞，不受粵典、rime 影響），但需要自己的詞表作為「這是詞」的判準，否則只是 n-gram。

### 5.7 其他曾列入但不適用
- 粵語審音配詞字庫（中大）：網站服務，無批量授權，rime-cantonese 以爬取方式取用（`lexi_can_crawler`），不用。
- CantoDict：條款頁未取得，未見下載，不用。

## 6. 結論

「可／不可／不確定」只回答「條款字面上允許隨商店 App 散布」，不是合規結論；括號內是需使用者判斷的點。

| 來源 | 隨商店 App（含日後收費）發佈 | 署名與 share-alike | 規模（兩字以上口語候選） | 備註 |
|---|---|---|---|---|
| 粵典 `wordslist` + `existingwordcount`（public domain 聲明） | **可，需使用者判斷範圍**：資料頁自標 public domain，不受 NC 協議條款約束的前提是該聲明有效（詞典整體仍是 NC，聲明與 NC 頁不一致處未被說明） | 無法律義務；「Credits to words.hk appreciated」建議照做。無 share-alike | 56,037 個詞（去星號去重；需清洗），其中約 2 萬不在 essay；`existingwordcount` 提供粗頻次，≥ 2 次 25,192 詞 | 動態網址無版本，需收進 `data/` 並記 sha256；更新日 2026-10-08 與 2026-03-30 |
| 粵典詞典整體（含釋義、例句） | **不可**：NC 協議，免費 App 是否算 Commercial Use 要看 §2(f)，收費、增值服務、廣告都明列為商用；§3(b) 六個月無回覆豁免有嚴格限制；也無公開全量下載 | 署名＋版權告示＋免責聲明＋特別鳴謝（或單一 URL 告示）；不得再授權 | 53,000+ 條（論文） | 要用須向 Hong Kong Lexicography Limited 申請另簽 |
| rime-cantonese / upstream（CC BY 4.0） | **不確定**：條款本身允許，但詞條來自粵典（NC 部分未區分）、紙本詞典、用戶詞庫、爬取的 LSHK 詞表，頻次含 GPL-3.0 的 Cifu；repo 沒有授權鏈說明，也無逐條來源欄可取乾淨子集（需使用者判斷：是否接受上游整理者的 CC BY 標示） | CC BY 4.0：署名、連授權、標示修改。無 share-alike；`maps` 檔是 ODbL（share-alike），不用即可 | `word.csv` 92,531 個詞，其中約 46,700 個也在粵典 PD 詞表內；無頻次（頻次在 `essay-cantonese.txt`） | 可釘 commit |
| HKCanCor 詞表 | **可**（CC BY 4.0，允許商用） | 署名 Luke & Wong (2015)、連授權、標示已改作；無 share-alike | 5,102 個詞（≥ 2 次 2,581；不在 essay 的 1,317，其中 ≥ 2 次 568）；有次數 | 可釘 commit `39aeadf`；量小 |
| 維基詞典（enwiktionary） | **不確定**：CC BY-SA 4.0／GFDL 雙授權；詞頭資料包要 BY-SA；是否牽連 App 與商店 DRM（§3(b)(3)）需使用者判斷 | CC BY-SA 4.0：署名＋同授權散布資料包；GFDL 另有要求 | 粵語口語詞數量未查證（`Cantonese lemmas` 192,203 頁含書面語） | kaikki 的檔會被覆蓋，要日期化快照需自取 dump 再跑 wiktextract |
| Cifu | **不確定偏不可**：GPL-3.0 強 copyleft（需使用者判斷） | GPL-3.0 | 47,863 個詞，有四體裁次數 | 語料授權未逐一列出 |
| CC-Canto | **不確定**：CC BY-SA 3.0 | 署名＋同授權 | 24,059 個詞（書面語多、無頻次） | zip 可 sha256 釘 |
| 開放詞典 kaifangcidian | **不確定**：網站聲明 CC BY 3.0，但沒有公開資料下載，只能爬網頁 | 署名 | 84,836 詞（頁面數字） | 取得方式未查清 |
| LSHK 粵拼詞表 | **不確定**：授權未見，原站已下線 | 未見 | 約 15,000 | 不建議 |
| 用 CC0 語料自己詞發現 | **可**（路徑單純） | 無 | 上限約數千且品質待驗證 | 未實測 |

**採用**（2026-11-11）：使用者選第 2 條，只建 HKCanCor；粵典的 public domain 聲明與詞典本體的非商業授權對不上，算查不清，不推翻「不從粵典複製」，待粵典書面確認（brief §1.5、§12.1）。實作見 plan §4.2。

**較可行的兩條路**（只列選項與義務，不拍板）：
1. **粵典 `wordslist`（public domain）作詞表，`existingwordcount` 作粗略排序，兩者加 HKCanCor（CC BY）驗證與補頻**。義務：HKCanCor 署名與標示改作；粵典建議列 credit；全部檔案收進 `data/` 並記 sha256 與取得日期（來源網址無版本）。待使用者決定：(1) 接受「資料頁自標 public domain」作為依據，還是先寫信向 Hong Kong Lexicography Limited 取得書面確認（網站有聯絡電郵，沒有專門商業授權窗口）；(2) `data/lexicon/yue-colloquial.tsv` 檔頭寫「不從 rime-cantonese、粵典或其他有授權的詞表複製」，若採此路要改寫這句與 brief §12.1 的對應條。清洗工作量：去星號、去標點與整句諺語、異體寫法對到 QIM 現有的 OpenCC 常見寫法規則。
2. **只用 HKCanCor（CC BY）加自行整理，維持現狀擴充**。義務最少、風險最低，量最小：多約幾百個 essay 沒有的詞。

## 7. 未查證

- Hong Kong Lexicography Limited 對 public domain 資料集與 NC 協議關係的書面說明；商業授權的實際窗口與條件（頁尾電郵沒有發信詢問）。
- 香港與美國有無 sui generis 資料庫權；單詞詞頭與短語是否為可版權的內容（影響 §2、§4 的判斷，需使用者判斷）。
- `charlist`、`charcount`、`englishindex` 三份資料的內容與更新日（只讀了頁面說明，沒下載）。
- 粵文庫語料的大小與來源授權（`existingwordcount` 的語料）。
- rime-cantonese 的 `jyut6ping3.phrase.dict.yaml`（搜狗／QQ／百度詞條）與 upstream `phrase_fragment.csv` 的對應；`cifu-freq.txt` 與 Cifu 頻次是否逐字相同；HEAD `essay-cantonese.txt` 是否仍含 Cifu 的頻次（分支 `703c56b` 的內容未比對）。
- 英文維基詞典 Chinese 詞條裡有粵語讀音或標示粵語口語的兩字以上詞條數（Wikimedia API 限流，沒有查完）；GFDL 對資料包的要求。
- CC BY-SA 3.0 與 4.0 的 share-alike 文字差異（CC-Canto、kaifangcidian 用的是 3.0）。
- 開放詞典詞條的下載管道與站方對爬取的態度；LSHK 詞表（EDB 來源）的再用條款。
- CTCPC（HuggingFace，PyCantonese 稱 CC0）的授權原文與規模：該資料集已改名，未讀；Cantonese 的其他 CC0／CC BY 詞表（如 EDB 字詞表的政府版權）未窮盡搜尋。
- 用 Common Voice／粵語維基自行詞發現的實際產出量與準確度。

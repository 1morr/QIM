# 11c: 核心：重排看 lattice 路徑、可插到第 2 位

Status: done
Blocked by: 11d
讀: `docs/research/model-ladder.md`〈停鍵層〉；plan §2.1（重排、`paths`、上下文）、§2.4（排位規則）、§2.7；ADR 0005；`scripts/experiments/llm-rerank/README.md`〈提示〉〈給票 11〉；`scripts/experiments/neural-rerank/README.md`〈接法〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M3」各條（重排與規則 3、4 那條）；票 15 的 Comments（記住的字串）

## What to build

票 09、10 都發現重排要看 lattice 的前 9 條整句路徑才有效，現在的 `apply_rerank` 只能在候選列第一頁內排（1.4M 的增益會從 8.5 點掉到 2.3～6.3 點）。把重排的要求與結果改成以路徑為單位。等票 11d 的 `qim-neural` 做完再開工（新基線的重量已做完，1.4M 在個人集改成只報不擋後採用，票 25）。

- `RerankRequest` 改帶：`generation`、前文（最多 50 字，brief §8）、`Session::paths(9)` 的路徑（文字、分段、路徑分）。候選列第一頁不再整份送出。
- `apply_rerank(order)` 的 `order` 指路徑的序號、好的在前：取第一條和首選不同的路徑放第 2 位；在候選列裡就移上來，不在就插入；其餘候選照原順序往後接、去重。首選不動（ADR 0005）。插入的候選選了之後照那條路徑的分段上屏與學習，同選一條多段的整句。
- 定重排和排位規則 3、4、5 的先後（M1 留下：重排會把第 2 位的使用者詞或英文擠到第 3；記住的 email 也排第 2 位起），寫回 plan §2.4。
- 網頁的假重排（`web/src/rerank.ts`）改成用路徑：「重排」選單的假模式改成把某一條路徑排上來，讓 Playwright 驗得到插入。
- 寫回 plan §2.1（重排那幾條）、§2.7。

## 驗收

- [x] 單元測試：候選列沒有的路徑插到第 2 位、已在候選列的移上來、`order` 第一個就是首選時取下一條、序號超出範圍或重複、過期的 `generation`、規則 3、4、5 的先後、聯想不給要求、插入的候選選了之後的上屏與學習。
- [x] Playwright：假重排把候選列沒有的路徑插到第 2 位，讀回位置、選它上屏讀回；截圖。
- [x] `cargo xtask bench --check` 不變（bench 不跑重排）；實際操作網頁亂按一輪；`cargo xtask check` 綠燈。

## Comments

- 2026-11-11 曾隨票 11d 的重量結果 wontfix（`30a043c`）；使用者把模型階梯的「個人集不退步」改成只報不擋後 1.4M 採用，票 25 revert 那個 commit，本票恢復。
- 2026-11-12 實作：`RerankRequest` 改帶 `generation`、前文（`Session` 記的前文接上選定的分段，最後 `RERANK_CONTEXT` = 50 字）與 `paths(RERANK_PATHS)`（9 條，`ScoredPath` 加 `segments`）；刪 `RERANK_LIMIT`。`apply_rerank(order)` 的 `order` 是路徑序號：取第一條顯示的字和首選不同的（序號 ≥ 9 不算），候選列有同樣顯示的字就移上來、沒有就插入 `Lattice::whole_path(k)`（選了照路徑的分段上屏、學習，同選多段整句）。寫回 plan §2.1、§2.4、§2.7、§3.1、GLOSSARY〈重排〉、spec 帶到 M3 的那條。
- 使用者決定（AskUserQuestion）：規則 3、4、5 優先，重排排在它們後面（`CandidateSpan::pinned_after_top`，排位規則釘在第 2 位起的標記；首選被規則 1 佔走時規則 2 移到第 2 位的也照同一條，plan §2.4 註明是延伸）；移到某一段改字時不給要求、也不套用。
- 自己的取捨：翻到後面的頁時收到的結果先記著（`waiting_rerank`），回到第一頁時 `generation` 沒變才套用、那次翻頁的快照帶新的 `generation`（code-review 指出原本直接丟掉，回第一頁後這輪重排就沒了）。網頁殼因此會對新的 `generation` 再要求一次，結果名次不變、不重畫；多一次要求，可接受。
- 網頁：「重排」選單的假模式 `raise-fifth` 改成 `raise-last`（最後一條路徑排最前）；`RerankRequest` 綁定改成 `paths`（文字）與 `scores`（路徑分）兩個陣列、不帶分段。模型階梯實驗（`scripts/experiments/model-ladder/`）跟著改 `PAGE_SIZE`、`ScoredPath.segments`，`cargo test --release` 過。
- `bench --check`（開工 `b377cf3`，`cargo xtask data` 重建後）：改動前後的表逐行相同、`bench 通過`；答錯／修正 yue-wiki 941／3239、zh-wiki 979／3493、common-voice-yue 892／3249、hkcancor 572／2148，都等於 baseline，有前文四集同（給前文 468／477／187／392）；12 個集合都沒有翻盤，固定題 84/84。bench 不跑重排，所以不變是預期的。本機沒有個人集。
- Playwright：`e2e/rerank.spec.ts` 在 `anr` 用 `raise-last`，第 2 位插入「日陪」（展開面板讀到的全部候選裡沒有），首選「門口」不動，按 2 上屏「日陪」；截圖 `.scratch/m3/evidence/11c-raise-last.png`。亂按一輪：固定種子 400 步（字母、空白、退格、Esc、Enter、← → ↓ ↑、翻頁、Shift、標點、數字鍵，夾雜切重排三種模式、簡體、停鍵 400 ms），沒有頁面錯誤，停鍵前後首選都沒變；截圖 `11c-monkey.png`，腳本沒進 repo。
- code-review 已處理：plan §2.4 規則 5、emoji 段「重排要求的前 9 個候選含它」改寫（要求不含它們，但前文可能含 email，遮蔽仍歸票 22）；plan §2.1 刪「以前只在第一頁內排」的舊版字樣；model-ladder README 的 `RERANK_LIMIT`；拿掉 `ScoredPath.segments` 的 `serde(default)`（內部格式不留相容層）；`held` 改名 `pinned_after_top`、`reranks` 改名 `can_rerank`；`fakeOrder` 去掉型別斷言；補測試：選定開頭一段後選插入的路徑（上屏與自動造詞）、簡體模式比顯示的字、翻頁中的結果回第一頁才套用與候選變了就作廢。
- code-review 未處理：文件寫死 9 與 50（文件要寫數字，常數名一起寫了）；wasm 不帶分段（模型用不到，註解說明）。給票 11e：簡體模式下前文是簡體字（殼給的與上屏的都是顯示的字），模型字表是繁體，前文會多 `<unk>`，和模型階梯量的條件不同；要不要在核心或殼轉回繁體由 11e 定。
- 2026-11-12 協調者回饋（插入的整句選兩次會不會成首選）：規則 2 只移動候選列已有的候選，核心測試的夾具沒有語言模型，學到的分數拉不進前 3 條句子，所以那邊只斷言學到的列。改用真資料包在網頁實打 `anr`、`hapi`、`omdhm`：插入的整句（日陪、簡勾、人殊生）選一次後，下次打不等重排就在第 2 位；選兩次後第三次打，不等停鍵就是首選。不用修，加 e2e「插入的整句選兩次後成首選，不等停鍵」守住；截圖 `.scratch/m3/evidence/11c-chosen-twice.png`。

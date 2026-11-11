# 11c: 核心：重排看 lattice 路徑、可插到第 2 位

Status: ready-for-agent
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

- [ ] 單元測試：候選列沒有的路徑插到第 2 位、已在候選列的移上來、`order` 第一個就是首選時取下一條、序號超出範圍或重複、過期的 `generation`、規則 3、4、5 的先後、聯想不給要求、插入的候選選了之後的上屏與學習。
- [ ] Playwright：假重排把候選列沒有的路徑插到第 2 位，讀回位置、選它上屏讀回；截圖。
- [ ] `cargo xtask bench --check` 不變（bench 不跑重排）；實際操作網頁亂按一輪；`cargo xtask check` 綠燈。

## Comments

- 2026-11-11 曾隨票 11d 的重量結果 wontfix（`30a043c`）；使用者把模型階梯的「個人集不退步」改成只報不擋後 1.4M 採用，票 25 revert 那個 commit，本票恢復。

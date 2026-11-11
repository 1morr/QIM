# 11d: 停鍵層：`qim-neural` 與 1.4M 模型資料包

Status: ready-for-agent
Blocked by: 11b
讀: `docs/research/model-ladder.md`〈停鍵層〉〈雲端層〉〈停鍵層在新基線重量〉〈依句長分組與口語長前文〉；`scripts/experiments/neural-rerank/README.md`（全部）；plan §1.3、§1.4、§2.7、§4.6、§4.7；brief §11、§12.3；`docs/research/ios-feasibility.md`〈小型神經模型〉；spec〈採用門檻〉〈共同規則〉

## What to build

新基線的重量已做完（2026-11-11，`docs/research/model-ladder.md`〈停鍵層在新基線重量〉）：口語前 2 名 +5.5 點、維基 +3.0 點、成本在門檻內；個人集 −0.53 點當時擋下，使用者 2026-11-11 把模型階梯的「個人集不退步」改成只報不擋（spec〈採用門檻〉，票 25），1.4M 照新門檻採用，本票恢復。

- 模型：直接用 `build/neural-rerank/lstm-c0.5/`（票 11d 重訓的那份，q8、λ 2）；`build/` 不進 git，主 checkout 有一份，worktree 裡從主 checkout 複製過來，檔不在才照 `scripts/experiments/neural-rerank/README.md` 重訓。實作時可以在 dev 上比較重訓的版本（例如換種子，這份訓練不穩），只在 dev 挑，不重看 test 與個人集。
- 新 crate `qim-neural`（plan §1.3，依賴 `candle-core`、`candle-nn`）：從位元組載入 q8 GGUF 權重與字表；給前文與路徑、回名次；前綴快取、路徑長度不一時補齊遮罩；前文取最後 50 字，標點收成 `<sep>`、字母數字收成 `<unk>`；接法 `路徑分 + λ·神經分`。實作從 `model-ladder` 的 `src/neural.rs` 搬一份，產品只留這份；實驗的兩份不動。
- 訓練程式從 `scripts/experiments/neural-rerank/` 搬到正式的位置（`docs/development.md` 寫怎麼訓練）；訓練不是決定性的，不進 `cargo xtask data --verify` 的決定性檢查，改用 `reference.json` 核對 PyTorch 與 candle 的分數。
- 模型資料包：plan §4.6 的 manifest 加 `model` 種類，`LICENSE`、`NOTICE` 寫訓練語料的授權（brief §11），加進線上更新的索引（plan §4.7；真的建 GitHub release 前問使用者）。
- 寫回 plan §1.3（若接法與表不同）、§2.7、§4.6、brief §12.5。

## 驗收

- [x] 新基線的重量結果寫進 `docs/research/model-ladder.md` 與 brief §12.5（2026-11-11 做完，見 Comments）。
- [ ] `qim-neural` 單元測試：和 `reference.json` 的分數差 ≤ 0.0001 nat、有沒有前綴快取同分、補齊遮罩（變異驗證拿掉會紅）、空路徑不 panic。
- [ ] 這台 Mac 上重排一次（9 條路徑、前文 50 字）p95 ≤ 40 ms；模型資料包有 manifest、`LICENSE`、`NOTICE`。
- [ ] 定案的模型跑 `model-ladder`（`neural --split test`、對新基線 `compare`，或等價指令），在 Comments 報口語、維基、個人集的前 2 名與翻好／翻壞數；個人集只列數字與題號、只報不擋（plan §2.7：`cargo xtask bench` 不跑重排，看不到停鍵層對個人集的影響）。
- [ ] `cargo xtask check` 綠燈（含 plan §1.3 的依賴規則）。

## Comments

- 2026-11-11 重量沒過，停下收回（使用者照門檻選），下面三條驗收不做。數字與量法在 `docs/research/model-ladder.md`〈停鍵層在新基線重量〉：
  - 在 `c325e3b`（合併 main `177a8d2` 前）量。票 09 的模型檔已不在，照 README 重訓 `lstm-c0.5`；這次 dev 損失比票 09 高（訓練不穩），只訓練一次。f32 對 `reference.json` 差 0.0000 nat。
  - dev 掃 λ 0.5／1／2：口語前 2 名 1,691／1,695／1,697（新基線 1,548），定 λ 2。
  - test 一次（q8）：口語 56.48% → 62.02%（198／52）過；維基 63.43% → 66.43% 過；個人集前 2 名 85.76% → 85.23%（3,874 → 3,850，133／157，p 0.18），門檻要 ≥ 3,852，差 2 題沒過。
  - 個人集題號只在本機（`build/model-ladder/test/<目錄>/personal.tsv`），不進票；個人集 baseline 沒動（重排不進 `cargo xtask bench`）。
- 收回（同一 commit）：票 11c、11e wontfix；plan §1.3 的 `qim-neural` 列與 `qim-wasm`、`qim-ffi` 欄裡的它、`xtask/src/deps.rs` 的 `RULES`；plan §1.4 的 candle 列；plan §2.1（重排、`paths`）、§2.7、§8；brief §1.5（1.4M 移到「不做」）、§7（拿掉停鍵重排的開關）、§8、§11、§12.5；spec〈範圍〉〈驗收〉〈票序〉；票 23 的 Blocked by；票 25 的 Blocked by 改成 11b、註明 1.4M 未採用照樣量（協調者要求）。
- 雲端重評：照 `docs/research/model-ladder.md`〈雲端層〉問使用者，使用者選不重量（flash 對舊基線 59.5% 只比新基線多 3.0 點，門檻要 +6.0）；票 22 維持 wontfix，註明條件成立與決定。
- 沒做：`qim-neural`、訓練程式搬家、模型資料包。實驗的程式（`scripts/experiments/neural-rerank/`、`model-ladder`）沒動。
- code-review（`c325e3b` 起，Standards 與 Spec 兩軸）：沒有硬違規，停止條件與數字驗算對得上。發現都已修：model-ladder.md 的 1.4M「採用」列、brief §7 開關的引用、`Session::paths` 的去留理由、「票 23 改成 11b」的描述；票 22 的「已採用的 1.4M」與 11c 引用；票 25 的「讀:」、雲端比較對象（改成比基線 +6.0）、`build/` 只在本 worktree。沒處理：11c、11e 的 Comments 相同（交叉記錄，故意的）；λ 最好的落在網格邊界 2、重訓的 dev 損失較高（已寫進 model-ladder.md，使用者照門檻停下，不再量）。
- 2026-11-11 恢復（票 25）：使用者把模型階梯的「個人集不退步」改成只報不擋，1.4M 照新門檻採用；`git revert 30a043c` 收回上一條的「收回」，本票、11c、11e 改回 `ready-for-agent`，票 23 的 Blocked by 加回。上面的重量數字不變；剩下的驗收照做，重量那步不重做。

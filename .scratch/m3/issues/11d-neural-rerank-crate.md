# 11d: 停鍵層：`qim-neural` 與 1.4M 模型資料包

Status: done
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
- [x] `qim-neural` 單元測試：和 `reference.json` 的分數差 ≤ 0.0001 nat、有沒有前綴快取同分、補齊遮罩（變異驗證拿掉會紅）、空路徑不 panic。
- [x] 這台 Mac 上重排一次（9 條路徑、前文 50 字）p95 ≤ 40 ms；模型資料包有 manifest、`LICENSE`、`NOTICE`。
- [x] 定案的模型跑 `model-ladder`（`neural --split test`、對新基線 `compare`，或等價指令），在 Comments 報口語、維基、個人集的前 2 名與翻好／翻壞數；個人集只列數字與題號、只報不擋（plan §2.7：`cargo xtask bench` 不跑重排，看不到停鍵層對個人集的影響）。
- [x] `cargo xtask check` 綠燈（含 plan §1.3 的依賴規則）。

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
- 2026-11-11 實作（開工 `e181d56`）：
  - 定案的模型：照用 `lstm-c0.5`（使用者選，不換種子重訓）。主 checkout 沒有字表，在本 worktree 跑 `cargo xtask data` 與 `train.py prepare` 重建（token 數與 README 相同）；dev 口語前 2 名 63.3%（λ 2、q8）與上一輪的 1,697 題一致，字表對得上。
  - 模型存放（使用者選）：q8 權重、字表、`model.toml`（λ、字表外的字數、訓練出處）進 git 的 `data/neural-rerank/`，`cargo xtask data` 只包成模型資料包 `zh-hant-neural-rerank`（種類 `model`，1,735,793 位元組，授權 `CC-BY-SA-4.0 AND CC-BY-4.0 AND CC0-1.0`）。f32 與 `reference.json` 在 `cargo xtask neural-import` 核對（差 0.000041 nat），CI 用 `train.py fixture` 產生的極小模型核對 ≤ 0.0001 nat；本機有 `build/neural-rerank/lstm-c0.5/` 時測試另比真的模型。產品量化出的 q8 和實驗的 `model.gguf` 位元組相同。
  - `qim-neural`：只搬 LSTM（23M 沒採用）；`Reranker::new(權重, 字表, 字表外的字數, λ)`、`rank(前文, [(文字, 路徑分)])` 回序號；同分照原順序（實驗是同分看路徑分，路徑本來就依路徑分排，結果相同）；空字串的路徑神經分 0、沒有路徑回空的。變異驗證：拿掉補齊遮罩、前綴狀態改成空前文，各有測試變紅。
  - `ModelPack` 在 `qim-core`（核心只核對、解開，權重原樣交給殼），`Engine::open` 收到它回 `NotForEngine`；`qim-databuild` 加依賴 `qim-neural`（匯入時核對、量化，`--verify` 加 `NeuralRerank` 抽查），改 plan §1.3 與 `RULES`。`pack-release` 照常收進索引；沒建 GitHub release。
  - 訓練程式搬到 `scripts/neural-rerank/`（`train.py`、`train.py.lock`），加 `fixture` 子指令；實驗的兩份 `neural.rs` 沒動，README 的路徑改了。
  - 延遲（`cargo run --release -p qim-neural --example latency`，9 條 × 18 字、前文 50 字、2 條執行緒，500 次）：p50 5.28 ms、p95 5.41／5.92 ms（兩次），載入 0.5–1.7 ms；量時 load average 約 9。
  - 模型階梯（`e181d56` 加本票，`eval --split test`、`neural --split test --model data/neural-rerank/model.gguf --lambdas 2`、`compare`），前 2 名；首選與修正次數不變：
    - 口語 56.5% → 62.0%（翻好 198／翻壞 52）；維基 63.4% → 66.4%（249／129）；有前文·前文50 61.0% → 66.9%；口語長前文·前文50 57.0% → 65.9%。
    - 個人集（只報不擋）85.76% → 85.23%（3,874 → 3,850 題，翻好 133／翻壞 157，p 0.18），和上一輪重量相同。題號（`rime-userdb.tsv` 的行號）：
      - 好：55、94、119、122、134、194、211、300、308、410、435、436、487、499、527、550、551、592、667、829、840、885、895、897、932、961、994、1012、1072、1074、1134、1169、1193、1261、1346、1358、1361、1428、1492、1508、1523、1532、1634、1665、1712、1721、1752、1855、1897、1909、1956、2059、2085、2093、2112、2152、2191、2203、2295、2358、2377、2380、2508、2518、2537、2569、2678、2721、2770、2841、2846、2898、2970、3068、3072、3076、3111、3124、3237、3241、3268、3291、3294、3328、3334、3340、3346、3369、3391、3398、3466、3590、3699、3716、3717、3723、3733、3744、3764、3767、3773、3784、3810、3822、3899、3917、3926、4000、4038、4047、4049、4064、4077、4080、4085、4086、4094、4100、4103、4114、4156、4175、4314、4324、4325、4333、4334、4372、4420、4446、4464、4473、4487
      - 壞：64、138、182、184、204、212、254、258、265、334、348、390、422、483、506、587、655、671、696、697、716、778、783、816、843、852、920、947、957、976、1093、1098、1150、1199、1272、1317、1345、1370、1433、1438、1440、1448、1475、1490、1499、1596、1603、1608、1664、1692、1718、1725、1731、1750、1753、1778、1811、1822、1824、1830、1838、1891、1977、2009、2068、2071、2153、2205、2233、2238、2246、2303、2327、2342、2344、2362、2408、2437、2450、2454、2533、2566、2577、2592、2598、2607、2656、2663、2677、2687、2738、2739、2766、2833、2847、2871、2876、2888、2889、2953、2971、3002、3059、3060、3075、3080、3085、3120、3216、3242、3249、3263、3266、3337、3339、3352、3389、3418、3435、3498、3538、3542、3667、3712、3913、3925、3959、3970、3981、4005、4012、4036、4046、4074、4075、4153、4165、4176、4195、4208、4244、4256、4282、4316、4323、4338、4340、4347、4385、4411、4412、4413、4414、4417、4433、4443、4459
  - 個人集 baseline 沒動：`cargo xtask bench` 不跑重排（plan §2.7），本票沒改核心的打分。
  - `cargo xtask bench --check`（本票改了 `qim-core` 的資料包種類與 `Engine::open`，沒改打分）：通過，各集合（含個人集）沒有翻盤、固定題 84/84，數字與開工時的 baseline 相同，baseline 沒動。
- code-review（`e181d56` 起，Standards 與 Spec 兩軸）。已修：`verify.rs` 的 `neural_rerank` 插在 `words_hk_lexicon` 的註解與函式之間；plan §1.2 的 `xtask/` 列漏了 `neural-import`、§1.4 的 `serde_json` 沒寫 `qim-databuild` 的新用途；`Error::Inference` 的說明不含輸入錯誤；兩處 `match bool`；`neural.rs` 兩份讀字表的邏輯併成 `vocab_chars`；`--verify` 的 `NeuralRerank` 原本只看名次是不是排列，改成同分的三條路徑裡要把「今日天氣好好」排第一（測試證明極小的隨機模型與壞掉的權重都過不了）；model-ladder.md 指的 README 位置。沒處理：
  - `Reranker::new` 的四個參數（權重、字表、字表外的字數、λ）在呼叫端一起出現：plan §1.3 不讓 `qim-neural` 依賴 `qim-core`，等票 11e 接網頁時看要不要在 `qim-neural` 包一個型別。
  - manifest 的種類叫 `model`（和 `language-model` 並列）：票指定的名字，GLOSSARY 已寫明「停鍵層模型」。
  - `neural-import` 先檢查 λ、`Reranker::new` 又檢查一次：前者是為了在讀檔、量化前就擋下，故意的。`reference.json` 的 `Case` 在 `qim-neural` 的測試與 `qim-databuild` 各一份（不同 crate、一份是測試）。`latency` example 自己讀 `model.toml` 的兩個值（`qim-neural` 不能依賴 `qim-databuild`，加 toml 只為 example 不值得）。
  - 前綴快取的測試拿 `score` 比 `score_one`，兩者共用 `prefix_state`；`prefix_state` 本身的錯由對 PyTorch 的參考分數測試抓（極小模型的參考題含前文）。
  - 給票 11e：`cargo xtask web` 會把模型資料包一起複製進 `web/public/packs/`；網頁的 `sessionPacks()` 只排除字根提示，使用者從檔案匯入模型資料包時 `Engine::open` 會回 `NotForEngine`、網頁退回內建。接神經重排時要把種類 `model` 和字根提示一樣另外載入。

# 16: 簡繁

Status: done
Blocked by: 13
讀: plan §1.4（`ferrous-opencc`）、§2.2（後處理）、§4.4、§8（`ferrous-opencc` 那列）；brief §4.1、§5.2；spec〈共同規則〉

## What to build

開簡體模式後，候選與上屏都是簡體：打 `anr` 首選「门口」。

- 開工先驗證 `ferrous-opencc` 能不能編進 `qim-wasm`（WebAssembly），結果寫進 `## Comments`；不能就照 plan §8 退回自讀 OpenCC 字典檔，記偏差。轉換放在哪一層（`qim-core` 後處理或另一個 crate）要守 plan §1.3，加依賴先改表。
- OpenCC `fae2a0a` 的字典釘進 `sources.toml`，簡繁資料包有自己的 `LICENSE`（Apache-2.0）、`NOTICE`。
- `Options.simplified`；轉換在排序之後（plan §2.2 的後處理），轉換後重複的候選去重。學習仍記繁體原文。
- 網頁加簡繁開關；量 WebAssembly 包大小與每鍵延遲的變化。

## 驗收

- [x] `ferrous-opencc` 能否編 WebAssembly 的結論在 `## Comments`（不能時偏差已記）。
- [x] 核心測試：簡體模式 `anr` 首選「门口」；轉換後重複候選只留一個；簡體模式下學習記的是繁體。
- [x] Playwright：開簡體後打 `anr`，讀回首選「门口」，空白上屏「门口」。
- [x] WebAssembly 包大小與延遲的前後數字在 `## Comments`。

## Comments

- `ferrous-opencc` 能編 WebAssembly：0.4.0 只開 `t2s-conversion`，在 `wasm32-unknown-unknown` 編得過，含它內嵌的字典約 0.52 MB。但它不能用來做這張票：
  - 字典由它的 build.rs 從 crate 內的 `assets/` 編進程式，和 OpenCC `fae2a0a` 的同名字典 sha256 不同（版本較舊），釘不了 `fae2a0a`。
  - 讀字典的公開 API 只有內建設定與讀檔的 `OpenCC::new(路徑)`；`Dictionary` trait 不公開，`FstDict::from_ocb_bytes` 讀進來也查不了。字典進不了獨立的資料包（brief §5.1），網頁也沒有檔案系統。
  - 所以照 plan §8 的退路自讀字典：`qim-databuild` 的 `opencc.rs` 讀 `fae2a0a` 的字典建 `opencc-hk2s` 資料包，`qim-core` 的 `simplifier.rs` 照 OpenCC 的規則轉換，`qim-core` 不加依賴（plan §1.3 不必改）。偏差記在 progress。`ferrous-opencc` 只留給 `qim-databuild` 的 `s2hk`。
- 轉換照 OpenCC `hk2s.json`（事實與連結在 brief §12.1）。兩處刻意和 OpenCC 預設不同：
  - `@tofu-risk` 的 896 列（TSCharactersExt）不用、保留原字：那些簡體字多數字型顯示不出，等同 OpenCC 不載入 tofu 風險字典。
  - 不處理表意文字描述序列（IDS），候選不會有。
- 和 `ferrous-opencc` 的 hk2s 比對全部基準句 6,630 句，差 8 句：「舖→铺」「谿→溪」等是 `fae2a0a` 字典較新；「癐」「輷」「鋂」是 tofu 字保留原字；「抬」它轉成「擡」是它的舊字典。轉換規則沒有出入（一次性比對，腳本沒進 repo）。
- 實作時定的細節，都寫進 plan §2.1、§4.4：
  - 簡繁資料包不由方案指名，`Engine::with_simplifier` 掛上：各方案都輸出繁體，共用一份；沒掛上時簡體模式照原樣輸出。
  - 轉換的單位是一個候選或一個分段，上屏是各段轉換後接起來，所以看到的就是上屏的。
  - `Session::refresh`：組字中換選項時殼呼叫，立刻重列候選（librime 換選項後也重算組字）。殼只呼叫 `set_options` 時，下一個鍵先依新選項重列再處理。
  - 去重後的候選學到的是排第一的繁體：例如 `乾`、`幹` 都轉成「干」，選「干」學到的一律是排前面的「乾」。符合票的「學習仍記繁體原文」，但可能強化錯的繁體字；要改得等使用者決定（例如簡體模式下不學去重掉的同形字）。
- 先紅：
  - `crates/qim-core/tests/simplifier.rs`、`simplified.rs` 先編不過。
  - 補上 API 後，`simplified.rs` 有 5 條紅在測試資料（`a` 開頭多出前綴候選「日」），改用 `ef` 碼。
  - code-review 後補的 `a_key_after_switching_without_refresh_uses_the_new_option` 紅在 `left: "门口"`、`right: "門口"`，修後轉綠。
- 變異驗證：
  - 學習改記轉換後的簡體，`learning_records_the_traditional_text` 紅。
  - `--verify` 給一份不轉換的簡繁資料包，只有新的「簡體模式 anr 有「门口」」那項紅（`a_simplifier_pack_that_does_not_convert_fails_the_spot_check`）。
- WebAssembly 包大小（`cargo xtask web` 的 release，`web/src/wasm/qim_wasm_bg.wasm`）。簡繁資料包另外 fetch：67,038 位元組，gzip 29,475。

  | | 改前 | 改後 | 差 |
  | --- | ---: | ---: | ---: |
  | 位元組 | 4,627,632 | 4,661,914 | +34,282（+0.7%） |
  | gzip -9 | 1,211,956 | 1,220,306 | +8,350 |

- 每鍵延遲（WebAssembly，Node v26.12.0）：機器同時跑其他 worktree，負載 3–11，`cargo xtask bench` 兩次之間 p95 就差 20–30 µs。所以另寫一次性腳本，用 HEAD 編的改前綁定與改後綁定，在同一個 Node 行程輪流打同一批輸入 3 輪（固定題加四個語料集合，122,435 鍵），單位 µs：

  | 版本 | 全部 p50 | 全部 p95 | 第 20 碼起 p50 | 第 20 碼起 p95 |
  | --- | ---: | ---: | ---: | ---: |
  | 改前 | 34.6 | 46.0 | 37.4 | 47.7 |
  | 改後、繁體 | 35.5 | 47.2 | 38.1 | 48.8 |
  | 改後、簡體 | 53.4 | 80.5 | 66.2 | 87.4 |

  簡體模式每鍵要轉換整串候選（去重要看全部），p95 多約 35 µs，離 50 ms 的預算很遠。
- `cargo xtask bench --check` 前後：
  - 四份資料包中，速成、詞庫、bigram 的 sha256 不變（`f9499e401df5`、`fe79d8a13d48`、`088a93199bd2`）。
  - 各集合（含個人集）的答錯、修正次數、KSPC、跳動、依長度分組全部相同，沒有翻盤，固定題 11/11。基準跑一般模式，不開簡體。
  - bench 自己印的延遲：p95 原生 43.38 → 37.96 µs、WebAssembly 45.88 → 46.54 µs。code-review 前那次曾到 62 µs，是負載雜訊，見上面的對照。
- 亂按一輪用 Playwright MCP 手動操作，沒有找到問題，內容如下：
  - 簡體連打中夾翻頁 `=` `-`、數字選字、全形標點、大寫、`z`、←、Tab、Enter 原樣上屏。
  - 組字中連按簡體開關 5 次，之後 Backspace，候選每次都立刻跟著換，焦點回到輸入框。
  - 開無痕後簡體打 22 碼的長輸入，再 Esc。
  - 沒組字時切換。
- code-review 已處理：
  - 只呼叫 `set_options`、沒呼叫 `refresh` 時，上屏用舊候選：改成下一個鍵先重列，補測試。
  - 「簡體才轉換」的判斷寫了兩次：收成 `Session::convert`，候選型別改名 `ShownCandidate`。
  - `Engine::simplify` 的 `Cow` 沒有用處，改回 `String`。
  - `longest_prefix` 每次配置 `Vec`，改成不配置。
  - 網頁切換改走 `apply`，觸控鍵盤也跟著重畫。
  - 名詞表加「簡體模式」。
- code-review 未處理：
  - `qim-databuild` 每多一份資料包，`Outcome`、`Report`、`PackBytes`、`main.rs` 都要改，sha256 用 4-tuple 取值。這是沿用原本的寫法，等再多一份資料包（票 15 的英文、票 22 的字根提示）時再收成清單。

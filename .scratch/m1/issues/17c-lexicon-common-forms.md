# 17c: 詞庫也用常見寫法

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: brief §4.1（字形用常見寫法）、§5.2；plan §4.2、§4.3；票 17b 的 Comments（常見寫法對照表）；spec〈共同規則〉

## What to build

使用者 2026-11-05 試用時發現：打 `sqvm` 首選是「羣組」，不是「群組」。票 17b 只把語料改成常見寫法；詞庫來源 rime-essay 用 OpenCC 標準繁體字形（「羣組」1394、「羣體」「羣衆」），沒有「群組」這個詞，所以「群組」只能靠單字拼、排在後面。使用者已決定字形用常見寫法（票 17b），詞庫要一致。

- 建詞庫時，把 rime-essay 的詞套上票 17b 的「香港寫法 → 常見寫法」對照；另外找出 rime-essay 的 OpenCC 標準字形和常見寫法不同、但 17b 對照表沒涵蓋的字（例如「羣→群」「衆→眾」，由 OpenCC 的 `TWVariants` / `HKVariants` 反查，只收同碼的），列出清單與每字影響的詞數，記進 Comments。
- 清單裡有拿不準哪個是「常見寫法」的字（例如「裏／裡」「綫／線」這類香港與台灣都常見的），用 AskUserQuestion 列給使用者決定（使用者在場），不要自己定。
- 轉換後兩種寫法撞成同一個詞時合併，詞頻取和；字形轉換只動詞庫，單字碼表照舊兩種寫法都收（不常見的寫法仍可單打或靠學習）。
- `--verify` 加檢查：詞庫裡不再有對照表左邊的字形。
- bench 前後對照；`--update-baseline` 寫理由。

## 驗收

- [ ] 核心或 Playwright：`sqvm` 首選「群組」；對照表其他字各挑一個詞驗首選。
- [ ] Comments 有對照表擴充的清單與影響詞數；使用者決定的字記下結論與日期。
- [ ] `cargo xtask data --verify` 綠燈，含新檢查；建兩次 sha256 相同。
- [ ] bench 前後對照貼在 Comments；`cargo xtask check` 綠燈。

## Comments

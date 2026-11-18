# 按鍵機率分觸控與實體鍵盤兩種，由殼選；實體鍵盤用鍵位先驗

行為由 `crates/qim-core/tests/key.rs`、`neighbor_readings.rs`、`crates/qim-layout/tests/spatial.rs`、`physical.rs`、`web/test/keys.test.ts` 說明；初值與來源在 `SpatialModel::default`、`PhysicalModel::default`。

- **兩種變體，由殼選**：觸控送 `Key::Weighted`、實體鍵盤送 `Key::Physical`，`Weights` 本身只是機率。鄰鍵讀法只收觸控，核心要分得出來；同 LatinIME 只對有座標的觸控事件用鄰鍵資訊，而殼最清楚按鍵從哪裡來。
- **實體鍵盤的中文模式整個當美式鍵盤**（使用者定）：網頁照實體鍵位（`KeyboardEvent.code`）換成美式鍵盤上的字元，換 Dvorak、AZERTY 時中文模式的手感跟美式一樣；英文直通與臨時英文照配列打。
- **實體鍵盤的鄰鍵先驗不拿觸控的 `Page` 幾何算**：實體鍵之間沒有縫、鍵中心距離比到形狀的距離更分得出錯位，而且觸控的 σ 是手指在玻璃上的精度，不是按錯鍵的機率。libime 只取同排左右（`docs/research/ranking-techniques.md` §5.2），這裡照各排錯位取鄰鍵。

## Considered Options

- 鄰鍵先驗只取同排左右（libime）：這裡照各排錯位，F 的鄰鍵除了同排的 D、G，還有上排的 R、T 與下排的 C、V。

## Consequences

- 平台殼接實體鍵盤時再定怎麼把系統的實體鍵位（macOS 的 keyCode、Windows 的掃描碼）交給 `PhysicalModel`。
- 兩組模型的初值都還沒在基準上調過。

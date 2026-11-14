# Gboard 英文鍵盤黑箱實測

回答的問題：Gboard 英文鍵盤的自動修正、退格還原、連按兩下空白、句首大寫、成對符號、建議列、欄位類型怎麼運作（M4 票 19；結果在 `docs/research/gboard-english-2026-11-15.md`）。換 Gboard 主版本時重量。

## 檔案

- `page/index.html`：多行欄與 `text`、`email`、`url`、`number`、`tel`、`inputmode=numeric`、`search`、`password` 欄；`page/p2.html`：`search`、`password`、`autocorrect="off"` 放在頂端，不用捲動。每個 `input` 事件把 `inputType`、`data`、欄位值、游標位置 POST 回 `server.py`。右上的 CLEAR 鈕清空目前的欄位、不搶焦點，鍵盤不收起。
- `server.py`：提供 `page/`，把 POST 來的事件一行一筆記進記錄檔。
- `kb.py`：用 `adb shell input tap` 點 Gboard 的鍵。
- `events.py`：印記錄檔裡不是單一字元上屏的事件，和最後的欄位值（游標寫成 `|`）。

## 跑法

1. 用沒有學習資料的 Gboard：新建 AVD，或 `adb shell pm clear com.google.android.inputmethod.latin` 後重新設為預設輸入法（`adb shell ime set com.google.android.inputmethod.latin/com.android.inputmethod.latin.LatinIME`）。在 Gboard 設定逐項確認是預設值。
2. 開測試頁：

```sh
export ANDROID_SERIAL=emulator-5560   # adb devices 列出的那台
python3 scripts/experiments/gboard-english-blackbox/server.py 18719 <暫存目錄>/events.log &
adb reverse tcp:18719 tcp:18719
adb shell am start -a android.intent.action.VIEW -d http://localhost:18719/ com.android.chrome
```

3. 點欄位叫出鍵盤，每一項先點 CLEAR、清空記錄檔，再打字、讀回：

```sh
: > <暫存目錄>/events.log
python3 scripts/experiments/gboard-english-blackbox/kb.py "my freind "
python3 scripts/experiments/gboard-english-blackbox/events.py <暫存目錄>/events.log
```

`kb.py` 的符號：小寫字母、大寫（Shift 加字母）、空白、`,` `.`、`<` 退格、`^` Shift、`~` Enter、`#` 開 ?123 頁；`kb.py --sym "<鍵>"` 在已打開的 ?123 頁點數字與符號，`A` 回字母頁。建議列與鍵盤用 `adb exec-out screencap -p` 截圖讀。

## 注意

- 鍵位是 1080×2400、420 dpi 模擬器上 Gboard 18.5.10 預設設定（沒有數字列）的截圖量出來的；換裝置、開數字列或換版本要重量 `kb.py` 開頭的座標。
- Gboard 會學打過的字：原樣上屏一次，之後就不再修那個錯字。每項用沒打過的錯字，要比對的先打對照組。
- 聚焦後等約 2 秒再打，太快第一鍵會掉。

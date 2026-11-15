# 英文模式：逐字上屏、建議跟著正在打的串，改字用 `delete_before`

英文模式（brief §4.7，M4 第二期）是觸控英文頁的狀態：核心吃下按鍵、出英文建議，可開自動修正與句首自動大寫。核心原有的英文狀態是英文直通（鍵交還宿主）與臨時英文（中文組字裡的一段），英文模式和它們分開：

- **逐字上屏，沒有組字**：英文模式的字母、數字、標點與空白，核心都吃下（`handled` 為真），當場經 `commit` 上屏，組字一直是空的。建議與修正的對象是「正在打的串」，也就是英文使用者詞已經在用的 `TypedRun`。這是 Gboard 在 Chrome 的做法（M47）：不留組字區，字母逐個上屏，修正時刪掉游標前整個詞再上屏修正字。這樣字母在宿主裡馬上看得到，各平台都做得到（iOS 的 marked text 宿主怎麼顯示沒測，P11、P25）。代價是修正要刪宿主裡的字，所以加 `delete_before`。退格、Enter、方向鍵照英文直通交還宿主：退格刪掉正在打的串的最後一字，建議跟著更新（`TypedRun::backspace` 現在就是這樣）；其他鍵讓這串打完。自動修正要不要接手 Enter 與退格（Gboard 在 Enter 時修，退格還原是子設定，M47）由票 25 定。
- **怎麼進出**：觸控的中英鍵切英文模式，實體鍵盤的單按 Shift 照舊切英文直通，所以桌面的英文仍是直通、沒有建議列（spec〈細節〉）。核心照切換鍵的來源決定，不另加選項：網頁同時有實體鍵盤與觸控鍵盤，也不必設定。在英文模式按任一種切換鍵都回中文。組字中按中英鍵照現在觸控切換的做法，先上屏整個組字再切（同右 Shift 的 `commit_text`）。臨時英文不變：它是中文組字裡的一段，不出英文建議。`Snapshot.ascii` 在英文模式也是真。按鍵的形狀（`Key` 加一種，或 `ToggleAscii` 帶來源）由票 24 定，殼要不要另一個欄位分辨直通與英文模式也由它定。
- **`delete_before`**：`Snapshot` 加 `delete_before: usize`。殼先刪游標前（有選取時是選取前）這麼多個字元，再上屏 `commit`；0 是不刪，也可以只刪不上屏。殼照做，不自己判斷（plan §1.3）。
  - 單位是 Unicode scalar，也就是 Rust 的 `char`、JS `Array.from` 的一項，同 Fcitx5（P28）與 Android 的 `deleteSurroundingTextInCodePoints`（P26）。
  - 核心只刪自己送出去、之後沒作廢的字（正在打的串、剛上屏的字），作廢的條件同前文。殼給的前文不刪：iOS 只給一兩句，Android 可能回 null（P11、P16）。
  - 會刪的都是 ASCII 字元與 BMP 的標點（英文串、空白、句點、成對符號），一個 scalar 就是一個 UTF-16 碼元，也是一個字素。所以 iOS 呼叫 N 次 `deleteBackward()`（P25，單位文件沒寫）、Android 的 `deleteSurroundingText(N, 0)`（P26，LatinIME 還原修正也這樣刪）、網頁的 `setRangeText` 都刪得剛好。
  - 核心送出 `delete_before` 時，把前文與正在打的串的最後 N 字拿掉。這等於 N 次退格，差別是核心知道刪了什麼，前文不作廢。
  - Android 殼用 `beginBatchEdit`／`endBatchEdit` 包住刪與上屏。macOS 只能用 `insertText:replacementRange:`，宿主不支援時會被忽略（P27）。桌面沒有英文模式，只有智慧標點可能用到，桌面要不要用由票 27 定。
- **英文的切詞**：正在打的串沿用英文使用者詞的串字元，也就是英文字母、數字與 `. _ % + - @ '`。email、網址整串是一個詞：建議列打到 `ivan.t` 還補得出 `ivan.test@example.com`，學與補完也是同一串。LatinIME 的分隔字元含 `. _ + /`，`@` 不算分隔（`donottranslate-config-spacing-and-punctuations.xml`），所以 email 會被 `.` 切開。詞頭詞尾的 `.` 與 `'` 不算進詞（同 `to_learn`），`teh.` 之後打空白，修的是 `teh`。自動修正只修去掉頭尾後全是字母的詞，中間可以有 `'` 與 `-`（LatinIME 的 word connectors）；含數字、`@`、中間有 `.` 的不修，同 Gboard（M47）。修正發生在串打完時，也就是打了空白、逗號這類非串字元；`.` 不算，所以 `teh.com` 不會在打 `.` 那一下被修。這點和 Gboard 不同，Gboard 在 `.` 那一下就修（M47）。細節照票 25。
- **預測不含下一個詞**：英文建議列只補完、修正正在打的串，空白之後不出下一個詞。Gboard 與 LatinIME 都有下一個詞（M47、LatinIME 的 bigram 詞典），但它要三樣東西：英文 bigram 資料包、學英文的個人 bigram、空白之後的建議列。這些和 M5 重做候選列一起做較省；M4 的建議列只是沿用候選列的最小版（spec〈範圍〉）。
- **`Options` 的三個新欄位**（brief §7），都只在英文模式作用：
  - `autocorrect`：自動修正，預設關。
  - `auto_capitalize`：句首自動大寫，預設關。
  - `english_corrections`：英文容錯，建議列除了前綴補完也列拼錯的修正，預設開。同 LatinIME 的「Show correction suggestions」。關掉時只補完，自動修正也沒有對象。
  - 英文容錯和中文的鄰鍵讀法（`neighbor_readings`）分成兩個開關：中文的會佔候選列第 2 位、打亂熟手的習慣，英文的只出在英文模式的建議列。brief §7 的「容錯」改成兩條。

## Considered Options

- **英文的字留在組字**（LatinIME 的 composing text）：自動修正不必刪字，空白時直接上屏修正字。但打的時候字母不在宿主裡（網頁、iOS 要另畫），Gboard 在 Chrome 也不這樣做，還原時一樣要刪字。
- **`Options` 加旗標，決定英文是直通還是英文模式**（Fcitx5 鍵盤引擎的單字提示是選項）：殼要依自己的形態設定，網頁同時有兩種鍵盤時不好設。照切換鍵的來源就不必設。
- **`delete_before` 用 UTF-16 碼元**（LatinIME 的 Java char）：Android、macOS、網頁可以直接用，但 Rust 核心要另外換算，Fcitx5 與 iOS 也不是這個單位。核心只刪單一 scalar 的字，兩種單位數目相同，所以選 Rust 自然的 `char`。
- **給要刪的文字，不給字數**：殼能核對，也能各自換算單位，但每個殼都要比字串，而核心刪的本來就是自己送出去的字。字數同 LatinIME、Fcitx5、Wayland 的介面。
- **照 LatinIME 以 `.` 切詞**：句尾打 `.` 那一下就能修，但 email 與網址被切成幾段，建議列補不出學到的 email。
- **M4 就做下一個詞**：英文模式較接近 Gboard，但要多做英文 bigram 資料包與學習，空白之後的建議列也會在 M5 重做時再改一次。

## Consequences

- 殼要會「先刪 N 字再上屏」：網頁在 M4 票 21 接上（`host.ts` 的 `insert`），各平台殼照 P25–P27 做。macOS 宿主不支援範圍取代時會少刪，所以桌面要用之前先確認。
- 核心的前文與正在打的串要跟著 `delete_before` 改，否則下一個字會接錯。
- 英文模式的字逐個上屏，宿主的 undo、拼字檢查與自動完成看到的是一個個字母（Gboard 在 Chrome 也是這樣）。
- 成對符號「插入一對、游標放中間」不靠 `delete_before`：Android 一次呼叫放不到中間（P26）。Gboard 不自動配對（M48），要不要做、要不要再加欄位由票 27 定。
- 下一個詞留到 M5 重做候選列時評估，記在 roadmap〈M5 觸控鍵盤與設定〉；票 22 的資料包不必帶英文 bigram。

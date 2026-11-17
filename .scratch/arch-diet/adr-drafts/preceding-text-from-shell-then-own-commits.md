# 前文以殼給的為準，沒給時用自己上屏的字；以標點結尾時接句首

> 草稿（arch-diet 票 02 從 plan〈核心〉〈介面〉〈上下文〉與〈排序：同一把尺〉搬來），使用者確認後由票 06 編號寫進 `docs/adr/`。

組字開頭的字要接「前文」（游標前的文字）算 bigram、個人 bigram 與數字後半形。前文有兩個來源：殼給的（`Session::set_context`）與核心自己送出去的字。QIM 合起來用：殼給的優先，每次給都整份取代；沒給時（宿主不提供、iOS 的 `documentContextBeforeInput` 是空的，`docs/research/facts-platforms.md`）靠自己記的這個輸入框上屏過的字。殼給的前文只有一兩句時也不拿記著的較長歷史補長度：bigram 只看最後一字，補長度只對停鍵層與雲端的模型有用，而量過送 200 字不比 50 字好（`docs/research/model-ladder.md`），雲端也不做。交還宿主的退格、Enter、方向鍵這類鍵改了游標前的字或移走游標，記的前文就作廢，等殼再給。

前文以字結尾時接那個字；以標點、空白或符號結尾，或是空的時接句首（`CLAUSE_START`）；結尾的字沒有統計、不知道前文、無痕時用 unigram。bigram 只在句內訓練，接不過標點。

## Considered Options

- 只用自己的 commit history（librime：`Translator::GetPrecedingText` 取 `commit_history().latest_text()`，最多 20 筆，退格、Enter 清掉；數字後半形 `punctuator.cc` 的 `is_after_number` 也看它）：不必殼配合，但使用者點了別處、宿主改了字時接錯。QIM 和它不同在方向鍵也作廢。
- 只用宿主給的（AOSP LatinIME：`RichInputConnection.reloadTextCache`，宿主回 null 就當沒有前文）：宿主不給時就沒有前文；bigram 只要最後一字，自己上屏的字就是游標前的字，所以不採用。
- fcitx5 的標點模組平時靠自己記的狀態、宿主有 SurroundingText 時在聚焦後用游標前一字校正；QIM 把兩種來源合起來，殼給的優先。
- 前文以標點結尾時略過標點、接上一句的最後一字：「有前文」的題給前文反而比不給答錯多 1–5%、修正多 0.3–4%（四個語料都是）；改接句首後給前文較好。規則先在 test 的「有前文」集合上看出，之後在 dev 切分重比三種規則，結論相同（`scripts/experiments/context-rule/`）。「有前文」的題前文都以標點結尾，量到的是句首的增益；「接前文的最後一字」只在上屏一段後重打剩下的字時走到，只影響修正次數。

來源：librime `c95c0d2` 的 [`script_translator.cc` L325–329](https://github.com/rime/librime/blob/c95c0d2390c827bf44e5eb8989704ba723ec0d0c/src/rime/gear/script_translator.cc#L325-L329)、[`commit_history.cc` L14–26](https://github.com/rime/librime/blob/c95c0d2390c827bf44e5eb8989704ba723ec0d0c/src/rime/commit_history.cc#L14-L26)、[`punctuator.cc` L78–86](https://github.com/rime/librime/blob/c95c0d2390c827bf44e5eb8989704ba723ec0d0c/src/rime/gear/punctuator.cc#L78-L86)；fcitx5-chinese-addons `e8d930f` 的 [`punctuation.cpp` L217–247](https://github.com/fcitx/fcitx5-chinese-addons/blob/e8d930f402e359cfd76a101ee3e46815d1331d83/modules/punctuation/punctuation.cpp#L217-L247)；AOSP LatinIME `127336e` 的 [`RichInputConnection.java`](https://android.googlesource.com/platform/packages/inputmethods/LatinIME/+/127336e9f29d69607eab55982324b210279ae8c5/java/src/com/android/inputmethod/latin/RichInputConnection.java)。

## Consequences

- 殼要在聚焦、移游標、上屏後給前文，失焦而給不出時呼叫 `clear_context`（`ARCHITECTURE.md`〈核心對殼〉）；沒給時退回這個輸入框上屏過的字。
- 無痕時核心一個字都不記、殼給的也不收，關掉無痕後第一次打字不含無痕期間的字。數字後半形的布林不含文字，無痕時照記；但無痕時連殼給的前文是不是數字結尾也不看，所以無痕時用滑鼠把游標移到中文後面，核心仍當是數字後。

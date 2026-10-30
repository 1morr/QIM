# 自己寫核心，不建在 librime 上

QIM 自己寫跨平台核心，參考 libime 與 librime 的做法但不依賴它們。研究發現使用者回報的三個 Rime 速成連打案例，根因在 librime 的規則與 rime-quick 的資料設計：有精確詞條就不造句、`derive` 讓倉頡碼滲進速成碼且不降權（`docs/research/rime.md` §4）。要修就得改 librime 的 translator 或寫外掛，而 QIM 要的「長詞條與造句用同一把尺比分數」、自訂學習公式、LLM 非同步重排介面，都得在它的 YAML 方案與編譯流程裡繞。速成每字最多兩碼，資料結構簡單，排序做法（lattice、Viterbi、詞頻、bigram、使用者學習）有 libime 與 librime 可參考。

## Considered Options

- 建在 librime 上（Rime 系各前端的做法）：成熟、五平台都有前端先例，但三個案例的修法要動它的核心規則，碼表與詞庫又綁 LGPL 的 Rime 資料。

## Consequences

- 使用者學習、詞庫管理、多輸入方案都要自己做。
- 不能直接沿用 Rime 生態的方案檔；要支援時另做匯入。

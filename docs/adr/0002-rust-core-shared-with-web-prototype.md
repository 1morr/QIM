# 核心用 Rust，網頁 prototype 共用同一個核心

核心用 Rust 寫，編成 WebAssembly 給網頁 prototype，用 uniffi 產生 Swift / Kotlin 綁定給 iOS、macOS、Android，用 C ABI 給 Windows TSF 與 Fcitx5。網頁 prototype 因此直接驗證真實核心，只有網頁 UI 是之後會丟的；佈局編輯器是例外（ADR 0004）。青簡已用 Rust 核心加各平台殼證明 macOS、Windows、Android 可行（`docs/research/platform-frameworks.md` §1）。

## Considered Options

- C++（Rime、libime 的路線）：生態成熟，但記憶體安全靠紀律，編 WebAssembly 與產生 Swift / Kotlin 綁定較麻煩。
- TypeScript：prototype 最快，但放不進手機鍵盤與 Windows，最後必須重寫，prototype 驗證的引擎也就不是真實核心。

## Consequences

- iOS 沒有 Rust 鍵盤殼的先例，記憶體上限要在平台可行性實驗（roadmap〈M2 平台可行性實驗〉）先量。

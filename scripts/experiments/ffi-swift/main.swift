// 經 uniffi 產生的 Swift 綁定（`cargo xtask ffi`）用核心：`demo` 打 anr 讀回首選、空白上屏；
// `latency` 量每鍵延遲，量法同 `qim-bench`（`docs/development.md`〈每鍵延遲〉）。用法見同目錄 README.md。

import Foundation

let usage = "用法：qim-swift demo|latency <資料包路徑>…（速成方案必給，詞庫、語言模型、英文選給）；latency 從 stdin 一行讀一個輸入碼"

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data("\(message)\n".utf8))
    exit(2)
}

/// 資料包依檔頭魔數分辨（`crates/qim-core/src/pack.rs`），同 `web/bench/latency.ts`。
func loadSession(_ paths: ArraySlice<String>) throws -> Session {
    var packs: [String: Data] = [:]
    for path in paths {
        guard let data = FileManager.default.contents(atPath: path) else { fail("讀不到 \(path)") }
        let magic = String(decoding: data.prefix(4), as: UTF8.self)
        guard ["QIMP", "QIML", "QIMB", "QIME"].contains(magic), packs[magic] == nil else {
            fail("\(path)：不認得的資料包或同種給了兩份。\(usage)")
        }
        packs[magic] = data
    }
    guard let scheme = packs["QIMP"] else { fail(usage) }
    let engine = try Engine(
        scheme: SchemePack(bytes: scheme), lexicon: packs["QIML"].map(LexiconPack.init),
        languageModel: packs["QIMB"].map(LanguageModelPack.init), simplifier: nil,
        english: packs["QIME"].map(EnglishPack.init))
    return Session(engine: engine)
}

func demo(_ session: Session) throws {
    var snapshot: Snapshot? = nil
    for c in "anr" {
        snapshot = try press(session, c)
        let first = snapshot!.candidates.first?.text ?? ""
        print("按 \(c)：輸入碼 \(snapshot!.input)，首選 \(first)")
    }
    let first = snapshot!.candidates.first?.text
    let committed = try session.press(kind: "space", value: nil).commit
    print("按空白：上屏 \(committed ?? "（沒有）")")
    guard first == "門口", committed == "門口" else { fail("期望首選與上屏都是「門口」") }
}

/// 從標準輸入讀輸入，一行一個；印出與基準同欄的表，微秒。
func latency(_ session: Session) throws {
    let inputs = AnyIterator { readLine() }
    print(try Latency.measure(session, inputs: IteratorSequence(inputs)).table("Swift（uniffi）").joined(separator: "\n"))
}

let args = CommandLine.arguments
guard args.count >= 3 else { fail(usage) }
do {
    let session = try loadSession(args[2...])
    switch args[1] {
    case "demo": try demo(session)
    case "latency": try latency(session)
    default: fail(usage)
    }
} catch {
    fail("\(error)")
}

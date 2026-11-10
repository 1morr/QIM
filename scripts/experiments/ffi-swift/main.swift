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
    return try Session(
        schemePack: scheme, lexicon: packs["QIML"], languageModel: packs["QIMB"],
        simplifier: nil, english: packs["QIME"])
}

/// 輸入碼的字元要按的鍵：分隔記號是標點，其餘是字母（`Key::for_input`）。
func press(_ session: Session, _ c: Character) throws -> Snapshot {
    try session.press(kind: c == "'" ? "punct" : "letter", value: String(c))
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

/// 照網頁讀出畫組字與候選要的欄位，免得編譯器省掉；回傳分段數。
func read(_ snapshot: Snapshot) -> Int {
    var size = snapshot.input.utf8.count + snapshot.candidates.count + Int(snapshot.focus)
    for segment in snapshot.segments { size += segment.radicals.utf8.count }
    if size < 0 { print(size) }
    return snapshot.segments.count
}

func ms(since start: UInt64) -> Double {
    Double(clock_gettime_nsec_np(CLOCK_UPTIME_RAW) - start) / 1_000_000
}

/// 排序後第 ⌈p/100 × n⌉ 個（nearest-rank），同 `qim_bench::percentile`。
func percentile(_ values: [Double], _ p: Int) -> Double? {
    guard !values.isEmpty else { return nil }
    let sorted = values.sorted()
    return sorted[max(1, (sorted.count * p + 99) / 100) - 1]
}

func row(_ group: String, inputs: Int, _ values: [Double]) -> String {
    let us = { (p: Int) in percentile(values, p).map { String(format: "%.2f", $0 * 1000) } ?? "—" }
    return "| Swift（uniffi） | \(group) | \(inputs) | \(values.count) | \(us(50)) | \(us(95)) |"
}

/// 同 `qim-bench` 的每鍵延遲：每題從空的組字逐鍵打；打完有兩段以上時按 ← 到第一段（不計時）再選它的
/// 首選，計那一次。印出與基準同欄的表，微秒。
func latency(_ session: Session) throws {
    var perKey: [[Double]] = []
    var selects: [Double] = []
    while let input = readLine(), !input.isEmpty {
        var times: [Double] = []
        var segments = 0
        for c in input {
            let start = clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
            segments = read(try press(session, c))
            times.append(ms(since: start))
        }
        perKey.append(times)
        if segments >= 2 {
            for _ in 0..<segments { _ = try session.press(kind: "left", value: nil) }
            let start = clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
            _ = read(session.select(index: 0))
            selects.append(ms(since: start))
        }
        _ = try session.press(kind: "escape", value: nil)
    }
    let long = perKey.filter { $0.count >= 20 }
    print("| 環境 | 輸入 | 輸入數 | 鍵數 | p50 | p95 |")
    print("| --- | --- | ---: | ---: | ---: | ---: |")
    print(row("全部", inputs: perKey.count, perKey.flatMap { $0 }))
    print(row("第 20 碼起", inputs: long.count, long.flatMap { $0.dropFirst(19) }))
    print(row("選開頭一段", inputs: selects.count, selects))
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

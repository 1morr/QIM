import Foundation

// 每鍵延遲的量法，Mac 的 `qim-swift latency`（main.swift）與 iPhone 鍵盤擴充（ios-keyboard 的
// CoreProbe.swift）共用，同 `qim-bench` 的每鍵延遲（docs/development.md〈每鍵延遲〉）。

/// 輸入碼的字元要按的鍵：分隔記號是標點，其餘是字母（`Key::for_input`）。
func press(_ session: Session, _ c: Character) throws -> Snapshot {
    try session.press(kind: c == "'" ? "punct" : "letter", value: String(c))
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

/// 一批輸入的每鍵延遲，毫秒。
struct Latency: Sendable {
    /// 每個輸入逐鍵的時間。
    var perKey: [[Double]] = []
    /// 打完有兩段以上時，選開頭一段那一鍵的時間。
    var selects: [Double] = []

    /// 每題從空的組字逐鍵打；打完有兩段以上時按 ← 到第一段（不計時）再選它的首選，計那一次；
    /// 最後按 Esc 清掉組字，所以不上屏、不學習。
    static func measure(_ session: Session, inputs: some Sequence<String>) throws -> Latency {
        var result = Latency()
        for input in inputs where !input.isEmpty {
            var times: [Double] = []
            var segments = 0
            for c in input {
                let start = clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
                segments = read(try press(session, c))
                times.append(ms(since: start))
            }
            result.perKey.append(times)
            if segments >= 2 {
                for _ in 0..<segments { _ = try session.press(kind: "left", value: nil) }
                let start = clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
                _ = read(session.select(index: 0))
                result.selects.append(ms(since: start))
            }
            _ = try session.press(kind: "escape", value: nil)
        }
        return result
    }

    /// 與基準同欄的表（微秒），`environment` 是第一欄。
    func table(_ environment: String) -> [String] {
        let long = perKey.filter { $0.count >= 20 }
        func row(_ group: String, inputs: Int, _ values: [Double]) -> String {
            let us = { (p: Int) in percentile(values, p).map { String(format: "%.2f", $0 * 1000) } ?? "—" }
            return "| \(environment) | \(group) | \(inputs) | \(values.count) | \(us(50)) | \(us(95)) |"
        }
        return [
            "| 環境 | 輸入 | 輸入數 | 鍵數 | p50 | p95 |",
            "| --- | --- | ---: | ---: | ---: | ---: |",
            row("全部", inputs: perKey.count, perKey.flatMap { $0 }),
            row("第 20 碼起", inputs: long.count, long.flatMap { $0.dropFirst(19) }),
            row("選開頭一段", inputs: selects.count, selects),
        ]
    }
}

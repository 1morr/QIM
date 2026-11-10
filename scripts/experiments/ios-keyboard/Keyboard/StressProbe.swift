import Darwin
import Foundation

/// 票 09：逐步佔用確實寫過的記憶體，直到系統終止擴充。
///
/// 每塊用 `arc4random_buf` 填滿：每頁都被寫過（算進 footprint），內容是亂數（壓縮不了，免得被系統的
/// 記憶體壓縮器壓小）。每步的 footprint 寫進擴充自己容器裡的 `stress.log` 並 fsync，程序被終止後，
/// 下次載入時讀回上一輪最後一步（README〈記憶體上限（票 09）〉）。
@MainActor
enum StressProbe {
    /// 連續加壓每步的大小與間隔：終止點落在最後一筆紀錄之後 1 塊以內。
    static let stepMB = 1
    static let intervalMs: UInt64 = 40
    /// 手按「+10MB」一次佔的大小。
    static let manualMB = 10

    private static var chunks: [UnsafeMutableRawPointer] = []
    private static var running = false
    private static var step = 0
    private static var file: FileHandle?
    private(set) static var allocatedMB = 0
    private(set) static var lastLine = ""
    static var onUpdate: (() -> Void)?

    static var active: Bool { !chunks.isEmpty || running }

    /// 上一輪（上一個程序）的摘要：載入時讀一次。
    static let previousLines: [String] = summarizePrevious()

    static var displayLines: [String] {
        active ? ["stress " + lastLine] : []
    }

    /// 手按一次：多佔一塊並記一筆。
    static func addManual() {
        allocate(manualMB, mode: "manual")
        onUpdate?()
    }

    /// 每 `intervalMs` 多佔 `stepMB`，直到程序被終止。
    static func runContinuous() {
        guard !running else { return }
        running = true
        Task { @MainActor in
            while true {
                allocate(stepMB, mode: "auto")
                onUpdate?()
                try? await Task.sleep(nanoseconds: intervalMs * 1_000_000)
            }
        }
    }

    /// 系統送來的記憶體警告：記下當時的 footprint，看終止前有沒有預警。
    static func memoryWarning() {
        record("warning allocated=\(allocatedMB)MB " + Probe.memoryLine("at"))
        onUpdate?()
    }

    private static func allocate(_ mb: Int, mode: String) {
        let size = mb * 1_048_576
        guard let chunk = malloc(size) else {
            record("malloc-failed allocated=\(allocatedMB)MB")
            return
        }
        arc4random_buf(chunk, size)
        chunks.append(chunk)
        allocatedMB += mb
        step += 1
        record("step=\(step) \(mode) allocated=\(allocatedMB)MB " + Probe.memoryLine("now"))
    }

    /// 寫一行到 `stress.log` 並 fsync，同時寫系統 log。
    private static func record(_ line: String) {
        lastLine = line
        Probe.log.notice("stress \(line, privacy: .public)")
        guard let handle = openFile() else { return }
        handle.write(Data((line + "\n").utf8))
        try? handle.synchronize()
    }

    /// 第一次寫時開檔並先寫這一輪的開頭（pid 與當時的 footprint）。
    private static func openFile() -> FileHandle? {
        if let file { return file }
        guard let url = logURL else { return nil }
        if !FileManager.default.fileExists(atPath: url.path) {
            FileManager.default.createFile(atPath: url.path, contents: nil)
        }
        guard let handle = try? FileHandle(forWritingTo: url) else { return nil }
        _ = try? handle.seekToEnd()
        file = handle
        let header = "run pid=\(getpid()) \(Probe.timestamp()) " + Probe.memoryLine("start") + "\n"
        handle.write(Data(header.utf8))
        return handle
    }

    nonisolated static var logURL: URL? {
        FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first?
            .appendingPathComponent("stress.log")
    }

    /// 上一輪的開頭、最後一步與警告數；檔案寫不進時說明。
    nonisolated private static func summarizePrevious() -> [String] {
        guard let url = logURL else { return ["prev no-container"] }
        guard let text = try? String(contentsOf: url, encoding: .utf8) else { return ["prev none"] }
        let lines = text.split(separator: "\n").map(String.init)
        guard let start = lines.lastIndex(where: { $0.hasPrefix("run ") }) else { return ["prev none"] }
        let run = lines[start...]
        let warnings = run.filter { $0.hasPrefix("warning") }
        var out = ["prev " + run.first!]
        if let firstWarning = warnings.first { out.append("prev warnings=\(warnings.count) first: " + firstWarning) }
        if let last = run.last(where: { $0.hasPrefix("step=") }) { out.append("prev last " + last) }
        return out
    }
}

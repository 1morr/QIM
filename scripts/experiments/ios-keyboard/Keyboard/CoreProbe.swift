import Foundation

/// 鍵盤擴充接上核心（M2 票 08）：從擴充的 bundle 逐份載入資料包、建一個 `Engine` 與一個 `Session`，
/// 每步記 footprint；打字、量每鍵延遲。
///
/// 狀態放在靜態變數：iphone-use 每次點擊都讓 view controller 重建（README〈用 iphone-use 操作時〉），
/// 放在 view controller 裡就會每次重建都多一份 `Engine`，記憶體翻倍、數字失真（票 07 轉入的提醒）。
@MainActor
enum CoreProbe {
    private(set) static var session: Session?
    private(set) static var busy = false
    private(set) static var snapshot: Snapshot?
    /// 載入各步與延遲的結果，顯示在鍵盤上方。
    private(set) static var lines: [String] = []
    private static var typingLine = ""
    private static var keyTimes: [String] = []
    private static var latencyLines: [String] = []
    static var onUpdate: (() -> Void)?

    static var active: Bool { session != nil || busy }

    /// 螢幕放不下表頭與太多行：每鍵時間併成一行（最近 4 鍵），延遲表略過表頭兩行；完整的都在 log。
    static var displayLines: [String] {
        let keys = keyTimes.isEmpty ? [] : ["keys " + keyTimes.suffix(4).joined(separator: " ")]
        return lines + [typingLine] + keys + latencyLines.filter { !$0.hasPrefix("| 環境") && !$0.hasPrefix("| ---") }
    }

    /// 組字與候選，候選列那一行。
    static var candidateLine: String {
        guard let snapshot else { return session == nil ? "（按「載入」）" : "" }
        let candidates = snapshot.candidates.prefix(6).enumerated().map { "\($0.offset + 1).\($0.element.text)" }
        return ([snapshot.input.isEmpty ? "—" : snapshot.input] + candidates).joined(separator: " ")
    }

    static func load() {
        guard !active else { return }
        busy = true
        lines = ["core loading…"]
        onUpdate?()
        Task.detached(priority: .userInitiated) {
            let (session, lines) = loadSession()
            await MainActor.run {
                self.session = session
                self.lines = lines
                busy = false
                log(lines)
                onUpdate?()
            }
        }
    }

    /// 按一個鍵，記下核心呼叫的時間與之後的 footprint；沒載入或正在量延遲時不按。
    static func press(_ kind: String, _ value: String?) -> (snapshot: Snapshot, coreMs: Double)? {
        guard let session, !busy else { return nil }
        let start = clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
        guard let snapshot = try? session.press(kind: kind, value: value) else { return nil }
        let coreMs = ms(since: start)
        self.snapshot = snapshot
        typingLine = Probe.memoryLine("typing")
        log([typingLine])
        return (snapshot, coreMs)
    }

    /// 一次手按的鍵：核心呼叫與含畫面更新的時間（後者見 KeyboardViewController.pressCore），毫秒。
    static func recordKey(_ label: String, coreMs: Double, screenMs: Double) {
        keyTimes.append(String(format: "%@=%.3f/%.1f", label, coreMs, screenMs))
        log([String(format: "key %@ core=%.3fms screen=%.2fms", label, coreMs, screenMs)])
        onUpdate?()
    }

    /// 用同一個 `Session` 跑 ffi-swift 那批輸入（bundle 的 `latency-inputs.txt`），每題結束按 Esc。
    static func runLatency() {
        guard let session, !busy else { return }
        busy = true
        latencyLines = ["latency running…"]
        onUpdate?()
        Task.detached(priority: .userInitiated) {
            let lines = measureLatency(session)
            await MainActor.run {
                latencyLines = lines
                busy = false
                log(lines)
                onUpdate?()
            }
        }
    }

    private static func log(_ lines: [String]) {
        for line in lines { Probe.log.notice("core \(line, privacy: .public)") }
    }

    /// 每份資料包「讀檔 → 解開 → 丟掉檔案位元組」前後的 footprint 差，再建 `Engine` 與 `Session`。
    /// 資料包依票 04 的順序；字根提示只給殼顯示、佈局 `qim-ffi` 還沒有介面，都不放（README〈接上核心〉）。
    nonisolated private static func loadSession() -> (Session?, [String]) {
        var lines = [Probe.memoryLine("core-before")]
        let loadStart = clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
        func step<T>(_ name: String, _ body: () throws -> (T, Int?)) throws -> T {
            let before = Probe.footprintMB() ?? 0
            let start = clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
            let (value, file) = try body()
            let elapsed = ms(since: start)
            let after = Probe.footprintMB() ?? 0
            let size = file.map { String(format: " file=%.2fMB", Double($0) / 1_048_576) } ?? ""
            lines.append(String(format: "%@%@ +%.2fMB → %.2fMB %.0fms", name, size, after - before, after, elapsed))
            return value
        }
        func pack<T>(_ name: String, _ make: (Data) throws -> T) throws -> T {
            try step(name) {
                guard let url = Bundle.main.url(forResource: name, withExtension: "pack") else {
                    throw CocoaError(.fileNoSuchFile, userInfo: [NSFilePathErrorKey: "\(name).pack"])
                }
                let data = try Data(contentsOf: url)
                return (try make(data), data.count)
            }
        }
        do {
            let scheme = try pack("quick", SchemePack.init)
            let lexicon = try pack("quick-lexicon", LexiconPack.init)
            let languageModel = try pack("zh-hant-bigram", LanguageModelPack.init)
            let simplifier = try pack("opencc-hk2s", SimplifierPack.init)
            let english = try pack("scowl-en-us", EnglishPack.init)
            let engine = try step("engine") {
                (try Engine(scheme: scheme, lexicon: lexicon, languageModel: languageModel,
                            simplifier: simplifier, english: english), nil as Int?)
            }
            let session = try step("session") { (Session(engine: engine), nil as Int?) }
            lines.append(String(format: "core-loaded %.0fms ", ms(since: loadStart)) + Probe.memoryLine("footprint"))
            return (session, lines)
        } catch {
            lines.append("core-failed \(error)")
            return (nil, lines)
        }
    }

    nonisolated private static func measureLatency(_ session: Session) -> [String] {
        guard let url = Bundle.main.url(forResource: "latency-inputs", withExtension: "txt"),
              let text = try? String(contentsOf: url, encoding: .utf8)
        else { return ["latency-failed no latency-inputs.txt"] }
        let start = clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
        do {
            let result = try Latency.measure(session, inputs: text.split(separator: "\n").map(String.init))
            return result.table("iPhone")
                + [String(format: "latency done %.1fs ", ms(since: start) / 1000) + Probe.memoryLine("footprint")]
        } catch {
            return ["latency-failed \(error)"]
        }
    }
}

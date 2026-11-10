import Foundation
import FoundationModels

/// 在鍵盤擴充裡試 Foundation Models（M2 票 10）：可用性、首次與之後每次回應的延遲、擴充的 footprint。
@MainActor
enum ModelProbe {
    /// plan §7.8 的提示格式：上下文 + 輸入碼 + 前 10 個候選，只回序號。正解是 3（門口）。
    static let instructions = "你是中文輸入法的候選排序器。根據上下文，從候選中選出最可能接在上下文後面的一個，只回答它的序號（阿拉伯數字），不要其他文字。"
    static let prompt = """
        上下文：我已經到咗你屋企樓下，喺
        輸入碼：anr
        候選：1.問 2.閂 3.門口 4.閃 5.悶 6.聞 7.閒 8.開 9.間 10.閱
        序號：
        """
    static let options = GenerationOptions(samplingMode: .greedy, maximumResponseTokens: 4)

    /// FM 鍵每按一次送幾個請求；第 1 個是停頓後的第一個請求。
    static let runs = 6
    private static var requestsInProcess = 0

    static func availabilityLines() -> [String] {
        let model = SystemLanguageModel.default
        var variant = "n/a"
        if #available(iOS 27.0, *) { variant = "\(model.variant)" }
        return [
            "fm-availability=\(model.availability) variant=\(variant)",
            "fm-locale=\(Locale.current.identifier) supported=\(model.supportsLocale())"
                + " zh-Hant-HK=\(model.supportsLocale(Locale(identifier: "zh-Hant-HK")))",
        ]
    }

    /// 支援的語言清單：太長，只寫 log。
    static func languagesLine() -> String {
        let ids = SystemLanguageModel.default.supportedLanguages.map { $0.maximalIdentifier }.sorted()
        return "fm-languages=" + ids.joined(separator: ",")
    }

    /// 鍵盤上要顯示的行：跑過就是最近一次的結果，否則是可用性。
    static var displayLines: [String] { latest.isEmpty ? availabilityLines() : latest }

    /// 最近一次的結果。iphone-use 每次點擊後鍵盤畫面會重建（新的 view controller、同一個程序），
    /// 所以結果放在這裡，由當下的畫面讀。
    private(set) static var latest: [String] = []
    /// 結果更新時呼叫；由當下顯示的 view controller 設定。
    static var onUpdate: (() -> Void)?
    private static var isRunning = false

    /// 依序跑 `runs` 次，每次新開 session（同 IME 每次停鍵送一個獨立請求）。
    static func run() async {
        guard !isRunning else { return }
        isRunning = true
        defer { isRunning = false }
        var lines = availabilityLines()
        lines.append(Probe.memoryLine("fm-before"))
        publish(lines)
        guard SystemLanguageModel.default.isAvailable else { return }
        lines.append("fm-context-size=\(SystemLanguageModel.default.contextSize)")
        var latencies: [String] = []
        for _ in 0..<runs {
            latencies.append("#\(requestsInProcess + 1)=\(await timedRequest(prewarm: false))")
            Probe.log.notice("keyboard fm \(latencies.last!, privacy: .public)")
            publish(lines + ["fm " + latencies.joined(separator: " ")])
        }
        lines.append("fm " + latencies.joined(separator: " "))
        lines.append(Probe.memoryLine("fm-after"))
        publish(lines)
    }

    /// 閒置多久後的第一個請求會變慢：按一次鍵，同一個程序裡依序隔 `gaps` 秒各送一個請求，中途不用再按。
    /// 第二輪在送出前 1 秒呼叫 `prewarm()`（官方說至少要提前 1 秒）。
    static let gaps: [Int] = [0, 2, 5, 10, 20, 40]
    static let prewarmGaps: [Int] = [10, 20, 40]

    static func runGaps() async {
        guard !isRunning, SystemLanguageModel.default.isAvailable else { return }
        isRunning = true
        defer { isRunning = false }
        var lines = [Probe.memoryLine("gap-before")]
        var direct: [String] = []
        for gap in gaps {
            try? await Task.sleep(for: .seconds(gap))
            direct.append("\(gap)s=\(await timedRequest(prewarm: false))")
            Probe.log.notice("keyboard gap \(direct.last!, privacy: .public)")
            publish(lines + ["gap " + direct.joined(separator: " ")])
        }
        lines.append("gap " + direct.joined(separator: " "))
        var prewarmed: [String] = []
        for gap in prewarmGaps {
            try? await Task.sleep(for: .seconds(gap - 1))
            prewarmed.append("\(gap)s=\(await timedRequest(prewarm: true))")
            Probe.log.notice("keyboard prewarm \(prewarmed.last!, privacy: .public)")
            publish(lines + ["prewarm " + prewarmed.joined(separator: " ")])
        }
        lines.append("prewarm " + prewarmed.joined(separator: " "))
        lines.append(Probe.memoryLine("gap-after"))
        publish(lines)
    }

    /// 新開 session 送一次固定提示，回「毫秒'回答'」；`prewarm` 時先 prewarm、等 1 秒才計時送出。
    private static func timedRequest(prewarm: Bool) async -> String {
        requestsInProcess += 1
        let session = LanguageModelSession(instructions: instructions)
        if prewarm {
            session.prewarm()
            try? await Task.sleep(for: .seconds(1))
        }
        let start = ContinuousClock.now
        do {
            let response = try await session.respond(to: prompt, options: options)
            return "\((ContinuousClock.now - start).milliseconds)ms'\(response.content)'"
        } catch {
            return "error@\((ContinuousClock.now - start).milliseconds)ms:\(error)"
        }
    }

    private static func publish(_ lines: [String]) {
        latest = lines
        onUpdate?()
    }
}

private extension Duration {
    var milliseconds: Int {
        let (s, atto) = components
        return Int(s) * 1000 + Int(atto / 1_000_000_000_000_000)
    }
}

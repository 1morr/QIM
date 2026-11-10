import Foundation

/// 在鍵盤擴充裡跑小型神經模型的開銷量測（M2 票 11）。量測本身在 Rust 靜態庫 `neural-cost`，
/// 這裡只挑模型檔、在背景執行緒呼叫 `nc_run`、把結果顯示在鍵盤上並寫 log。
@MainActor
enum NeuralProbe {
    /// 鍵名與 `models/` 裡的檔名（`neural-cost gen` 寫出的四個檔）。
    static let models = [
        ("L32", "lstm-1.4m-f32.safetensors"),
        ("L8", "lstm-1.4m-q8.gguf"),
        ("G32", "gpt-23m-f32.safetensors"),
        ("G8", "gpt-23m-q8.gguf"),
    ]
    /// 與 Mac 的 `run.sh` 相同：每個上下文長度 30 次、2 條執行緒。
    static let iterations: UInt32 = 30
    static let threads: UInt32 = 2

    /// 最近一次的結果；iphone-use 點擊後畫面會重建，結果放在這裡由當下的畫面讀（同 `ModelProbe`）。
    private(set) static var latest: [String] = []
    static var onUpdate: (() -> Void)?
    private static var isRunning = false

    static func run(_ file: String) {
        guard !isRunning else { return }
        guard let url = Bundle.main.url(forResource: file, withExtension: nil, subdirectory: "models") else {
            publish(["nc missing \(file)"])
            return
        }
        isRunning = true
        let before = Probe.memoryLine("nc-before") + " pid=\(getpid())"
        publish([before, "nc running \(file)"])
        let (path, iterations, threads) = (url.path, iterations, threads)
        DispatchQueue.global(qos: .userInitiated).async {
            let start = Date()
            let result = nc_run(path, iterations, threads)
            let text = result.map { String(cString: $0) } ?? "error: nc_run returned null"
            nc_free(result)
            let seconds = Date().timeIntervalSince(start)
            Task { @MainActor in
                isRunning = false
                let lines = text.split(separator: "\n").map(String.init)
                publish([before] + lines + [Probe.memoryLine("nc-after") + String(format: " wall=%.1fs", seconds)])
            }
        }
    }

    private static func publish(_ lines: [String]) {
        latest = lines
        for line in lines { Probe.log.notice("keyboard \(line, privacy: .public)") }
        onUpdate?()
    }
}

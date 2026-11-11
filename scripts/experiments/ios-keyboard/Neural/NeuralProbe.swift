import Foundation

/// 在鍵盤擴充裡跑小型神經模型的開銷量測（M2 票 11）。量測本身在 Rust 靜態庫 `neural-cost`，
/// 這裡只挑模型檔、在背景執行緒呼叫 `nc_run`、把結果顯示在鍵盤上並寫 log。
@MainActor
enum NeuralProbe {
    /// 鍵名、`models/` 裡的檔名（`neural-cost gen` 寫出的四個檔）與路徑形狀（條數, 每條字數）。
    /// 沒給形狀的照 neural-cost 的預設 6 × 8（M2 票 11）；`·9` 兩個鍵是 M3 票 09 用的 9 條 × 18 字
    /// （票 07 定的 N 與口語 dev 路徑長度 p95）。
    struct Model {
        let key: String
        let file: String
        /// 路徑條數與每條字數；nil 照 neural-cost 的預設。
        let paths: (count: Int, length: Int)?
    }

    static let models = [
        Model(key: "L32", file: "lstm-1.4m-f32.safetensors", paths: nil),
        Model(key: "L8", file: "lstm-1.4m-q8.gguf", paths: nil),
        Model(key: "G32", file: "gpt-23m-f32.safetensors", paths: nil),
        Model(key: "G8", file: "gpt-23m-q8.gguf", paths: nil),
        Model(key: "L8·9", file: "lstm-1.4m-q8.gguf", paths: (9, 18)),
        Model(key: "G8·9", file: "gpt-23m-q8.gguf", paths: (9, 18)),
    ]
    /// 與 Mac 的 `run.sh` 相同：每個上下文長度 30 次、2 條執行緒。
    static let iterations: UInt32 = 30
    static let threads: UInt32 = 2

    /// 最近一次的結果；iphone-use 點擊後畫面會重建，結果放在這裡由當下的畫面讀（同 `ModelProbe`）。
    private(set) static var latest: [String] = []
    static var onUpdate: (() -> Void)?
    private static var isRunning = false

    static func run(_ model: Model) {
        let file = model.file
        guard !isRunning else { return }
        guard let url = Bundle.main.url(forResource: file, withExtension: nil, subdirectory: "models") else {
            publish(["nc missing \(file)"])
            return
        }
        isRunning = true
        // neural-cost 的 `bench::run` 讀這兩個環境變數（README〈上次結果〉）。
        if let paths = model.paths {
            setenv("NC_PATHS", String(paths.count), 1)
            setenv("NC_PATH_LEN", String(paths.length), 1)
        } else {
            unsetenv("NC_PATHS")
            unsetenv("NC_PATH_LEN")
        }
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

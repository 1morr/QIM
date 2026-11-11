// Foundation Models 重排的命令列（M3 票 10，README〈Foundation Models〉）。
//
//   fm-rerank info
//   fm-rerank run <提示.jsonl> <回覆.jsonl> [--max-tokens N]
//   fm-rerank tail <提示.jsonl> <回覆.jsonl> [--gaps 2,5,10] [--repeat N] [--follow N]
//
// 提示檔由 `llm.py prompts` 產生，一行一題；回覆檔一行一個請求，`llm.py orders` 把它換回文字。
// 每個請求新開 `LanguageModelSession`（同 IME 每次停鍵送一個獨立請求，同 M2 票 10 的 ModelProbe）。
import Foundation
import FoundationModels

struct PromptLine: Codable {
    let split: String
    let set: String
    let line: Int
    let instructions: String
    let prompt: String
}

struct ReplyLine: Codable {
    let split: String
    let set: String
    let line: Int
    let reply: String
    let ms: Double
    var error: String?
    /// 這個程序的第一個請求。
    var processFirst: Bool?
    /// `tail`：送出前停了幾秒、這是停頓後的第幾個（從 0 起）。
    var gap: Int?
    var burstIndex: Int?
}

func fail(_ message: String) -> Never {
    FileHandle.standardError.write((message + "\n").data(using: .utf8)!)
    exit(1)
}

func readLines<T: Decodable>(_ path: String, as: T.Type) -> [T] {
    guard let text = try? String(contentsOfFile: path, encoding: .utf8) else { fail("讀不到 \(path)") }
    let decoder = JSONDecoder()
    return text.split(separator: "\n").enumerated().map { i, line in
        do { return try decoder.decode(T.self, from: Data(line.utf8)) } catch {
            fail("\(path) 第 \(i + 1) 行：\(error)")
        }
    }
}

final class Output {
    let handle: FileHandle
    let encoder = JSONEncoder()

    init(_ path: String) {
        if !FileManager.default.fileExists(atPath: path) {
            FileManager.default.createFile(atPath: path, contents: nil)
        }
        guard let h = FileHandle(forWritingAtPath: path) else { fail("寫不了 \(path)") }
        h.seekToEndOfFile()
        handle = h
        encoder.outputFormatting = [.sortedKeys, .withoutEscapingSlashes]
    }

    func write(_ reply: ReplyLine) {
        var data = try! encoder.encode(reply)
        data.append(0x0A)
        handle.write(data)
    }
}

nonisolated(unsafe) var requestsInProcess = 0

func request(_ p: PromptLine, maxTokens: Int) async -> ReplyLine {
    requestsInProcess += 1
    let session = LanguageModelSession(instructions: p.instructions)
    let options = GenerationOptions(samplingMode: .greedy, maximumResponseTokens: maxTokens)
    let start = ContinuousClock.now
    var reply = ""
    var error: String?
    do {
        reply = try await session.respond(to: p.prompt, options: options).content
    } catch let e {
        error = "\(e)"
    }
    let d = ContinuousClock.now - start
    let ms = Double(d.components.seconds) * 1000 + Double(d.components.attoseconds) / 1e15
    return ReplyLine(
        split: p.split, set: p.set, line: p.line, reply: reply, ms: (ms * 10).rounded() / 10,
        error: error, processFirst: requestsInProcess == 1 ? true : nil)
}

func info() {
    let model = SystemLanguageModel.default
    print("availability=\(model.availability)")
    if #available(macOS 27.0, *) { print("variant=\(model.variant.displayName)") }
    print("contextSize=\(model.contextSize)")
    print("zh-Hant-HK=\(model.supportsLocale(Locale(identifier: "zh-Hant-HK")))")
    let os = ProcessInfo.processInfo.operatingSystemVersionString
    print("os=\(os)")
}

func requireModel() {
    guard SystemLanguageModel.default.isAvailable else {
        fail("Foundation Models 不能用：\(SystemLanguageModel.default.availability)")
    }
}

func flag(_ args: [String], _ name: String) -> String? {
    guard let i = args.firstIndex(of: name), i + 1 < args.count else { return nil }
    return args[i + 1]
}

/// 逐題送出；回覆檔已有的題跳過（中斷後重跑接著做）。
func run(_ args: [String]) async {
    guard args.count >= 2 else { fail("用法：fm-rerank run <提示.jsonl> <回覆.jsonl>") }
    requireModel()
    let maxTokens = Int(flag(args, "--max-tokens") ?? "8") ?? 8
    let prompts = readLines(args[0], as: PromptLine.self)
    var done = Set<String>()
    if FileManager.default.fileExists(atPath: args[1]) {
        for r in readLines(args[1], as: ReplyLine.self) { done.insert("\(r.split)/\(r.set)/\(r.line)") }
    }
    let out = Output(args[1])
    var count = 0
    for p in prompts where !done.contains("\(p.split)/\(p.set)/\(p.line)") {
        out.write(await request(p, maxTokens: maxTokens))
        count += 1
        if count % 200 == 0 {
            FileHandle.standardError.write("\(count) / \(prompts.count - done.count)\n".data(using: .utf8)!)
        }
    }
}

/// 停頓後第一個請求的延遲：依序停 `gaps` 秒、送 1 + `follow` 個請求，整輪做 `repeat` 次。
/// 提示依序輪流取用，同一題不連著送。
func tail(_ args: [String]) async {
    guard args.count >= 2 else { fail("用法：fm-rerank tail <提示.jsonl> <回覆.jsonl>") }
    requireModel()
    let gaps = (flag(args, "--gaps") ?? "2,5,10,30,60,120,300").split(separator: ",").compactMap { Int($0) }
    let repeats = Int(flag(args, "--repeat") ?? "3") ?? 3
    let follow = Int(flag(args, "--follow") ?? "2") ?? 2
    let prompts = readLines(args[0], as: PromptLine.self)
    guard !prompts.isEmpty else { fail("沒有提示") }
    let out = Output(args[1])
    var next = 0
    for _ in 0..<repeats {
        for gap in gaps {
            try? await Task.sleep(for: .seconds(gap))
            for k in 0...follow {
                var r = await request(prompts[next % prompts.count], maxTokens: 8)
                next += 1
                r.gap = gap
                r.burstIndex = k
                out.write(r)
                FileHandle.standardError.write("gap \(gap)s #\(k)：\(r.ms) ms\n".data(using: .utf8)!)
            }
        }
    }
}

let args = Array(CommandLine.arguments.dropFirst())
switch args.first {
case "info": info()
case "run": await run(Array(args.dropFirst()))
case "tail": await tail(Array(args.dropFirst()))
default: fail("用法：fm-rerank info | run <提示> <回覆> | tail <提示> <回覆>")
}

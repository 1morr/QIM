import UIKit

/// 量 `textDocumentProxy` 讀得到多少游標前後文（M2 票 10）。
enum ContextProbe {
    /// 測試長文：9 段，每段 40 個 10 字的單元，第 4、5 段之間空一行。
    /// 單元是 `[段.序]天地玄` 加「，」或（每第 5 個）「。」，看到開頭就知道從哪個單元起讀。
    static let longText: String = {
        let paragraphs = (1...9).map { p in
            (1...40).map { u in
                String(format: "[%d.%02d]天地玄", p, u) + (u % 5 == 0 ? "。" : "，")
            }.joined()
        }
        return paragraphs[0..<4].joined(separator: "\n") + "\n\n" + paragraphs[4...].joined(separator: "\n")
    }()

    /// 沒有標點、沒有換行的一段 3000 字：300 個 `[序號]天地玄黃` 單元（各 10 字），看沒有句界時的上限。
    static let longParagraph: String = (1...300).map { String(format: "[%04d]天地玄黃", $0) }.joined()

    /// 一行摘要：字數（Character）、UTF-16 長度、換行數、頭尾各 12 字（換行顯示成 ⏎）。
    static func summary(_ label: String, _ text: String?) -> String {
        guard let text else { return "\(label)=nil" }
        let newlines = text.reduce(0) { $1 == "\n" ? $0 + 1 : $0 }
        return "\(label) chars=\(text.count) utf16=\(text.utf16.count) nl=\(newlines)"
            + " head=\(visible(text.prefix(12))) tail=\(visible(text.suffix(12)))"
    }

    @MainActor static func lines(_ proxy: UITextDocumentProxy) -> [String] {
        [
            summary("before", proxy.documentContextBeforeInput),
            summary("after", proxy.documentContextAfterInput),
        ]
    }

    private static func visible(_ s: Substring) -> String {
        "'" + s.replacingOccurrences(of: "\n", with: "⏎") + "'"
    }
}

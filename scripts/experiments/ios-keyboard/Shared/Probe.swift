import Darwin
import Foundation
import os

/// 主 App 與鍵盤擴充共用的量測：常駐記憶體與 App Group 共享容器。
enum Probe {
    static let log = Logger(subsystem: "qim.exp", category: "probe")

    /// `phys_footprint`，單位 MB；推論為系統記憶體上限所看的 footprint，依據見 README〈常駐記憶體〉。
    static func footprintMB() -> Double? {
        var info = task_vm_info_data_t()
        var count = mach_msg_type_number_t(MemoryLayout<task_vm_info_data_t>.size / MemoryLayout<integer_t>.size)
        let kr = withUnsafeMutablePointer(to: &info) {
            $0.withMemoryRebound(to: integer_t.self, capacity: Int(count)) {
                task_info(mach_task_self_, task_flavor_t(TASK_VM_INFO), $0, &count)
            }
        }
        guard kr == KERN_SUCCESS else { return nil }
        return Double(info.phys_footprint) / 1_048_576
    }

    /// `ledger_phys_footprint_peak`（`task_vm_info` REV3 起的欄位）：程序到讀數時為止 footprint 的最高點，
    /// 單位 MB。iOS 會還回釋放的大塊配置，只看載入後的 footprint 會漏掉解開途中的峰值（M2 票 11）。
    static func footprintPeakMB() -> Double? {
        var info = task_vm_info_data_t()
        var count = mach_msg_type_number_t(MemoryLayout<task_vm_info_data_t>.size / MemoryLayout<integer_t>.size)
        let kr = withUnsafeMutablePointer(to: &info) {
            $0.withMemoryRebound(to: integer_t.self, capacity: Int(count)) {
                task_info(mach_task_self_, task_flavor_t(TASK_VM_INFO), $0, &count)
            }
        }
        guard kr == KERN_SUCCESS else { return nil }
        return Double(info.ledger_phys_footprint_peak) / 1_048_576
    }

    /// 離系統終止還剩多少記憶體（`os_proc_available_memory`），單位 MB；0 表示系統不提供。
    static func availableMB() -> Double {
        Double(os_proc_available_memory()) / 1_048_576
    }

    static func memoryLine(_ stage: String) -> String {
        let fp = footprintMB().map { String(format: "%.2f", $0) } ?? "?"
        return String(format: "%@ footprint=%@MB available=%.1fMB", stage, fp, availableMB())
    }

    // MARK: App Group

    /// 主 App 寫、擴充讀的檔案。
    static let appFile = "from-app.txt"
    /// 擴充試寫的檔案；沒有完整取用權限時應該寫不進去（brief §12.3）。
    static let keyboardFile = "from-keyboard.txt"

    static var groupID: String {
        Bundle.main.object(forInfoDictionaryKey: "QIMAppGroup") as? String ?? "?"
    }

    static var containerURL: URL? {
        FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: groupID)
    }

    static func read(_ name: String) -> String {
        guard let dir = containerURL else { return "no-container" }
        do {
            return "ok:" + (try String(contentsOf: dir.appendingPathComponent(name), encoding: .utf8))
        } catch {
            return "read-failed:\(errorSummary(error))"
        }
    }

    static func write(_ name: String, _ text: String) -> String {
        guard let dir = containerURL else { return "no-container" }
        do {
            try text.write(to: dir.appendingPathComponent(name), atomically: true, encoding: .utf8)
            return "ok"
        } catch {
            return "write-failed:\(errorSummary(error))"
        }
    }

    static func timestamp() -> String {
        ISO8601DateFormatter().string(from: Date())
    }

    private static func errorSummary(_ error: Error) -> String {
        let ns = error as NSError
        if let underlying = ns.userInfo[NSUnderlyingErrorKey] as? NSError {
            return "\(ns.domain)#\(ns.code)/\(underlying.domain)#\(underlying.code)"
        }
        return "\(ns.domain)#\(ns.code)"
    }
}

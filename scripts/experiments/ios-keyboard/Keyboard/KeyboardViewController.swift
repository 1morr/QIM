import UIKit

/// 實驗鍵盤：送出「日」、量常駐記憶體與 App Group（票 06）、讀游標前後文長度與試 Foundation Models（票 10）、
/// 接上核心打 `anr`、量記憶體與每鍵延遲（票 08）。結果顯示在鍵盤上方並寫 log。
final class KeyboardViewController: UIInputViewController {
    /// 這個程序裡各鍵被按的次數與 view controller 載入次數，診斷用。
    private static var taps: [String: Int] = [:]
    private static var loads = 0
    private let status = UILabel()
    private let candidates = UILabel()
    private var loadLine = ""
    private var memoryLines: [String] = []
    private var contextLines: [String] = []

    override func viewDidLoad() {
        super.viewDidLoad()
        Self.loads += 1
        loadLine = Probe.memoryLine("viewDidLoad") + " pid=\(getpid()) loads=\(Self.loads)"
        Probe.log.notice("keyboard \(self.loadLine, privacy: .public)")

        status.numberOfLines = 0
        status.font = .monospacedSystemFont(ofSize: 9, weight: .regular)
        status.accessibilityIdentifier = "qim-status"
        status.setContentHuggingPriority(.defaultLow, for: .vertical)

        let globe = UIButton(type: .system)
        globe.setImage(UIImage(systemName: "globe"), for: .normal)
        globe.addTarget(self, action: #selector(handleInputModeList(from:with:)), for: .allTouchEvents)
        globe.isHidden = !needsInputModeSwitchKey
        globe.widthAnchor.constraint(equalToConstant: 36).isActive = true

        let keyRow = row([
            globe,
            key("日", id: "qim-key-ri") { $0.textDocumentProxy.insertText("日") },
            key("長文", id: "qim-key-long") { $0.textDocumentProxy.insertText(ContextProbe.longText) },
            key("長段", id: "qim-key-para") { $0.textDocumentProxy.insertText(ContextProbe.longParagraph) },
            key("讀", id: "qim-key-read") { $0.readContext() },
            key("FM", id: "qim-key-fm") { _ in Task { @MainActor in await ModelProbe.run() } },
            key("間隔", id: "qim-key-gaps") { _ in Task { @MainActor in await ModelProbe.runGaps() } },
        ])
        let moveRow = row([-1000, -100, -10, -1, 10, 100].map { offset in
            key(offset < 0 ? "←\(-offset)" : "→\(offset)", id: "qim-key-move\(offset)") { $0.move(by: offset) }
        } + [
            // 往前移 before 的長度：連按就一句一句往前走，可走到段首。
            key("⇤", id: "qim-key-back-context") { $0.move(by: -($0.textDocumentProxy.documentContextBeforeInput?.count ?? 0)) },
        ])

        candidates.font = .systemFont(ofSize: 18)
        candidates.accessibilityIdentifier = "qim-candidates"
        let coreRow = row([
            key("載入", id: "qim-key-load") { _ in CoreProbe.load() },
        ] + ["a", "n", "r"].map { letter in
            key(letter, id: "qim-key-\(letter)") { $0.pressCore(letter, kind: "letter", value: letter) }
        } + [
            key("空白", id: "qim-key-space") { $0.pressCore("space", kind: "space", value: nil) },
            key("延遲", id: "qim-key-latency") { _ in CoreProbe.runLatency() },
        ])

        let stack = UIStackView(arrangedSubviews: [status, candidates, coreRow, keyRow, moveRow])
        stack.axis = .vertical
        stack.spacing = 6
        stack.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 6),
            stack.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -6),
            stack.topAnchor.constraint(equalTo: view.topAnchor, constant: 6),
            stack.bottomAnchor.constraint(equalTo: view.bottomAnchor, constant: -6),
            keyRow.heightAnchor.constraint(equalToConstant: 44),
            moveRow.heightAnchor.constraint(equalToConstant: 44),
            coreRow.heightAnchor.constraint(equalToConstant: 44),
            view.heightAnchor.constraint(equalToConstant: 380),
        ])
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        memoryLines = [loadLine, Probe.memoryLine("viewDidAppear")]
        // App Group 與完整取用三行螢幕放不下，只寫 log。
        log(memoryLines + [
            "full-access=\(hasFullAccess)",
            "group-read=\(Probe.read(Probe.appFile))",
            "group-write=\(Probe.write(Probe.keyboardFile, "keyboard \(Probe.timestamp())"))",
        ])
        ModelProbe.onUpdate = { [weak self] in self?.render() }
        CoreProbe.onUpdate = { [weak self] in self?.render() }
        log(ModelProbe.displayLines + [ModelProbe.languagesLine()])
        readContext()
    }

    override func textDidChange(_ textInput: UITextInput?) {
        super.textDidChange(textInput)
        readContext()
    }

    override func selectionDidChange(_ textInput: UITextInput?) {
        super.selectionDidChange(textInput)
        readContext()
    }

    private func readContext() {
        contextLines = ContextProbe.lines(textDocumentProxy)
        log(contextLines)
        render()
    }

    /// 宿主更新 proxy 的前後文是非同步的，移動後稍等再讀。
    private func move(by offset: Int) {
        textDocumentProxy.adjustTextPosition(byCharacterOffset: offset)
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) { [weak self] in self?.readContext() }
    }

    /// 按核心的鍵：上屏快照的 `commit`，記下核心呼叫與含畫面更新的時間。含畫面更新的是從這裡開始到
    /// 這次畫面的 Core Animation transaction 送出（completion block），不含觸控事件送達前的時間，只當大概值。
    private func pressCore(_ label: String, kind: String, value: String?) {
        let start = clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
        guard let (snapshot, coreMs) = CoreProbe.press(kind, value) else { return }
        if let commit = snapshot.commit {
            textDocumentProxy.insertText(commit)
        } else if !snapshot.handled, kind == "space" {
            textDocumentProxy.insertText(" ")
        }
        CATransaction.begin()
        CATransaction.setCompletionBlock {
            MainActor.assumeIsolated { CoreProbe.recordKey(label, coreMs: coreMs, screenMs: ms(since: start)) }
        }
        render()
        CATransaction.commit()
    }

    private func render() {
        let taps = "taps=" + Self.taps.sorted { $0.key < $1.key }.map { "\($0.key):\($0.value)" }.joined(separator: ",")
        // 接上核心後只顯示核心的行，螢幕放不下全部；其他行照樣寫 log。
        let probes = CoreProbe.active ? CoreProbe.displayLines : contextLines + ModelProbe.displayLines
        status.text = (memoryLines + [taps] + probes).joined(separator: "\n")
        candidates.text = CoreProbe.candidateLine
    }

    private func log(_ lines: [String]) {
        for line in lines { Probe.log.notice("keyboard \(line, privacy: .public)") }
    }

    private func key(_ title: String, id: String, action: @escaping (KeyboardViewController) -> Void) -> UIButton {
        let button = UIButton(type: .system)
        button.setTitle(title, for: .normal)
        button.titleLabel?.font = .systemFont(ofSize: 18)
        button.backgroundColor = .secondarySystemBackground
        button.layer.cornerRadius = 8
        button.accessibilityIdentifier = id
        button.addAction(UIAction { [weak self] _ in
            Self.taps[title, default: 0] += 1
            if let self { action(self); render() }
        }, for: .touchUpInside)
        return button
    }

    private func row(_ views: [UIView]) -> UIStackView {
        let row = UIStackView(arrangedSubviews: views)
        row.spacing = 6
        row.distribution = .fillProportionally
        return row
    }
}

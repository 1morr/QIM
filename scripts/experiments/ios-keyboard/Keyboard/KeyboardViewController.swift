import UIKit

/// 空的鍵盤擴充：一個鍵送出「日」，每次啟動量常駐記憶體、讀寫 App Group，結果顯示在鍵盤上並寫 log。
final class KeyboardViewController: UIInputViewController {
    private let status = UILabel()
    private var loadLine = ""

    override func viewDidLoad() {
        super.viewDidLoad()
        loadLine = Probe.memoryLine("viewDidLoad")
        Probe.log.notice("keyboard \(self.loadLine, privacy: .public)")

        status.numberOfLines = 0
        status.font = .monospacedSystemFont(ofSize: 11, weight: .regular)
        status.accessibilityIdentifier = "qim-status"

        let key = UIButton(type: .system)
        key.setTitle("日", for: .normal)
        key.titleLabel?.font = .systemFont(ofSize: 28)
        key.backgroundColor = .secondarySystemBackground
        key.layer.cornerRadius = 8
        key.accessibilityIdentifier = "qim-key-ri"
        key.addAction(UIAction { [weak self] _ in self?.textDocumentProxy.insertText("日") }, for: .touchUpInside)

        let globe = UIButton(type: .system)
        globe.setImage(UIImage(systemName: "globe"), for: .normal)
        globe.addTarget(self, action: #selector(handleInputModeList(from:with:)), for: .allTouchEvents)
        globe.isHidden = !needsInputModeSwitchKey

        let row = UIStackView(arrangedSubviews: [globe, key])
        row.spacing = 8
        row.distribution = .fill
        globe.widthAnchor.constraint(equalToConstant: 44).isActive = true

        let stack = UIStackView(arrangedSubviews: [status, row])
        stack.axis = .vertical
        stack.spacing = 8
        stack.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 8),
            stack.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -8),
            stack.topAnchor.constraint(equalTo: view.topAnchor, constant: 8),
            stack.bottomAnchor.constraint(equalTo: view.bottomAnchor, constant: -8),
            row.heightAnchor.constraint(equalToConstant: 56),
            view.heightAnchor.constraint(equalToConstant: 200),
        ])
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        let appearLine = Probe.memoryLine("viewDidAppear")
        let read = Probe.read(Probe.appFile)
        let write = Probe.write(Probe.keyboardFile, "keyboard \(Probe.timestamp())")
        let lines = [
            loadLine,
            appearLine,
            "full-access=\(hasFullAccess)",
            "group-read=\(read)",
            "group-write=\(write)",
        ]
        for line in lines { Probe.log.notice("keyboard \(line, privacy: .public)") }
        status.text = lines.joined(separator: "\n")
    }
}

import SwiftUI

@main
struct QIMKeyboardExpApp: App {
    var body: some Scene {
        WindowGroup { ContentView() }
    }
}

/// 啟動時把時間戳寫進 App Group，給鍵盤擴充讀；也讀回擴充試寫的檔案。
struct ContentView: View {
    @State private var lines: [String] = []

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("QIM 鍵盤擴充實驗").font(.title2.bold())
            Text("設定 → 一般 → 鍵盤 → 鍵盤 → 新增鍵盤 → QIM 實驗鍵盤")
                .font(.footnote)
            ForEach(lines, id: \.self) { Text($0).font(.system(.footnote, design: .monospaced)) }
            Button("重寫 App Group 檔") { probe() }
            Spacer()
        }
        .padding()
        .onAppear(perform: probe)
    }

    private func probe() {
        let stamp = Probe.timestamp()
        let write = Probe.write(Probe.appFile, "app \(stamp)")
        let readBack = Probe.read(Probe.appFile)
        let fromKeyboard = Probe.read(Probe.keyboardFile)
        lines = [
            "group=\(Probe.groupID)",
            "container=\(Probe.containerURL?.path ?? "nil")",
            "app-write=\(write)",
            "app-read=\(readBack)",
            "keyboard-file=\(fromKeyboard)",
        ]
        for line in lines { Probe.log.notice("app \(line, privacy: .public)") }
    }
}

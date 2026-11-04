// 用法：lat <pid> "<按鍵>"
// 逐鍵把按鍵事件送給 pid，輪詢焦點元素的無障礙值直到改變，每鍵印「鍵 毫秒」（逾時 1 秒印 -1）。
// 只支援小寫字母與空白。量法與結果見同目錄 README.md。
import Cocoa

let pid = pid_t(CommandLine.arguments[1])!
let keys = Array(CommandLine.arguments[2])
let keyCodes: [Character: CGKeyCode] = [
  "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9, "b": 11,
  "q": 12, "w": 13, "e": 14, "r": 15, "y": 16, "t": 17, "o": 31, "u": 32, "i": 34, "p": 35,
  "l": 37, "j": 38, "k": 40, "n": 45, "m": 46, " ": 49,
]

print("trusted:", AXIsProcessTrusted())
let app = AXUIElementCreateApplication(pid)
var focusedRef: CFTypeRef?
AXUIElementCopyAttributeValue(app, kAXFocusedUIElementAttribute as CFString, &focusedRef)
guard let focusedRef else {
  print("找不到焦點元素")
  exit(1)
}
let focused = focusedRef as! AXUIElement

func currentValue() -> String {
  var value: CFTypeRef?
  AXUIElementCopyAttributeValue(focused, kAXValueAttribute as CFString, &value)
  return (value as? String) ?? ""
}

let source = CGEventSource(stateID: .hidSystemState)
for key in keys {
  let before = currentValue()
  let down = CGEvent(keyboardEventSource: source, virtualKey: keyCodes[key]!, keyDown: true)!
  let up = CGEvent(keyboardEventSource: source, virtualKey: keyCodes[key]!, keyDown: false)!
  let start = DispatchTime.now().uptimeNanoseconds
  down.postToPid(pid)
  up.postToPid(pid)
  var elapsedMs = -1.0
  while true {
    let ms = Double(DispatchTime.now().uptimeNanoseconds - start) / 1e6
    if currentValue() != before {
      elapsedMs = ms
      break
    }
    if ms > 1000 { break }
    usleep(200)
  }
  print(key == " " ? "␣" : String(key), String(format: "%.2f", elapsedMs))
  usleep(150_000)
}

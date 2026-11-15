import CoreGraphics
import Foundation

// 列出当前屏上所有窗口: id|owner|name|x|y|w|h
let opts: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
guard let list = CGWindowListCopyWindowInfo(opts, kCGNullWindowID) as? [[String: Any]] else {
    exit(1)
}
let target = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : ""
for w in list {
    let owner = w[kCGWindowOwnerName as String] as? String ?? ""
    let name = w[kCGWindowName as String] as? String ?? ""
    let id = w[kCGWindowNumber as String] as? Int ?? 0
    let b = w[kCGWindowBounds as String] as? [String: Any] ?? [:]
    let x = b["X"] as? Int ?? 0
    let y = b["Y"] as? Int ?? 0
    let wid = b["Width"] as? Int ?? 0
    let h = b["Height"] as? Int ?? 0
    if target.isEmpty || owner.lowercased().contains(target.lowercased())
        || name.lowercased().contains(target.lowercased()) {
        print("\(id)|\(owner)|\(name)|\(x)|\(y)|\(wid)|\(h)")
    }
}

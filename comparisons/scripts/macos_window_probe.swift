import AppKit
import CoreFoundation
import CoreGraphics
import Darwin
import Foundation

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data((message + "\n").utf8))
    exit(1)
}

func parseProcessIds(_ value: String) -> Set<Int> {
    Set(value.split(separator: ",").compactMap { Int($0) })
}

func visibleWindows(processIds: Set<Int>) -> [[String: Any]] {
    guard let raw = CGWindowListCopyWindowInfo(
        [.optionOnScreenOnly, .excludeDesktopElements],
        kCGNullWindowID
    ) as? [[String: Any]] else {
        return []
    }
    return raw.compactMap { window in
        guard
            let processId = window[kCGWindowOwnerPID as String] as? Int,
            processIds.contains(processId),
            let number = window[kCGWindowNumber as String] as? Int,
            let layer = window[kCGWindowLayer as String] as? Int,
            layer == 0,
            let boundsObject = window[kCGWindowBounds as String]
        else {
            return nil
        }
        let boundsValue = boundsObject as! CFDictionary
        var bounds = CGRect.zero
        guard CGRectMakeWithDictionaryRepresentation(boundsValue, &bounds) else {
            return nil
        }
        guard bounds.width >= 320, bounds.height >= 240 else {
            return nil
        }
        let result: [String: Any] = [
            "pid": processId,
            "window_id": number,
            "x": bounds.origin.x,
            "y": bounds.origin.y,
            "width": bounds.width,
            "height": bounds.height,
        ]
        return result
    }
}

let arguments = Array(CommandLine.arguments.dropFirst())
guard let command = arguments.first else {
    fail("usage: macos-window-probe windows <pid,...> | hide <pid> | churn <pid> <seconds> <hz>")
}

switch command {
case "windows":
    guard arguments.count == 2 else { fail("windows requires a comma-separated PID list") }
    let windows = visibleWindows(processIds: parseProcessIds(arguments[1]))
    let data = try JSONSerialization.data(withJSONObject: windows, options: [.sortedKeys])
    FileHandle.standardOutput.write(data)
case "hide":
    guard arguments.count == 2, let processId = Int32(arguments[1]) else {
        fail("hide requires one PID")
    }
    guard let application = NSRunningApplication(processIdentifier: processId) else {
        fail("no NSRunningApplication exists for PID \(processId)")
    }
    _ = application.activate(options: [.activateAllWindows, .activateIgnoringOtherApps])
    guard application.hide() else {
        fail("NSRunningApplication rejected hide")
    }
    for _ in 0..<40 {
        if visibleWindows(processIds: [Int(processId)]).isEmpty {
            exit(0)
        }
        usleep(50_000)
    }
    fail("NSRunningApplication.hide left an on-screen application window")
case "churn":
    guard
        arguments.count == 4,
        let processId = Int32(arguments[1]),
        let seconds = Double(arguments[2]),
        let hz = Double(arguments[3]),
        seconds > 0,
        hz > 0
    else {
        fail("churn requires PID, positive seconds and positive Hz")
    }
    guard let application = NSRunningApplication(processIdentifier: processId) else {
        fail("no NSRunningApplication exists for PID \(processId)")
    }
    let halfInterval = useconds_t((1_000_000.0 / hz / 2.0).rounded())
    let deadline = Date().addingTimeInterval(seconds)
    while Date() < deadline {
        autoreleasepool {
            _ = application.hide()
            usleep(halfInterval)
            _ = application.unhide()
            _ = application.activate(options: [.activateIgnoringOtherApps])
            usleep(halfInterval)
        }
    }
default:
    fail("unknown command: \(command)")
}

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
case "launch":
    guard arguments.count >= 2 else { fail("launch requires an application bundle") }
    let bundle = URL(fileURLWithPath: arguments[1]).standardizedFileURL
    let configuration = NSWorkspace.OpenConfiguration()
    configuration.createsNewApplicationInstance = true
    configuration.activates = true
    configuration.addsToRecentItems = false
    configuration.arguments = Array(arguments.dropFirst(2))
    configuration.environment = ProcessInfo.processInfo.environment
    var completed = false
    var launched: NSRunningApplication?
    var launchError: Error?
    NSWorkspace.shared.openApplication(at: bundle, configuration: configuration) { application, error in
        launched = application
        launchError = error
        completed = true
    }
    let deadline = Date().addingTimeInterval(20)
    while !completed && Date() < deadline {
        RunLoop.current.run(until: Date().addingTimeInterval(0.025))
    }
    guard let application = launched, let launchDate = application.launchDate else {
        launched?.terminate()
        fail("LaunchServices did not return an application identity: \(String(describing: launchError))")
    }
    let result: [String: Any] = [
        "pid": Int(application.processIdentifier),
        "launch_token": String(launchDate.timeIntervalSince1970),
        "bundle": application.bundleURL?.standardizedFileURL.path ?? "",
    ]
    FileHandle.standardOutput.write(try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys]))
case "state", "terminate", "kill":
    guard arguments.count == 3, let processId = Int32(arguments[1]) else {
        fail("\(command) requires PID and launch token")
    }
    let application = NSRunningApplication(processIdentifier: processId)
    let matches = application?.launchDate.map {
        String($0.timeIntervalSince1970) == arguments[2]
    } ?? false
    let alive = matches && application?.isTerminated == false
    if alive && command == "terminate" { _ = application?.terminate() }
    if alive && command == "kill" { _ = application?.forceTerminate() }
    FileHandle.standardOutput.write(try JSONSerialization.data(withJSONObject: ["alive": alive]))
case "windows":
    guard arguments.count == 2 else { fail("windows requires a comma-separated PID list") }
    let windows = visibleWindows(processIds: parseProcessIds(arguments[1]))
    let data = try JSONSerialization.data(withJSONObject: windows, options: [.sortedKeys])
    FileHandle.standardOutput.write(data)
case "hide":
    guard (2...3).contains(arguments.count), let processId = Int32(arguments[1]) else {
        fail("hide requires one PID")
    }
    guard let application = NSRunningApplication(processIdentifier: processId) else {
        fail("no NSRunningApplication exists for PID \(processId)")
    }
    if arguments.count == 3 && application.launchDate.map({ String($0.timeIntervalSince1970) }) != arguments[2] {
        fail("application identity changed before hide")
    }
    guard application.activate(options: [.activateAllWindows, .activateIgnoringOtherApps]) else {
        fail("NSRunningApplication rejected activation")
    }
    let activationDeadline = Date().addingTimeInterval(3)
    while !application.isActive && Date() < activationDeadline {
        RunLoop.current.run(until: Date().addingTimeInterval(0.05))
    }
    guard application.isActive else {
        fail("NSRunningApplication did not become active before hide")
    }
    guard application.hide() else {
        fail("NSRunningApplication rejected hide")
    }
    for _ in 0..<40 {
        if visibleWindows(processIds: [Int(processId)]).isEmpty {
            exit(0)
        }
        RunLoop.current.run(until: Date().addingTimeInterval(0.05))
    }
    fail("NSRunningApplication.hide left an on-screen application window")
case "churn":
    guard
        (4...5).contains(arguments.count),
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
    if arguments.count == 5 && application.launchDate.map({ String($0.timeIntervalSince1970) }) != arguments[4] {
        fail("application identity changed before repaint drive")
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

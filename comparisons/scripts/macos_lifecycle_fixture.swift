import AppKit
import Foundation

// A native AppKit control for the external measurement driver's lifecycle.
// It has no framework, renderer, document or menu dependencies.
let application = NSApplication.shared
application.setActivationPolicy(.regular)
let window = NSWindow(
    contentRect: NSRect(x: 100, y: 100, width: 640, height: 480),
    styleMask: [.titled, .closable, .resizable],
    backing: .buffered,
    defer: false
)
window.title = "Native lifecycle control"
window.backgroundColor = .windowBackgroundColor
window.makeKeyAndOrderFront(nil)
application.activate(ignoringOtherApps: true)
DispatchQueue.main.asyncAfter(deadline: .now() + 30) {
    application.terminate(nil)
}
application.run()

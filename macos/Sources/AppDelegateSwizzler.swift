import AppKit
import ObjectiveC.runtime

#if ENABLE_PUSH_NOTIFICATIONS

enum AppDelegateSwizzler {
  static weak var plugin: NotificationPlugin?

  static func swizzlePushCallbacks() {
    guard let delegate = NSApplication.shared.delegate else { return }

    // didRegisterForRemoteNotificationsWithDeviceToken
    swizzle(
      type(of: delegate),
      #selector(NSApplicationDelegate.application(_:didRegisterForRemoteNotificationsWithDeviceToken:)),
      #selector(PushForwarder.ta_application(_:didRegisterForRemoteNotificationsWithDeviceToken:))
    )

    // didFailToRegisterForRemoteNotificationsWithError
    swizzle(
      type(of: delegate),
      #selector(NSApplicationDelegate.application(_:didFailToRegisterForRemoteNotificationsWithError:)),
      #selector(PushForwarder.ta_application(_:didFailToRegisterForRemoteNotificationsWithError:))
    )

    // didReceiveRemoteNotification
    swizzle(
      type(of: delegate),
      #selector(NSApplicationDelegate.application(_:didReceiveRemoteNotification:)),
      #selector(PushForwarder.ta_application(_:didReceiveRemoteNotification:))
    )
  }

  private static func swizzle(_ cls: AnyClass, _ original: Selector, _ replacement: Selector) {
    guard
      let swizzledMethod = class_getInstanceMethod(PushForwarder.self, replacement)
    else { return }

    if let originalMethod = class_getInstanceMethod(cls, original) {
      method_exchangeImplementations(originalMethod, swizzledMethod)
    } else {
      class_addMethod(
        cls,
        original,
        method_getImplementation(swizzledMethod),
        method_getTypeEncoding(swizzledMethod)
      )
    }
  }
}

// Extension to hold swizzled implementations (silences "nearly matches" warnings)
final class PushForwarder: NSObject {
  @objc func ta_application(_ application: NSApplication,
                            didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
    let hex = deviceToken.map { String(format: "%02x", $0) }.joined()

    AppDelegateSwizzler.plugin?.handlePushTokenReceived(hex)
    try? AppDelegateSwizzler.plugin?.trigger("push-token", data: ["token": hex])

    if responds(to: #selector(ta_application(_:didRegisterForRemoteNotificationsWithDeviceToken:))) {
      self.ta_application(application, didRegisterForRemoteNotificationsWithDeviceToken: deviceToken)
    }
  }

  @objc func ta_application(_ application: NSApplication,
                            didFailToRegisterForRemoteNotificationsWithError error: Error) {
    AppDelegateSwizzler.plugin?.handlePushTokenError(error)
    try? AppDelegateSwizzler.plugin?.trigger("push-error", data: ["message": error.localizedDescription])

    if responds(to: #selector(ta_application(_:didFailToRegisterForRemoteNotificationsWithError:))) {
      self.ta_application(application, didFailToRegisterForRemoteNotificationsWithError: error)
    }
  }

  @objc func ta_application(_ application: NSApplication,
                            didReceiveRemoteNotification userInfo: [String: Any]) {
    // Convert to [String: String] for Encodable
    var stringDict: [String: String] = [:]
    for (key, value) in userInfo {
      stringDict[key] = String(describing: value)
    }
    try? AppDelegateSwizzler.plugin?.trigger("push-message", data: stringDict)

    if responds(to: #selector(ta_application(_:didReceiveRemoteNotification:))) {
      self.ta_application(application, didReceiveRemoteNotification: userInfo)
    }
  }
}

#endif

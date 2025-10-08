import UIKit
import Tauri
import ObjectiveC.runtime

#if ENABLE_PUSH_NOTIFICATIONS

enum AppDelegateSwizzler {
  static weak var plugin: NotificationPlugin?

  static func swizzlePushCallbacks() {
    guard let app = UIApplication.shared as UIApplication?,
          let delegate = app.delegate else { return }

    // didRegisterForRemoteNotificationsWithDeviceToken
    swizzle(
      type(of: delegate),
      #selector(UIApplicationDelegate.application(_:didRegisterForRemoteNotificationsWithDeviceToken:)),
      #selector(PushForwarder.ta_application(_:didRegisterForRemoteNotificationsWithDeviceToken:))
    )

    // didFailToRegisterForRemoteNotificationsWithError
    swizzle(
      type(of: delegate),
      #selector(UIApplicationDelegate.application(_:didFailToRegisterForRemoteNotificationsWithError:)),
      #selector(PushForwarder.ta_application(_:didFailToRegisterForRemoteNotificationsWithError:))
    )

    // didReceiveRemoteNotification (silent/background)
    swizzle(
      type(of: delegate),
      #selector(UIApplicationDelegate.application(_:didReceiveRemoteNotification:fetchCompletionHandler:)),
      #selector(PushForwarder.ta_application(_:didReceiveRemoteNotification:fetchCompletionHandler:))
    )
  }

  private static func swizzle(_ cls: AnyClass, _ original: Selector, _ replacement: Selector) {
    guard
      let swizzledMethod = class_getInstanceMethod(PushForwarder.self, replacement)
    else { return }

    if let originalMethod = class_getInstanceMethod(cls, original) {
      // Original method exists - exchange implementations
      method_exchangeImplementations(originalMethod, swizzledMethod)
    } else {
      // Original method doesn't exist - add our method
      class_addMethod(
        cls,
        original,
        method_getImplementation(swizzledMethod),
        method_getTypeEncoding(swizzledMethod)
      )
    }
  }
}

/// A helper that hosts the swizzled implementations.
/// NOTE: These method names must exactly match the selectors we swizzle.
final class PushForwarder: NSObject, UIApplicationDelegate {
  // Token success
  @objc func ta_application(_ application: UIApplication,
                            didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
    // Convert token to hex string
    let hex = deviceToken.map { String(format: "%02x", $0) }.joined()

    // Notify plugin about token
    AppDelegateSwizzler.plugin?.handlePushTokenReceived(hex)

    // Also emit event for JS/Rust listeners
    try? AppDelegateSwizzler.plugin?.trigger("push-token", data: ["token": hex])

    // Call original only if it was swapped (not added)
    if responds(to: #selector(ta_application(_:didRegisterForRemoteNotificationsWithDeviceToken:))) {
      self.ta_application(application, didRegisterForRemoteNotificationsWithDeviceToken: deviceToken)
    }
  }

  // Token failure
  @objc func ta_application(_ application: UIApplication,
                            didFailToRegisterForRemoteNotificationsWithError error: Error) {
    // Notify plugin about error
    AppDelegateSwizzler.plugin?.handlePushTokenError(error)

    // Also emit event for JS/Rust listeners
    try? AppDelegateSwizzler.plugin?.trigger("push-error", data: ["message": error.localizedDescription])

    // Call original only if it was swapped (not added)
    if responds(to: #selector(ta_application(_:didFailToRegisterForRemoteNotificationsWithError:))) {
      self.ta_application(application, didFailToRegisterForRemoteNotificationsWithError: error)
    }
  }

  // Background/remote (silent) payloads
  @objc func ta_application(_ application: UIApplication,
                            didReceiveRemoteNotification userInfo: [AnyHashable : Any],
                            fetchCompletionHandler completion: @escaping (UIBackgroundFetchResult) -> Void) {
    // Emit event for push message
    if let jsData = JSTypes.coerceDictionaryToJSObject(userInfo) {
      try? AppDelegateSwizzler.plugin?.trigger("push-message", data: jsData)
    }

    // Call original only if it was swapped (not added)
    if responds(to: #selector(ta_application(_:didReceiveRemoteNotification:fetchCompletionHandler:))) {
      self.ta_application(application, didReceiveRemoteNotification: userInfo, fetchCompletionHandler: completion)
    } else {
      // If no original implementation, we should still call the completion handler
      completion(.noData)
    }
  }
}

#endif

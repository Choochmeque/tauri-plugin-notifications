import UIKit
import Tauri
import ObjectiveC.runtime

#if ENABLE_PUSH_NOTIFICATIONS

enum AppDelegateSwizzler {
  static weak var plugin: NotificationPlugin?

  private static var originalIMPs: [Selector: IMP] = [:]
  private static var swizzled = false

  static func swizzlePushCallbacks() {
    guard !swizzled else { return }
    guard let delegate = UIApplication.shared.delegate else { return }
    swizzled = true
    let cls: AnyClass = type(of: delegate)

    install(
      cls,
      #selector(UIApplicationDelegate.application(_:didRegisterForRemoteNotificationsWithDeviceToken:)),
      #selector(PushForwarder.ta_application(_:didRegisterForRemoteNotificationsWithDeviceToken:))
    )
    install(
      cls,
      #selector(UIApplicationDelegate.application(_:didFailToRegisterForRemoteNotificationsWithError:)),
      #selector(PushForwarder.ta_application(_:didFailToRegisterForRemoteNotificationsWithError:))
    )
    install(
      cls,
      #selector(UIApplicationDelegate.application(_:didReceiveRemoteNotification:fetchCompletionHandler:)),
      #selector(PushForwarder.ta_application(_:didReceiveRemoteNotification:fetchCompletionHandler:))
    )
  }

  /// Installs the forwarder, keeping the delegate's own implementation so it can still be chained.
  private static func install(_ cls: AnyClass, _ selector: Selector, _ replacement: Selector) {
    guard let replacementMethod = class_getInstanceMethod(PushForwarder.self, replacement) else { return }
    let replacementIMP = method_getImplementation(replacementMethod)
    let typeEncoding = method_getTypeEncoding(replacementMethod)
    if let previousIMP = class_replaceMethod(cls, selector, replacementIMP, typeEncoding) {
      originalIMPs[selector] = previousIMP
    }
  }

  fileprivate static func callOriginalDidRegister(
    _ receiver: Any, _ selector: Selector, _ application: UIApplication, _ deviceToken: Data
  ) {
    guard let imp = originalIMPs[selector] else { return }
    typealias Fn = @convention(c) (Any, Selector, UIApplication, Data) -> Void
    unsafeBitCast(imp, to: Fn.self)(receiver, selector, application, deviceToken)
  }

  fileprivate static func callOriginalDidFail(
    _ receiver: Any, _ selector: Selector, _ application: UIApplication, _ error: Error
  ) {
    guard let imp = originalIMPs[selector] else { return }
    typealias Fn = @convention(c) (Any, Selector, UIApplication, Error) -> Void
    unsafeBitCast(imp, to: Fn.self)(receiver, selector, application, error)
  }

  fileprivate static func callOriginalDidReceive(
    _ receiver: Any, _ selector: Selector, _ application: UIApplication,
    _ userInfo: [AnyHashable: Any], _ completion: @escaping (UIBackgroundFetchResult) -> Void
  ) -> Bool {
    guard let imp = originalIMPs[selector] else { return false }
    typealias Fn = @convention(c) (Any, Selector, UIApplication, [AnyHashable: Any], @escaping (UIBackgroundFetchResult) -> Void) -> Void
    unsafeBitCast(imp, to: Fn.self)(receiver, selector, application, userInfo, completion)
    return true
  }
}

/// Hosts the implementations installed onto the app delegate class. At call time
/// `self` is the app delegate, so these may only touch params and static state.
final class PushForwarder: NSObject, UIApplicationDelegate {
  @objc func ta_application(_ application: UIApplication,
                            didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
    let hex = deviceToken.map { String(format: "%02x", $0) }.joined()
    AppDelegateSwizzler.plugin?.handlePushTokenReceived(hex)
    try? AppDelegateSwizzler.plugin?.trigger("push-token", data: ["token": hex])
    AppDelegateSwizzler.callOriginalDidRegister(
      self,
      #selector(UIApplicationDelegate.application(_:didRegisterForRemoteNotificationsWithDeviceToken:)),
      application,
      deviceToken
    )
  }

  @objc func ta_application(_ application: UIApplication,
                            didFailToRegisterForRemoteNotificationsWithError error: Error) {
    AppDelegateSwizzler.plugin?.handlePushTokenError(error)
    try? AppDelegateSwizzler.plugin?.trigger("push-error", data: ["message": error.localizedDescription])
    AppDelegateSwizzler.callOriginalDidFail(
      self,
      #selector(UIApplicationDelegate.application(_:didFailToRegisterForRemoteNotificationsWithError:)),
      application,
      error
    )
  }

  @objc func ta_application(_ application: UIApplication,
                            didReceiveRemoteNotification userInfo: [AnyHashable : Any],
                            fetchCompletionHandler completion: @escaping (UIBackgroundFetchResult) -> Void) {
    if let jsData = JSTypes.coerceDictionaryToJSObject(userInfo) {
      try? AppDelegateSwizzler.plugin?.trigger("push-message", data: jsData)
    }
    let chained = AppDelegateSwizzler.callOriginalDidReceive(
      self,
      #selector(UIApplicationDelegate.application(_:didReceiveRemoteNotification:fetchCompletionHandler:)),
      application,
      userInfo,
      completion
    )
    if !chained {
      completion(.noData)
    }
  }
}

#endif

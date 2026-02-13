import UserNotifications
import Intents

class NotificationService: UNNotificationServiceExtension {
  private var contentHandler: ((UNNotificationContent) -> Void)?
  private var bestAttemptContent: UNMutableNotificationContent?

  override func didReceive(
    _ request: UNNotificationRequest,
    withContentHandler contentHandler: @escaping (UNNotificationContent) -> Void
  ) {
    self.contentHandler = contentHandler
    self.bestAttemptContent = (request.content.mutableCopy() as? UNMutableNotificationContent)

    guard let bestAttemptContent else {
      contentHandler(request.content)
      return
    }

    guard #available(iOS 15.0, *) else {
      contentHandler(bestAttemptContent)
      return
    }

    let userInfo = request.content.userInfo
    let senderName = userInfo["sender_name"] as? String ?? bestAttemptContent.title
    let senderId = userInfo["sender_id"] as? String ?? senderName
    let senderAvatarUrl = userInfo["sender_avatar_url"] as? String
    let conversationId = userInfo["conversation_id"] as? String

    let senderImage = loadImage(urlString: senderAvatarUrl)
    let sender = INPerson(
      personHandle: INPersonHandle(value: senderId, type: .unknown),
      nameComponents: nil,
      displayName: senderName,
      image: senderImage,
      contactIdentifier: senderId,
      customIdentifier: senderId,
      isMe: false,
      suggestionType: .none
    )

    let intent = INSendMessageIntent(
      recipients: nil,
      outgoingMessageType: .outgoingMessageText,
      content: bestAttemptContent.body,
      speakableGroupName: nil,
      conversationIdentifier: conversationId,
      serviceName: nil,
      sender: sender,
      attachments: nil
    )

    let interaction = INInteraction(intent: intent, response: nil)
    interaction.direction = .incoming
    interaction.donate { [weak self] error in
      guard let self else { return }
      if error != nil {
        self.contentHandler?(bestAttemptContent)
        return
      }
      do {
        let messageContent = try request.content.updating(from: intent)
        self.contentHandler?(messageContent)
      } catch {
        self.contentHandler?(bestAttemptContent)
      }
    }
  }

  override func serviceExtensionTimeWillExpire() {
    if let contentHandler, let bestAttemptContent {
      contentHandler(bestAttemptContent)
    }
  }

  @available(iOS 15.0, *)
  private func loadImage(urlString: String?) -> INImage? {
    guard let urlString, let url = URL(string: urlString) else {
      return nil
    }

    let semaphore = DispatchSemaphore(value: 0)
    var imageData: Data?

    let task = URLSession.shared.dataTask(with: url) { data, _, _ in
      imageData = data
      semaphore.signal()
    }
    task.resume()

    _ = semaphore.wait(timeout: .now() + 2.0)
    if let imageData {
      return INImage(imageData: imageData)
    }
    return nil
  }
}

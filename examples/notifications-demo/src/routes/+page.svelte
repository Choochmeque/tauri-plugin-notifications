<script lang="ts">
  import {
    sendNotification,
    requestPermission,
    isPermissionGranted,
    registerActionTypes,
    registerForPushNotifications,
    unregisterForPushNotifications,
    pending,
    cancel,
    cancelAll,
    active,
    removeActive,
    removeAllActive,
    createChannel,
    removeChannel,
    channels,
    onNotificationReceived,
    onAction,
    onNotificationClicked,
    Schedule,
    ScheduleEvery,
    Importance,
    Visibility,
    type Channel,
    type PendingNotification,
    type ActiveNotification,
    type NotificationClickedData,
  } from "@choochmeque/tauri-plugin-notifications-api";
  import { onMount } from "svelte";

  // ============================================================================
  // STATE MANAGEMENT
  // ============================================================================

  let permissionGranted = $state(false);
  let permissionStatus = $state("Unknown");
  let logs = $state<string[]>([]);
  let pendingNotifications = $state<PendingNotification[]>([]);
  let activeNotifications = $state<ActiveNotification[]>([]);
  let notificationChannels = $state<Channel[]>([]);

  // Form states for different notification types
  let basicTitle = $state("Hello from Tauri!");
  let basicBody = $state("This is a notification message.");
  let scheduledTitle = $state("Scheduled Notification");
  let scheduledBody = $state("This notification was scheduled!");
  let scheduleDelay = $state(5); // seconds

  // Channel creation
  let newChannelId = $state("app-channel");
  let newChannelName = $state("App Notifications");
  let newChannelDescription = $state("General app notifications");

  // Push notifications
  let pushToken = $state<string | null>(null);
  let pushRegistered = $state(false);

  // ============================================================================
  // UTILITY FUNCTIONS
  // ============================================================================

  /**
   * Adds a log entry to the display
   */
  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString();
    logs = [`[${timestamp}] ${message}`, ...logs].slice(0, 50); // Keep last 50 logs
  }

  /**
   * Refreshes the list of pending notifications
   */
  async function refreshPending() {
    try {
      pendingNotifications = await pending();
      addLog(`Loaded ${pendingNotifications.length} pending notifications`);
    } catch (error) {
      addLog(`Error loading pending: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Refreshes the list of active notifications
   */
  async function refreshActive() {
    try {
      activeNotifications = await active();
      addLog(`Loaded ${activeNotifications.length} active notifications`);
    } catch (error) {
      addLog(`Error loading active: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Refreshes the list of notification channels
   */
  async function refreshChannels() {
    try {
      notificationChannels = await channels();
      addLog(`Loaded ${notificationChannels.length} channels`);
    } catch (error) {
      addLog(`Error loading channels: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  // ============================================================================
  // PERMISSION MANAGEMENT
  // ============================================================================

  /**
   * Checks if notification permission is already granted
   */
  async function checkPermission() {
    try {
      permissionGranted = await isPermissionGranted();
      permissionStatus = permissionGranted ? "Granted" : "Not Granted";
      addLog(`Permission status: ${permissionStatus}`);
    } catch (error) {
      addLog(`Error checking permission: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
      permissionStatus = "Error";
    }
  }

  /**
   * Requests notification permission from the user
   */
  async function handleRequestPermission() {
    try {
      const result = await requestPermission();
      permissionGranted = result === "granted";
      permissionStatus = result;
      addLog(`Permission request result: ${result}`);
    } catch (error) {
      addLog(`Error requesting permission: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  // ============================================================================
  // PUSH NOTIFICATIONS
  // ============================================================================

  /**
   * Registers the device for push notifications and retrieves the push token
   */
  async function handleRegisterPush() {
    try {
      const token = await registerForPushNotifications();
      pushToken = token;
      pushRegistered = true;
      addLog(`✅ Registered for push notifications`);
      addLog(`📱 Push token: ${token.substring(0, 20)}...`);
    } catch (error) {
      addLog(`❌ Error registering for push: ${error}`);
    }
  }

  /**
   * Unregisters the device from push notifications
   */
  async function handleUnregisterPush() {
    try {
      await unregisterForPushNotifications();
      pushToken = null;
      pushRegistered = false;
      addLog(`❌ Unregistered from push notifications`);
    } catch (error) {
      addLog(`Error unregistering from push: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Copies the push token to clipboard
   */
  async function copyPushToken() {
    if (!pushToken) {
      addLog("No push token available");
      return;
    }

    try {
      await navigator.clipboard.writeText(pushToken);
      addLog(`📋 Push token copied to clipboard`);
    } catch (error) {
      addLog(`Error copying to clipboard: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  // ============================================================================
  // BASIC NOTIFICATIONS
  // ============================================================================

  /**
   * Sends a simple notification with title and body
   */
  async function handleSendBasic() {
    if (!permissionGranted) {
      addLog("Permission not granted. Please request permission first.");
      return;
    }

    try {
      await sendNotification({
        title: basicTitle,
        body: basicBody,
      });
      addLog(`Sent notification: "${basicTitle}"`);
    } catch (error) {
      addLog(`Error sending notification: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Sends a notification with a custom ID for later reference
   */
  async function handleSendWithId() {
    if (!permissionGranted) {
      addLog("Permission not granted. Please request permission first.");
      return;
    }

    try {
      const notificationId = Math.floor(Math.random() * 1000000);
      await sendNotification({
        id: notificationId,
        title: "Notification with ID",
        body: `This notification has ID: ${notificationId}`,
      });
      addLog(`Sent notification with ID: ${notificationId}`);
    } catch (error) {
      addLog(`Error sending notification: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  // ============================================================================
  // SCHEDULED NOTIFICATIONS
  // ============================================================================

  /**
   * Schedules a notification to appear after a delay
   */
  async function handleScheduleDelayed() {
    if (!permissionGranted) {
      addLog("Permission not granted. Please request permission first.");
      return;
    }

    try {
      const scheduledDate = new Date();
      scheduledDate.setSeconds(scheduledDate.getSeconds() + scheduleDelay);

      await sendNotification({
        title: scheduledTitle,
        body: scheduledBody,
        schedule: Schedule.at(scheduledDate),
      });
      addLog(
        `Scheduled notification for ${scheduleDelay} seconds from now (${scheduledDate.toLocaleTimeString()})`,
      );
      await refreshPending();
    } catch (error) {
      addLog(`Error scheduling notification: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Schedules a recurring notification that fires every minute
   */
  async function handleScheduleRecurring() {
    if (!permissionGranted) {
      addLog("Permission not granted. Please request permission first.");
      return;
    }

    try {
      await sendNotification({
        title: "Recurring Notification",
        body: "This notification repeats every minute",
        schedule: Schedule.every(ScheduleEvery.Minute, 1),
      });
      addLog("Scheduled recurring notification (every minute)");
      await refreshPending();
    } catch (error) {
      addLog(`Error scheduling recurring notification: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  // ============================================================================
  // INTERACTIVE NOTIFICATIONS (with actions)
  // ============================================================================

  /**
   * Registers action types and sends a notification with action buttons
   */
  async function handleSendWithActions() {
    if (!permissionGranted) {
      addLog("Permission not granted. Please request permission first.");
      return;
    }

    try {
      // First, register the action types
      await registerActionTypes([
        {
          id: "demo-actions",
          actions: [
            {
              id: "accept",
              title: "Accept",
              foreground: true,
            },
            {
              id: "decline",
              title: "Decline",
              destructive: true,
            },
            {
              id: "reply",
              title: "Reply",
              input: true,
              inputButtonTitle: "Send",
              inputPlaceholder: "Type your reply...",
            },
          ],
        },
      ]);

      // Send notification with the action type
      await sendNotification({
        title: "Interactive Notification",
        body: "This notification has action buttons!",
        actionTypeId: "demo-actions",
      });
      addLog("Sent notification with action buttons");
    } catch (error) {
      addLog(`Error sending interactive notification: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  // ============================================================================
  // NOTIFICATION STYLES
  // ============================================================================

  /**
   * Sends a notification with large body text (Big Text style on Android)
   */
  async function handleSendLargeBody() {
    if (!permissionGranted) {
      addLog("Permission not granted. Please request permission first.");
      return;
    }

    try {
      await sendNotification({
        title: "Large Body Notification",
        body: "Short preview...",
        largeBody:
          "This is a much longer notification body that demonstrates the large text style. " +
          "On Android, this will expand to show all this text. It's useful for detailed messages " +
          "that need more space than a standard notification provides.",
        summary: "Detailed information",
      });
      addLog("Sent notification with large body text");
    } catch (error) {
      addLog(`Error sending large body notification: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Sends a notification with inbox style (multiple lines)
   */
  async function handleSendInbox() {
    if (!permissionGranted) {
      addLog("Permission not granted. Please request permission first.");
      return;
    }

    try {
      await sendNotification({
        title: "Inbox Style Notification",
        body: "You have 5 new messages",
        inboxLines: [
          "Alice: Hey, are you there?",
          "Bob: Meeting at 3pm",
          "Carol: Check out this link",
          "Dave: Great work today!",
          "Eve: Don't forget the report",
        ],
        summary: "5 messages",
      });
      addLog("Sent inbox-style notification");
    } catch (error) {
      addLog(`Error sending inbox notification: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  // ============================================================================
  // CHANNEL MANAGEMENT (Android)
  // ============================================================================

  /**
   * Creates a new notification channel
   */
  async function handleCreateChannel() {
    try {
      await createChannel({
        id: newChannelId,
        name: newChannelName,
        description: newChannelDescription,
        importance: Importance.Default,
        visibility: Visibility.Public,
        lights: true,
        vibration: true,
      });
      addLog(`Created channel: ${newChannelName} (${newChannelId})`);
      await refreshChannels();
    } catch (error) {
      addLog(`Error creating channel: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Sends a notification to a specific channel
   */
  async function handleSendToChannel() {
    if (!permissionGranted) {
      addLog("Permission not granted. Please request permission first.");
      return;
    }

    if (notificationChannels.length === 0) {
      addLog("No channels available. Create a channel first.");
      return;
    }

    try {
      const channelId = notificationChannels[0].id;
      await sendNotification({
        title: "Channel Notification",
        body: `Sent to channel: ${channelId}`,
        channelId: channelId,
      });
      addLog(`Sent notification to channel: ${channelId}`);
    } catch (error) {
      addLog(`Error sending to channel: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Deletes a notification channel
   */
  async function handleDeleteChannel(channelId: string) {
    try {
      await removeChannel(channelId);
      addLog(`Deleted channel: ${channelId}`);
      await refreshChannels();
    } catch (error) {
      addLog(`Error deleting channel: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  // ============================================================================
  // NOTIFICATION MANAGEMENT
  // ============================================================================

  /**
   * Cancels a specific pending notification
   */
  async function handleCancelPending(id: number) {
    try {
      await cancel([id]);
      addLog(`Cancelled pending notification: ${id}`);
      await refreshPending();
    } catch (error) {
      addLog(`Error cancelling notification: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Cancels all pending notifications
   */
  async function handleCancelAllPending() {
    try {
      await cancelAll();
      addLog("Cancelled all pending notifications");
      await refreshPending();
    } catch (error) {
      addLog(`Error cancelling all: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Removes a specific active notification
   */
  async function handleRemoveActive(id: number, tag?: string) {
    try {
      await removeActive([{ id, tag }]);
      addLog(`Removed active notification: ${id}`);
      await refreshActive();
    } catch (error) {
      addLog(`Error removing active notification: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Removes all active notifications
   */
  async function handleRemoveAllActive() {
    try {
      await removeAllActive();
      addLog("Removed all active notifications");
      await refreshActive();
    } catch (error) {
      addLog(`Error removing all active: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  }

  /**
   * Clears the log display
   */
  function clearLogs() {
    logs = [];
    addLog("Logs cleared");
  }

  // ============================================================================
  // LIFECYCLE & EVENT LISTENERS
  // ============================================================================

  onMount(async () => {
    addLog("Demo app initialized");

    // Check initial permission status
    await checkPermission();
    // Ask for permission on launch if needed, then register for push on iOS
    if (!permissionGranted) {
      await handleRequestPermission();
    }
    if (permissionGranted) {
      await handleRegisterPush();
    }

    // Load initial data
    await refreshPending();
    await refreshActive();
    await refreshChannels();

    // Register event listeners
    try {
      // Listen for incoming notifications
      const unlistenReceived = await onNotificationReceived((notification) => {
        addLog(
          `📨 Notification received: ${notification.title || "No title"}`,
        );
        refreshActive();
      });

      // Listen for action button clicks
      const unlistenAction = await onAction((data) => {
        addLog(`🔘 Action performed - actionId: ${data.actionId}, title: ${data.notification?.title || "No title"}, input: ${data.inputValue || "none"}`);
      });

      // Listen for notification clicks/taps
      const unlistenClicked = await onNotificationClicked(
        (data: NotificationClickedData) => {
          addLog(
            `👆 Notification clicked - ID: ${data.id}, Data: ${JSON.stringify(data.data)}`,
          );
        },
      );

      addLog("Event listeners registered");

      // Cleanup on unmount
      return () => {
        unlistenReceived.unregister();
        unlistenAction.unregister();
        unlistenClicked.unregister();
      };
    } catch (error) {
      addLog(`Error setting up event listeners: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    }
  });
</script>

<!-- ============================================================================ -->
<!-- MAIN UI -->
<!-- ============================================================================ -->

<main class="container">
  <header>
    <h1>🔔 Tauri Notifications Plugin Demo</h1>
    <p class="subtitle">
      Demo showcasing all features of the notifications plugin
    </p>
  </header>

  <div class="content">
    <!-- ====================================================================== -->
    <!-- PERMISSION SECTION -->
    <!-- ====================================================================== -->
    <section class="card">
      <h2>📋 1. Permission Management</h2>
      <p class="description">
        Before sending notifications, you need to request permission from the
        user. This is required on all platforms.
      </p>

      <div class="status-badge" class:granted={permissionGranted}>
        Status: {permissionStatus}
      </div>

      <div class="button-group">
        <button onclick={checkPermission}>Check Permission</button>
        <button onclick={handleRequestPermission} class="primary">
          Request Permission
        </button>
      </div>
    </section>

    <!-- ====================================================================== -->
    <!-- PUSH NOTIFICATIONS -->
    <!-- ====================================================================== -->
    <section class="card">
      <h2>📱 2. Push Notifications</h2>
      <p class="description">
        Register for push notifications to receive remote notifications via FCM
        (Android) or APNs (iOS/macOS).
      </p>

      <div class="status-badge" class:granted={pushRegistered}>
        Push Status: {pushRegistered ? "Registered" : "Not Registered"}
      </div>

      <div class="button-group">
        <button onclick={handleRegisterPush} class="primary">
          Register for Push
        </button>
        <button onclick={handleUnregisterPush} class:danger={pushRegistered}>
          Unregister
        </button>
      </div>

      {#if pushToken}
        <div class="push-token-container">
          <h3>📱 Device Push Token</h3>
          <div class="token-display">
            <code>{pushToken}</code>
          </div>
          <button onclick={copyPushToken} class="copy-button">
            📋 Copy Token
          </button>
          <div class="info-box" style="margin-top: 1rem;">
            <strong>How to test:</strong>
            <ol style="margin: 0.5rem 0 0; padding-left: 1.5rem;">
              <li>Copy this token using the button above</li>
              <li>
                Send a test push notification from your FCM/APNs console using
                this token
              </li>
              <li>The notification will appear even when the app is closed</li>
              <li>
                Check the Event Log when you tap the notification to see the
                data
              </li>
            </ol>
          </div>
        </div>
      {:else}
        <div class="info-box">
          <strong>Note:</strong> Push notifications require additional setup:
          <ul style="margin: 0.5rem 0 0; padding-left: 1.5rem;">
            <li><strong>Android:</strong> Configure Firebase Cloud Messaging</li>
            <li><strong>iOS/macOS:</strong> Configure Apple Push Notification service</li>
            <li>
              <strong>Linux/Windows:</strong> Push notifications are not supported
            </li>
          </ul>
        </div>
      {/if}

      <div class="code-example">
        <code>
          // Register<br />
          const token = await registerForPushNotifications();<br />
          <br />
          // Unregister<br />
          await unregisterForPushNotifications();
        </code>
      </div>
    </section>

    <!-- ====================================================================== -->
    <!-- BASIC NOTIFICATIONS -->
    <!-- ====================================================================== -->
    <section class="card">
      <h2>📢 3. Basic Notifications</h2>
      <p class="description">
        Send simple notifications with a title and body message.
      </p>

      <div class="form-group">
        <label for="basic-title">Title</label>
        <input id="basic-title" type="text" bind:value={basicTitle} />
      </div>

      <div class="form-group">
        <label for="basic-body">Body</label>
        <textarea id="basic-body" bind:value={basicBody} rows="2"></textarea>
      </div>

      <div class="button-group">
        <button onclick={handleSendBasic} class="primary">
          Send Notification
        </button>
        <button onclick={handleSendWithId}>Send with Custom ID</button>
      </div>

      <div class="code-example">
        <code>
          await sendNotification(&#123;<br />
          &nbsp;&nbsp;title: "{basicTitle}",<br />
          &nbsp;&nbsp;body: "{basicBody}"<br />
          &#125;);
        </code>
      </div>
    </section>

    <!-- ====================================================================== -->
    <!-- SCHEDULED NOTIFICATIONS -->
    <!-- ====================================================================== -->
    <section class="card">
      <h2>⏰ 4. Scheduled Notifications</h2>
      <p class="description">
        Schedule notifications to appear at a specific time or on a recurring
        interval.
      </p>

      <div class="form-group">
        <label for="scheduled-title">Title</label>
        <input id="scheduled-title" type="text" bind:value={scheduledTitle} />
      </div>

      <div class="form-group">
        <label for="scheduled-body">Body</label>
        <input id="scheduled-body" type="text" bind:value={scheduledBody} />
      </div>

      <div class="form-group">
        <label for="schedule-delay">Delay (seconds)</label>
        <input
          id="schedule-delay"
          type="number"
          bind:value={scheduleDelay}
          min="1"
          max="3600"
        />
      </div>

      <div class="button-group">
        <button onclick={handleScheduleDelayed} class="primary">
          Schedule Delayed
        </button>
        <button onclick={handleScheduleRecurring}>Schedule Recurring</button>
      </div>

      <div class="code-example">
        <code>
          // Delayed<br />
          schedule: Schedule.at(futureDate)<br />
          <br />
          // Recurring<br />
          schedule: Schedule.every(ScheduleEvery.Minute, 1)
        </code>
      </div>
    </section>

    <!-- ====================================================================== -->
    <!-- NOTIFICATION STYLES -->
    <!-- ====================================================================== -->
    <section class="card">
      <h2>🎨 5. Notification Styles</h2>
      <p class="description">
        Different notification styles for rich content display (Android/iOS
        features).
      </p>

      <div class="button-group">
        <button onclick={handleSendLargeBody}>Large Body Text</button>
        <button onclick={handleSendInbox}>Inbox Style (Lines)</button>
        <button onclick={handleSendWithActions}>With Action Buttons</button>
      </div>

      <div class="info-box">
        <strong>Note:</strong> Some styles are platform-specific. Large body and
        inbox styles are primarily for Android.
      </div>
    </section>

    <!-- ====================================================================== -->
    <!-- CHANNELS (Android) -->
    <!-- ====================================================================== -->
    <section class="card">
      <h2>📺 6. Notification Channels (Android)</h2>
      <p class="description">
        Channels allow users to manage notification preferences by category.
        Create channels before sending notifications to them.
      </p>

      <div class="form-row">
        <div class="form-group">
          <label for="channel-id">Channel ID</label>
          <input id="channel-id" type="text" bind:value={newChannelId} />
        </div>
        <div class="form-group">
          <label for="channel-name">Channel Name</label>
          <input id="channel-name" type="text" bind:value={newChannelName} />
        </div>
      </div>

      <div class="form-group">
        <label for="channel-desc">Description</label>
        <input
          id="channel-desc"
          type="text"
          bind:value={newChannelDescription}
        />
      </div>

      <div class="button-group">
        <button onclick={handleCreateChannel} class="primary">
          Create Channel
        </button>
        <button onclick={handleSendToChannel}>Send to Channel</button>
        <button onclick={refreshChannels}>Refresh List</button>
      </div>

      {#if notificationChannels.length > 0}
        <div class="list-container">
          <h3>Available Channels ({notificationChannels.length})</h3>
          {#each notificationChannels as channel}
            <div class="list-item">
              <div class="list-item-content">
                <strong>{channel.name}</strong>
                <span class="channel-id">{channel.id}</span>
                {#if channel.description}
                  <p class="channel-desc">{channel.description}</p>
                {/if}
              </div>
              <button
                onclick={() => handleDeleteChannel(channel.id)}
                class="danger-small"
              >
                Delete
              </button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty-state">No channels created yet</p>
      {/if}
    </section>

    <!-- ====================================================================== -->
    <!-- PENDING NOTIFICATIONS -->
    <!-- ====================================================================== -->
    <section class="card">
      <h2>📅 7. Pending Notifications</h2>
      <p class="description">
        View and manage scheduled notifications that haven't been delivered yet.
      </p>

      <div class="button-group">
        <button onclick={refreshPending}>Refresh Pending</button>
        <button onclick={handleCancelAllPending} class="danger">
          Cancel All
        </button>
      </div>

      {#if pendingNotifications.length > 0}
        <div class="list-container">
          <h3>Pending Notifications ({pendingNotifications.length})</h3>
          {#each pendingNotifications as notification}
            <div class="list-item">
              <div class="list-item-content">
                <strong>{notification.title || "No title"}</strong>
                <span class="notification-id">ID: {notification.id}</span>
                {#if notification.body}
                  <p>{notification.body}</p>
                {/if}
              </div>
              <button
                onclick={() => handleCancelPending(notification.id)}
                class="danger-small"
              >
                Cancel
              </button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty-state">No pending notifications</p>
      {/if}
    </section>

    <!-- ====================================================================== -->
    <!-- ACTIVE NOTIFICATIONS -->
    <!-- ====================================================================== -->
    <section class="card">
      <h2>🔴 8. Active Notifications</h2>
      <p class="description">
        View and dismiss notifications currently displayed in the notification
        center.
      </p>

      <div class="button-group">
        <button onclick={refreshActive}>Refresh Active</button>
        <button onclick={handleRemoveAllActive} class="danger">
          Remove All
        </button>
      </div>

      {#if activeNotifications.length > 0}
        <div class="list-container">
          <h3>Active Notifications ({activeNotifications.length})</h3>
          {#each activeNotifications as notification}
            <div class="list-item">
              <div class="list-item-content">
                <strong>{notification.title || "No title"}</strong>
                <span class="notification-id">ID: {notification.id}</span>
                {#if notification.body}
                  <p>{notification.body}</p>
                {/if}
              </div>
              <button
                onclick={() =>
                  handleRemoveActive(notification.id, notification.tag)}
                class="danger-small"
              >
                Remove
              </button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty-state">No active notifications</p>
      {/if}
    </section>

    <!-- ====================================================================== -->
    <!-- EVENT LOG -->
    <!-- ====================================================================== -->
    <section class="card">
      <h2>📊 9. Event Log</h2>
      <p class="description">
        Real-time log of notification events, API calls, and user actions.
      </p>

      <div class="button-group">
        <button onclick={clearLogs}>Clear Logs</button>
      </div>

      <div class="log-container">
        {#if logs.length > 0}
          {#each logs as log}
            <div class="log-entry">{log}</div>
          {/each}
        {:else}
          <p class="empty-state">No logs yet</p>
        {/if}
      </div>
    </section>
  </div>

  <footer>
    <p>
      📚 <a
        href="https://github.com/Choochmeque/tauri-plugin-notifications"
        target="_blank">Documentation</a
      >
      | Built with Tauri + SvelteKit
    </p>
  </footer>
</main>

<!-- ============================================================================ -->
<!-- STYLES -->
<!-- ============================================================================ -->

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family:
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      Roboto,
      Oxygen,
      Ubuntu,
      Cantarell,
      sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
  }

  .container {
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }

  header {
    text-align: center;
    color: white;
    margin-bottom: 2rem;
  }

  h1 {
    margin: 0;
    font-size: 2.5rem;
    font-weight: 700;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .subtitle {
    margin: 0.5rem 0 0;
    font-size: 1.1rem;
    opacity: 0.95;
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .card {
    background: white;
    border-radius: 12px;
    padding: 1.5rem;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .card h2 {
    margin: 0 0 0.5rem;
    font-size: 1.5rem;
    color: #333;
  }

  .description {
    margin: 0 0 1rem;
    color: #666;
    line-height: 1.5;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #555;
  }

  input,
  textarea {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid #e0e0e0;
    border-radius: 6px;
    font-size: 1rem;
    transition: border-color 0.2s;
    box-sizing: border-box;
  }

  input:focus,
  textarea:focus {
    outline: none;
    border-color: #667eea;
  }

  .button-group {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    background: #f0f0f0;
    color: #333;
  }

  button:hover {
    transform: translateY(-1px);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  button.primary {
    background: #667eea;
    color: white;
  }

  button.primary:hover {
    background: #5568d3;
  }

  button.danger {
    background: #ef4444;
    color: white;
  }

  button.danger:hover {
    background: #dc2626;
  }

  button.danger-small {
    padding: 0.5rem 1rem;
    background: #fee;
    color: #dc2626;
    font-size: 0.875rem;
  }

  button.danger-small:hover {
    background: #fcc;
  }

  .status-badge {
    display: inline-block;
    padding: 0.5rem 1rem;
    border-radius: 20px;
    font-weight: 600;
    margin-bottom: 1rem;
    background: #fee;
    color: #dc2626;
  }

  .status-badge.granted {
    background: #d1fae5;
    color: #065f46;
  }

  .code-example {
    background: #f8f9fa;
    border-left: 4px solid #667eea;
    padding: 1rem;
    border-radius: 4px;
    margin-top: 1rem;
  }

  .code-example code {
    font-family: "Monaco", "Courier New", monospace;
    font-size: 0.875rem;
    color: #333;
    line-height: 1.6;
  }

  .info-box {
    background: #e0f2fe;
    border-left: 4px solid #0284c7;
    padding: 1rem;
    border-radius: 4px;
    margin-top: 1rem;
    color: #0c4a6e;
  }

  .push-token-container {
    margin-top: 1rem;
  }

  .push-token-container h3 {
    margin: 0 0 0.75rem;
    font-size: 1.1rem;
    color: #555;
  }

  .token-display {
    background: #1e1e1e;
    border-radius: 6px;
    padding: 1rem;
    margin-bottom: 0.75rem;
    overflow-x: auto;
  }

  .token-display code {
    color: #4ade80;
    font-family: "Monaco", "Courier New", monospace;
    font-size: 0.875rem;
    word-break: break-all;
  }

  .copy-button {
    background: #667eea;
    color: white;
    width: 100%;
  }

  .copy-button:hover {
    background: #5568d3;
  }

  .list-container {
    margin-top: 1rem;
  }

  .list-container h3 {
    margin: 0 0 1rem;
    font-size: 1.1rem;
    color: #555;
  }

  .list-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: #f8f9fa;
    border-radius: 6px;
    margin-bottom: 0.5rem;
  }

  .list-item-content {
    flex: 1;
  }

  .list-item-content strong {
    display: block;
    color: #333;
    margin-bottom: 0.25rem;
  }

  .list-item-content p {
    margin: 0.5rem 0 0;
    color: #666;
    font-size: 0.9rem;
  }

  .notification-id,
  .channel-id {
    display: inline-block;
    font-size: 0.75rem;
    color: #999;
    background: #e0e0e0;
    padding: 0.125rem 0.5rem;
    border-radius: 10px;
    margin-left: 0.5rem;
  }

  .channel-desc {
    font-size: 0.85rem;
    color: #777;
  }

  .empty-state {
    text-align: center;
    padding: 2rem;
    color: #999;
    font-style: italic;
  }

  .log-container {
    background: #1e1e1e;
    border-radius: 6px;
    padding: 1rem;
    max-height: 400px;
    overflow-y: auto;
    font-family: "Monaco", "Courier New", monospace;
    font-size: 0.875rem;
  }

  .log-entry {
    color: #d4d4d4;
    padding: 0.25rem 0;
    border-bottom: 1px solid #333;
  }

  .log-entry:last-child {
    border-bottom: none;
  }

  footer {
    text-align: center;
    margin-top: 2rem;
    color: white;
    opacity: 0.9;
  }

  footer a {
    color: white;
    text-decoration: underline;
  }

  @media (max-width: 768px) {
    .container {
      padding: 1rem 0.5rem;
    }

    h1 {
      font-size: 1.75rem;
    }

    .form-row {
      grid-template-columns: 1fr;
    }

    .button-group {
      flex-direction: column;
    }

    button {
      width: 100%;
    }
  }
</style>

import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock the Tauri API before imports
const mockInvoke = vi.fn();
const mockAddPluginListener = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
  addPluginListener: (...args: any[]) => mockAddPluginListener(...args),
}));

import {
  Schedule,
  ScheduleEvery,
  Importance,
  Visibility,
  sendNotification,
  isPermissionGranted,
  requestPermission,
  registerForPushNotifications,
  unregisterForPushNotifications,
  registerActionTypes,
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
} from "./index";

describe("Schedule", () => {
  describe("Schedule.at", () => {
    it("should create a schedule with date and default values", () => {
      const date = new Date("2024-12-25T10:00:00");
      const schedule = Schedule.at(date);

      expect(schedule.at).toBeDefined();
      expect(schedule.at?.date).toBe(date);
      expect(schedule.at?.repeating).toBe(false);
      expect(schedule.at?.allowWhileIdle).toBe(false);
      expect(schedule.interval).toBeUndefined();
      expect(schedule.every).toBeUndefined();
    });

    it("should create a repeating schedule", () => {
      const date = new Date("2024-12-25T10:00:00");
      const schedule = Schedule.at(date, true);

      expect(schedule.at).toBeDefined();
      expect(schedule.at?.repeating).toBe(true);
      expect(schedule.at?.allowWhileIdle).toBe(false);
    });

    it("should create a schedule with allowWhileIdle", () => {
      const date = new Date("2024-12-25T10:00:00");
      const schedule = Schedule.at(date, false, true);

      expect(schedule.at).toBeDefined();
      expect(schedule.at?.repeating).toBe(false);
      expect(schedule.at?.allowWhileIdle).toBe(true);
    });

    it("should create a repeating schedule with allowWhileIdle", () => {
      const date = new Date("2024-12-25T10:00:00");
      const schedule = Schedule.at(date, true, true);

      expect(schedule.at).toBeDefined();
      expect(schedule.at?.date).toBe(date);
      expect(schedule.at?.repeating).toBe(true);
      expect(schedule.at?.allowWhileIdle).toBe(true);
      expect(schedule.interval).toBeUndefined();
      expect(schedule.every).toBeUndefined();
    });

    it("should preserve exact date object reference", () => {
      const date = new Date("2024-01-01T00:00:00");
      const schedule = Schedule.at(date);

      expect(schedule.at?.date).toBe(date);
    });
  });

  describe("Schedule.interval", () => {
    it("should create an interval schedule with default allowWhileIdle", () => {
      const interval = { hour: 10, minute: 30 };
      const schedule = Schedule.interval(interval);

      expect(schedule.interval).toBeDefined();
      expect(schedule.interval?.interval).toBe(interval);
      expect(schedule.interval?.allowWhileIdle).toBe(false);
      expect(schedule.at).toBeUndefined();
      expect(schedule.every).toBeUndefined();
    });

    it("should create an interval schedule with allowWhileIdle", () => {
      const interval = { hour: 10, minute: 30 };
      const schedule = Schedule.interval(interval, true);

      expect(schedule.interval).toBeDefined();
      expect(schedule.interval?.interval).toBe(interval);
      expect(schedule.interval?.allowWhileIdle).toBe(true);
    });

    it("should handle complex interval with all time components", () => {
      const interval = {
        year: 2024,
        month: 11,
        day: 25,
        weekday: 3,
        hour: 14,
        minute: 30,
        second: 15,
      };
      const schedule = Schedule.interval(interval);

      expect(schedule.interval?.interval).toBe(interval);
      expect(schedule.interval?.interval.year).toBe(2024);
      expect(schedule.interval?.interval.month).toBe(11);
      expect(schedule.interval?.interval.day).toBe(25);
      expect(schedule.interval?.interval.weekday).toBe(3);
      expect(schedule.interval?.interval.hour).toBe(14);
      expect(schedule.interval?.interval.minute).toBe(30);
      expect(schedule.interval?.interval.second).toBe(15);
    });

    it("should handle partial interval with only hour", () => {
      const interval = { hour: 15 };
      const schedule = Schedule.interval(interval);

      expect(schedule.interval?.interval).toEqual({ hour: 15 });
    });

    it("should preserve interval object reference", () => {
      const interval = { minute: 45 };
      const schedule = Schedule.interval(interval);

      expect(schedule.interval?.interval).toBe(interval);
    });
  });

  describe("Schedule.every", () => {
    it("should create an every schedule with default allowWhileIdle", () => {
      const schedule = Schedule.every(ScheduleEvery.Day, 1);

      expect(schedule.every).toBeDefined();
      expect(schedule.every?.interval).toBe(ScheduleEvery.Day);
      expect(schedule.every?.count).toBe(1);
      expect(schedule.every?.allowWhileIdle).toBe(false);
      expect(schedule.at).toBeUndefined();
      expect(schedule.interval).toBeUndefined();
    });

    it("should create an every schedule with allowWhileIdle", () => {
      const schedule = Schedule.every(ScheduleEvery.Hour, 2, true);

      expect(schedule.every).toBeDefined();
      expect(schedule.every?.interval).toBe(ScheduleEvery.Hour);
      expect(schedule.every?.count).toBe(2);
      expect(schedule.every?.allowWhileIdle).toBe(true);
    });

    it("should handle all ScheduleEvery enum values", () => {
      const intervals = [
        ScheduleEvery.Year,
        ScheduleEvery.Month,
        ScheduleEvery.TwoWeeks,
        ScheduleEvery.Week,
        ScheduleEvery.Day,
        ScheduleEvery.Hour,
        ScheduleEvery.Minute,
        ScheduleEvery.Second,
      ];

      intervals.forEach((interval) => {
        const schedule = Schedule.every(interval, 1);
        expect(schedule.every?.interval).toBe(interval);
      });
    });

    it("should handle different count values", () => {
      const counts = [1, 2, 5, 10, 100];

      counts.forEach((count) => {
        const schedule = Schedule.every(ScheduleEvery.Minute, count);
        expect(schedule.every?.count).toBe(count);
      });
    });

    it("should create schedule for every second", () => {
      const schedule = Schedule.every(ScheduleEvery.Second, 30);

      expect(schedule.every?.interval).toBe(ScheduleEvery.Second);
      expect(schedule.every?.count).toBe(30);
    });

    it("should create schedule for every year", () => {
      const schedule = Schedule.every(ScheduleEvery.Year, 1);

      expect(schedule.every?.interval).toBe(ScheduleEvery.Year);
      expect(schedule.every?.count).toBe(1);
    });
  });

  describe("Schedule mutual exclusivity", () => {
    it("should have only at field when using Schedule.at", () => {
      const schedule = Schedule.at(new Date());

      expect(schedule.at).toBeDefined();
      expect(schedule.interval).toBeUndefined();
      expect(schedule.every).toBeUndefined();
    });

    it("should have only interval field when using Schedule.interval", () => {
      const schedule = Schedule.interval({ hour: 10 });

      expect(schedule.interval).toBeDefined();
      expect(schedule.at).toBeUndefined();
      expect(schedule.every).toBeUndefined();
    });

    it("should have only every field when using Schedule.every", () => {
      const schedule = Schedule.every(ScheduleEvery.Day, 1);

      expect(schedule.every).toBeDefined();
      expect(schedule.at).toBeUndefined();
      expect(schedule.interval).toBeUndefined();
    });
  });
});

describe("ScheduleEvery", () => {
  it("should have correct enum values", () => {
    expect(ScheduleEvery.Year).toBe("year");
    expect(ScheduleEvery.Month).toBe("month");
    expect(ScheduleEvery.TwoWeeks).toBe("twoWeeks");
    expect(ScheduleEvery.Week).toBe("week");
    expect(ScheduleEvery.Day).toBe("day");
    expect(ScheduleEvery.Hour).toBe("hour");
    expect(ScheduleEvery.Minute).toBe("minute");
    expect(ScheduleEvery.Second).toBe("second");
  });

  it("should have exactly 8 enum values", () => {
    const values = Object.values(ScheduleEvery);
    expect(values).toHaveLength(8);
  });

  it("should contain all expected values", () => {
    const values = Object.values(ScheduleEvery);
    expect(values).toContain("year");
    expect(values).toContain("month");
    expect(values).toContain("twoWeeks");
    expect(values).toContain("week");
    expect(values).toContain("day");
    expect(values).toContain("hour");
    expect(values).toContain("minute");
    expect(values).toContain("second");
  });
});

describe("Importance", () => {
  it("should have correct enum values", () => {
    expect(Importance.None).toBe(0);
    expect(Importance.Min).toBe(1);
    expect(Importance.Low).toBe(2);
    expect(Importance.Default).toBe(3);
    expect(Importance.High).toBe(4);
  });

  it("should have sequential numeric values", () => {
    expect(Importance.Min).toBe(Importance.None + 1);
    expect(Importance.Low).toBe(Importance.Min + 1);
    expect(Importance.Default).toBe(Importance.Low + 1);
    expect(Importance.High).toBe(Importance.Default + 1);
  });

  it("should have exactly 5 importance levels", () => {
    const values = [
      Importance.None,
      Importance.Min,
      Importance.Low,
      Importance.Default,
      Importance.High,
    ];
    expect(values).toHaveLength(5);
  });

  it("should start at 0", () => {
    expect(Importance.None).toBe(0);
  });

  it("should end at 4", () => {
    expect(Importance.High).toBe(4);
  });
});

describe("Visibility", () => {
  it("should have correct enum values", () => {
    expect(Visibility.Secret).toBe(-1);
    expect(Visibility.Private).toBe(0);
    expect(Visibility.Public).toBe(1);
  });

  it("should have exactly 3 visibility levels", () => {
    const values = [Visibility.Secret, Visibility.Private, Visibility.Public];
    expect(values).toHaveLength(3);
  });

  it("should have sequential values from -1 to 1", () => {
    expect(Visibility.Secret).toBe(-1);
    expect(Visibility.Private).toBe(0);
    expect(Visibility.Public).toBe(1);
  });

  it("should have Private as middle value", () => {
    expect(Visibility.Private).toBe(0);
    expect(Visibility.Secret).toBeLessThan(Visibility.Private);
    expect(Visibility.Public).toBeGreaterThan(Visibility.Private);
  });
});

describe("Schedule edge cases", () => {
  it("should handle date with zero milliseconds", () => {
    const date = new Date("2024-01-01T00:00:00.000Z");
    const schedule = Schedule.at(date);

    expect(schedule.at?.date.getMilliseconds()).toBe(0);
  });

  it("should handle interval with zero values", () => {
    const interval = { hour: 0, minute: 0, second: 0 };
    const schedule = Schedule.interval(interval);

    expect(schedule.interval?.interval).toEqual(interval);
  });

  it("should handle every with count of zero", () => {
    const schedule = Schedule.every(ScheduleEvery.Day, 0);

    expect(schedule.every?.count).toBe(0);
  });

  it("should handle weekday boundary values (1-7)", () => {
    const interval1 = { weekday: 1 };
    const interval7 = { weekday: 7 };

    const schedule1 = Schedule.interval(interval1);
    const schedule7 = Schedule.interval(interval7);

    expect(schedule1.interval?.interval.weekday).toBe(1);
    expect(schedule7.interval?.interval.weekday).toBe(7);
  });

  it("should handle maximum time values", () => {
    const interval = {
      year: 9999,
      month: 11,
      day: 31,
      hour: 23,
      minute: 59,
      second: 59,
    };
    const schedule = Schedule.interval(interval);

    expect(schedule.interval?.interval).toEqual(interval);
  });

  it("should handle empty interval object", () => {
    const interval = {};
    const schedule = Schedule.interval(interval);

    expect(schedule.interval?.interval).toEqual({});
  });

  it("should handle future date", () => {
    const futureDate = new Date("2050-01-01T00:00:00");
    const schedule = Schedule.at(futureDate);

    expect(schedule.at?.date).toBe(futureDate);
    expect(schedule.at?.date.getFullYear()).toBe(2050);
  });

  it("should handle past date", () => {
    const pastDate = new Date("2000-01-01T00:00:00");
    const schedule = Schedule.at(pastDate);

    expect(schedule.at?.date).toBe(pastDate);
    expect(schedule.at?.date.getFullYear()).toBe(2000);
  });
});

describe("Schedule type safety", () => {
  it("should have mutually exclusive schedule types", () => {
    const atSchedule = Schedule.at(new Date());
    const intervalSchedule = Schedule.interval({ hour: 10 });
    const everySchedule = Schedule.every(ScheduleEvery.Day, 1);

    expect(atSchedule.at).toBeTruthy();
    expect(atSchedule.interval).toBeFalsy();
    expect(atSchedule.every).toBeFalsy();

    expect(intervalSchedule.interval).toBeTruthy();
    expect(intervalSchedule.at).toBeFalsy();
    expect(intervalSchedule.every).toBeFalsy();

    expect(everySchedule.every).toBeTruthy();
    expect(everySchedule.at).toBeFalsy();
    expect(everySchedule.interval).toBeFalsy();
  });

  it("should return Schedule type from all factory methods", () => {
    const atSchedule = Schedule.at(new Date());
    const intervalSchedule = Schedule.interval({ hour: 10 });
    const everySchedule = Schedule.every(ScheduleEvery.Day, 1);

    expect(atSchedule).toBeDefined();
    expect(intervalSchedule).toBeDefined();
    expect(everySchedule).toBeDefined();
  });
});

describe("Notification Functions", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
    mockAddPluginListener.mockClear();
  });

  describe("isPermissionGranted", () => {
    it("should call invoke with correct plugin command", async () => {
      mockInvoke.mockResolvedValue(true);

      const result = await isPermissionGranted();

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|is_permission_granted",
      );
      expect(result).toBe(true);
    });

    it("should return false when permission not granted", async () => {
      mockInvoke.mockResolvedValue(false);

      const result = await isPermissionGranted();

      expect(result).toBe(false);
    });
  });

  describe("requestPermission", () => {
    it("should call invoke with correct plugin command", async () => {
      mockInvoke.mockResolvedValue("granted");

      const result = await requestPermission();

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|request_permission",
      );
      expect(result).toBe("granted");
    });

    it("should return denied when permission denied", async () => {
      mockInvoke.mockResolvedValue("denied");

      const result = await requestPermission();

      expect(result).toBe("denied");
    });
  });

  describe("registerForPushNotifications", () => {
    it("should call invoke and return push token", async () => {
      const mockToken = "abc123token";
      mockInvoke.mockResolvedValue(mockToken);

      const result = await registerForPushNotifications();

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|register_for_push_notifications",
      );
      expect(result).toBe(mockToken);
    });
  });

  describe("unregisterForPushNotifications", () => {
    it("should call invoke with correct plugin command", async () => {
      mockInvoke.mockResolvedValue("");

      await unregisterForPushNotifications();

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|unregister_for_push_notifications",
      );
    });

    it("should return the result from invoke", async () => {
      const mockResult = "unregistered";
      mockInvoke.mockResolvedValue(mockResult);

      const result = await unregisterForPushNotifications();

      expect(result).toBe(mockResult);
    });
  });

  describe("sendNotification", () => {
    it("should send notification with string title", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await sendNotification("Test Title");

      expect(mockInvoke).toHaveBeenCalledWith("plugin:notifications|notify", {
        options: { title: "Test Title" },
      });
    });

    it("should send notification with full options object", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const options = {
        title: "Test",
        body: "Test body",
        id: 123,
        channelId: "test-channel",
      };

      await sendNotification(options);

      expect(mockInvoke).toHaveBeenCalledWith("plugin:notifications|notify", {
        options,
      });
    });

    it("should send notification with schedule", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const options = {
        title: "Scheduled",
        schedule: Schedule.at(new Date("2024-12-25T10:00:00")),
      };

      await sendNotification(options);

      expect(mockInvoke).toHaveBeenCalledWith("plugin:notifications|notify", {
        options,
      });
    });

    it("should send notification with all optional fields", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const options = {
        title: "Full notification",
        body: "Body text",
        largeBody: "Large body",
        summary: "Summary",
        actionTypeId: "action-1",
        group: "group-1",
        groupSummary: true,
        sound: "notification.mp3",
        inboxLines: ["Line 1", "Line 2"],
        icon: "ic_notification",
        largeIcon: "ic_large",
        iconColor: "#FF0000",
        attachments: [{ id: "att1", url: "file://image.jpg" }],
        extra: { key: "value" },
        ongoing: true,
        autoCancel: false,
        silent: true,
        visibility: Visibility.Private,
        number: 5,
      };

      await sendNotification(options);

      expect(mockInvoke).toHaveBeenCalledWith("plugin:notifications|notify", {
        options,
      });
    });
  });

  describe("registerActionTypes", () => {
    it("should register action types", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const types = [
        {
          id: "message-actions",
          actions: [
            { id: "reply", title: "Reply", input: true },
            { id: "delete", title: "Delete", destructive: true },
          ],
        },
      ];

      await registerActionTypes(types);

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|register_action_types",
        { types },
      );
    });

    it("should register action types with all optional properties", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const types = [
        {
          id: "full-actions",
          actions: [
            {
              id: "action1",
              title: "Action 1",
              requiresAuthentication: true,
              foreground: true,
              destructive: false,
              input: true,
              inputButtonTitle: "Send",
              inputPlaceholder: "Type here...",
            },
          ],
          hiddenPreviewsBodyPlaceholder: "Hidden",
          customDismissAction: true,
          allowInCarPlay: false,
          hiddenPreviewsShowTitle: true,
          hiddenPreviewsShowSubtitle: false,
        },
      ];

      await registerActionTypes(types);

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|register_action_types",
        { types },
      );
    });
  });

  describe("pending", () => {
    it("should retrieve pending notifications", async () => {
      const mockPending = [
        {
          id: 1,
          title: "Pending 1",
          body: "Body 1",
          schedule: Schedule.at(new Date()),
        },
      ];
      mockInvoke.mockResolvedValue(mockPending);

      const result = await pending();

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|get_pending",
      );
      expect(result).toEqual(mockPending);
    });

    it("should return empty array when no pending notifications", async () => {
      mockInvoke.mockResolvedValue([]);

      const result = await pending();

      expect(result).toEqual([]);
    });
  });

  describe("cancel", () => {
    it("should cancel notifications by IDs", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await cancel([1, 2, 3]);

      expect(mockInvoke).toHaveBeenCalledWith("plugin:notifications|cancel", {
        notifications: [1, 2, 3],
      });
    });

    it("should cancel single notification", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await cancel([42]);

      expect(mockInvoke).toHaveBeenCalledWith("plugin:notifications|cancel", {
        notifications: [42],
      });
    });

    it("should handle empty array", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await cancel([]);

      expect(mockInvoke).toHaveBeenCalledWith("plugin:notifications|cancel", {
        notifications: [],
      });
    });
  });

  describe("cancelAll", () => {
    it("should cancel all pending notifications", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await cancelAll();

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|cancel_all",
      );
    });
  });

  describe("active", () => {
    it("should retrieve active notifications", async () => {
      const mockActive = [
        {
          id: 1,
          title: "Active 1",
          body: "Body 1",
          groupSummary: false,
          data: {},
          extra: {},
          attachments: [],
        },
      ];
      mockInvoke.mockResolvedValue(mockActive);

      const result = await active();

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|get_active",
      );
      expect(result).toEqual(mockActive);
    });

    it("should return empty array when no active notifications", async () => {
      mockInvoke.mockResolvedValue([]);

      const result = await active();

      expect(result).toEqual([]);
    });
  });

  describe("removeActive", () => {
    it("should remove active notifications by ID", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeActive([{ id: 1 }, { id: 2 }]);

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|remove_active",
        {
          notifications: [{ id: 1 }, { id: 2 }],
        },
      );
    });

    it("should remove active notification with tag", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeActive([{ id: 1, tag: "news" }]);

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|remove_active",
        {
          notifications: [{ id: 1, tag: "news" }],
        },
      );
    });

    it("should handle empty array", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeActive([]);

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|remove_active",
        {
          notifications: [],
        },
      );
    });
  });

  describe("removeAllActive", () => {
    it("should remove all active notifications", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeAllActive();

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|remove_active",
      );
    });
  });

  describe("createChannel", () => {
    it("should create notification channel with minimal properties", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const channel = {
        id: "test-channel",
        name: "Test Channel",
      };

      await createChannel(channel);

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|create_channel",
        { channel },
      );
    });

    it("should create channel with all properties", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const channel = {
        id: "full-channel",
        name: "Full Channel",
        description: "Channel description",
        sound: "notification.mp3",
        lights: true,
        lightColor: "#FF0000",
        vibration: true,
        importance: Importance.High,
        visibility: Visibility.Public,
      };

      await createChannel(channel);

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|create_channel",
        { channel },
      );
    });
  });

  describe("removeChannel", () => {
    it("should delete notification channel", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeChannel("test-channel");

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|delete_channel",
        {
          id: "test-channel",
        },
      );
    });
  });

  describe("channels", () => {
    it("should retrieve all notification channels", async () => {
      const mockChannels = [
        {
          id: "channel1",
          name: "Channel 1",
          importance: Importance.Default,
        },
        {
          id: "channel2",
          name: "Channel 2",
          importance: Importance.High,
        },
      ];
      mockInvoke.mockResolvedValue(mockChannels);

      const result = await channels();

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|list_channels",
      );
      expect(result).toEqual(mockChannels);
    });

    it("should return empty array when no channels", async () => {
      mockInvoke.mockResolvedValue([]);

      const result = await channels();

      expect(result).toEqual([]);
    });
  });

  describe("onNotificationReceived", () => {
    it("should register notification received listener", async () => {
      const mockUnlisten = vi.fn();
      mockAddPluginListener.mockResolvedValue(mockUnlisten);

      const callback = vi.fn();
      const unlisten = await onNotificationReceived(callback);

      expect(mockAddPluginListener).toHaveBeenCalledWith(
        "notifications",
        "notification",
        callback,
      );
      expect(unlisten).toBe(mockUnlisten);
    });

    it("should call callback when notification received", async () => {
      const mockNotification = { title: "Test", body: "Body" };
      let capturedCallback: ((notification: any) => void) | undefined;

      mockAddPluginListener.mockImplementation((_plugin, _event, cb) => {
        capturedCallback = cb;
        return Promise.resolve(vi.fn());
      });

      const callback = vi.fn();
      await onNotificationReceived(callback);

      capturedCallback?.(mockNotification);

      expect(callback).toHaveBeenCalledWith(mockNotification);
    });
  });

  describe("onAction", () => {
    it("should register action performed listener", async () => {
      const mockUnlisten = vi.fn();
      mockAddPluginListener.mockResolvedValue(mockUnlisten);

      const callback = vi.fn();
      const unlisten = await onAction(callback);

      expect(mockAddPluginListener).toHaveBeenCalledWith(
        "notifications",
        "actionPerformed",
        callback,
      );
      expect(unlisten).toBe(mockUnlisten);
    });

    it("should call callback when action performed", async () => {
      const mockNotification = { title: "Test", actionTypeId: "action-1" };
      let capturedCallback: ((notification: any) => void) | undefined;

      mockAddPluginListener.mockImplementation((_plugin, _event, cb) => {
        capturedCallback = cb;
        return Promise.resolve(vi.fn());
      });

      const callback = vi.fn();
      await onAction(callback);

      capturedCallback?.(mockNotification);

      expect(callback).toHaveBeenCalledWith(mockNotification);
    });
  });

  describe("onNotificationClicked", () => {
    it("should register notification clicked listener", async () => {
      const mockUnregister = vi.fn().mockResolvedValue(undefined);
      mockAddPluginListener.mockResolvedValue({ unregister: mockUnregister });
      mockInvoke.mockResolvedValue(undefined);

      const callback = vi.fn();
      const listener = await onNotificationClicked(callback);

      expect(mockAddPluginListener).toHaveBeenCalledWith(
        "notifications",
        "notificationClicked",
        callback,
      );
      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|set_click_listener_active",
        { active: true },
      );
      expect(listener).toHaveProperty("unregister");
    });

    it("should notify native side on unregister", async () => {
      const mockUnregister = vi.fn().mockResolvedValue(undefined);
      mockAddPluginListener.mockResolvedValue({ unregister: mockUnregister });
      mockInvoke.mockResolvedValue(undefined);

      const callback = vi.fn();
      const listener = await onNotificationClicked(callback);

      mockInvoke.mockClear();
      await listener.unregister();

      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:notifications|set_click_listener_active",
        { active: false },
      );
      expect(mockUnregister).toHaveBeenCalled();
    });

    it("should call callback when notification clicked", async () => {
      const mockClickedData = { id: 123, data: { key: "value" } };
      let capturedCallback: ((data: any) => void) | undefined;

      mockAddPluginListener.mockImplementation((_plugin, _event, cb) => {
        capturedCallback = cb;
        return Promise.resolve(vi.fn());
      });

      const callback = vi.fn();
      await onNotificationClicked(callback);

      capturedCallback?.(mockClickedData);

      expect(callback).toHaveBeenCalledWith(mockClickedData);
    });

    it("should handle notification click without data", async () => {
      const mockClickedData = { id: 456 };
      let capturedCallback: ((data: any) => void) | undefined;

      mockAddPluginListener.mockImplementation((_plugin, _event, cb) => {
        capturedCallback = cb;
        return Promise.resolve(vi.fn());
      });

      const callback = vi.fn();
      await onNotificationClicked(callback);

      capturedCallback?.(mockClickedData);

      expect(callback).toHaveBeenCalledWith(mockClickedData);
      expect(callback.mock.calls[0][0].data).toBeUndefined();
    });
  });
});

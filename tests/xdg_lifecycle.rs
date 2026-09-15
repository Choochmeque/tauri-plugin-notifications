#![cfg(any(target_os = "linux", target_os = "freebsd"))]

use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};
use tauri_plugin_notifications::NotificationsExt;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Debug, PartialEq)]
enum Delivery {
    Shown {
        id: u32,
        title: String,
        body: String,
    },
    Closed(u32),
}

struct NotificationService {
    next_id: AtomicU32,
    deliveries: UnboundedSender<Delivery>,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationService {
    // zbus requires a receiver even when the D-Bus method returns constant metadata.
    #[allow(clippy::unused_self)]
    fn get_capabilities(&self) -> Vec<String> {
        vec!["body".into()]
    }

    #[allow(clippy::unused_self)]
    fn get_server_information(&self) -> (String, String, String, String) {
        ("test".into(), "test".into(), "1".into(), "1.2".into())
    }

    #[allow(clippy::too_many_arguments)]
    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        hints: HashMap<String, zbus::zvariant::OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        let _ = (
            app_name,
            replaces_id,
            app_icon,
            actions,
            hints,
            expire_timeout,
        );
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.deliveries
            .send(Delivery::Shown {
                id,
                title: summary.into(),
                body: body.into(),
            })
            .expect("notification receipt receiver disconnected");
        id
    }

    fn close_notification(&self, id: u32) {
        self.deliveries
            .send(Delivery::Closed(id))
            .expect("close receipt receiver disconnected");
    }
}

async fn start_notification_service(deliveries: UnboundedSender<Delivery>) -> zbus::Connection {
    let service = zbus::connection::Builder::session()
        .expect("connect to the isolated D-Bus session")
        .serve_at(
            "/org/freedesktop/Notifications",
            NotificationService {
                next_id: AtomicU32::new(1),
                deliveries,
            },
        )
        .expect("register the notification interface")
        .build()
        .await
        .expect("build the notification daemon connection");
    let bus = zbus::fdo::DBusProxy::new(&service)
        .await
        .expect("create the session bus proxy");
    let name = "org.freedesktop.Notifications"
        .try_into()
        .expect("valid notification bus name");
    assert!(
        !bus.name_has_owner(name)
            .await
            .expect("inspect the notification bus name"),
        "use an isolated session bus"
    );
    service
        .request_name("org.freedesktop.Notifications")
        .await
        .expect("own the notification bus name");
    service
}

async fn next_delivery(deliveries: &mut UnboundedReceiver<Delivery>) -> Delivery {
    tokio::time::timeout(Duration::from_secs(5), deliveries.recv())
        .await
        .expect("notification daemon did not receive the operation")
        .expect("notification daemon disconnected")
}

// Run with `dbus-run-session -- cargo test --test xdg_lifecycle -- --ignored`.
// This exercises the real plugin and D-Bus transport, without requiring a GUI.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires an isolated D-Bus session without a notification daemon"]
async fn shows_tracks_and_closes_notifications() {
    assert!(std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some());
    tokio::time::timeout(Duration::from_secs(30), async {
        let (sender, mut deliveries) = unbounded_channel();
        let _service = start_notification_service(sender).await;

        let app = tauri::test::mock_builder()
            .plugin(tauri_plugin_notifications::init())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("build the notification test app");
        let notifications = app.notifications();

        for (id, title) in [(1, "first"), (2, "second")] {
            notifications
                .builder()
                .id(7)
                .title(title)
                .body("body")
                .show()
                .await
                .expect("show a notification with a shared caller ID");
            assert_eq!(
                next_delivery(&mut deliveries).await,
                Delivery::Shown {
                    id,
                    title: title.into(),
                    body: "body".into(),
                }
            );
        }
        let active = notifications
            .active()
            .await
            .expect("read active notifications");
        let active = serde_json::to_value(active).expect("serialize active notifications");
        let active = active
            .as_array()
            .expect("active notifications are an array");
        assert_eq!(
            active.len(),
            2,
            "equal caller IDs must retain both notification handles"
        );
        assert!(active.iter().all(|notification| notification["id"] == 7));
        notifications
            .cancel(vec![7])
            .expect("cancel the shared caller ID");
        assert!(
            notifications
                .active()
                .await
                .expect("read notifications after cancel")
                .is_empty()
        );
        let mut closed = Vec::new();
        for _ in 0..2 {
            match next_delivery(&mut deliveries).await {
                Delivery::Closed(id) => closed.push(id),
                other @ Delivery::Shown { .. } => {
                    panic!("expected a daemon close receipt, got {other:?}")
                }
            }
        }
        closed.sort_unstable();
        assert_eq!(closed, [1, 2]);

        for (id, caller_id) in [(3, 8), (4, 9)] {
            notifications
                .builder()
                .id(caller_id)
                .title("close")
                .show()
                .await
                .expect("show a notification for removal");
            assert_eq!(
                next_delivery(&mut deliveries).await,
                Delivery::Shown {
                    id,
                    title: "close".into(),
                    body: String::new(),
                }
            );
            if caller_id == 8 {
                notifications
                    .remove_active(vec![caller_id])
                    .expect("remove the active notification");
            } else {
                notifications
                    .cancel_all()
                    .expect("cancel all notifications");
            }
            assert_eq!(next_delivery(&mut deliveries).await, Delivery::Closed(id));
            assert!(
                notifications
                    .active()
                    .await
                    .expect("read notifications after removal")
                    .is_empty()
            );
        }
    })
    .await
    .expect("notification lifecycle timed out");
}

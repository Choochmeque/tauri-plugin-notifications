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
    fn get_capabilities(&self) -> Vec<String> {
        vec!["body".into()]
    }

    fn get_server_information(&self) -> (String, String, String, String) {
        ("test".into(), "test".into(), "1".into(), "1.2".into())
    }

    #[allow(clippy::too_many_arguments)]
    fn notify(
        &self,
        _app_name: &str,
        _replaces_id: u32,
        _app_icon: &str,
        summary: &str,
        body: &str,
        _actions: Vec<String>,
        _hints: HashMap<String, zbus::zvariant::OwnedValue>,
        _expire_timeout: i32,
    ) -> u32 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.deliveries
            .send(Delivery::Shown {
                id,
                title: summary.into(),
                body: body.into(),
            })
            .unwrap();
        id
    }

    fn close_notification(&self, id: u32) {
        self.deliveries.send(Delivery::Closed(id)).unwrap();
    }
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
        let service = zbus::connection::Builder::session()
            .unwrap()
            .serve_at(
                "/org/freedesktop/Notifications",
                NotificationService {
                    next_id: AtomicU32::new(1),
                    deliveries: sender,
                },
            )
            .unwrap()
            .build()
            .await
            .unwrap();
        let bus = zbus::fdo::DBusProxy::new(&service).await.unwrap();
        let name = "org.freedesktop.Notifications".try_into().unwrap();
        assert!(
            !bus.name_has_owner(name).await.unwrap(),
            "use an isolated session bus"
        );
        service
            .request_name("org.freedesktop.Notifications")
            .await
            .unwrap();

        let app = tauri::test::mock_builder()
            .plugin(tauri_plugin_notifications::init())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let notifications = app.notifications();

        for (id, title) in [(1, "first"), (2, "second")] {
            notifications
                .builder()
                .id(7)
                .title(title)
                .body("body")
                .show()
                .await
                .unwrap();
            assert_eq!(
                next_delivery(&mut deliveries).await,
                Delivery::Shown {
                    id,
                    title: title.into(),
                    body: "body".into(),
                }
            );
        }
        let active = serde_json::to_value(notifications.active().await.unwrap()).unwrap();
        let active = active.as_array().unwrap();
        assert_eq!(
            active.len(),
            2,
            "equal caller IDs must retain both notification handles"
        );
        assert!(active.iter().all(|notification| notification["id"] == 7));
        notifications.cancel(vec![7]).unwrap();
        assert!(notifications.active().await.unwrap().is_empty());
        let mut closed = Vec::new();
        for _ in 0..2 {
            match next_delivery(&mut deliveries).await {
                Delivery::Closed(id) => closed.push(id),
                other => panic!("expected a daemon close receipt, got {other:?}"),
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
                .unwrap();
            assert_eq!(
                next_delivery(&mut deliveries).await,
                Delivery::Shown {
                    id,
                    title: "close".into(),
                    body: String::new(),
                }
            );
            if caller_id == 8 {
                notifications.remove_active(vec![caller_id]).unwrap();
            } else {
                notifications.cancel_all().unwrap();
            }
            assert_eq!(next_delivery(&mut deliveries).await, Delivery::Closed(id));
            assert!(notifications.active().await.unwrap().is_empty());
        }
    })
    .await
    .expect("notification lifecycle timed out");
}

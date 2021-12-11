use std::time::Duration;

use tokio::time::sleep;

pub(crate) async fn wait_for_onoff() {
    const WAIT_FOR_ONOFF: Duration = Duration::from_millis(300);
    sleep(WAIT_FOR_ONOFF).await
}

pub(crate) async fn wait_for_write() {
    const WAIT_FOR_WRITE: Duration = Duration::from_millis(100);
    sleep(WAIT_FOR_WRITE).await
}

pub(crate) async fn set_online(cpu_ids: Vec<u64>) -> Vec<u64> {
    let mut onlined = vec![];
    if !cpu_ids.is_empty() {
        let offline = crate::cpu::devices_online().await.unwrap_or_default();
        for cpu_id in cpu_ids {
            if offline.contains(&cpu_id) && crate::cpu::set_online(cpu_id, true).await.is_ok() {
                onlined.push(cpu_id);
            }
        }
        if !onlined.is_empty() {
            wait_for_onoff().await;
        }
    }
    onlined
}

pub(crate) async fn set_offline(cpu_ids: Vec<u64>) -> Vec<u64> {
    let mut offlined = vec![];
    if !cpu_ids.is_empty() {
        let online = crate::cpu::devices_online().await.unwrap_or_default();
        for cpu_id in cpu_ids {
            if online.contains(&cpu_id) && crate::cpu::set_online(cpu_id, false).await.is_ok() {
                offlined.push(cpu_id);
            }
        }
        if !offlined.is_empty() {
            wait_for_onoff().await;
        }
    }
    offlined
}

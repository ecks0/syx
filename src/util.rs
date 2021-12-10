use std::time::Duration;

use tokio::time::sleep;

pub(crate) async fn wait_for_cpu_onoff() {
    const WAIT_FOR_CPU_ONOFF: Duration = Duration::from_millis(300);
    sleep(WAIT_FOR_CPU_ONOFF).await
}

pub(crate) async fn wait_for_cpu_related() {
    const WAIT_FOR_CPU_RELATED: Duration = Duration::from_millis(100);
    sleep(WAIT_FOR_CPU_RELATED).await
}

pub(crate) async fn set_cpus_online(cpu_ids: Vec<u64>) -> Vec<u64> {
    if cpu_ids.is_empty() {
        return Default::default();
    }
    let offline = crate::cpu::offline_devices().await.unwrap_or_default();
    let mut onlined = vec![];
    for cpu_id in cpu_ids {
        if offline.contains(&cpu_id) && crate::cpu::set_online(cpu_id, true).await.is_ok() {
            onlined.push(cpu_id);
        }
    }
    if !onlined.is_empty() {
        wait_for_cpu_onoff().await;
    }
    onlined
}

pub(crate) async fn set_cpus_offline(cpu_ids: Vec<u64>) -> Vec<u64> {
    if cpu_ids.is_empty() {
        return Default::default();
    }
    let online = crate::cpu::online_devices().await.unwrap_or_default();
    let mut offlined = vec![];
    for cpu_id in cpu_ids {
        if online.contains(&cpu_id) && crate::cpu::set_online(cpu_id, false).await.is_ok() {
            offlined.push(cpu_id);
        }
    }
    if !offlined.is_empty() {
        wait_for_cpu_onoff().await;
    }
    offlined
}

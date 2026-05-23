/// PROBE: confirm that the device ignores the schedule rule's
/// year/month/day wire fields.
///
/// The library always sends `year=1970, month=1, day=1` for those
/// fields (see `PLACEHOLDER_DATE` in `tapo/src/requests/schedule.rs`),
/// so we just add a `clock_once` rule for ~2 minutes from now and
/// watch whether it fires.  If it does, the device clearly didn't
/// care that we asked it to schedule something "in 1970".
///
/// To try a different placeholder date, edit `PLACEHOLDER_DATE` in
/// `schedule.rs` to something like `(2099, 12, 31)` and re-run.
///
/// Build / run:
///   HTTPS_PROXY=... cargo run --example probe_date_ignored
///
/// Environment variables: TAPO_USERNAME, TAPO_PASSWORD, IP_ADDRESS.
use std::env;
use std::time::Duration;

use chrono::{Timelike, Utc};
use tapo::ApiClient;
use tapo::requests::ScheduleRule;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device = ApiClient::new(env::var("TAPO_USERNAME")?, env::var("TAPO_PASSWORD")?)
        .p110(env::var("IP_ADDRESS")?)
        .await?;

    let info = device.get_device_info().await?;
    let initial_on = info.device_on;
    // `time_diff` is the device's UTC offset in minutes (e.g. 600 for AEST).
    let time_diff_minutes = info.time_diff.unwrap_or(0);
    println!(
        "Baseline: plug device_on={initial_on} time_diff={time_diff_minutes} \
         (device-clock UTC offset, minutes)",
    );

    let utc_now = Utc::now();
    let utc_min = utc_now.hour() as i64 * 60 + utc_now.minute() as i64;
    let device_now_min = (utc_min + time_diff_minutes).rem_euclid(1440);
    let target_minute = ((device_now_min + 2) % 1440) as u16;
    let hour = (target_minute / 60) as u8;
    let minute = (target_minute % 60) as u8;
    let target_state = !initial_on;
    println!(
        "Adding clock_once for {hour:02}:{minute:02} (turn_on={target_state}). \
         Wire-format year/month/day are forced to 1970-01-01."
    );
    let added = device
        .add_schedule_rule(ScheduleRule::clock_once(hour, minute, target_state)?)
        .await?;
    let rule_id = added.id.clone().expect("device returns an id");
    println!("  added id={rule_id}");

    let rules = device.get_schedule_rules().await?;
    let stored = rules
        .iter()
        .find(|r| r.id.as_deref() == Some(rule_id.as_str()))
        .expect("rule we just added must come back");
    println!(
        "Device-stored fields: time_kind={:?} freq={:?} minute_of_day={} turn_on={} week_day={:#b}",
        stored.time_kind, stored.frequency, stored.minute_of_day, stored.turn_on, stored.week_day,
    );

    let total_wait_secs = 2 * 60 + 30; // 2 min until target minute hits, plus 30 s slack
    println!("Waiting {total_wait_secs}s for the rule to fire...");
    tokio::time::sleep(Duration::from_secs(total_wait_secs)).await;

    let final_on = device.get_device_info().await?.device_on;
    println!("After wait: plug device_on={final_on} (expected {target_state})");

    if final_on == target_state {
        println!("PROBE RESULT: rule fired despite year=1970,month=1,day=1 → date fields ignored.");
    } else {
        println!("PROBE RESULT: rule did NOT fire → date fields appear to matter.");
    }

    println!("Cleanup: removing the probe rule.");
    device.remove_schedule_rule(rule_id).await?;
    if initial_on {
        device.on().await?;
    } else {
        device.off().await?;
    }
    println!("done.");
    Ok(())
}

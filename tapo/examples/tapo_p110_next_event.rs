/// Demo: querying the plug for its next-scheduled event.
///
/// This is the easiest way to discover what time the device thinks
/// sunrise / sunset is: add a `sunrise_once(0)` (or `sunset_once(0)`)
/// rule and read back the `start_time` via `get_next_event`.
///
/// Build / run:
///   cargo run --example tapo_p110_next_event
///
/// Environment variables: TAPO_USERNAME, TAPO_PASSWORD, IP_ADDRESS.
use std::env;

use log::info;
use tapo::ApiClient;
use tapo::requests::ScheduleRule;

mod common;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    common::setup_logger();

    let device = ApiClient::new(env::var("TAPO_USERNAME")?, env::var("TAPO_PASSWORD")?)
        .p110(env::var("IP_ADDRESS")?)
        .await?;

    info!("Asking the device when its next sunrise will be...");
    let sunrise_probe = device
        .add_schedule_rule(ScheduleRule::sunrise_once(0, true)?)
        .await?;
    let next = device.get_next_event().await?;
    if let Some(event) = &next {
        info!(
            "Next event after adding sunrise_once(0): id={} start_time={} turn_on={}",
            event.id, event.start_time, event.turn_on,
        );
    }
    device
        .remove_schedule_rule(sunrise_probe.id.clone().expect("device returns id"))
        .await?;

    info!("Asking the device when its next sunset will be...");
    let sunset_probe = device
        .add_schedule_rule(ScheduleRule::sunset_once(0, true)?)
        .await?;
    let next = device.get_next_event().await?;
    if let Some(event) = &next {
        info!(
            "Next event after adding sunset_once(0): id={} start_time={} turn_on={}",
            event.id, event.start_time, event.turn_on,
        );
    }
    device
        .remove_schedule_rule(sunset_probe.id.clone().expect("device returns id"))
        .await?;

    info!("With no rules added by this demo, next_event reflects whatever else lives on the plug.");
    let resting = device.get_next_event().await?;
    match resting {
        Some(e) => info!(
            "Resting next_event: id={} start_time={} turn_on={}",
            e.id, e.start_time, e.turn_on,
        ),
        None => info!("Resting next_event: None (no schedule rules armed)."),
    }

    Ok(())
}

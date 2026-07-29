/// P110, P110M and P115 Schedule Example
use log::info;
use tapo::ApiClient;
use tapo::requests::{ScheduleRule, week_day};
use tapo::responses::PowerState;

mod common;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    common::setup_logger();

    let [tapo_username, tapo_password, ip_address] =
        common::require_env_vars(["TAPO_USERNAME", "TAPO_PASSWORD", "IP_ADDRESS"])?;

    let device = ApiClient::new(tapo_username, tapo_password)
        .p110(ip_address)
        .await?;

    // Rules already on the device are left alone; this example only
    // removes the ones it adds.
    info!("Adding four rules...");
    let added = [
        // Turn on once, the next time the clock hits 06:30.
        device
            .add_schedule_rule(ScheduleRule::clock_once(6, 30, PowerState::On)?)
            .await?,
        // Turn off weekly at 23:30 on Mondays and Wednesdays.
        device
            .add_schedule_rule(ScheduleRule::clock_weekly(
                23,
                30,
                week_day::MON | week_day::WED,
                PowerState::Off,
            )?)
            .await?,
        // Turn on every day, one hour after sunset.
        device
            .add_schedule_rule(ScheduleRule::sunset_weekly(
                60,
                week_day::EVERY_DAY,
                PowerState::On,
            )?)
            .await?,
        // Turn off on weekdays (Mon–Fri), 30 minutes before sunrise.
        device
            .add_schedule_rule(ScheduleRule::sunrise_weekly(
                -30,
                week_day::WEEKDAYS,
                PowerState::Off,
            )?)
            .await?,
    ];
    for rule in &added {
        info!("Added rule: {rule:?}");
    }

    let rules = device.get_schedule_rules().await?;
    for rule in &rules {
        info!("Rule: {rule:?}");
    }

    info!("Removing the four rules that were added...");
    for rule in &added {
        if let Some(id) = &rule.id {
            device.remove_schedule_rule(id.clone()).await?;
        }
    }

    Ok(())
}

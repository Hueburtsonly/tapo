"""P110, P110M and P115 Schedule Example"""

import asyncio

from tapo import ApiClient
from tapo.requests import MON, WED, WEEKDAYS, EVERY_DAY, ScheduleRule
from tapo.responses import PowerState

from common import require_env_vars


async def main():
    tapo_username, tapo_password, ip_address = require_env_vars(
        "TAPO_USERNAME", "TAPO_PASSWORD", "IP_ADDRESS"
    )

    client = ApiClient(tapo_username, tapo_password)
    device = await client.p110(ip_address)

    # Rules already on the device are left alone; this example only
    # removes the ones it adds.
    print("Adding four rules...")
    added = [
        # Turn on once, the next time the clock hits 06:30.
        await device.add_schedule_rule(ScheduleRule.clock_once(6, 30, PowerState.On)),
        # Turn off weekly at 23:30 on Mondays and Wednesdays.
        await device.add_schedule_rule(
            ScheduleRule.clock_weekly(23, 30, MON | WED, PowerState.Off)
        ),
        # Turn on every day, one hour after sunset.
        await device.add_schedule_rule(ScheduleRule.sunset_weekly(60, EVERY_DAY, PowerState.On)),
        # Turn off on weekdays (Mon–Fri), 30 minutes before sunrise.
        await device.add_schedule_rule(ScheduleRule.sunrise_weekly(-30, WEEKDAYS, PowerState.Off)),
    ]
    for rule in added:
        print(f"Added rule: {rule.to_dict()}")

    rules = await device.get_schedule_rules()
    for rule in rules:
        print(f"Rule: {rule.to_dict()}")

    print("Removing the four rules that were added...")
    for rule in added:
        if rule.id:
            await device.remove_schedule_rule(rule.id)


if __name__ == "__main__":
    asyncio.run(main())

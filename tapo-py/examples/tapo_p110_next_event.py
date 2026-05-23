"""Demo: querying the plug for its next-scheduled event.

The easiest way to discover what time the device thinks sunrise /
sunset is: add a ``sunrise_once(0)`` (or ``sunset_once(0)``) rule and
read back ``start_time`` via ``get_next_event``.

Build / run:
  cd tapo-py && maturin develop --release
  python examples/tapo_p110_next_event.py

Environment variables: TAPO_USERNAME, TAPO_PASSWORD, IP_ADDRESS.
"""

import asyncio
import logging
import os

from tapo import ApiClient
from tapo.requests import ScheduleRule


async def main() -> None:
    logging.basicConfig(level=logging.INFO, format="%(asctime)s %(message)s")
    log = logging.getLogger("next-event")

    device = await ApiClient(
        os.environ["TAPO_USERNAME"], os.environ["TAPO_PASSWORD"]
    ).p110(os.environ["IP_ADDRESS"])

    log.info("Asking the device when its next sunrise will be...")
    sunrise_probe = await device.add_schedule_rule(ScheduleRule.sunrise_once(0, True))
    event = await device.get_next_event()
    if event is not None:
        log.info(
            "Next event after adding sunrise_once(0): id=%s start_time=%s turn_on=%s",
            event.id, event.start_time, event.turn_on,
        )
    await device.remove_schedule_rule(sunrise_probe.id)

    log.info("Asking the device when its next sunset will be...")
    sunset_probe = await device.add_schedule_rule(ScheduleRule.sunset_once(0, True))
    event = await device.get_next_event()
    if event is not None:
        log.info(
            "Next event after adding sunset_once(0): id=%s start_time=%s turn_on=%s",
            event.id, event.start_time, event.turn_on,
        )
    await device.remove_schedule_rule(sunset_probe.id)

    log.info("With no rules added by this demo, next_event reflects whatever else lives on the plug.")
    resting = await device.get_next_event()
    if resting is None:
        log.info("Resting next_event: None (no schedule rules armed).")
    else:
        log.info(
            "Resting next_event: id=%s start_time=%s turn_on=%s",
            resting.id, resting.start_time, resting.turn_on,
        )


if __name__ == "__main__":
    asyncio.run(main())

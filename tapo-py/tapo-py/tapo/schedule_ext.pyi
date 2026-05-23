from typing import List, Optional, Protocol

from tapo.requests import ScheduleRule
from tapo.responses import NextEvent

class ScheduleExt(Protocol):
    """Extension class for the plug's schedule rules (the "Schedule"
    feature in the Tapo app).  Schedule rules live on the device, so
    they keep firing even if the phone / Wi-Fi router / Tapo cloud is
    offline."""

    async def add_schedule_rule(self, rule: ScheduleRule) -> ScheduleRule:
        """Adds a new schedule rule.  Returns the same rule with its
        device-assigned ``id`` filled in.  Construct ``rule`` via the
        ``ScheduleRule.clock_weekly`` / ``clock_once`` / ``sunrise_*``
        / ``sunset_*`` factories."""

    async def edit_schedule_rule(self, rule: ScheduleRule) -> None:
        """Edits an existing rule.  ``rule.id`` must be set to the
        id of the rule to update."""

    async def get_schedule_rules(self) -> List[ScheduleRule]:
        """Returns every schedule rule currently stored on the device."""

    async def remove_schedule_rule(self, id: str) -> None:
        """Removes the schedule rule with the given id."""

    async def remove_all_schedule_rules(self) -> None:
        """Removes every schedule rule from the device."""

    async def get_next_event(self) -> Optional[NextEvent]:
        """Returns the next event the device is scheduled to act on
        (typically the soonest-firing schedule rule), or ``None`` if no
        rule is currently armed to fire.  Useful for surfacing the
        device-computed sunrise / sunset firing time for sun-relative
        schedule rules."""

from datetime import datetime
from typing import Optional

from tapo.to_dict_ext import ToDictExt

class NextEvent(ToDictExt):
    """The next event the device is scheduled to act on — typically a
    schedule rule firing."""

    id: str
    """Device-assigned id of the schedule rule that will fire next."""
    start_time: datetime
    """When the event fires (UTC)."""
    end_time: Optional[datetime]
    """Optional end-time for two-stage rules.  ``None`` when the rule has no end action."""
    event_type: int
    """Wire-level event type code as the device reports it.  Value ``1`` has
    been observed for plug schedule rules; the field is kept for forward
    compatibility with other event kinds."""
    turn_on: bool
    """Whether the device will turn on (``True``) or off (``False``) when the
    event fires."""

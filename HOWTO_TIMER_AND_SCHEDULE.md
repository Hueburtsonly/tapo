# P110 Timer & Schedule — How To Test

This branch (`p110-timer-and-schedule-howto`) sits on top of
`p110-timer-and-schedule` in the
[`Hueburtsonly/tapo`](https://github.com/Hueburtsonly/tapo) fork and
adds this guide. The actual library changes are the two commits on
`p110-timer-and-schedule`:

```
208b1e0 feat(tapo, tapo-py): add plug Schedule rule API (closes #166)
812e607 feat(tapo, tapo-py): add plug Timer (countdown) API
```

## What's new

`PlugHandler` and `PlugEnergyMonitoringHandler` gain:

| Feature   | Methods                                                                                                |
| --------- | ------------------------------------------------------------------------------------------------------ |
| Timer     | `set_timer`, `get_timer`, `clear_timer`                                                                |
| Schedule  | `add_schedule_rule`, `edit_schedule_rule`, `get_schedule_rules`, `remove_schedule_rule`, `remove_all_schedule_rules` |

The wire-format API on the device is `*_countdown_rule` / `*_schedule_rule`;
in Rust / Python this surfaces as a typed `Timer` result and a typed
`ScheduleRule` struct built via six factory methods (one per
clock/sunrise/sunset × once/weekly combination).

## Prerequisites

* A P110 (or compatible plug — P100, P105, P115, P110M) that can be
  reached on the same network from your machine.
* Tapo account credentials for the device.
* For Rust: a stable `cargo` toolchain.
* For Python: Python 3.10+ and `maturin` (inside a virtualenv).

```bash
pip install maturin
```

## Get the code

```bash
git clone https://github.com/Hueburtsonly/tapo.git
cd tapo
git checkout p110-timer-and-schedule
```

(That branch is the one whose code you're testing. This `*-howto`
branch only adds this markdown and the verification probe.)

## Set environment variables

```bash
export TAPO_USERNAME="your-tapo-account@example.com"
export TAPO_PASSWORD="your-tapo-password"
export IP_ADDRESS="192.168.1.121"   # your plug's IP
```

## Rust

> **Note:** all `cargo run --example …` commands in this guide pass
> `-p tapo`. If you omit it, Cargo activates the `python` feature on
> the `tapo` crate (because the workspace also contains `tapo-py`,
> which depends on `tapo` with `features = ["python", "debug"]`).
> That makes the link step pull in unresolved PyO3 symbols (`Py_TYPE`,
> `PyErr_SetObject`, etc.). Running with `-p tapo` — or `cd tapo`
> first and running without `-p` — keeps the example link clean.

### Timer demo (~30 s)

```bash
RUST_LOG=info cargo run -p tapo --example tapo_p110_timer
```

Expected output (timings are approximate):

```
INFO  tapo_p110_timer Baseline: plug off, no armed timer.
INFO  tapo_p110_timer Arming a 10-second 'turn ON' timer...
INFO  tapo_p110_timer Armed: id=C1 delay=10s
INFO  tapo_p110_timer Read back: id=C1 remain=10s turn_on=true
INFO  tapo_p110_timer Waiting 15 seconds for the timer to fire (10s delay + slack)...
INFO  tapo_p110_timer Timer fired — plug is ON.
INFO  tapo_p110_timer Arming a 5-second 'turn OFF' timer and clearing it before it fires...
INFO  tapo_p110_timer Waiting 10 seconds to confirm the cleared timer did not fire...
INFO  tapo_p110_timer PASS
```

What it exercises:

1. `clear_timer` and `get_timer` on a fresh-baseline plug.
2. `set_timer(Duration::from_secs(10), true)` — the timer fires within
   ~10 s and the plug turns on.
3. `set_timer` followed immediately by `clear_timer` — the plug stays
   in its original (on) state past the original deadline because the
   timer was removed.

### Schedule demo (~3 s)

```bash
RUST_LOG=info cargo run -p tapo --example tapo_p110_schedule
```

Expected output:

```
INFO  tapo_p110_schedule Pre-existing rules on the device: <N> (left alone)
INFO  tapo_p110_schedule Adding four demo rules...
INFO  tapo_p110_schedule   added ids: ["S<a>", "S<b>", "S<c>", "S<d>"]
INFO  tapo_p110_schedule Read back sunset rule: id=Some("S<c>") time_kind=Sunset freq=Weekly offset_minutes=0 week_day=0b1111111 turn_on=true
INFO  tapo_p110_schedule Cleaning up: removing the four demo rules.
INFO  tapo_p110_schedule Cleanup OK — <N> pre-existing rule(s) left intact: [...]
INFO  tapo_p110_schedule PASS
```

The four demo rules are: a `clock_once` at 06:30, a `clock_weekly`
turn-off at 23:30 on Mon and Wed, a `sunset_weekly` every day at
sunset, and a `sunrise_weekly` on weekdays at sunrise. The demo reads
the sunset rule back, then removes all four. Pre-existing rules
already on the plug (e.g. created from the Tapo app) are left
untouched.

### Trying sunrise / sunset rules

The library exposes a clean builder for sun-relative rules:

```rust
use tapo::requests::{ScheduleRule, week_day};

// Turn on 1 h 3 min before sunrise on Mondays.
let r = ScheduleRule::sunrise_weekly(-63, week_day::MON, true);

// Turn off 7 min after sunset on Fridays.
let r = ScheduleRule::sunset_weekly(7, week_day::FRI, false);

// One-shot at the next sunset:
let r = ScheduleRule::sunset_once(0, false);
```

The device handles the astronomy itself based on the location you
configured in the Tapo app (this was verified against a P110 in
Sydney — a sunset rule reads back with `time_kind = Sunset` without
the caller ever needing to know what time sunset is on a given day).

## Python

### Build & install the wheel

```bash
cd tapo-py
python3 -m venv .venv && source .venv/bin/activate
pip install --upgrade pip maturin
maturin develop --release
cd ..
```

### Timer demo (~30 s)

```bash
python tapo-py/examples/tapo_p110_timer.py
```

Expected output mirrors the Rust demo.

### Schedule demo (~3 s)

```bash
python tapo-py/examples/tapo_p110_schedule.py
```

Expected last line: `PASS`.

### Sunrise / sunset from Python

Day-of-week bits are exposed as module constants — combine them with
`|` for the `*_weekly` factories:

```python
from tapo.requests import ScheduleRule, MON, FRI, WEEKDAYS

# Turn on 1 h 3 min before sunrise on Mondays.
r = ScheduleRule.sunrise_weekly(-63, MON, True)

# Turn off 7 min after sunset on Fridays.
r = ScheduleRule.sunset_weekly(7, FRI, False)

# Turn off at sunrise on weekdays.
r = ScheduleRule.sunrise_weekly(0, WEEKDAYS, False)
```

Constants available: `SUN`, `MON`, `TUE`, `WED`, `THU`, `FRI`, `SAT`,
`WEEKDAYS` (Mon–Fri), `WEEKEND` (Sat + Sun), `EVERY_DAY`.

## Verifying that the device ignores the wire `year/month/day` fields

The schedule wire format requires `year`, `month`, and `day` integers
alongside `s_min` (clock minute-of-day) or `time_offset` (sunrise /
sunset offset).  The Tapo app's schedule UI never asks the user for a
date, so we suspected those fields were unused by the device's firing
logic; this branch ships a probe that confirms that experimentally.

The probe is at `tapo/examples/probe_date_ignored.rs` and:

1. Reads the device's `time_diff` (UTC offset in minutes) from
   `get_device_info` so it can compute "two minutes from now" in the
   device's local clock.
2. Sends `add_schedule_rule(ScheduleRule::clock_once(hour, minute, …))`,
   but with the wire `year/month/day` patched to `1970-01-01` (see
   the one-line patch below).
3. Waits 2 min 30 s.
4. Checks that the plug toggled to the requested state.
5. Removes the probe rule and restores the plug's original on/off
   state.

To re-run the probe yourself, apply this one-line patch to
`tapo/src/requests/schedule.rs` (replacing the constant placeholder
the library normally sends with the same value, written here to make
the intent explicit) — or replace `1970, 1, 1` with anything else
you want to test, e.g. `2099, 12, 31`:

```diff
-const PLACEHOLDER_DATE: (i32, u8, u8) = (1970, 1, 1);
+const PLACEHOLDER_DATE: (i32, u8, u8) = (1970, 1, 1); // try (2099, 12, 31) etc.
```

Then:

```bash
cargo run -p tapo --example probe_date_ignored
```

Observed against a P110 in Sydney (AEST, `time_diff=600`):

```
Baseline: plug device_on=false time_diff=600 (device-clock UTC offset, minutes)
Adding clock_once for 17:12 (turn_on=true). Wire-format year/month/day are forced to 1970-01-01.
  added id=S1
Device-stored fields: time_kind=Clock freq=Once minute_of_day=1032 turn_on=true week_day=0b0
Waiting 150s for the rule to fire...
After wait: plug device_on=true (expected true)
PROBE RESULT: rule fired despite year=1970,month=1,day=1 → date fields ignored.
Cleanup: removing the probe rule.
done.
```

Conclusion: the device evaluates HH:MM (or sunrise/sunset offset)
against its own configured clock and timezone, independent of the
calendar date carried in the request — so the library can safely
send a constant placeholder for `year/month/day`.

## Notes / gotchas

* The plug supports **one armed timer at a time**. `set_timer` is
  defined as "replace": it removes any existing timer and adds the
  new one, so callers don't have to think about that constraint.
* For schedule rules, `s_min` and `time_offset` on the wire are
  overloaded depending on `s_type`; the library hides this — you set
  `hour:minute` (or sunrise/sunset offset) and the appropriate field
  is filled in.
* `ScheduleRule` is immutable from Python. To "edit" a read-back rule
  with a different `enable` (or to slot in an `id` from elsewhere),
  use `.with_enable(bool)` / `.with_id(str)` which return a modified
  copy.
* The on-the-wire `year/month/day` are always sent as `1970-01-01`;
  the device ignores their values (see the probe above).

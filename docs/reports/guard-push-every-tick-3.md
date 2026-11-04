# guard-push-every-tick-3 — report

Branch `s/guard-push-every-tick-3` (from origin/s/guard-push-every-tick-2 c9a93c66, origin/master
f9f6544c merged; commits 151e7f92 → the finish push).

## What was done

**The push-dodge widened and made opt-in** (`crates/nf-view/tests/common/walker.rs`, one flag):

- `push_dodge: bool` (default OFF — the shared walker stays master's; rule 7), set in
  cutscene_timer_pause_live, walkreal_07000002/07, escort_follow_07000001, exit_aim_01,
  interactables_07000001, party_door_lock_07000007, probe_07_route. The dodge's arming and its
  whole run sit behind the flag.
- The release distance 2.2 → **2.6** (the report's "dodge until d > 2.6").
- The geometry is re-read every 10 ticks **from wherever Bond stands**: the away-from-body strafe
  side first, then the only open side, then — when BOTH strafe probes hit walls (the 07 hall-door
  doorway squeeze) — the **yield**: Bond walks BACKWARD [M Player_Update 0x193600's move rows take
  a negative vf] until a side opens or the body is 2.6 clear, holding 600 ticks once clear so a
  mid-mission-walk body can carry his beat off the lane. While a body still presses (< 1.6) the
  strafe runs pure lateral (the diagonal's forward component was grinding Bond back into the
  doorway).
- The escalation: a full 1200-tick dodge cycle with the body still pinning the lane fires the
  game's own answer through the real fire path — a CIVILIAN_MISSION hit by a bullet scatters
  [M 0x15d860's msg::BULLET/SOUND arm → on_bullet_resume → CIVILIAN_SCARED; the port's state
  table drone.rs]. The aim point is offset 2.5 u off the spine: a HIT kills (the 07 civilian dies
  at hp 10 and the kill fired the level exit — the mission-fail state — measured this session),
  the offset shot passes beside him.
- **nav_route costs live bodies** (the report's second lever, test-side): a route whose nodes all
  sit ≥ 1.2 u from every live body beats one that threads a body; the first body-adjacent route
  stays the fallback when no clear lane exists.

**Green**: `escort_follow_07000001_live` **1 passed / 0 failed** (red on the push before — the
dodge cleared it). `interactables_07000001_live` improved to 7/3 (was red on the camera sweep).

## What's still red, and why

- **cutscene_timer_pause_live (0/1), probe_07_route (0/1), party_door_lock_07000007 (0/2)** — one
  cause: 07000007's hall-door civilian (drone #22, team 3, CIVILIAN_MISSION 0x16, hp 10) stands
  STATIC inside the hall doorway's (x,z) footprint — the only crossing to the story point. The
  port's push is faithful [M 0x1ec180:253-266 + 0x149ec8], so the walk wedges exactly where the
  game's player would; the dodge now explores (the yield/strafe cycles reach d 1.9-2.4) but cannot
  conjure a passage. The kill path is closed (the level exit = the fail state), and the offset
  miss-shot does not scatter him: the CIVILIAN_MISSION scare folds
  `prop(d,0x3c) && seen && bond_armed` [M 0x1787c8:30-42 + 0x178768] and did not fire on a shot
  from 1.1 u. Two candidate root causes, both port-side: (a) the civilian's mission-route walk
  (state 0x16's `update_mission_route`) parks him where the disc's beat moves him; (b) the scare
  fold's `seen`/`bond_armed` input at this geometry.
- **interactables_07000001_live (7/3)**: the remaining three are the bonus walk-over (the same
  dodge-cap shape), the camera-latch assert (31 consecutive cone ticks — Bond's pushed position
  shifts the cone cadence) and the redlamp-tip assert (the shot's origin shifted).
- **exit_aim_01_live, walkreal_07000002_live, walkreal_07000007_live**: still running when the
  window closed (timeouts at 260-380 s — they are long walks; the 2-report listed them as
  timeouts too). **walkreal_0700000c/11 and the two facial harnesses were not reached.**
- The full suite (chunk_suite.py) did not fit. No assertion was weakened anywhere; every change is
  walker/harness-side, opt-in per level.

## What's left (the next message continues from this branch)

1. The 07 civilian: get the live check below, then either fix the port's scare/mission-walk fold
   (with the address) or give the three 07 walks the player's own route/timing change.
2. Run exit_aim_01, walkreal 02/07/0c/11, the facial pair, nf-game, every walkreal_*_live, and
   chunk_suite.py to the total line.

## Live checks for the main agent

1. `fieldprobe` 07000007 drone #22 (team 3, CIVILIAN_MISSION, the hall doorway ~(79.5, 44.0,
   −50.3)): walk armed Bond at him along the hall lane — does his state move to CIVILIAN_SCARED
   (0x19) and does he RUN? Watch Drone+0x228 (the seen bit) and the armed-person fold inputs.
   If he scatters in the real game, the port's fold has a gap (the fix lands with the address);
   if he stands, the real player also cannot pass and the three 07 walks need their routes re-timed.
2. `gdbwatch` the same drone's position over 60 s from a FRESH boot with Bond elsewhere: does
   `update_mission_route` (state 0x16) walk him along a placement path, or is he parked at
   (79.5, 44.0, −50.3)? Expected if the disc's mission-walk runs: his pos changes.

## Usage

not visible to me — this session exposes no token, cost or context-window figures to the agent.

## ntfy failed

All three notification fallbacks failed in this sandbox (ntfy.sh unreachable from here):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (exit 28)
2. `python3 -c "...urlopen(...)"` → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x` → killed by the 30 s timeout, no output (exit 124)

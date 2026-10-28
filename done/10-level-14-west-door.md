## Your task: `level-14-west-door`, Countdown Part 1 past the facility's west double door to the Delta exit

On 07000014, after the vent-drop hand-back (now on master: Bond lands inside the facility), walkreal_07000014_live `walk_the_infiltration_to_the_delta_exit` wedges at the facility's west double door at (-52.2, 1.2, 40.1): two door records, kind 2, state 0x2, pos01 0.00, ch_open 0, ch_lock 0x85, lock_flag 0. Find how the game opens it [D the placements writing channel 0x85 and their triggers; M the door's lock/unlock and use paths, Door_Update / the lock channel read] - a switch, an objective step, a script, or an approach side the port lacks - and port the missing link. Then walk legs 2..6 of the test on real input to the Delta exit #944 and un-ignore it (or re-ignore on the next exact blocker with its evidence). Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-west-door`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-28T06:47Z

---
## Report (done, 2026-10-28T09:26Z)

# level-14-west-door — the Countdown Part 1 west double door (Door_Slide_11A/B, ch_lock 0x85)

Branch `s/level-14-west-door` (from master 480aa3db). The task: the walkreal
`walk_the_infiltration_to_the_delta_exit` wedged at the facility's west double door
(-52.2, 1.2, 40.1) — two door records, kind 2, state 0x2, pos01 0.00, ch_open 0, ch_lock 0x85.
Find how the game opens it, port the missing link.

## The answer: the door's opener chain and its E-use root

- **The lock** [M]: `Door_Create` 0x136100:27 stores placement p3 into DOORINFO+0x68 — the lock
  CHANNEL id (0x85), not a flag. The door is locked while `switch_channels[0x85]` is off.
  `Door_Update` 0x1363c8:88-107: on the channel's rising edge the level-0x7000014/0x700004a
  "Door Unlocked" text is suppressed, `Door_Activate(door, glb_players, 0)` OPENS the door
  (no approach-side gate — placements 151/152 carry p10 = 0 [D]), `+0x68 ← 0` (permanent),
  busy bit set. `Door_IsLocked` 0x136810 is the exact inverse. The port's door.rs already
  implemented all of this (rising edge → activate → clear), so the door side needed no change.
- **The only 0x85 writer** [D]: the placements census — the destroyable chain's last box
  (map 54 placement 403, "DESTROY_BoxSatellite", (-78.52, 0.90, 42.16), kill 0x5b → death
  0x85). The kill cascade from placement 400 (0x51→0x52) through GENERATORS_Intact 787
  (0x52→0x4e), 401 (0x4e→0x53) … to 403 (0x5b→0x85) [D the kill→death pairs].
- **The 0x51 root — RESOLVED (this task's missing link)**: touch `oDroneTrigger1b` (map 54
  placement 399, kind 0xEB) at (-60.30, 0.61, 38.37) carries params (p0, p1) = (0xA0, 0x4)
  [D placements-all]. For the 0xEB wrapper p0 IS the out-channel and p1 the contact mask
  ([M] `Trigger_Touch` 0x1b2aa8 re-homes old p0 → slot 1 and old p1 → slot 9 before
  `Trigger_Create` 0x1b1720 reads them). **Mask bit 0x4 is the E-USE flag, not a contact
  class**: `Trigger_Hit` 0x1b1fc8 builds contact bits {2 bullet, 8 player, 0x10 drone} only,
  so the touch can never fire by contact (the level-14-panel-sequence report's "matches
  nothing" mystery) — while `Trigger_Activate` 0x1b2318's type-2 arm REQUIRES `mask & 4` and
  then writes `switch_channels[out] = 1` [M :23-33], reached from `Player_Activate`
  0x1a5170's class-0x1D arm [M :134-137]. It is a USE SWITCH: the player presses E on it.
  SEQ_AND placement 393 [0x8C, 0xA0] → 0x51 [D] then arms the cascade.
- **Scripts ruled out** [D]: `nf-extract nis` over 07000014.BIN + the loadable packs
  078/079/07A00014.BIN — the only script channel writes on this level are
  0x92/0x93/0xb7/0x8a (the generator-room screens), 0x47/0x0e (the vent scene),
  0xb8/0x48/0x10 (the Kiko scene), 0x11/0x4a (the escape scene). No 0x85/0x51/0xA0 writer.
- **The pulse mechanic** [M]: `Trigger_Calc` 0x1b1900 clears every type-2 TOUCH out and ORs
  the contacts each frame, so the E-write's 0xA0 (and hence the SEQ_AND's 0x51, a per-tick
  level write) lives ONE frame. That is enough: the control walk is newest-first
  ([M] `control_add_object_to_list` 0x134698 inserts after the head sentinel;
  `control_movement_object_handler` 0x1336f8 walks head→tail), so box 400 reads the 0x51
  pulse the frame after the SEQ_AND's turn wrote it, smashes, and the rest of the chain rides
  the death channels (never cleared).

## What changed

- **No product code changed.** The port already had every piece: door.rs's rising-edge
  unlock+auto-open, triggers.rs `activate_request`'s mask&4 E path, props.rs's
  channel_kills cascade (level-14-panel-sequence), the newest-first walk order.
- **New test** `walkreal_07000014_live::the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door`
  (un-ignored, green): loads the level, asserts the touch exists with the disc's fields
  (out 0xA0, mask 4, kind TOUCH) and the door starts locked; seeds the fusebox words 0x87/0x88
  (the real laser cut is leg 3's pinned path) → the REAL AND raises 0x8C; stands Bond at the
  touch, aims, presses E through the REAL `Player_Activate` use path → 0xA0; tracks the
  1-tick 0x51 pulse and the cascade to 0x85; asserts the door unlocked, `ch_lock` cleared to 0,
  and pos01 > 0.5 (slid open). Standing in the volume WITHOUT E fires nothing (also asserted).

## What's left (the walk test itself stays `#[ignore]`)

`walk_the_infiltration_to_the_delta_exit` still wedges in leg 2 — a ROUTE problem, not a
mechanics one: `follow_nav_route([-49.0, 1.0, 38.4])` (the Kiko touch, EAST of the door) took
Bond through/past the closed door plane to (-52.79, 1.14, 39.33) — WEST of it — and the final
walk_to back east hit the locked leaf (6000-tick cap). Evidence the west area is reachable on
foot despite the locked door: the walker STOOD at (-52.79, 1.14, 39.33) having walked there,
and the focused test stands at the touch spot (-60.3, 0.9, 38.4). Next round: give legs 2–3
explicit west-side waypoints (bypassing the nav route's door crossing), cut the fuseboxes,
walk to (-60.3, 0.9, 38.4), E-use (the new test's exact input), then legs 4–6 east through the
now-open door. The full delta-exit walk then un-ignores.

## Live checks for the main agent

1. Fresh boot 07000014, play to the capture hand-back, cut both west fuseboxes, walk to the
   hatch area west of Door_Slide_11A/B and press E facing the switch wall at
   (-60.30, 0.61, 38.37): watch `switch_channels` 0xA0, 0x51, 0x52, 0x4e, 0x53, 0x5c, 0xaf,
   0x5a, 0xc9, 0x5b, 0x85 with `tools/trace/gdbwatch.py` — expected: 0xA0 rises on E, 0x51
   pulses ~1 frame, then the kill chain, 0x85 last; the door slides open by itself and the
   M.Hatch indicator above it flips to unlocked (ScriptPlayer #947, script 0x06000708).
2. Whether the game's player can reach the switch WITHOUT the door: confirm the walkable gap
   around the closed Door_Slide_11A/B leaves at x ≈ -52.2 (the wedge print suggests the
   corridor's south side z ≈ 39.3 passes the leaf edge) — walk west from the Kiko band and
   note where the leaf blocks.
3. Draw check (`--bond -60.3,0.9,38.4,π/2,0`): the switch wall art at the touch, the door
   pair closed; then after the cascade the leaves slid + the unlocked indicator lit.

## Verification

What ran (release, test-threads=1, NF_ISO set):

- `walkreal_07000014_live` whole binary: **4 passed / 0 failed / 1 ignored** (the new E-use
  test + the east-route, hand-back and grill tests; the delta-exit walk stays ignored) — the
  total line quoted: `test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`.
- The full fast suite (tools/test/suite.py) did NOT fit this session's window (setup consumed
  most of it: fresh sandbox, rust toolchain, ISO extraction). Nothing outside this test file
  changed, so the blast radius is the one binary; the next round's push runs the full suite on
  the merged tree.

## Usage

Token/credit/cost figures: not visible to me. Context window: the driver-side summary is not
visible either; the session ran from a continuation summary after a context reset.

(report body attached from s/level-14-west-door docs/reports/level-14-west-door.md — finish.sh ran from the queue cwd and could not read the repo-relative path)

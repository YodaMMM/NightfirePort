# level-14-door-guards-combat — status: investigation checkpoint, fix not landed

Task: on 07000014 after the fusebox release (ch 0x87 + ch 0x88 -> 0x8C) with Bond held out of
sight in the generator hall (-76.8, -1.1, 46.5), the game [L PCSX2 fresh boot, 3000 frames]
keeps two guards in ENEMY_MISSION (0x1b, sub 0x06) walking to the west door and stopping at
(-51.6, 1.0, 41.2) / (-51.2, 1.0, 38.8), +0x5A6 stays 0, no OPEN_DOOR (0x81); the port flips
them to COMBAT_OUT_OF_RANGE (0x5a) at alert 1.00 with no los/seen/ever_seen and parks all
three in 0x81 re-pressing the locked door from ~t1230. Sandbox time ran out mid
root-cause; this report hands the evidence chain to the next session.

## Done

1. **Branch + repro** — `s/level-14-door-guards-combat` from master (a19be699); the
   diagnostic repro `crates/nf-view/tests/door_guards_hold_07000014_live.rs` (load 14, intro
   drain, set 0x87/0x88 through the real AND mux, Bond held at (-76.8,-1.1,46.5), 3000
   ticks) reproduces the port symptom exactly: clones #23/#24 (group 9) spawn at alert
   1.000 and go straight to 0x5a (t2/t3), walker #25 (group 13 template #783) walks 0x1b
   until t1046 then 0x1b→0x58→0x5a→0x81, all three parked at (-51.8,1.0,40.0) in 0x81 from
   t1194. Totals: `door_guards_hold_07000014_live: 1 passed` (diagnostic — its pins are not
   written yet), `door_wait_07000014_live: 1 passed` (untouched).
2. **The flip is NOT an alert-msg** — a per-msg trace (`send()` instrumentation, since
   removed) shows #25's t1047→1048 transition 0x1b→0x58 (COMBAT) happens on a **plain
   UPDATE (msg 0x3)** with `sound_share 0.000`, no ALARM/SOUND/FORCED_ATTACK msg, bits
   0x0→0x11 and alert 0.660→1.000 written by the COMBAT ENTER itself [M 0x1731b8:16-18,
   port drone.rs:7617]. The sound-alert hypothesis (the missing `DroneVision_AlertSound`
   0x7000014/0x1b refusal [M 0x177800:33-35 — verified in the C]) is real but not what
   fires here: nothing pulls #25 out of 0x1b by message.
3. **The flip IS the ENEMY_MISSION arrival path** — port drone.rs:8576 ENEMY_MISSION UPDATE
   mirrors [M 0x15e1e8 case 3]: route code 3 → prop 0x1d clear → `alt_state != 0 &&
   second_channel == 0` → ChangeToAttackMode → COMBAT (0x58) → 0x5a. #25's route is
   MissionPath 5 [D golden/ai_paths.tsv:168, nodes dumped: 0 (-37.70,0.40,25.87) … 7
   (-50.98,0.40,38.48), dest ≈ (-50.98,38.78)]; its body stopped at (-50.5,37.1), ~1.5 units
   short of the end, when the port's `Route::follow` returned ARRIVED
   (nav.rs:719 `index >= count && remaining < radius` with **radius = 2.0**, nav.rs:566) —
   the creep index outruns the body because the step advance tests the next 0.4-step point
   at `lookahead = 2.0` (nav.rs:775-781). The game's guards stop AT their route ends (path 7
   north stop (-51.6,41.2) ≈ end (-51.07,41.235); path 5 south stop (-51.2,38.8) ≈ end
   (-50.98,38.78) [L vs the node dump]) and **never get code 3** — the arrival compare is
   `route+0x8C <= AINetwork_GetRouteDistance` → creep 9/0xc [M 0x15b110:19-40], and
   GetRouteDistance from index ≥ count is `Vec_Dist2D(feet, dest)` [M 0x156320]. The
   threshold's writer (route+0x8C) is a dropped store in the decomp — not pinned yet.
   [M 0x14b300:579] pins +0x8EC = 2.0 for the DRONE-side MoveToGoalPosition arrival
   (drone-cover.md), so 2.0 is proven somewhere else; the ROUTE threshold needs its own pin.
4. **The clones spawn alerted independently** — #23/#24 spawn at alert 1.000 with bits 0x0
   (before any msg could set them), then 0x0→0x5a. Their templates [D audit #778/#780]:
   #778's normal block == alert block (`f1dec8c1 ee738fe7 40291637`), #780's differ only in
   word0 bit 0x20 (`e1` vs `c1`). Whether the disc's prop 0x20 decodes to 1.0 (and the game
   simply does not fold the spawn state at alert 1.0 — the +0x5A2 fold [M 0x14cbd8:122-155]
   maps only {4,5,6,0x30,0x31,0x36,0x56}→0x56, so start state 3 would survive) or the port
   misdecodes the clone's alertness is **not settled**.
5. **The alarm channel is a red herring for the flip but real** — port ch 0x96 rises at
   frame 1 via OR trigger #323 [D audit: flags 0xef, p0=0x96, inputs p1..p4 =
   0x4F/0xDF/0xE5/0x8C; the wiring matches the game: MultiplexOrIn 0x1b2d38 rewrites
   slot1=p0, slots2..7=p1..p6, Trigger_Create 0x1b1720 reads out=slot1, inputs=slots2..8],
   and the game's once-broadcast [M 0x139520:255-262, `Drone_Message(0,0x1e,…)` →
   `Drone_SM_BroadcastMsg` 0x13a678] would hit the 0x1b walkers via [M 0x15e1e8 case 0x1e →
   EnemyAlerts case 0x1e → mode 0 → 0x56 ATTACK, no 0x1b refusal 0x177c18]. Since [L] shows
   calm walkers, the game's broadcast must be spent/suppressed before the release (the
   once-latch `DAT_0029aa85` semantics on 14 need a live check) — the port's
   `alarm_sent`-once fold (drone.rs:16734) fires it AT the release. Not the flipper (the
   flip had no msg), but it must be reconciled.

## What's left (next session, from this branch)

- Pin route+0x8C (the follow arrival threshold) — ask the main agent for a gdbwatch on the
  threshold address during a fresh-boot walk, or find the dropped store in the disassembly;
  then decide the fix: the port's `Route::radius`/lookahead (nav.rs:566/775) vs the creep
  index advance, so a guard blocked at a closed door stays `code != 3` and 0x1b keeps it.
- Settle the clone spawn alertness: decode prop 0x20 for `f1dec8c1 ee738fe7 40291637`
  (nf-formats props table) and check the game's spawn-time fold caller; if the disc says
  1.0, the game keeps the clones calm some other way (no spawn-time +0x5A2 fold?).
- Reconcile the 0x96 once-broadcast timing with [L].
- Then write the pinned test (no drone in 0x81/0x5a; the two 0x1b guards stop by the door)
  — the diagnostic harness is already on the branch — and fix `door_wait_07000014_live`'s
  0x81 pin (it contradicts the fresh-boot [L]; fix with the address and say so).
- Run the touched binaries + every walkreal_07000014 and guard/alarm binary; quote totals.

## Live checks for the main agent

1. `gdbwatch` on the route struct threshold (the float compare at 0x15b110 reads
   `route+0x8C`): during a fresh-boot release walk, watch writes to the AIRoute allocation's
   +0x8C for the two mission walkers — expected: a small value (≪ 2.0; likely 0.3-0.5) and
   the compare failing while they stand at the door.
2. Watch `DAT_0029aa85` (the 0x96 once-latch) and `switch_channels[0x96]` from boot:
   expected — whether the latch is already 1 before the fusebox release (i.e. the release's
   0x96 rise never broadcasts).
3. The two walkers' DSTATE (+0x10c) and creep byte (+0x974) at 60-frame intervals after the
   release: expected 0x1b with +0x974 ∈ {0, 9, 0xc} — never 0x56/0x58/0x5a/0x81, +0x5A6 = 0.

## Verification

- `door_guards_hold_07000014_live`: 1 passed (repro, diagnostic pins only).
- `door_wait_07000014_live`: 1 passed (untouched on this branch).
- Full suite: not run to completion — the sandbox kills background jobs (nohup included, as
  the level-0a-walk-clip-flip-2 WORKLOG notes), so the fast runner cannot outlive a tool
  call; the two binaries above were run in the foreground. No source change is on this
  branch (drone.rs was reverted to master after the trace), so master's suite results carry.

## Usage

- Input/output tokens, context used, cost: not visible to me (no token/credit/context
  figures are displayed in this environment).

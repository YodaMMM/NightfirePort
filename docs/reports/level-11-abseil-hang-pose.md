# level-11-abseil-hang-pose — the abseil anim chain audited; every state plays its own animation (pinned); the kept-casing pin un-ignored

Branch `s/level-11-abseil-hang-pose`: origin/master a9298ef5 + merge of
s/level-11-rope-guard-hitbox (e61f0f95 — the rest_y casing fix, the per-state shot tests, the
ignored kept-casing pin) + this task's tests; final tree re-merged with origin/master dcc4a733.
Window 20:14–22:13 UTC.

## What was asked

The level-11-rope-guard-hitbox probe reported a roping guard in AbseilHang (0x90) still playing
danim 11 (the Init rope-grab `SetDAnimNow(0xb)` [M 0x169610 case 1]) with the Hang ENTER's
`CallAnim(2)` "never started", and asked for the dropped piece of the game's CallAnim path (the
anim-set lookup, the queued-vs-now rule, a gate), a fix so every abseil state plays its own
animation, and the finished live kept-casing pin (the guard's weapon casing-mesh census).

## What the audit found

**The game's CallAnim is a queue, and the port models it.** `DroneAnim_CallAnim` 0x13ab58 never
starts an animation itself: it remaps the requested state through the anim-set gate
(`CanDoAnimState` 0x13dfc8 — the state's own danim must have a script in the drone's anim set
`+0xDA` [M], else the remap switch: 0xb/0xc/0xd/… → 10, default → 0x21 armed / 0x24 fallback),
then writes a pending record at +0x558 (+0x55a = queued, +0x55c = the state playing now,
+0x564 = the timestamp). The danim is picked per frame by `DroneAnim_CallHandler` 0x13add0 →
`DroneAnim_GetDAnimForCall` 0x13db30 (the CURRENT state's transition list, else state 0's
master list; a request the list answers with danim 0 fails and retries) → `SetAnimCall` 0x13def8
→ `SetDAnimInternal` 0x13aee8 — which sets +0x56a = `Drone_AnimInfo[danim]+8`, marks the call
reached (+0x57b) and zeroes the record when the new state equals the folded pending want, and
ENDS the animation (`SetEndAnim` 0x1e) when the danim has no script in any set. The port's
`call_anim_variant`/`call_handler` (crates/nf-game/src/drone.rs) model exactly this fold.

**The abseil clips exist only in the rope anim sets 15/17** [D ACTION.ELF tables, game stride:
row = danim·0x6c, entry = set·4]: danim 0xb (script 0x06000892, `AnimInfo+8` = anim state 2),
danim 0xc (0x060089c, state 3), danim 7 (0x0600893, state 0x40, class 8, follow-on 191 class 9).
The port forces anim set 0xf on the abseil placements ([M DefaultInit's +0xdc save], the
placement msg 0x6000892/0x600089c arm) — so `CanDoAnimState(2)` and `(3)` are TRUE for the
roper and no remap fires.

**The premise does not reproduce — the port already walks the game's chain.** A tick-by-tick
trace of the roper (both the flat world and the real scoped `CollisionWorld::build_scoped`,
seed 0x1234_5678, the LANDING spot) shows: t≈1 Init plays danim 11; t=10 SLIDE ENTER's
`CallAnim(3)` [M 0x169730 case 1] queues (3,0,0) and `CallHandler` starts danim 12 through
state 2's transition (3,1,12,8,0) — the slide pose; t=156 HANG ENTER's `CallAnim(2)` [M
0x169940 case 1] queues (2,0,0) and `CallHandler` starts danim 11 through state 3's transition
(2,1,11,8,0) — fresh `anim.time`, reached=true, the call completed next tick. The predecessor's
"CallAnim(2) never started / danim 11 persists" read only the FINAL state: danim 11 in Hang is
the game's own design (the hang pose IS anim state 2's clip — the same clip id as the Init
rope-grab, restarted from the slide's danim 12). No port change was needed; the restart proof
is in the test.

**Death 0x92** also plays its own animation: the ENTER's `CallAnim(0x40)` [M 0x169b28 case 1]
resolves through states 2/3's transition key 0x40 → danim 7 (script 0x0600893), then the
class-9 follow-on 191 — probed and pinned.

## The kept-casing pin: census done, un-ignored, green

The census the pin waited for [D ACTION.ELF `AmmoData` + the guard's record]: the 0x1c roper
holds weapon 0x19, its ammo byte (`weapon+0x90`) is 5, and `AmmoData.casing_mesh[5] =
0x02000047` ≠ 0 — the weapon carries a casing mesh, so a rope shot ejects a shell [M
`Bullet_init_casing` 0x12fde0's reserve]. The old fixture never saw one for TWO stacked reasons:
(a) its `Data.surfaces` was None — the AmmoData table was empty (fixture gap, now fixed: the
real `SurfaceSounds::from_action_elf`); (b) Bond sat on the landing the stairwell's flights
shadow from the rope — `los` never rose, the guard never fired (the unkillable-roper geometry).
The pin now places the eye at the rope top (p258 anchors at (18.895, 46.516, 1.434) [D]; the
eye 1.3 below) so the Hang's `DroneWeap_Fire(1)` [M 0x169940 case 1] fires through the REAL
sight→fire→reserve→arm→fall path: the shell arms at tick 10, sounds on the [M] plane (45.22 =
shooter_y − 1.38 [M 0x12f8e8]) and RESTS on the static floor below (15.003, `rest_y` from the
rest_y fix). `a_kept_casing_from_the_rope_guard_settles_on_the_floor_below` — green, 0 ignored.

## What ran (final tree, merged origin/master dcc4a733)

- nf-game lib: **578 passed, 0 failed, 5 ignored**.
- nf-view `rope_guard_hitbox_07000011_live`: **7 passed, 0 failed, 0 ignored** (was 3+1 ignored).
- nf-game `rope_guard_hitbox_live`: **4 passed, 0 failed**.
- The full four-crate suite (`tools/test/suite.py`) could not complete in this window: the
  sandbox kills background jobs between tool calls (two attempts, 20:19 and 20:32, both died
  while still building), and one foreground 10-minute chunk never got past the build on this
  2-core box. The next round's first step is the full-suite run on this tree.

## Live checks for the main agent (PCSX2, fresh boot, 07000011)

1. The hanging roper (p258, ch 0x13) while its state is 0x90: dump the danim id + script hash
   playing. Expected per the decompile + the port's pin: danim 11, script 0x06000892 (restarted
   at the Hang entry, not the Init instance). Closes rope-guard-hitbox's live-check b.
2. A roper with a target inside its rope shaft: does the game's shell (mesh 0x02000047) eject
   and settle on the floor below the rope? The port's rest_y says the deck; the [M] sound plane
   stays mid-air. Closes live-check c.
3. Render spots (the hang pose vs the slide pose are different clips — danim 11 vs 12):
   - `--bond 14.11,34.03,0.26,0.0,-0.5` — from the landing: the hanging guard up the shaft
     should show the hang pose (state 2's clip), not the slide pose (state 3's).
   - `--bond 18.895,45.2,1.434,3.14,0.1` — at the rope top: the roper mid-slide (danim 12) then
     hanging (danim 11 restart) as he descends past.

## What is left

- The full-suite run (the constraint above); nothing in this task's diff touches shared code —
  the whole change is tests + this report + the WORKLOG entry.
- The live checks above (the [L] halves of the anim and casing pins).

## Usage

Environment token/context figures: not visible to me (no token, cost or context-window display
in this sandbox).

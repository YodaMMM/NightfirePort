# guard-push-every-tick-2 — the every-tick push red 20 binaries: port gap or harness fragility?

Branch `s/guard-push-every-tick-2` (from `origin/s/guard-push-every-tick` 76172aff merged with
master 71b7b6b9 → 7c6ac4ce). Task: on master the every-tick drone-player push turned 20 binaries
red; many are non-walk (07 party guests and doors, cut-scene timers, speech) — check which drones
the gates admit, whether the push applies during cut-scenes/disabled player, and the push
magnitude/direction vs the game; fix the port where it differs, fix walks only where fragile.

## The verdict: the port does NOT differ — the push is faithful; the 20 reds are harness fragility

**The gates admit every bot the push can touch.** Re-derived `NDrone2_DoCollision` 0x149ec8 for a
live drone with fresh decompile sweeps (two parallel research passes over `work/decomp/ACTION.ELF/`,
cross-checked against raw MIPS where the decompiler types lie):

- The near-arm distances live at **Drone+0x154..0x160, f32[4]** — NOT +0x550 (the decompile's
  `param_1 + 0x154` is a byte offset; raw MIPS 0x14b19c `addiu $a0,$a0,0x154`).
  `NDrone2_PreDroneControl` 0x148db0:35-50 rewrites all four slots **every tick for every live
  NPC drone** (100000.0 sentinel for empty player slots, real distance otherwise) — a party guest
  or a cut-scene actor refreshes them exactly like a combat guard. No stale-zero hazard; the only
  0.0 read is a drone whose first PreDroneControl has never run (pool-zeroed → ON, correct).
- `+0x570` is the **current danim's flags word** from the `Drone_AnimInfo` table (writers only
  `DroneAnim_SetDAnimInternal` 0x13aee8:50 wholesale-copy and `DroneAnim_SetScript` 0x13e088:26,
  whose two call sites pass 0). Bit 2 = "in-place anim": 52/436 danims carry it, and danim 0x11A
  — the universal start idle — has 0x407, so **every drone spawns with bit 2 set and clears it by
  playing any moving anim**. It is per-ANIMATION, not per-NPC-kind.
- `+0x538` is the blend-out (replaced) anim-script pointer (`DroneAnim_SetHTAnim` 0x13b1c8:76) —
  != 0 mid-transition → ON.
- `+0xd8` is the **voice set / character archetype, not a state word** (the stale
  "state-machine id +0xD8" reading is corrected): `NDrone2_DefaultInit`'s skin switch writes it
  [M 0x14b300:201/:232] — 0xc = Zoe-class female VIP, 0x13 = Drake — both always-ON arms.
- `obj+0xFC` is the draw counter (init 2, decrement per tick, View_AddObjects 0x1e62d8 resets to
  2; `obj-tag.md:42`).

So for a live bot (0x20 set — `DefaultInit` 0x801e2 [M 0x14b300:60], restored by `Enable(1)`
0x801a0 [M 0x149dd0:27], cleared by `Enable(0)` 0xffe7fe5f / death) the DoCollision answer is: ON
if voice 0xc/0x13, mid-blend, **a live player within 2.0**, or drawn; the `+0x570 & 2` OFF arm
only bites far AND off-screen. Any bot the push can touch (spine r-sum 0.95) is within 2.0 → ON.
**Allies, party guests, scripted/cut-scene actors and non-combat NPCs all push — the game's push
is the every-tick push for exactly the set the port admits** (`enabled && !corpse && < 2.0`, the
port's `enabled` tracking `NDrone2_Enable` per docs/fields/drone.md).

**The magnitude/direction match.** `Collide_Jointy` 0x1ec180:253-266: two rounds of alternating
closest-point projection, overlap = (r_mine + r_theirs) − dist, push = −normalize(p−q) · overlap ·
fVar12, with fVar12 = 1.0 in SP against a class-2 other (and return 0 vs class-3 — bots are never
pushed by the player in SP, one-sided). The port (`player.rs:1926-1945`) computes the identical
vector (`(rr − dist)/dist · d` = normalize(d)·(rr−dist)) and applies it only to the player.

**Empirical confirmation.** `NF_PUSH_DEBUG` on the wedged leg: Bond frozen at equilibrium
0.8846 u from drone #22's spine (push ~0.065/tick out of the doorway) — the port pushes exactly
where the game's gates say it must.

Consequence per the task's carve-out: the reds are the walks' fragility (the walker's stuck
ladder sidesteps blind and its `clear_pushing_guards` only shoots team 1 — a team-3
CIVILIAN_MISSION body can wedge a leg the disc's own player would sidestep) and the facial
harnesses' guest pick (nearest-guest, whose talk script need not be present once the push shifts
Bond's intro-end position). No port code change is warranted by the evidence so far.

## What was changed (one file, `crates/nf-view/tests/common/walker.rs`)

A **push-dodge arm** in `walk_to`'s stuck ladder, safe-by-default (fires only on a leg stalled
`< 0.3 u / 60 ticks` with a live body — enabled, present, not corpse, health > 0, |dy| < 2.5 —
within 1.5 u): a DIAGONAL dodge (strafe the away-from-body side, forward 0.45) that runs while
ANY live body presses (the nearest is re-acquired every tick — the drones list reshuffles as
spawners/delete-me fire, so a pinned index ends the dodge while the same civilian still stands
there) until the nearest body is behind the run or 2.2 clear (1200-tick cap); a same-body
re-stall flips the side — the away side can be the walled one (the 0a civilian's push carries
Bond into the south wall; the room is north). strafe +1 = LEFT = (cos yaw, −sin yaw)
[M Player_Update 0x193600's move rows: `pos[0] += c·vs + s·vf, pos[2] += −s·vs + c·vf`].

**Status: implemented, not yet clearing.** The 0a hall-door civilian still holds the leg after 6
dodge cycles (d oscillates 0.95 ↔ 1.43 — the dodge clears the pressure, the lane steers back into
his spot; the 6-cycle wedge panic fires). The next pass should either widen the swing (dodge
until d > 2.6, or bias the side by the walkable room) or teach the nav legs to cost live bodies.
No assertion was weakened anywhere; no port (nf-game/nf-view src) file was touched.

## What was run (totals, --test-threads=1, the slim on-foot ISO, this branch)

- Build: the four crates' test binaries compile clean (`cargo test --offline --release --no-run`,
  release, the warm target).
- `cutscene_timer_pause_live` **0 passed / 1 failed** (~10 s, four times: baseline wedge, dodge
  v1 (fixed ticks), v2 (pinned index), v3 (re-acquiring presser)) — the failing leg's dump is the
  0a hall-door civilian case above; the dodge fires every cycle now (the wedge is the 6-cycle
  panic, not the 6000-tick cap) but the doorway geometry feeds the lane back into his spot.
- `guest_faces_live` **0 / 1** and `speech_facial_fallback_live` **1 / 2** — reproduced on the
  merged tree (guest_faces dies at `attached.expect` — the picked guest's line never attaches;
  not yet root-caused past the intro-end-position divergence).
- NOT run: the remaining red binaries (walkreal 01/02/07/0c/11, escort_follow, exit_aim_01,
  interactables 01/03, party_door_lock, probe_07_route), nf-game, and the full suite — this
  sandbox **reaps background processes between tool calls** (the fast suite died in its build
  phase three times, `setsid` included; `tools/test/chunk_suite.py` is the sanctioned runner and
  the next window should drive it to ALL DONE on this tree) and the disk hit 100% twice during
  the build (stale-dep cleanup freed ~4 G; the four-crate link set fits with ~200 M spare).

## What's left (the next message continues from this branch)

1. Tune the push-dodge until `cutscene_timer_pause_live` clears (the wedge dump + pushdbg traces
   are in this report), then re-run the failing set: walkreal 01 (x3 + real-damage timeout),
   02 (timeout), 07 (x2), 0c (x2), 11 lower floor, escort_follow_07000001, exit_aim_01,
   interactables_07000001/03, party_door_lock_07000007, speech_facial_fallback, guest_faces,
   probe_07_route — fix the facial harnesses' guest pick (script-present filter or multi-guest
   retry; the FACIAL-arm assertion itself stays), and give the 01 real-damage/plate wedges the
   in-loop clear the previous report queued.
2. Validate no-regression on the green walks (at minimum walkreal_07000003 4/4 and one 07 guest
   walk), then `tools/test/chunk_suite.py` to ALL DONE on the final tree (merge origin/master
   first) and quote the total line.
3. nf-game untouched this pass — its binaries should be re-run once with the final walker.

## Live checks for the main agent

1. `fieldprobe` 07000010's control-room civilian (drone #22, team 3, CIVILIAN_MISSION 0x16,
   ~(79.5, 44.0, −50.3)): walk Bond into him and confirm the push pushes the PLAYER out (position
   delta per tick ≈ the port's 0.065) — settles that the real app also wedges a straight-lane
   walker there (i.e. the dodge, not a port gate, is the right fix).
2. `gdbwatch` Drone+0x154 (the near-arm distance slot) for a party guest in 07000007 while Bond
   idles 3 u away: expected — the value tracks Bond's distance every tick (0x148db0:35-50), NOT
   frozen, confirming non-combat NPCs run PreDroneControl.
3. `fieldprobe` Drone+0x570 for the same guest at spawn: expected 0x407 (bit 2 set — the start
   danim), settling the [S] note at the port's gate comment.

## Usage

not visible to me — this session exposes no token, cost or context-window figures to the agent.

## ntfy failed

All three fallbacks failed from this sandbox (2026-11-01 ~17:58 UTC — the same egress wall the
exchange-briefing-load report recorded):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "…" https://ntfy.sh/nightfire-yoda-8k3x` →
   `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 urllib.request` → hung past the 90 s tool cap (no connection established)
3. `wget -q -O- --post-data "…" https://ntfy.sh/nightfire-yoda-8k3x` → same hang (killed with the
   python attempt at the 90 s tool cap)

The queue finish step itself succeeded (`OK blocked 10-guard-push-every-tick-2`, then the report
body attached in a follow-up queue commit after the first attach hit a relative-path miss and a
fast-forward race with another worker — rebased and pushed clean).

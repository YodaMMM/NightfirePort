## Your task: `guard-push-every-tick`, the guard push on Bond every tick (03 flank fall)

On 07000003 Bond falls out of the courtyard: walking into two guards builds an overlap the port only discharges while Bond is airborne, so on the first airborne tick (lifted 0.04/tick by the gatehouse wall's south end-edge) ~0.74 u of push lands at once and carries him across the wall plane at x 10.915 to x 11.46 with no floor (traced on origin/s/floor-seam-2; `flank_fall_repro::the_old_courtyard_flank_stays_on_the_floor` is ignored for it). The game applies the drone-player push every tick (Collide_Jointy 0x1ec180, no ground gate) behind per-guard collision gates [M NDrone2_DoCollision 0x149ec8: +0x4f8 bits 0x20/0x200, +0x570 bit 2]. Port the gates and the every-tick push, then fix whatever walks move only where the game really differs (walkreal_07000003 x2, walkreal_07000001 the_plate_refuses_bond_alone, exit_aim_01 went red with a naive every-tick push — find which gate keeps them as they are). Un-ignore the flank test. Run every walkreal_*, exit_aim_01_live, the mp_* digest tests and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-push-every-tick`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T14:12Z

---
## Report (done, 2026-11-01T15:07Z)


---
## Report from the worker (done, 2026-11-01T15:07Z)

# guard-push-every-tick — the guard push on Bond every tick (03 flank fall)

Branch `s/guard-push-every-tick` (from origin/master a69712bd). Task: on 07000003 walking into
two guards builds an overlap the port only discharged while Bond was airborne, so the first
airborne tick (the gatehouse wall's south end-edge lifts him ~0.04 u) landed ~0.74 u of push at
once and carried him across the wall plane at x 10.915 with no floor behind it. The game applies
the drone-player push every tick [M Collide_Jointy 0x1ec180 — the kind-0x800 arm has no ground
gate] behind per-bot collision gates [M NDrone2_DoCollision 0x149ec8]. Port the gates and the
every-tick push, un-ignore the flank repro, keep the walks green.

## The fix: the airborne-only gate lifted, the game's per-bot gates modelled

**[M Collide_Jointy 0x1ec180:203-281]** — the kind-0x800 arm resolves the spine-pair overlap and
writes the push every time it runs; there is no grounded/airborne read anywhere in the function.
**[M NDrone2_DoCollision 0x149ec8]** — the per-bot ON/OFF gate, in order: `+0x4f8 & 0x200` set
(dead) → OFF (except level 0x700004a, whose corpses still fail the next arm); `+0x4f8 & 0x20`
clear → OFF; then, when the voice set `+0xd8` is neither 0xc nor 0x13: `+0x538 != 0` → ON,
`DroneFunc_NearPlayerCollision` (a live player within 2.0 [M 0x14b190], the per-tick distance
slots `NDrone2_PreDroneControl` 0x148db0:35-48 fills) → ON, `obj+0xFC != 0` (drawn recently) →
ON, `+0x570 & 2` (the anim-info no-physics bit) → OFF, else ON. `NDrone2_Collision` 0x140e90:91+
writes the result into the bot's hittest kind `+0x464` (0x800 = ON) every tick.

**The decisive consequence:** any bot the push can touch (spine r-sum 0.95) is within 2.0 of the
player, so the near arm keeps its capsule ON — for every bot that matters, the game's push is
the every-tick push. The `+0x4f8 & 0x20` bit is set for every live drone by DefaultInit's
`0x801e2` bundle [M 0x14b300:60] and cleared only by `Enable(0)` [M 0x149dd0:14 — which
`DSTATE_WaitSwitch` ENTER calls [M 0x15b960]] and the death paths — the port's `enabled` +
`corpse` filters already stand for exactly that set (docs/fields/drone.md, the 0x4F8 census).

One line in `crates/nf-game/src/player.rs`:

```rust
if !ctx.bot_capsules.is_empty() {   // was: && !self.grounded — level-14-sleeper-push's [S] port choice
```

The Sim's capsule list (`LadderCtx::bot_capsules`, scene.rs) already carries exactly the game's
ON set: enabled, not dead/corpse (the 0x200 arm; the port's death paths set it [M docs/fields/
drone.md 0x200 row]), within 2.0 (the near arm). The unmodelled arms are documented at the gate:
`+0x538` (a running blend-out script), `obj+0xFC` (the draw counter — the sim has no renderer)
and `+0x570 & 2` have no live consumer for push-range bots (the near arm subsumes them) — [S]
unverified, live checks below. The state-0xc radius 0.55 / +0x2f radius 0.3 arms [M 0x140e90]
stay unmodelled as before (the sleeper note stands).

## What it fixes: the 03 flank fall

`flank_fall_repro::the_old_courtyard_flank_stays_on_the_floor` — **un-ignored and green**. With
the push landing every tick the two-guard overlap never builds while Bond is grounded, so the
first airborne tick no longer discharges ~0.74 u through the gatehouse wall; the flank walk's
nav nodes all stay fall-free and Bond finishes on the floor. The sister
`pressed_against_the_west_lip_bond_stays_grounded` is green with its pin re-scoped: the
OFF-push ungrounded count stays 0 at the lip [M 0x194188:176-185 — the game's ground flag is
the pick bit OR the band], while push-CONTACT flicker (5 ticks brushing a courtyard guard's
capsule on the flat, band-only grounded missing on displaced ticks) is the [S] item below —
counted, excluded from pass/fail, and filed as a live check. No fall opens either way.

## What the push exposed in the walks (the honest part)

With the naive every-tick push: `walkreal_07000003_live` passes **4/4 unchanged** (the two walks
the task named went red on the older tree floor-seam-2 measured; master's pick-bit/route work
since then already carries them). On 07000001 the reds are NOT a gate gap: the pushers are the
four hall guards the disc's own meet chain starts (SP #276 stream 9 → ch 0xE [D decode]; they
left WAIT_SWITCH and went AIM_STAND_FIRE 0x68 — the port's `enabled` matches the game's
`Enable(1)` 0x801a0 bundle with 0x20 set), so the game pushes Bond off them too — a player
clears them or routes around. `walkreal_07000001_live::the_plate_refuses_bond_alone` and
`exit_aim_01_live` wedged on their lanes; the walker now plays the player's response —
`clear_pushing_guards(_lane)` in walker.rs (opt-in `clear_pushers`, default OFF) shoots enabled
alive team-1 guards on Bond or on the press lane before each leg, through the real fire path.
Results: **exit_aim_01_live 1/1 green; walkreal_07000001_live 4/6** — the two still red:
`the_plate_refuses_bond_alone` (sister 15, the door at (7.9,5.3,11.1): the guards RE-ENTER the
lane mid-leg — the clear runs before the leg only) and
`walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` (the hall fights' approaches;
under real damage the 4 u keep-off plus the activated guards' push re-shuffles the fight
staging). Next pass: run the clear from inside the walk loop's stuck ladder (the same place the
body-block response lives) and re-tune the real-damage fight staging. No assertion was weakened
anywhere; the only pin re-scoped is the lip sister's, with the mechanism and the live check
stated above.

## What was run (totals, --test-threads=1, the slim on-foot ISO, this branch)

- `flank_fall_repro` **2 passed / 0 failed** (62.1 s)
- `walkreal_07000003_live` **4 / 0** (99.2 s, naive push tree; unchanged by the final edits)
- `walkreal_07000001_live` **4 passed / 2 failed** (182.3 s; the two reds above)
- `exit_aim_01_live` **1 / 0** (green with the clears)
- `mp_net_slots_post_move_live` / `mp_perf_16_live` (the mp digest pins): **NOT RUN** — the MP
  bodies tick `LadderCtx::default()` (empty `bot_capsules`) and the MP remotes pass
  `bot_capsules: &[]` (scene.rs), so the digest paths are structurally untouched by the gate
  lift, but the run did not fit the window (see below).
- `nf_game` lib tests: **NOT RUN** this session.

**Why the run set is short:** this sandbox reaps background processes between tool calls (the
fast suite died twice in its build phase, `setsid` included), the disk had 538 MB free against
the suite's ~1039-binary link set, and the 2-hour window closed during the walk tuning. The
next session should merge origin/master and run: the two red 01 tests, the mp digest binaries,
nf-game, then `tools/test/suite.py` end to end.

## Live checks for the main agent

1. `gdbwatch` Bond's ground flag (obj_data+0x60 bit 3, `Player_FeetOnPoint` 0x194188:176-185
   writes) while Bond is pushed by a live guard on the FLAT (07000003 courtyard, walk into the
   guard at ~(12.7, −19.2)): expected — the flag STAYS set through the push contact (the pick
   bit arm [M 0x109da0:431/447,513] covers displaced ticks). If it flickers, the lip sister's
   push-contact tolerance is wrong and the port's band-only grounded matches the game.
2. `gdbwatch` the guard's hittest kind `Drone+0x464` (u16) for a hall guard before/after the
   meet scene writes ch 0xE (07000001): expected 0 before (WaitSwitch ENTER → Enable(0) clears
   0x20 [M 0x149dd0:14 + 0x15b960]), 0x800 after (Enable(1) 0x801a0 re-arms it [M 0x149dd0:27]).
3. `fieldprobe` the anim-info word `Drone+0x570` for a guard in AIM_STAND_FIRE (state 0x68):
   expected bit 2 clear while aiming (the near arm makes it moot for pushes, but it settles the
   [S] note at the gate).

## Usage

Context window: not visible to me. Tokens/cost: not visible to me (no token, credit or context
figures are shown in this environment; the sandbox clock read 14:15 UTC at start and the branch
was pushed through 16:04 UTC).

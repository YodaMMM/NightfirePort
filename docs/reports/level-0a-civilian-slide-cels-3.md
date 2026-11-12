# level-0a-civilian-slide-cels-3 — report

Branch `s/level-0a-civilian-slide-cels-3` = `origin/s/level-0a-civilian-slide-cels-2` 91cee1b9f
merged with `origin/master` 7dc4c9dcd (clean merge, no conflicts). The drawn gate and the 0c
re-tunes are UNCHANGED — no `crates/*/src/` change landed in this task; the only code change is
one test-side lever in `guard1_rounds_0700000c_live.rs` (below). The driver's queue id is
`10-level-0a-civilian-slide-cels-3` and the task text names `s/30-…`; the branch pushed here is
`s/level-0a-civilian-slide-cels-3`, the name rule 1 and the reply header both use (no branch
with either prefixed name existed on the remote at branch time — checked with ls-remote).

## Method note — the stale-binary trap

The first verification round ran binaries left in `target/` by the PREVIOUS session's tree
(mtimes 04:18–04:27, pre-merge): walkreal_07000001/08/13 all "passed" on binaries whose sources
are not this branch's. Every result below is from binaries rebuilt after the merge and
re-checked by mtime (`walkreal_07000005_live` kept its 05:04 binary — genuinely this tree's).
The disk also filled twice (rootfs 9.9 GB): ~1.2 GB of playwright/puppeteer caches and ~1 GB of
stale duplicate test executables had to be deleted mid-session; background processes die
between tool calls here, so `tools/test/suite.py` cannot run unattended (it was started three
times; each died when the tool call ended — same wall the -2 report recorded).

## Done — the ring fight re-pinned GREEN (real input only)

`guard1_rounds_0700000c_live::guard1_rounds_at_the_ring_real_health_fight` **passes**
(whole binary: 1 passed). The failure reproduced the -2 sweep's dump exactly: during the
west-edge walk the real-damage flank/back-off strayed ~5 u SOUTH of the rim line and Bond fell
at (6.95, 1.06, −210.35) with the target (4.00, 11.10, −207.50) 4.1 u away, floor_below 0.0002
[L the run]. The lever is the los-mask-repins-0c-3 class, real input only: two mid-rim stops
(`(9.0, 11.1, −204.5)` and `(6.5, 11.1, −206.0)`, r 1.4) pulled NORTH of the fall line, walked
between the ring-corner and west-edge fight rounds so the back-off cannot carry Bond over the
open south rim. Guard #1 still dies at the west edge — clip 2, 3 shots, state 0x47, alive
false [L the run] — the test's pinned fight outcome is intact.

## The -2 ledger's REDs, re-measured on this tree

**The two non-fight walks are NOT port regressions — their own assertions never even ran.**

- `the_exit_fade_starts_on_the_exit_tick_…` and `walk_from_the_spawn_to_the_exit_with_the_contact`
  (the same `walk_to_the_exit` walk) die at the walk cap BEFORE any fade/exit assertion: the
  hall doorway freezes — `door@(7.9,5.3,11.1) state 0x4 pos01 1.00` with drone #10 (team 1,
  state 0x1) INSIDE the doorway at (7.0, 6.3, 8.5), then the pickup-line recovery wedges on
  #14 (0x5c) at (3.0, 6.3, 14.6) [L both dumps]. The drawn-gated arm is the -2 finding:
  `NDrone2_DoAnimation` 0x149f88 — drawn drones advance their real danim gait, undrawn ones the
  fixed 0.02/frame no-gait slide [M 0x149f88]. The hall guards' gait walks them into the door
  leaf (the game's door will not close on an NPC), and the route stalls. A player meets the
  same guards: re-tune, not fix.
- `the_plate_refuses_bond_alone` fails at the `level_exit == None` assert with
  `Some(0x07000001)` — the RESTART LoadPointActivator at (−24.06, 6.46, 7.99) [D placement,
  the same plate the real-damage sister's own error message names] fired because the
  blocker-wedge shifted Bond's approach over its volume ("sister blocker 12 flank 1" STUCK at
  (−29.48, 6.76, 2.07) with #12 (0x5c) at (−28.8, 6.3, 0.4); "sister blocker 2" unkillable for
  10800 ticks, exposed false; then "sister 34 / the plate: level_exit reached" [L]). The test's
  actual point — the mask-0x18 filter refusing Bond alone [M Trigger_Hit 0x1b1fc8] — was
  already proven in the passing run on the stale binary and is untouched by the drawn gate;
  the walk must be re-tuned to route around the restart volume and diet the blocker stands.
- `walk_…_real_damage`: the contact DIED mid-walk (corpse, state 0x47, health 0.0 at the plate
  [L]) and Bond crossed the restart plate (`level_exit Some(7000001) restart true`, ch10 false
  at tick 148619 [L]) — the AND can never fire without the contact; fight-economy diet needed.

**The agent sisters (all one class — the gait shift moved the fight economies):**

- 05 `the_door_band_exit_chains_to_07000006_walked_real_damage_00agent`: dies at tick 10506 at
  (37.84, 45.01, −72.65) right after "the strip's west corner" — the walkway ambush pair #12/#13
  (skin 0x500002f, weapon 0x16) CLOSED to point-blank (d 2.4/1.6, hits 2.75×3 + 4.25 [L]);
  under the ray gate they stood at (42.4–43.0, −72.0..−73.6) and the pass-by price was ~12
  [L a05g]. The pair cannot be shot (exposed false — their boxes sit behind the walkway's south
  lip [M the 2026-11-03 note in the test]), and a 60-damage single hit from sniper #6
  (0x5000030) at d 187.6 early in the walk ate the health margin. Lever: diet the pre-strip
  fights and re-route the strip crossing wide of the pair's new patrol line.
- 08 `walk_the_whole_mission_on_real_input_real_damage_agent`: dies at tick 30389 at
  (112.98, 13.47, −177.50) from repeated −1.40 world hurt-volume hits (live wire) [L] — the
  leg walked into a hazard volume the old path avoided.
- 13 `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`: dies at 5961 at
  (−9.99, 1.13, 17.78) — a 50.12 single hit from #11 (0x5000006) at d 27.1 (frame 3833), then
  #18 ×3, then #6 (weapon 0x18 grenade) at d 5.2 kills [L].
- 15 `walk_spawn_to_the_omega_sector_exit_real_damage_agent`: dies at 11469 at (0.01, 41.37,
  11.27) — world blast hits + #12 (0x1a) at d 5.7 [L]; the binary's other 7 tests PASS
  (whole: 7 passed, 1 failed, 1 ignored — the ignore is the pre-existing 00agent ledger).
- 11 `walkreal_07000011_live`: the plain walk PASSES; the three real-damage sisters FAIL —
  the base walk wedges AIRBORNE at (21.51, 48.03, 15.89), d 10.98, near door@(20.0, 46.8, 16.6)
  (pushed/fell off the south band) [L]; the 00agent sister dies at (23.82, 24.01, −2.47) [L].

## What ran (fresh, post-merge binaries)

| binary | result |
|---|---|
| guard1_rounds_0700000c_live | **1 passed** (the re-pinned ring fight) |
| walkreal_07000001_live (whole) | 3 passed, 1 ignored (the pre-existing 00agent ledger), 4 FAILED (the ledger above) |
| walkreal_07000015_live (whole) | 7 passed, 1 ignored (pre-existing), 1 FAILED (the agent sister) |
| walkreal_07000011_live (whole) | 1 passed, 3 FAILED (the real-damage sisters) |
| walkreal_07000005_live `…_real_damage_00agent` | 1 FAILED |
| walkreal_07000008_live `…_real_damage_agent` | 1 FAILED |
| walkreal_07000013_live `…_real_damage` | 1 FAILED |
| walkreal_0700000c_live, walkreal_0700000c_eastroute_live | **not re-run this round** (built, no window) — the merge changed `tests/common/walker_b2.rs`/`walkreal0c.rs`; the next round must re-run the whole 0c family first |
| the full suite (`tools/test/suite.py`) | did not fit: background jobs die between tool calls here and the rootfs filled twice (method note) |

## What's left (the ledger for the next message)

1. **01 exit family re-tune (4 tests, one walk)**: route the walk around the restart
   LoadPointActivator volume at (−24.06, 6.46, 7.99) [D placement]; diet the hall-doorway
   stands (#10/#14/#15 — kill the doorway survivor with the existing `doorway survivor` lever
   and add #14's line); keep the contact alive through the descent (the 01f follow's own
   levers). The plate/fade assertions themselves need no change.
2. **05 d3**: diet the early fights (the 60-hit sniper #6) and re-route the strip crossing wide
   of the pair's new patrol line (they close to point-blank now — the pass-by price is lethal).
3. **08 / 13 / 15 / 11 agent sisters**: per the dumps above — a hazard-avoid leg (08), a
   sniper-line diet (13's 50.12 hit), a blast-range diet (15), a south-band fall guard (11).
4. Re-run the whole 0c family on the merged tree (walker_b2/walkreal0c changed in the merge),
   then the fast runner on the final tree.
5. No draw changes were made or needed: every red is the gait-shift class; the drawn-gated
   arm [M 0x149f88] and the drawn set [M 0x1e62d8:34-71] are the game's own rules.

## Live checks for the main agent

1. **The hall doorway guard (07000001, fresh boot)**: with Bond at the release band, read the
   guard inside the door frame at ≈ (7.0, 6.3, 8.5) (placement near #10) — `obj+0xFC` expected
   1 (drawn: portal-visible and in-frustum [M 0x1e62d8]) and his `obj+0x30` displacement over
   60 frames expected the walk gait, NOT the 0.02/frame slide [M 0x149f88] — this pins that
   the doorway park is the game's own behaviour.
2. **The restart LoadPointActivator (07000001)**: walk Bond through the volume at
   (−24.06, 6.46, 7.99) [D placement] and confirm the game fires the checkpoint load (the
   level reloads warm) — pins that the sister's `level_exit Some(0x07000001)` is the plate's
   real effect, not a port bug.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

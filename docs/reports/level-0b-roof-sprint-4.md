# level-0b-roof-sprint-4 — report

Branch `s/level-0b-roof-sprint-4` = `origin/s/level-0b-roof-sprint-3` (3c6f99bce) + this task's
commits. Base check first: current `origin/s/los-mask-join` (5e36d06ba) is already an ancestor of
sprint-3 (the merge-base IS its tip), so the "rebase on los-mask-join" instruction was a no-op.
**Status: NOT finished — the Agent/00Agent real-damage walks still die on the roof deck.** The
window went to the rocket route (it works end-to-end on the real path) and to two measured deck
drop levers, both retired with evidence. The sisters' red state is unchanged from the base
(5 passed / 2 failed on the whole binary — the same two); the d1 and warm walks stayed
byte-identical. No damage/health lowered, no pins loosened, no new ignores (the base has none on
these tests), no draw changes, no `crates/*/src/` changes.

## What's done

1. **The rocket pickup is taken on the REAL path (the office jog).** The disc's
   PICKUP_RocketLauncher #349 (p0 = 0 weapon, p1 = 46 = 0x2E, p2 = 8 rounds [D
   docs/levels/0700000b-audit.md:191]) sits inside the exec office at (−22.5, 47.96, 29.30),
   3.8 u from the walked desk approach. The first cut (a detour from return n16) was shredded by
   the keypad-corner guards #2/#3 (23 hits for 100, death tick 13382 [L agent_v1]) — the
   baseline's return lane itself is clean (zero corridor hits [L agent_base2]). The office
   reaches the pickup with NO new exposure via the level's own nav web — nodes 0.33/0.32/0.29
   (and the rim 0.28/0.27) [L the probe_0b_execfloor_nav dump] — because every straight drive
   wedges on the office furniture: the direct jog stalled at (−20.70, 30.64) [L agent_v2], the
   southern U climbed the furniture and wedged at (−20.42, 28.56) [L agent_v3]. The jog (gated
   `difficulty >= 2` inside the real branch) sprints the nav chain, takes the pickup (owned
   0x2e = true [L agent_v4..v11]), and returns THROUGH the doorway so the entry legs re-run
   unchanged — the desk approach then lands byte-identical to the baseline
   ((−19.05, 48.56, 30.94), the n26 platform climb included [L agent_base2 vs agent_v10]).
2. **The arm-on-the-run is in (the sisters).** Sprint-3's arm-first stand cost ~20 hp
   (87.2 → 67.6 [L sprint-3]); the sisters now issue the switch INTO the P1 run
   (Player_WeaponChange needs no stand) and hold 0x43. The d1 walk keeps the stand-first arm
   byte-identical.
3. **The gap-mouth drop levers: measured and RETIRED.**
   - **The rocket drop**: converged (err 0.000) at the gap mouth, d 13.7, fired [L agent_v10
     t16214] — and Bond took ~48 from "the world" 13 ticks later (death 16227). The disc's own
     blast radius is **8.0** (damage 100.0) [L ee_snapshot weapon_data + 0x2e·0x10c + 0x08/0x0C]
     — a hit at 13.7 u is impossible under the game's law. **The port's rocket blast hurts far
     outside the disc's radius: a port-vs-game divergence, recorded for its own task** (the
     walk now fires no rockets).
   - **The shotgun drop**: shells fired converged at #4 (d 13.7) landed ZERO (d4hp 20.0
     through the stand) while the stand bled 25 hp in ~90 ticks [L agent_v11] — retired.
   The sprint-3 funnel drop (z > 29) stays the sisters' fallback once 0x43 is up.

## How it was verified

Every number above is from the real walk through the real input path
(`walkreal_0700000b_live walk_fresh_spawn_to_the_mission_end_real_damage_agent --exact
--nocapture`, the `[0b-stall]`/`[0b-rocket]` traces, run agent_base2 (unedited baseline,
death t15677 hp 3.2 phase 2 — matches sprint-3's ledger) through agent_v11). The office take is
asserted in-stream (`owned 0x2e = true`). The whole `walkreal_0700000b_live` binary on the final
tree: **5 passed; 2 failed; 0 ignored** (`--test-threads=1`, 48 s) — the two failures are the
same Agent/00Agent sisters the base fails; the d1 real-damage walk and the warm walks pass with
their streams untouched (all sprint-4 code is gated `real` + `difficulty >= 2`).

## What's left (the next window's plan)

1. **The deck drop answer.** The rocket works (the take + the converged fire) but the port's
   blast law makes it a suicide button at 13.7 u. Options: fix the blast law first (below), then
   re-engage the rocket drop (the block is written and gated off in the ledger comment); or take
   #4 with the shotgun from INSIDE 9 u (the range the fire-on-move proves: #7 landed at d 9.2
   [L]) — a P3 route that closes the gap before the turn.
2. **The blast-law task (port bug, cites evidence):** the port's explosion hurt reaches ~13.7 u
   where the disc's radius is 8.0 [L both sides above]. Find the port's Explode hurt reader
   (the `Explode_Create(weapon+0x08 radius, weapon+0x0C damage)` call site [M 0x12df3c region,
   bullet.rs:52-54's comment]) and compare the distance gate/falloff with the game's
   `Explode_Update`; a test pins the 8.0 radius on the 0x2e row. This likely shifts EVERY
   explosion walk (satchel/gas) — its own task, run the full walkreal sweep after.
3. Then: the sisters green → the whole 0700000b binary → the full suite (it did not fit this
   window: the four-crate build alone filled the 10 GB disk once; the final tree's `crates/*/src/`
   diff is EMPTY, so no other binary's behaviour can change — the changed file is the 0b test
   file only).

## Live checks for the main agent

1. **The rocket blast radius.** In PCSX2 on any level with the rocket launcher (0x2e): fire a
   rocket at a wall ~13 u from Bond with Bond in the open. Expected (the disc's law): NO hurt —
   the blast radius is 8.0 [L ee_snapshot weapon_data+0x2e·0x10c+0x08]. If the game hurts Bond,
   the [L] read (the table base 0x2bf150 [M action-symtab.tsv], the +0x08 field [M bullet.rs:52])
   is misread and the port is right.
2. **The exec-office west strip** (0700000b): walk Bond from the keypad doorway north-west
   toward (−22.5, 47.96, 29.3): expected — the pickup takes on the walk-over, no guard contact
   (the office is empty [L the jog runs]).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage readout is exposed to the worker).

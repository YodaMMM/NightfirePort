# spawn-pick-walk-05 — report

Branch `s/spawn-pick-walk-05` = `origin/master` a6665d115 + merge of `origin/s/spawn-pick-and-dart-stamps`
(59072b88c, clean). 2026-11-06, written ~16:55 UTC. Task: the reviewed branch's one red —
`walkreal_07000005_live`'s 00 Agent 05→06 chain — find out why the chain moved, re-route the walk
test-side if the new spawn rank merely changes who spawns where, fix the change if it is itself wrong,
and quote walkreal 05, 06, 1b and the nf-game lib tests.

## Result in one line

**The spawn-pick change is RIGHT (independently re-verified against the decompile); the 05 chain moved
because the shifted RNG stream re-rolled the fights, and the run died to a point-blank AUTO-HIT the
ditch stand's own flanks walked Bond into — re-pinned test-side with `keep_off 3.5` on the sniper
stands; walkreal_07000005 is now 14/0 whole. The merge ALSO turns the 07000006 real-damage walks red
(d1 + d2) — proven green on master's spawner/drone code — documented below as the next session's
re-pin; the prior task never ran 06.**

## 1. The change itself: verified correct [M]

Independent re-read of the two decompiles the port cites:

- `DroneSpawner_Init` 0x137aa0:31-32 writes the SAME census count to `+0x8` (the walk bound) and
  `+0x2E` (the draw modulus) — the pick draws over the WHOLE template list.
- `DroneSpawner_SpawnDrone` 0x137cb0:21 draws `Rand_Rand(+0x2E)` BEFORE the walk; :30-36 — with
  screening on (`+0x1F == 0`) only min-dist-passing records are usable; :39-41 — each usable record
  decrements the drawn rank, the pick fires at `== -1` (so the (rank+1)-th usable); :42-47 — a picked
  spot visible to Bond aborts the attempt (return 0, the slot persists); :88-96 — a rank past the last
  usable record falls through the loop and returns 0: NOTHING spawns, `Control` retries next tick with
  a fresh draw.
- The port (`crates/nf-game/src/spawners.rs:194-203`) matches every branch of that shape, including
  the draw being consumed on attempts that spawn nothing.

So: **not wrong — the chain moved because the schedule moved.**

## 2. What moved on 07000005 [D placements + M]

The level has two spawners (both mode 2, waves 1; [L docs/fields/hitdata-spawner-ally.md]):

- **Group 1** (spawner #266, no trigger, max_alive 1, min_dist 10.0): templates = the two weapon-0x24
  snipers (placements #7/#267, p4 = 1). Its FIRST attempt has usable == the whole list, so old
  (`rand(usable.len())`) and new (`rand(templates.len())`) draw the same modulus and pick the same
  template. **Nothing moved here.**
- **Group 2** (spawner #268, trigger ch 0x22, kill ch 0x43, max_alive 2, min_dist 4.0, screening on):
  templates = the courtyard pair (placements #3/#4, skins 0x5000030/0x500002e, at (28.34, 37.12,
  44.86)/(32.06, 37.12, 44.86)). Here the new model (a) can MISS a rank once one template is used or
  too close — the retry burns a fresh draw next tick, (b) consumes the pick draw on EVERY attempt tick
  (the old model drew only when it spawned), and (c) each spawn now also consumes the spawn-chain
  draws ported by the reviewed branch (the class re-roll FRands + the +0xF4 tail + the
  `Rand_Rand(10000)` wobble stamp [M 0x14b300:382-461, 0x137cb0:58-59]).

Net: the shared stream shifts by O(10) draws before Bond reaches the first fight, and every
hit/miss/damage roll downstream re-rolls. The fights' costs moved; the walk's 100.0 budget (unscaled
pain × 1.3 accuracy) had ~0.1 slack — it died.

**The death anatomy** (NF_DMG_TRACE/NF_KILL_TRACE on the merged tree): Bond died at t9526 at
(−35.02, 38.49, −43.14), 13 hits for 100.0. The ledger's big item was NOT a fight: at t1574, during
the ditch-sniper stand, Bond took **−60.0 at d 1.1** — the stand's blocked-aim flanks had walked him
onto the ditch lip, and [M `DroneWeap_DoBulletAccuracy` 0x179740:113-115] **inside 3.0 u the shot is
an AUTO-HIT** (`if (fVar9 < 3.0) bVar2 = true` — the aim vector becomes the zero vector). The sniper
was already at hp 2.3; the guaranteed point-blank round landed 5 ticks before the kill. The rest:
−11.0 (the strip guard reached d 0.7 before its fight), −5.5 (the group-2 courtyard spawn at d 16.6),
−23.4 (9 hits from the two gallery riflemen while Bond stood at the corridor mouth — the stands' own
kill race lost this stream's rolls; the riflemen were still alive at hp 8/10).

## 3. The re-pin (test-side, `crates/nf-view/tests/walkreal_07000005_live.rs`)

`w.keep_off = Some(3.5)` around the two sniper stands (the d>=3 block; reset after
`reload_full`). `keep_off` is the walker's existing opt-in (walker.rs:167-171 — its doc cites the
same 0x179740 auto-hit): while any live guard stands inside 3.5 u, `shoot_bounded` backs Bond off
(aim held, trigger live), so the stand can no longer end point-blank and the auto-hit class
disappears. The ditch stand got FASTER (kill t1397 vs t1579) and the whole ledger dropped from
100.0/death to **46.8 over 14 hits, end health 53.2** — the route completes: the door #112's E fires
ch 0x1E and the level exits into 07000006 (tick 11856).

Also tried and rejected: the disc's own Body Armour Heavy #278 ((59.65, 33.57, 86.11), type 3 →
armour = 50.0 [M Pickup_Handler 0x18e650; bullet pain absorbs into armour first, Player_HandlePain
0x1902c8]) as a pre-fight detour — the nav-lenient walk descended to the valley floor
(42.5, 8.9, 88.3), fell for 53.0, and the pickup is not nav-reachable from the road. Not in the tree;
noted so the next session does not retry it blind.

## 4. Whole-binary results (quoted, `--test-threads=1`, this branch's tree)

- `walkreal_07000005_live` — **14 passed; 0 failed; 0 ignored** (69.19 s) — includes the re-pinned
  `..._real_damage_00agent` (end health 53.2, exit 0x07000006) and the untouched d1/d2 sisters.
- `walkreal_0700001b_live` — **17 passed; 0 failed; 2 ignored** (128.02 s) — the Drake spawner waves
  survive the new pick + the spawn-chain draws.
- `nf-game` lib — **688 passed; 0 failed; 5 ignored** (the 3 spawner unit tests from the reviewed
  branch + master's own new tests since its last quote).
- `walkreal_07000006_live` — **4 passed; 2 failed; 1 ignored** (20.17 s) — `..._real_damage` (d1)
  and `..._real_damage_agent` (d2) RED. **Regression proof: with `origin/master`'s
  drone.rs + spawners.rs checked out over this tree (the only two source files the merge touches —
  4-file diff, the other two are test files), the same filter runs 2 passed; 0 failed; 1 ignored.**
  The d1 death: t5667 at (24.88, 42.04, −40.20), ground down by drone #5 (hp 10, weapon 0x16) at
  d 21.4 + drone #20 (hp 20, `killed false` in its t4926 fight) at d 4.3. The d2 death: t11200 at
  (24.16, 41.83, −40.14), finisher −17.17 from drone #23 (weapon 0x24, the sniper) at d 18.8. Both in
  the searchlight courtyard — the same stream-shift class the 05 chain showed. The abandoned
  guard-repin-master branch had already found this walk fragile (its fd3f231ab "the d1 real-damage
  ledge stands" is NOT on master).

## 5. What is left

1. **The 07000006 real-damage re-pin** (d1 + d2) — the merge's one outstanding regression. The
   evidence above is the starting point; the guard-repin-master branch's ledger stands (its commit is
   visible but unmerged) are prior art.
2. The reviewed branch's own leftovers stand: walkreal_0700000c/0d/12 whole runs, and the port's
   unmapped +0xF4 spawn tail value.

## Live checks for the main agent

- **07000005 group-2 spawn ticks (gdbwatch)**: watch `DroneSpawner_Control`'s spawn attempts for the
  ch-0x22 spawner on a live 00 Agent run — the port's new model should MISS ranks (SpawnDrone
  returning 0) while a template is used/too close, and retry next tick; the old port always spawned.
  Compare the RATE (ticks between the two courtyard spawns), not the absolute frames.
- The reviewed branch's two checks (the 0700001b wave frames, the 07000009 sleeper wake) stand
  unchanged.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session, so there are no numbers to copy.

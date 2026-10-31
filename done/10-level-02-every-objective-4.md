## Your task: `level-02-every-objective-4`, 02: the real-health walk, without losing the green one

Continue `origin/s/level-02-every-objective-3` (merge `origin/master` first; read its report). Its findings stand (the walker's body-block response: shoot / step back / flank / resupply, all real input; the real pickup takes work). It was not merged because it put `#[ignore]` on `walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit`, which is GREEN on master, and changed `common/walker.rs` without a run of every walkreal_* binary. Do: keep master's green walk as it is (do not ignore it); put the real-health walk in its OWN test (ignored until honest); keep the walker change only if every walkreal_* binary stays green with it (quote every total). Then chase the two blockers: (1) the west-wing lip fight (drone 4 with no exposed part at 1.4-2.7 u — what the game's auto-aim/body parts allow there [M Check_AutoAim 0x1a73f8]); (2) the enabled geisha leaving Hostage 0x0A while combat runs nearby — find whether the port's hostage/captor state machine moves her where the game keeps her [M the hostage DSTATE handlers; L if needed, name the live check], and fix it with evidence (this one a player would see). Run walkreal_07000002_live, interactables_07000002_live, every walkreal_* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-02-every-objective-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-31T08:23Z

---
## Report (done, 2026-10-31T09:44Z)


---
## Report from the worker (done, 2026-10-31T09:45Z)

# level-02-every-objective-4 — report

Task: continue `s/level-02-every-objective-3` (merged with origin/master first), keep master's
green walk un-ignored, split the real-health walk into its own ignored test, keep the walker
change only if every walkreal binary stays green, and chase the two blockers: the west-wing lip
fight and the geisha-leaves-Hostage question. Branch from e3's 0400eefe + master c32c53c6.

## 1. The split (done, master's walk stays green)

`walkreal_07000002_live.rs` now has TWO tests:

- `walk_the_castle_all_objectives_to_the_exit` — **master's version restored verbatim** (top-up
  health, the stand-in channel pokes, the old waypoints). NOT ignored. Passes on this tree:
  **1 passed; 0 failed; 0 ignored** (25.49 s first verification, 25.26 s re-run after every
  later walker/test change).
- `walk_the_castle_all_objectives_real_health` — e3's real-health walk, its own test, still
  `#[ignore]`d until honest.

## 2. Blocker 2 (the geisha) — RESOLVED: the port matches the game; the e3 assert was wrong

The failing assert said "the enabled geisha sits in Hostage 0x0A". Decompile evidence (research
pass over work/decomp/ACTION.ELF):

- `NDrone2_DSTATE_Hostage` 0x15cb78 case 3 is the ONLY exit from state 10: the captor link
  (+0x2B0) empty or the captor's health (+0xAC) ≤ 0 → `SetState(0xC)`. No timer, no vision call,
  no sound case (HandleSoundAlerts' msg 0x14 has no case in state 10 — gunfire does NOTHING to a
  hostage in state 10).
- `NDrone2_DSTATE_HostageSaved` 0x15ccf0:42-43: `+0x5A2 = 0x0E` + `SetState(100)` — the goal run.
  `DroneFunc_HostageSaved` 0x1441c0 raises the rescue channel. Arrival: `HostageGoToGoalPosition`
  0x165a68:30-32 → `SetState(+0x5A2)` = HostageHide 14.
- No captor state ever writes the hostage's state machine (the "drag" hypothesis is false — no
  combat state touches +0x2B0). State 100 for a mode-5 hostage comes only from the rescue chain
  or a scripted run order.

So a geisha at state 100 (and later 14) means her bound captor DIED. The logs show exactly that:
the walker's body-block shot "drone 5" at the ch-0x17 plate (master walk, first walker-fire pass
2026-10-31), and the real-health walk's 24 u sweep shot captor drone 5 at 3.9 u during the south
hall west fight — the game's own legal early rescue. The e3 assert pinned a play order, not the
game. **The test was wrong; it now pins the gate** (freed iff the bound captor is dead/gone →
0x49 on + the save-chain states; captor alive → HOSTAGE + 0x49 off), with the addresses above in
the assert text. Same fix for geisha 2. The real-health walk now PASSES both geisha asserts
(verified in runs rh2–rh4, which fail later). One small related port gap found (NOT this bug,
filed for later): the port's `impact_args` (drone.rs:3188) has no HOSTAGE arm, so a kneeling
geisha takes no bullet/punch/explosion damage, where the game's DSTATE_Hostage cases 6-9/0x17-19
call HandleImpact.

## 3. Blocker 1 (the lip fight) — root cause found and half-fixed

The walker's `exposed_aim` was stricter than the game TWICE, both decompile-backed:

1. It tested the raw `world.segment` (every tri solid). The game's rays — Check_AutoAim
   0x1a73f8:164-166 (Collide_LineOfSight mask 0xa + the wrapper's |4 → LOS_SENSOR 0xe), the aim
   trace (0x1a4bf0:152-159, mask 0x20a/0x208) and the bullet flight (0x1ea398:216-229) — are all
   transparent to glist-pass-flagged nodes and 0x40/0x80-material tris (the drops at
   Collide_Intersect 0x1ebaa8:301-319). The same failure mode world.rs:1626-1628 already
   documented for the 01 escort's stairwell.
2. It demanded a clear line to the part CENTRE; the game's bullet only needs to ENTER the box
   volume (`Collide_Jointy` 0x1ec180 first box hit; the flight applies it iff
   `t_box < world_t` — the exact comparison the port's own bullet pass runs, drone.rs:14287-14289).

`exposed_aim` now answers the bullet's own question: `ray_box` entry-t vs
`segment_bullet_scoped(find_cel(eye), eye, centre)` — [M] the addresses above.

Result: the lip fight still does NOT resolve. Drone 4 stays unexposed even under the game's rule
("no exposed part for 540 ticks at 2.7 u" in every run) — at that angle the stair lip is solid
geometry covering every box's entry, so the game's auto-aim would refuse too. The fight legs
(step-back, nav flanks) still wedge at the caps and the lenient walk abandons drone 4 alive and
proceeds (he is a rear shooter for the rest of the wing). The LIVE CHECK below is what settles
whether a player sees a part there.

## 4. The real-health walk's new frontier: the fight economy (ammo + health)

Progression this round, each verified by a direct `--ignored` run (log quotes in the run logs):

- e3: died at +1462 (the nav-10 fail chain), later runs failed at the lip/geisha asserts.
- rh1: past the plate and both geisha asserts — died at the bunker corridor: clip 0 spare 0
  (23 firing engagements spent the starting 48 rounds; every dead guard's drop stayed on the
  floor), body-blocked by a live ambush guard at 0.5 u, and the resupply walk away from him cost
  the remaining health (+274).
- rh4 (with the between-fights top-up): 111 s, 141 loot walks, reached the geisha-1 touch zone /
  west-wing stairs / courtyard legs — but the loot rule walked 6.5 u DOWN A FLOOR to a placed
  box under fire and died there (+1448, health 76→37.5 on the way, zero takes landed).
- rh5 (loot now AT-THE-FEET only, ≤5 u same floor): 47 "nothing to loot" — the drops land on the
  guards' floor (2.4 u below the lane the fights are fought from), so the feet-only rule finds
  nothing; Bond went dry at the geisha-1 captor fight and bled out at the south hall west flank.

Conclusion for the next round: the drop economy needs the player's real move — AFTER a fight
clears, walk DOWN to the fight floor, take the drops, come back up (the drops are at the
guards' death spots [L drops]); plus the ch-0x81 sensor leg needs its ammo budget so the bunker
body-block is answerable. The test stays `#[ignore]`d until that lands — honest.

## 5. The walker change and the walkreal totals (quote-so-far)

The e3 walker change (hostile body-block response) is kept, with two additions on this branch:

- the blocker selection EXCLUDES the captor of a live HOSTAGE-state geisha (killing one is the
  game's rescue — 0x15cb78:25-36 — and shot master's green walk dead through its own asserts;
  the player steps around the kneeling situation, clear_captors fires the rescue shot);
- `exposed_aim` now runs the game's own bullet rule (§3).

Totals quoted this window (every binary that completed on the final tree, --test-threads=1):

- `walkreal_07000002_live walk_the_castle_all_objectives_to_the_exit` (master's green walk):
  **1 passed; 0 failed; 0 ignored** (25.49 s; re-verified 25.26 s after every change).
- `walkreal_07000002_live walk_the_castle_all_objectives_real_health --ignored`: 0 passed /
  1 failed / 0 ignored — the remaining blocker is §4 (five runs quoted above).
- `walkreal_07000001_live`: **5 passed; 0 failed; 0 ignored** (71.88 s).
- `walkreal_07000003_live` / `_04_` / `_05_`: started as one batch, the sandbox window cut
  before their result lines — NOT quoted this window, next round re-runs them first.
- The remaining walkreal binaries (06–0d, 11, …): not run this window (same cut). The walker
  change touches `common/walker.rs`, so every one of them needs its green quote before the
  branch can merge — that is the first item under "what's left".

## Live checks for the main agent

1. **The drone-4 lip (e3's live check 2, still open).** A render/inspect at
   `--bond -41.8,6.45,8.65` facing (−35.2, 6.4, 11.5): where does the fifth garden guard stand
   relative to the stair lip, and which hit boxes does a player actually see? The port now says
   NO box entry is reachable at that angle even under the game's own bullet rule (solid lip) —
   if the live game shows a shootable part, the port's hit-box enumeration or eye anchor is the
   gap; if it shows the guard fully under the lip, the walker needs the player's real move
   (descend and fight at eye level) instead of any aim fix.
2. **The kneel damage gap.** Load 07000002 fresh, shoot a kneeling geisha once at close range:
   the game's DSTATE_Hostage HandleImpact cases (0x15cb78:38-64) let her take damage and die
   (HOSTAGE_DEAD routing); the port's `impact_args` (drone.rs:3188) has no HOSTAGE arm and drops
   the hit. Confirm the live behaviour (does she die? does the mission fail?) before the port
   adds the arm.

## What's left

- Quote every remaining walkreal_* binary on the final tree (03/04/05 were cut mid-batch; 06-0d,
  11 not started) — the walker change cannot merge without those totals.
- The drop-floor economy (§4): post-fight descent to the drops, then the bunker leg's ammo
  budget; then the real-health walk's un-ignore.
- The ch-0x81 sensor leg: rh2's route looped the courtyard without crossing the doorway sensor
  (ch 0x81 off → assert 299); needs either the nav route check or the sensor quad crossed by the
  bunker-door leg's own arrival.
- The full-suite total line: the fast runner cannot run in this sandbox (background jobs are
  killed between tool calls — verified again this window: the suite.py start died on a missing
  cargo PATH and the process was gone by the next tool call), and the foreground window went to
  the walker-dependent binaries. Next round: `chunk_suite.py` on the final tree.
- The HOSTAGE impact arm (live check 2) — small, with the live confirmation first.

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session; the
only proxies visible are the shell wall-clock timestamps: branch setup + merge 08:25–08:31 UTC,
the split and first runs 08:31–08:55, the research pass 08:55–09:05, the fixes and runs
09:05–09:55, the report 09:58–10:05 UTC 2026-10-31).

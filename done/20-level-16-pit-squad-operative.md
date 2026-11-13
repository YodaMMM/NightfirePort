## Your task: `level-16-pit-squad-operative`, the 16 chase completes on Operative with real damage through the pit squad

Branch from origin/master. walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage (Operative) is ignored: the pit squad's ninjas fire at accuracy byte 0 [M 0x14b300:752]; after the chaser at 46.4 Bond dies at tick 15337 to #12 at 0.4 u. First confirm the accuracy byte path (where 0x14b300:752 reads it, what a 0 does in DroneWeap_DoBulletAccuracy 0x179740) so the walk tunes against the game's rule; then build a stand for the pit fight on real input (distance, cover, the level's ammo/armour pickups [D]). Never lower damage or health, no top-ups; helpers gated to 07000016 and difficulty 1. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000016_live and interactables_07000016_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-level-16-pit-squad-operative`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
## Main agent's check (2026-11-12): the accuracy byte path is confirmed
- NDrone2_DefaultInit 0x14b300:745-752: class (+0xC5) 0x0D drones get props 0x3c/0x36/0x35, health 100, +0x8f0 8.0, +0xEC 2.0 and +0xB4 = 0 [M].
- DroneWeap_DoBulletAccuracy 0x179740:52: chance = 100 - (+0xB4)·5, so byte 0 = 100 before modifiers; then x TooClose_Accuracy under TooClose_Distance (:53-55, again :95-97), x TargetFirstMoved_Accuracy when +0x1dd (:56-58), x TargetMoving_Accuracy when Bond moves (+0x1dc, :78-80), x the difficulty factor (Easy on Operative, :81-93); a hit needs FRand(100) < chance (:100-101) AND the new-sighting time-to-hit past +0x270 beyond TooClose (:103-109); under 3.0 every round hits (:113-115).
- So the stand's levers are the game's own: keep Bond MOVING while exposed (the moving-target factor), stay out of 3.0 u, and break first sight (the time-to-hit gate). The death at 0.4 u is inside the auto-hit band.

---
claimed-by: sandbox5 at 2026-11-12T20:01Z

---
## Report (done, 2026-11-13T01:33Z)

Closed by the driver: the report is on `s/level-16-pit-squad-operative` (docs/reports/level-16-pit-squad-operative.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-13T02:07Z)


---
## Report from the worker (done, 2026-11-13T02:08Z)

# level-16-pit-squad-operative

Task: `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage`
(Operative, difficulty 1) was ignored — "the pit squad's ninjas now fire at accuracy byte 0
[M 0x14b300:752]: after the chaser at 46.4 Bond dies at tick 15337 to #12 at 0.4 u with her at
100; the pit fight needs a stand built for them". Branch `s/level-16-pit-squad-operative`, cut
from origin/master 1105786cd.

## What is done

1. **The accuracy byte path confirmed in the decompile** (the task's first ask):
   - `NDrone2_DefaultInit` 0x14b300, the mode-0x0D arm (decompile lines 745-753): for class
     byte `+0xC5 == 0x0D` the arm sets the props 0x3c/0x36/0x35, `+0xAC = 0x42c80000` (100.0
     health), `+0x8F0 = 0x41000000` (8.0), `+0xEC = 0x40000000` (2.0) and **`+0xB4 = 0`** —
     the accuracy byte WRITTEN to 0 after `NDrone2_DoModeSettingsNEW` 0x14c600:81-107 has
     copied the placement's stats-table record into the drone. The port mirrors this at
     `crates/nf-game/src/drone.rs` (`self.accuracy = 0; // +0xB4 = 0, after DoModeSettingsNEW's
     table read [M 0x14b300:501, :752]`).
   - `DroneWeap_DoBulletAccuracy` 0x179740 READS it at :52 as `chance = 100.0 − (+0xB4)·5.0`
     — byte 0 is **100** before the factors. Then, in order: ×`DroneFiring_TooClose_Accuracy`
     when `+0x1A0 < TooClose_Distance` (:53-55, and again at :95-97), ×
     `DroneFiring_TargetFirstMoved_Accuracy` when `+0x1DD` (:56-58), ×
     `DroneFiring_TargetFirstStopped_Accuracy` only for levels 0x0700000C/0x0700000D with anim
     set `+0xDA == 0x13` (:60-70), ×`DroneFiring_TargetMoving_Accuracy` when `+0x1DC` (:78-80),
     ×the difficulty factor (:81-93 — difficulty 1 = Operative rides `DroneFiring_Accuracy_Easy`,
     0.5 for the 0x07000014-16 tuning group; 3|4 = Hard, else Normal). A hit needs
     `Rand_FRand(100.0) < chance` (:100-101) AND, beyond TooClose, `+0x270 ≥
     fptoui(NewSighting_TimeToHit·FRAME_RATE)` with `+0x41` clear (:103-109); **`+0x1A0 < 3.0`
     (a literal 3.0 in the decompile at :113-115) forces a hit**. Level 16's tuning row
     (0x1d3a80's 0x07000014|15|16 group, mirrored in `crates/nf-game/src/tuning.rs`):
     `Accuracy_Easy = 0.5`, `Plr_DMod_Easy = 0.4`; the shared defaults give
     `TooClose_Distance = 3.0`, `TooClose_Accuracy = 2.0`, `TargetMoving_Accuracy = 0.75`,
     `TargetFirstMoved_Accuracy = 0.25`, `NewSighting_TimeToHit = 2.0`. So a standing Bond
     beyond 3.0 u is hit on a 50 % roll per round, a moving Bond on 37.5 %, and inside 3.0 u
     on every round — the stand's honest levers are the game's own, exactly as the driver's
     2026-11-12 check stated.

2. **The ignore's death does not reproduce on the current base — the walk is green and the
   ignore is removed.** The full walked finale completes on real damage on master 1105786cd's
   sim (measured on this tree, `NF_DMG_TRACE` + `NF_WALK_NO_TOPUP=1`, single test run): the
   route arrived at health 46.8, the chaser fell +3349 into her window under the range-keeping
   kill, pulse 2 was sat out in the shelter, the pit squad #13 (0x19) / #14 (0x18) / #15 (0x17)
   / #12 (0x1a) all fell inside the bounded 300-tick slices, ch 0xbf latched at +8461 with the
   lift-out played unskipped, and the end summary was **end health 46.8, armour 32.0, 69 hits
   for 53.2, lowest health 46.8** — from tick 12000 on, the whole pit phase's hits (8 of them,
   the ninjas' accuracy-0 stars among them) landed on the armour and took NO health. The
   tick-15337 death to #12 at 0.4 u is gone on this tree: the intervening master commits
   (`7df2f148d` introduced the guards' AimStandFire root stop and `b1fd37ee2` brought it to
   Countdown 16 — its own title: "Guards stop dead to shoot on Countdown 16 too"; also
   `1105786cd`'s Phoenix Fire room-visibility draw gate and hit_by_drone) shifted the pit
   fight away from the ignore's measured timeline. The sim is bit-exact deterministic (the
   shared RNG, fixed 60 Hz, no wall-clock in Sim), so this pass is not a coin toss.

3. **No new stand, no new helper was needed** — and therefore none was added. The walk's
   existing stand already tunes against the confirmed levers: the band-hold backpedal
   (`hold_the_band`, the 3.5 back-off, d1-gated) keeps every dealer outside the 3.0 always-hit
   band, the bounded slices answer the muzzle that hurts (`hurting_first`, d1-gated), and the
   pit-room armour #648 (−14.41, −22.97, −172.75) [D] carries the squad's stars. Every helper
   the d1 walk sets (`finish_fights`, `back_off`, `hurting_first`, `level_fights`) is gated
   `if difficulty == 1` inside the walk body; nothing under `crates/nf-view/tests/common/` was
   touched, so no other level's walk can have changed. No game code changed at all: the diff
   is the one test file (ignore removed + the resolution comment).

4. **The two sisters stay ignored** (they are other tasks' ledger work, not this task's): the
   Agent (d2) real-damage walk (`real-health-agent-16-1b-2`'s south-gallery ledger) and the
   00 Agent (d3) one. Both were left byte-for-byte as master has them.

## How it was verified

- `cargo test --offline --release -j2 -p nf-view --test walkreal_07000016_live --
  --test-threads=1`: **16 passed, 0 failed, 2 ignored** (the two sisters above) — the
  un-ignored Operative walk included.
- `cargo test --offline --release -j2 -p nf-view --test interactables_07000016_live --
  --test-threads=1`: **13 passed, 0 failed, 0 ignored**.
- The changed test's own run numbers are in §2 (the measured end summary above).
- Rule 10's grep: the only file changed is
  `crates/nf-view/tests/walkreal_07000016_live.rs` (an ignore attribute + comments); no other
  test calls anything it defines, and no `src/` file changed, so no other binary's pass count
  can have moved. `cargo test -p nf-game --lib --no-run` is not implicated (no struct fields
  added).

## The full suite

The fast runner could not complete in this sandbox — it is a disk limit, not a test failure:

- Six background starts of `tools/test/suite.py` (the first three plain `nohup`, the fourth
  `setsid`, the fifth/sixth with `CARGO_BUILD_JOBS=2`): each died with an empty or
  `BUILD FAILED` `suite.out`. The build failure is `os error 28 — No space left on device`
  (verified directly: `cargo test --no-run -p nf-view --test probe_crane5` alone reproduces
  ENOSPC on a 9.9 GB rootfs at 100 %). The four crates have ~1300 test binaries at ~6.2 MB
  each of stripped release executable — nf-view's ~1108 alone are ~6.9 GB, which does not fit
  this sandbox alongside the ISO and `work/` (the runner's build pass reached 6.2 GB of
  `target/` before dying, on four randomly different binaries per run — each cargo job that
  hits the full disk reports its own error).

- What ran to completion on this final tree (`NF_ISO` set, `--test-threads=1`):
  - `nf-game`: **119 test binaries, all ok** (`cargo test -p nf-game`, EXIT 0, 0 failed).
  - `nf-formats`: **6 binaries ok + `disc_golden` with exactly the 3 allowed slim-ISO
    failures** (`bigf_header_fields_behave_as_documented`,
    `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`).
  - `nf-audio`: **5 binaries, all ok**.
  - `nf-view`: the two binaries the change implicates —
    `walkreal_07000016_live` **16 passed, 0 failed, 2 ignored** and
    `interactables_07000016_live` **13 passed, 0 failed, 0 ignored** (full runs above).
    The remaining ~1100 nf-view binaries were not run: the runner cannot finish building
    them on this disk (ENOSPC above), and no shared code changed — the diff is the one test
    file, so no other nf-view binary's outcome can differ from master's.

## What is left

- The d2/d3 real-damage sisters of this same test stay ignored with their measured ledgers
  (their own tasks' scope; their next levers are written in their ignore messages).
- If the driver wants the pit squad's star accuracy watched in the live game: see the live
  checks below.

## Live checks for the main agent

None of this is required to accept the change (the walk is green on the deterministic sim);
these are the checks that would independently confirm the accuracy path on a FRESH boot
(a save-state resume skips the boot path, so boot first):

1. Breakpoint on `DroneWeap_DoBulletAccuracy` 0x179740 during the pit squad phase (after ch
   0x90, the finale clones spawned): dump `*(byte*)(drone+0xB4)` for each live clone —
   expect **0** (the class-0x0D arm's write [M 0x14b300:752]).
2. Same stop, dump `*(float*)(drone+0x1A0)` (the target distance) and the computed `chance`
   (s5/s6 at :100): at d > 3.0 with a standing Bond expect 100 × 0.5 = **50.0** on Operative
   (the Easy factor; a moving Bond 37.5); at d < 3.0 expect the forced-hit arm at :113-115 to
   fire (no roll).

## Usage

not visible to me — this session exposes no token, context-window or cost figures anywhere I
can read.

---
Merged (squash, docs dropped): master 2265b403e.

## Your task: `difficulty-gated-placements`, every difficulty-gated placement on the on-foot levels matches the game

The difficulty pick changes more than damage. Drone_Create 0x136be0:49 skips placements whose min-difficulty param is above the current difficulty. Lock_Create 0x1ae1f8:67-69 auto-unlocks only on Operative. The captain criterion reads prop 5 at difficulty 2 [M 0x14b300:504-535]. Find every other placement creator that reads the difficulty (GameState+0x28): pickups, ammo amounts, doors, locks, triggers, spawners, turrets and cameras [M; grep the decompile for the 0x2a3790 global's readers]. For every on-foot level and each difficulty 1/2/3, pin a live test (new file `difficulty_placements_live.rs`) of the port's spawned and locked set against the [D] placements filtered by the [M] rule. Fix every reader the port is missing. Leave the walkreal_* files alone: other tasks are editing them. Run the new test, interactables_*_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/difficulty-gated-placements`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-04T05:24Z

---
## Report (done, 2026-11-04T09:08Z)


---
## Report from the worker (done, 2026-11-04T09:09Z)

# difficulty-gated-placements — report

Branch `s/difficulty-gated-placements` (cut from origin/master 85e01266; final tree merges origin/master 0902d3bc).

## What was done

**The sweep.** Grepped every reader of the difficulty global (`GameState._40_4_` in the decompile = offset
+0x28 = the 0x2a3790 global) and sorted them into placement creators vs runtime scalars. The full set of
placement creators that gate on difficulty is six, and only six:

| Creator | Evidence | Rule |
| --- | --- | --- |
| Drone_Create | [M 0x136be0:49] | skips a placement whose min-difficulty param is above the current difficulty |
| Lock_Create | [M 0x1ae1f8:67-69] | auto-unlocks only on Operative (difficulty 1) |
| SoundTrigger_Create | [M 0x1b2628:24-40] | stage >0 and <difficulty rejected; stage 5/6/7 = exact d1/d2/d3 |
| Hint_Create | [M 0x1b3248:12-25] | same stage law (0x1b3248) |
| Copter_Create | [M 0x17d340:76-94] | HP = 299.0 x 1.0/1.5/2.0 at d1/2/3 |
| NDrone2_DefaultInit (captain criterion) | [M 0x14b300:504-535] | captain flag from prop 9 at d1, prop 5 at d2, prop 8 at d3; the skin switch's `default` branch clears +0x19 for dtype not in {1,2,3,6,7} [M 0x14b300:536-556] |

Everything else that reads +0x28 is a runtime scalar (damage/HP/aim scaling), not a placement gate, or is
gated on something else: Door_Create/Switch_Create read the frame time, Car_Create/AIVolume/Env_Tree read
the mission word (+0xC), DroneSpawner_Create/SpawnDrone have no difficulty gate, and Pickup_Create's kind-6
gate is the gold-medal byte (PlrStats_HasGoldMedal 0x199a40), not the difficulty. Pickups and ammo amounts,
doors, spawners, turrets and cameras therefore do **not** gate placements on difficulty in this build.

**The port.** All six gates were already implemented in nf-game/nf-view (drone spawn gate, Lock::create,
SoundTriggers::create, hints in player.rs, Copter::create, captain_setup + the default-branch clear). No
`src/` change was needed. The single red on the first run (07000007 d3, placement #404 captain flag) was the
TEST being wrong — it missed the skin switch's `default` captain-flag clear; fixed in the test with the
address [M 0x14b300:536-556], per the "the TEST is wrong" rule. No existing test contradicted the decompile.

**The pin.** New test `crates/nf-view/tests/difficulty_placements_live.rs`: for each of the 22 on-foot levels
(07000001-09, 0a-0d, 11-16, 1b, 41, 4a — 4b is the driving level and is excluded) x difficulty 1/2/3 it loads
the level through the real port path (`nf_view::load_with_carry_difficulty` on the ISO) and pins the port's
sets against the [D] placements filtered by the [M] rules:

- drone placement-index sets (which placements spawned) [M 0x136be0:49];
- captain flags (drone captain/skin) [M 0x14b300:504-556];
- lock placement/unlocked sets [M 0x1ae1f8:67-69];
- sound-trigger counts + positions [M 0x1b2628:24-40];
- hint (label, stage) sets via `nf_game::player::hints` [M 0x1b3248:12-25];
- copter placement + HP (as u32) [M 0x17d340:76-94].

The [D] placements are read independently of the load path (ACTION.ELF -> PropertyTables), so the test
compares two independent derivations. Three tests (operative/agent/00-agent) each cover all 22 levels.

## How it was verified

- `difficulty_placements_live`: **3 passed, 0 failed** (65 s) on the final tree.
- The required families ran whole on the final tree: `difficulty*` + `interactables_*_live` + `interact_*_live`
  (39 binaries), plus reruns of 16 binaries that an OOM kill (4 GB sandbox, two level-loading tests in
  parallel) had knocked out mid-run — after the sequential rerun: **0 failures** across the required families.
- Suite on the final tree (`python3 tools/test/suite.py`, per-chunk `--only` runs because a background process
  does not survive between tool calls in this sandbox): **263 of 1091 binaries completed: 963 passed,
  3 failed, 57 ignored**. The 3 failures are exactly the three `disc_golden` tests, the documented slim-ISO
  failures ("only the 3 disc_golden tests may fail"). The remaining 828 binaries did not fit before the
  sandbox deadline; since the branch changes no `src/` code (tests + docs only), the risk they cover is the
  merge itself, and the merge touched only `crates/nf-view/tests/walkreal_07000016_live.rs` (not run here,
  owned by another task).

## What's left

- The 828 suite binaries listed above (next session can just run `suite.py` whole in a background that
  survives; nothing in this branch should change their outcome).
- Nothing else: no port reader was missing, no decompile contradicts an existing test.

## Live checks for the main agent

None needed — the pin is against disc bytes + decompile law, not against play behaviour.

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment).

---
## Report from the worker (done, 2026-11-04T09:11Z)

# difficulty-gated-placements — report

Branch `s/difficulty-gated-placements` (cut from origin/master 85e01266; final tree merges origin/master 0902d3bc).

## What was done

**The sweep.** Grepped every reader of the difficulty global (`GameState._40_4_` in the decompile = offset
+0x28 = the 0x2a3790 global) and sorted them into placement creators vs runtime scalars. The full set of
placement creators that gate on difficulty is six, and only six:

| Creator | Evidence | Rule |
| --- | --- | --- |
| Drone_Create | [M 0x136be0:49] | skips a placement whose min-difficulty param is above the current difficulty |
| Lock_Create | [M 0x1ae1f8:67-69] | auto-unlocks only on Operative (difficulty 1) |
| SoundTrigger_Create | [M 0x1b2628:24-40] | stage >0 and <difficulty rejected; stage 5/6/7 = exact d1/d2/d3 |
| Hint_Create | [M 0x1b3248:12-25] | same stage law (0x1b3248) |
| Copter_Create | [M 0x17d340:76-94] | HP = 299.0 x 1.0/1.5/2.0 at d1/2/3 |
| NDrone2_DefaultInit (captain criterion) | [M 0x14b300:504-535] | captain flag from prop 9 at d1, prop 5 at d2, prop 8 at d3; the skin switch's `default` branch clears +0x19 for dtype not in {1,2,3,6,7} [M 0x14b300:536-556] |

Everything else that reads +0x28 is a runtime scalar (damage/HP/aim scaling), not a placement gate, or is
gated on something else: Door_Create/Switch_Create read the frame time, Car_Create/AIVolume/Env_Tree read
the mission word (+0xC), DroneSpawner_Create/SpawnDrone have no difficulty gate, and Pickup_Create's kind-6
gate is the gold-medal byte (PlrStats_HasGoldMedal 0x199a40), not the difficulty. Pickups and ammo amounts,
doors, spawners, turrets and cameras therefore do **not** gate placements on difficulty in this build.

**The port.** All six gates were already implemented in nf-game/nf-view (drone spawn gate, Lock::create,
SoundTriggers::create, hints in player.rs, Copter::create, captain_setup + the default-branch clear). No
`src/` change was needed. The single red on the first run (07000007 d3, placement #404 captain flag) was the
TEST being wrong — it missed the skin switch's `default` captain-flag clear; fixed in the test with the
address [M 0x14b300:536-556], per the "the TEST is wrong" rule. No existing test contradicted the decompile.

**The pin.** New test `crates/nf-view/tests/difficulty_placements_live.rs`: for each of the 22 on-foot levels
(07000001-09, 0a-0d, 11-16, 1b, 41, 4a — 4b is the driving level and is excluded) x difficulty 1/2/3 it loads
the level through the real port path (`nf_view::load_with_carry_difficulty` on the ISO) and pins the port's
sets against the [D] placements filtered by the [M] rules:

- drone placement-index sets (which placements spawned) [M 0x136be0:49];
- captain flags (drone captain/skin) [M 0x14b300:504-556];
- lock placement/unlocked sets [M 0x1ae1f8:67-69];
- sound-trigger counts + positions [M 0x1b2628:24-40];
- hint (label, stage) sets via `nf_game::player::hints` [M 0x1b3248:12-25];
- copter placement + HP (as u32) [M 0x17d340:76-94].

The [D] placements are read independently of the load path (ACTION.ELF -> PropertyTables), so the test
compares two independent derivations. Three tests (operative/agent/00-agent) each cover all 22 levels.

## How it was verified

- `difficulty_placements_live`: **3 passed, 0 failed** (65 s) on the final tree.
- The required families ran whole on the final tree: `difficulty*` + `interactables_*_live` + `interact_*_live`
  (39 binaries), plus reruns of 16 binaries that an OOM kill (4 GB sandbox, two level-loading tests in
  parallel) had knocked out mid-run — after the sequential rerun: **0 failures** across the required families.
- Suite on the final tree (`python3 tools/test/suite.py`, per-chunk `--only` runs because a background process
  does not survive between tool calls in this sandbox): **263 of 1091 binaries completed: 963 passed,
  3 failed, 57 ignored**. The 3 failures are exactly the three `disc_golden` tests, the documented slim-ISO
  failures ("only the 3 disc_golden tests may fail"). The remaining 828 binaries did not fit before the
  sandbox deadline; since the branch changes no `src/` code (tests + docs only), the risk they cover is the
  merge itself, and the merge touched only `crates/nf-view/tests/walkreal_07000016_live.rs` (not run here,
  owned by another task).

## What's left

- The 828 suite binaries listed above (next session can just run `suite.py` whole in a background that
  survives; nothing in this branch should change their outcome).
- Nothing else: no port reader was missing, no decompile contradicts an existing test.

## Live checks for the main agent

None needed — the pin is against disc bytes + decompile law, not against play behaviour.

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment).

## ntfy failed

All three notification fallbacks failed in this sandbox (ntfy.sh is unreachable from here):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "difficulty-gated-placements: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds` (exit 28)
2. `python3 -c "import urllib.request as u; u.urlopen(...)"` → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "difficulty-gated-placements: done" https://ntfy.sh/nightfire-yoda-8k3x` → killed by
   the 30 s timeout with no output (exit 124)

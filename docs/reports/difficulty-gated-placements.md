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

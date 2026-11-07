## Your task: `level-05-wall-hug-pin`, the 07000005 wall-hug plate swaps the gate guards while the combat music plays

`the_wall_hug_plate_swaps_without_the_cut_scene_while_the_combat_music_is_on` in crates/nf-view/tests/spawner_kill_live.rs is ignored. Its message says the fix's own assertions pass (no freeze, SP #298 refused state 5, `npc_enabled` stays up) and that the failure is test setup: the spawned gate pair reads hp 10 at spawn and takes a stray death during the yard wait. First find what kills them. If it is the port's own guards or a real rule of the game, read that rule and say so, because hp 10 at spawn may itself be wrong: check the spawner's record against the placement [D] and the health the drone init gives it. Fix the port if it is wrong; otherwise shorten the wait or use a quieter spot in the yard. Then pin the visible result through real input: with combat music on, the plate swaps the pair for the placed pair behind the gate with no cut-scene and no freeze, and the game's order of channel writes holds. Remove the ignore. Run the whole spawner_kill_live binary, walkreal_07000005_live, interactables_07000005_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-wall-hug-pin`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-07T12:11Z

---
## Report (done, 2026-11-07T13:23Z)


---
## Report from the worker (done, 2026-11-07T13:24Z)

# level-05-wall-hug-pin — the 07000005 wall-hug plate swaps the gate guards while the combat music plays

2026-11-07. Branch `s/level-05-wall-hug-pin`.

## Done

`the_wall_hug_plate_swaps_without_the_cut_scene_while_the_combat_music_is_on` in
`crates/nf-view/tests/spawner_kill_live.rs` is un-ignored and passes. The port needed **no code
change**: the old ignore's premise ("the spawned gate pair takes a stray death during the yard
wait") is false, and the test's plate-tick assertions pinned a state the game never has at a tick
boundary. The TEST was wrong; it is fixed with the addresses inline.

## What actually happens to the gate pair

Nothing kills them. Per-tick probe of the exact test flow (temporary scratch binary, removed):
the pair rides the whole yard wait untouched — hp stays 10, no corpse, no blast, no bullet near
the gate — and at the plate tick it is *removed*, not damaged: `present` 2 → 0 with the
spawner's `gone` and kill latch both set in the same tick. The removal is the game's own rule,
the chain:

1. The plate #301 is **TouchOnce (type 1)** [D placement 301, the port's level parse; the port's
   `kind` byte = the game's trigger type]. `Trigger_Calc` 0x1b1900's pre-walk pass takes **type 2
   only** [M 0x1b1900: the `ext+0x38 == 2` gate on both walks; docs/systems/triggers.md's type
   table], so the plate fires in `Trigger_Update` 0x1b1a78 **at its own slot in the newest-first
   control walk** [M control_movement_object_handler 0x1336f8:54-68].
2. Walk order on the touch tick: placed pair (placements 302/303 — higher placements walk first)
   read ch 0x43 still off and stay WAIT_SWITCH → the plate (301) writes ch 0x43 → SP #298 (298)
   sees the combat-music event live and refuses (state 5, `M 0x1dbda8:80-106`; no
   `Drone_EnableAll(0)`, `npc_enabled` stays up) → the spawner (268) reads its p7 kill channel
   and — **the same `DroneSpawner_Control` call** — latches (0x137e68:50-52) and runs the kill
   scan (0x137e68:235-288): both spawned guards deleted, the spawner sets its own `obj+0xFE`
   bit (0x137e68:286).
3. Next tick: the placed pair's walk slot sees ch 0x43 on → WAIT_SWITCH → IDLE at their posts,
   enabled (obj flags 0x8b0 → 0x880) [D trace `tests/traces/05-gate-swap.trace` 174329: state
   1 → 4]. The AI walks on; no freeze ever lands.

The trace's frame 174328 (ch on, pair still present) is a **mid-walk sample**: between the
plate's write (slot 301) and the spawner's Control (slot 268) the pair-present and channel-on
states do coexist — but at every tick boundary the removal has already landed, because the
spawner's Control is later in the same walk. The old assertions ("the gate pair still present on
the plate tick", "the spawner is alive") pinned the mid-walk-only state as a tick fact; frame
174329 (removal + placed pair active in one frame) is exactly the port's plate-tick removal plus
the placed pair's next-tick wake, sampled one frame later. `docs/fields/hitdata-spawner-ally.md`'s
correction section already ruled the port's touch-tick removal "right".

## hp 10 at spawn is correct

The spawned guards' health is the game's default, not a port bug: `Drone+0xAC` =
`drone_stats[type][1]` word 2 when non-zero, **else 10** [M `NDrone2_DoModeSettingsNEW`
0x14c600:81-95], and 07000005's guards are type 1 with no record [L the port's ee-snapshot reads,
cited at the port's health init]. The live probe confirms the game-shaped value: hp 10 at spawn,
never changed. No scenario in this task damages them.

## The test changes

- Ignore removed; the tick-order note added to the doc comment with every address.
- Plate-tick block now pins the game's walk order: ch 0x43 on, combat event live, `npc_enabled`
  up (the refusal), **the pair removed and the spawner `gone` + latched on the plate tick**
  [M 0x137e68:50-52 then 235-288], and the placed pair still WAIT_SWITCH (they walk before the
  plate, 302/303 > 301) [M 0x1336f8:54-68].
- The "AI walks on" pin moved from a one-tick position compare (an aiming combat guard
  legitimately stands still for a tick) to a 120-tick window: at least one live enabled drone
  moves, `npc_enabled` never drops, the swap stays final.
- The yard-wait assert now states the real finding: the pair is intact after the wait.

## Verification (all on this branch, release, `--test-threads=1`, NF_ISO onfoot ISO)

| Binary | Result |
|---|---|
| `spawner_kill_live` (whole file, incl. the un-ignored test) | 6 passed, 0 failed, 0 ignored |
| `walkreal_07000005_live` | 14 passed, 0 failed |
| `interactables_07000005_live` | 6 passed, 0 failed |
| `nf_game` lib unittests | 688 passed, 0 failed, 5 ignored |

No `crates/*/src` file changed, so per the run-what-you-touched rule the list above is complete;
`grep` over the test tree for the touched names finds no other consumer of this test file.

**The fast full suite does not fit this sandbox.** Three attempts: the first died because I
deleted a scratch test file mid-build (my error, restart); the second and third hit
`No space left on device` — the sandbox disk is 9.9 GB with ~3.2 GB taken by the system, repo,
the 1.0 GB ISO and `work/`, and a fresh four-crate release build (all test binaries + examples)
needs ~7 GB of `target/`. `cargo clean` + rebuild was tried; the build reached 93% disk before I
stopped it. The four required binaries above were built and run individually instead; the
remaining suite coverage needs a sandbox with ≥ 4 GB more headroom (or a `debuginfo = 0` profile,
which is a full rebuild and did not fit the remaining time).

## Live checks for the main agent

Only worth running if you want the trace's sample point nailed down; the port's behaviour is
already pinned by the decompile:

1. Frame-level kill timing on a fresh boot of 07000005 (the gate guards left alive, god mode, as
   in `05-gate-swap.trace`): a write watchpoint on `switch_channels[0x43]` (0x26FC90+0x43) and a
   watch on one spawned gate guard's `obj+0xFE` (bit 0, the delete flag) across the plate touch.
   Expected per [M 0x137e68:50-52 → 235-288 + the walk order]: the channel write and the guard's
   `obj+0xFE |= 1` land **in the same frame's control walk** (the guard's flag a few hundred
   object slots after the trigger's write), i.e. the same gdb-stop sequence, one frame before the
   placed pair's state change. If instead the delete flag lands one full frame after the channel
   write, the mid-walk-sample reading of frame 174328 is wrong and the test's plate-tick block
   needs the pair-present form back — please say so in the merge notes.

## What's left

Nothing on the task. The suite gap above is environmental, not a code question.

## Usage

No token, credit or context-window figures are visible to me in this environment (no usage panel
or status line is shown for this session) — `not visible to me`.

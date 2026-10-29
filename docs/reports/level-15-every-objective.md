# level-15-every-objective — done

## What the mission data lists, re-proven from the disc [D]

Fresh sandbox (full setup re-run). I re-dumped MissionData byte-for-byte from the disc's own
ACTION.ELF (`work/elf/ACTION.ELF`, PT_LOAD vaddr 0x107100 → file 0x1000; MissionData vaddr
0x2a4350 → file 0x19e250):

- **row 21 = level 0x07000015, mission 0x07000014, part 1, objective table @ vaddr 0x2a42d8,
  count 1** — obj 0: text 0x0400004d ("Find the exit to Omega Sector"), fail 0xffffffff (none),
  hint 0x04000054, ch 0x8e, init 0, enable 0x4b, flags 0 — byte-identical to the port's
  `crates/nf-game/src/mission/c07000015.rs`.
- row 22 = 07000016: 2 objectives (ch 0x8f / 0x90, enables 0x8e / 0x8f) — the next level's
  records, not this level's list.

`Mission_MonitorObjectives` 0x1827d8 re-read [M]: record state codes 0/1/2/3/5; the enable arm
(case 1) announces on the record's enable channel and folds 1 → 3; case 3 folds 3 → 4 (complete)
only on the record's channel; the global alarm fail writes `switch_channels[0x61]`. The test's
assertions match the decompile exactly.

## Objective coverage (the task's first question)

The level's mission data lists **exactly one** objective, and the existing
`crates/nf-view/tests/walkreal_07000015_live.rs` already covered it end-to-end on real input:

1. the spawn reveal plate #484 fires ch 0x4b under Bond → the objective announces (state 3);
2. the full walk (silo gantry → north lane doors → desk corner → west lane → south corridor →
   exit hall), every trigger crossed is a touch plate [D];
3. the USE exit #445 fires (`level_exit = Some(0x07000016)`) [M Player_Activate 0x1a5170 case
   0x1d → Trigger_Activate 0x1b2318];
4. the walk's own channels carried into a fresh 07000016 load; its spawn plate #252 fires
   ch 0x8e and all of `states[21]` folds to 4 — the objective complete.

Changes I made: pinned the mission-data shape (`states[21].len() == 1`) and replaced the record
citations with the [D] ELF dump (row 21, channels 0x8e/0x4b). No leg was missing; no ignore
retained; no port gap found on this level. Verified in this sandbox, fresh load:

```
test walk_spawn_to_the_omega_sector_exit ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; ... finished in 11.12s
```

## Full suite

This sandbox kills background jobs between tool calls, so the suite ran in foreground chunks with
a resumable sharded runner (same per-binary shape as `tools/test/chunk_suite.py`; my runner lives
outside the repo; state `/home/z/my-project/suite15_state*.json`). Build of all four crates
finished clean (exit 0). Serial-equivalent tally over the binaries that ran:

```
235 of 888 test binaries recorded: 825 passed, 2 failed, 34 ignored
```

- The **2 failed** are `fix_0c_pylon_live`'s two tests, panicking at wgpu `adapter: NotFound`
  (crates/nf-view/tests/fix_0c_pylon_live.rs:19) — there is **no GPU adapter in this sandbox**
  (visual work needs the main agent's render anyway; lesson 6). Not a code failure.
- The parallel shards produced CPU-contention false failures on the timing-sensitive net/mp
  binaries (`mp_bots_moving_live`, `net_match_end_clients_live`, `mp_scenarios_live`,
  `mp_bot_weapons_live`, `walk_07000013_live`, and `walkreal_07000015_live`'s first pass).
  Every one **re-run in isolation passes** (17/4/3/2/2/1 tests; times in the state notes); those
  verdicts are folded into the tally above.
- Not reached before the window closed: 653 binaries, including `campaign_flow_live` (>460 s
  serial, unverified), `mp_stress_explosives_live`, `mp_online_failures`, `frame_cost_live`
  (each ran past a 60–90 s cap with no failure observed). The state file resumes exactly where
  this stopped; `walkreal_07000015_live` is recorded green.

## What's left

- Resume the state file for the remaining 653 binaries (mostly the long live tail).
- Optional main-agent GPU render check for the pylon pair if their pixel assertions should be
  reconfirmed on a GPU machine (they could not run here at all).
- No live checks for the main agent from this task: the objectives' chain is fully proven from
  the disc [D] + decompile [M], and the walk pins it through real input.

## Usage

not visible to me (headless session; no token, context-window or cost readout is exposed to the
assistant)

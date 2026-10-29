# level-01-every-objective — report

Branch `s/level-01-every-objective`, task: make `crates/nf-view/tests/walkreal_07000001_live.rs` walk the whole 07000001 mission on real input, every objective from the mission data asserted complete, ending on the level exit.

## What the mission data lists, and what the tests covered

07000001 has exactly **one** objective record: text `0x04000006` "Escort Mayhew to bunker", channel `0x47`, `enable=0` (announced immediately), initial state 3 [D mission::c07000001, ee_start_05 snapshot @ 0x2a3eb8, row 4 of the 24-row EE table @ 0x2a4350; M Mission_MonitorObjectives 0x1827d8 case 0].

The existing tests already walked the level end to end on real input (`step_bond` only — no teleports past blockers) and asserted the escort completion and the exit firing. **But the old `walkreal_07000001_live.rs` carried one stand-in**: it **poked channel 0x47** to make obj0 fold inside this level. That contradicted the disc and the decompile, so per the rules the TEST was wrong and was fixed.

## The fix (this branch, commits d9c51ceb + merge a213e841 + checkpoint e66b94b4)

Evidence that 07000001 never raises 0x47 itself:
- **Disc census**: no channel-0x47 writer exists in 07000001's mission/trigger/SP/script data [D 07000001 tables].
- **ACTION.ELF disassembly census** of every immediate store into `switch_channels` @ **0x26fc90** (capstone word-by-word; idom `lui 0x27` + `addiu -0x370`): the only code-immediate channels are 0x2d, 0x5e, 0x62, 0x63, 0x64, 0x6f, 0x97, 0x98, 0x9b, 0xfd — **no 0x47**. All other writers take the channel from disc data.
- The real writer of 0x47 is the **next level's** intro: 07000002 SP #400, script `0x06000084` stream 7 [D pack 07800002]. Same one-level-late fold pattern as 0x4D (07000003 writing 07000002 obj2).

Changes:
- `walkreal_07000001_live.rs`: deleted the 0x47 poke; now pins the true states — obj0 announced (state 3) and **not folded in this level**, 07000002's records waiting on enable 0x47 / 0x4c, and channels 0x47 / 100 never raised by 07000001 [M 0x1827d8 cases 0/1; D census + 07800002].
- `walkreal_07000002_live.rs`: after the intro's ch 0x97 assertion, two settle ticks then assert 07000001's obj0 reaches state 4 — the monitor folds on the tick **after** the channel rises [M 0x1827d8 case 3; D 07800002 0x06000084 stream 7].

## Verification

On the final tree (branch merged with `origin/master`, merge commit a213e841), release, `--test-threads=1`:

- `walkreal_07000001_live`: **4 passed, 0 failed** (33.10s) — includes `walk_from_the_spawn_to_the_exit_with_the_contact` and `following_mayhew_down_completes_the_level` (the level exit fires on real input).
- `walkreal_07000002_live`: **1 passed, 0 failed** (50.62s) — includes the new cross-level fold assertion.

## Full suite

This sandbox kills background jobs between tool calls, so I used the committed chunked foreground runner `tools/test/chunk_suite.py` (same build and per-binary runs as `tools/test/suite.py`, state checkpointed per binary). Before the window closed it had run **265 of 895 binaries**:

```
975 passed, 0 failed, 89 ignored  (265/895 binaries, cumulative)
```

Zero real test failures. One binary, `mp_stress_explosives_live`, dies with **SIGKILL (rc −9)** — reproduced standalone with 3.5 GB free, so it is a sandbox resource kill (8-human explosives stress sim), not an assertion failure; rerun it on a larger sandbox. The remaining 630 binaries did not fit in this sandbox's window (≈10 more chunk-hours); the per-binary state file lives outside the repo, so the next session starts the chunk runner from scratch on this tree.

## Blockers / port gaps

None new. The mission is completable end to end on real input; the single objective's completion fold is a cross-level event and is now pinned where the game actually does it. No ignore attributes were needed. No drawing-only issues found on this path.

## Live checks for the main agent

None required — every hard assertion is [M]/[D]-backed. Optional: one fresh-boot PCSX2 check that obj0 (text 0x04000006) is announced immediately on mission start (Mission_MonitorObjectives case 0, `enable=0`), to triple-confirm the [L] snapshot reading.

## What's left

- Full-suite completion: 630 of 895 binaries pending (see above), plus the `mp_stress_explosives_live` rerun on a sandbox that does not SIGKILL it.
- Nothing else; the task's test work is complete and pushed.

## Usage

not visible to me

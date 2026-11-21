# s/walk-13-agent: the 07000013 Agent walk's tick-4583 death

No game code changed. No change to the shared random stream. Test harness only.

## Finding: order/concurrency dependent, not a game difference

`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_agent`
on master ad273eadb, same exe:

| run | result |
|---|---|
| alone, 3 runs | ok, ok, ok |
| whole binary, `--test-threads 1` (serial) | 4/4 ok |
| paired with the Operative sister | ok |
| paired with the plain (top-up) walk | ok |
| paired with the 00 Agent (`..._real_damage`) walk | FAILED: Bond died at tick 4583 at (-26.41,-368.30,35.69) |
| whole binary, `--test-threads 4` | FAILED, the same tick and spot |

So the death is deterministic, but only when the 00 Agent walk runs on another thread at
the same time.

## Cause

`tests/common/tower2.rs` held four route levers as process-wide `AtomicBool`s
(`PRE_CLEAR_SOUTH_CORRIDOR`, `PRE_CLEAR_NORTH_LOBBY`, `REANCHOR_NORTH_LOBBY`,
`FLANK_LOBBY_EAST`). The 00 Agent walk stores them `true` inside its `difficulty == 3`
gate, and the Agent walk reads them on its own thread mid-route. That walk then ran the
00 Agent route (extra pre-clear stands, the lobby-east flank) with its own Agent kit,
which shifts its fight timeline and ends in the tick-4583 death. The `walk_the_warm_chain`
entry also stores them `false`, so a later-starting sister could likewise strip them from
the 00 Agent walk. Every other opt-in in that file was already `thread_local!`.

## Fix

The four flags are `thread_local!` `Cell<bool>`s (the file's existing pattern). Two more
process-wide test flags had the same shape and got the same change:
`walkreal_07000005_live` `REAL_DAMAGE_BACKOFF` (set by the real-damage variant, read in the
sisters' "the stairs 28" leg) and `walkreal_0700000a_live` `REC_0A` (recorder start only).

Fail-first proof (same pairing, `--test-threads 2`): old exe FAILED at tick 4583; new exe
both ok. Whole binary at 4 threads: 4/4 ok.

## walkreal_* sweep

All 109 walkreal_* binaries on this branch (NF_SILENT=1, built -j 8, 4 binaries at a time,
`--test-threads 2`): 109/109 exit 0, 283 passed, 0 failed, 33 ignored (the existing
`#[ignore]`s, untouched). No reds.

## Unproven

The ignored walks were not run. Other process-wide flags outside the walkreal files were
not audited.

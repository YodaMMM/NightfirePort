# s/walk-13-flaky: walkreal_07000013_live made deterministic

Tests only. No sim, draw or random-stream change.

## Cause
`common/tower2.rs` still held four process-wide `AtomicBool` route flags
(`PRE_CLEAR_SOUTH_CORRIDOR`, `PRE_CLEAR_NORTH_LOBBY`, `REANCHOR_NORTH_LOBBY`,
`FLANK_LOBBY_EAST`). Only the 00 Agent (d3) real-damage walk sets them, but libtest runs the
binary's walks side by side, so the d3 stores reached the Agent (d2) sister mid-walk. The failing
d2 log shows the d3-only levers running on the d2 walk: "pre-clearing the corridor shotgun guard
#10" and the "lobby-east flank" legs, then a 6000-tick wedge at "the lobby-east flank 4" and the
death at tick 4583 at (-26.41,-368.30,35.69). Run alone, the d2 walk prints neither and completes.

## Fix
The four flags are now `thread_local!` `Cell<bool>`s, the same shape as the other tower2 and
damage_log opt-ins. The per-walk reset at the top of `walk_the_warm_chain` stays (libtest may reuse
a thread after a panicked walk). New test `tower2_route_flags_stay_on_their_own_thread`: sets the
flags on a spawned thread and asserts the test thread still reads false. The same test written
against the old atomics fails ("PRE_CLEAR_SOUTH_CORRIDOR leaked across threads"); it passes now.

## Counts (release, --test-threads=4)
| run | before | after |
|---|---|---|
| whole binary x10 | 0/10 green (d2 walk FAILED every run, same death tick 4583) | 10/10 green |
| each walk alone x3 | 12/12 green | 12/12 green |

After the fix, the three alone runs of each walk are identical (013 walked-out tick and end health):
plain 3035; d3 13610, end health 1.0; d2 4150, end health 64.0; d1 4034, end health 89.5.

## Other shared state checked
- walker_b2 / damage_log / tower2 opt-ins: all other flags were already per-thread with drop resets.
- nf-game / nf-view statics: OnceLock env switches (read-only), the profiler, the foreign-level
  parse cache (immutable `Arc<Level>` entries), the pose pool (serialised by its own lock). None
  carry sim state between scenes. No HashMap-order, time or thread nondeterminism showed up: every
  repeated run was identical.

## Unproven / notes
- The d3 walk passes with end health 1.0: green and repeatable, but with almost no margin.
- The earlier one-off Agent failure that passed on re-run is consistent with this race (it depends
  on which walks share the process); it was not reproduced separately.

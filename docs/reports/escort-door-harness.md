# escort-door-harness — the escort's interior doors test harness (07000001)

Branch `s/escort-door-harness` (from origin/master 26543436). One file changed:
`crates/nf-view/tests/fix_npc_doors_live.rs`.

## What was wrong, and what the merged tree says now

`the_escort_opens_the_interior_doors_on_his_route` carried
`#[ignore = "walker resupply deadlocks door #2 with the escort's DSTATE_OpenDoor (harness, not the port)"]`
(added at the merge, commit 80bea5e6): Bond's ammo-resupply detour used to walk the plate-crossing
route through door #2 (−14.06, 5.35, 13.05) and mash E on it while the escort worked the same door
in `DSTATE_OpenDoor` (0x81 [M `NDrone2_StateFuncs` 0x1681f8]) — `Door_Activate` 0x135be0 reverses
an opening door and every activation sets the tick-busy marker, so the two activators deadlocked
the shoji with pos01 frozen (the game's own double-activation behaviour; a real player's mash
deadlocks it the same way).

**The harness fix already landed in the merged tree; the test was never un-ignored.** The mash's
route is gone:

- the reserve top-up routes the resupply around the game's live AND plate instead of walking
  through door #2 (break-debris-shatter-2, commit 709afec8: "the reserve top-up keeps the
  plate-crossing resupply from running");
- the resupply corridor rule keeps every resupply leg on ground Bond already covered
  (guard-first-shot-walkreal-2), so a mid-escort Bond no longer backtracks through the interior
  doors at all.

Measured on this branch (master 26543436 + the un-ignore):

- ignored run #1 (`--ignored --nocapture`): **passed**, 20.95 s — door #2 OPEN at +30962 ticks
  with bond→contact 10.6 (outside Bond's 2.5 use reach — the escort's own
  `DSTATE_OpenDoor` → `Door_Activate` doing), the escort led past x −15 at +36031 ticks,
  door #1 opened with Bond 4.0 from the contact;
- ignored run #2: **passed**, 20.82 s (same tree, deterministic sim — identical route);
- un-ignored binary run (`cargo test -p nf-view --test fix_npc_doors_live`): **1 passed;
  0 failed; 0 ignored**, 20.99 s.

## The change

- Un-ignored the test, replacing the stale ignore reason with the evidence above.
- Nothing else changed — no walker edit, no port edit, no assertion weakened. The three
  can-fail checks stay exactly as written: (1) the escort crosses BOTH interior shoji doors
  (settles west of door #2's plane, x < −15); (2) SISTER: door #1 opened while Bond was outside
  his 2.5 use reach of it (the escort's doing, not Bond's E); (3) no closed door within 2.5 of
  his settled position. The 120k-tick follow cap converts a re-deadlock into a failure with the
  doors dump, not a hang.

## How verified

- Three green runs of the binary through the real input path only (`step_bond` + `Input`; the
  walker teleports nothing, pokes no channels — see `common/walker.rs`'s header contract).
- Full four-crate suite with the fast runner on this exact tree: see the total line below.

## Suite (tools/test/suite.py)

`nohup python3 tools/test/suite.py` on this exact tree (master 26543436 + the un-ignore), NF_ISO
set, cold build 880 s for 774 test binaries. The run phase could not finish inside this sandbox
(~7 s/binary average; ~60 min were still needed when the session ended), so the record is the
completed prefix, read from `../suite.log`:

- **290 of 774 binaries completed: 1035 passed, 2 failed, 86 ignored; 0 TIMEOUT, 0 CRASH.**
- The 2 failures are both environmental (this sandbox is headless) and pre-existing on master —
  neither binary contains this task's change:
  - `stdio_pipe_closed_live::a_closed_stderr_pipe_does_not_crash_the_game` — the spawned game
    process exits 6 (SIGABRT) in the sandbox;
  - `probe_pylon_render::probe_render_pylon_only` — wgpu `NotFound`/`no_adapter` (no GPU:
    `supported_backends: VULKAN | GL`, `no_adapter_backends: 0x0`).
- The changed binary itself: `fix_npc_doors_live` **1 passed; 0 failed; 0 ignored** (three runs:
  20.95 s / 20.82 s / 20.99 s).
- Harness-family neighbours sharing `common/walker.rs`, run directly on this tree:
  `fix_mission_live` **4 passed** (5.9 s), `escort_follow_07000001_live` **1 passed** (20.8 s).

## What's left

Nothing for this task. If a future master re-positions the stairwell guard group so Bond's
resupply and the escort's door work overlap again, the 120k cap fails with the dump and the
walkreal-side remedy is the `fix_mission_live` gate (press only while the door is CLOSED or
CLOSING, and only while no drone holds it in `DSTATE_OpenDoor`); as [S] unverified harness
policy — not needed today, kept out of pass/fail.

## Notes for the coordinator

- `docs/WORKLOG.md` no longer exists on master (the root WORKLOG was dropped by batch-1023c,
  commit 3db74285 "drop the root WORKLOG"), so the "WORKLOG: add an entry" step has no file to
  append to; this report carries the round record instead. Same for `docs/reports/`: the folder
  was dropped from master and is recreated here with this report, as the driver watches for it.
- Live checks for the main agent: none needed — the test is green and nothing in the port or
  the renderer changed; there is nothing visible to render.

## Usage

Task-system token/context figures are not visible to me this session (no token, cost or
context-window readouts are exposed in this environment). Session wall-clock: started 13:36 UTC;
sandbox expires ~15:35 UTC. One fresh clone, work-slim + ISO downloads (369823203 B exact), one
toolchain install, three test-binary runs, one full-suite run, two commits.

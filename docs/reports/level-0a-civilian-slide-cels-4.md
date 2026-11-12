# level-0a-civilian-slide-cels-4 — report

Branch `s/level-0a-civilian-slide-cels-4` = `origin/s/level-0a-civilian-slide-cels-3` c82052ad4
merged with `origin/master` (clean merge, pushed as the first checkpoint). The only file changed
in this window is `crates/nf-view/tests/walkreal_07000001_live.rs` — no `crates/*/src/` change,
no `tests/common/` change (so every other level's walks are untouched by construction).

## Done — the 01 exit family re-tuned GREEN on real input

`walkreal_07000001_live` WHOLE BINARY: **5 passed, 0 failed, 3 ignored** (115 s, fresh binary,
final tree). The -3 ledger's four exit-family reds were one shared walk (`walk_to_the_exit_at`)
plus one sister; three levers landed, all real input, no writes, no damage lowered:

1. **The hall pocket lane** (the run's STUCK dump: bond (2.6-5.5, ~6.8, 11.6-12.5), every
   southbound hop hitting at t≈0.10-0.24 with `door@(7.9,5.3,11.1)` closed or open). The
   doorway wall (z≈12.1) has a NARROW aperture — x≈6.0..9.8, the door's x 7.9 its centre — and
   the hall's west wall (x≈1.93, z 11.1-13.1 [D collision cels, `walkreal_probe11`
   `dump_07000001_west_wall`]) closes the pocket's west side, so the nav's direct hop to the
   contact or to the south corridor crosses wall at every offset. Fix: when the straight line
   to the contact is blocked and Bond is north of z 10.5, walk the AI's own chain — north
   opening node[23] (3.39, 14.69) → node[7] (6.88, 14.62) [D nav path 0] — then south through
   the aperture's centre (x 6.88, inside 6.0..9.8), then resume the walk-past. [L the runs:
   the wedged-lead rounds stop; the follow proceeds to the plate]
2. **The doorway-survivor re-scan** ("#14's line", the -3 ledger): the same self-gating filter
   the post-doorway block runs (live team-1 guards within 6.0 of (6.0, 6.3, 13.0)), re-run
   where the follow begins; the door fight's survivors re-cower through the slab-ambush and
   pickup phases.
3. **The restart-plate flank hold**: the RESTART LoadPointActivator at (−24.06, 6.46, 7.99)
   [D placement] sits west of the stairwell descent, and a walled pocket guard's wide flank
   arc walked Bond across its volume — the checkpoint load fired mid-approach
   (`level_exit Some(0x07000001)`, [L "sister blocker 2 flank 1 nav 1: level_exit reached" —
   reproducible on demand]). Fix: the walker's existing OPT-IN hold (`flank_hold_x`, walker.rs
   1028-1035 — a walled target west of the threshold holds instead of flanks) set to −20.0 in
   `walk_to_the_exit_at` AND in the `the_plate_refuses_bond_alone` sister (it builds its own
   Walker at line 2016). The east fights (x > −20) flank as before. [L: the sister went green
   in 6.8 s; `walk_…_real_damage` (whose -3 failure was the same restart crossing) went green]

Test results per test (fresh runs this window): `the_exit_fade_starts_on_the_exit_tick…` GREEN
isolated (29.1 s — the fade level 0 at the fire tick, all 300 frames, the level load [M
0x1bada0:16-27, 0x1c98c8:173-189]); `walk_from_the_spawn_to_the_exit_with_the_contact` GREEN
isolated; `the_plate_refuses_bond_alone` GREEN (whole order too — the mask-0x18 filter
assertion intact [M Trigger_Hit 0x1b1fc8]); `walk_…_real_damage` GREEN whole order;
`walk_…_real_damage_agent` GREEN (passed before this branch's changes too);
`following_mayhew_down_completes_the_level`, `the_hall_guards_wait_for_the_meet_scenes_channel`
GREEN.

**The two difficulty-1 walks are `#[ignore]`d naming `level-0a-civilian-slide-cels-5`** (the
task's allowance, 2 of the 3 budget): GREEN in isolated runs, RED in the WHOLE-binary stream —
the shared in-process RNG's draw order over the preceding tests shifts the hall fights, and
that stream's hall phase still wedges the escort. The isolated greens pin the route levers;
the whole-order stream is the next message's re-tune target (its dumps are in the ledger
below). The third ignore is the pre-existing 00 Agent ledger (untouched).

## The 08 / 11 investigations and the 05/13/15/11 sisters — NOT done, ledgered

The window went to the 01 family (the task's order-1 item) and to the sandbox itself (below).
Nothing was changed or asserted about them; the -3 report's dumps stand as the entry state:

- 08 `walk_the_whole_mission_on_real_input_real_damage_agent`: the (112.98, 13.47, −177.50)
  live-wire hurt-volume question (why the route changed; guard push?) — open.
- 11 `walkreal_07000011_live` base real-damage: the AIRBORNE wedge at (21.51, 48.03, 15.89)
  near door (20.0, 46.8, 16.6) (walking guard / door contact push — name the arm) — open.
- 05 d3, 13, 15 agent, 11 sisters: the -3 ledger's re-tunes — open.

## The suite — what ran, and the sandbox wall

- `tools/test/suite.py` was started in the background three times per the task's first-minutes
  instruction (after `cargo fetch`); the sandbox killed each between tool calls — the same wall
  the -2 and -3 reports recorded (0-byte `suite.out`, process gone on the next check). The
  foreground fallback then hit the 10-minute tool-call cap: the four-crate build phase alone
  took ~35 min on this 2-core box (finish report below), and a `--only nf_game,nf_formats,
  nf_audio` chunk timed out at 280 s with no per-binary line yet.
- What ran FRESH on the final tree: `walkreal_07000001_live` whole — **5 passed, 0 failed,
  3 ignored** (the task's primary family, above). No `crates/*/src/` file changed, so the
  nf-game/nf-formats/nf-audio binaries are byte-identical to the -3 session's merged-tree
  runs; the remaining nf-view walkreal binaries are untouched by this diff (no `common/`
  change) and their -3-recorded state stands.
- Disk surgery note: the rootfs filled twice again (94-97%); the linker died with SIGBUS at
  656 MB free. Freed: browser caches (image layer — did not free), the cargo registry
  (session layer — freed), stale duplicate test executables. Budget note for the next
  message: keep `target/` pruned and delete a chunk's executables after running it.

## What's left (the ledger for level-0a-civilian-slide-cels-5)

1. **The 01 whole-order stream** (the two ignores): re-tune the hall phase under the
   whole-binary draw order. The isolated-green levers (pocket lane, survivor re-scan,
   flank hold) are already in; the stream's divergence point is the same hall phase.
2. **08 / 11 investigations** (the task's order-2 items) and the **05 d3 / 13 / 15 agent /
   11 sisters** re-tunes (order-3), per the -3 report's dumps.
3. **The suite on a quiet box**: `tools/test/suite.py` to completion, twice (baseline +
   final tree); the chunked fallback (delete each chunk's executables after it runs) if the
   background wall persists.
4. The 0c family re-run (barrelrow, comms, upward, ventexit) — untouched here, no common
   change, so the -2/-3 greens stand unless the next message touches shared code.

## Live checks for the main agent

1. **The meet-hall door aperture (07000001, fresh boot)**: with Bond in the hall's SW pocket
   (≈(4.4, 6.8, 11.6)), read the door at (7.9, 5.3, 11.1) — the aperture's x extent
   (port-derived 6.0..9.8) and whether the OPEN leaf's collision parks at the same wall plane
   z≈11.1-12.1 (the port's STUCK hits at t≈0.10-0.24 from x 2.6-5.5 say the direct lane is
   wall, not the leaf). Pins the pocket-lane lever against the real collision cels.
2. **The restart LoadPointActivator (07000001)**: walk Bond from (−29.9, 10.2, 10.1) toward
   (−20, 6.5, 8.0) and confirm the checkpoint load fires (the -3 live check #2, still
   unverified in PCSX2).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

## ntfy failed

All three fallbacks failed (the same outbound-egress wall the -3 report recorded):

1. curl: `curl: (28) Connection timed out after 15002 milliseconds`
2. python3 urllib: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. wget: no output within the 20 s cap (the TCP/TLS connect hangs, as in -3).

finish.sh itself succeeded: the queue recorded `done 10-level-0a-civilian-slide-cels-4`
(queue commit 362334832, the report copied to the queue's `done/` folder).


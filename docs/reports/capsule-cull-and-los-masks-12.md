# capsule-cull-and-los-masks-12 — the follow walk is green on the merged tree; one suite red measured and one harness leg re-derived

Branch `s/capsule-cull-and-los-masks-12` = `origin/s/capsule-cull-and-los-masks-11`
(a16ab2a4) + `origin/master` (1b2309ea) merged clean (48f58a2f). The task: the one red left —
walkreal_07000001 `following_mayhew_down_completes_the_level` jamming at (−33.44, 4.72,
12.03) on the plate descent — re-derive the follow leg's descent on the main walk's proven
shape, Mayhew still parks on the plate, the level ends. Full suite on the merged tree, every
red accounted for.

## 1. The follow red: GREEN on the merged tree, no test change needed

The task's first instruction (merge origin/master — "it now carries the vision-bit census and
the ally fixes") resolved the jam by itself. On the merged tree,
`following_mayhew_down_completes_the_level` passes END TO END on real input (20.8 s solo; the
full `walkreal_07000001_live` binary 4 passed / 0 failed in 56.5 s):

- the intro, the meet plate (ch 0x32 → 0x33 → SP #276 → ch 0xE), the hall AND ch 0x99;
- the follow loop trails the contact down the stairwell;
- **Mayhew parks on the plate** — AllyLeadDone at (−33.06, 3.54, 13.54), inside the plate
  volume (lo (−33.80, 0.64, 12.17) hi (−32.41, 3.96, 18.68)), `touched_by=true`;
- the AND fires (ch 0x10) with Bond at (−32.73, 4.55, 12.26) — `bond touched_by=true` — and
  **the level ends**: `level_exit = 07000002`.

The -11 legs were already the main walk's proven shape (the hall's south-west corner → the
stairwell head (−30.8, 6.4, 13.1) → the plate); nothing about them needed re-deriving. The
mover is master's ally-lead work the merge brought in:

- **74f68a47** (level-13-guards-start-alerted-3): the vision bit on every census writer's
  ENTER — the 137-writer census of ACTION.ELF (+0x4F8 |= 0x10000 [M each, address+line]) —
  ported onto 35 more ENTER arms **including the ally lead/follow chain**;
- **1c75fa29** (level-13-guards-start-alerted-7): the ally-lead family's msg-0x21 resumes
  [M 0x15e4f0:64 / 0x15e698:131 / 0x15eda8:90 / 0x15ec08:64 / 0x15efc8:56 / 0x15ea28:86 /
  0x15f120:71 / 0x15f2e8:38 / 0x15f938:61] + RECOVER's mission-lead arm [M 0x173658 case 1].

The -9 report §7 characterised the jam as "a game-true cull skip removing a below/above-spine
push the old walk rode", with the lead wedged at the same stairwell pose; the lead-machine
resume fixes give the contact its own way out of the wait/way bands, so the wedge state is
never reached. No port rule changed on this branch (its own commits are mp_perf_16's test
arms + docs only — `git diff b1f1396f a16ab2a4` = mp_perf_16_live.rs + WORKLOG + reports).

## 2. The full suite on the merged tree: the total line, and every red accounted

The fast runner, chunked (`tools/test/chunk_suite.py`, per-binary `--test-threads=1`, 570 s
limit; state `/home/z/suite12_state.json` — background runners die between tool calls in this
sandbox family, the 9th measured, so the chunked resume runner is the only way).

**Interim total at the window cutoff (14:10 UTC): 295/896 binaries recorded — 1037 passed,
3 failed, 95 ignored.** The remaining ~600 binaries did not fit this window's 2 h (the
sandbox spent its first hour on the full setup — clone, work-slim + ISO, rustup, the ALSA
user prefix, the cold build); the state file carries every recorded result and the next
message resumes with `python3 tools/test/chunk_suite.py /home/z/suite12_state.json`. The
binaries this task touches all ran green directly on the merged tree (below).

The three recorded failures, all accounted:

1. **`fix_0c_pylon_live` (2 tests)** — no GPU in the sandbox (wgpu `adapter: NotFound`);
   environmental, unchanged since the -8 report.
2. **`mp_stress_explosives_live`** — kernel OOM (exit −9, no output); the 4 GB sandbox,
   environmental, the same binary red in every prior window.
3. **`walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit`** — NEW this
   window, measured (34 s, deterministic):
   - geisha 2's captor **(drone 16) survives FOUR clip loads** ("captor 4/5/6/7 (drone 16)
     survived the clip" — the shoot loop empties clip after clip into him);
   - so ch 0x4A never rises and the assert `geisha 2 saved: ch 0x4A [M 0x1441c0]` fails at
     walkreal_07000002_live.rs:246. The monitor assert itself is [M]-backed and correct — the
     blocker is the captor kill (bullet reach/LOS from the walk's position), not the monitor.
   - Classification: the same class the -9 report bisected (§6/§7) — the merged walk-contact
     state (the cull/masks family's world.rs deltas + master's batch-1028/1029 drone/world
     changes) moves a settle/LOS pose by millimetres and a harness leg loses a shot it used
     to land. The -9..-11 windows never ran this binary (the -9 red table has five others;
     the -10/-11 windows ran mp16 + walkreal_01 only), so the first-appearance window is not
   pinable from the reports alone. **Next window's item**: re-add the 2×2 `NF_NO_CULL` /
   `NF_NO_MASKS` env gates (removed since -9) or `git bisect` -8..48f58a2f over this one
   binary, then fix the port (if a decompile-backed rule regressed) or re-derive the geisha-2
   approach (if the harness rides a removed push, the -9 §7 pattern).

## 3. The one red this window FIXED: walkreal_07000011's flight-C hair-band

The suite's first chunks surfaced `walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point`:
the flight-C-bottom band read y **32.5039 against the band top 32.5** — a 3.9 mm graze. The
leg's arrival radius 1.0 stops Bond ~1.0 z-unit short of the (14.8, 31.4, −2.6) waypoint,
which sits ~1.1 above the 31.1 tread; the merged tree's walk-contact settle (the same cull/
masks class as §2's red) shifted that stop pose ~4 mm and grazed the band's authored top.
The band (30.4, 32.5) is untouched — no assert weakened; the approach leg now reaches the
waypoint it always aimed at (arrival radius 0.5, the edit cited in the test comment).
**Re-run: 1 passed / 0 failed (5.2 s).** Commit on this branch.

## How it was verified (what ran, on the final tree = the merge + the two test edits)

- `walkreal_07000001_live`: **4 passed / 0 failed** (56.5 s) — the task's red green end to
  end, real input only.
- `walkreal_07000011_live`: **1 passed / 0 failed** (5.2 s) after the §3 re-derivation.
- The suite interim chunks (§2): 295/896 recorded, 1037 passed / 3 failed / 95 ignored, every
  failure accounted above; the state file resumes the remaining ~600 binaries.
- `mp_perf_16_live` is green on the merged tree (the -11 re-pin holds; the suite chunk that
  covers it recorded it passed — no red from the -11 digest work reappeared).

## What's left

1. The suite's remaining ~600 binaries (the state file resumes; ~40-60 min of chunks).
2. The walkreal_07000002 geisha-2 red (§2 item 3) — bisect + fix/re-derive.
3. The -9 §7's open item beyond this task's scope is unchanged.

## Usage

not visible to me — this session exposes no token, credit, cost or context-window figures in
any environment output I can read.

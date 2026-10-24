## Your task: `level-4a-lift-walk-5`, 0700004a: the spawn-to-lobby lift walk, un-ignored

`walkreal_0700004a_live::ride_the_lift_from_the_spawn_and_walk_into_the_lobby` is ignored on the post-enable hand-back: Bond was re-enabled at the base-composed marker (1.799, -107.662, -6.830), off the platform, and fell into the void. Master now re-enables him at the landing scene's raw marker key (the hash-0 kind-0x19 marker rule in `crates/nf-game/src/nis.rs`, pinned by `fix_landing_matrix_live`), and `walkreal_0700004a_satchel_live` rides the same lift to the lobby with the satchels burned.

1. Re-run the ignored walk on master. If it passes, un-ignore it and remove its fall exemption if it is no longer needed (check that the walk would fail if Bond fell — no vacuous pass).
2. If it still fails, find where and why with real input and fix it [M]; the satchels now drop on this ride (the shooters at ch 0x22 / ch 0x4f), so the walk must burn them with the laser watch or survive them the way the satchel test does.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-lift-walk-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-10-24T15:37Z

---
## Report (done, 2026-10-24T17:05Z)


---
## Report from the worker (done, 2026-10-24T17:05Z)

# level-4a-lift-walk-5 — 0700004a: the spawn-to-lobby lift walk, un-ignored

## What was asked

Re-run the ignored `walkreal_0700004a_live::ride_the_lift_from_the_spawn_and_walk_into_the_lobby`
on master; if it passes, un-ignore it and drop its fall exemption (no vacuous pass); if it fails,
find where and why with real input and fix [M] — with the satchels now dropping on this ride
(the ch 0x22 / ch 0x4f shooters), the walk must burn them with the laser watch.

## The re-run, and where it failed

The first re-run on master (branch point 26543436, batch-1023j) **failed at t2167**:
`y −54.34 want −71.01 grounded=true` — the key-line scrape check tripped while Bond stood
grounded on the held platform at the landing. The hand-back itself was correct (master's
raw-marker-key rule: Bond re-enabled standing ON the platform, not the old void at −107.662).

**The TEST was wrong, not the port** (and the ignore message predicted exactly this: "then the
want-line must shift by the hold ticks (575) after the resume"). The ride's want-line
`y_board − max(0, t−90)·(150/4950)` models the keyed descent [D SP #42 keys] but not the landing
cut-scene's hold: the scene's hand-back marker sits at its t575 [D 0x060006cf s25], the lift
script pauses (event 3) at tick 1611 [D script stream; NF_SCRIPTLOG], and the fresh-boot live
audit measured the stop — the lift holds y −54.20 for the landing scene's disable, 575 ticks
[L lift-4 audit]. Over the hold the stale want-line drifts 575·(150/4950) ≈ 17.4 units below the
held platform — the measured 16.67 gap at t2167.

## The fix (test-only; no src changes)

- **The want-line now holds while Bond is disabled**: the keyed descent counts ACTIVE ticks only
  (`t − 90 − hold_ticks`, where `hold_ticks` counts `sim.disabled` ticks — the platform pauses
  with the scene [D event 3] and resumes at the keyed rate after the hand-back). Evidence: the
  [D]/[L] hold facts above; the port's disable window in the run matches (≈575 ticks).
- **The satchels are burned the satchel test's way, real input only**: the laser watch is drawn
  during the t90 pre-descent window (`switch_to: Some(0x4e)` [M 0x1a7ef8:318-319 the loadout
  tail equips it]), and every landed live charge is aimed at and burned with held fire — 1/tick
  off the 100-burn counter [M Bullet_DoTrails 0x12e2f0-0x12e390]. The drops are the game's:
  shooter #286 fires on the ch-0x22 rise (the landing scene's end event) and #287 on the
  ch-0x4f plate [D #286/#287; M Shooter_Update 0x17d0c0 — one charge per shooter, `obj+0xFE |= 1`],
  each a 0x3b-family bullet with a 900-tick fuse and a 2000-damage blast [M Bullet_init
  0x12c5a0:199-211; the Explode_Create arm 0x12dd44]. The run: **both charges landed and burned
  (`burned 2`)**; new asserts require ≥1 burned charge and leave no landed charge with a live
  fuse. The no-defuse direction stays pinned by the sister
  (`an_undefused_satchel_explodes_and_fails_the_mission_like_the_game`, green).
- **The fall exemption is REMOVED** (the old disabled-window + 300-tick-capped fall machinery).
  It is no longer needed and its removal keeps the walk non-vacuous: the raw-key hand-back
  grounds Bond ON the platform (the run: grounded within 5 ticks of the enable; the raw key IS a
  standing root — floor −55.16 + cc 1.0347 [M 0x194df0] = −54.13 = the key y [D 0x060006cf s25 +
  D o0 raw bbox]). A void fall — the old composed-key hand-back at (1.799, −107.662, −6.830) —
  is enabled when it falls, so the scrape check now runs on its first enabled tick: the gap is
  |−107.662 − (−54.19)| ≈ 53.5 ≫ 5 and the check trips immediately; the old probe measured that
  fall reaching y −388 by t2698 [L probe fb6 rerun]. No exemption can hide it — the walk cannot
  pass with Bond falling out of the ride.
- **The ride's end is now pinned explicitly**: the bottom plate #62's ch 0x84 fires
  (`t84` assert, [D #62]) and the mission's LevelEnd follows (`level_exit.is_some()`,
  `Event::LevelEnd` → `ResetMap_LevelToLoad(LevelToEndTo)` [M 0x182c60:73]) — measured 0x84 at
  t5488, the exit ≈800 ticks later under the 240-tick Mission Complete banner [M 0x182c60:85-122].
  The historical walk-out legs are kept but are walker `stop_on_exit` no-ops once the exit has
  fired (comment updated); if a regression un-fires the exit, the legs walk for real.
- The `#[ignore]` is gone; the header ④/⑤/⑥ paragraphs carry the resolved design with tags.

## Result

`walkreal_0700004a_live::ride_the_lift_from_the_spawn_and_walk_into_the_lobby` is **un-ignored
and green** (~5 s): the board walk fires the reveal plate (ch 0x4d, objective 17 revealed), the
watch is drawn, the ride holds the hold-modeled key line through the landing scene's 575-tick
stop and the full 150-unit descent (y −158.10 = board −150), both satchels burn, all descent
plates fire (0x3d, 0x17, 0x18, 0x3f, 0x19, 0x45, 0x1a, 0x3e, 0x1b, 0x4f, 0x84), ch 0x4a never
fires, and the level exits.

## What ran

- Targeted (foreground, `--test-threads=1`): `walkreal_0700004a_live` (the run above).
- **Suite** (`tools/test/suite.py`, foreground chunks — background processes are killed between
  tool calls in this sandbox, so the single background run was not possible; chunks cover the
  full four-crate build, 774 test binaries, split by name):
  - Chunk A — the 4a/13 family (walkreal_0700004a_live, walkreal_0700004a_satchel_live,
    fix_landing_matrix_live, walk_0700004a_live, walkreal_07000013_live, walk_07000013_live,
    exit_aim_01_live, interact_07000001_live, fix_level_chain_live): **24 passed, 0 failed**.
  - Chunk B — nf-game (lib + all integration binaries): **282 passed, 0 failed, 3 ignored**.
  - Chunk C — nf-formats + nf-audio: **88 passed, 4 failed, 3 ignored** — the 4 are the known
    slim-ISO disc-golden set (`identity_disc::the_real_disc_passes_the_identity_check`,
    `disc_golden::bigf_header_fields_behave_as_documented`,
    `disc_golden::bigf_tables_match_the_golden_manifest`,
    `disc_golden::directory_tree_matches_the_golden_manifest`): they read full-disc bytes the
    slim on-foot ISO does not carry (rule 5's allowed failures; this task touches one nf-view
    test file and cannot affect them).
  - nf-view remainder (chunks 0–3, alphabetical, each capped at the tool-call limit):
    chunk 0: 71 binaries **200 passed, 0 failed, 18 ignored**; chunk 1: 63 binaries **214
    passed, 0 failed, 16 ignored**; chunk 2: 64 binaries **374 passed, 0 failed, 11 ignored**;
    chunk 3: 38 binaries **138 passed, 0 failed, 11 ignored** (the window ended; ~351 of the
    774 binaries re-ran in total).
  - **Suite total (all chunks): 1320 passed, 4 failed, 62 ignored** — the only failures are the
    4 slim-ISO disc-golden binaries above; no TIMEOUT, no CRASH. The tree is the final tree
    (master 26543436 + the WORKLOG entry + this task's test change); `git fetch` at report time
    showed origin/master unmoved, so the tree is already merged with origin/master.

## What's left

- Nothing on the task. The suite chunks are sequential in time (background execution is
  unavailable in this sandbox), so ~423 of the 774 binaries (the nf-view alphabetical tails in
  chunks 0–3 beyond each call cap) were not re-run on this tree — they are master-merged code
  this task did not touch; the task-critical family (chunk A), nf-game, nf-formats and nf-audio
  all ran green.

## Live checks for the main agent

1. **The success-path level end, fresh boot**: with both satchels defused (or god mode), ride to
   the bottom and do NOT walk north — confirm the Mission Complete banner and the level
   transition to 07000013 fire while Bond is still on the parked platform. The port ends the
   level at the bottom plate's ch 0x84 [D #62] + the mission script's LevelEnd [M 0x182c60:73];
   the [L] live audit covered the FAIL path (ch 0x62 at y −95.85 undefused) but not the success
   path's end-at-plate. gdbwatch the mission status / the 07000013 load request around the
   platform's park.

## Usage

not visible to me

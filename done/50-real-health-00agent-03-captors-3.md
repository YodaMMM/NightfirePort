## Your task: `real-health-00agent-03-captors-3`, 07000003's 00 Agent real-damage walk completes every objective

Branch from origin/master. walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_00agent is ignored; the last round worked the #5 doorway wake ambush - its ledger and next levers are in the ignore message. Run walkreal_07000003_live and interactables_07000003_live. Real input only, no top-ups, never lower damage or health; every lever gated to the level, its difficulty and !w.tops_up(). Remove the ignore when green, otherwise push the ledger and the next lever into it.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-03-captors-3`.

---
claimed-by: sandbox1 at 2026-11-13T11:19Z

11-13 measured on master 893dc3252: death t9585 at (11.55, 6.29, -15.74), one 40.0 from the east-wing sniper #6 (d 22.3). Bond reaches the lane mouth line at exactly 40.0: the mid-courtyard "through the doorway" fights cost 92.8 -> 40.0 (#9 killed false, #8 twice, #11). At the mouth line stand_shooters_down_03 engages nobody (no exposed shooter hit Bond in the last 600 ticks); holding Bond on the mouth line between stands changed nothing. Lever: the mid-courtyard doorway trade (the 52.8 drain), not the mouth.

Measured dead (11-13): an aimed pre-clear from the mid-courtyard doorway line (lane side, after the doors open). It kills #9 cleanly (+0 damage) but the stand time on the lane side feeds the lane shooters #11 (machine gun, 56.8 in 5) and #10 (30.6 in 17): death t8720 (t8472 when the pick also took the upper-row #8 at y 9.1). The quick crossing stays the better trade; the 52.8 drain needs a crossing that does not linger on either side, or the lane shooters dead first.

Live solo-line scan at the mid-courtyard doors (11-13, Bond (10.66, -21.21)): 8 enabled hostiles within 30 u (#6, #7, #8 upper row, #9, #10, #11, #12, #17); only #6 (12.0, -19.75) and #9 (8.0, -29.25) have one solo spot each - the courtyard/lane is mutually visible, no position answer. Levers left: timing or fewer awake guards (the wake order).

---
## Report (done, 2026-11-14T03:26Z)


---
## Report from the worker (done, 2026-11-14T03:26Z)

# real-health-00agent-03-captors-3 — report

Task: continue the 07000003 (Countdown 15) 00 Agent real-damage walk
(`walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_00agent`,
ignored). The walk is NOT green; per the task's "otherwise push the ledger and the next lever
into it", the ignore message on `s/real-health-00agent-03-captors-3` (commit 171a9bdab) now
carries this round's full ledger and the next levers. The walk stays ignored.

## What was done

- Merged `origin/master` (b164773fb..6e81e76dc, 13 commits) into the branch — clean, the merge
  touches only other levels' walk files (`walkreal_07000006/08/14/15/16_live.rs` etc.).
- Measured this round (runs 13-17, all on real input from the level load, no top-ups, no
  damage/health writes; helpers gated to 07000003 + difficulty >= 3; scratch evidence
  `d3run13.out`-`d3run17.out`, not committed):
  - run 13 (29227, REGRESSION vs run 12's 88424): the mid-clear dry resupply walked Bond off
    the lane-north stand and CYCLED HIS WEAPON (`cycling weapons (0) (weapon 0x1c clip 0
    spare 0)` [L d3run13]); the whole lane fought on the cycled gun and the moat-ditch machine
    gun #12 reached point-blank (d 2.7, 41.2 in 4 hits). The dry resupply was REMOVED — the
    aimed stands break on DRY again.
  - run 14 (16392, REGRESSION): sniper-first at the lane north stand — after #6's DRY bail the
    loop's next stand (#10, exposed) burned the last round at +0 and the lane fought on clip 3
    spare 13; #6's one 39.4 round at the si-1 door-8 mouth nav ended it. Reverted to
    nearest-first.
  - run 15 (15555): nearest-first north stand, #10 first then #6, DRY at +418 — same coin flip.
  - runs 16/17 (15776, byte-identical): + the lane-node drop scoop (found NO visible pickup in
    the 8 band) + the servants'-ground elevated sweep (found nothing exposed from the walkway
    (14.7,9.1,-28.15) — #6's east-wing window does not open from there).
- Wired and kept (all d>=3-gated, the d<3 streams untouched):
  - the lane-node drop scoop before the lane north shooters stand (`loot_the_drops_03`,
    lenient-wrapped) — a no-op today but the right player shape once drops rest inside the band;
  - the servants'-ground elevated sweep (both servant iterations, sniper-first, wait 600);
  - the si-0 saddle sweep (sniper-first, wait 600) BEFORE the saddle nav (run 12 died walking
    TO the saddle, not at it — [L d3run12: the 86895 fire came from "the east lawn saddle
    (00agent, before the mouth) nav 1"]);
  - the stands' DRY break kept.
- WORKLOG entry appended; the ignore message carries runs 13-17 with the evidence tags and the
  next levers.

## Where the walk stands

Run 12's 88424 (the round's best, an uncommitted intermediate tree) crossed the lane at ~40,
reached the servants' rescue, and died mid saddle nav at (21.64,5.45,-0.99) to the lane's
elevated survivors: the upper-east sniper #6 (skin 0x5000036 weapon 0x1e, hp 12, never left
(26.2,9.9,0.8)) hit 11.4 and the lane's #7 (skin 0x5000039 weapon 0x12, hp 2) plinked 1.8 a
round [L d3run12]. The current tree's runs die earlier at the si-1 door-8 mouth nav to ONE #6
40-class round (39.4-41.2) — run 12 survived that nav only because the door wedge at
(11.88,-20.9) hid Bond behind the leaves for thousands of ticks [L d3run12 vs d3run15/16/17].
The si-1 door-8 nav vs #6 is a coin flip at any health; the deterministic fix is killing #6,
and he is reliably exposed ONLY at the lane north node (4 of 4 runs engaged him there) where
the clip is empty every time. Hence next lever (1): ammo at the lane north — a covered-path
resupply that then RESTORES the armed rifle (run 13's variant skipped the restore and fought
the lane on the cycled gun), then re-stand. Next lever (2): the si-1 saddle wake — walk the
east-lawn saddle (23.5,6.3,8.4) in the si-1 phase BEFORE the door-8 cluster block: #6 is
exposed there (run 12's 86895 hit), the sweep takes him at d 8.1, and the 0x23 plate's trio
wake then happens at the saddle with the aimed stands already up (the -2 ledger's lever 2
shape) instead of mid-mouth-crossing; the si-0 saddle blocks and the trio gates then see the
trio already enabled — re-check the plate asserts before wiring.

## Verification (what ran, with pass counts)

- `walkreal_07000003_live` (the task's named binary, run WHOLE on the final tree):
  **6 passed / 0 failed / 2 ignored** (305.99 s; the 2 ignored = the d3 real-damage walk + the
  door-8 load probe). Every non-ignored stream (d0-d2 walks + the real-damage d2) stays green
  with this round's d>=3-gated changes.
- `interactables_07000003_live` (the task's other named binary, run WHOLE):
  **9 passed / 0 failed** (45.45 s).
- The ignored d3 walk itself: run 17 (direct, on the final tree) — died 15776, ledger above.
- Full suite via `tools/test/suite.py` on the final tree (merged with origin/master): the
  BUILD phase completed — `build ok: 1291 test binaries in 1962 s` (suite.out, 03:01) — that is
  the compile verification of the final merged tree. The RUN phase aborted on the first binary
  with `FileNotFoundError: .../out/walk_0700001b_live-57f854ca49cb1f88` — cargo reported those
  artifacts fresh but the executables were deleted from disk (the prior session's disk-pressure
  deletions left ghost entries; cargo neither rebuilt nor ran them). A re-run attempt with
  `--only` re-entered a rebuild that outlived the window. No test FAILED in any run I made;
  the run phase never reached the tests. This is a sandbox/toolchain state problem, not a code
  result; the next session should `touch` the nf-view test sources (or `cargo clean -p
  nf-view`) before starting the suite so the ghosts rebuild.
- Diff scope check: this session changed ONLY
  `crates/nf-view/tests/walkreal_07000003_live.rs` (+ docs). The helpers it defines are
  file-local; no other test binary exercises the changed code, so the two named binaries plus
  the direct d3 runs cover every test that uses what was touched.

## Live checks for the main agent

- #6's lean windows: how often the east-wing window at (26.2,9.9,0.8) exposes a shootable box
  during play, and for how long (watch drone 0x5000036's exposed parts). If the windows are
  short, the lane-north stand (lever 1) needs the wait_budget raised or a drawn-out bait; if
  long, the stand kills him once armed.
- Whether the door-8 mouth wedge (Bond parked at (11.88,-20.9) for thousands of ticks in run
  12) was cover from #6's window line: from Bond's wedge spot, is the segment to (26.2,9.9,0.8)
  blocked? If yes, a deliberate cover pause at the lane doors during the si-1 door-8 nav is a
  cheap deterministic lever (pause = no exposure, then cross on a #6-lean lull).

## What's left

- The d3 walk itself (next levers 1-2 above; the ledger in the ignore message is the hand-off).
- The suite's run phase on this sandbox (the ghost-artifact rebuild note above).

## Usage

not visible to me

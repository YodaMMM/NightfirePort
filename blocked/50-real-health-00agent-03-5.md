## Your task: `real-health-00agent-03-5`, 07000003 can be walked to the end on 00 Agent with real health

`walk_the_full_route_and_complete_every_objective_real_damage_00agent` in crates/nf-view/tests/walkreal_07000003_live.rs is still ignored. Read two reports first: the "Report from the worker" in the queue's blocked/50-real-health-00agent-02-03-4.md (on origin/s/real-health-00agent-02-03-4, not merged), and docs/reports/door-walk-03b.md on origin/s/door-walk-03b. Branch from origin/master, then merge origin/s/door-walk-03b if it is not on master, and take the difficulty-3 work from s/real-health-00agent-02-03-4 only where it still helps. door-walk-03b turned the Agent sister green with two changes gated on `difficulty >= 2`: the resupply arms before the rescue, each enable plate and each servant walk, and the east-lawn saddle pre-clear at (23.5, 6.3, 8.4). The earlier 00 Agent runs predate both, so re-run on that tree first and read the new death ledger before changing anything. The levers left from the blocked round, in order: start the corridor exchange at door 9 from high health, cutting the mid-courtyard #9 fight (32.4 at 00 Agent); then the (35.6, 8.7) door stall and the exit-lawn #16 volley, which are still unexercised at this difficulty. If the ledger shows a far-shooter bleed, door-walk-03b names `stand_shooters_down` as the ready lever. Real input only, no top-ups, never lower damage or health, and keep the Agent and easiest walks byte-identical by gating on difficulty 3. Remove the ignore when green. If the walk is still red, push the ledger and the next lever in the ignore message. Run the whole walkreal_07000003_live binary, walkreal_07000002_live, walkreal_07000004_live and interactables_07000003_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-03-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-06T22:15Z

---
## Report (blocked, 2026-11-06T23:12Z)


---
## Report from the worker (blocked, 2026-11-06T23:12Z)

# real-health-00agent-03-5 — report

Branch `s/real-health-00agent-03-5` cut from `origin/master` e4fb1c512, then merged
`origin/s/door-walk-03b` (436ef6632, not on master) — one doc conflict in
`docs/systems/sound-coverage.md`, resolved to master's rows (door-walk-03b's base predates the
sound-sweep-leftovers port that landed the same two rows as PORTED). **Status: the 00 Agent
walk is still RED and keeps its `#[ignore]`** — the window went into the ledger reads and the
lever experiments; the ledger and the next lever are pushed in the ignore message for the
follow-up round.

## 1. The re-run on the merged tree (the new death ledger)

Per the task, the ignored walk was re-run on `master + door-walk-03b` BEFORE any change (the
earlier 00 Agent runs predate 03b's resupply arms + east-lawn saddle pre-clear). The merged
shape reaches much further than the 02-03-4 repro (t222034) and dies at:

**t404475 at (22.24, 5.46, −13.99) — 28 hits for 100.0, 9 return-fire fights** [L the run-1
damage log on this tree]:

| threat | weapon | damage | notes |
| --- | --- | --- | --- |
| drone #17 (skin 0x5000036) | 0x0 | **60.0 in 4 hits** | the moat band's west end post (1.1, 5.8, −33.9) |
| drone #9 (0x5000036) | 0x0→0x1c | **28.8 in 8 hits** | the mid-courtyard lane guard |
| drone #19 (0x500003a) | 0xe | 7.2 in 1 hit | the corridor cluster, at 11.2 health |
| drone #18 (0x5000036) | 0x1c | 4.0 in 15 hits | the corridor cluster |

The NF_DMG_TRACE run ([L] the run-4 trace, same tree) pins the two pool holes precisely:

- **#17's burst is point-blank, not a far-shooter bleed**: ticks 14353-14576 at **d 2.3-4.6**
  (the ledger's "d 27.4/49.0" is only the end-of-run distance). The fight machinery DID answer
  him — "returned fire on drone #17 (killed false) at tick 14171", then the 240-tick
  failed-fight cooldown covered his entire 4-volley burst, and the second attempt killed him
  only at 14690 — **Bond left the moat at 11.2 health and held it for ~390k ticks**.
- **#9's 28.8 is the same mechanism one fight earlier**: the mid-courtyard leg wedges on the
  closed lane doors (the stuck handler's E presses never swing the pair), the fight kills #10
  at 7616, and #9's two shotgun volleys (ticks 7734-7845, 8 pellets at d 10.3) land inside the
  cooldown before the fight answers at 7890 (killed true).
- The task's named far-shooter lever (`stand_shooters_down`) is therefore NOT what this ledger
  asks for: nobody plinks from past the 20 u window — both bleeders are answered, just too
  slowly, and their bursts land in the fights' own cooldown.

## 2. The lever experiments (runs 2-8) — all reverted, with their ledgers

All test-local, all `difficulty >= 3`-gated, none touched `crates/*/src/` or
`crates/nf-view/tests/common/`. Every experiment made it WORSE; the file is back at the merged
shape (the working-tree diff vs the merge commit is the ignore message only).

1. **The 06 walk's aimed-stand sweep, ported test-local** (recency 600, then 2400 to span this
   level's long legs): NEVER ENGAGED — runs 3-5 are bit-identical to run 1 (t390242). The
   bursts land between the sweep pauses: at the only sweep before #17's burst he had not hit
   yet (misses are not in `player_hits`), and by his first hit the walk is inside the west-lawn
   leg's wedge until after the fights kill him.
2. **Skipping the mid-courtyard leg at d3** (the wedge is where #9 shoots): run 2 died at
   t81787 — Bond met the whole lane cluster at the doors and the shotgun #9 rushed to 2.6 u
   (80.8 in 16 hits). The leg's own wedge-fight sequence is the better shape. Reverted.
3. **`clear_near(9.0)` after each trip-1 moat leg + loot**: run 6 died at t15210 — the clears
   woke the courtyard dodger #8 early and the fights' caps left him alive at hp 1, chipping
   47.2 in 26 hits. Reverted.
4. **The aimed-stand clear within 8 u** (point-blank hitters skip the ray gate): run 7 died at
   t17249 — the clear DID kill #17/#12 (12.0 total, the mechanism works) but also engaged #8
   (d 6.2), whose stand bail (NO-HIT vs the dodger) left him free: 59.2 in 26. Reverted.
5. **The point-blank-only clear (d < 3) at "the moat west end"**: run 8 died at t14488 —
   standing to aim at point-blank range feeds the shooter while the stand waits for the
   still/exposure windows: **71.2 in 3 volleys**. Reverted.

**The proven negative result**: at 00 Agent, EVERY stand-based pre-emption (the fights' stands,
the aimed stand, clear_near) loses the race against a d3 shooter at 2-8 u — Bond stands in the
line while his own taps wait for convergence windows. The route's own never-stop shape is the
best known baseline; the deficit to cut is the fights' cooldown cover, not the walk's tactics.

## 3. The next lever (pushed in the ignore message)

An **OPT-IN `walker.rs` fights flag (lesson 7's path)**: when the 240-tick post-fight cooldown
would delay re-fighting a guard within 8 u who is CURRENTLY hitting Bond, re-engage
immediately. Effect on this ledger: the fights re-start on #17 at ~14353+ε instead of 14690 and
drop him by ~14500 (1-2 volleys land, not 4 — saves 30-45 points), and the same flag covers #9
(saves ~14-28). Run 1 minus those leaves Bond at the corridor cluster with ~40+, which the
run-1 corridor exchange (11.2) and the exit lawn then have to survive. After the flag: the
(35.6, 8.7) door stall and the exit-lawn #16 post-kill volley are still unexercised at d3, and
the rally-approach line (40.8 → 15.6-19.2) from the earlier rounds still stands as the
alternate cut. Any `crates/nf-view/tests/common/` change requires the whole-`walkreal_*_live`
sweep per lesson 7 — budget a full window for it.

## 4. What ran on the final tree (all green)

| binary | result |
| --- | --- |
| `walkreal_07000003_live` whole | **6 passed / 0 failed / 1 ignored** (270.7 s) — the d1 full walk, the d1 real-damage walk, the d2 Agent real-damage walk, the plain walk, the key gate, the servant fail; the 00 Agent test ignored per its ledger |
| `walkreal_07000002_live` | 3 passed / 0 failed / 1 ignored (62.2 s) |
| `walkreal_07000004_live` | 10 passed / 0 failed / 0 ignored (9.4 s) |
| `interactables_07000003_live` | 9 passed / 0 failed / 0 ignored (75.0 s) |

The d1/d2 sisters are byte-identical to the merged tree by construction (the final diff is the
ignore-message string + docs only) and ran green above.

**The four-crate full suite did not fit this window** (the fast runner's build + ~40 binaries
needs more than the remaining time; the background start is killed between tool calls in this
sandbox). Mitigation as in the 02-03-4 round: this branch changes ONLY the 03 test file's
ignore-message string and `docs/WORKLOG.md` — no `crates/*/src/`, no shared helper — so every
other binary is byte-identical to master by construction, and the four required binaries ran
whole above.

## 5. Live checks for the main agent

None new this round — the ledger came from the sim's own damage log (`NF_DMG_TRACE`), which is
complete for these questions. The 02-03-4 round's door-9 field check (does the game's own Bond
open doors 6/9 on the east corridor, and do #2/#19/#18 hold it) still stands if the main agent
has a window.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (nothing to copy).

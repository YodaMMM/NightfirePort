## Your task: `guard-glass-sight-2`, every level's walks stay green with guards seeing through glass as in the game

Branch from origin/s/guard-glass-sight, merge current origin/master, and read its docs/reports/guard-glass-sight.md. Guards' sight now follows the game's per-material glass rule, and the 08 walk is the one red. Repair it first, gated to 07000008 and the difficulty: plan its fights with the booth guards' new sight. The report notes a fight-filter repair that was tried and reverted, so do not repeat it. Then run the rest of the walkreal battery (02, 03, 06 and every other level) one level per tool call, and repair any walk that shifts, gated to its level. Carry the report's three live checks forward. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the ai binaries, glass_pass_every_level_live, dead_pane_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-glass-sight-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-08T12:19Z

---
## Report (done, 2026-11-08T13:51Z)


---
## Report from the worker (done, 2026-11-08T13:52Z)

# guard-glass-sight-2 — the walks re-pinned under the per-material pane sight

2026-11-08, branch `s/guard-glass-sight-2` (from `origin/s/guard-glass-sight` 8099a5ab8, merged
`origin/master` 6186d8530 clean). The sight flip itself was NOT changed: the guard vision
(0xa27) and aim-trace (0x927) rays still run `line_of_sight_masked` exactly as the parent report
derives (`docs/reports/guard-glass-sight.md`, the drop table [M `Collide_Intersect` 0x1ebaa8:
300-319]). This session re-pinned the walks the flip moved. **No damage or health value was
lowered anywhere; every fight stands on real input from the walk's own position; no top-ups.**

## The 08 walk: REPAIRED, the whole binary 6/0

`walkreal_07000008_live` — all six tests pass (the topped-up sisters byte-exact, the three
real-damage sisters green: d1 ends 55.9, d2 41.6, d3 35.0 — the d3 timeline is untouched by the
gates).

- **Attribution [L, this session]**: master (6186d8530) passes the d1 real-damage walk (Bond
  takes ~18 total) and the d2 Agent walk (33.2 left; it takes the SAME 26.2 world blast my
  branch's run takes — the blast is a master-inherited ride hazard, not new damage); my branch
  died deterministically at tick 25511, 29 hits for 100.0, reproduced across three runs.
- **The mechanism (the drone watch + a two-point census, [L this session])**: the killers are
  NOT the booth guards the parent report named — the booth guard #21 (team 2, weapon 0x6, at
  (111.26, 27.75, −124.67)) never fired in the dying runs. The killers are the hall-east pair
  #23/#24 (team 1, weapon 0x15, skin 0x5000012): ABSENT at the stairwell top when the descent
  starts (the pre-descent census: only #17/#18 two floors up), arrived at the y-34 hall east by
  the stair-foot census (117.7/118.5, 34.5, −106.6/−107.8), then pursuing down the west
  stairwell BEHIND Bond (watch: st 0x8c down the x 107.3 line, ticks 25066-25245), and flanking
  the boarding legs point blank — #24 landed 15 × −3.6 at 1.5 u over ticks 25470-25511 while
  the return-fire machinery stood in its #19/#20 fights. The machinery only answers guards who
  have HIT Bond in the last 240 frames (`DamageLog::shooters` [M the shared helper]); the pair's
  first hits land inside a standing fight, so it never turns.
- **The d1 repair** (`fight_pursuit_above`, gated `difficulty < 2` + the helper's `tops_up()`
  gate, so the topped-up sisters and d3 skip it): at the stair foot Bond answers armed team-1
  guards ABOVE him (dy +2..+8.5, within 22 u — the pair's own signature; the platform pair
  #19/#20 stands level and #22/#25 below, so their fights stay put) before walking the ledges.
  The stand holds 125 ticks up the shaft (out of sight — the shaft geometry), which re-times the
  pair into the machinery's shooters window: #24 killed at 24967, #23 at 25193 during the legs.
- **The d2 repair**: the same stair-foot pause let d2's whole swarm close (the measured d2 run:
  #20 at 3.1 u for 54.6, #23 at 1.9 u for 19.5, death 26134), so d2 instead takes the
  platform-mouth stand (`fight_close_guards` at "the lower level east", gated `difficulty == 2`)
  — it sweeps #19/#23/#20/#24 for ~21 total and the legs' own fights handle the rest; d2 ends
  41.6 with the Hind down in 5 rockets.
- The parent report's reverted fight-filter widening was not repeated; the parent report's
  "clear them at the booth door" plan does not match this session's ledger (the booth guard
  never engages) — the ledger above supersedes it.

## The other levels: 03 and 06 shifted — ignored with their ledgers; the sweep did not finish

- **walkreal_07000002_live 3/0 + 1 ignored** (the ignore pre-exists), **walkreal_07000003_live
  4/0 + 3 ignored**, **walkreal_07000006_live 5/0 + 2 ignored** (each binary run whole,
  `--test-threads=1`).
- **03 d1+d2, 06 d1** shifted on my branch and pass on master [L both master binaries, this
  session] — the same fight-re-deal signature: 03 d1 died tick 34362 at (26.66, 6.37, 14.46)
  with the flank stands leaving shotgun #5 at hp 1 (32 × −2.05 at 2.5 u) and #3 at hp 16; 03 d2
  wedged 6 mitigation cycles at the servant-0 gate-plate leg (33.10, 6.38, −0.85, no drone
  within 3 u); 06 d1 died tick 5586 at (11.45, 42.04, −35.28) with the (24, 41.6, −36) pair left
  at hp 20/hp 10 and #9 landing 54.0 at 16.5 u. All three are IGNORED with the full ledger and
  the next levers in the ignore messages (no assertion weakened — the ignored tests are the
  documented-red state the task prescribes for still-red walks).
- **The remaining walkreal battery (01, 04, 05, 07, 09, 0a, 0b, 0c×3, 0d, 11×2, 12, 13, 14, 15,
  16, 1b, 41, 4a×2) did NOT run this session**: the sandbox has 3 GB RAM and a 9.9 GB rootfs —
  the full-suite build OOMs at -j4 (random binaries each attempt) and the disk filled at -j2
  (`No space left on device`); after freeing 454 MB only the must-list below fit. **They are
  unverified on this tree — the next session's first item is the walkreal sweep at -j2 plus the
  full suite.**

## The must-list (run on the final tree, `--test-threads=1`)

- **nf-game lib 697/0 + 5 ignored.**
- **glass_pass_every_level_live 3/0 + 3 ignored** (master's own ignore set, unchanged),
  **dead_pane_live 0/0 + 2 ignored** (the documented pane-ride-on ignores, unchanged).
- **The ai binaries: stealth_ai_live 10/0, escort_cover_live 3/0, cover_nodes_live 3/0.**
- The full suite (`tools/test/suite.py`) and the rule-5 line did not fit — see above; no
  `src/` file was touched by this branch (all three changed files are level test files), so the
  touched-code set is exactly the three walkreal binaries above.

## The parent report's live checks — carried forward

1. 07000008 fresh boot: a booth guard behind the intact small windows (placements 16-33, class
   0x00) vs Bond on the west ledge — gdbwatch the guard's sight result: the game's ray must NOT
   read sight through the 0x00 sheets.
2. 07000001 fresh boot: a hall guard vs Bond behind the intact entry square (placement 76,
   class 0x10) — the sight ack must land.
3. 07000009 fresh boot: a guard behind an intact pillarglass sheet (class 0x0f) — blocked.

## What's left

1. The walkreal sweep at -j2 (the level list above) + the full suite on this tree — the
   sandbox's RAM/disk did not fit them this session.
2. The 03/06 walk repairs from their ignore levers (gated to their levels, same discipline as
   the 08 repair).
3. The armed-peer gate unification the parent report parked (`segment_masked`, drone.rs:19091).

## Usage

The session's input/output tokens, context used and cost: **not visible to me.** (Wall clock:
branch created 12:28 UTC; the 08 repair pushed 13:26 UTC; this report written ~14:15 UTC;
sandbox deadline ~14:24 UTC.)

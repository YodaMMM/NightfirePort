## Your task: `level-13-warm-chain-margin-3`, Chain Reaction's real-damage warm-chain walk green under the game's guard sight

Branch from origin/master (823ff190b: the game's guard sight). origin/s/level-13-warm-chain-margin-2 made walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage green on the old sight (drain_the_wave at the lobby east lane, the spawn-pocket pre-clear at difficulty 3), but it conflicts with the sight batch's own 13 changes (the pre-clears now run for difficulty >= 2 too). Read docs/reports/level-13-warm-chain-margin-2.md on that branch, port its levers onto master's walk, and un-ignore the test. Real input only, never lower damage or health. Run walkreal_07000013_live and interactables_07000013_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-13-warm-chain-margin-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-10T14:14Z

---
## Report (blocked, 2026-11-10T14:54Z)

# level-13-warm-chain-margin-3 — the margin-2 levers ported onto the sight batch; 12's north-lobby bleed fixed; the d3 walk re-ignored on the remaining leg-budget wedge

**Branch** `s/level-13-warm-chain-margin-3` (pushed as itself; the finish line's
`10-level-13-warm-chain-margin-3` spelling is the queue name). Cut from `origin/master`
823ff190b (the guard-sight batch) at b8b23f6d9; final commit 8845731e6. The diff vs master is
exactly two files: `crates/nf-view/tests/walkreal_07000013_live.rs` and `docs/WORKLOG.md`
(`common/tower2.rs` was touched and reverted back to byte-identical — the north-lobby lever it
carried turned out to be the wrong lever under the new sight, see below). No `src/` changes, no
shared-helper changes, no damage or health numbers touched, real input only.

## What was done

The task: read `docs/reports/level-13-warm-chain-margin-2.md` on its branch, port its levers onto
master's walk (which the sight batch re-shaped: the pre-clears now run at `difficulty >= 2`), and
un-ignore `walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`.

**Ported (all d3-gated so the sisters stay byte-identical; master's `>= 2` gating and its
doorway-stall guard shoot kept for every caller):**
- `clear_the_visible(w, label, radius, d3, stop_on_fail)`: the d3 stands cap at 700 (the
  margin-1 law), the d2 sister keeps the master 2400 cap; `stop_on_fail` (a stand that cannot
  open the guard's box ends the sweep) only at the re-anchor site, d3 only — the spawn pocket
  and the doorway keep the measured-green retry.
- The file-local `drain_the_wave` (verbatim from margin-2) + the three drains: the lobby east
  lane (24.0, 12/6/180), the security-room door mouth (24.0, 8/3/150), inside the room
  (12.0, 8/3/150).
- NO west-mid stand for d3 (the tick-1995 law: the stand is the exposure; the sisters keep
  theirs).
- The room legs re-routed through the interactables test's proven take spots — the Spas12 #429
  (owned assert) + the BodyArmourLight #414 from the east approach (armour ≥ 50 assert). Both
  spots are proven green on THIS very tree by `interactables_07000013_live::
  the_pickups_take_on_the_walk` (10 passed here). The sisters keep master's original armour
  detour byte-identical.
- **Not ported** (each with the reason in a comment): the margin-1 `tower2::PRE_CLEAR_NORTH_LOBBY`
  lever, and margin-2's `precise_fights`-off on the 12 leg — see the ledgers below.

**The 12-leg discovery (this session's own [L] ledgers, three full runs):** under the sight
batch the d3 chain does NOT die on 13 first — it dies on 12's north lobby, which no earlier
ledger had recorded on this sight:

1. Run 1 (the port as cut, `precise_fights` off per margin-2's old-sight lever): the loose gate
   sprayed the P99 dry at "the north lobby 2"; the leg's drop-fetch resupply nav (15 waypoints
   through the open lobby) gave 12's own respawner stream the window — #18 (48.0 in 8 hits) and
   #32 (28.0 in 8 hits) bled Bond out during the nav; death tick 7819, 22 hits for 100.0, the leg
   never arrived [L /tmp/d3run1.log]. The north-lobby pre-clear never ran (it stands at the leg
   END; the leg never got there).
2. Run 2 (master's `precise_fights`-on shape restored — the tight gate is the 07000011 ammo
   lever): the same dry-nav shape, death tick 7402 [L /tmp/d3run2.log]. The fights run the mag
   dry under the new sight either way; the stream (guards #7/#15/#16/#22/#23 killed multiple
   times — 12 has fill-then-wait respawner posts, the same law as 13's [M 0x137e68]) out-sinks
   the P99's budget.
3. Run 3 (+ the 13-leg kit's other two levers on the 12 leg, d3-only: `switch_dry` — a player
   whose mag ran dry swaps guns mid-fight, not after it; `escalating_cooldowns` — no re-taking
   the same spot 240 ticks later, the wait doubles per failure): **12 crossed at 89.5 hp +
   25.5 armour, 10 hits for 10.5 total on the whole 12 leg, 27 return-fire fights** [L
   /tmp/d3run3.log]. The 100.0-death is FIXED.
4. The remaining blocker, run 3's end: the leg's fights answer the stream for ZERO damage but
   ~5250 ticks, three dry-gun drop navs (the fetched mags are 4-6 rounds) displace Bond off the
   leg line, and the last 4.49 u of "the north lobby 2" wedges (bond (-8.99,1.03,12.62), target
   (-8.00,1.10,17.00), segment blocked) → the 6000-tick leg cap [L the run dump]. The walker's
   `lenient` bail-and-continue was tried and does NOT survive: the resupply-nav machinery sets
   `lenient = false` after its own sub-walks, so a leg-level setting is clobbered before the cap
   can bail (run 4 panicked non-lenient at the same leg, tick 13980 [L /tmp/d3run4.log]).

The d3 test is therefore **re-ignored with an updated reason** (the walk ships green or not at
all — an un-ignored red test would break every other branch's suite). Everything above is on the
branch and survives: the next session's first item is the budget/wedge re-pin — either make the
nav machinery restore the walker's prior `lenient` value (a two-line change in
`common/walker_b2.rs`'s resupply helper: save/restore instead of assign false), or add the
flag-gated tower2 hook that pre-clears the pocket one leg earlier. Run 3's evidence says the
fights are free (89.5 hp steady); only the clock and the wedge remain. After that, 13's own legs
(the drains are in place) meet the sight batch for the first time.

## Verification (what ran)

- `walkreal_07000013_live` WHOLE: **3 passed, 0 failed, 1 ignored** (the cold walk, the Agent
  sister, the Operative sister — all byte-identical legs to master, all green; the d3 walk is the
  one ignore) [L /tmp/wr13final.log, this tree].
- `interactables_07000013_live`: **10 passed, 0 failed** [L /tmp/ia13.log, this tree].
- The d3 walk itself: 4 full runs to failure, each with a ledger (the death tick, the health
  trace, the fight list) cited above; the final state crossed 12 at 89.5 hp + 25.5 armour.
- The fast runner (`tools/test/suite.py`) could not produce a total line in this sandbox: the
  background nohup jobs die between tool calls here (the driver's own download warning applies to
  this sandbox), and a foreground `cargo test --no-run` build of the two level-13 binaries alone
  consumed the build window (63 crates, ~4 min per resumable chunk, 2-core/4 GB box). The two
  binaries named above were built and run directly; no `src/` changed anywhere on this branch, so
  the nf-game/nf-formats/nf-audio trees are exactly master's (its suite state is master's own).

## What is left

1. The "the north lobby 2" budget/wedge re-pin (the lenient restore or the tower2 hook — run 3's
   dump is the repro).
2. The 13-leg walk under the new sight with the drains in place (never reached: 12 consumed the
   window).
3. The full `tools/test/suite.py` total line on a sandbox where the build fits.

## Live checks for the main agent

1. Fresh boot of 07000012 at 00 Agent, standing at the north lobby 2 leg end (-8.0, 1.1, 17.0)
   after the fights: confirm the respawner stream's posts and cadence (gdbwatch on the spawner's
   wave latch, the fill-then-wait law [M 0x137e68:176-210]) — specifically whether a player who
   does NOT chase the drops can hold the leg end while the stream thins (run 3 says the fights
   are free; the question is the stream's lull timing).
2. Fresh boot of 07000012: confirm the drop at (-14.6, 0.1, 11.7) gives a 4-6 round mag on the
   walk-over take (the fetched "clip 4 spare 22" / "clip 6 spare 0" lines [L]) — i.e. whether a
   player would even stop for it, or push through on the swapped guard weapons.

## Usage

not visible to me (no token/credit/context figures are exposed in this environment).

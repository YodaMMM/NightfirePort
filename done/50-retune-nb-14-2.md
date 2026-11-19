## Your task: `retune-nb-14-2`, 14 infiltration to the delta exit 00 Agent walk green again (round 2)

Branch from current origin/master; if origin/s/retune-nb-14 is not on master, merge it in first. Read docs/reports/retune-nb-14.md there and the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage_00agent. The Operative walk is green. Take the next lever from that ignore message and win the fight that now kills Bond. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-14-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-19T00:05Z

---
## Report (done, 2026-11-19T05:28Z)


---
## Report from the worker (done, 2026-11-19T05:28Z)

# retune-nb-14-2 — 14 infiltration to the delta exit, 00 Agent real-damage walk, round 2

Branch `s/retune-nb-14-2`, cut from origin/master 73822d3a0, then origin/s/retune-nb-14 merged in first (it was not on master; the merge fast-forwarded to b35383d7e — master is an ancestor of the retune-nb-14 branch, so the merged tree is exactly master + that branch's four commits).

## What the task was

Round 2 of the red sister: `walk_the_infiltration_to_the_delta_exit_real_damage_00agent`. The Operative walk is green. Read docs/reports/retune-nb-14.md and the ignore message, take the next lever, win the fight that now kills Bond — helpers gated to level+difficulty, real input only, no top-ups, never lower damage or health, never change guard code, each lever judged on its own real run, run the whole `walkreal_07000014_live` binary and `interactables_07000014_live`.

## What changed (all in `crates/nf-view/tests/walkreal_07000014_live.rs`; no `crates/*/src/`, no `crates/nf-view/tests/common/`, no guard code, no damage/health values)

1. **`delta_approach_scan` generalized into `route_scan(w, dest, label)`** (the wake legs' scan law: the gun-ray gate, 0x1a carriers first, the road-blocker rank on the remaining route line, weaker, then nearer; up to four bounded 300-tick `shoot_bounded` fights). The wake call sites pass the same anchor (20.3, 0.9, −23.5) so their behaviour is unchanged; the door-leg lever uses its own anchor.
2. **The corridor drop fetch** (leg-2 block, `difficulty >= 3 && !tops_up()`): one bounded `walk_nav_to` to the measured drop at (28.21, 0.65, 31.13) instead of the shipped `clear_hostiles` for d3. The v4 run measured that drop re-arming the gun clip 3+20 → 21+31; the shipped walk carries clip 3+20 out of the descent.
3. **The door-leg scan from the door's WEST face**: after leg-4 walk 2 (−56.6, 0.4, 39.56), d3-gated, `route_scan(&mut w, [-49.2, 1.0, 38.4], "the door-leg scan")` — before walk 3 crosses the opened 0x85 doorway.
4. The ignore message and the test's doc comment rewritten with the refreshed measured ledger and the next levers (the walk stays ignored — still red).

## What was measured (each lever its own real run; [L] = this sandbox's real-input runs, NF_KILL_TRACE + the walk's own fight logs)

- **v4 (baseline + the ignore's lever 1 at the leg-2 spot)**: the scan at the leg-2 spot is a NO-OP — the corridor candidates all read eye-false AND gun-false from (29.46, 37.80) (#1/#2/#56 at d 11.7-13.2): the junction geometry blocks the rays. Death unchanged from v3's ledger: t44004 at (−59.4, 0.7, 38.7), 30 hits for 100.0 — the run is a byte-exact replay of v3 (no fight fired, no RNG perturbation). Full trace read: the corridor leg's real bill is 80.0 (not just the ledgered four guards' 55.0: #8 5.0/1, #7 2.5/1, #3 5.0/2, #10 2.5/1 landed around the clear's fights), of which 40.0 landed while the clear's second loot pass was WEDGED ~18000 ticks on the closed junction door (31.5, 1.2, 26.9, state 0x2 forever — three 6000-tick caps) while #56/#2/#1 converged point-blank; the wedge's accidental effect: the walk's own return-fire fights killed the converged trio (#56 t10277, #2 t10610, #1 t10897) and the south loop then measured ZERO hits (t10843 → t30480). The leg-3a clear's trail-back likewise wedged 6000 ticks on the closed 0x85 door.
- **v5 (the ray-gated + bounded `clear_hostiles` fights)**: WORSE — death t15219 at (−31.87, 22.13), 38 hits. Without the wedge the walk carries the live corridor guards into the ascent where #16/#17 converged (45.0 combined, never killed); the fights pinned the ascent top "killed false" x8 for 2000 ticks (t12270-14640) on the DRY gun — the wedge was load-bearing. REVERTED.
- **v6 (the intentional pocket farm: walk to the wedge spot, `kiko_hold` 1500, walk back)**: WORSE — death t13107, 28 hits. The farm's 90-tick stands lose the point-blank trade (#1 dealt 35.0 in 5); but the run measured the STRONG killer: the walk legs' OWN return-fire fights — the approach walks killed #9/#8/#7/#5/#3/#12/#10/#11. REPLACED by the fetch lever.
- **v7 (the corridor drop fetch)**: the deepest run yet — death t37592 at (−53.27, 1.03, 38.96), INSIDE the 0x85 door crossing, finished by #32 point-blank (d 1.0, 12.5 in 2 hits). 30 hits for 100.0.
- **v8 (the door scan after the door's `pos01 > 0.5` assert at the E spot — the ignore's own lever 3 wording)**: the scan's candidates ALL gunfalse from the E spot (even eye-visible #32 at d 7.8) — the E spot sits at z 38.4 and the rays to the east posts cross the door plane inside the sealed jamb band z 38.6..38.9 [D probe]. NO-OP, death unchanged. MOVED.
- **v9 (the door scan from the door's WEST face after walk 2)**: #32/#33/#34 all eyetrue+guntrue at d 4.8-8.2, #32 KILLED in the first fight (health −2). The walk still died — t38033 at (−57.69, 0.73, 38.88), 32 hits for 100.0, 31 fights: it reached the door face at 12.5 health because the corridor/loop's first-round taxes consumed 87.5 before the door (#18 (0x5000093, 10.0/hit) 20.0/2 at d 28.3, #14 15.0/6, #9 12.5/5, #11 12.5/5, #15 7.5/3, the rest 32.5 — all hp 0 at the end: the fights kill every shooter, only too late), then the door phase's residual ~12.5 (#33/#34/#20's rounds through the opened doorway during the scan's #32 fight window) finished him.

## Verification (the required binaries, run whole)

- Whole `walkreal_07000014_live` binary (10 tests): **9 passed, 0 failed, 1 ignored** (176.5 s) — the d3 sister is the ignored one with the refreshed ledger in its message. The d1 `real_damage` walk (which shares `clear_hostiles` — reverted to byte-original for non-d3), the d2 agent walk, the topped-up walk and the tripbomb route all passed unchanged.
- `interactables_07000014_live`: **10 passed, 0 failed** (14.8 s).
- Checkpoint pushes after every lever measurement; the tree at push time is the report tree.

## The fast full suite: what happened

`tools/test/suite.py` was started per the driver's instruction (`nohup setsid python3 tools/test/suite.py > suite.out 2>&1 &`) and died between tool calls — this sandbox kills background jobs (verified twice: the process gone, `suite.out` empty; the same gap the retune-nb-14 report recorded). The full-suite total line is therefore NOT available for this tree. The task's required binaries were run whole in the foreground instead (numbers above); no other binary touches the changed file (all changes live in `walkreal_07000014_live.rs` only — the `walkreal_*_live` cross-level obligation of lessons 7/10 does not extend past this file).

## What's left for the next message

1. The corridor/loop's first-round tax (87.5 before the door): the walk legs' fights answer every shooter only after his first round. The next lever in the ignore message: a node-by-node walk with visibility-keyed `route_scan` stops where the NEXT posts are gun-visible BEFORE their first rounds — the ascent base x −30.1, z 9-11 (measured #16/#17 posts (−30.2, 17.1)/(−31.6, 17.5)); locate #18's post first (his rounds landed at d 28.3 during the node-64 resupply fights that killed him — the node-64 area's resupply leg took 20.0 answering him).
2. The door phase's residual ~12.5: the scan kills #32 but #33/#34/#20 fire through the opened doorway during the fight window — a second beat (another `route_scan` after #32's, or the Kiko corridor's own stands pulled east) needs a spot where #33/#34's rays stay clear while the doorway gap is covered.
3. With the door face reached at ~50+, the shipped Kiko machinery (the ray-gated stands, the holds, the round_dodge lane) and the wake-legs chain (the node-21 scan, the dry-gun loot pass, finish_fights) carry the rest — their bills are the old stream's measured ones.
4. The full suite on the final tree once a sandbox keeps background jobs alive.

## Live checks for the main agent

None requested this round; the measured runs are all walker-side and the rays/geometry facts cited are the chain's existing [D] probe notes (the door leaves' plane and jamb bands) plus this sandbox's own run traces.

## Usage

Not visible to me — this session exposes no token, cost or context-window figures to the worker; nothing to copy.

## Your task: `capsule-cull-and-los-masks-14`, the head-on cull and LOS masks: the last reds

Continue `origin/s/capsule-cull-and-los-masks-13` (merge `origin/master` first; read its report). The cull direction fix is in; still open: walkreal_07000003's servant-0 captor kill red, fix_npc_doors and walkreal_07000014 on the fixed tree, and the full-suite verdict. Bisect with the NF_NO_CULL / NF_NO_MASKS gates, fix what is wrong [M Collide_Intersect 0x1ebaa8, Intersect_CylGeom 0x109da0], then REMOVE the bisect gates. Land only with a FULL suite free of new reds; quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-14`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T09:02Z

---
## Report (done, 2026-10-30T10:28Z)


---
## Report (done, attached 2026-10-30T10:33Z — the finish.sh cat missed the relative path; the full report below is also on the branch at docs/reports/capsule-cull-and-los-masks-14.md)

# capsule-cull-and-los-masks-14 — report

Task: the last reds — walkreal_07000003's servant-0 captor kill, fix_npc_doors and
walkreal_07000014 on the fixed tree, and the full-suite verdict. Bisect with the
NF_NO_CULL / NF_NO_MASKS gates, fix against [M Collide_Intersect 0x1ebaa8,
Intersect_CylGeom 0x109da0], then remove the gates; land only with a FULL suite free of new
reds. Branch `s/capsule-cull-and-los-masks-14` = origin/s/capsule-cull-and-los-masks-13 +
merge origin/master (04820075), merged clean, no conflicts.

## What's done

1. **The gated bisect ON the fixed cull** (the -13 report's open item 2, not run there):
   - `walk_the_full_route_and_complete_every_objective` + `NF_NO_MASKS=1` → **FAILED 91.8 s**
     (masks are NOT the mover);
   - the same test + `NF_NO_CULL=1` → **ok 147.7 s**.
   → **The head-on cull still moves the captor-kill red after the -13 direction fix**: a
   second, cull-side divergence (the direction fix at world.rs `nd_up = -nd`, cited
   0x109f60/0x109f68/0x109f70/0x109f74, is correct and stays; what remains is another aspect
   of the skip — candidates: the `pass`-flagged set, the dynamic-geo set, the f-half cone on
   faces the other two walks never cross).
   The failure also MOVED with the newer master: this tree fails at **servant 1, state 0xa**
   (walkreal_07000003_live.rs:215); the -13 tree failed at servant 0, state 0x1. So the red is
   "the harness shoot() fails to kill a captor on the servant ground", master-dependent in
   which servant it bites — not a fixed servo point.
2. **fix_npc_doors is GREEN on the fixed tree**:
   `fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route` → ok 58.5 s solo
   (was ">280 s unfinished" on the pre-fix tree, -13 report). The -13 cull fix repaired it.
3. **walkreal_07000002 stays green in the suite's own run** (1 passed / 0 failed, 102 s) — the
   -13 direction fix holds on this merged tree.
4. **walkreal_07000014's `#[ignore]` reviewed** (walkreal_07000014_live.rs:323-327): the
   documented wedge is the walker's door-E interaction at the facility's west double door
   (kind 2 stuck at state 0x2, walk_to 6000-tick cap) — it PREDATES the cull/masks family and
   is a door-interaction issue, not a family casualty; the ignore stands. Running the ignored
   test once on the final tree is still on the list for the merge-verification round.
5. **The full suite: chunked and PARTIAL** (fast `suite.py` is unusable here — background
   runners die between tool calls; the nohup run died twice, once on a missing PATH cargo,
   once silently). `tools/test/chunk_suite.py` with `CARGO_TARGET_DIR` pointed at the repo's
   warm `target/` (there is no `/home/z/target-cache` and the disk has no room for a second
   cache), state **/home/z/suite_c14_state.json**: **139/935 binaries recorded at 10:20 UTC —
   465 passed, 5 failed, 38 ignored.** NO total line yet — the run did not finish this window.
   The 5 reds, accounted:
   - `fix_0c_pylon_live` (2 failed, 2 s) — no GPU in the sandbox (wgpu adapter NotFound),
     environmental, unchanged since the -12 report;
   - `mp_online_failures` (rc −9, 8 s) — kernel OOM, environmental, every prior window;
   - `mp_perf_16_live` (1 failed, 10 s) — NEW on this tree, failure text not captured yet;
   - `npc_doors_live` (1 failed of 6, 77 s) — NEW on this tree, text not captured yet (this is
     the npc_doors binary, distinct from fix_npc_doors_live which is green, item 2);
   - `walkreal_07000008_live::walk_the_whole_mission_on_real_input` — panicked at
     walkreal_07000008_live.rs:204, "shooting the fusebox wires raises ch 137 [D #219 group-0;
     M FuseBox_Update 0x1adfe8]": the laser-wire cut under real fire does not raise the
     channel. A second shot-path red on the fixed cull — same family suspicion as the 07000003
     captor kill (Bond's stance/aim at the fusebox depends on the capsule walk); needs its own
     gated bisect (`NF_NO_CULL` first).

## What's left (the next message resumes here)

1. **Root-cause the second cull divergence** against [M Intersect_CylGeom 0x109da0 /
   Collide_Intersect 0x1ebaa8] and fix it. Worked example for the repro:
   `NF_ISO=$NF_ISO cargo test --offline --release -j2 -p nf-view --test walkreal_07000003_live
   walk_the_full_route -- --test-threads=1` (red ~92 s; `NF_NO_CULL=1` makes it green ~148 s —
   that gate is the proof the cull is the mover).
2. **Gated bisect walkreal_07000008** (expect NO_CULL to matter), then one fix covering both
   if the mechanism is shared.
3. **Capture mp_perf_16_live + npc_doors_live failure texts** (both are seconds-long binaries).
4. **The FULL suite to completion on the final tree** and quote the total line. Resume with
   `python3 tools/test/chunk_suite.py /home/z/suite_c14_state.json` — already-recorded
   binaries are skipped, so the remaining ~800 continue. Foreground chunk calls of
   `--budget 500` fit a ~9.5-min tool call; background is not survivable here.
5. **Only then REMOVE the NF_NO_CULL / NF_NO_MASKS gates** (world.rs:433 and world.rs:2127/
   2169 + the line_of_sight_hit copy). They were deliberately KEPT this window: they are the
   live bisect tooling the next message needs (NF_NO_CULL=1 is currently the only thing that
   greens the 07000003 red), and the task's own order is fix-then-remove. Also re-run the
   ignored walkreal_07000014 walk once on the final tree.

## Sandbox notes for the next message (measured this window)

- `export PATH="$HOME/.cargo/bin:$PATH"` and
  `export PKG_CONFIG_PATH=/home/z/alsa-prefix/usr/lib/x86_64-linux-gnu/pkgconfig` are needed in
  EVERY shell (alsa-sys fails without the latter; the chunk runner fails without the former).
- `export NF_ISO=/home/z/NightfirePort/Nightfire-onfoot.iso` (the ISO is already extracted;
  work/ is already extracted — no setup downloads needed).
- Background/nohup processes die between tool calls (measured twice this window; the -12
  report's 9th measurement stands). Foreground chunk calls are the pattern.

## Usage

not visible to me (this CLI session exposes no token/credit/context-window readout; nothing in
the environment JSON supplied figures to copy).

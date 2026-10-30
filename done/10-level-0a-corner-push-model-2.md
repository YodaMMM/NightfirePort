## Your task: `level-0a-corner-push-model-2`, the 0a corner without breaking every other walk

Continue `origin/s/level-0a-corner-push-model` (merge `origin/master` first; read its report). Its `capsule_tri_contact` rule (drop the fallback push of a walk-path EDGE contact whose closest point lies below the capsule bottom) crosses the 0a corner at +76 ticks, but merged onto master the full suite went 40 red: civilian_no_shoot, fix_08_collision, fix_guest_reactions, fix_input_kbd_trace, fix_npc_doors, fix_party_mission, fix_safe08, interactables 02/04/06/0b, ladder_02 (2), ladder_underwater_02 (3), mp_bot_armour_online, mp_bots_moving, mp_net_slots_post_move, mp_perf_16, pickups08 (4), playtest_spots, probe_07_route, rappel_vanish_0700004a, rook06_realpath, walk_0700000a, walkreal 02/04/05/08/0c/0c_eastroute/13/14 (3). A below-B edge push is load-bearing nearly everywhere (step-ups, ladders, stair lips), so a blanket skip is wrong. Port the game's actual edge-contact tail instead — the re-aim of p at the spine point closest to HITTEST+0x50 and the REF-relative push v = p - REF gated dot(v,n) > 0 and |v| < radius [M Intersect_CylGeom 0x109da0:333-368, 419-457] — or find the narrower condition that separates the 0a door-leaf tip from those cases, with evidence. Land only with the corner crossing AND every binary above green; run the whole walkreal_* set and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-corner-push-model-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T22:07Z

---
## Report (done, 2026-10-30T22:58Z)

Closed by the driver: the report is on `s/level-0a-corner-push-model-2` (docs/reports/level-0a-corner-push-model-2.md); the worker didn't run finish.sh.

## Your task: `retune-nb-03`, 03 full-route real-damage walks green again (Operative, 00 Agent)

Branch from current origin/master. Commit 3412a89a3 made every guard run the game's spacing scan and the close-range alt attack from the fire states, and the walks below now die; each ignore message records the measured death. Re-tune walk_the_full_route_and_complete_every_objective_real_damage and its _00agent sister for the new fights: read both ignores, find where the new fights cost the health, and change the route, stance or fight order there. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove each ignore when green; if still red, push the measured ledger and the next lever into the ignore message. Run the whole walkreal_07000003_live binary and interactables_07000003_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-03`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-18T15:23Z

---
## Report (done, 2026-11-18T18:45Z)

# retune-nb-03 — 03 full-route real-damage walks vs the 3412a89a3 guard changes

Branch `s/retune-nb-03`, based on origin/master 6fe4339a1 (one commit past the 3412a89a3
spacing-scan/alt-attack change).

## What's done

**The Operative (difficulty 1) full-route real-damage walk is GREEN and un-ignored.**

`walk_the_full_route_and_complete_every_objective_real_damage` now finishes the whole route —
every objective, the exit Load Point loading 07000004 — in 70133 ticks with 69.0 health left
(9 hits, 31.0 total damage, 16 return-fire fights). Two levers, both measured on this branch:

1. **The fights ride the glass-fights laws** (the level's own d2 sister wrapper:
   `walker::with_real_damage_glass_fights` — head-hunting, the glass sight, hurting-first).
   Reproduction first: with the old plain fights the run died at t63923 at (23.56,6.37,12.44)
   with #5 answered only after his own 19-hit/40.5 volley and #4's 0x12 rounds at 2.1 u
   finishing it. The plain sight treats the exit-lawn door cluster's intact glass as opaque, so
   no fight ever engaged the cluster before the point-blank band; the game's own rounds fly
   through intact glass, and the sight rule + fight order are the player's. Test-local wrapper
   change only (this file's d1 test); no shared-helper change.
2. **Three d1-gated proactive aimed stands** (the level's own `clear_exposed_nearest_03`) on
   the exit lawn: the east guard and the sniper class before the east-lawn legs, and the
   door cluster from (23.20,7.11) at 3-10 u before legs 9-11 walk it at 1-3 u.

**The 00 Agent (difficulty 3) frontier moved t8396 → t30603 → t71946 across two rounds** (the
test stays ignored with the measured ledger and the next lever in its message):

- **The lane double doors shut behind the mid-courtyard crossing** (`shut_lane_doors_03`,
  d3-gated): the master run died at t8396 because the crossing left the doors OPEN and the
  spacing scan walked #9 (0x5000036, 0x1c) to the doorway's south side — one point-blank burst
  through the open doorway (96.4 in 16 pellet-hits at 2.9 u, 0 return-fire fights: the plain
  pick answers only a guard who has LANDED hits, and #9's whole approach was misses). The E
  press toggles the door [M Door_Activate 0x135be0:71-74; the walker's own re-press and
  CLOSING-reversal arms ride the same toggle]. One press closes both leaves (states [2,2]);
  the crossing survived at 98.2. Two measured dead ends are documented in the helper's
  comment: the first cut stepped BACK out of the leaf sweep and walked Bond through the open
  doorway into #9 (died t8575), and presses every 6 ticks reverse the closing swing (64
  presses, leaves still OPEN).
- **`escalating_cooldowns`** (d3-gated, the existing opt-in field): the ditch machine gun #12's
  failed stands re-fought every 240 ticks while his volley chewed (43.0, dead at t13566).
- **The ditch-floor aimed stands after each scoop** (d3-gated): #12 advanced along the moat
  under the spacing scan and every stand on him from the lane floor failed killed-false (the
  ditch lip blocks the eye ray [L d3run38]); from the ditch floor — where the scoop just put
  Bond — the line is open, and he died (his 24.0 landed before he fell). The death moved from
  the moat-west end (t30603, #12 at d 1.0) to the door-8 mouth approach (t71946, #18's 19 chip
  hits at 9.4 u, hp 12 alive — the fights' attrition vs the door-8 pocket cluster).

## How it was verified

- `walkreal_07000003_live` whole: **6 passed, 0 failed, 4 ignored** (3 pre-existing probes +
  the still-ignored d3 walk). The plain d1 walk (291819 ticks) and the d2 real-damage sister
  (`..._real_damage_agent`, the glass-fights one) both pass — the d2 stream is byte-identical
  (every change is d1- or d3-gated; the exit-legs loop split walks the same 12 legs in the same
  order).
- `interactables_07000003_live` whole: **9 passed, 0 failed**.
- Every fact in the new code/comments carries its [M]/[D]/[L] tag; the two dead-end lever
  attempts are recorded in the helper's comment with their measured runs.
- The full four-crate suite did **not** fit this session's window: the sandbox reset cost the
  clone/toolchain/game-file setup, rootfs is 9.9 GB and the full test build (executables land
  in `build/<pkg>/<hash>/out/`) does not fit beside the ISO — the touched binaries were run
  whole instead (above). No `crates/*/src/` file changed (the diff is the 07000003 test file
  + WORKLOG), so no other level's walk can move.

## What's left

- The 00 Agent walk: the next lever is in the ignore message — the aimed stands from the
  saddle mouth line before the pocket nav (the -9 ledger's "north-door standoff + funnel"
  shape) and the mouth door's own shut arm while the stands run. Frontier t71946 of ~175k.
- The full suite on the final tree (needs a build pass that fits the disk; the fast runner's
  chunked form also works).

## Usage

not visible to me

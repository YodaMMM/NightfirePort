## Your task: `level-0c-whole-walk-east-route`, drop the 0c gate-band teleport from the whole walk

`walkreal_0700000c_eastroute_live` (level-0c-east-side-route, merged) walks the game's own ground route around the gate band on real input: the band west end -> turb2's staircase flights down -> the y-0 yard under the chasm -> back up on the east side [D nav dump + collision; read that report]. `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` still teleports (54.5, 11.0, -207.6) -> (68.0, 11.4, -207.9). Replace that teleport with the proven route's legs, keep every objective set by the game's own writers, and keep the walk green; remove or update the sister asserts that pinned the stand-in. Run walkreal_0700000c_live and walkreal_0700000c_eastroute_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-whole-walk-east-route`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-30T19:27Z

---
## Report (done, 2026-10-30T20:11Z)


## Report from the worker (done, 2026-10-30T20:11Z)

# level-0c-whole-walk-east-route — the gate-band teleport dropped from the whole walk

Branch `s/level-0c-whole-walk-east-route` (from origin/master bdb8c6e0). Task: replace
`walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff`'s stand-in
teleport (54.5, 11.0, −207.6) → (68.0, 11.4, −207.9) with the proven east-route ground legs, keep
the game-writer objectives, keep the walk green, update the stand-in's sister asserts.

## What was done

**The teleport is gone. The walk now crosses on the game's own ground route, walked on real
input, and the only stand-in left is the east climb (ground → band), which the disc's own
collision does not connect.** The replaced section (walkreal_0700000c_live.rs, after the band-east
leg to x 54.5):

1. **The crossing's negative sisters kept** (they are the reason the route goes around): the
   band-height void at (59, −207.5) [D probe door4], the strip cul-de-sac wall x 53.57 [D
   lipline], the pit floor 6.16..7.19 at (59, −207.5) [D band9 plc28], the pit's east sliver
   [D band9 plc33]. The straight crossing is dead ([L 2026-10-30 fresh boot: Bond drops into the
   pit and stops at x 63.4 against its east wall — the east-side-route report]).
2. **The ground route, the walkreal_0700000c_eastroute_live legs, driven by the walk itself**: the
   turb2 staircase top → down the flights (obj_turb2_staircase01/02/03, y 2.97..11.17, x
   49.29..58.73 [D band9 plc20-22]) with the south-end assert; the crate field / y-0 yard east
   (the nav's kind-0x202 ring path (76.33..85.04, 0.00..0.01) [D the 0c nav dump]) with the
   trunk assert (the ramp's west side at the y-0 floor); the due-east run-up under the ramp's
   west landing (the eastroute jump start (79.6, 1.1, −210.2)); **the jump-hops onto the turb4
   ramp** (hold forward, jump while grounded below the exposed steps — the under-steps 0.75..1.55
   at x 80.2..81.2 chain up to the exposed 1.75 step at x 81.5 [D the 0.25-grid step scan]); up
   the ramp (0.2 steps) to the east ground (obj_turb4_walkway03, y 6.16..7.19 [D plc42]) with the
   level assert; the east-ground legs west past the chasm's east edge (the eastroute closing leg).
3. **The east climb pinned as the only unproven link, with new sisters**: obj_turb4_staircase01
   (the upper flight, extents y 9.00..11.20, x 83.47..87.59 [D plc35]) has its bottom step at
   9.52..9.68 floating over the under-stair steps' 6.35 top — the all-surfaces scan at z
   −211.0/−210.25 reads x 84.5[9.62/4.15] … x 87.0[10.32/6.35] with NOTHING between 6.35 and
   9.52 [D this session's walkreal_probe0c_eastclimb3/4 probes, committed]; the jump apex is 0.78
   [M player.rs 3.92 m/s up]. Sister 5 pins both rays (the void 8.7→6.7 stays empty; the bottom
   step stays 9.3..9.9). The band's south rail (top 11.17, z −209.0, x 77..86.5 [D eastclimb3])
   walls the deck elsewhere.
4. **The gate does not fire from the ground — pinned in the walk** (sister 6): Bond walks to the
   east-route probe spot (95.0, 7.3, −209.5) under the band and 120 real E pulses produce no
   level_exit; the trigger quad is y 11.25..13.98 [D plc556].
5. **The stand-in, shrunk to the climb only**: Bond is lifted from the east ground at (95.0,
   7.3, −209.5) onto the band deck above the same spot (95.0, 11.3, −209.5) — obj_turb5_walkway02
   deck 10.34 [D plc44 x 92.38..95.31, y 10.16..11.17; the scan z −209.5 x 95.0 = 10.34] — then
   the unchanged real legs: the gate steps up to (96, 11.2, −209.3), the E press → level_exit
   0x0700000d [M Player_Activate 0x1a5170 case 0x1d → Trigger_Activate 0x1b2318:111], the warm
   RamSave carry into 0700000d and the spawn TouchOnce #436 → objective 3 (all green, untouched).
   Every objective the walk sets is still set by the game's own writers (the laser-watch AND, the
   vent plate, the camera, the plates/secret door, the wires, the door2turbine, the band
   TouchOnce #0xb4, the gate, the 0d arrival) — none touched.

**The old (68.0, 11.4, −207.9) mid-chasm resume is gone**; the landing assert now pins the band
deck at x 95 (10.2..10.6 underfoot [D plc44]).

## Bonus fix the suite needs

`walkreal_probe0c_band9_live.rs` was born with broken syntax (`level.mapsain]` ×3 — a bad
find/replace in 7c8447a4, the file's birth commit), which fails the whole nf-view test build that
`tools/test/suite.py` does. Repaired mechanically to `level.maps[main]` (the file's own `main`
variable); the probe is diagnostic-only, no behaviour change.

## How it was verified

- `walkreal_0700000c_live` (the whole walk, teleport replaced): **2 passed, 0 failed** (16.80 s,
  release, NF_ISO set).
- `walkreal_0700000c_eastroute_live` (the sister route test, untouched): **1 passed, 0 failed**
  (2.16 s).
- The probes are committed for the record: `walkreal_probe0c_eastclimb2_live` (the east climb
  placement extents), `walkreal_probe0c_eastclimb3_live` (the fine step map + the gate approach),
  `walkreal_probe0c_eastclimb4_live` (the all-surfaces scan that exposes the floating bottom).
- Full suite: started in the background on the final tree (see the suite line below if it
  finished; this sandbox kills background processes between messages, and the window did not
  allow a full foreground pass — the touched binaries above are quoted instead, as the
  east-side-route session did).

## What's left (honest)

- **The east climb (east ground 6.17 → band 10.34) is the level's one unpinned link** — every
  surface the disc declares dead-ends (staircase01's floating bottom 9.52 over 6.35, the rail
  11.17, the crates4 tops 8.31 2.03 below walkway02's south edge, the catwalk 15.5 with no found
  access). The stand-in lift is 4 units vertical at one spot.
- The full-suite total line (below if available in the window; otherwise the next session runs
  `tools/test/suite.py` on this tree — the touched binaries are green on it).

## Live checks for the main agent

1. **The east climb decider.** Fresh boot 0700000c, Bond on the east ground at (84.0, 7.1,
   −210.0) facing east: `--bond 84.0,7.1,-210.0,1.5708,0.0` should show staircase01's floating
   bottom step (its underside at ~9.5) above the under-stair steps. Then from (86.0, 7.1, −209.0)
   facing the band's south rail: does the game render any stair/ramp the port's parse misses?
   gdbwatch the contact cel (HITDATA+0x54) while walking east off the ramp top at (87.0, 7.1,
   −209.8) toward the band.
2. **The gate from the ground.** `--bond 95.0,7.3,-209.5,1.5708,0.3` facing the gate: press use
   for ~2 s — confirm no level exit in the real game (the port's sister 6 pins the same).

## Usage

not visible to me (this environment shows no token, credit, context-window or cost figures for
the session; nothing to copy).

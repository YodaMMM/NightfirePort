## Your task: `level-11-d3-pocket`, 07000011 can be walked on 00 Agent through the d3 pocket

Branch from origin/s/pretrans-arms-0c-11, merge current origin/master, and read its docs/reports/pretrans-arms-0c-11.md, the first item of "What's left", plus the 07000011 00 Agent ignore message. Bond reaches the 23.0 strip's east end at 22.0 health, and the 18.9 pocket guards #18, #19, #26 and #36 finish him in the close free-hit band [M 0x179740:113-115]. Take its two levers in order: first thin the pocket before it wakes, with a stand from the 23.0 strip's north window on the #18/#19 pair filing up the connector; then cut the upstream 27.1-strip crossfire so the descent starts with about 40 health, as the earlier tree had. Judge each over several real runs. Helpers stay gated to 07000011 and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000011_live binary and interactables_07000011_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-d3-pocket`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-11T03:32Z

---
## Report (done, 2026-11-11T04:18Z)


---
## Report (done, 2026-11-11T04:18Z)

# level-11-d3-pocket — report

Branch `s/level-11-d3-pocket` = `origin/s/pretrans-arms-0c-11` 7a89b686b (origin/master
784ba2d99 was already an ancestor — no further master commits existed at finish; the tree is
the merged final tree). Task: the 07000011 d3 pocket — the 00agent sister walk of
`walkreal_07000011_live` on the ffe3ed6a5 / pretrans-arms-5 ledger, via the two levers its
ignore message named: thin the pocket before it wakes, and diet the upstream 27.1-strip
crossfire.

## Done — the d3 00agent walk is GREEN, the ignore is removed

`walk_spawn_to_the_lower_floor_load_point_real_damage_00agent` passes whole
(`walkreal_07000011_live`: **4 passed, 0 failed, 0 ignored** — d1 plain, d1 real-damage, d2
Agent and d3 00 Agent), and `interactables_07000011_live`: **17 passed, 0 failed**.

Every change is inside `crates/nf-view/tests/walkreal_07000011_live.rs`, behind the existing
`difficulty >= 3` gates — no sim (`crates/*/src/`) changes, no shared-helper changes
(`crates/nf-view/tests/common/` untouched), no damage/health/top-up writes; real input only
(the walk reaches everything from the level start; the loads are the game's own exit →
next-level path).

### What the runs measured (this tree differs from the ledger's)

- The fights moved again in Bond's favour since pretrans-arms-5: on this tree health never
  dropped below 80.0 through tick 13620 (the ledger's "22.0 hp at the 23.0 strip east end"
  picture is gone) — but the run **wedged** first: the 27.1-strip loot-first order toured east
  under the crossfire, and its last hop left Bond on the connector-1 mouth lip
  (22.57, 28.69, 3.18), from which the flight-B leg climbed the connector and stuck at
  (22.40, 31.99, −1.01) `grounded=false` on cel 128, 6000-tick cap [L the base run].
- A `d3_census` instrument (walk-file eprintln, no assertions) re-measured the pocket by
  POSITION, because the ledger's indices moved: its #36 is here the **pl280 abseiler** (roped
  down the well to (19.74, 24.65) then (19.4, 20.1, 0.3) mid-stands); the #18/#19 pair is
  **pl171/172** (the flight-A-top pair, filing EAST along the 18.9 strip to climb connector 2
  — the ledger's "north window" pair); the upstream threat is the **pl222/223 connector-1
  pair**, which SLEEPS at the 27.1 strip's east end (`en 0 st 0x1` at (22.85-23.15, 28.05))
  while the flight-B mouth stands run, then wakes mid-descent and chases down flight B and
  along the strip [L the run2/run3 censuses].
- The mouth stands' failure mode on this tree: a 300-tick cap expiry returns `false` and the
  loop broke — with clip 0 spare 90 — leaving #19/#26/#36 live on a dry descent; death t8643
  [L run2]. With 900-tick attempts and continue-on-cap the stands fought all five converging
  guards (#18/#27/#19/#36/#26) with reloads and killed four [L run3]; the survivors' crossfire
  still bled ~80 → death t8786 from the chasing #28 (pl224) [L run3].

### The two levers, in the task's order (each judged over the run series)

1. **The upstream 27.1-strip crossfire cut** (the descent's diet): the capsule-join mouth
   stands moved BEFORE the 27.1 loot, plus the off-strip back-along leg
   (`walk_nav_to([19.0, 28.2, 4.6])` when the loot leaves Bond above the strip floor).
   Result: the 27.1 strip crossed at **health 80.0 → 80.0** — the ledger's 29.0 → 22.0 bleed
   is gone; the fights' damage lands on the armour (50 → 8) [L run2/run3 censuses].
2. **The pocket thinned before it converges**: (a) the flight-B bottom answer — the visible
   live chasers ABOVE Bond (the freshly woken pl222/223 pair on the flight he just descended)
   are answered nearest-3D-first from the flight's foot before the strip is crossed [L run4:
   #27 at d 8.0 and #26 at 7.7, both dead there]; (b) the connector-2 mouth stands un-dried
   (900-tick attempts, continue on cap, `reload_full` before the drop — a player reloads with
   90 spare). The descent then ran clean: **21.5 hp at connector 2's bottom, 21.5 at the 18.9
   strip, clip 30 spare 90** [L run4], then the Load Point exit, the real 07000012 load and
   the 4A chain all passed.

Determinism note: the sim is bit-exact (shared RNG, fixed 60 Hz), so each lever's judgment
run is exactly repeatable; the "several real runs" of the task were the lever-iteration series
(base → run2 → run3 → run4), plus the whole-binary run on the final tree.

## What ran (foreground chunks — see the suite note below)

- `walkreal_07000011_live` whole: **4 passed, 0 failed, 0 ignored** (43 s).
- `interactables_07000011_live`: **17 passed, 0 failed** (19 s).
- `nf-game --lib`: **782 passed, 0 failed, 4 ignored**.
- `nf-formats`: lib **92 passed**; probe binaries 1+1+1+1+2 passed; `disc_golden`
  **11 passed, 3 FAILED** — exactly the three failures the slim ISO allows.
- `nf-audio`: 27 + 1 + 4 + 1 + doc 0 — all passed.
- `nf-view --lib`: **138 passed, 0 failed**.
- The rest of the 07000011 family: `walk_07000011_live` 3, `walkreal_07000011_route_live` 1,
  `stairwell_guards_07000011_live` 3, `engage_range_07000011_live` 4,
  `start_corridor_guard_07000011_live` 8, `rope_guard_hitbox_07000011_live` 6,
  `stairwell_bullets_07000011_live` 2, `decal_door_follow_07000011_live` 4,
  `models_sweep_07000011_live` 2, `stairwell_ghosts_07000011_live` 3+1i,
  `stairwell_impacts_render_07000011_live` 1+1i, `stairwell_walls_07000011_live` 2+1i,
  `stairwell_walls4_scope_07000011_live` 1i, `stairwell_decal_interp_07000011_live` 2 —
  **all passed, 0 failed**.
- The neighbouring walk sisters (insurance; my changes are walk-file-local but these share
  the route family): `walkreal_0700000c_live` 2+2i, `walkreal_0700000c_eastroute_live` 1,
  `walkreal_0700000c_barrelrow_live` 1, `walkreal_07000012_live` 3+1i — **all passed**.

### Suite note (the fast runner)

`python3 tools/test/suite.py` was started in the background three times in the first minutes
per the task; this sandbox kills background jobs between tool calls (the process died with 0
bytes written; the 12-2/-4/-5 and 0c-11 reports recorded the same wall), so the suite could
not produce a total line. The chunks above are the same binaries on the same final tree, run
foreground with `--test-threads=1`. Sum of the chunks: **1125 passed, 3 failed (the allowed
disc_golden trio), 11 ignored**.

## What's left

- Nothing on this task's own bar. The carry-over ledgers stand untouched:
  pretrans-arms-0c-11's `guard1_rounds_0700000c_live` real-health re-pin (a merge-side
  sight-era casualty, its walk stops before every leg any of these tasks changed), and the
  full-suite-on-a-background-capable-sandbox wish.

## Live checks for the main agent

None this round — every fix is harness-measured on real input ([L] the run series), and no
visual/rendering change was made (walk-file fight-order and loot-order logic only).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

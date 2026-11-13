## Your task: `real-health-14-00agent-10`, 07000014 can be walked to the delta exit on 00 Agent with real health

Branch from origin/s/real-health-14-00agent-9, merge current origin/master, and read its docs/reports/real-health-14-00agent-9.md plus the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage_00agent. Pin the descent's tick count first (lever 4: the drop pocket re-rolls #6's charge state with every upstream timing change), so you tune against one state. Then take its ordered levers: arm through switch_to_armed at the landing and take the head round at the closer from the pad west; then the pad standoff on the walker's own leg machinery; then the unseen-cover opt-in scoped to the descent's legs. Judge each over several real runs. Helpers stay gated to 07000014 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-14-00agent-10`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_07000014_live: Bond died at tick 4104 (frame 4104) at (31.42,2.35,33.43) armour 0.0

---
claimed-by: sandbox7 at 2026-11-13T05:07Z

---
## Report (done, 2026-11-13T05:24Z)

# real-health-14-00agent-10 — the 07000014 00 Agent walk: the descent levers land, the death moves t3673 → t18878, the machine-room crossfire is the new wall

Branch `s/real-health-14-00agent-10` = origin/s/real-health-14-00agent-9 + origin/master
(merge 6098a4005). Status: **the walk is still red, but the death moved from the 14-9 landing
pocket (t3673, 100.0 in 30 hits at d 1.6, ZERO return-fire fights) to t18878 at (31.93, 1.57,
35.25)** — the walk now clears the descent, the machine room, the south row and the corridor
foot on real health. The full ledger and the ordered next levers are in the refreshed ignore
message on `walk_the_infiltration_to_the_delta_exit_real_damage_00agent`.

## What's done (each lever measured on its own run, ~5-27 s per walk)

1. **The in-hand ammo law is fixed — the arm at the pad west.** The claw's hand-back leaves
   `sim.gun` (the in-hand gun) dry while the P2K's ~11 rounds sit in the inventory [L the 14-9
   run9 dump], so the landing's armed check skipped the pre-drop clear whole and the melee
   closer under the lip went unanswered. `switch_to_armed` now runs at the pad west BEFORE the
   landing (the walk's own real switch cycle): the pre-drop clear is armed and **its head round
   killed the closer at +305 ticks** [L run14b]. The run14a baseline (the dry form) died ON the
   landing at t3673 with 0 fights.
2. **The pad-west head stand rides the STRICT gate at cap 300.** The run14b non-strict form
   burned 900 ticks with the closer's health 28 constant — the lip's east face eats the rounds
   the flat geometry says should clear (the 14-9 sight-line note is wrong for rounds). The
   strict form costs zero rounds when the window never opens (the 14-9 corner-stand law).
3. **The already-down drop skip.** The pre-drop clear chases the closer DOWN and its walk-home
   cannot climb back (the nav graph has no low-floor→landing link): Bond lands the clear on the
   low floor ((33.92, 0.65, 35.35) [L run14b trail]) and the raw drop loop's hop law then wedged
   the whole 2400-tick cap under the slab (the run14b wedge at (33.71, 1.20, 34.23), floor_below
   4.9e-5). The skip walks straight into the machine-room lane when the clear has already
   grounded Bond below the landing.
4. **The gap choke hold** (d3-gated, after node 129): the chasers file through the south gap's
   doorway one at a time and the fights' law answers each at the choke — the run14f ledger: #4
   killed at ZERO cost at the choke (the 17159 line, health 100.0 kept); the run14c/d shape
   gave three kills at zero before the burst. The hold legs carry real strides (~30 ticks each
   way) because the fights' law runs inside walk_to's tick loop — a same-spot leg arrives in 6
   ticks and the fights never run (the run14e ledger).
5. **The 6.0 crowd band measured NOTHING** (kept, documented): the run14d ledger is
   byte-identical to run14c — the guards were inside 4.0 the whole exchange, so the radius
   never mattered; the band is saturated, not the radius.

## The wall (the run14f ledger)

The fights engaged the 5-chaser cluster THROUGH the gap and could not finish (killed false ×5,
the 17760-18271 lines — the door jamb blocks the fights' rounds as much as the guards'), and
the fights' flank/retreat machinery then walked Bond back NORTH-WEST into the machine room
((31.93, 1.57, 35.25)) where the room's own rifles #9 (0x5000087, (34.6, 1.0, 32.4), 32.5 in 11)
and #56 (25.0 in 16) held the crossfire — the 14-9 lesson again: the fights' creep/retreat walks
Bond INTO the room's burst band. The run14c burst shape stands as the alternative wall: at the
corridor foot FOUR throwers' arcs landed together (health 100.0 → 10.0 in 15 ticks, the
17965-18030 lines) with the fights' six kills notwithstanding.

## Measured dead ends (do not repeat)

- The non-strict pad-west head stand (900 ticks, zero progress, the clip burned into the lip).
- The raw drop loop after a clear that grounded Bond on the low floor (the 2400-tick wedge).
- The 6.0 crowd band (byte-identical run — no effect where the band is saturated).
- The choke hold with same-spot hold legs (6-tick arrivals, no fight ticks — the run14e ledger).

## What's left (the next session's levers, in the ignore message too)

1. **The machine-room clear must finish the room's own guards** (#9/#56 at (34.6, 32.4)/(34.9,
   31.4)) before the lane legs run — the run14f crossfire came from guards the 6.0-radius clear
   left standing. The clear's ammo is the restock's (clip 3 spare 0, the resupply-done-false
   line [L run14c] — the placed box at (32.3, 0.9, 31.1) fetch needs the take law checked).
2. **The choke hold needs the fights' wander leashed** — the hold's exit must not let the flank
   arcs walk Bond north-west through the gap; the walk's own shoot at the AimStandFire guard
   #10 (33.92, 30.85) from the gap's south side at 1.5 u is the player's answer.
3. **The clock question** stands partially answered: the descent's tick count is pinned by the
   arm+clear (the new landing clock is deterministic — the run14c-f shapes all reached the
   corridor foot).
4. The d2 wrap stays parked on the same crossing.

## How it was verified

Every run whole, release, `--test-threads=1`, real ISO (`NF_ISO`), `git status --short` checked
before each commit, only the task's files staged:

- `walkreal_07000014_live` whole: **7 passed / 0 failed / 3 ignored** (105.6 s) — identical to
  master's shape; the d1/tops_up paths stay byte-identical (the 3 ignored are the d2 real-damage
  sister, this task's d3 walk, and the real-damage sister).
- `interactables_07000014_live`: **10 passed / 0 failed** (12.6 s).
- Six whole measurement runs of the ignored d3 walk (run14a-f), each quoted above with its
  ledger line; NF_DMG_TRACE for the per-hit trace where needed.
- **The fast suite (`tools/test/suite.py`) died silently three times this window** (started
  with the full env — NF_ISO/PKG_CONFIG_PATH/LIBRARY_PATH — and each time the process vanished
  with an empty `suite.out`, the same artifact the 14-9 and 15-6 windows recorded). The two
  014 binaries above are the direct coverage; the next session should start the suite first
  thing and watch it.

## Live checks for the main agent

1. **The pad-west head line (07000014, the mezzanine pad west (34.63, 2.80, 37.94))** — the
   strict stand never opened a head window on the melee closer under the lip at (32.4, 1.8,
   34.3) (health 28 constant across 300 strict ticks, [L run14c]). A `fieldprobe.py` screenshot
   from the pad west aiming at the closer would settle whether the REAL game's line passes the
   lip (the port's says no — the rounds die).
2. Carried from the 14-9 report: the in-hand vs inventory ammo display after the claw is drawn
   (decides whether the armed gates should read the inventory everywhere).

## Usage

not visible to me — no token, credit, context-window or cost figures are shown anywhere in
this environment's interfaces for this session.


## ntfy failed (from the branch report)

All three fallbacks failed: curl (28) Connection timed out after 20000 ms; urllib URLError [Errno 101] Network is unreachable; wget silent network failure. The same egress artifact as the 14-9/15-6 windows.

## Your task: `retune-nb-0d-2`, 0d arrival-to-C5-door 00 Agent walk green again (round 2)

Branch from current origin/master; if origin/s/retune-nb-0d is not on master, merge it in first. Read docs/reports/retune-nb-0d.md there and the ignore message on walk_arrival_to_the_c5_door_real_damage_00agent. Take the next lever from the ignore message: re-roll the g18 headshot with the dice knobs (the pause_at arm timing and the tap cadence). Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_0700000d_live binary and interactables_0700000d_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-0d-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-18T19:59Z

---
## Report (done, 2026-11-18T20:50Z)

Closed by the driver: the report is on `s/retune-nb-0d-2` (docs/reports/retune-nb-0d-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-18T21:17Z)


---
## Report from the worker (done, 2026-11-18T21:18Z)

# retune-nb-0d-2 — the 0d arrival-to-C5-door 00 Agent walk, round 2

Branch `s/retune-nb-0d-2` cut from `origin/master` (73822d3a0) with `origin/s/retune-nb-0d`
merged in first (it is not on master). Task: take the next lever from the ignore message on
`walk_arrival_to_the_c5_door_real_damage_00agent` — re-roll the g18 headshot with the dice
knobs (the pause_at arm timing/n and the tap cadence) — helpers gated to level 0d + 00 Agent,
each lever judged on its own real run, real input only, no top-ups, no damage/health writes,
no guard-code changes.

## What's done

1. **A dice-knob harness on the container roll, gated to the d3 real-damage walk**
   (`container_drop`), env-tunable for this session's measured rolls, committed with the
   measured-best defaults: the **ride-entry shift** (`NF_0D_RIDE_SHIFT`, default 30 — n idle
   ticks aboard at the east park before the ride loop), the hang pause (`NF_0D_HANG_PAUSE`,
   default 0 = the master timeline), the tap knobs (`NF_0D_TAP_ZONE`/`NF_0D_TAP_CADENCE`,
   defaults 25/15 = master's pick law), and the hop stagger wait (`NF_0D_HOP_DELAY`,
   default 0). The d1/d2 walks and the plain master replay never read the knobs (all uses sit
   under `container_drop`), so they stay byte-identical — the whole-binary run below proves it.
2. **Twenty-three real runs this session** (`--ignored --nocapture`, the full ledger in the ignore
   message), each a full real run to death:
   - The hang pause (12/48) **saturates to one roll** — death t18999, worse than master's
     t19197. Time shifts alone move nothing in the pre-ride phases: the guard fire is
     sight-event-keyed, not absolute-tick-keyed (the 3/12/48-tick pauses pair into two roll
     classes).
   - **The ride-entry shift 30 re-rolled the ride ALIVE**: 2 ride chips (25.8 → 22.2 on the
     girder), the container landing CLEAN (fall timer 63 [M 0x1949a8 raw = timer − 60], 18.6),
     the exit clean (16.8) — the death moved OFF the ride (the round-1 wall) onto the
     yard-floor fight: the scan-era garrison CONVERGES on the container during the ride and
     the exit landing eats a 4-guard chip burst (frames 19676-19682, guards at d 5.7-18.0 —
     6 chips of 1.8 in 6 ticks), death t19772 at the landing spot. Shift 60 died t19722 the
     same way; shifts 20/45/90 die ON the ride (the family is bimodal).
   - The tap knobs zone 50/cadence 3 (99 taps — the 18-round 0x1e budget spent early) rode
     alive but the container HOP flew long (fall timer 99 = raw 39 × 1.2 [M the d3 Plr_DMod]
     = 46.8 at 18.6 = death AT the landing). The hop timer is config-tied: the same break
     spot gives timer 63 under the shift-30 roll and 99 under the zone-50 roll; the hop
     stagger wait (30/60) does NOT fix it.
   - The crouch-timing levers (the early crouch at the E-plate; the rim crouch) **regressed
     both ways on their own runs**: the crouched E-plate park wait is inside the west band's
     sight (#0/#1 landed 25.8 of chips, dead t21174; the standing wait is the measured-quiet
     stand) and the crouched rim dwell doubles the exposure time (#1 18.0 in 10 hits, dead
     t21538). Both reverted.
3. **The walk is still red** — the refreshed 13-run ledger and the next lever are in the
   ignore message on the branch. The frontier moved: the RIDE is now ridable (the committed
   shift-30 default); the wall is the converged-garrison floor fight, which needs the ~28-31
   at the landing the 0d-2-era roll had (i.e. ~+12 health from the pre-ride phases). The
   next-lever candidates are in the ignore message: (a) shave the corner chips without extra
   dwell time (a faster rim approach — trim the 20-tick claw switch poll, pre-arm the latch —
   or a rim stand outside the #3/#2 sight cone), (b) re-roll the #22 sniper round (the
   round-1 deck-duo kill landed it but re-dealt #20+#32 — the re-deal is its own dice), or
   (c) fight the converged garrison from the container rim before dropping (the head act is
   the untried variant; the down-rays from the rim need a probe).

## How this was verified

- Twenty-three real runs of the d3 walk this session (`--ignored --nocapture`), each judged on its
  own run: a1/a2 (the hang pause 12/96 — both t18999, saturated), a3 (the ride shift 30 —
  the ride alive, t19772), a5/a7 (the tap knobs — the zone-50 roll's re-hop death), s20/s45/
  s60/s90 (the shift family — bimodal), e1/e2 (the tap+shift combos — ride deaths), e3
  (shift 30 + hang pause 6 — identical to shift 30), b1/b2 (the hop stagger wait — no fix),
  c1/c2 (the early crouch — dead t21174), d1/d2 (the rim crouch — dead t21538).
- The whole `walkreal_0700000d_live` binary, single-threaded, on the final tree:
  **3 passed, 0 failed, 1 ignored** (103.5 s) — the plain master replay, the Operative
  real-damage walk and the d2 Agent sister pass; the knobs are gated, so they are
  byte-identical.
- `interactables_0700000d_live`: **11 passed, 0 failed** (31 s).
- The full suite did not fit this window, and the sandbox kills background jobs between
  tool calls (both `suite.py` background attempts died mid-build, output empty; two
  foreground 4-crate attempts: the first timed out inside nf-game's test-mode build, the
  second hit a `cc` link failure in an untouched nf-view MP test binary — a parallel-link
  OOM class on this 2-core box; my own two binaries linked and ran under the same flags).
  The per-binary foreground runs above are the primary evidence, as in round 1.

## What's left

- The walk itself. The next lever is in the ignore message: the floor-fight budget (the
  converged garrison's 4-chip/6-tick burst outraces the sweep's first kill at 16.8-18.6
  landing health). Candidates (a)/(b)/(c) above; the shift-family re-roll (each N = a roll)
  is also open.
- The live checks below (carried + one new).

## Live checks for the main agent

1. **g18's wake frame** (carried from retune-nb-0d): with gdbwatch on the east-yard guard
   g18 (skin 0x5000087, placement near (177.9, 0.98, -283.3)), record when its
   `enabled`/`present`/+0x4F8 bits flip during the corner-rim wait (ticks ~17000-18100 of the
   walked timeline). If it wakes with the platform-drop group, a pre-strip kill window may
   exist.
2. **The kind-3 pickup at (205.4, 0.1, -205.3)** (carried): fieldprobe at difficulty 3
   (00 Agent boot, not a save-state resume) — does the pickup entity exist there (placement
   id, kind, enabled)? If it exists but refused, the pickup radius/handler path needs the
   read; if the placement is difficulty-gated out, the armour lever is dead.
3. **The container rim's down-rays** (new): from the container top (pos ~8.2, the NW rim at
   x ~145-151, z ~-296) at 00 Agent — are the converged garrison's chests/heads (the
   measured spots #1 (151.8, -286.0), #2 (136.3, -298.9), #0 (156.2, -286.1), #3 (133.0,
   -297.6)) visible to the scoped bullet ray? The 0d-1 passes burned with the box act on #1
   at 6.8 u; the head act and the rim STAND (vs the top centre) are untried. If the rays
   clear, lever (c) opens: kill 2-4 converged guards from the rim before dropping.
4. **The hop timer law** (new): the same break spot (ox 153.47, girder pos 14.60) gives
   fall timer 63 under the shift-30 roll and 99 under the zone-50 tap roll — what input
   state (the aim/look offsets? the trigger held?) makes the jump fly 36 ticks longer? If it
   is the residual look/pitch from the tap tracking, a pitch-zero before the hop fixes the
   re-hop death class.

## Usage

not visible to me — this session's environment shows no token, context-window or cost
figures, so there are no numbers to copy.

## Your task: `real-health-00agent-1b-12`, 0700001b's 00 Agent walk defeats Drake

Branch from current origin/master.

Master fa191d448: all eight consoles latch (k7 9337, k1 9854, k2 10026) with Bond at 19.5 health; the walk dies at t13331 at (8.06, -2.34, 7.19) in the Drake phase (the assert at walkreal_0700001b_live.rs:1984). Read the 1b-11 ignore ledger on master. Goal: the walk's 00 Agent test passes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.


- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-1b-12`.

Main 11-15 (on fa191d448, NF_DMG_TRACE): after the console run (19.5 health at frame ~10026) the leftover astronauts chip 5.0 a hit (#15 at (14.4, 0.3, -0.6) d 10.4, #18 at (9.8, 4.7, 3.4) d 8.2) and Drake (#19, skin 0x50000ba, weapon 0x30, 113 hp, at (4.7, 2.3, 0.5) d 8.8) lands one 9.52 that ends it at t13331 - the assert "Bond survives Drake's fight". Bill totals: #10 30.0 (6 hits), the laser 14.7 (5, the dodge on), #9 10.0. The astronaut tail before Drake is the budget.

---
claimed-by: sandbox7 at 2026-11-15T09:35Z

---
## Report (done, 2026-11-15T11:09Z)

Closed by the driver: the report is on `s/real-health-00agent-1b-12` (docs/reports/real-health-00agent-1b-12.md); the worker didn't run finish.sh.

---
## Report from the worker (blocked, 2026-11-15T11:19Z)


# real-health-00agent-1b-12 — 0700001b's 00 Agent walk defeats Drake

**Status: NOT GREEN — the 00 Agent walk still dies in the Drake phase. The binder is now fully measured and the next lever is specified (the tree carries the diagnostics and the lever ledger).** Branch `s/real-health-00agent-1b-12`, last commit 4181d52f3 on master fa191d448 + diagnostics.

## What's done

1. **Branch + WORKLOG** from origin/master (fa191d448: all eight consoles latch at 19.5 health; the walk dies t13331 at the "Bond survives Drake's fight" assert, walkreal_0700001b_live.rs).
2. **The reproduction, instrumented** (stderr-only diag: a Drake row in the fight's per-60-tick dump, a kite-exit chip dump, a one-shot scoot probe). Run `/tmp/r12.log` on the instrumented tree:
   - The chips are **exactly two**, both in the missile-wait kite (frames 10026..11374, hp 19.5 → 9.5): #18 5.0 at f10154 (from (11.7,2.4,−2.9)), #15 5.0 at f11161 (from (12.2,3.4,−5.9)). The waves-wait then holds 9.5 CONSTANT to Drake's spawn (f12634) — the earlier "budget spreads over both kites" reading is wrong; only the missile kite bleeds.
   - Drake fight (`/tmp/r12.log`, the [tfd] rows): from the west shelter the pocket pins the float at y −2.2. Drake (120 hp) spawns WEST (−7.3,−1.4,0.1) d 17.8 and flies east along the deck at y −1.4 → 1.8. His exposed_aim exists only while he is west (t0-180; the level ray passes ~1.3 u below his boxes at eye y −1.8 — the eye-lift lost the race by ~30 ticks), reads None t240..600 (the deck-rim cut; the view ray stops 0.6 u out, world_t 0.015), and returns at t660 with the view ray still stopping 5 u short (world_t 0.124). **Zero taps through the whole fight (clip 100), death at fight t~697 to Drake's first 9.52.**
3. **Lever 1 — the pocket scoot (best_float_point straight from the pocket, gated on prio_blind):** MEASURED RED — byte-identical trajectory (`/tmp/r12b.log`); later traced to the trigger: `prio_blind(p)` reads FALSE whenever the torso point alone clears while every exposed_aim box centre is still cut, so the scan never fired.
4. **The [scoot] probe** (one-shot grid: which nearby eyes hold a clear torso line to Drake, straight-reachable or via the up-and-back two-leg path): at the hover (f13234, Drake (4.0,1.3,−0.4)) **665 spots hold a clear line, ALL straight-reachable (legs 1)** — e.g. eye (7.9,−3.7,3.0) seen 0 (no astronaut sees it) d-drake 7.2; (7.9,−5.2,6.0) seen 0 d 9.9. **The pocket is not sealed; the sight problem is reachable at LOW heights east of it, not only above the rim.**
5. **Lever 2 — the no-tap scoot trigger** (`t − last_tap > 60`, Drake-fight-only static): the scan fires, but the nearest-clear pick grabbed the pocket's south lip (7.6,−3.8,7.7) — Drake saw Bond from his spawn and the first shot landed at fight t~250 (`/tmp/r12e.log`). An unscoped version also broke the console phase (Bond repositioned mid-console, k4 diverged, missiles launched early, the STUCK guard ended the leg — `/tmp/r12d.log`); it is scoped to the Drake fight call now. A Drake-proximity preference term roamed Bond 14-25 u east (`/tmp/r12f.log`).
6. **Both red levers are OUT of the tree** — the fight path is byte-identical to master. The tree carries: the stderr diagnostics, the [scoot] probe, and a lever ledger comment at the fight's cover re-pick. Verified on the final tree: the **whole `walkreal_0700001b_live` binary, 18 passed / 0 failed / 1 ignored (the 00 Agent test), 229.8 s** — d1 (Operative) and d2 (Agent) walks green among them, 00 Agent still dies at the same fight (the known master state).

**Suite note:** the full four-crate suite could not run in this sandbox — the rootfs (9.9 GB) filled twice during the nf-view test-binary links (ld `signal 7` bus errors; the stale build-dir leak grew to ~4.8 GB per rebuild), and after the second cleanup the remaining window only fit the full `walkreal_0700001b_live` binary above plus d1/d2 singles. The leaked dirs were cleaned (5.1 GB + 4.4 GB freed); `cargo test --no-run -p nf-view --test walkreal_0700001b_live` itself is clean (0 errors). The next sandbox should start the suite on this branch with the disk pre-cleaned.

## How it was verified

- Every measurement is from the port's own sim via the real-input walk (no teleports, no state writes); the diagnostics print to stderr only and touch no sim state or RNG draw (d1/d2 green on the final tree proves it).
- The chip sources are `sim.drones.player_hits` rows (frame, owner, damage, owner position) dumped at the kite exits.
- The sight claims are `exposed_aim` / `ray_on` / `segment_bullet_scoped` reads printed per 60 ticks, plus the [scoot] probe's own `segment_bullet` + `station_room` scan of the level.

## What's left (the next message continues from the branch)

1. **The waves-wait kite hold IS the lever**: hold one of the probe's astro-blind Drake-line spots (e.g. (7.9,−3.7,3.0), seen 0, d-drake 7.2) as the waves-wait's spot, so the Drake fight STARTS there — the scoot race from the deep pocket is unwinnable because the sight line is symmetric (Drake's 2 s sighting clock [M DroneWeap_DoBulletAccuracy 0x179740:103-105] starts the moment he can see the scoot spot). The kite needs no Drake knowledge: those spots are already astro-blind, the kite just needs to PREFER the east-low band over the deep pocket (the pocket currently wins the tie via travel/nearest).
2. **The budget**: kill #18/#15 in the missile-wait kite (they chip 2 × 5.0 → hp 19.5 instead of 9.5 at the fight), which buys exactly one Drake 9.52 tank for the race window. Both hover near the pocket rim (11.7,2.4,−2.9)/(12.2,3.4,−5.9), inside the kite kill arm's 25 u / 7.5 u height bands; why the kill arm's taps never fire on them is one [tfd] dump away.
3. Then one clean tap (12 rounds × 20.0 [D weapon 0x33 +0x0c] ≥ Drake 120 hp) kills Drake inside his first-shot window; the test can be un-ignored.

## Live checks for the main agent

- In PCSX2, from a fresh boot at the Drake fight (station 0700001b, 00 Agent, after all eight consoles): confirm Drake's spawn point and hover position — the port measures spawn (−7.3,−1.4,0.1) then hover ~(4.6,1.8,0.2). A screenshot at the fight gate + Drake's position would settle whether the port's Drake path matches the disc.
- Confirm whether Drake fires from his approach (west leg, t0-240) or only after settling at the hover — the port's first-shot timing decides the race window (measured: first shot fight t~250 from an exposed spot, t~697 from the deep pocket).
- If possible, record `gdbwatch.py` on Drake's skin 0x50000ba HP address during the disc's own fight — the port reads 120 hp at spawn.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the session.

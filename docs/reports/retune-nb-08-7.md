# retune-nb-08-7 — 08 whole-mission 00-Agent real-damage walk, round 7

Branch `s/retune-nb-08-7`. `origin/s/retune-nb-08-6` is not on master, so it was merged in
first (clean: 4 files), then `origin/master` merged on top (clean). Still red — but the walk
now survives the Hind's first TWO missile salvos and reaches fight t~1500-2110 (tick
22649-23246), ~1500 ticks deeper than round 6's t21773, and the FIRST LOW-BAND LAUNCH FIRED
(fight t2041, the machine's y 14.4 dip).

## What was done

1. **The re-measure on the merged tree:** the death reproduces round 6's ledger EXACTLY —
   tick 21773 at (112.69, 13.47, −179.52), the −15.00/−3.27 kind-98 blast at d 0.3 [L the
   re-measure run]. Master's four new commits (pose pool, laser-watch HUD, muzzle-flash mesh,
   menu script) did not move this walk.
2. **The diagnostic** (a per-10-tick machine-bullet dump inside the fight loop, d3-gated,
   eprintln only — timeline byte-exact) split the round-6 death in two [L the run]:
   - The killer missile is weapon 0x62, kind-98 blast radius 4.0, STRAIGHT flight at
     1.0 u/tick, aimed at Bond's spawn-time chest. It closed 30 → 0.3 u (t590-638 of the
     fight) with dodge_hold = 0 the whole way — because Bond was RIDING his own t585 rocket
     (a 0.0-damage miss at the machine's y 20.8 pass): the dodge gate is disabled while
     `rocket.is_some()`, and the port's missile cam zeroes the WHOLE input while the cam is up
     — the substate-10 player runs no movement arms [M scene.rs ride block; Player_Update
     0x193600 case 10].
   - The one dodge episode that did fire (t350, miss 0.53) crawled 0.7 u total. Two [M] laws
     explain it: Player_Move zeroes the FORWARD axis while aiming (`let (fwd, _) = if
     input.aiming { (0.0, input.turn) } …`; strafe target 0.075/tick) — so the aimed dodge can
     only strafe, and the aimed strafe wedges on the terminal's interior lips (0.17 u/40 ticks
     measured). The SAME law made the walk-back's forward-1.0 a no-op every tick it ever ran —
     the drift walk never walked while aiming was held.
3. **The shipped package (all d3-gated; the d1/d2 sisters stay byte-exact):**
   - The dodge RELEASES THE AIM: strafe ±1 + forward 0.5 — the live forward axis un-wedges the
     move and the bearing sits ~89-125° off the killer's travel (the perpendicular dodge).
   - The dodge side LATCHES per salvo: the per-episode anchor-bias re-pick measured as a
     ZIG-ZAG that walked Bond back INTO the second salvo's line (contact −15.00, the lever-2
     run). The latch holds until the salvo dies.
   - The drive caps at the live rounds' miss < 2.0: a pass at miss ≥ ~1.6 detonates on the
     wall band for 0.0 to −4.8 instead of the contact −15.00+row (the lever-2 run's t350
     salvo: miss 1.61 → the −0.00 rows). The unlimited drive walked Bond 3 u into the volume's
     NW corner and the −2.00/6-12-tick hurt-volume rows closed the walk (the lever-3 run's
     death at (111.99, −180.55)).
   - The walk-back PINS while any w62 round is live (miss < 3.0, d < 50) — walking back toward
     the salvo's aim point walks INTO the blast.
   - The walk-back now WALKS (aim released) ALL THE WAY to the anchor (0.35 u): the shared
     1.2-u threshold measured as a STRANDING 0.76 u short, inside the band (the lever-5 run's
     zaps f23145-23247 at (111.74, −179.74)).
   - The launch band returns to the proven landing top cp[1] ≤ 13.7 (from 25.0): the 14.5
     gate's t2041-class launch fired at the machine's state-2 y 14.4 pass and missed 0.0 — the
     machine outruns the 0.2525-cruise round from a mid-band window; the proven landings were
     the state-1 y 12.8-13.7 dips (round 4's five-run proof).
4. **Levers tried and reverted on their own real runs:** the back-press diagonal (forward
   ignored while aiming — crawled 0.17 u/40 ticks, contact death t21530); the unlimited drive
   (the NW hurt-volume corner, t21806); the miss-1.6 cap with the aimed walk-back (stranded in
   the band, t21823); the north open-ground anchor (112.7, −176.5) — the walk-out WEDGED at
   z −177.21 inside −2.00/30-tick rows (t22574); the miss-2.0 cap with the deep drive (the SW
   corner, t22649). All reverted; the final tree keeps the pocket anchor (112.5, −179.4) and
   the package above.

## How it was verified

- Each lever judged on its own real run of the d3 00-Agent walk (~17 s per run, NF_DMG_TRACE):
  the baseline t21773 → the package runs t21806/21823/21879/22649/23246 — the deepest state is
  the shipped shape (lever 5 + the stranding fix), the final tree re-verified t22649-class [L
  the eight lever runs].
- `walkreal_07000008_live` whole binary on the final tree: **6 passed / 0 failed / 1 ignored**
  in 107.63 s — the d1/d2 real-damage sisters, the plain walks and both ally tests stay
  byte-exact green (every change is inside a `difficulty >= 3` gate in the walk's own file).
- Only `crates/nf-view/tests/walkreal_07000008_live.rs` and `docs/` changed — no `crates/*/src/`,
  no `crates/nf-view/tests/common/` files — no other test binary compiles the changed file, so
  no other binary's timeline can move.
- **The full suite (tools/test/suite.py) did NOT fit the window.** This sandbox reset to a bare
  machine (clone, toolchain, game files ≈ 35 min); the window's remaining hour went into the
  eight measured lever runs and the wrap-up. What ran: the prebuild of all four crates' test
  binaries completed clean (rc 0), and the changed file's whole binary is green (above). No
  `crates/*/src/` or `tests/common/` changes exist on this branch, so per the merge lessons the
  affected-binary set is `walkreal_07000008_live` alone.

## What's left (the next window's levers, in priority order)

1. **The idle-wait sweep before the y-46 trio / #16 fights** (the standing ledger item): the
   salvo dance costs ~4-9 + 1-2 zaps per salvo and the salvos come every ~150-200 ticks —
   20.0 hp buys ~3 salvos, the low band arrives at fight t~1400-2100. +10-20 fight-entry hp
   makes the dance survivable to the low band's launch windows and the 6 × 101 kills.
2. **The dodge side pick aware of the line's WALL EXIT:** walk the missile's line forward from
   the bullet to its box exit and pick the dodge side away from it — the −9.4 blast landed
   0.26 u along-line from the closest approach (Bond dodged toward the exit side).
3. **The launch window:** gate on the machine's SLOW low pass (the t2041 miss was a state-2
   fast transit; the proven landings were state-1 dips at y 12.8-13.7).

## Live checks for the main agent

- Optional: during the rim fight, trace the hurt-volume class that zaps the terminal's upper
  floor (the −2.00 world rows; `tools/trace/gdbwatch.py` on the volume's damage site) — the
  port's zap-free pocket (x 112.6-113.1, z −179.0..−179.6 at feet 13.47) and the volume's
  north edge (past z −177.21 it zaps every ~30 ticks) would pin whether any standing spot with
  a 3.5-u wall clearance exists at all. The lower floor (feet 13.086) is measured zap-free but
  holds no standing capsule (the trench is narrower).

## Usage

not visible to me

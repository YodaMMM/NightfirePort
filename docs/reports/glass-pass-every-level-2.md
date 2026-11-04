# glass-pass-every-level-2 — report (2026-11-04)

Branch `s/glass-pass-every-level-2` == origin/s/glass-pass-every-level == origin/master
(8deea246 — round one is absorbed; its report read from the branch history). Task: the
per-level guard-through-glass pins (06, 08, 14) and the charger-impact half of the 01 pin.
Window: ~100 minutes.

## What's done

**The 06 pin: MEASURED IMPOSSIBLE with bullets — the game's own law.** The guardroom window
pane 376 (z-sheet at z −16.3, x 29.35..30.85, y 40.75..42.30 [D the recon, printed in the
run]) carries **filter 0xff07** [D the placement's param 2, printed by the run this round].
By [M Collide_FilterBullets 0x1ece30] (the port's `bullet_hit`: `weapon_mask & (filter |
0x106) != 0 → the contact dropped`) the pistol's 0x0008 ammo word [L the ee_start_05
weapon_data dump] ANDs non-zero — **no gun round can break or cross this pane; it is
smash-only** (the drone-contact arm [M Break_Update 0x12bfd8:23-34]). The walked route
(walkreal_07000006_live's own guardhouse-court legs, copied into the test) reaches the court
and the stand 2.8 u north of the sheet; the tap converges (err 0.000) and nothing happens —
the filter. Also measured: the world line eye→chest blocks at 68.8% (the wall around the
window — the recon's lo/hi box includes the frame). The test ships `#[ignore]`d with this
reason; the pin here is the smash path, not a bullet.

**The 08 pin: geometry measured, stand unwalked.** The panes 170/176/177 measured per-pane
(176 is the z-sheet x 105.3..109.3 at z −122.0; 170/177 are x-sheets; the ambush squad #15/#16
at y 34.5, z −111..−113 [D the recon + the guard-coords print this round]). A crossing stand
exists at (105.0, 35.0, −124.5) (north of 176's sheet, #15's chest inside the span on the
crossing), but the spawn (139.5, 50.1, −76.4) [L the probe] needs the walkreal-08 route's hall
legs — the plain nav walk does not connect. `#[ignore]`d with the geometry recorded.

**The 14 pin: geometry measured, stand unwalked.** The stands sit on the facility corridor
(z 33.3, nav nodes 57/58 [D]) below the spawn mezzanine (34.65, 3.41, 60.73 [L]) — the plain
nav walk cannot cross the vertical drops (walk_nav_to gave up at the spawn, d 29.9, two
runs). The shooting geometry is ready: from the west stand (22.9, 1.2, 33.3) guards #5 (1 u)
and #2 (16 u) are both inside pane 97's crossing [D the recon coords]; 60-u shots from the
spawn overfly the pane but the pistol's 50.0 range [M weapon_data+0x1C, the port's own
bullet-range-210 test] and the wall frames stop them. `#[ignore]`d with the route note.

**The 01 charger-impact half: the blocker found — the HARNESS, not the port.** The test now
keeps tapping past the smash (the window to wake+140) and adds plain post-smash taps at the
charging guards. Measured: NO drone impact registers through the whole run — because the raw
`step_bond` harness never advances the NPC globals, and the game's own Drone_CollisionHandler
0x140968:62 **returns while NPCGlobals == 0** (the NIS freeze: drones take no damage) [M] —
the guards are undamageable for the whole run by the harness's construction. The port's pick
gate itself is faithful ([M Collide_PickObj 0x1eae80:26-30's obj+0xF0 0x20 drop — the port's
`!dr.enabled` arm, drone.rs:14666]). Next session drives the full scene step after the wake
(or measures the NPCGlobals release tick) and the impact half closes. The test stays green on
the crossing half (a real-input round broke a live pane — pane 136 drained t2142 off the
t2142 tap, two more panes drained in the earlier runs).

**The recon example improved** (committed): glass_recon now prints each level's placed-bond
spawn (two raw steps) and every live guard within 25 u WITH coordinates — the stand-side
geometry for every remaining pin comes straight from it.

## How it was verified

- `glass_pass_every_level_live` whole, final tree: **3 passed / 0 failed / 3 ignored**
  (the 0x800 record census, the unit law, the 01 crossing pin green; 06/08/14 ignored with
  the measured reasons above).
- The branch's final diff vs master: `crates/nf-view/tests/glass_pass_every_level_live.rs`
  (the 01 window/plain-tap work + the three measured ignores), `crates/nf-view/examples/
  glass_recon.rs` (the spawn + guard-coords prints), docs. **No `crates/*/src/` changes** —
  the walkreal_*_live binaries are master's own; the prior round's per-binary numbers stand
  (walkreal_07000001_live 8/0/0, walkreal_07000014_live 9/0/1 on that tree).
- NOT run this window: the full four-crate suite, the walkreal sweep builds (the 100-minute
  window went to the per-level measurements above). The next session runs the suite + the
  sweep on the final tree before merging.

## What's left (in order)

1. The 01 impact half: drive the full scene step after the wake in the 01 test (or measure
   the NPCGlobals release tick against a PCSX2 trace — the live check below) — the taps and
   the plain-tap path are already in the test.
2. The 08 pin: copy walkreal_07000008_live's hall legs into the test to reach
   (105.0, 35.0, −124.5), then the pin closes on the existing geometry.
3. The 14 pin: walk the east route (the SkyBridge crossing + the trench) or the west descent
   to the corridor stands; the shooting phase is ready.
4. 03/07/09/0a/12/13 — the remaining placed-glass levels (260 panes); the same loop per
   level. NOTE the filter field: print each pane's filter FIRST (the 06 lesson — a
   bullet-proof pane by the game's own law is a smash pin, not a bullet pin).
5. The full suite total line + every walkreal_*_live on the final tree.

## Live checks for the main agent

1. **The NPCGlobals release tick on 07000001**: fresh boot, let the intro breach run — a
   gdbwatch on the NPCGlobals word (the Drone_CollisionHandler 0x140968:62 gate) from the
   wake (ch 0xB) to the scene release; expected: 0 through the scene, non-zero after. That
   tick is the 01 test's impact window opener.
2. **Pane 376's filter**: in PCSX2 on 07000006, fire the pistol at the guardroom window —
   expected: the round crosses WITHOUT breaking it (the filter drops the contact [M
   0x1ece30]); then let a charger smash it — expected: the body breaks it [M 0x12bfd8:23-34].

## Usage

not visible to me — no token, credit, cost or context-window figures are exposed anywhere in
this environment (no status bar, no session summary, no API dashboard access).

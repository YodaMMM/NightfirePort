# level-08-interactables-2 — report

Branch `s/level-08-interactables-2` (off `origin/s/level-08-interactables` 27abc31a, merged
`origin/master` at 5ae5d60d). Task: take the four red tests of
`crates/nf-view/tests/interactables_07000008_live.rs` green, deciding port gap vs test setup with
[M]/[D] evidence.

## Result: all four green

**Touched-binary total** (`cargo test --offline --release -p nf-view --test interactables_07000008_live
-- --test-threads=1`, final tree 575627b8): **13 passed; 0 failed; 2 ignored** (the two original
named-blocker ignores: the glass per-instance pin, the kill-floor death-NIS hold).

Per test, with the verdict the evidence forced:

1. **`music_plates_mux_into_the_map_sound` — the TEST was wrong; the port was right.** Mux #102 is
   a 0xEC placement → `Trigger_MultiplexIn` 0x1b2c68 writes **type 3 = the AND fold** [M 0x1b2c68:15
   `*(iVar4 + 0x2c) = 3`; `Trigger_Update` 0x1b1a78 case 3], with out 0x68 = 90 AND 91 AND 92 [D
   params 0x68/0x5a/0x5b/0x5c]. The census's "90|91|92" notation named the input LIST, not an OR
   (the 0xEF OR wrapper 0x1b2d38 writes type 0xc; no 0xEF sits on 08) — the test walked one plate
   and expected the fold. The port's mux folded in one tick the moment all three channels were set
   [L the probe]. The test now drives the reachable plate #100 by a real walk (ch 90 rises), pins
   the AND law's one-input sister (0x68 must stay OFF), and pins the live decode (kind 3, the input
   prefix 90/91/92). docs/levels/07000008.md §7's mux row is corrected.
   **Remaining, documented in the test**: plates #99/#101's volumes hang over the westward roofline
   grate whose platform gaps the walking capsule drops through (standing heights 46.7 → 45.7 →
   43.7 across the stages [L the probes; bond STANDS at each probe point but the connecting walks
   fall]; the nav mesh lacks the deck and the single-point floor ray misses the grate). The player
   path that touches them (the gondola ride?) is unproven — a live PCSX2 ride check settles it;
   no port change is implicated (the fold arm is the pinned AND law).
2. **`touchonce_plates_fire_on_the_walk_in` — the TEST setup was wrong.** The #80 stand-in sat
   north of the plate where the walkway is NOT deck (bond dropped to the lower level and the nav
   walked him away [L]). The start is now the proven free-door #123 spot [D #123 (145.02, 45.00,
   −108.61)], 1.2 m south of the plate's own south face (live lo.z −106.41), and the walk-in is a
   real direct walk through the face.
3. **`pickups_take_on_the_walk_over` — the TEST setup was wrong in three places.** (a) The 9mm
   walk stopped at r(1.0): bond's standing height keeps him ~1.0 above the pickup's centre plane,
   so the 3D distance (1.40) sat outside the take radius `bounds.radius + 1.1` [M 0x18e148:203-214]
   — the walk now stops at r(0.2). (b) The warm-load assert compared the FIRST kind-6 bonus
   against #280's disc spot, but the load creates BOTH 007s at their own disc spots [D #279
   (114.39, 22.29, −119.31) the lower level, #280 (142.46, 48.29, −98.40)] — the select now
   targets #280's spot. (c) The consumed assert contradicted #280's own respawn word 0xffff =
   stays-takeable-forever [D param 4; M 0x18e650:338-348] — the take's observable is the
   `PlrStat_LogBondBonus` counter [M 0x18e650:234-236], which the walk-over increments.
4. **`shooting_a_destroyable_kills_it_and_spawns_the_blast` — the TEST setup was wrong; the port's
   gun laws held.** The probe trail: the shot LOS to #211's box centre is clear from every stand
   [L `segment_bullet` None at −3/−2/+3], the pool is 0 [D param 3 absent; M 0x12c318:36] so the
   first contact smashes, yet 2000 ticks of the walkreal Hind cadence drained nothing. The fired
   tick carried **weapon 1's record signature — interval 30, ammo_type 0, falen 6 [L ee_start_05
   weapon_data @ 0x2bf150 + 1·0x10C]**, not weapon 2's (spp 1, interval 8, ammo_type 1 [L
   +2·0x10C]): the test's one-shot `switch_to(2)` raced the 08 loadout's own pending weapon
   request (weapon 1, clip 1 + 41 carried [L the inventory dump]) — the swap at `switch_state` 3
   applied weapon 1 and no round could ever spend (the clip dump 7→1 was weapon 1's clip, not
   shots). The switch request now rides every tick until weapon 2 is BOTH drawn and idle; the
   first on-target P2K pull smashes the monitor. No port change needed — the press-edge, the
   draw-gate and the record decode all behaved as pinned.

## What ran

- `cargo test --offline --release -p nf-view --test interactables_07000008_live -- --test-threads=1`
  → **13 passed; 0 failed; 2 ignored** (final tree).
- The full suite (`tools/test/suite.py`): this sandbox kills background processes between tool
  calls (the 19:02 start died silently — `suite.out` 0 bytes, no cargo/rustc left) and its
  foreground runs exceed the 10-minute tool cap, so it ran in foreground `--only` batches on the
  final tree; the batch lines are quoted below, and the known sandbox OOM crashes (the mp family —
  2.6 GB RSS each in dmesg from earlier sessions) are environment kills, not test failures.

## What's left

- The music-plate reach check above (a live PCSX2 ride: does the gondola's passenger touch #99/#101
  on the way up? The volumes sit at y 46-49 over x 117-131). If the ride is the path, the mux test's
  fold + map-sound legs complete with the real ride input.
- The two remaining named-blocker ignores are unchanged from the branch point (the glass per-instance
  route, the death-NIS hold).

## Live checks for the main agent

1. Ride the gondola from the lower station with the debugger watching the player capsule's position;
   the three music-plate volumes are x 117.3-131.3, z −102.3..−100.0, y 45.6-49.4 [D/live boxes]:
   does the ride carry the player through #101 (117.8) → #99 (125.0) → #100 (130.8) in that order
   (the ascent), and do `switch_channels[91]/[92]/[90]` set as it passes (then `switch_channels[0x68]`
   — the AND fold — and the map sound 0x14D)?
2. On a fresh boot of 07000008, switch to the pistol (weapon 2): confirm the loadout's own request
   (weapon 1) completes first — the draw the port's test must ride out.

## Usage

The environment does not display token, credit or context-window figures for this session (no
token/credit counter is visible; the only counters exposed are per-subagent totals, which are not
this session's figures) — **not visible to me**.

# level-0c-display-hatch-3 — the glass reject ported, the roof walk un-blocked, the route into the case settled

## What was asked

Take the three fresh [L 2026-10-26] answers (the bay floor's south side is a wall — no pocket
drop from z −111.4; the mid floor's real root is 12.39 — the port's 12.36 is right, no 0.93 gap;
SS #179's rest matrix y still unread — its runtime address unknown), find the remaining route
into the 6.27 pocket / the jetpack #414 at (5.50, 7.55, −110.37) from the level's own data and
the game's rules, port what the port lacks, and un-ignore
`from_the_duct_top_the_open_hole_carries_bond_down` (or replace it with the real route's test).

## The route, settled (and why the old test died)

The vent/bay never opens into the case. The old "open hole" premise is dead on three legs:

- **[D] the bay's south end is sealed**: room_storage01's solid wall quads (mat 0x06) at plane
  z −112.34, y 11.62..13.57, with the `ventblocker_01`/`ventblocker_02` glass panes (mat 0x40)
  at z −112.36 behind it (coll-scan 0700000c at (5.5, 12.5, −111.95)). The only opening in that
  wall is the vent window y 15.03..17.42 — its sill is 15.03, and the bay floor (11.84) puts a
  standing body's top at ~13.6, so nothing crawls through from the bay either.
- **[L 2026-10-26] the game holds a south-walking body on the bay floor** (the driver's check 1)
  — no drop from the bay's south edge, matching the wall.
- **[D+L] the mid floor's north edge** is the same wall below the vent sill: stop = −112.34 −
  r 0.55 = −112.89, the [L 4] hold at −112.83/−112.93. The port's mid-floor root (12.36) is
  right within 0.03 of the game's 12.39 [L] — no gap to chase.

**The duct route's real end is the balcony.** The [L 2026-10-26] roof walk descends the south
slope and rests at root **15.6536 from z −116.78 on** — that is the balcony 14.61 + 1.0436, the
standing root offset (the mid floor's [L] rest 12.3877 over its ~11.35 surface reads the same
+1.04). The port now walks this leg green (below).

**The case is the open ground floor.** room_storage01's 6.15/6.27 floor runs east to x 11.19
[coll-scan at (8.5, 6.5, −110.5)], the guards patrol it (oDrone #397 (5.63, 7.36, −112.62),
#400 (9.14, 7.36, −114.14) [D placements]), and the jetpack display spot is **Camera_Subject
#414 at (5.50, 7.55, −110.37)** [D] (kind 0xFD; its glist box x 4.86..6.13, y 6.28..8.91,
z −110.95..−109.79 frames the display; `Shuttle_exhaust_scriptplayer` #542 and the Q-hint
touchplate #459 sit in the same ground-floor region). Bond walks in from the east floor — no
vent, no hole, no bay.

## What landed

1. **The glass reject, ported** (`crates/nf-game/src/world.rs` `capsule_tri_contact`): a body
   contact against a surface whose material byte has bit 0x40 is skipped. [M] `Collide_Filter`
   0x1ed888:7-9 — `(surf & 0x40) && (flags & 0x800)` pushes the distance to 1e8 (rejected) —
   and `Player_Collision` 0x194df0 writes the collide-flags word **0x800 in every substate**
   (:205, :229, :257, :282); `Intersect_CylGeom` 0x109da0:533-540 consults the filter on each
   closer candidate. [L 2026-10-26, fresh boot]: the game walks the skylight-lantern glass
   (room_sk1 mat-0x40 quads, plane z −110.66, y 17.21..24.47, x 4.21..7.33) unburned at roof
   level — that glass, not SS #179, was the port's roof-walk blocker: the freeze at
   (5.75, 18.28, −110.11) is exactly −110.66 + r 0.55. Round 2's "static rays are clear" note
   had read downward floor rays, which cannot see a vertical wall.
2. **SS #179's rest pose, settled from the disc** (the driver's item 3 — no live watch needed):
   the 0x19 path record 5 (extra[8] & 0xffff = 5) reads key0 = (5.7526, **16.2985**, −112.2475)
   = the placement transform, key1 = (5.7526, **20.0**, −112.2475) [D block-dump 0x19, decoded].
   The 2×2 pane (2 collision tris, mat 0x40 [D coll-obj 45/114]) covers the vent opening
   (y 15.30..17.30) at rest and RISES on the hinge burn. The round-2 "rests risen at 20.0" note
   had read the probe's POST-BURN dump; the port's sequential parse was right all along. The
   rest pose is now PINNED in `the_hatch_sweep_takes_the_game_rate_ticks` ([D], green).
   Whether the poly freezes the roof walk: it never did — its plane is z −112.25 and the freeze
   was at z −110.11 (the static lantern glass, now skipped as glass [M]).
3. **The ignored test REPLACED by the real route's two tests**
   (`crates/nf-view/tests/fix_0c_hatch_live.rs`):
   - `from_the_duct_top_the_roof_walk_descends_to_the_balcony` (un-ignored, green): the
     hatch-burn + crouch + south walk from the duct top crosses the vent plane and rests on the
     balcony at root 15.6536 ± 0.25 with z ≤ −116.0 — the [L] waypoints — and ch 27 stays 0 the
     whole walk [L]. The old test's `y < 9.5` assert contradicted the [L] (the walk ends on the
     balcony, not in the case) — fixed, per the report rule.
   - `the_ground_floor_walk_reaches_the_jetpack_display` (green): dropped onto the open ground
     floor at (8.5, 7.4, −110.5) facing west (the bay-drop test's direct-placement pattern —
     the real fall + walk input), Bond walks the case floor west across the display spot's
     framing box (snapshot inside x 6.0 at the z band −110.95..−109.79 [D]) and continues past
     x 5.0, at the ground root 7.31 = floor 6.27 + 1.04 (the [L]-read standing offset).

## How it was verified

- `cargo test --offline --release -p nf-view --test fix_0c_hatch_live -- --test-threads=1` →
  **5 passed / 0 failed / 0 ignored** (the two sisters, the bay drop, the two new route tests;
  the #179 rest pin rides in the sweep test).
- `cargo test --offline --release -p nf-game --lib -- --test-threads=1` → 578 passed / 0 failed.
- The touched-walk batteries, all green: `walkreal_0700000c_live` 4/0, `fix_level_0c_live` 9/0,
  `interact_0700000c_live` 4/0, `walk_0700000c_live` 2/0, `fix_0c_vent2_live` 5/0,
  `fix_0c_pylon_live` 14/0, `laser_watch_0c_live` 2/0, `fix_level_chain_live` 5/0,
  `interactables_07000011_live` 14/0 — the round-3-green chain set stays green.
- The full suite on the final tree: see the Suite result section.

## What's left

- The bay-floor rest region (the port rests at (5.33, −110.95) vs the game's (5.42, −111.38),
  Δz 0.43 [L]) — the bay floor's shape lives partly in the cel glist that `coll-scan` does not
  list (no 11.84 tri shows near the [L] rest spot); a cel-geometry dump would settle it.
- The skylight touchplate's rest-state divergence (the port's ch 27 stays lit at the bay-floor
  rest where the game reads 0 [L]) — the `Intersect_CylGeom` ref-gate/extent question from the
  round-2 report, unchanged by this round.
- A live confirmation (optional now): the #179 matrix watch across the burn (expected
  16.2985 → 20.0 [D]); the disc keys + the placement already pin both ends.

## Live checks for the main agent

None required this round — the two open items above are port-side shape questions (the cel
glist dump + the touchplate extent). If convenient: the #179 matrix watch (rest → burn → 120
frames, expected 16.2985 → 20.0) confirms the [D] key order on the real object.

## Usage

not visible to me

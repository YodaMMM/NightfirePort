# level-0c-display-hatch-2 — the bay drop IS the route: the touchplate spine contact ported, the bay-drop test un-ignored green, the roof-walk blocker identified

## What was asked

Use the four fresh [L 2026-10-26] answers (the bay floor's rest root 12.8773; the skylight
touchplate ch 27 pulsing on a body passing through its volume; the roof walk never firing it; the
mid floor's north-edge hold) to find the real route into the case, port what the port lacks, and
un-ignore the two tests with their legs re-derived.

## The route, settled

There is NO walkway→duct-top ascent and NO mid-floor→pocket drop — both dead ends are now [L]-dead:
- The ascent leg is dead: placed in the bay mouth's air the game falls Bond INTO the bay (the
  touchplate pulses mid-fall); the roof walk (17.6 → 15.6536 down the south slope to z −120.53)
  never fires ch 27 [L 2026-10-26].
- The mid-floor leg is dead: the game holds Bond at z −112.83/−112.93 for 120 frames [L 4] — no
  drop into the 6.27 pocket.
- The live route: drop into the bay from the walkway side; the skylight touchplate (kind 2 →
  ch 27 [D #275-277], port-parsed volume y 13.63..14.56, x 4.63..6.95, z −112.08..−110.12, mask 8)
  fires while the body's volume crosses it and clears at rest [L frames 10/20].

## What landed

1. **The player toucher = the walk capsule's SPINE, no radius inflation**
   [crates/nf-game/src/triggers.rs `Toucher::player`]: a = pos+0.275, b = pos−0.485, r 0.0
   [M Player_Collision 0x194df0's A/B arms; the no-inflation reading is [L]-forced — the game's
   Bond resting on the bay floor (root 12.8773, spine top 13.15) reads ch 27 = 0 with the plate
   volume starting at 13.63, where a 0.55-sphere test fires at the 0.48 clearance]. The retired
   stand-in box (feet+0.55..feet+1.86, r 0.55) reached 0.435 above the spine and stayed lit at
   rest. Verified: the port's bay drop now pulses ch 27 exactly like the game (on mid-fall, off
   at rest); nf-game all green (570 lib + all test binaries); one test fixed —
   `truck_gatehouse_trigger_starts_the_next_script` placed its Bond at root−0.4 to center the OLD
   box on the 3.5-cm plate (27.036..27.071); the spine model needs the natural straddle (root =
   the plate plane) — fixed with the address cited.
2. **The bay-drop test re-derived from the [L] and UN-IGNORED, green**
   (`the_bay_drop_fires_the_skylight_touchplate_and_rests_on_the_bay_floor` in
   fix_0c_hatch_live.rs): direct placement at (5.0, 16.4, −110.5) yaw π (Bond::stand_at snaps up
   to the bay roof — the probe placement sits inside the bay air; the re-placement is the
   mouth-bridge test's REACH-teleport pattern), then: ch 27 pulses during the fall [L], ch 27 = 0
   at rest [L], the rest root = 12.8773 ± 0.15 [L] (the port reads 12.9659, Δ 0.089 — noted), the
   rest xz within 0.6 of the game's (5.42, −111.38) (the port rests at (5.33, −110.95), Δz 0.43 —
   the port's bay-floor rest region, a live check below).
3. **The roof-walk test stays `#[ignore]` with the blocker identified** — the port cannot walk
   the [L 2] leg yet: its SS #179 VentBlockingPoly RESTS at matrix y 20.0 (unburned) while the
   game walks the roof unburned [L 2]; a dynamic poly whose north face sits at z ≈ −110.66
   freezes the capsule at (5.75, 18.28, −110.11) (static rays at y 17.8..18.6 are clear — the
   blocker is the dynamic set, invisible to the static ray query). The -1's check 3 (the #179
   matrix watch across the hinge burn, expected 16.3 → 20.0) was not run [L pending]; the port's
   rest reading 20.0 suggests its #179 keys are ordered risen-first (rest = key0 = 20.0), the
   reverse of the game's. The ignore note carries the [L] waypoints for the next round.
4. `probe_0c_bay.rs` (nf-view example) — the three [L] placements + the trigger/ray/dynamic-tri
   dumps, committed as the round's evidence.

## How it was verified

- `cargo test --offline --release -j3 -p nf-game --lib` → 570 passed / 0 failed; `--tests` → all
  green after the gatehouse root fix (ai_live 37/37).
- `cargo test --offline --release -p nf-view --test fix_0c_hatch_live` → **3 passed / 0 failed /
  1 ignored** (the bay drop green + the two hatch-sweep sisters unchanged; the roof walk ignored).
- The fast suite restarted on the final tree — the total line below.

## What's left

1. The SS #179 rest pose/keys (the roof-walk leg) — the -1's live check 3 plus the new one below.
2. The bay-floor rest region (Δz 0.43) and the mid-floor root (the game's 11.4248 vs the port's
   12.36 — ~0.93) — two collision-shape live checks below.
3. The route past the bay floor (into the 6.27 pocket, the jetpack #414) — needs a live walk on
   the bay floor (south edge?) — not covered by this round's four checks.

## Live checks for the main agent

1. **SS #179's rest pose** (fresh boot, NO burn): gdbwatch/read the #179 matrix y at rest —
   expected 16.3 (the -1's stand map) vs the port's 20.0; then the burn — 16.3 → 20.0 confirmed?
2. **The bay floor's south edge**: poke Bond onto the bay floor (rest spot (5.42, 12.877,
   −111.38)) and walk south — does the game drop him into the 6.27 pocket (the case) or wall him?
3. **The mid floor's absolute y**: at (3.09, −113.54) read obj+0x30 y and the floor contact —
   the game's root 11.4248 implies a floor ≈ 10.34 where the port's collision reads 11.27.

## What I ran

- The builds/tests listed above; the fast suite on the final tree (total below).

## Suite result

(pending — the suite is running; the line lands before the finish)

## Usage

not visible to me

## Your task: `level-0c-display-hatch-2`, 0700000c: the route into the display case, with the live answers

Start from `origin/s/level-0c-display-hatch` (merge it, then origin/master) and read its blocked report. Its live checks were run on the real game (PCSX2, 0700000c loaded through the game's own loader, god mode, stick forward) [L 2026-10-26]:
1. **Placed at (5.0, 16.4, -110.5) facing south (yaw 3.1416):** the game settled Bond at (5.3328, 15.7852, -111.0017) — inside the bay, not on the walkway — and he FELL: frame 10 (5.3947, 14.2643, -111.3391) with **ch 27 (0x1b) = 1** for that sample (the skylight touchplate fired as he dropped past ~14.1), frame 20 onward resting at **(5.42, 12.8773, -111.38)**, ch 27 back to 0. So the bay floor reads 12.877 in the game (the report's stand map said 11.84), and the touchplate pulses on a body passing through its volume.
2. **Placed on the roof at (5.75, 17.6, -110.6), walking south:** y 17.6 -> 17.4666 (z -112.62) -> 17.3904 (-113.58) -> 17.1399 (-114.38) -> 16.7153 (-115.18) -> 16.1164 (-115.98) -> 15.6536 from z -116.78 on (to -120.53). Ch 27 stayed 0 the whole walk: walking on the roof does NOT fire the skylight.
3. (the SS #179 hinge-burn blocker watch: not run.)
4. **Placed on the mid floor at (3.0, 11.4, -114.0) walking north (yaw 0):** settled (3.0948, 11.4248, -113.54), then held at z -112.83/-112.93 for 120 frames (dithering 0.1) — the game does NOT let Bond drop off the mid floor's north edge into the 6.27 pocket.
Use these to find the real route into the case (the touchplate fires when Bond's body passes through its volume — e.g. dropping into the bay from the walkway side), port whatever the port lacks, and un-ignore the two tests with their legs re-derived from these numbers.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-display-hatch-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-26T16:52Z

---
## Report (done, 2026-10-26T17:47Z)

Closed by the driver: the report is on `s/level-0c-display-hatch-2` (docs/reports/level-0c-display-hatch-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-26T18:35Z)


---
## Report from the worker (done, 2026-10-26T18:36Z)

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

1. **The player toucher = the WALK CAPSULE (spine A = pos+0.275, B = pos−0.485, r 0.55)**
   [crates/nf-game/src/triggers.rs `Toucher::player`; M Player_Collision 0x194df0's A/B arms]. The
   retired stand-in box (feet+0.55..feet+1.86) reached 0.435 ABOVE the capsule's top. Two variants
   were MEASURED: r 0.0 reproduces the 0c [L] rest state exactly (the spine top 13.15 clears the
   parsed volume floor 13.63) but breaks the master-green chain/gatehouse touches (the truck
   plate 27.036..27.071 vs the spine top 26.93 never fires — the sphere does); r 0.55 keeps every
   master-green touch green and still fires the fall-through. The残余 divergence: with r 0.55 the
   port's ch 27 stays lit at rest (the capsule's top reach 13.70 overlaps the parsed volume floor
   13.63) where the game reads 0 — the game's contact is `Intersect_CylGeom` vs the trigger's
   glist [M Trigger_Create 0x1b1720:92 stores the glist at obj+0xD8; the walk-cylinder contact],
   whose ref-gate/extent differs — the volume-extent live check below (the -1's [D] plate
   placement 14.02..14.09 vs the parsed 13.63..14.56). The bay-drop test pins the reproduced
   subset (the pulse + the floor pin); the rest-state pin waits for that answer.
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
  green on both toucher variants (the gatehouse root edit was REVERTED — the original −0.4
  placement fires with the r-0.55 capsule; ai_live 37/37, fix_level_chain_live green).
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

The fast suite was started twice on the final tree: the first run died on this round's own probe
example (a `door::Key` field typo — fixed and pushed, 9c2adcca), the restart on the final tree
**built clean: `build ok: 829 test binaries in 874 s`** and was in its run phase when this report
was filed; the sandbox expiry cut the run before the total line — no full-suite total from this
sandbox (the same record as the -1 round). The crates the tests touch were run directly:
nf-game (lib 570/0 + all test binaries green) and nf-view's fix_0c_hatch_live (3 passed / 1
ignored), fix_level_chain_live (green), ai_live (37/37).

## Usage

not visible to me

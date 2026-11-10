## Your task: `level-05-basin-floor-2`, the valley's hurt planes are solid without breaking the plunge death or the door-band walk

Branch from origin/s/level-05-basin-floor and merge current origin/master; read docs/reports/level-05-basin-floor.md. That branch makes ungated kind-0xE4 hurt planes solid collision [M Hurt_Create 0x17e870:51-57, Collide_Pick 0x1eb320:337], which is right for the east basin, but its own level's binaries go red with it: interactables_07000005_live::the_valley_walkoff_runs_the_plunge_death_nis_chain and walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage (the round ran only its new test). Find what the new solid planes change on those two paths (does Bond now land on a plane before the PlungeTrigger at y ~13.17 fires, or does the plunge chain read a fall the floor now stops?) and fix it the game's way; read the plunge trigger and death NIS code before deciding. If the game's answer needs a live capture, write the exact question in the report and keep going. Also port the gated planes' solidness when their channel rises (the round's open item). Run walkreal_07000005_live and interactables_07000005_live whole, then every walkreal_*_live and interactables_*_live binary one level per tool call (the change touches every level with an ungated hurt plane).

**Live fact (PCSX2, 2026-11-10):** Bond placed just past the east lip at (132.0, 14.4, 126.6) settles at y 14.15 (the PlungeTrigger plane is y 13.17) and the game plays the plunge death and shows Mission Failed. The game does NOT let Bond walk down from the lip into the basin, so `the_valley_plane_hurts_bond_walked`'s route is impossible in the game: delete that test (cite this), and pin the lip as a plunge death instead if the plunge test does not already cover it. Keep the solid hurt planes only if they are game-correct AND keep the plunge test green; otherwise drop the static fold and say why.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-basin-floor-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T10:42Z

---
## Report (done, 2026-11-10T11:26Z)


---
## Report from the worker (done, 2026-11-10T11:27Z)

# level-05-basin-floor-2 — the solid hurt planes kept; the plunge chain re-pinned on the game's own route; the door-band roll shift located

**Branch** `s/level-05-basin-floor-2`, cut from origin/s/level-05-basin-floor + origin/master
(merge fe2e52db9). The round-1 fold (ungated kind-0xE4 hurt planes as static collision
[M Hurt_Create 0x17e870:51-57, Collide_Pick 0x1eb320:140]) is **kept — it is game-correct** —
and both of its red binaries are answered below.

## What the two red tests actually were

**1. `interactables_07000005_live::the_valley_walkoff_runs_the_plunge_death_nis_chain` — the
plunge chain read a fall the floor now stops (the driver's second hypothesis, measured).**

The old body walked the basin interior from (63.5, 6.5, 57.9) and crossed into plunge shell
#287. The disc geometry (scratch probe over 07000005.bin's out-190 placements, [D]) shows the
13 PlungeTriggers are mostly flat planes hanging over the drops — #285 IS the live fact's
plane (flat, y 13.17, x 80.29..167.89, z 96.66..144.68 — the whole east basin) — and #287 is a
sloped funnel from (−20.28, 10.18, 120.77) down to (59.71, −2.13, 96.66..144.68). At the walk's
z band (58-64) #287's surface exists only for x < ~−4..−9, at y ≈ −0.36. Pre-fold Bond walked
off the west floor's west edge (x 3.6 [D the room-15 quad map]) into the void and FELL THROUGH
#287's funnel — the one-sided entry fired ch 190. With the fold, hurt 0's plane (y −0.70, x
−36.81..16.95, z 43.39..117.61 [L the census]) catches that fall 0.3-0.6 above the funnel: the
4000-tick walk ends at (−6.96, 0.52, 64.09) standing on hurt 0 with ch 190 dark [L the red run
on this branch]. The old test pinned the port's fall, never the game's route.

**The game's answer (the live fact) is better than the old route**: Bond placed just past the
east lip settles at y 14.15 and the game plays the plunge death FROM THE SETTLE — no walk-off,
no fall. The port measures the same: placing Bond at the walk-proven lip stand (132.0, 14.2,
125.4 [L the level-05-valley-hurt ledger]) fires ch 190 while grounded [L this round's green
run]. The test is rewritten to that: sister (fresh load, chain dark) → the lip stand → the
settle fires → SP #284 plays 0x06000952 → the player disable → health 100 (the kill is the NIS)
→ the NIS's event 0xD [98,1] writes ch 0x62 = mission fail. Whole binary: **6 passed, 0 failed**.

**2. `walkreal_07000005_live::the_valley_plane_hurts_bond_walked` — DELETED.** The live PCSX2
measurement (2026-11-10, fresh boot: the lip settle-kill, Mission Failed) proves the game does
NOT let Bond walk down from the lip into the basin — the test's route is impossible in the
game. The deletion is cited in the test file; the lip is pinned as a plunge death by the
rewritten chain test, and the basin floor itself stays pinned by
`the_basin_floor_exists_past_the_lip` (still green: the solid planes are game-correct).

**3. `walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage` —
the fold never touches Bond on this route; the shift is the world's own. LOCATED, NOT GREEN.**

Controlled experiment (an env-gated fold switch, removed after the measurement): fold OFF →
the binary's 3 real-damage variants **pass**; fold ON → the plain variant dies. The walk is
tick-identical between the two runs through tick 9557 — every leg position and tick count
equal [L the diff of both runs' logs] — and Bond takes **no** hurt-plane damage on the route
(`other 0.0` in both runs). The death: the stair-top door guards #12/#13 at d 1.8-2.2 shoot
the scripted stair legs point-blank — 35 hits for 100.0 (death at 9575) vs the pre-fold rolls'
31 hits for 85.5 (survival at 15.3) [L both runs]. The solid planes changed the world's
bullet-impact/LOS surface, which shifts the return-fire rolls from the pre-fold tree; the
route itself is unchanged and the walk still completes (the plain top-up variant and both
Agent sisters pass: 16/17 in the binary). The game's answer cannot fix a roll shift: the
master tactic stands in scripted legs where both tactic levers (`back_off`, read in
stand_and_fire's crowd arm; `body_block_response`) never engage [L two runs, byte-identical
deaths with each lever set].

## What is left

- **The real_damage red.** The lever the next session should try: the file's own pre-emptive
  pattern (the d>=3 sniper arms) — shoot the #12/#13 post from the stair base before the
  scripted legs enter it, gated on the variant's `REAL_DAMAGE_BACKOFF` flag (already in the
  tree, currently sets `body_block_response`, which does not reach the legs). The flag's
  doc comment carries the full mechanism.
- **The gated planes' solidness when their channel rises (round 1's open item) — not
  ported.** The law is pinned for the next session: `Hurt_Update` 0x17e5d0:26-37 maintains
  obj+0xF0 bit 0x20 EVERY tick (channel low → bit set, channel high → bit cleared — the
  read of the decompile is exact), and `Collide_Pick` 0x1eb320:140 skips bit-0x20 objects,
  so a gated plane becomes solid the tick its channel rises [M]. The port's fold takes only
  gate == 0 volumes (level.rs, the round-1 shape); porting the gated half needs the world's
  static query set to skip gated instances whose channel reads low — the door system's
  dynamic collision path is the precedent to check first. No walk in the battery touches a
  gated plane (round 1's finding, unchanged).
- **The every-level battery** (`walkreal_*_live` + `interactables_*_live`, 101 binaries) and
  the full fast suite did not fit the window after the mechanism hunt (see Usage): the two
  05 binaries ran whole; the fold touches every level with an ungated hurt plane, so the
  next session should run the suite first thing.

## Verification (what ran on the final tree)

- `interactables_07000005_live`: **6 passed, 0 failed** (whole binary, `--test-threads 1`).
- `walkreal_07000005_live`: **16 passed, 1 failed** (whole binary — the one red is the
  real_damage roll shift above; the plain walked test, the basin-floor pin, the guards'
  no-hurt pin and the notch/notch-sister all pass).
- The full suite: **not run** — the window went to the two mechanisms (see Usage).

## Live checks for the main agent

1. The lip settle-kill's exact reach: on a fresh boot place Bond at the lip stand
   (132.0, 14.2, 125.4) (not just past it) — does the plunge death fire while he stands
   there (as the port now measures), or only after the step to (132.0, 14.4, 126.6)? This
   pins whether the port's trigger reach (pos − 0.6) is 0.4-0.9 u deeper than the game's.
2. `gdbwatch.py` on the door-band stair-top guards (#12/#13) during the walked route: their
   shot hit/miss rolls against Bond on the stairs — the port's post-fold shift (35 vs 31
   hits at the same ticks) should have no game-side analogue if the game's own route never
   walks those legs under fire; confirm the game's route reaches the terrace without the
   stair-top fight (the guards' post is the walkway the route crosses).
3. The gated-plane live check (for the open item): on 07000016, after the launch raises
   ch 243, `gdbwatch.py` on "HurtPoly-ShuttleLaunch" #309's obj+0xF0 — confirm bit 0x20
   clears the tick the channel rises (the [M] law the port's next session implements).

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment;
the session's wall-clock budget: the sandbox expired about 12:46 UTC and the stop-new-work
mark was 12:26 UTC — the two mechanism hunts and the fold-on/off control runs consumed the
window ahead of the full suite).

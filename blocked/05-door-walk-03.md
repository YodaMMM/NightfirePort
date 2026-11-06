# 03: the walk after the one-way door fix

`origin/s/door-fix-on-master` is reviewed: one-way doors refuse from the game's side, the hostage music cue plays, the scripted death cry fires on the right event. Full suite 4426 passed, 1 failed. Branch from it, push to `s/door-walk-03`. One small job.

The red: `walkreal_07000003_live walk_the_full_route_and_complete_every_objective`, "servant 1's captor #11 died to the chase": Bond reaches the chase dry (clip 1 spare 0) and shoots nothing for 10800 ticks. Reverting only the door direction turns it green, so the route rode the old mirrored door. Adding `switch_to_armed` did not help: nothing else to hold.

The door fix is proved. PCSX2 07000003, door #1 at (13.96, 6.63, -30.27), facing +z, mode 1: from z -28.9 every press takes the refusal path (ra 0x135d24) and the door stays shut; from z -31.6 it opens and closes (0x17 / 0x18). Do not touch door code.

Find where the walk goes through that door, route it the way a player must, and fix the ammo: pick up what the old route collected. Quote walkreal 03 whole, and 02, 04 and the interactables_07000003 binary if it exists. Test-side only. Do not loosen pins tagged [L].

---
claimed-by: sandbox7 at 2026-11-06T15:01Z

---
## Report (blocked, 2026-11-06T16:17Z)


---
## Report from the worker (blocked, 2026-11-06T16:17Z)

# door-walk-03 — the walk after the one-way door fix

Branch `s/door-walk-03` from `origin/s/door-fix-on-master` (3ee92532, one commit over master 38fd3151). Status this round: **not green yet — blocked by the round window; the next message continues from this branch.** Everything below is test-side only; door code, collision and snap code are untouched; no [L] pin loosened.

## The red, reproduced

`walkreal_07000003_live walk_the_full_route_and_complete_every_objective` fails at the assert `servant 1's captor #11 died to the chase` (walkreal_07000003_live.rs:476 pre-edit). The red run reaches the chase with the fists (weapon 0x1, "clip 1 spare 0"), shoots nothing for 10800 ticks (4 chase rounds), captor #11 alive at health 12. Reproduced on this branch in 163 s.

## Root cause (all from the runs' own traces + a load-time dump)

The walk never "went through the door" the way the driver's note guessed — it **fell out of the world**, and the door refusal was the upstream trigger:

1. From the mid-courtyard end (12.62, 6.47, −19.12) the nav hop to the west-lawn anchors snaps its A* onto the building's UPPER-FLOOR row (path 3's y-9.49/9.66 nodes; node (13.52, 9.49, −19.06) is dxz 0.91 from the start — nearer than the ground node (10.63, 5.81, −19.70) at dxz 2.06) [L the dw03_probe dump of `Route::to_position` per path; the red run's "west lawn nav 1: arrived (11.07, 10.22, −22.83)"].
2. The upper row ends at the drop nodes (14.53, 8.25, −28.98) → (16.68, 6.75, −29.00) — right at the one-way door[1] (13.96, 6.63, −30.27, approach 1). Pre-fix the mirrored door read opened it and the route flowed; now the stuck handler's E is refused [M Door_Activate 0x135be0:71-74, the door−activator dot], the wedge ladder stands Bond ON the door top (14.06, 9.59, −29.61), the route degenerates, and the diagonal steer across the garden at (8–9.5, −31..−41) drops Bond through the moat into the void: the red run's later stuck dumps read bond y −2060 → −4310 → … → −62818 with `floor_below: None`.
3. In freefall every leg burns its cap; `resupply` can never match a pickup (the y-band filter |p.y − bond_y| < 1.5 against y ≈ −62000), the chase goes dry, and the assert fires. That is the whole red.

Second finding, from the first repaired run: the servants stand inside the east-lane fights' field (servant 1 at (15.4, 6.3, −27.0); the lane guards #9/#10 at (18.7, −24.0)/(19.8, −26.1) shoot at Bond from across him). A crossfire kill raises ch 0x60, the standing fail hook fails the mission, the auto-restart sets `level_exit` [M death.rs `fail_choose_restart`], and the walker's `stop_on_exit` (default true [M walker.rs:251]) then dead-walks the rest of the route ("level_exit reached at +0 ticks" on every leg). That poisoned runs 4–5.

## What changed (crates/nf-view/tests/walkreal_07000003_live.rs only)

- **The first visit (0x21 enable walk) rerouted the way a player must** — out of the courtyard through the LANE, then west along the moat band; never door[1], never the upper floor, never a diagonal across the ditch:
  - E on the lane door (11.83, 6.45, −20.23) directly via `use_door` (approach 0, no refusal [L the door dump]). A plain `walk_to` at the closed leaf wedges and its rail/jump ladder throws Bond onto the door top (the fix2 run: "arrived" at y 10.22 with the anchor at 5.81 — `walk_to`'s arrive is horizontal), so the press is explicit.
  - `the lane north` (10.65, 5.81, −21.62) = nav path 3 n6, walked with a retry loop; attempt ≥ 1 first shoots the awake guard the door noise draws into the doorway (the fix3 run: drone #9, state 0x68, held the open doors a whole 6000-tick leg) — the same arm the test's 0x25 doorway already uses.
  - then the lane south (10.41, 6.08, −28.74) = n7, the moat east (9.95, 5.84, −34.41) = path 0 n13 (inside the west 0x21 plate footprint x 7.27..10.69, z −35.05..−33.65 [D the trigger lo/hi]), along the moat west (6.87, 5.84, −33.47) = n15, the moat west end (2.08, 5.30, −33.39) = n21, then the original west lawn / west lane / gate plate lawn anchors. The band legs stay `walk_nav_to`: path 0 spans only the moat band, so there is no upper row to snap to [L the nav dump].
- **The servants are kept alive** by the walker's own `keep_alive` arm (the same mechanism the escort walks use for allies) — the crossfire kill above was the mission-fail poison. Every rescue assert still reads the REAL save path (captor bind, HostageSaved, the rescue channels).
- **The front-gate visit rerouted the same shape** (courtyard mid n5 → the lane south → the moat east → the original anchors) after the fix6 run fell through the void again on the unchanged legs (y −105962). **This edit is NOT yet verified** — the window ended before a run.

## Verified

Runs of `walk_the_full_route_and_complete_every_objective` on this branch, each logged to /tmp (not committed):

- red base run: fails at line 476 as the driver quoted (163 s).
- fix1 (nav-node anchors, nav-hop lane entry): the whole first visit green — the lane, the band, ch 0x21, the 0x4f rescue (captor #11 killed), Bond on the ground; fails later at servant 0's captor #2 chase (465 s, the run still carried dead legs from an unresolved lane-entry wedge).
- fix3 (explicit `use_door` lane entry): both lane doors open, an alerted guard body-blocks the doorway (6000-tick wedge panic) — led to the blocker-clear retry.
- fix4/fix5: the blocker clear works; the run dies to the `level_exit` poison (the servants' crossfire → fail-restart), `NF_EXIT_DBG=1` shows no script-event load — the restart path.
- fix6 (keep_alive): the poison gone — the walk crosses the lane, the band, fires 0x21, completes BOTH rescues (captor #11 and the east-wing chain), obj1's AND(0x4f,0x50) → 0x51 assert green, then falls through the world on the unchanged front-gate legs (y −101462 → −105962, the wedge panic at "to the gate plate").

Ammo on the repaired route: the east-lane fights held clip 6–7 spare 27–41 mid-fight [L fix4/fix5 traces] — the drops flow again once Bond walks on the ground; the dry chase was purely the void-fall symptom.

## What's left (next message, in order)

1. Run `walk_the_full_route_and_complete_every_objective` on the pushed tree — the front-gate visit is rerouted but unverified; expect the gate plate/door sequence to run as before once Bond reaches "along the moat" on the band.
2. If the safe/spawned-key or exit legs shifted, re-anchor them the same way (all from nav nodes; the dump is reproducible with the scratch probe `crates/nf-view/examples/dw03_probe.rs` — uncommitted, delete after use).
3. Then run the WHOLE `walkreal_07000003_live` binary (7 tests), plus `walkreal_07000002_live`, `walkreal_07000004_live` and `interactables_07000003_live` whole (they exist; none was run this round — the window went into the diagnosis), list each with its pass count here, and only then the full four-crate suite (only the 3 `disc_golden` fails allowed on the slim ISO).

## Live checks for the main agent

None required — the failure reproduces in the Sim deterministically; no live address is disputed.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the session.

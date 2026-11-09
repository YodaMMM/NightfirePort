# level-0d-operative-walk-2 — the levers landed: the crouch-dodge is proven, the real-damage reel bug is fixed, the boarding/hang/ride/drop all pass; the walk stays red at the yard on upstream health, the ledger and the next lever are pushed

## The task

0700000d walked to the end on Operative (`walkreal_0700000d_live
::walk_arrival_to_the_c5_door_real_damage`, real damage, no top-ups). Branch from
`origin/s/level-0d-operative-walk`, merged current `origin/master` (already contained).
Take the -1 report's levers in order: (a) the crouch-dodge test at the corner, (b) the
drop-then-sweep. The standing-capsule change stayed in its own task. Real input only;
helpers gated to 0700000d and the difficulty; no damage or health lowered.

## What was done (all changes in `crates/nf-view/tests/walkreal_0700000d_live.rs`)

1. **The real-damage reel bug — found and fixed.** The baseline (this tree, deterministic,
   11.4 s) died at the hang assert, not the hop: "Bond hangs at the pulley", got
   `[186.06, 10.67, −294.13] grapple true` — the claw LATCHED, but Bond never left the deck
   floor. Cause: the hang loop broke on `max_ox > 177` alone, and the real-damage twin's
   E-plate park wait + the corner walk eat the beam's east crawl, so the loop started
   at/past the apex and released Bond mid-reel (the reel is `gadget::grapple_tick` phase 1 →
   2 [M 0x192860:91-94]; the -1's "tick 13800 best run alive at 15.0" was this assert with
   its stored 60-tick summary misread as alive). The break now ALSO waits for the reel's
   phase 2 (the hang). The plain master replay's reel always finishes before `max_ox`
   crosses 177, so its break tick is byte-identical — the whole binary stayed green.
2. **Lever (a) — the crouch-dodge at the corner: PROVEN.** Crouch ON at the rim corner
   (the Toggle press edge — held 5 ticks then released; the stand needs a NEW press edge
   [M the crouch arms, Player_HandleJump 0x190630 dispatch]), stay crouched through the
   wait, the latch, the hang and the ride; a fresh press stands before the hop. MEASURED:
   the baseline's rim step pulled 0.6-per-hit 0x12 fire at a 12-tick cadence (#3
   (131.6, 5.0, −301.1), hits at ticks 13824/13836); WITH the crouch: ZERO hits from tick
   12517 to the drop (~1300+ ticks). Mechanism: the crouch drops the guards' aim/sight
   target 0.45 (the eye's BL+0x90C crouch drop [M Player_GetHeadPos 0x195aa0:39-51]; the
   port's `Player::target` = camera − 0.45, scene.rs `run()`) so the bay/yard opening line
   clips the crouched chest, and the visibility factor ×0.75 [M Drone_ProcessOpponents
   0x1390c0:40-42] drops under the 0.7 LOS gate [M 0x176a28]. All real-damage-gated; the
   plain twin never crouches.
3. **Lever (b) — the drop-then-sweep: LANDED.** With (1)+(2) the boarding → the hang →
   the gantry ride (boarded at offset 2.00, the apex 138.19, the carry held) → the slot hop
   ALL PASS, and the drop landed in the yard at (137.98, 5.02, −296.08) — the walk moved
   from tick 13800 (the hang) to 15646 (the yard sweep), +1846 ticks of route. The sweep is
   now the nearest-clear-box-first dynamic loop over the yard/trench band (y < 10.5,
   x 118..=182, z −335..−263, `clear_box_aim` required, skipped when blocked): the fixed
   box order swept the far corner while #3 had WALKED to the drop point during the ride
   (the trace: d 55 at the corner → d 7.9 at the landing) and its landing bursts killed.

## The ledger (why the walk is still red)

The boarding arrives at ~15.0 health — 9 hits for 78.6 upstream, none of them the corner:
#29's two ladder-top sniper hits (14.0 @3462, 16.0 @4332, before her +222 kill), #18's
bay-enter 16.0 (@3491 + the 10812 return-fire window), #17's ride-window 16.0 (@7437 — the
Operative ground-band sniper at (157.9, 1.0, −226.1); the d2 band sweep is difficulty-gated
off), #10's deck 16.0 (@8071). The slot fall costs ~10-12 and skips armour [M 0x1902c8:84-104],
so Bond lands at ~3.2 health and dies in 64 ticks to #3's point-blank bursts — no aim
converges that fast (the sweep's first tap needs ~250 ticks). NEXT LEVER: take the four
hits off upstream — extend the ladder-top band sweep to the Operative difficulty for
#17/#10 (the -1's d2 law holds: the ladder top looks down the band line over the low
x ~160 wall), and audit the in-bay #18 kill (400 ticks have never landed a 10-health guard
at d 19.7 in-room; the leg landed false in every run including the -1's — check the aim
converge and the ray). The ignore message carries this ledger.

## How it was verified

- `walkreal_0700000d_live` whole: **1 passed** (the plain master replay, 22.3 s), 2 ignored
  (the Operative + Agent sisters). The Operative sister itself runs to the yard death at
  15646 (the ledger above); the ignore message carries the state.
- `interactables_0700000d_live`: **11 passed, 0 failed** (30.5 s, `--test-threads=1` — the
  default 2 threads OOM'd the 4 GB sandbox).
- `nf-game` whole (all test binaries): **1179 passed, 0 failed, 8 ignored**.
- `nf-formats` + `nf-audio` whole: **142 passed, 3 failed** — the 3 failures are exactly the
  slim-ISO `disc_golden` tests allowed on this ISO (`bigf_header_fields_behave_as_documented`,
  `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`).
- **The full four-crate suite does not fit the window**: `crates/nf-view/tests/` holds 1069
  test targets (792 `*_live`), and `tools/test/suite.py`'s build phase enumerates them all —
  the build alone exceeds the 2-hour sandbox on this 2-core/4 GB box (the same build-phase
  constraint the -5 and -1 reports recorded). The crates that compile the changed code are
  all green above; the diff touches no `src/` and nothing under `tests/common/`, so no other
  binary's behaviour can change.
- Every run used the real input path only (`step_bond` + `Input`); no health, armour, ammo
  or channel writes; no teleports after the load; the shared RNG untouched.

## What's left

1. The upstream health floor (the next lever above): the Operative band sweep from the
   ladder top, and the in-bay #18 kill audit. With ~20+ health at the boarding, the landed
   drop-then-sweep should close the yard, the plate, the photograph and the door legs.
2. The port-side capsule fix [M NDrone2_Collision 0x140e90:91-121] — its own task.
3. The Agent sister's d2 levers — its own ignore message.

## Live checks for the main agent

1. **The crouched-chest opening line (the lever (a) proof).** In PCSX2 on 0700000d, stand
   at the rim corner (≈185.3, 11.4, −294.8) STANDING, then CROUCHED, and let the yard 0x12
   grunts (#3 at (131.6, 5.0, −301.1) is the measured shooter) fire their lines. A gdbwatch
   on the guard's `DroneWeap_Ready2Fire` during both stands: expected — the game's fire
   stops (or its ray's wall-clip fraction jumps) once Bond crouches, matching the port's
   zero-hit window. If the game STILL hits a crouched Bond at the corner, the port's quiet
   is a port-geometry artifact (the 0.45 chest drop vs the port's opening top edge) and the
   port's opening geometry needs the game's measured clip fraction (the -1's 0.521
   strip-mouth measurement method).
2. **The late-latch reel.** The game's claw hang at the pulley #370 (178.77, 16.32, −296.39)
   when the latch happens with the gantry already at its east apex: confirm the game reels
   Bond fully up regardless of the beam's position (the port's reel has no beam-position
   dependence — the fix only waits for phase 2 — but the game's own timing pins the route's
   release window).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
this session.

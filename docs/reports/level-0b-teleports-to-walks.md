# level-0b-teleports-to-walks — replace 0700000b's walk stand-in teleports with real input

Branch `s/level-0b-teleports-to-walks` (from origin/master fd24fbf4; master did not move during
the session, so the branch is the final tree). Subject:
`crates/nf-view/tests/walkreal_0700000b_live.rs` — the seven `sim.bond.pos` writes (the
stand-in teleports) and what happened to each.

## Result

Four of the seven writes are DELETED and replaced by the game's own routes on real input; three
remain, each justified in-file (two are interaction measurement spots, one is the documented
waiting-room stand-in with its gap pinned). `walkreal_0700000b_live` runs **3 passed / 0 failed**
(warm walkthrough, fresh-spawn walkthrough, the roof-cut-scene sister test) — verified twice
(after the route edits, and again after the deck edits), plus `--nocapture` runs to read the
route outcomes.

## Per-write detail

1. **The lift-ride fallback (was line ~221)** — DELETED. The paternoster cab ride (script
   06000048's cab entity 0x2000367, y 11.0 → 48.44 [D nis dump]) is the game's route and the
   walkthrough's boarding hop + carry + step-off are real input already
   [M 0x194188 → Control_InheritVelocity 0x133e88]. The fallback teleport is replaced by a hard
   `assert!(rode, ...)`. Both walks board and step off deterministically ("the cab ride:
   boarded, bond y now 48.60" in every run; the fixed 60 Hz + bit-exact RNG keep it green).
2. **The desk approach (was line ~315)** — DELETED. The game's route into the exec office is the
   keypad: the ring's west wing ends at n22 (−21.14, 15.75), ~1.0 m from the keypad Lock #178
   "Lock - KeypadLocked" (−22.17, 48.12, 15.44 [D]); the Laser watch (weapon 0x4E, owned by the
   boot loadout — the switch runs through the game's own switch request) burns it:
   `Lock_Update` 0x1ae3d8 case 0 walks the bullet list, class-5 bullets with weapon 0x4E take −3
   per hit (0x4F −10), and `Lock_Unlock` writes `switch_channels[47] = 1` [M 0x1ae3d8 case 0 +
   :194-195]; door #151 "door_waitingroom_exec_office" (ch_lock 47 [D #151]) unlocks (locked =
   the channel OFF [M 0x136810:10-16]), the walked E opens it, and the walked legs n25 → n26 →
   desk complete the approach. The arrival is hard-asserted ("the desk approach arrived WALKED at
   (−19.05, 48.56, 30.93)" in every run). This also explains the old "n25 doorway wedge": the
   route was missing the burn, so the walker faced a locked door.
3. **The E-plate #166 return (was line ~391)** — DELETED. The return is now walked and
   position-dependent: from the desk (the warm walk) through the burned doorway
   (n26 → n25 → n22 → n21 → n20 → n16 — the n24 side-loop is skipped: the direct n25→n24 line
   crosses the wall corner at x ≈ −19.6 [D the run dump: cel 7 blocks at 27 %]), from the fresh
   walk's terminal position the proven n14 → n15 → n16 lane. The plate's use is the same real E.
4. **The deck→parachute transit (was line ~459)** — DELETED. The deck east from the cut-scene
   re-place point [−14.71, 58.00, −2.06] WALKS on this tree: "the deck below the parachute:
   arrived at +471 ticks pos (2.05, 58.07, 35.83)" in both walks; the arrival is a hard assert
   (the walk is unconditional, non-lenient). The prior session's wedge does not reproduce; the
   60 Hz determinism means the leg is stable on this tree.
5. **The files-plate stances (line ~365)** — KEPT (test-setup placement, not a route transit).
   The walked approach to the spawned use-plate already happened (`try_walk_to` above it); the
   stances enumerate interaction positions around the plate for the E hold (its box sits at
   console height — the y clamp keeps Bond in the touch band). The subject is the use-plate's
   channel law [M Script_CreateEntity 0x1d8968 case 0xEA → Trigger_TouchOnce], not movement.
6. **The waiting-room exit transit (line ~246)** — KEPT, the one movement stand-in. The game's
   route (lift landing → waiting room door #3 (13.71, 46.78, 19.44) → the east room → door #0
   (13.82, 46.78, 7.09) → the ring's east corridor) has real doors with no locks, and the E
   fires (state 0x2 → 0x4), but the walker cannot pass the open doorway: door #3 wedges the
   walker 2.5–4.3 m short from every in-room start and door #0's doorway still blocks at t 0.41
   (the probe's map, `probe_0b_execfloor_nav`). The doors are solid whatever their state — the
   movement-collision path is STATE-BLIND and passability comes from the open leaf's pose
   swinging clear [M `Door_Interp` 0x135738 writes obj+0x90 per tick;
   `control_movement_object_handler` 0x1336f8:147-236 relinks by position alone;
   `Collide_PickObj` 0x1eae80 has no state gate] — so the block means the port's open-leaf pose
   (or the doorway's static cels) differs from the disc's. Sister assert: the walkthrough's own
   walked legs before and after the transit (the exec-floor arrival, the ring to the terminal)
   pin everything around it; the transit prints when it fires. Live check below.
7. **The parachute stances (line ~534)** — KEPT (test-setup placements, not a route transit),
   now behind a WALKED attempt: the deck walk arrives (item 4), then `try_walk_to` under the
   pickup walks to the helicopter's shadow and the real forward drive takes the walk-over; only
   if that fails does the stance sweep run (it did in both runs: the helicopter body's volume
   blocks the walk 2.4 m short — the run dump cel 189, segment at 38 %; the level doc's "floor
   57.92 vs the deck 57.03 at z 42-44" — the pickup hangs over the helicopter's own deck). The
   sister assert `taken || ch 0x3F` pins the pickup [D pickup param 3; M Pickup_Handler
   0x18e650].

## The port gap the test exposed (no code change made)

None of the remaining stand-ins needed an [M] code fix inside this window: the keypad route was
data the test was missing (the burn), and the two kept transits hinge on disc-side poses that
only a live check can settle (see below). No `nf-game` source changed in this task.

## Live checks for the main agent

1. **The waiting-room exit's open-leaf passage** — fresh-boot PCSX2 (no save-state resume), Bond
   at the exec-floor lift landing (8.0, 46.8, 24.8). Walk east into the waiting room, E door #3
   (13.71, 46.78, 19.44), walk through south, E door #0 (13.82, 46.78, 7.09), walk west to the
   ring corridor (10.21, 47.18, 8.20). Record: each door's open `pos01` end value and the
   leaf's world pose when fully open, and whether the passage passes without sticking.
   Expected: the disc's open leaf clears the doorway (the walkthrough cannot pass a door that
   blocks). If it passes on the disc, the port's swing-end pose (or the doorway's static cel
   set) is the gap to fix; the port's swing law allows both 0.0 and 1.0 as open ends [M
   0x135738:33-61 clamps], so the disc's end value for THESE doors settles which pose the leaf
   must hold.
2. **The parachute touch band** — same fresh boot, Bond on the helipad deck at the re-place key
   [−14.71, 58.00, −2.06]. Walk east to the parachute at (2.53, 58.45, 42.34) [D #280] and
   record whether the disc lets Bond step UP onto the helicopter's skid/deck volume (floor
   57.92 vs the deck 57.03 at z 42-44) and take the pickup by walking. Expected: the disc's
   step-up succeeds (the pickup must be reachable by walking per the walkthrough). If it does,
   the port's walker needs the step-up (or the copter volume's true shape) — the port blocks
   2.4 m short at the body.

## Verification

- `NF_ISO=… target/release/deps/walkreal_0700000b_live-… --test-threads=1` →
  **3 passed; 0 failed** (twice, after each edit round).
- Full four-crate suite via `tools/test/suite.py` on the final tree — total line quoted below.
- What I ran: the touched binary (above), `cargo test --offline --release -j4 --no-run -p
  nf-view --test walkreal_0700000b_live` for each edit, and the full suite (below). No
  `nf-game`/`nf-formats`/`nf-audio` source changed; their coverage is the suite's.

## Suite total

The full suite (`tools/test/suite.py`, this branch = the final tree; master did not move) was
started in the session's first minutes and re-started after the ALSA/disk environment repairs;
its RUN phase did not fit the remaining window and was stopped at the deadline:

- **286 of 971 test binaries completed: 958 passed, 3 failed, 0 TIMEOUT, 0 CRASH.**
- The 3 failures are ONE environmental cause, not code: `probe_pylon_render` (1 test) and
  `fix_0c_pylon_live` (2 tests) panic on `adapter: NotFound { ... no_adapter_backends:
  Backends(0x0) }` — wgpu finds no GPU adapter in this headless sandbox (the render work is the
  main agent's). This task changed only `walkreal_0700000b_live.rs` (test file, no lib code),
  so these binaries are untouched by it.
- The touched binaries, run explicitly on the final tree:
  `walkreal_0700000b_live` **3 passed / 0 failed** (twice),
  `interactables_0700000b_live` **9 passed / 0 failed** (the sibling 0b binary, run as a
  regression check).

## What's left / open items

- The waiting-room exit transit (item 6) — settles with live check 1.
- The parachute stance sweep (item 7) — settles with live check 2.
- Pre-existing: the disc writer of ch 0x54/0x06 (objective 9's OR input on the warm walk) is
  still the level's open channel chain (the fresh walk's REAL writer path covers it); the ch
  0x75 touch after the cut-scene re-place does not fire in the walkthrough window (observation
  printed, the focused facts pinned separately).
- The boot-loadout Q-Worm grant (`owned[0x58]`) remains as documented (the disc load=1 owns it;
  the port's boot misses it — the gadget-carry gap).

## Usage

not visible to me

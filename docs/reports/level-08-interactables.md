# level-08-interactables — report

Branch `s/level-08-interactables` (master 8ce724ad). Task: census every player-touchable placement on
07000008, write `crates/nf-view/tests/interactables_07000008_live.rs` with one live test per KIND, fix
every port gap the tests expose.

## What's done

- **The census** was already in `docs/levels/07000008.md` §7 (per-kind table); this task verified it
  against a fresh disc parse (52 maps, 308 main-map placements; the tool `census_07000008.rs --ignored`,
  env gate `NF_ISO`) and corrected two readings (below).
- **`crates/nf-view/tests/interactables_07000008_live.rs`** — 15 tests, one per player-touchable kind
  (doors, fusebox 0x31, locks 0x2e + SS 0xe2 + mux 0xec, destroy 0xe1, LoadPointScript 0xeb + the
  boarding mux, TouchOnce 0xea, spawners 0xf1, sound trigger 0xf9 via the music plates, ladders 0x22,
  wire/zip 0x3e + the 0xfa context icon, pickups 0xf0 + the kind-6 medal gate, glass 0x20, hints 0x33,
  copter 0x2f, hurt 0xe4).

**Touched-binary totals** (`cargo test --offline --release -p nf-view --test interactables_07000008_live
-- --test-threads=1`, final tree): **9 passed; 4 failed; 2 ignored** (each ignore carries a named
blocker in its attribute). The green set: free-door E-open + the #122 locked-refusal sister; the
fusebox laser cut → ch 137 → SP #221 + music #277 + the box #234 smash + the #122 E-open; the safe
bolts' burn → 134/135 → mux #278 → ch 136 → SS #205's sweep; the cable-car floor plate → ch 4 with the
mux #235 AND-sister; the intro plate #237 walked from the real spawn → ch 0x49; the ladder grab/climb/
dismount; the zip line jump-grab (E-refusal sister) + the ride to the low end; the walk-over pickups
with the medal gate both ways; the hint plate #241's INFO message; the copter gated hover.

## Findings (evidence-carrying)

1. **The doc census's "#234's death writes ch 137" is wrong per the decompile.** `props.rs`'s decode
   [M 0x12c318:36, 0x12c4f8:30-35] makes param 0 the death-write channel (tag+0x12) and param 1 the
   external-KILL channel; #234 has no param 0 and param 1 = 137 — ch 137 (the fusebox's write) SMASHES
   the box, not the other way round; its 10000 pool (param 3) makes it unshootable by design. The test
   pins the smash on the ch-137 rising edge. The census row's "the alternate GuardRoom unlock" reading
   should be corrected in docs/levels/07000008.md.
2. **Map-origin placements resolve through object bounds.** The disc translations of the extinguisher
   #136, the monitors #211/#213, the plunge/music-volume triggers are (0,0,0)-ish; the live boxes
   (`Destroy.lo/hi`, `Break.lo/hi`, `Trigger.lo/hi`) carry the world pose. Tests target box centres.
3. **The extinguisher #136's mask 0xf8 drops handheld rounds** [D param 4 → ~0xf8; M 0x12c318:47-52,
   Collide_FilterBullets 0x1ece30] — it is decor, not the shootable arm; the LCD monitors #211/#213
   carry no mask param and are the pool arm's targets.
4. **The spawner screening law is observable on 08**: after ch 44 arms the group spawners
   [M 0x137e68's trigger-consume block], the wave spawns only from spots ≥ min_dist (10) and unseen
   [M 0x137cb0:21-43] — the test walks Bond back out of the stairwell first.

## What's left (the next message continues from the branch)

The 4 red tests, each with its observed symptom:
1. `music_plates_mux_into_the_map_sound` — plate #100 fires ch 90 [D] but mux #102's OR-fold
   (out 0x68 = 90|91|92 [D]) never rises within 120 ticks. **Possible PORT GAP**: the walkreal suite
   only ever rode the AND-muxes (#235/#276/#278); the OR-mux arm of `triggers.rs`'s 0x1b2c68 loops is
   unpinned elsewhere. First check: does the port's mux read param 0 as out and the list as OR-terms?
2. `touchonce_plates_fire_on_the_walk_in` (the #80 leg) — the walk-in from outside the plate's own
   AABB did not fire ch 43 (the intro-plate leg is green from the real spawn). Check the #80 volume's
   tris vs the walk line (the nav routed around the AABB?).
3. `shooting_a_destroyable_kills_it_and_spawns_the_blast` — the P2K on monitor #211's box centre did
   not drain the pool in 600 ticks; check the guard-room LOS from (centre − 3x) and the bullet→destroy
   contact pass (drone.rs's destroy walk).
4. `pickups_take_on_the_walk_over` — the floor-resolved armour/9mm take; the first fail point is the
   armour pick (line ~560). The disc's parked translations may need the object-bounds treatment like
   the monitors.
- The kind-6 warm walk-over is conditional on the bonus spot's floor (the test prints when it skips).

## Verification

- Every assertion cites [M] a decompile address, [D] disc bytes/placement params, or [L] a snapshot, in
  the assert message. Teleports are stand-in transits, each naming its documented blocker/pinned route;
  every operation (E, walk-in, trigger pull, walk-over, jump grab) is real input.
- The full suite (`tools/test/suite.py`, started in the background at 16:42 on the final tree) was
  killed by the sandbox expiry mid-run: `build ok: 938 test binaries in 8 s`, then
  `CRASH net_e2e_loopback_live (exit -9)` (environment kill) — the run did not reach its total. The
  touched binary's totals are quoted above; no source files outside the new test file were changed, so
  the other binaries' risk is nil (the new file is additive).

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment; the only
counters available are per-subagent token totals, which are not this session's figures).

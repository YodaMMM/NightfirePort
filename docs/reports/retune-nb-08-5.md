# retune-nb-08-5 — 08 whole-mission 00-Agent real-damage walk, round 5

Branch `s/retune-nb-08-5`, from origin/master. `origin/s/retune-nb-08-4` does not exist on
the remote; master already carries the round-4 shipped tree (its ignore-message ledger), so
the round branched from master directly and no merge was needed.

## What was done

1. **The run-11 "crouch latch" mystery is SOLVED from the decompile [M].** The latch is the
   stand/crouch LOCKOUT: `Player_ChangeSubState` 0x190630:48 writes `BL+0x960 = FRAME_RATE`
   (60) on EVERY substate change; `Player_SSCrouch` 0x1924c0:45-53 gates the stand press on
   `+0x960 == 0` and :84-105 reads the crouch action; the port mirrors it (player.rs
   crouch-on sets `lockout = 60`, the stand branch requires `lockout == 0` plus the headroom
   LOS pos→pos+0.95). Run 11's 4-tick release could never outlive the 60-tick lockout armed
   by the drop's crouch press.
2. **Lever (stand-up after the crouch drop): implemented and measured — TRIED and DEAD.**
   Waiting the lockout out on the lower floor and then holding crouch DOES stand Bond up
   (`crouched false lockout 59` in the log), but he stands inside the hole's footprint and
   the expanding standing capsule re-bridges the shaft (feet 13.47, eye 14.18; master's fall
   lands 13.106 — verified by a byte-exact master control run through the shared prefix).
   The fight's first launch then self-blasts −14.00 and the walk dies t20944 (master t21070).
   The plate variant (crouch-walk off the hole first) measured the same pop (t20942). Both
   reverted.
3. **The pickup scan the task ordered (the fallback path) — the round's key discovery [L].**
   At fight entry: armour 0.0, health 14.0, rocket clip 4 carried 0 — four rockets = 404
   damage < the machine's 598 hp (DiffMod·299 [M Copter_Create 0x17d340; L the dmg 0.0/598
   dump]) — the fight is UNWINNABLE on the carried ammo. The cable car's three rocket
   pickups (a launcher and two ammo boxes, 4 rounds each [D 0x0600004C streams 37-39]) sit
   UN-GONE 1.2-2.4 u from Bond: with them 12 rockets = 1212 damage wins. The take is the
   game's own sphere check [M Pickup::touches: root distance < bounds.radius + 1.1 then the
   root→centre LOS ray, 0x18e148:210-214].
4. **Collection walks: TRIED and DEAD on their own real runs.** Both routes (post-drop
   crouched legs; pre-anchor standing legs) climb the parked carriage's exterior (feet
   13.71-14.14) into the terminal's hurt volume and the −2.00 zaps close the 14.0-hp walk
   earlier (t21016, t21286). The car's interior is not reachable by walk_to's straight steer
   from the landed spot or the lip.
5. **Shipped tree:** master's shipped run-10 walk (the best measured state, death t21070,
   55 hits) + the fight-entry pickup scan (eprintln only — the timeline verified byte-exact:
   the scan run reproduces master's t21070/55 hits and the same [08 anchor] positions) + the
   refreshed round-5 ledger in the ignore message.

## How it was verified

- Each lever judged on its own real run of the d3 00-Agent walk (15 s per run): t20944,
  t20942, t21286, t21016 — all worse than master's t21070, so all reverted.
- A byte-exact master control run (checkout origin/master's file) confirmed the shared
  prefix and master's fall/landing (13.106) before the stand-pop diagnosis.
- Final tree: the d3 walk reproduces master's ledger exactly (t21070, 55 hits, the same
  [08 anchor] positions) with the scan active.
- `walkreal_07000008_live` whole binary: **6 passed / 0 failed / 1 ignored** in 150.04 s
  (master's measured 6/0/1, 153.6 s).
- Only `crates/nf-view/tests/walkreal_07000008_live.rs` and `docs/` changed — no
  `crates/*/src/`, no `tests/common/` files, so no other binary's timeline can move.
- The full suite runner (tools/test/suite.py) did NOT fit the window: setup (full sandbox
  reset: clone, toolchain, ISO) consumed the first ~35 minutes and the lever runs the rest;
  the whole walkreal_07000008_live binary above is what ran. The next window should run it
  on the final tree first.

## What's left (the next window's levers, in priority order)

1. **Collect the three rocket pickups DURING the ride** (the ch1→ch53 loop): they are
   parented to the carriage [D 0x0600004C streams 37-39] and ride with Bond, so small real
   walks inside the car while it descends (constant relative offset, no world-space aim)
   put his root inside the take reach. No exposure. Then the fight has 12 rockets.
2. The stand press is proven mechanically; with a stand spot OFF the hole (e.g. the plate's
   east edge (112.3,−179.15)) the fight can run standing on the lower floor (eye 13.77).
3. Sweep the idle wait before the y-46 trio / #16 fights (each wait shifts the guards'
   roll): the upstream bleed (84.0 before the fight) leaves 14.0 hp at fight entry — the
   wall even with the ammo.
4. Live check for the main agent: none required this round — the lockout question is
   answered from the decompile [M].

## Usage

not visible to me

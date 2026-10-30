# capsule-cull-and-los-masks-10 — two of the four last reds closed from the game's behaviour; the other two carry full evidence chains

Branch `s/capsule-cull-and-los-masks-10` (from `origin/s/capsule-cull-and-los-masks-9`
e6fb14dd, `origin/master` 929c6606 merged clean). The task: close the four remaining -8×master
reds one by one from the game's behaviour [M], re-deriving a test only where it leaned on the
old non-game rule.

## Closed this window (both are harness re-derives; no product code changed)

1. **fix_npc_doors `the_escort_opens_the_interior_doors_on_his_route` — GREEN.** The escort
   stalled forever in PLAYER_IN_WAY (0x1e) at (−4.1, 6.3, 14.2) with Bond parked at 1.27.
   That is the game's own machine working: the stay band is the 1.0472 rad (60°) facing cone
   AND `Dist2D < 3.5` [M `NDrone2_DSTATE_AllyLeadPlayerInWay` 0x15ea28 case 3 — `fVar7 < 3.5`
   returns 1 (stay)], and the lead runs WEST into a parked follower, so both wait. The old
   harness closed to 4.0 and stood inside the band; the real player steps back. The harness
   now (a) backs Bond off inside 3.6 during the follow (real reverse input) and (b) on a
   PLAYER_IN_WAY stall walks him out of the band along the Bond→contact line reversed (real
   walk input), then lets the lead resume [M 0x15ea28: the band exit re-enters Lead]. The
   masks mover needs no port change — the 0xa27 sight drops changed the escort's hide rhythm
   upstream, which is what first parked the pair inside the band; the game's contract is the
   player clearing the cone. Run: 1 passed / 0 failed (97.7 s).
2. **fix_searchlight_remainder `a_courtyard_head_tracks_bond_until_the_lost_limit` — GREEN
   (the binary 3/0).** The old harness chased the spot's landing by teleport — two non-game
   assumptions stacked: (a) the landing is often ON BOND'S SPHERE once tracking (the game's
   track ray hits the player sphere — the spot pos = the sphere hit + n·0.01 [M 0x1cb900 hit
   branch]), so teleporting his feet onto it lifts him ~1.2; (b) the sweep's courtyard
   landings at y 41.01 (x 1.4..8.2, z −17.2) are a surface the capsule falls through from
   above (the game's head-on cull [M Intersect_CylGeom 0x109da0:302-322] is exactly what
   makes it non-standing; the old un-culled push held him there). The fall cycle pre-loaded
   the game's lost counter [M 0x1cb900:251-263 — dist ≤ 2.0 resets, else +1, FRAME_RATE·15
   gives up] to 44–96 ticks, so the head returned at 804–815 instead of 900. The re-derive:
   Bond starts where the game's player stands — the courtyard floor at the housing's base
   (housing (−1.12, 47.57, −17.33); the probe-measured settle: the capsule holds y 42.07
   through 240 idle ticks, the sweep's spot lands on his sphere (min dist 0.95) and the head
   ARMS from stillness) — then no input at all through the arm/track/lost phases. The
   ride/track/lost asserts are untouched in meaning; the tracking assert's text now says
   "stands in the beam".

Also landed (harness, keeping the walk honest): the shared walker's stuck-mitigation jumps
are guarded with a 2.2 m headroom check — under the 01 stairwell's soffit the jump launched
Bond's head into the ceiling and the fall re-pinned him on the same tread edge every cycle
(the real player doesn't jump with the ceiling ~1.4 over his feet). The follow test's plate
leg now walks the main test's proven hall-corner + stairwell-head shape.

## Still red (each with the evidence chain and the next step)

3. **walkreal_07000001 `following_mayhew_down_completes_the_level` (cull)** — the plate
   descent jams at (−33.44, 4.72, 12.03): the capsule stands grounded on the z-11 tread's
   edge (the scoped floor 0.567 below — the tread 4.14) with its head IN the z-12 soffit
   (6.1): pinned between the tread edge below and the ceiling above [D probe: the flight
   descends 4.7 → 0.7 over z 10..16, treads ny 1.00, ceilings 6.1..7.4; a capsule teleported
   to the spot is pinned — forward input moves it zero in 240 ticks]. The game's player
   never glides there: the walk's step-down tolerance (the scoped-floor snap, 0.57) lets the
   capsule hover at the old tread height across the 0.6/0.8 risers until the soffit catches
   it — the old un-culled corner push used to knock it down the flight. Next step (next
   window): measure the game's walk-down snap (is the game's step-down tolerance larger than
   the port's step-UP 0.5? — the live check below), then either port the game's walk-down
   snap or re-derive the leg. The OTHER three tests in the binary are green (the sister
   descent through the same stairwell works via the hall-corner anchor).
4. **mp_perf_16 `sixteen_participants_fixed_run_digest_is_pinned` (both)** — digest
   0xfd4fcbf81586b3f1 vs pin 9929307665006445228. The re-pin method stands (capsule-9's):
   dump both per-tick `net_state_hash` streams (a master build + this tree) and name the
   first diverging tick's mover — this window spent its budget on the two closable reds and
   the disk-full recovery; the two mp_perf_16 binaries (master-era and branch) are both in
   `target/release/deps` and the master one passes its pin, so the divergence is purely the
   -8 port's game-true cull/masks behaviour reaching the MP sim.

## How it was verified (what ran)

- `fix_npc_doors_live`: 1 passed / 0 failed (97.7 s).
- `fix_searchlight_remainder_live`: 3 passed / 0 failed (4.1 s).
- `walkreal_07000001_live`: 3 passed / 1 failed (the follow red, above; 89.5 s).
- `mp_perf_16_live`: 1 passed / 1 failed (the digest red, above; early_out green).
- The full fast suite on the final tree: see the total line below.

## Live checks for the main agent

1. **07000005/06 courtyard spawn** (capsule-9's check 1, still open): fresh boot, read Bond's
   standing y at the level start and confirm the courtyard band (x 1.4..8.2, z −17.2, the
   y=41.01 landings) is NOT standable for the player capsule (he falls through into the
   under-croft) while the housing's base floor (y 42.07) is. Expected: the game's spot lands
   on the player's sphere from the housing-base floor, as the port now runs it.
2. **07000001 stairwell descent**: fresh boot, walk Bond down the stairwell from the hall
   (the flight at x −33.5..−32.3, z 10..16) and watch the capsule's feet on the treads
   (HITTEST+0x20/+0x30 A/B, the grounded flag, feet y vs the treads 4.7/4.1/3.5/2.7): does
   the live game's walk snap him DOWN each riser (glued), or does he fall/step per tread? If
   glued, the port's walk-down snap (0.57 scoped tolerance) is the port bug to fix and the
   follow red closes with it.

## Usage

not visible to me — this environment exposes no token, cost or context-window figures for
this session (no usage banner, no /cost, no token readout anywhere in the interface this
worker sees).

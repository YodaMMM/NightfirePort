# level-0a-low-beam-standing-toucher — the report

Task: make the 0a low laser beam #621 trip when Bond walks through it standing [L PCSX2 2026-11-02, fresh goto_level 0700000a: Bond standing, walked z −31.0→−33.78 across #621's line, ch 0x24 0→1], by finding and porting the game's player-vs-trigger toucher shape [M Collide_PickObj 0x1eae80 and its HITTEST type word, the player's contact list build], un-ignoring `walkreal_0700000a_live::low_beam_621_fires_standing`, and re-checking every other trigger test against the same law.

## What was found — the game's shape, pinned [M]

The player's trigger toucher IS his collide-pass HITTEST, end to end:

1. **The HITTEST lives on the player object**: obj+0xD4 = BLData+0x740 [M Player_Init 0x1910f8:40].
2. **The stamp**: `Player_Collision` 0x194df0's second switch (indexed substate−1, jump table @0x2fc0d0) writes, for the walk (the DEFAULT arm, substate 0 falls through): A = pos + up·0.275 → BLData+0x760, B = pos + up·(0.55 − cc) → BLData+0x770, radius 0.55 → BLData+0x7cc, and the **mode word 0x800 → BLData+0x7d4 in every arm** [M 0x194df0:315-333 + the shared tail :405-408]. (The earlier revert notes' "pos+0.55 / pos+0.275−cc r 0.275" are the case 0/6 climb/creep arm [M :202-222] — the wrong arm.)
3. **The pick**: `Collide_PickObj` 0x1eae80:96-119 — type 0x800 = the closest point on the HITTEST's A→B segment (clamped [M vecutil 0x1df7e0: t ∈ [0,1]]) vs the candidate object's own sphere (centre obj+0x80, radius obj+0x8c): hit iff dist < obj_r·1.5 + *(h+0x8c).
4. **The narrow phase**: `Collide_Intersect` 0x1ebaa8:226-247 dispatches type 0x800 to `Intersect_CylGeom` 0x109da0 against the trigger's OWN glist triangles — obj+0xD8 = the placement's celglist [M Trigger_Create 0x1b1720:92], so the port's parsed trigger tris ARE the game's collision glist — with the strict `dist < radius` refine at 0x109da0:447; a miss writes the 1e8 sentinel that the distance sort drops [M Collide_Update 0x1ea398:118-158].
5. **The contact list**: `Collide_Update` 0x1ea398:160-215 mirrors every contact into BOTH sides' obj+0xD0 lists.
6. **The credit**: `Trigger_Hit` 0x1b1fc8:27-47 walks the trigger's obj+0xD0 list and builds the class mask (class 3 → bit 8 = mask-8, 2 → 0x10, 5 → 2, 0x4a → 0x10); the player IS class 3 — **[L ee_start_0700000a.bin: glb_players[0] @0x2d88e0 → 0x1b78640, +0xff = 3]**.

So mask-8 fires off that segment vs the trigger's parsed triangles. The segment bottoms at B−r = pos−cc = floor+0.05 standing (cc 1.0347 [L]), inside #621's band y 0.13-0.39 [D] — the game's standing walk fires it; the port's box stand-in (bottom pos−0.6 = 0.53 on 0a's floor) misses it. `SoundTrigger_Update` 0x1b2790 has no geometry (it plays off a gate channel), `Trigger_Touch` 0x1b2aa8 forwards to the same `Trigger_Create` — no per-kind geometry law.

## What was wired and what it broke — measured, then reverted

Wiring `player.body` (the port already builds the case-correct arm at scene.rs's Player site) as the standing toucher made **`low_beam_621_fires_standing` GREEN** (and `laser_band_rule_crouch_clears_607_and_low_beams_fire_either_way` stayed green — the crouch-under cases and the #614 jump/crouch sisters). The same wiring broke three shipped measured pins, so it was REVERTED to the box with the full chain documented at the toucher site (drone.rs `player_touchers`):

1. **fix_truck_live 0/2**: the standing ride fired the ch-0x70 park plate (the segment's reach dips ~0.16 below B to the plate at pos−0.573/−0.648). The game's standing-RIDE contact is unmeasured — the 05-truck trace played crouched only — and the ride's own class-5 truck contact is not ported (trigger-port-check #11).
2. **interactables_07000011_live** `sound_triggers_created_and_played_per_the_disc_gate`: the Bond-Moment plate #235's glist rides ~0.85 over pos — above even the segment's pos+0.825 top — the game fires it through the glist ref-gate/extent the port has not modelled (the same divergence class as the 0c ch-27 bay volume documented on `Toucher::player`).
3. **walkreal_0700000a_live** `walk_office_floor_qworm_keyboard_hatch` (+ both real-damage sisters, which died to the laser): the office-ring pin "no laser/alert channel ever fired" — the port's walker path brushes low bands the game's footpath never did. Route-side waypoint divergence, not a shape error.

## State

- The box stand-in is unchanged in behavior; the evidence chain and the three levers are documented at `drone.rs` `player_touchers` and in the test's ignore message.
- `low_beam_621_fires_standing` stays `#[ignore]` with the UPDATED message: the shape is pinned, the wiring went green here, the next levers are (1) the 0a office-ring route-side tightening, (2) the glist ref-gate/extent (011 plate #235, the 0c ch-27 volume), (3) the ride's class-5 truck contact path.
- WORKLOG entry appended (branch cut from origin/master 68298698; WORKLOG restored from s/real-health-00agent-08-09-0a-0b's copy since master strips process files).

## Binaries run (each whole, --test-threads=1, release, NF_ISO set)

- `walkreal_0700000a_live`: with the segment wired — 9 passed / 3 failed / 2 ignored; after the revert — **11 passed / 0 failed / 3 ignored** (the target test among the ignored).
- `fix_truck_live`: with the segment — 0/2; after the revert — **2 passed / 0 failed**.
- `interactables_07000011_live`: with the segment — 15/1; after the revert — **16 passed / 0 failed**.
- The full suite (`cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p nf-view`) did **not** fit this session's window (the toucher research + the two wiring/verify cycles consumed it; the sandbox expired at ~23:59 UTC). The three binaries above are the ones the change site can affect; the reverted tree is byte-identical to master's behavior except comments + the two doc/ignore-message updates.

## Live checks for the main agent

1. **The standing truck ride (05)**: fresh goto_level 0700000b→05 truck ride, ride STANDING (no crouch) through the park: does ch 0x70 ever pulse? If it DOES fire standing in the game, the box's ride silence is the wrong pin and the segment can land with the ride test updated.
2. **The 011 Bond-Moment plate #235**: watch the trigger's glist collision tri set actually loaded at obj+0xD8 (data+8) vs the port's parse — its vertical extent decides the ref-gate/extent question (same for the 0c ch-27 bay volume at the bay-floor rest).
3. **The 0a office ring**: Bond's walked footpath xz through the office ring vs the port's walker waypoints — which low line the game's path passes, at what clearance.

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment; the session's own boundary was the ~23:59 UTC sandbox expiry).

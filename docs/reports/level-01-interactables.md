# level-01-interactables — every interactable kind on 07000001 works like the game

Branch `s/level-01-interactables` (from master 61c7e3ea). Task: census every player-touchable
placement on 07000001, read what the existing `interact_`/`walk` tests already cover, add the
missing kinds as `crates/nf-view/tests/interactables_07000001_live.rs` (model:
`interactables_0700000b_live.rs`), fix every port gap the tests expose.

## Done

- **The kind census** (docs/levels/07000001.md + `nf-extract placements-all 07000001.bin` [D]):
  the existing tests already pin doors (0xDB), simple scripts (0xE2), breakables (0x20), the
  armour pickup (0xF0 kind 3), the shooter trap (0xD2), the muxes (0xEC/0xEE), TouchOnce plates
  (0xEA), rotors (0x71), the monitor (0x30), the touch/exit route, and the static kinds. The
  kinds nothing covered: **rigid bodies (0xD8)**, **destroyables (0xE1)**, **the hint marker
  (0x33)**, **the ammo and Bond-bonus pickups (0xF0 kinds 1/6)**, **the music trigger (0xFB)**,
  and the never-created kinds (**0x31 fuseboxes**, **0x29 switches — zero placements**).
- **`crates/nf-view/tests/interactables_07000001_live.rs` — 6 green + 1 ignored** (one test per
  missing kind, each driven by real input — the walk, the aim, the trigger pull, the walk-over —
  each with a sister):
  - `the_redlamp_rigid_body_takes_the_bullets_impulse` — the four `rb_redlamp` bodies (type
    0xFFFF, mass 1 [D params]) take the bullet's class-5 contact: the shot tips the lamp off
    its identity rotation while the translation stays [M RB_RestingContactRot 0x1d73a0's
    class-5 arm; RB_EulerEvolve 0x1d7810; the 0x1d7124 type-0 gate]. Sister: the unshot lamp
    rests forever. Note: the body's world sphere is the glist's r≈0.33 ball at the mount [M
    Control_BuildWorldSph 0x133070; D the object radius] — a dead-centre hit is parallel and
    tips nothing, so the test aims a fifth-unit off-axis (ρ≈0.2 → torque).
  - `shooting_the_comms_room_screen_smashes_it` — Destroy #107 (health 0, kind 1, p0 = 0
    [D params]) dies on the first bullet [M Destroy_CollisionHandler 0x12bd48:31-49 →
    Destroy_Smash 0x12bb60]. Sister: a fresh load stays whole.
  - `the_hint_marker_registers_but_the_disc_parks_it_in_the_void` — the level's one 0x33
    marker carries the disc's label 0x02000057 [D #330 param 0] but its placed collision
    volume spans y −0.7..0.7 at (11.02, 0, −3.31) with no world triangle within 3 and no cel
    link ("cel false") [D coll-scan] — the game's collide pass admits contacts only from
    straddled cels' chains [M Collide_Pick 0x1eb320:125-134], so `Hint_Update` 0x1b3380 can
    never see a class-3 contact. The port agrees (the marker's straddle set is empty); the
    test walks the comms room above the volume and pins the never-fire.
  - `the_ammo_box_credits_the_mp5_pool_on_the_walk_over` — the id-6 ammo pair [D #324/#325]
    credits weapon_data[2]'s pool +20 on the walked take [M Pickup_Handler 0x18e650 case 1 →
    Player_EquipAmmo 0x1a8d18; the 6→2 remap]. Sister: the untouched load's pool waits.
  - `the_bond_bonus_needs_the_gold_medal` — the kind-6 rows exist only under the gold-medal
    word for BaseMap 0x0700001 [M 0x18dda0:93-95; L ee_start_07000001.bin 0x30cba0], both at
    the disc spots [D #315/#316]. Sister: the bare boot creates none.
  - `the_music_trigger_row_and_the_never_created_kinds_match_the_disc` — the ch-13 music
    trigger row (event 2) [D #313]; the 12 flags-0x2031 fusebox placements never create (the
    parse's `flags & 0xa000` gate [M parsemap_block_map_data_dynamic 0x1d1b68]) and the 0x29
    switch kind has zero placements [D census].
- **The port gap the tests exposed — fixed [M class code]:** `Props::smash` wrote
  `switch_channels[tag+0x12]` unconditionally; the game's channel-0 arm only flips obj+0xff
  and reaches the channel write in the else branch [M 0x1bb60:34-37 vs :40]. The tv2's smash
  (p0 = 0) was raising switch channel 0 — the monitor's own channel — through a bullet. Fixed
  in `crates/nf-game/src/props.rs` (`if d.channel != 0`), cited at the site.
- **The census corrections** (report-only): 0x33's marker is dead content on this level (the
  void parking above — the kind itself is proven live on 07000008's safe marker,
  fix-context-hints). 0x2E Lock, lifts and gadget targets have zero placements on 07000001
  [D census] — nothing to drive.

## How it was verified

- The new binary: `interactables_07000001_live` — `6 passed; 0 failed; 1 ignored` (the ignored
  sibling is the bonus walk-over take, blocker below), run with `--test-threads=1` against the
  slim on-foot ISO.
- The touched binaries were run before the push: `interactables_07000001_live` (above) and the
  nf-game unit suite for the props.rs change (the `Props::smash` callers' tests in nf-game —
  no nf-game test asserts the channel-0 write; the change only stops a write the decompile
  does not make).
- The full four-crate suite on the final tree (merged with origin/master — my branch started
  at master's HEAD): see the total line below.

```
(total line pasted from suite.out after the final run)
```

## What's left (the ignored test + the open walks)

- `the_bond_bonus_walk_over_takes_it` (`#[ignore]`, named blocker): the walk-over consumption
  of the kind-6 bonus [M 0x18e650:234-236]. The bonus floors sit ~1.6 below every floor the
  harness reaches: the spawn-side nav ends on the upper hall (y 7.18), the west corridor's low
  route rides the escort path this harness walks only with the contact, and a stand-in drop
  onto the bonus's own floor is ejected back up by the sim. The medal gate half (the green
  test) is fully pinned; only the take remains.
- The ammo kiosk descent: the walk-over take DID complete through the walked stage approaches
  (the pool credited +20), but the entry the take used is the walker's stage-edge approach —
  a live check of the real player path into the kiosk at (−3.25, 5.85, −9.06) would settle
  whether the walk the port found is the game's.
- The destroyable smash's kind-1 blast script (0x0600027) is queued and drained inside one
  Drones::tick — its visible fireball is the render layer (a live check, not a sim assert).

## Live checks for the main agent

1. **The RB bullet contact scale** — the port stands the bullet's HITDATA+0x8 float at 1.0
   [S, flagged at the site in drone.rs]. gdbwatch the lamp #9 object's obj+0xD0 chain during a
   shot at the mount (2.01, 7.89, 1.95): expect a class-5 contact record whose +0x8 scale the
   game wrote for a gun round (the port needs the real value).
2. **The ammo kiosk** — fieldprobe a walk into the box floor at (−3.25, 5.85, −9.06) from the
   spawn stage (Bond at y 6.77): which input reaches it in the real game (a drop through the
   stage skirt, a side lane)? The port's walker has no descent.
3. **The west-hall bonus floor** — fieldprobe Bond standing at (−13.24, 5.63, 10.35) (the
   kind-6 spot) via the escort's west route without the contact: the harness cannot reach that
   floor from either side; a real-game walk settles the player's path.

## Usage

not visible to me

# Aim: finish the 14 cut and re-quote

Branch from `origin/s/aim-node-pass-2`, merge current `origin/master`, push to `s/aim-finish`.

The branch carries src changes from the aim stack (door.rs, drone.rs, nis.rs, world.rs, scene.rs), so its binaries must be re-run on the merged tree. "Test-side only" in its report covers the last step only.

1. Rewrite the cut loop in `interactables_07000014_live` the way `fusebox_geometry_07000014_live` does it: aim from `bond.head()`, set `bond.yaw/pitch` directly, hip-fire with plain input. Keep every assertion.
2. Run and quote counts for: fusebox_geometry_07000014_live, interactables_07000014_live, rocket_stop_plane08_live, rocket_glass08 tests, rocket_origin_live, solar_panel_hits_live, walkreal 08 / 0c / 14 / 16 / 1b.
3. Check the two aim filters read `inst.placement.flags`, not the object flags, and print a measured count of filtered instances per level. A filter that drops nothing makes a pass void.

---
claimed-by: sandbox3 at 2026-11-06T07:17Z

---
## Report (done, 2026-11-06T07:48Z)

# aim-finish — the 14 cut finished; the re-quote measured the aim stack's merged-tree cost

Branch `s/aim-finish` = `origin/s/aim-node-pass-2` (cb62c58e) with origin/master merged. The
merge is a NO-OP — master (c2c4a6e) was already an ancestor of the base branch (the base's own
merge be17877ed). So the "merged tree" is the aim stack's tip, and every count below is that
tree.

## What's done

1. **The interactables_07000014 cut rewritten the fusebox_geometry way** (task item 1). The
   old loop aimed through the file's `aim()` helper and raced the laser's raise with 40 fixed
   equip ticks. The rewrite, keeping every assertion:
   - the aim runs from `bond.head()` — the shot's own ray anchor [M Player_WeaponInitBullet
     0x1a4bf0:152-159, Player_GetHeadPos through Player_GetAimingPoint] — set directly on
     `bond.yaw/pitch` (the aim-mode pad cursor saturates [−0.296,+0.4] within 60 ticks and
     never relaxes [M scene.rs step_bond] — the west-up bend finding);
   - the pitch is the **asin form** — `asin(dy/len) = atan(dy/horiz)`, the view's own slope
     convention. The old helper computed `atan(dy/len)` (the hypotenuse as the adjacent), which
     under-angles this steep downward stance shot (asin(0.5)=0.524 vs atan(0.5)=0.464 — ~0.12 u
     low at 2 u) and slid the beam off the thin wire slab;
   - the equip waits until `gun.weapon == 0x4e` (the raise-script pitch pin applies while the
     raise plays; 40 fixed ticks raced it), then **hip-fires with plain input** (`tick_fire` —
     the walker's `step_bond(input, true, …)` plus the bookkeeping the asserts read).
   Result: the cut test green in 1.76 s (was the 600-tick cap, ch 0x22 never written). The
   whole interactables binary went 9/1 → **10/0**.

2. **The re-quote on the merged tree** (task item 2), every binary whole, `--test-threads=1`,
   NF_ISO set:

   | binary | merged tree (this session) | master baseline (same session, same box) |
   |---|---|---|
   | fusebox_geometry_07000014_live | **1 passed, 0 failed** | — (the base's own green stands) |
   | interactables_07000014_live | **10 passed, 0 failed** (was 9/1) | — |
   | rocket_stop_plane08_live | **4 passed, 0 failed** | — |
   | rocket_glass08_live | **3 passed, 0 failed** | — |
   | rocket_origin_live | **1 passed, 0 failed** | — |
   | solar_panel_hits_live | **1 passed, 0 failed** | — |
   | walkreal_07000008_live | 5 passed, **1 failed** | 6 passed, 0 failed |
   | walkreal_0700000c_live | 2 passed, **1 failed**, 1 ignored | 3 passed, 0 failed, 1 ignored |
   | walkreal_07000014_live | 6 passed, **3 failed**, 1 ignored | 9 passed, 0 failed, 1 ignored [L the level-14 worker's 07:45 run] |
   | walkreal_07000016_live | 13 passed, **1 failed**, 2 ignored | 14 passed, 0 failed, 2 ignored |
   | walkreal_0700001b_live | 15 passed, **2 failed**, 2 ignored | 17 passed, 0 failed, 2 ignored |

3. **The two aim filters** (task item 3):
   - **Filter A — the level stage's `aim_pick`** (world.rs `build` → `segment_aim_scoped`):
     reads **`inst.placement.flags & 0x8000`** ✓ — the placement word, per the game's runtime-cel
     gate [M parsemap_block_map_data_static 0x1d1998:92]. Verified in code and in history: the
     filter was introduced reading the OBJECT flags (6b049b4e0), fixed to the placement word
     (fc6e21313 "the object flags never carry 0x8000, so the filter dropped all static level
     geometry"), and never touched since.
   - **Filter B — the object stage's `obj+0xF0` gate** (`Drones::aim_snap`, the script-entity
     loop and the doors loop): reads the object's glist word via `world.glist_flags(hash)` —
     `obj+0xF0 = glist+0x44` [M Control_SetGList 0x134960:11; parsemap_block_entity_params
     0x1d1508]. This one is **the object flags by the game's own design** — the runtime word the
     gate reads is the glist record's +0x44 dword, not the placement's; switching it to
     `inst.placement.flags` would contradict the [M] chain (the 0700001b audits pinned panel f0
     0x81 / pieces 0xc1 as the OBJECT words [L]).
   - **The measured counts** (`crates/nf-view/tests/aim_filter_census.rs`, run with --ignored;
     no void pass on any level):

     | level | filter A kept / dropped (of collision-bearing instances) | filter B gated objects / placements on them |
     |---|---|---|
     | 07000008 | 96 / 122 of 218 | 28 / 41 |
     | 0700000c | 170 / 169 of 339 | 48 / 77 |
     | 07000014 | 173 / 391 of 564 | 17 / 162 |
     | 07000016 | 103 / 116 of 219 | 10 / 27 |
     | 0700001b | 8 / 162 of 170 | 14 / 15 |

4. **Lesson 9 found live and fixed**: the aim stack's `glist_flags` field on CollisionWorld was
   missing from the four `#[cfg(test)]` literals in world.rs (2865/2893/2933/2958 pre-fix) —
   `cargo test -p nf-game --lib` did not compile on the aim stack's tree. Added
   `glist_flags: Default::default()` to all four. After: nf-game lib **24/0**, nf-formats
   **92/0**, nf-audio **683/0** (5 ignored).

## The regression, attributed and mechanism-named

The aim stack's src changes (door.rs f0/bullet_pass, drone.rs aim_snap + the object stage,
nis.rs, world.rs aim_pick/segment_aim_scoped/glist_flags, scene.rs's aim_snap call) **regress
8 walkreal tests across 5 levels** vs master (the table above). Master has NO aim snap — the
aim point was the raw `camera + dir·256`; the stack snaps it to the crosshair ray's first hit.
The deterministic evidence: on 1b, consoles 1 and 2 never latch on the merged tree (master:
all eight latch; the merged tree latches 3/4/5/6 only) — the console shots are aimed AT script
entities, but the snap ray's object stage EXCLUDES the script entities (the aim-snap-objects
exclusion, commit 05cac0260, measured to keep the [L]-pinned 08 pierce lines), so a shot AT an
entity snaps past it onto the level geometry behind and the round's convergence misses the
console sphere. The same mechanism plausibly drives the other seven (all fight-heavy real-
damage/00agent sisters whose outcomes depend on shot convergence). The aim-object-flags report
already names the full fix: the snap's exact candidate/position chain needs the live debugger
(its "what is wired and what deliberately is not" section) — wiring the entity stage alone
broke the [L] 08 pins in that session's measurement. **This session did not touch the snap.**

## What's left

- The snap's entity-stage completion (the live-debugger chain above) — the one fix that should
  recover the 8 walkreal reds without breaking the [L] 08 pins. The 1b console pair is the
  cheapest deterministic repro for it (console 1 at the merged tree vs master).
- The full 4-crate suite did not fit the window: the three src crates ran (24/0, 92/0, 683/0),
  the lesson-10 set (the 11 aim/walkreal binaries above) ran whole; nf-view's remaining
  non-walkreal tests did not.
- `Door.bullet_pass`'s doc/code mismatch, found while checking the filters: the field's doc
  says "the TRUE word (f0), not the placement flags stand-in this field read before" but the
  constructor still reads `p.flags & 0x60 != 0` (the kind byte 0xdb & 0x60 = 0x40 — every door
  passes by the kind-byte accident the aim-object-flags report described). The 08 pins
  currently ride that stand-in (the panel-model-glist session measured the true-word gate
  stopping the round) — flipping it is a measured-unsafe change that belongs to the snap work,
  not a drive-by fix.

## Live checks for the main agent

1. The console shot (the cheapest repro): in PCSX2 on 0700001b, fire one aimed shot at console
   1 from the walk's stance and record the snapped aim point + the bullet's first object
   contact. Expected on the game: the contact is the console's own object (the snap lands ON
   the entity's built sphere/geometry [M Collide_Pick 0x1eb320's object walk]). If the game's
   snap also passes script entities, the port's exclusion is right and the miss is elsewhere.
2. The `obj+0xF0` word for the console entities on 0700001b (gdbwatch on obj+0xF0 at the
   console's control_create_object): does any of them carry & 0x60 or & 0x3000? If yes, the
   game's own gate would skip them too and the miss needs a different mechanism.
3. The 1b/16 real-damage sisters' first divergent hit tick vs master (gdbwatch on health writes
   around the console phase) — pins whether the regression is the snap's aim point or the door
   f0/bullet_pass word changing a flight stop.

## Usage

not visible to me — this environment shows no token, credit or context-window figures anywhere
I can read.

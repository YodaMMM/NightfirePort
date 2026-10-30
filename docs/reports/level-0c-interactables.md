# level-0c-interactables — report

Branch `s/level-0c-interactables` (from origin/master b3c4e4a1; master had not moved when the
branch was cut, so the tree is the merged tree). Task: one live test per KIND of
player-touchable placement on 0700000c, modelled on `interactables_0700000b_live.rs`, adding
the kinds the level's existing tests don't already pin, and fixing every port gap the tests
expose.

## What's done

### 1. The coverage read

The kinds already green on 0700000c, per the existing tests:

| Kind | Where pinned |
|---|---|
| Door E (`Door_Activate` via 0x1a5170 case 0x1C) | `interact_0700000c_live.rs` (door2turbine #281 + sister) |
| TouchOnce walk-over → channel | `interact_0700000c_live.rs` (#551 → ch 0x43 + sister) |
| Touch plate → objective | `walk_0700000c_live.rs` (#506/#274 → ch 0x1b, monitor states) |
| CamSubject photograph / LoadLevel gate | `walk_0700000c_live.rs` (the camera 0x54 shot; the exit #556) |
| Pickup walk-over (armour, case 3) | `interact_0700000c_live.rs` (#540 armour 50 + sister) |
| Destroyable shot (mask 0 barrel) | `interact_0700000c_live.rs` (#431 dies, far-barrel sister) |
| FuseBox param-less inertness | `interact_0700000c_live.rs` (the twelve 0x31s create none) |
| Zipline (wire_type 2) grab + ride | `interact_0700000c_live.rs` (#512 + sister) |
| Q-Claw grapple latch + reel | `interact_0700000c_live.rs` (#500 + sister) |
| MusicTrigger zone re-arm | `interact_0700000c_live.rs` (#502 + sister) |
| Ladders (both #164/#495) | `walkreal_0700000c_live.rs` leg 5, `ladder495_walk_grab_and_climb_to_the_top` |
| Hand-over-hand wires | `walkreal_0700000c_live.rs` (the jump-grab + SS_WIRE crossings), `fix_wire2_live.rs` |
| Lock burns + AND mux + SS sweeps | `fix_level_0c_live.rs` (hinges #139-142 → #505 → ch 18 → #138; VentLatch #182 → ch 84 → #180/#184) |
| Drones / spawners | guards-0c / guards-0b |

### 2. The census and the correction it exposed

The census is docs/levels/0700000c.md §7 (the placements-all + triggers dumps re-taken for this
task over 0700000c.bin). Checking every player-touchable kind against the coverage table left
five unpinned behaviours, and exposed one census error:

- **The `touchplate_skylight01` rows #275-277 were recorded as "masks 0x1b out ch 2 — the
  secret-door AND's player half". The disc says otherwise** [D triggers dump]: type 2 (Touch),
  **out 0x1b** (the objective-1 channel), **mask 0x2** (drone contact), and **no volume at all**
  (tris 0, box ∞) — inert by disc data. The secret-door AND #560's ch-2 input is the
  body-touch **#559** (mask 8, same spot). docs/levels/0700000c.md §2/§7.1/§7.2/§6 rows are
  corrected with the dump evidence.

### 3. The new file — `crates/nf-view/tests/interactables_0700000c_live.rs` (5 tests, all green)

1. `the_secret_door_needs_the_body_touch_and_the_e_together` — the E-operated trigger volume
   (#558, out ch 3, **mask 4** = the use-key flag) through the real E: the use point (head + 1 m
   of look) lands in #558's box while the body stands one step north of the shared volume, so
   `Trigger_Activate` 0x1b2318's type-2 arm writes ch 3 [M; D #558 mask] but the AND's ch-2 half
   is off and the door holds (the sister is the AND gate itself: ch 3 on, ch 8 off, SS at rest).
   Then the real walk across the box edge writes ch 2 [D #559 mask 8; M Trigger_Hit's mask-8
   arm], the AND #560 raises ch 8 [D list 2,3; M Trigger_Update case 3], the SS door #544
   **swings** off its rest pose (the whole posed matrix moves — the door rotates, the doc's
   "rest 0.75" is the 75 % opening) [M SS_Update 0x1a9948; D #544 p3=8], and the chain's own
   SoundTrigger #562 (on ch 8 → id 1534, non-global [D #562]) starts — the SoundTrigger kind's
   start arm pinned on the chain the game actually drives [M 0x1b2790].
2. `the_hint_plate_fires_ch55_and_the_global_rule_kills_456_while_550_holds` — the hint plate
   #455 walks (TouchOnce → ch 0x55 [D #455; M Trigger_Update case 1]) and its SoundTrigger #456
   (global, [D p5=1]) then attempts to start while the fresh boot's mission voice #550 (global,
   [D p5=1], fired with the spawn walkway's TouchOnce #551) holds the single GlobalVoiceHandle —
   the game's own rule kills the hint object at the attempt [M 0x1b2790:19-28]. Sister: without
   the crossing #456 stays alive in the list.
3. `the_crossbow_pickup_arms_the_weapon_on_the_walk_over` — the weapon pickup kind (case 0):
   #557 Pickup_Crossbow (p1=17, p2=15 [D]) arms weapon 0x11 with rounds on the walk-over
   [M 0x18e650 case 0] and is consumed; sister un-owned.
4. `the_medal_gated_bond_bonuses_create_and_take` — the four `Pickup 007` kind-6 rows (#546-549
   [D]) exist only under the gold word for the level's BaseMap 0x0700000c [L ee_start_05.bin
   @0x2a4478 record 11; M Pickup_Create 0x18dda0:93-95, PlrStats_HasGoldMedal 0x199a40]: the
   gold-word load holds 15 pickups, the walk-over consumes #547 [M 0x18e650:234-236], the
   fresh-boot zero word creates none (sister).
5. `the_bullet_filter_drops_handheld_fire_on_564_and_admits_it_on_492` — the destroyables'
   bullet-category mask (tag+0x14 = ~param4 [M 0x12c318:47-52]): the lit barrel #564 (p4=8 →
   0xfff7 [D]) drops the handheld gunfire word 0x0008 [M Collide_FilterBullets 0x1ece30:26-33] —
   sustained fire kills nothing; the yard barrel #492 (p4=248 → 0xff07 [D]) admits it — the same
   fire kills it. One weapon, both arms, plus the ammo-drain control.

### 4. Port gaps

**None.** Every missing kind worked through the real input path on the first correct test; all
fixes during development were test-side (the #492 leaf lookup keyed on the placement index
instead of a centre radius that matched a neighbouring barrel; the swing detected in the posed
matrix rows, not just the translation; the stand spots moved to clear the walk capsule r 0.55
[M 0x1953f4]; the non-global sound pair chosen for the headless-assertable start arm). The
decompile-backed tests elsewhere were not touched.

### 5. The headless caveat (not a port gap)

The port's `Sounds` nodes never finish headlessly (no audio driver reports sample lengths), so
a one-shot voice that is never stopped holds its GlobalVoiceHandle forever — #456's kill on the
busy slot (test 2) is the decompile's rule [M 0x1b2790:19-28] and the fresh boot's mission voice
really does fire first [D #551/#550], but the "hint voice plays once the mission line ends" leg
is not observable in this sandbox. Listed as a live check below.

## How it was verified

- The touched test binary, on the final tree:
  `NF_ISO=$NF_ISO cargo test --offline --release -j2 -p nf-view --test interactables_0700000c_live -- --test-threads=1`
  → **5 passed; 0 failed; 0 ignored** (11.3 s).
- The full suite with the fast runner, started in the background per the task and re-run to the
  final tree: see the line below (filled from `suite.out`).

```
<pending: the suite total line>
```

- Every game claim in the tests and this report cites [M] a decompile address, [D] the disc
  parse (placements-all + triggers dumps over 0700000c.bin, taken 2026-10-30) or [L] the
  ee_start_05 snapshot reference the level doc already carries. No [S] item is in any
  pass/fail path.

## What's left

- Nothing for the kind map: every player-touchable kind on 0700000c (§7.1) is pinned by the
  tests above or the existing ones (the table in "What's done").
- The #476 plate (out ch 0x66, over #274's spot): its walk-reach from the walkway is not
  reproducible in this sandbox (the stand wedges at the z −109.5 step; the plate band sits
  below the upper stand's feet). It is a route question, not a kind question — the SoundTrigger
  kind is pinned through #562 and #456 instead. Worth a walkreal probe if the east route is
  ever re-driven.
- The three `obj_stor*_pulley` winch SSs (#351/#364/#377) stay dead per the disc (contact-sway
  props, no translation keys [D]; the `SS_Operate` contact driver remains the filed WIP in
  docs/levels/0700000c.md §6) — not player-touchable, listed for completeness.

## Live checks for the main agent

1. **The hint voice after the mission line** — fresh boot (no save-state resume), stand at
   (5.0, ~7.0, −140.2) on the display ground facing −z, wait ~10 s (let sound trigger #550's
   line 1516 finish and free the GlobalVoiceHandle), then walk south across z −141.5 (hint
   plate #455, out ch 0x55 [D]). Expected: the Q hint voice (id 1517) plays. Access type: the
   EE `switch_channels[0x55]` write at the crossing plus the audible voice; the port expects
   the same channel write and, once its driver reports the line's end, the same voice.
2. **The secret door's swing** — a render check: `--bond 52.7,1.0,−171.5` facing the door
   (placement #544 `Door_secret1` at (51.93, 2.15, −174.36)). Before the chain the leaf rests
   shut; after the body-touch + E chain (ch 2 AND ch 3 → ch 8 [D #560]) the leaf should be
   ~75 % open (the posed matrix swing the test measures). The rest pose and the swept pose are
   the two expected frames.

## Usage

not visible to me — this sandbox exposes no token, cost or context-window figures for the
session (no usage panel or status line is shown to me), so there are no numbers to copy.

# level-06-interactables — every interactable kind on 07000006 works like the game

Branch `s/level-06-interactables` (master 2f1edf31 + 8ce724ad merged). Task: 07000006 had only
`interact_07000006_live.rs` (doors, the ambient/trigger ScriptPlayers, the armour walk-over, the
inert hurt volume); census every player-touchable placement, add the missing kinds as
`crates/nf-view/tests/interactables_07000006_live.rs` (model `interactables_0700000b_live.rs`),
fix every port gap the tests expose.

## The census (docs/levels/07000006.md §5 + the audit dump)

Player-touchable kinds on 07000006 and where each is pinned live:

| Kind | Placements | Pinned by |
|---|---|---|
| Doors (0xDB) | #47 E-opens, #369 locked on ch 200, 10 more | `interact_07000006_live.rs` |
| ScriptPlayers (0xD9) | #807 radio (mode-8 ambient + the mask-2 bullet arm), #808 boot NIS, #809 ice, #810 trigger-start | `interact_07000006_live.rs`, `urn_break_live.rs`, `walk_07000006_live.rs`, `cutscene_06_live.rs` |
| Pickups (0xF0) | 11 (ammo ×6, 007 tokens ×2, armour ×2 — one per the take path) | `interact_07000006_live.rs` (armour); **this file** (the kind-6 medal gate) |
| Fusebox (0x31) | #747 (the only parameterised one) | `fusebox_06_live.rs` |
| TouchOnce plates + inverter + siren | #765/#749/#702 (the alarm chain), the route plates | `alarm_panel_06_live.rs`, `walkreal_07000006_live.rs` |
| Searchlights (0xDE) | 4 heads | `searchlight_flip_07000006_live.rs`, `fix_searchlight_alarm_live.rs` |
| LoadPoint trigger (0xEB) | #746 → ch 0x83 | `walk_07000006_live.rs` |
| Hint volumes (0x33) | 3 | `fix_context_hints_live.rs` |
| Hurt volume (0xE4) | #373 (game-inert [M 0x17e5d0 + D #373]) | `interact_07000006_live.rs` |
| **Explodeables (0xE1)** | 6 (oil barrels #321/#322/#419/#420, fuel tanks #69/#327 zero-position) | **this file — NEW** |
| **SimpleScript traps (0xE2)** | 26 barrel rolls/legs/beams (out ch 14 west / 43 east) + Corona_3 #124 + IceSheet #387 | **this file — NEW** (behaviour; the draw was already `simplescript_live.rs`) |
| Not player-touchable | room/key cels, sky-lights, snow, cover nodes, AI volumes/points, spawners (channel-driven), starts, sound triggers (ride the alarm chain) | — |

No 0x29 switch, 0x2E lock, lift, copter or music trigger exists on the level [D census].

## What this task adds

**`crates/nf-view/tests/interactables_07000006_live.rs` — 3 tests, all green:**

1. **`a_player_round_detonates_the_oil_barrel_and_the_blast_takes_the_twin`** (kind 0xE1). The oil
   barrel #321's disc row pins through `Destroy_Create` 0x12c318's param map: out ch 0x29, smash
   type 0xa, health 20 (p3), mask ~0xf8 = 0xFF07, blast damage p5·0.01 = 3.0 [D #321 params]. A
   P2K round through the real trigger (`Sim::step_bond` fire) drains the pool and kills
   [M `Destroy_CollisionHandler` 0x12bd48:30-49]; the death writes `switch_channels[0x29]` and
   the type-0xa blast script 0x0600052 plays and propagates [M `Destroy_Smash` 0x12bb60:40,:59-76;
   `Explode_CollisionHandler` 0x17cde0's one-shot → `Explode_Propagate`]. Sisters: both barrels
   whole and ch 0x29 silent before the shot.
   **The port gap it exposed:** the game's `Explode_Propagate` case 0x21 smashes every reached
   destroyable whose falloff ≥ 1.0 and whose tag+0x14 mask opens the 0x20 bit [M 0x17c518:168-176]
   — #322 stands 0.82 m from #321 with mask 0xFF07, so the game takes the twin. The port's
   `spawn_blast` swept `destroys: &[]`/`breaks: &[]` (only the mounted-laser path swept them).
   **Fixed** in `nf-game/src/drone.rs` `spawn_blast`: it now builds the same live destroyable/
   breakable candidate lists `run_beam_explodes` passes, and handles `hits.destroys` (smash → the
   type-script blast queued, draining next tick — the game's own frame order) and `hits.breaks`
   (the `Break_ApplyDamage` subtract, the beam path's kill). The gate itself was already correct
   in `explode::propagate` (the 0x20 mask bit, the ≥ 1.0 floor). The 0c twin test
   (`a_bullet_explodes_the_oil_barrel_and_the_far_barrel_stays_whole`) still holds: its sister
   #434 is 20+ units and five floors from the blast, far past any falloff.
2. **`a_player_round_knocks_the_barrel_roll_and_the_channel_riders_follow`** (kind 0xE2, the west
   cluster). The disc rows [D audit #353-#365]: every roll carries mask p2 = 0x1 or 0x2 — no use
   bit, no walk-in bit — so the E is dead on them [M `SS_Activate` 0x1a9ea8's `& 4` gate] and a
   walk-through moves nothing [M `SS_Operate` 0x1a9df8:21-40 — class 3 → 0x8, no barrel mask
   carries it]. The player's real input is the BULLET: a round through the mask-0x2 roll #354
   operates it [M 0x1a9df8:35-39], the 0x40 arm latches the sweep [M `SS_Update` 0x1a9948:85-90],
   and — mask bit 0 clear — the latch writes out ch 14 [M 0x1a9948:20-23], whose rising value
   operates the mask-0x1 rider #353 through the channel half [M 0x1a9df8:47-51]. The sweep is
   one-shot: wrap_flag 1 disarms to state 0 holding the LAST key's pose [M 0x1a9948:119-128], and
   ch 14 stays (the write is one-shot HIGH; no other placement on the level writes 0xe [D census
   §5.9]). Sisters: ch 14 silent and both playheads at rest before; the E attempt and the 120-tick
   walk-in each leave the cluster armed. The pinned params: #354 (0x40, wrap 1, mask 0x2, ch 0xe,
   sound 0x11b) [D]; #353 (0x40, mask 0x1) [D]. No port change needed — the mask read, the bullet
   contacts, the latch write and the sound arm were all landed (the operate tick's audio dump
   shows sample 283 among the active ids).
3. **`the_007_tokens_need_the_gold_medal_and_the_walk_over_takes_one`** (kind 0xF0, the kind-6
   arm). The two "Pickup 007" tokens #750/#751 [D §5.6, hash 0x0200092a] are created ONLY with
   the level's gold medal: `Pickup_Create` 0x18dda0:93-95 drops the kind-6 row when
   `PlrStats_HasGoldMedal` reads 0 (the port's pickup.rs:174 arm). 07000006 plays under BaseMap
   0x07000005 [L ee_start_05.bin @0x2a4378: the mission row (level 0x07000006, base 0x07000005,
   order 1)]. Sisters: the fresh-boot word 0 creates no kind-6 pickup anywhere (ordinary pickups
   still exist on the same load); the medal load creates the token at the east-gate yard and the
   walk-over takes it [M `Pickup_Handler` 0x18e650's take — the path the armour test pins].

Teleport stand-ins (documented, operations real): the barrel pair west of the tunnel doors and
the west barrel-roll cluster both sit off `walkreal_07000006`'s legs (its nearest walked anchor
is the hedge gap); the east-gate token yard likewise. Every operation — the aim walk, the
trigger pull, the walk-over take — is the player's own input.

**Touched binaries and totals:** `interactables_07000006_live` 3 passed / 0 failed / 0 ignored
(release, --test-threads=1, NF_ISO set). Full suite on the final merged tree: see below.

## Suite (final tree = master 8ce724ad merged)

<!-- SUITE_TOTAL -->

## Live checks for the main agent

- **The barrel chain on the disc**: shoot one of the courtyard oil barrels (#321/#322, west of
  the tunnel doors, around (−10.94, 42.25, −47.47)) in a fresh-boot PCSX2 run and watch whether
  the 0.82 m twin detonates a beat later (the decompile says it must: case 0x21, mask 0xFF07
  opens the 0x20 bit, blast damage 3.0 ≥ 1.0 [M 0x17c518:168-176; D #321/#322 p4 0xf8]). A
  screenshot a second after the first boom would settle it.
- **The barrel-roll knock driver**: shoot roll #354 at (6.01, 38.87, 2.16) (mask 0x2 [D]) and
  confirm the roll tips and ch 14's sound (sample 283) plays; the walk-through (no shot) should
  do nothing (no class-3 bit in any barrel mask [M 0x1a9df8:21-40]).
- **The blast radius scale** (the audit's open check): the port's blast radius is the object's
  glist sphere × the type multiplier (0xa → ×3); a live measurement of how far #321's boom hurts
  Bond would confirm the multiplier.

## What's left

- Nothing known for the three added kinds. The remaining 07000006 kinds are pinned by the files
  in the census table above. The F3 tuxedo skin swap (the dressing scene's model vanish) stays
  the documented unported item [M Player_Init 0x1910f8:96-107] — not an interactable.

## Usage

- My own session's token/credit/context figures: not visible to me (no usage display in this
  environment).
- The two research subagents this task launched reported (their result metadata, copied exactly):
  explodeable research `Tokens -> prompt:137627 completion:7520 total:145147`; barrel-roll
  research `Tokens -> prompt:116493 completion:7048 total:123541`.

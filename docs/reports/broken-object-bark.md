# broken-object-bark — the broken-object guard bark (Drone_BrokenObject)

Branch `s/broken-object-bark` (cut from origin/master 17a784980; `origin/s/audit-voice-triggers`
merged as 2bc25c6c7 — it was not on master). Shell casings not touched (out of scope of this
task's row).

## What's done

`Break_Kill` 0x12b970's else arm calls `Drone_BrokenObject` 0x138588, and the port never made
that call (breakable.rs only documented it). Ported, with every gate the decompile shows.

**The game function, read in full** [M 0x138588, disasm 0x138588-0x1386d0]:
- Gates, in order: `Drone_bDisableSystem == 0` (0x13858c); `GameState+0xc == 0x07000001`
  (0x1385b0-0x1385bc); the hitdata's +0x58 object exists (0x1385c4), has +0xe0 (0x1385d0), its
  obj+0xff is kind 5 (0x1385d8-0x1385e0 — kind 5 is the BULLET obj, `Bullet_init` 0x12c5a0:87),
  and that bullet's +0xe0 → +0x30 is `glb_players` (0x1385ec-0x1385f4 — the player's weapon
  system). The hitdata that satisfies this is the VICTIM'S MIRRORED bullet contact (+0x58 = the
  bullet, hitdata-spawner-ally.md §4 [M 0x1ea398:189]), which `Break_Update` 0x12bfd8's walk
  passes to `Break_Kill` as the LAST damaging contact [M 0x12bfd8:48-52].
- The list walk from 0x298e14 [M 0x138618-0x1386b0], in list order: the type byte +0xc5 ==
  0x0c else skip (0x138620); a type-0x0c drone with health +0xac <= 0 ABORTS the whole walk
  (0x138628-0x138634 `c.ole.s` + `bc1tl` → return — later drones get nothing); the body-obj
  gate (drone+0xc, its kind 2 or 0x11 [M 0x13863c-0x13865c]); then ONE
  `DroneFunc_ChooseSFX(0x40, 0x43)` = 0x40 + Rand(4) [M 0x1497e8 = lo + Rand(hi−lo+1)] and
  `NDrone2_SpeechSFX(&dcvars, line, 0, 0, 0)` [M 0x138688-0x1386a4] per barking drone.
- Position in `Break_Kill`: after the 41% sound roll [M 0x12b970:66-76], BEFORE the material
  fix and `Debris_Create_Loop` [M 0x12b970:135-141]; the shatter arm (model type 2 = glass)
  NEVER reaches it [M 0x12b970:78-133]. So the bark's `Rand(4)` draws sit between the sound
  roll and the debris draws, and glass breaks never bark.

**Who reaches it** (why the port's other kill sites must NOT bark):
- A player BULLET break: hitdata +0x58 = the bullet (kind 5) from the player's system → barks.
- An EXPLOSION break: `Explode_Propagate` case 0x20 calls `Break_ApplyDamage(*(iVar8+0x58),
  iVar8)` [M 0x17c518:163-164] — the hitdata's +0x58 is the BREAKABLE ITSELF (kind 0x20 from
  `Break_Create` [M 0x12c230]), which fails the kind-5 gate before any draw. Even a
  player-thrown grenade: no bark.
- A guard WALKING through glass: the drone contact's +0x58 is the drone (kind 2) [M
  0x12bfd8:24-33] — fails, and it is glass anyway.
- A DRONE's bullet: kind 5 but +0xe0 → +0x30 is not `glb_players` — fails.

**The port** (crates/nf-game/src/breakable.rs + drone.rs):
- `Breaks::kill_by_bullet(i, a, b, rng, drones, level, system_disabled, out)` — the
  player-bullet kill site (`update_player_bullets`' breakable arm) calls it; the bark walk runs
  INSIDE the kill between the sound roll and the debris draws (the game's draw order), collects
  `(drone index, line 0x40..=0x43)` pairs, and the caller turns each into the drone's
  non-forced talk request (`talks.push((line, false))` — the `NDrone2_SpeechSFX` port's inline
  form, drained by the same per-tick speech pass as every other bark).
- The plain `kill()` is unchanged for the explosion/drone-step sites: their hitdata +0x58 fails
  the kind-5 gate draw-free, so `None` reproduces the identical (draw-free) RNG stream.
- Gate mapping: `Drone_bDisableSystem` = the port's `npc_system_disabled` (explicit param);
  `GameState+0xc` = `level_current`; the bullet/player pair = structural (the caller IS the
  player-bullet walk); the body-obj kinds 2/0x11 = structural (every SP drone's body obj is
  kind 2 [M `NDrone2_CreateObj` 0x14d7c8:14]; 0x11 is the MP bot body [M `BOT_respawn`
  0x124e38:90]); not-present list rows are skipped (the port pre-creates spawner drones the
  game's list would not hold yet [S — the game's list is created-drones only, `Drone_Create`
  0x136be0 / `DroneSpawner_Create` 0x137e68 append]).
- The bark rows 0x40..0x43 are sfx 721/719/720/718, each with a facial id, repeat 180 [L the
  parsed `Drone_SfxInfo`]. On 07000001 the rows' facial scripts (0x60006c4/0x60006c7/
  0x60006ca/0x60006cc) are ABSENT from the level's anim-script set [D the parsed kind-5
  entries] — the game's hashtable-miss arm plays the line DIRECT [M 0x141b00], and so does the
  port's speech pass.

## How it was verified

`crates/nf-game/tests/broken_object_bark_live.rs` (NEW, 4 tests, all on the real ISO through
the real `fire_player` → `update_player_bullets` → `Break_Kill` path):

| test | pins |
|---|---|
| `bond_breaks_a_castle_object_and_the_guards_bark` | 07000001's placement-22 breakable (kind 0, health 0, not glass [D the parsed row]; the disc's voice rows pinned 721/719/720/718 [L]) dies to Bond's pistol round; the bark plays — a live type-0x0c guard's voice becomes one of the four bark sfx |
| `the_same_break_on_another_level_is_silent` | the identical kill on 07000002's placement-3 breakable (kind 0 [D]) kills it but barks nothing [M 0x1385b0] |
| `an_explosion_break_barks_nothing` | the same 07000001 breakable killed through the real blast drain (`pending_blasts` → `Explode_Propagate` case 0x20) dies with no bark [M 0x17c518:163-164 kind gate] |
| `the_walk_aborts_at_a_dead_type0c_guard_and_skips_others` | a dead type-0x0c guard aborts the walk — no later guard barks [M 0x138628-0x138634 `bc1tl`]; the same break with the only 0x0c guard re-typed to 0x02 barks nothing [M 0x138620] (the positive test already shows the walk reaching him past the dtype-0x02 drones ahead of him) |

Run results (`--test-threads=1`, release, NF_ISO set):

| suite | result |
|---|---|
| `nf-game` FULL (lib + every test binary) | **1070 passed, 0 failed** |
| `nf-game broken_object_bark_live` (new) | 4 passed, 0 failed |
| `nf-view interactables_07000001_live` | 10 passed, 0 failed |
| `nf-view walkreal_07000001_live` | **7 passed, 1 FAILED** — see below |

**The walkreal_07000001_live failure is NOT this task's change** (stash-verified twice):
- plain `origin/master`: **8 passed, 0 failed** (run in a clean worktree this session);
- master + the audit-voice-triggers merge (my branch's base, my sources stashed): **6 passed,
  2 FAILED** — `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` (Bond dies at
  tick 3253 to hall-guard fire) and `following_mayhew_down_completes_the_level`;
- my branch on top: 7 passed, 1 FAILED (only `..._real_damage`; mayhew passes).

The new bark triggers the voice-triggers merge ported (its 4 fixed triggers, e.g. the
PlayScript 0x6000168/0x6000169 ENTER/msg-3/msg-5 barks [M 0x15ba68:72/:120/:157]) draw the
shared RNG mid-fight that the old stream did not, and my bark's `Rand(4)` draws add the game's
own `Break_Kill` draws on top — both shift the walker's combat RNG, which is the game's own
behaviour (the game draws in exactly those places), but moves the real-damage walk's outcome.
The walkreal trees need re-verification against the new stream by the main agent; the failure
exists on the merged base without my change and one of the two even flips back with it.

Full `python3 tools/test/suite.py` (all four crates): **did not complete in this session** —
the runner was started in the background three times and the sandbox killed it between tool
calls (suite.out stayed empty), and the foreground build of the 107 nf-view test binaries did
not fit the window either. What ran instead, in the foreground: the full nf-game set (above),
the two named nf-view binaries, and — earlier in the session, same sandbox — the
fx-npc-footsteps branch's full walkreal sweep. The remaining nf-view binaries did not run
here.

## What's left

- The nf-view suite beyond the two named binaries (the sandbox's background-job kills + the
  window; nothing known against it — nf-game is fully green and the two nf-view pins pass,
  modulo the pre-existing walkreal note above).
- Live checks for the main agent:
  1. 07000001, shoot a castle pot/crate (placement 22's row is the first, kind 0) with a guard
     of type 0x0c alive (the level has exactly one in its start set, skin 0x0500004f [D]): a
     breakpoint on `Drone_BrokenObject` 0x138588 must HIT, and `NDrone2_SpeechSFX`'s id
     argument must be 0x40..0x43. Access: breakpoint, read a1 (the chosen line).
  2. The same break with that guard dead first: 0x138588 must still hit (the gates pass) but
     the walk must stop at the dead drone — no `NDrone2_SpeechSFX` call. Access: breakpoint.
  3. An explosion break (grenade into the same pot): 0x138588 must NOT be reached (the kind-5
     gate fails on the breakable's own kind 0x20). Access: breakpoint.

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session).

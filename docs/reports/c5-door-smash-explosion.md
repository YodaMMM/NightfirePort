# c5-door-smash-explosion — the C5 door blows without its explosion

Branch `s/c5-door-smash-explosion` from origin/master 6f16146e.

## What the game does [M]/[D]

When a Destroy leaf dies, `Destroy_CollisionHandler` 0x12bd48:46-49 reaches `Destroy_Smash`
0x12bb60, which — for EVERY leaf whose type byte (placement param 2 & 0xff) is non-zero — runs
`Explode_Create` 0x17cbd0 [M 0x12bb60:LAB_0012bc50]:

- the script by the type byte: 1 → 0x6000027, 2 → 0x600004f, else 0x6000052 [M 0x12bb60];
- the blast magnitude = the OBJECT's parsed bounding-sphere radius (+0x8c, written by
  `parseentity_transform_bounding_box` 0x1d0fb0 from the cel's field) × the type multiplier
  (1→4, 2→2, else 3) [M 0x12bcdc `lwc1 $f12, 140($16)` + 0x12bb60's `*(float *)(iVar6 + 0x8c) * uVar5`];
- `Camera_Shake(pos, radius)` only from a blast ≥ 0.5 [M 0x17cbd0:40-44; 0x17cc28];
- a class-0x40 explosion object whose script plays facing the closest player
  (`Mat_Align2Dir` + `Script_Play`) [M 0x17cbd0:69-110], lifetime the script's +0xAB0 u16
  (165 for the standard blast [L/M 0x17cd18]).

The C5 pair [D #93 bigdoor01 / #96 bigdoor02, class 0xe1]: types 5 and 6 → both take the else
arm → script 0x6000052; the laser kill drains bigdoor02 (~201 contacts [L measured]) and the
same tick's channel chain (ch 0x1f → `Destroy_Update` 0x12c4f8:29-33) smashes bigdoor01 at full
health — so the game's death plays TWO same-tick script-0x6000052 blasts. No debris from the
blast itself (`Destroy_Smash`'s u16 debris arg is an explicit 0 [M 0x12bcf8]).

## The port gap and the fix

The port's `Props::smash` (props.rs) carried a `< 0.05` sphere guard that returned None — no
blast at all — for a leaf with a degenerate sphere. It was a draw workaround from the one-slot
era ("a same-tick empty blast hid the C5 fireball"); obsolete since the draw holds BLAST_COPIES
slot sets per blast script (concurrent_blasts_draw_live). Dropped: the port now spawns the
blast for every non-zero-type leaf, exactly the game's gate. All the existing blast machinery
already ran for the big blasts — `spawn_blast`'s script SP (`spawn_once_scaled`), the camera
shake, the explosion record — so the C5 door's fireball exists end to end; the twin now spawns
too.

[D] data note: bigdoor02 #96's object record (#94) parses with a real sphere (3.144 → the
9.43 blast), while bigdoor01 #93's object record (#91) carries the parse's ±1e7 sentinel bounds
→ sphere 0.0001 → its twin fireball plays at scale ~0 (invisible). If the game's own parse gives
#93 a real radius, its twin fireball is visible there — that is a level-parse gap, not a smash
gap (live check below).

## How it was verified

- New real-input test `0700000d_gunimp_door_live::the_laser_kill_spawns_the_smash_blasts`:
  ST-3 plate → the mount → the aim → the held trigger through the real laser path; on the death
  tick it pins BOTH blast ScriptPlayers (script 0x6000052 ×2 — the kill and the channel-chain
  twin [M 0x12c4f8:29-33]), the door blast's scale = the leaf sphere·3 = 9.432468
  [M 0x12bcdc], the live camera-shake byte (the decayed Camera_Shake input [M 0x17cbd0:40-44]),
  and that every fireball entity hash resolves to session art (the draw has meshes).
- `destroy_draw_keys_0700000d_live::the_c5_smash_sets_off_one_fireball_at_the_door` — its
  one-fireball pin encoded the workaround era; updated to the game's two-blast shape with the
  citations (the big blast's position and radius still pinned).
- Runs: 0700000d_gunimp_door_live 4/4; walkreal_0700000d_live green (1 passed, 1 ignored — the
  pre-existing real-damage ignore); destroy/blast binaries (blast_debris_draw,
  destroy_channels_02, destroy_wreck_07000014, explode_live, fix_blast_scripts,
  concurrent_blasts_draw, g049_blast_light, copter_death_debris, abseil_damage) all green;
  walkreal_07000014/06/02/05 green (2+12+5+8); nf-game + nf-formats + nf-audio green except the
  3 allowed `disc_golden` failures on the slim ISO.

## Render check for the main agent (--bond)

- `--bond 137.3,9.3,-323.0` (the deck gun seat, aiming at the door): fire the laser until the
  C5 door dies (~200 contact ticks, health 10000 → the death) — the fireball plays AT the door
  (134.4, 5.7, −269.5), burns ~165 ticks, the door leaf stops drawing.
- `--bond 130.0,4.0,-262.0` facing the door plane: the same kill from the courtyard side — the
  fireball fills the doorway.
- The camera shake: the kill tick shakes the view (the shake byte 9.43 at the drain).

## Live checks for the main agent

1. 0700000d, level load: fieldprobe the C5 twin's object parse — bigdoor01 #93's obj+0x8c
   (read) — expect the parsed door-model radius (~3.1, matching bigdoor02's) if the game merges
   the model; the port's object record #91 has sentinel bounds [D] and its twin fireball plays
   at scale ~0. If the game's value is ~0, the port matches and nothing is missing.

## What's left

- The full nf-view suite slice did not fit the window; the destroy/blast-adjacent binaries and
  the destroy-heavy walkreals (0d, 02, 05, 06, 14) ran green (above). No
  `crates/nf-view/tests/common/` files were touched, so the shared-helper opt-in rule and its
  every-walkreal sweep do not trigger.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for the
session (no numbers were visible in any interface output to copy).

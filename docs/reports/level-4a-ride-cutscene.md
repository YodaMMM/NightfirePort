# level-4a-ride-cutscene — the 0700004a ride cut-scene's objects

Task: on 0700004a the ride cut-scene (SP #289, pack 0x060006cf, ticks ~1611-2186) played with
pieces missing — the satchel the guard is holding and the lift floor under Bond. Decode the
pack's streams, list every object it spawns/attaches with the op that does it, and port the
missing ops/lookups.

## 1. The pack decode [D 0780004a.bin 0x060006cf]

`nf-extract nis` + a direct levelbin walk: camera **true**, length 575, 28 streams, 1410 keys.
The pack lives ONLY in 0780004a.bin (kind-7 entry, 80,480 bytes) + six kind-4 sequences
(0x040005fd/fe/ff, 0x040006e6/7/8); the game reaches it through the type-0xC loadable record
(the port's fix-cutscenes pack merge). The play archive 0700004a.bin holds the skins
(0x05000004/6/7), the anim scripts (0x060006d0/d1/d2) and all four glists the pack names.

| Stream | Op | Object | Evidence |
|---|---|---|---|
| s0 | camera (op 0xD family) | the scene camera, 126 keys, descends with the car | [D s0 keys] |
| s1/s2/s3 | op 0xA `Script_AnimStart` 0x1d8f40 | anims 0x060006d2/0x060006d1/0x060006d0 on skins 0x0500006/0x0500007/0x0500004 (0x0500004 = the level's guard skin [D 0700004a.md §7]) | [D s1-s3] |
| s4 t0 | op 0x12 event 0xB param 0x18 | the held-item byte on the PREVIOUS stream's anim (s3): `sAnimObject+0x62 = 0x18` — weapon_data[0x18]'s third-person mesh on the hand datum (`AnimObjectDraw` 0x1c4490:256-265) — the satchel in the guard's hand | [M `Script_EventHandler` 0x1d94c0 case 0xb; D s4] |
| s6 | op 7 Entity hash 0, kind 0xff | the marker stand-in | [D s6] |
| s7 | op 7 Entity 0x02000758 kind 0x31 | glist **OBJECT_DemoCharge** (0700004a.bin map 64 [D gfx census]), keyed 0..594 — the satchel prop's hand path (local y +1.49 → −0.78) | [D s7] |
| s8 t595 | event 5 | `Drone_EnableAll(0, hash)` — the garrison freeze at the scene's end | [M 0x1d94c0 case 5; D s8] |
| s9 t0 / t574 | events 4, 7 | `Player_Disable` (the scene freeze); flags 0x80 (re-enable drones on free) | [M cases 4/7; D s9] |
| s10 | op 7 Entity 0x020006e1 kind 0x31 | glist **NIS_LIFTHATCH** (map 63), keys t0 → t575 (local y −7.99 → −14.31) | [D s10] |
| s11 | op 7 Entity hash 0 kind 0xff | marker stand-in, 278 keys | [D s11] |
| s12 | op 7 Entity 0x020006b1 kind 0x31 | glist **NIS_P99** (map 70), 528 keys | [D s12] |
| s13 | op 7 Entity 0x020006ee kind 0x31 | glist **NIS_lift** (map 62) — the car piece the scene shows, keys t0 (+0.99) → t575 (−5.33): the car departs mid-scene | [D s13] |
| s14-s25 | op 0x16 | ten 2-D sounds (0x210/0x27b/0x280/0x283/0x445/0xa3/0x25a) | [D] |
| s25 t575 | op 7 Entity hash 0 kind 0x19 | the world-space hand-back marker (1.79945, −54.12811, −3.45340) — pinned live by 4a-lift-4 (the hash-0 kind-0x19 class skips the base) | [D s25; L walkreal probe_l3] |
| s26 t575 | event 6 param 1 | `Player_Enable` at the previous stream's object (s25's marker) | [M 0x1d94c0 case 6; D s26] |
| s27 t575 | event 0xD [0x22, 1] | switch channel 0x22 = 1 — the hand-back mark that releases the lift SP | [M case 0xd; D s27] |

`Script_GetObj` 0x1db688: the event handler's target is the PREVIOUS stream's object — the
handler's record base is SCRIPTINFO+0x80 (one 0x28 record below `Script_Run`'s +0xa8, and
`+0xab3` is the run loop's stream counter [M `Script_Update` 0x1da998:25-41]), so event 0xB's
s4 write lands on s3's anim object. The port already implements exactly this (nis.rs event()
0xB arm, verified by the test's weapon-byte assert).

## 2. What the port already carries (pinned by the new test)

`crates/nf-view/tests/ride_cutscene_objects_live.rs` (board the lift with real input, ride to
the scene, assert mid-scene): the sim carries ALL FOUR glist entities (satchel 0x02000758 first
seen at (3.43, −52.05, −4.21) = SP #289 base (0,−53.53,−3.38) [D placement] + the local key
(3.43, +1.49, −0.84), still tracking its keyed path at t300; NIS_lift at (0,−52.54,−3.38);
NIS_P99; NIS_LIFTHATCH), the s3 guard anim runs 0x060006d0 with the weapon byte 0x18, all three
characters' poses resolve (the pack sequences load under the 0x80|skeleton keys), the scene has
the entity draw slots (per-copy k: 2 per stream — #273 AND #289 both run the pack [D census]) and
the three anim characters. 576 disabled ticks (575 [D]).

## 3. The gap found (NOT yet fixed — the sandbox hit its 19:19 line)

**The render's script-anim characters skip the SP base compose.** The pack's body-sequence roots
are CAR-LOCAL — [D] sequence 0x040005fd (222 ch, flags 0) channels 0/1/2 at frame 1 = (2.35,
0.85, −0.64), constant over the scene — the same car-local space as the satchel's entity keys,
which the port DOES base-compose (its entity rule, render-verified on 07000014/08). The port's
`frame_state_for` script arm poses the character with an identity object matrix ("the object
sits at the origin and the sequence's root carries it" — true for the WORLD-root sequences the
older cut-scene fixes were pinned on, e.g. 07000001's body tracks whose roots hold absolute
world positions), so the pack's cast draws at the raw root ≈ (2.5, 0.6, −0.9) — ~53 units above
the car, inside the shaft's top rock. The guard is not in the scene, and the satchel mounted on
his hand datum (event 0xB → the weapon-data mesh) goes with him — the reported "the satchel the
guard is holding is not there". ("Bond stands on nothing" is the same class: the scene's car
pieces ride the pack keys + the SP base; with the base mis-applied only on the character path
the composed picture loses the car interior the camera expects. The entity path's compose is
correct per the data; the render claim needs the main agent's render to confirm.)

**Fix recipe (next session):** carry the owning script's base on `Obj::Anim` (a new field on its
own line, set at `Script::anim_start` from `self.matrix` — the game's SCRIPTINFO+0x00 the entity
path composes [M 0x1d9a08:94-133; `Script_AnimStart` 0x1d8f40 stores the owner at obj+0xe0]),
expose `Scripts::anim_object_base(src, skin)` (the newest-match walk `script_anim` already
uses), and in `frame_state_for`'s script arm compose it into the character's object matrix.
Scope note: world-root cut-scene SPs sit at base ≈ origin (checked on 0700004a: #42/#273 at
(0,0,0), #276 at (−0.2, 1.69, 0)), but other levels' camera scripts were NOT probed — the
compose must land with a render check (rule 6).

## 4. Verification run

- `cargo test --offline --release -p nf-view --test ride_cutscene_objects_live` — 1 passed (the
  new test; NF_ISO onfoot ISO).
- `cargo test --offline --release -p nf-game --lib` — 570 passed, 0 failed, 3 ignored.
- No source files changed this session (test + docs only), so the touched-crate surface is
  nf-view's new test + nf-game's lib run above. The full four-crate suite could NOT run in this
  sandbox: background processes are killed between tool calls here, and the foreground budget
  went to the decode/probe work. The next session should run `tools/test/suite.py` on the final
  tree (merged with origin/master) before the merge.

## 5. Live checks for the main agent

1. **The cast's drawn position** (the gap): render 0700004a mid-ride (the scene ticks ~1750-2150)
   at `--bond 3.2,-52.0,-4.2,0,0` — the guard should stand in the car; expect him missing (or at
   the shaft top) before the fix.
2. `gdbwatch` the game's anim object for skin 0x0500004 during the scene: obj+0x30 over ticks —
   expect the car-local root (2.35, 0.85, −0.64) composed with the SP base (0,−53.53,−3.38) at
   the draw (obj+0x90 ≈ (2.5, −52.9, −4.3)); this pins WHERE the game applies the base (the
   obj+0xe0 owner chain) so the port composes at the same spot.
3. The entity compose sanity: the satchel entity's obj+0x90 during the scene — expect the base +
   local keys ((3.43, −52.05, −4.21) at t0 [matches the port], descending ~0.011/tick) — confirms
   the entity path's compose against the real game.

## Usage

not visible to me — this sandbox exposes no token/credit/context-window or cost figures (no
usage meter is shown in the session UI or tool output).

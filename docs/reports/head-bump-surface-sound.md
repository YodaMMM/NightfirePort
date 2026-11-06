# head-bump-surface-sound

Task: `Player_HandleJump` 0x1916b0:194-198 reads `*(obj+0xDC)+0x50` — the anim record's contact
byte, not a level tag. The port's `LadderCtx::level_bump_5fb` comment and
docs/systems/sound-coverage.md had the level-tag reading, the flag was never set, so Bond always
played 0xA3.

## What's done

**The byte's writer chain, confirmed from the decomp + disassembly** ([M] throughout):

- `Player_HandleJump` 0x1916b0:194-198 (C): the bump arm plays 0x5FB when `*(byte *)(*(obj+0xdc)+0x50) - 0xd < 2` (0xD/0xE), else 0xA3; the arm is gated on `*(obj+0xdc)+0x60 & 8`.
- `Player_CollisionHandler` 0x1949a8:29 clears the byte every tick [M disasm 0x194a18 `sb zero,80(v1)` — the only direct byte store in the whole collision path; the other three functions (0x194188/0x194458/0x194df0) have none].
- The byte is written by 0x60-byte HITDATA copies: `Player_FeetOnPoint` 0x194188 tail (C :183-191, [M disasm 0x1943cc-0x19440c] — gated on the bit-8 gate + `surfaced` + the byte not already 0xD/0xE; the copied HITDATA is the downward feet-ray's first material-nonzero hit) and `Player_DealWithObjHit` 0x194458:65-85 (object hits whose material is 0xD/0xE). HITDATA+0x50 is the surface material [M docs/fields/hitdata-spawner-ally.md +0x50; writers `Intersect_PointGeom` 0x1e92d8:321, `Intersect_CylGeom` 0x109da0:568]. So the byte = **the surface material of the contact** whose HITDATA was copied, 0 when nothing was hit — confirming the task's reading over the level-tag one. (docs/fields/animobject.md +0x50 carries a competing "0x0D/0x0E = airborne action state" row with no writer of 0xD/0xE anywhere in the decomp — the material reading is the only one with writers.)
- Tick order [M]: `Player_SSWalk` → `Player_HandleJump` runs inside `Player_Move` (Player_Update:182), `Player_Collision` (→ the handler → the byte writes) at Player_Update:361 — the bump arm reads the PREVIOUS tick's byte.

**The port change** (crates/nf-game/src/world.rs, crates/nf-game/src/player.rs, crates/nf-view/src/scene.rs):

- `CollisionWorld::capsule_push_scoped` / `capsule_push` now return a third value: the raw surface byte of the last contact whose push pointed DOWN (the ceiling contact), 0 when nothing pushed down (the game's own material-0 no-hit sentinel [M Player_FeetOnPoint 0x194188:104]).
- The blanket `mat & 0x3f == 0xE` skip in `capsule_tri_contact` became `top_guard (the swim path) || n[1] > -0.7`: the swimmer and the up-facing/vertical water planes stay pass-through exactly as before, but a DOWN-facing water plane (the pond undersides) now collides the walk/NPC capsule — the game's `Intersect_CylGeom` tests every triangle whatever the material (the byte only picks effects).
- `Bond::bump_surface` (new field, on its own line) captures the byte when `head_bump` sets; the bump arm plays `(bump_surface.wrapping_sub(0xd) < 2) ? 0x5FB : 0xA3` — the game's byte test verbatim. `LadderCtx::level_bump_5fb` is removed (the wrong mechanism), with every literal updated.
- The `NF_WALK_CLIP` experiment's clip path carries no material (its contact body is `capsule_tri_clip`); under that env-gated experiment the bump stays on the 0xA3 arm. Documented in the code.

**The comment and the doc**: the `LadderCtx::level_bump_5fb` doc is gone with the field; docs/systems/sound-coverage.md's head-bump row now states the contact-byte mechanism and its live-check entry is rewritten.

**The disc census for the test spots** ([D] — a probe over every `0700*.bin` placement collision in the ISO, all 36 level bins): material 0xD appears **nowhere** in any level; 0xE down-facing triangles exist only in 07000002 (map 51 obj 19, the pond underside, y 4.33-4.34) and 07000016 (map 63 obj 94, the pool underside, y −4.84). The 07000002 underside sits inside a WATER cel — the Sim (and the game) swims there, and `Player_HandleJump` runs only from the walk [M Player_Update 0x193600 dispatch], so **no walk-jumpable water ceiling exists on the disc**: the 0x5FB arms ride unit pins, and the real-level tests pin the 0xA3 else-arm on two levels' dry ceilings. A live check to adjudicate the byte's value at a swim-adjacent bump is listed in sound-coverage.md.

## How it was verified

- `crates/nf-view/tests/head_bump_surface_sound_live.rs` (new, 5 tests, all green):
  - unit pins on a hand-built world: the 0xE ceiling bump plays 0x5FB, the 0xD ceiling bump plays 0x5FB, the dry (0x4) ceiling bump plays 0xA3 twice (launch + bump) and never 0x5FB;
  - real-level pins: 07000002's 0x4 concrete underside at (−46.14, 8.16, −18.48) over the 6.32 floor and 07000003's 0x4 underside at (8.95, 8.87, −32.30) over the 6.93 floor — Bond placed on the floor (the gunimp tests' placement pattern), jump held through the real Sim: the bump keeps 0xA3, never 0x5FB.
- **Fail-first, proven by stash**: with the src fix stashed the suite runs red — `the_pond_underside_head_bump_plays_5fb` FAILED (`[163]` — on master the old blanket 0xE skip meant the pond underside never even bumped) and `the_0xd_surface_head_bump_also_plays_5fb` FAILED (`[163, 163]` — the 0xD bump played 0xA3); the dry guards were green before and after.
- Quoted whole-binary results (`NF_ISO=$NF_ISO cargo test --offline --release -j2 … -- --test-threads=1`):
  - `nf-game --lib`: **683 passed, 0 failed, 5 ignored**;
  - `head_bump_surface_sound_live`: **5 passed, 0 failed**;
  - `walkreal_07000002_live`: **3 passed, 0 failed, 1 ignored** (81.9 s);
  - `walkreal_07000005_live`: **14 passed, 0 failed** (73.6 s — the swim-heavy level, guarding the 0xE skip change);
  - `walkreal_07000003_live`: **6 passed, 0 failed, 1 ignored** (412.4 s);
  - `interactables_07000002_live`: **18 passed, 0 failed**;
  - `mp_bot_body_sounds_live`: **1 passed, 0 failed** (the bots never play 0xA3/0x5FB — unchanged).
- The full four-crate suite did not fit this session's budget; every battery that touches the changed code (nf-game lib + the collision/water levels' walkreals + the MP sound pin) ran whole as listed. nf-formats/nf-audio are untouched by the diff.

## What's left

- A PCSX2 live check (listed in docs/systems/sound-coverage.md): confirm the byte's value at a swim-adjacent bump under the 07000002 culvert (gdbwatch on `Sound_Play3D` 0x1dd548's `$a0`), and whether the game's own bump-on-static-ceiling byte carries the FEET-RAY hit's material (the FeetOnPoint tail's copy source) rather than the bumped ceiling's — for static ceilings the port now passes the ceiling's material through per the task; the decomp's copy-source reading (the downward ray's hit) differs only where ceiling and floor-below materials disagree, and the disc has no walk-jumpable spot that separates them.
- The `NF_WALK_CLIP` experiment's clip path could thread the material like the push path if that experiment ever lands.

## Live checks for the main agent

1. 07000002 culvert: swim up to the pond underside and jump-bump from the water's edge with gdbwatch on `Sound_Play3D`'s `$a0` (0x1dd548) — expect 0x5FB iff the contact byte at the bump reads 0xD/0xE; record the byte at obj+0xDC+0x50.
2. Any low dry ceiling (e.g. 07000003's 0x4 underside at (8.95, 8.87, −32.30)): same watch — expect 0xA3.

## Usage

No token, credit, cost or context-window figures are visible to me in this environment (no usage meter is shown in the session UI or the tool outputs). not visible to me.

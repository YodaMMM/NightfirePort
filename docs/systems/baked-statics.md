# The baked 0x8019/0x8018 statics (floating-rifle-02)

The disc's kind-0x8019/0x8018 placements (`flags & 0xffff`) are static-scenery records the
static parse turns into runtime cels: `parsemap_block_map_data_static` 0x1d1998:71-120 allocates
a cel record for every placement whose flags have bit 0x8000, copies the glist pointer (+0x38),
the transform (+0x60..+0x78) and the glist flags word (cel+0x94 = celglist+0x44), then runs
`parseentity_fixup_entity` 0x1d00c0.

## The bake rule [M 0x1d00c0:152-179]

The fixup's 0x8019/0x8018 arms (and the converted-dynamics arms that fall through to
LAB_001d04d0) test the record's matrix rows 1 and 2 against the identity
(`Vec_IsEqual(row@+0x60/+0x70, 0x2d6200)` — the epsilon-taking compare): BOTH identity → flags
bit 0x8000000. That bit is the artist's BAKE mark — the art already lives in the room's cel
geometry; the record is a marker and contributes no draw of its own. A rotated record keeps
drawing as a live inner-node static (`View_AddCels` 0x1e6ba0's ±dist² list, gated by
`Vision_InView`).

Evidence anchors:
- 07000002's `bath-sandalorange`/`bath-sandalblue` (kind 0x8019, rotation [0,0,0], glist centres
  (−39.073, 5.396, 9.961)/(−39.098, 5.395, 11.266) [D placements + glist bounds]) — the port
  drew them a second time floating over the wall right of the bathhouse doorway (the user's F12
  005-07000002 frame 4206; the driver's `--bond -43.11,6.46,10.37,2.305,0 --hold-npcs --frame
  300`, screen ≈ (1020, 480) of 1280×720 — 5 material batches, 4× texture 142 + the strap's
  texture 12) while PCSX2 shows nothing there [L 2026-10-20, fresh boot].
- Rotated 0x19s draw in the game: 07000009's `liftswitch_fake` dressing (yaw π [D placements])
  is visible in PCSX2 [L]. A yaw-π record has row 1 identity but row 2 [0,0,−1] — the game's AND
  fails — so the row-1-and-2 test, not a whole-matrix test, is the game's own.

## The port

`scene.rs`'s static World walk skips `flags & 0xffff ∈ {0x8019, 0x8018}` placements whose
`object_matrix(rotation, translation)` rows 1 and 2 are the identity (1e-4 tolerance — the game's
PS2Sinf(π/2) is one ulp above 1.0 [M placement.rs]). The 0x8018 arm shares the same fixup block
[M 0x1d00c0:168-179] but has no live anchor of its own yet — the kind-wide rule is [S] for 0x8018
alone. Collision is untouched (the statics' cel records join the game's cel collision machinery;
no [L]/[M] evidence says the baked records lose it).

Tests: `crates/nf-view/tests/baked_statics_02_live.rs` — the sandals draw nothing (draw list =
`scene.materials` 1:1 [M gpu.rs]), the rotated `northerngarden4_lightfixture` (yaw 0.733) still
draws, the room cels survive.

# The silencer's white artifact: the skin-extra copy of the +0x7C view-entity (2026-10-20)

Symptom (2026-10-19): weapon attachments such as the silencer showed a white artifact.
The silenced P2K (record 0x03) drew the suppressor mesh TWICE: once as the fp skin's rigid
extra part and once as the +0x7C datum-0 attach (fix-silencer-visual). Both copies carry the
mesh's additive sheen batch, so the two adds stack — a blown-out white sleeve around the
suppressor. The fix skips the skin extra whose hash equals the record's +0x7C; the datum
attach stays the mesh's only draw.

## The data chain [M decompile / D disc / L snapshot]

1. **The suppressor mesh is the record's +0x7C view-entity.** `Player_WeaponFiring` 0x1a5460:96-110:
   while the held record's flags (+0x68) carry 0x800, `AnimDatumSetEntity(vm, 0, weapon+0x7C, 0xff)`
   every tick; the twin arm (0x1a5460:111-118) clears datum 0 while the bare record is held. The
   twins share +0x7C (the P2K pair's = 0x02000842) [L the golden weapon table 2fd48d6d].
2. **The game hides the weapon-slot objects every tick.** The same `Player_WeaponFiring` opens with
   32 `View_SetDrawInNoViews` calls over BL+0x7ec..+0x86c plus BL+0x86c (the slots
   `Player_InitWeapon` 0x1a7ef8 created as empty view-only objects), before the datum attach — the
   extra copies of the mesh must not draw; the datum entity is the mesh's only visible draw.
3. **The fp skin ALSO carries the mesh as a rigid extra on bone 43** [D, the kind-3 skins of
   07000005.bin]: skin 05000088 (the P2K) extra bone 43 = 0x02000842 'Suppressor'; likewise
   05000047→0x020001f2, 05000050→0x020005b2, 050000a8→0x0200099d, 050000a9→0x020009a3 — all on
   bone 43. `AnimObjectNew` 0x1be910:151-172 creates one child object per extra (glist =
   `AnimProcessSkinData` 0x1bd500's +0x8 array, parent = the anim object); `AnimObjectDraw`
   0x1c4490 draws ONLY the main skinned list (+0x2c/+0x4) and the datum entities (+0x2f/+0x20,
   +0xa4 glists, +0xb4 countdowns) — never the extra glists.
4. **The mesh's second batch is an additive sheen sleeve** [D 07000005.bin map 45 object 3
   0x02000842 'Suppressor']: leaf 0 batch 1 = 18 verts (an octagonal sleeve r≈0.03, z 0→0.193),
   ALPHA_1 = 0x7f00000048 (additive Cs·As + Cd [M render.md]), ZBUF_1 = 0x0131000000 (no depth
   write), TEST_1 = 0x5001b inherited, vertex rgba (255,255,255,64), map texture 3 = 64×64 opaque
   (alpha 255 everywhere) specular gradient with 384/4096 pure-white texels. Two overlapping
   additive copies double the add — the white artifact. The first batch (the tube, 34
   verts, ALPHA_1 0x44 normal, vertex alpha 128) is unaffected by the doubling (opaque replace).
5. **Datum 0 sits at the muzzle** [D the skin's datum table]: id 0, bone 38, t (−0.0013, 0.0909,
   0.1415), q ≈ identity — the datum attach and the bone-43 extra occupy the same world spot.

## The fix

`crates/nf-view/src/scene.rs` (the view-model build): skip the skin extra whose hash equals the
record's +0x7C (when nonzero). The bare twin loses the suppressor (the game's twin-clear arm —
the game shows no suppressor on the bare record), the silenced twin draws the suppressor exactly
once (the datum-0 attach). No draw code changes; the GPU/blend paths are untouched.

## Tests

`crates/nf-view/tests/silencer_white_texture_live.rs` (NF_ISO):
- the silenced twin's suppressor draws ONCE: no bone-43 ident attachment on either twin's view
  model; the datum-0 entry (bone 38, the datum's q/t) on the silenced twin only — sisters pin the
  body extras (bones 38-42) unchanged on both twins and the datum-0 mesh's batch state;
- the suppressor mesh's batches pin [D]: batch 0 normal 0x44, batch 1 additive 0x48/no-depth-write,
  both textures resolve in the scene (the white-fallback count of the view model stays 0);
- the sister weapon without an attachment: the bare P2K's body parts keep their textures.

## Live checks (open)

- `--give 2 --bond <05 spawn> --shot`: the bare P2K first person, close on the muzzle — NO
  suppressor, no white sleeve (pre-fix: the suppressor's tube+sheen sat on the gun).
- `--give 3` (or the B mode press) first person, close on the suppressor: ONE suppressor tube with
  at most the game's subtle sheen; NO white band (pre-fix: the doubled additive sleeve).
- PCSX2 comparison spot: level 05 armory, silencer equipped, zoom on the suppressor from the left.

# Driving: projectile models, smoke ribbons, lights, explosions and bullet streaks

Port: `crates/nf-formats/src/driveproj.rs` (data) and `crates/nf-game/src/driving/projectile_gfx.rs`
(logic). Disc test: `crates/nf-game/tests/drive_projectile_gfx_disc.rs`; unit tests in both files.

Tags: [M] DRIVING.ELF decompile/disassembly (work/decomp/DRIVING.ELF/), [D] disc bytes, "ELF data" =
initialised DRIVING.ELF words. Nothing here is a live capture. The view does not use any of it yet
(it still draws a stand-in glow per projectile).

## 1. The model: a copy of a shooter car item

The fire event `FUN_0014ca60` creates the `Missile` object and then calls
`FUN_001c7b60(missile render object, shooter render object, sub-item, 0x355bb0, 0xfb, 0x40)`: the
shooter's `in#0` item `sub-item` is copied as the projectile's only item, with the identity local
matrix (0x355bb0), its hidden bit cleared, bytes `+0x0d/+0x0e` set to 0xfb / 0x40 [M]. The weapon id
comes from the same event: `FUN_001c9328` reads the item's action byte, `as + 0x1a` (`+ 0x1b` when the
item is mirrored, flag 0x80), and `FUN_0020e678` returns the weapon whose barrel list (0x33a0b0)
holds it [M]. When the event carries a type-6 or type-2 `ef` entry, the copied item is that entry's
`+0x15` [M]. So the rocket that flies is the rocket mesh on the car.

The car data [D, all car files of the 8 track archives and MISC; every copy of a car is identical]:
the weapon items carry an `ef` list whose type-6 entry names the launch effect and, at `+0x15`, the
item to copy. A mirrored item shares its twin's `as` set, so both barrels copy the unmirrored item.

| car | weapon | barrel → item → model item | launch effect |
|---|---|---|---|
| vanquish | Rocket | 5 → 39 → 39, 6 → 40 → 39 | `Wp_Rock` |
| vanquish | Missile | 7 → 47 → 47, 8 → 48 → 47, 0x34 → 30 → 30, 0x35 → 31 → 30 | `Wp_Rock` |
| vanquish | Q-Pulse | 0x24 → 14 → 14 | `Wp_EMPtail`, `Wp_EMP` |
| vanquish | Gun | 9 → 36 → 36 | `Dummy` (two entries at the muzzles) |
| vanquish | Q-Smoke / Q-Booster | 0xe → 15; 0xd → 50/51 → 50 | `Cr_Smoke`; `Cr_Boosz` |
| vanquishsub | Torpedo / Remote Torpedo | 0x3d/0x3e → 22/23 → 22; 0x3f/0x40 → 12/13 → 12 | `Cr_Torp` |
| alpha_sub | Torpedo | 0x3d/0x3e → 2/3 → 2 | `Cr_Torpbig` |
| mini_sub | Torpedo, Floating Mine | 0x3d/0x3e → 21/20; 0x41 → 19 | `Cr_Torp`; `Cr_Mtail` |
| ultralight(big), supersnow | Rocket | 5/6 → two items, one each | `Wp_Rock` |
| jungle_hench / truck / snow, jungleheli, openheli, subcopter | Missile | 7 (and 0x34 or 8) | `Wp_Rock` |

Items with the action byte but no effect list (the vanquish's racks 32-35, 41-46) are not copied
by this rule; that they are the visible rack missiles is a reading, not traced.

**Matrix** [M `FUN_00210540`]: the launch orientation rows are x = norm(up × f), y = f × x, z = f
(up = (0, 1, 0)); the position is the physics body's. The game then turns a quaternion; the port
turns the forward axis only, so the roll is not modelled. `projectile_gfx::body_matrix`.

## 2. The smoke ribbon (`Missile + 0xa0`, 0x650 bytes) [M]

- Type (`+0xa8`): Snowmobile Rocket 1, Torpedo and Remote Torpedo 2, else 0 (`FUN_00180bc8`).
  Half-width 0x334e20 = 0.3575 / 0.1 / 0.1, jitter 0x334e30 = 0.2 / 0 / 0 [ELF data].
- Feed (`FUN_001e4640`, each draw while flying): on even calls a sample takes two vertices (cap
  0x60); the newest pair is centre ± right·w with centre = pos + three draws of the shared
  generator (x, y, z: `-j/2 + bits·j/65536`); `right` is the first row of the view matrix behind
  `DAT_00333760 + 4` (which camera axis that is was not traced). After the hit only the call count
  moves (`FUN_001e43e0`); neither runs while `DAT_00339f44 == 3`.
- Draw (`FUN_001e4258`): a0 = calls rounded up to even, end = min(n, a0), start = a0 - 0x28 (0
  below 1); drawn when n > 3 and end - start > 3. So at most 40 vertices trail the projectile, the
  buffer fills after ~96 calls and the ribbon runs out about 134 calls after launch (or after the
  hit). Texture `trai` (common.ssh, 64x64 [D]); vertex colour RGB 0x80, alpha (k/12)²·30 for k < 13
  else 30 from the window start; uv = (1 for even / 0 for odd vertices, bit 1 of the index); plain
  alpha blending (`FUN_001a0f80`) [M `FUN_001e3e28`].
- Drawn for every kind but the Missile (2) and the FOD Rocket (0x1e); the Q-Pulse branch returns
  before the ribbon code (`FUN_00181960`).

## 3. Lights [M]

`FUN_001af520` keeps at most 4 point lights a frame (position, size in w, RGB; light mode
`DAT_003349e0 + 0x30 == 1` zeroes red and halves blue, its writer not traced). The list is double
buffered and cleared each frame (`FUN_001aede8`).

- **Glow** (`FUN_00181960` → `FUN_001afba8`, read by `FUN_001afd50`): type 1 Rocket, 2 Snowmobile
  Rocket, 0 every other kind; none for the Torpedo, the Q-Pulse or a spent body. Measure =
  `(2·(fps/4 - age))²` for the first fps/4 ticks, then `age`, × 1/fps. Two slots: the third request
  replaces the slot with the largest measure. Light = `Colour{type}`, size `Size{type}` × max(measure,
  1) — a launch flash 15× the size at 60 fps that is back to 1× by tick 12, growing again after one
  second.
- **Q-Pulse** (kind 0xe): once age > the homing delay, colour (0.3, 0.3, 1.0) size 4 (DAT_00378660
  [ELF data]); after the hit size 1 and the colour × linger/fps.
- **Explosion** (`FUN_001afa88`, `FUN_001afd50`): class by damage > 800 Huge, > 400 Large, > 200
  Medium, > 100 Small, else Tiny; 4 slots, a new one takes the oldest; dt = now - start, f = 2/dt -
  0.02, colour = `Colour{class}` × min(`Decay{class}`·f, 16), size `Size{class}`; off at dt >= 100.

Tuning (`Render\Lighting\<track>.tun`, `FUN_001ae630`) [D, all 8 tracks]: `Colour{Missile}` 0.4,
0.4, 0.26 with `Size{Missile}` 1.9206 (MIS13B 1.9803, MIS4/RACE 2.0201); Rocket 0.1, 0.1, 0.05 size
5; SnowmobileRocket 0, 0, 0.1 size 5 (MIS3: 0.22, 0.42, 0.48 size 0.3087); EMPInflight / EMPTarget
all zero (no traced reader indexes them); explosions colour 0.04, 0.03, 0.02, sizes 1 / 0.8 / 0.4 /
0.2 / 0.1, decays 35 / 90 / 120 / 180 / 240. The `.data` defaults differ only in `Decay{Tiny}` = 45
and the glow table [ELF data 0x333878 / 0x3338f0].

## 4. The explosion (`FUN_0015a098`, the event `FUN_00200088` → `FUN_00159fb8` builds) [M]

Fields: [0xc] the object hit, [0xe] the projectile kind, [0xf] damage, [0x10] the world hit's
surface byte (0 for an object hit), [0x13] an explicit name (0 from projectiles). Effect name, first
match: FOD Rocket → `FODHitEffect`; surface 9 → `snowspecificexplosion`; surface 6 →
`rocketsplash`; the player vehicle's object (+0x1d4) with `+0xbc` set → `Un_ExplL` (> 600) /
`Un_ExplM` (> 200) / `Un_ExplS`; surface 2-5, 10, 0x50, 0x60 → `Ex_SmallD{,2,3}` by now % 3; else
`Ex_SmB00/01/02` when the player's vehicle was hit, `Ex_Small/2/3` otherwise, cycling. Then the
radius damage (`FUN_00200200`), a scorch (`FUN_001eb078(8.0, …, pos, 5)`, decal.dat record 5), the
light, a camera shake within 20 units by min(10/dist, 1) when the player object's +0x50 is set, and
the "Explosions" sounds. Gallery gaps [D]: no gallery holds `FODHitEffect`, `rocketsplash` or
`Dummy`; gall1 lacks `snowspecificexplosion`; sav_gall1 the `Un_Expl*`; ears_gall1
`snowspecificexplosion`, the `Un_Expl*` and the two Q-Pulse effects.

## 5. Bullet streaks [M]

`FUN_0014ca60` adds a streak (start, end, BULLET_STREAK_TYPE) on the shot whose counter is 0
(`FUN_0019a9b8`, list cap 0x40). Each frame (`FUN_001993e8`): drop those born more than 60 ticks
ago; t = age/fps (t² when following a barrel in POV mode); head = start + dir·(t·Speed + Length);
removed once the head passes the end; else `FUN_001d7d78`: v = norm(camera - head), side =
norm(v × dir)·Width, tail = head - dir·Length; two triangles (h+s, h-s, t-s), (t-s, t+s, h+s), uv
from 0x426490; colour = the tun colour with each byte halved; texture `BulletStreakTexture` →
`wtrc` / `ytrc` / `otrc` / `btrc` (common.ssh 32x16 [D]); additive (ALPHA 0x8000000048); a quad
only while the texture's vertex count is < 0x46.

## Not traced / unproven

- The event dispatcher that fills the fire event (`DAT_0032e710..0x32e73c`): which `ef` entry a
  barrel's action sends is inferred from the data (the entry whose `+0x15` names the item).
- Whether the copied item's own effects (the launch `Wp_Rock` entry) play during flight; the
  scene-object effect path starts type 6 on object events, not traced for projectiles.
- The streak quad's VU expansion, the bucket counter reset, the colour word packing of
  `FUN_00143ad8`, the muzzle flash of the car guns (`GIOT::Dummy` is in no gallery).
- The view matrix row used as the ribbon's side vector, the meaning of `DAT_00339f44 == 3` and
  of the player object's `+0xbc` / `+0x50`, light mode 1.

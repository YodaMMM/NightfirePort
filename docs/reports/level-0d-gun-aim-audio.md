# level-0d-gun-aim-audio — the Chain Reaction mounted laser's beam axis and fire sound

Branch `s/level-0d-gun-aim-audio` (master @ b864d3d6 + the s/gunbeam-0d baseline merge +
origin/master @ 1a3ba0b4). Task: on 0700000d the deck-mounted laser's beam draws off axis
from the gun and the fire audio stutters; fix both from the game.

## Done

**The beam's start and direction now come from the matrix the gun draws with.**
The baseline (s/gunbeam-0d, merged here) fed the beam quad from `GunImp::muzzle()` /
`barrel_dir()` — separate trig that used the smoothed yaw **without the mount's base yaw**
(`barrel_dir` read `yaw_s` alone while the drawn `barrel_matrix` composes
`RotY(yaw_base + yaw_s)`) and added the `TurretGuns` offset's x/y components unrotated. Any
mount whose base yaw is not zero gets a beam rotated away from the drawn barrel; the offset
x/y are wrong for every non-forward-only entry. The game derives both from ONE matrix [M
GunImp_Update 0x1320d8]: the gun cel composes `base·RotY(yaw_s)·RotX(pitch_s)` (:165-172 —
the disassembly settles the decompile's ambiguity: `Concat(src, dst)` 0x1e4a90 computes
`dst = src·dst` into its second argument, and `Mat_CopyRot` 0x1e3370 memcpys only rows 0-2,
keeping the destination's translation row, so the cel rides the base translation); the fire
muzzle is `TurretGuns[+0x00]`'s 12-byte offset against that cel's matrix rows plus its +0xC0
translation (:196-211); the fire direction is `Mat_GetDir(dir, cel+0x90)` (:325) and
`Mat_GetDir` 0x1e3398 reads the matrix's third row un-negated (the disassembly). The port's
`muzzle()` / `barrel_dir()` now read from `barrel_matrix()` — the same matrix the draw poses
the gun child with.

**The fire audio holds one loop on the game's cadence.** The old code played a fresh
`play3d` of the `TurretGuns[0xb]+0x18` sound (1307 [D ACTION.ELF @0x26f520]) every cadence
tick while firing — each a new looping voice stacking under the last (the "jittery at first
then actually plays"). The game [M 0x1320d8:247-334]: on a fire tick with no handle stored
(`ext+0x98`) the laser (weapon 0x67, also 0x70) plays the sound ONCE at the imp's own
position and stores the handle (:284-291); a held trigger keeps that loop across the
cooldown ticks — the pressed+cooling path returns without stopping (:256-258, :271-274);
`Sound_Stop` + handle zero runs when the trigger releases or the hot byte is set
(:261-269), and on dismount ([M GunImp_Deactivate 0x132c18:62-63]). The 0x61 arm plays the
0x5e4 spin-up once and stores the 0x5e5 loop (:302-314) with a 0x5e6 spin-down on stop
(:262-264) and heat +8.0 per shot (:313); every other weapon plays its table sound per shot
at the muzzle (:322, :332-334). All arms ported; `GunImp.fire_handle` models `ext+0x98`.

**Overheat now recovers.** The port never drained the heat or cleared the hot byte. The
game drains heat and cooldown 1.0 per tick while positive, clamped at 0, and the heat's
floor clears the hot byte [M 0x1320d8:100-112] — with fire every tick the laser saturates
(heat > 200, :296-298) after ~3.3 s of continuous fire, stops, and recovers ~3.3 s later.
Ported in `gunimp_tick`.

## How it was verified

- `crates/nf-view/tests/0700000d_gunimp_beam_axis_live.rs` [M]: mounts the real 0d gun
  through the view's step path and, at six aim angles across the clamp range, asserts the
  beam origin equals `barrel_matrix()·TurretGuns[0xb].muzzle` (recomputed in the test, not
  through the code under test), the beam direction equals the drawn matrix's third row, the
  far tail is the 1000-unit ray [M :373-381], and the round spawns on the same muzzle
  [M :327-331]. Pins [D placements #225]: translation 137.2719/9.331467/−324.6627, rotation
  identity (`yaw_base == 0` through the sim).
- `crates/nf-view/tests/0700000d_gunimp_fire_sound_live.rs` [M]: drives `gunimp_tick` —
  the first shot starts exactly one 1307 voice; 130 held ticks (two-plus cadence cycles)
  keep exactly one; release stops it and zeroes the handle; a re-press starts it once; the
  dismount kills it; driving the heat past 200 with the trigger held stops the loop on the
  hot tick. Read through the `Sounds::active_ids` test hook.
- Existing tests re-run green on the final tree: the aim (3), beam-lifetime (1), rate (1),
  gun-draw (1) and mounted/dismount (2) tests — 8/8.
- Full suite: started in the background on the final merged tree; the tail line and any
  FAILED/TIMEOUT entries are recorded below.

**Ran:** `cargo test --offline --release -j2 -p nf-view -- --test-threads=1` over the five
gunimp test binaries above plus the two new ones (10/10 green), then the full
`tools/test/suite.py` on the merged tree.

## What's left / for the main agent

- **Visual confirmation** (no GPU here): the sim+draw math now provably share one matrix,
  but only a rendered frame can confirm the on-screen symptom is gone. If the beam still
  reads off-axis in play, the next suspect is the mounted camera framing itself — the
  game's view pitch is `−pitch_s·0.5` [M :194], so the crosshair sits at half the barrel's
  pitch by the game's own math.
- Residual [S]: an idle mount's heat does not cool while dismounted (the game drains every
  imp every tick; the port drains the mounted one). One mount in SP, so only the
  dismount-quick-remount case could notice.

## Live checks for the main agent

1. Render 0700000d with Bond mounted on the deck gun (#225), fire held, at aim
   (yaw_s, pitch_s) = (0, 0), (π/2, 0), (π, 0.3), (−2.0, −0.5), (0.7, 0.78): the beam quad
   must start at the drawn barrel's muzzle and run along the barrel's axis — report any
   visible divergence between the beam line and the gun mesh's barrel.
2. Same mounts, audio: hold fire ~3 s — the laser loop must start once and hold steady (no
   re-attack jitter), stop on release; ~3.3 s of continuous fire overHeats and the loop
   stops until it recovers (~3.3 s).

## Usage

not visible to me

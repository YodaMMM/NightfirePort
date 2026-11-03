# level-12-copter-sound-distance — report

Branch `s/level-12-copter-sound-distance` (cut from origin/master 6f16146e). Task: 07000012's
helicopter sounds near full strength from behind and away — find the game's 3D positioning for the
copter's looping sounds and the records' radii, compare with the port, fix differences, pin the
emitter's volume/pan to the game's law.

## 1. The game's chain, with addresses

**What plays the copter's sounds.** The engine is a SCRIPT sound: the copter's own ScriptPlayer
(`Copter_Create` 0x17d340 builds it from the placement's script params — 07000012 plc #970 carries
script 0x060005bd [D]; the port's SP is placement #707) starts **Sound 0x10b at script t=0,
3-D, attached to the stream-4 object** (`Script_SoundStart` 0x1d8820: the flag byte 0 →
`Sound_Play3D(100.0, −1.0, −1.0, id, obj+0xc0)` when the object exists, else the 2-D
`Sound_Play` [D decode: stream 10 t=0 `SOUND id=0x10b flat=false has_obj=true obj_stream=4`;
stream 4 creates entity 0x0200062b]). `Script_Interp` case 5 (0x1d9a08:206-233) re-posts the
position every tick: `Sound_SetPosition(handle, obj+0xc0)`. The gun is `Copter_FireGun`
0x17d828:85's `Sound_Play3D(100.0, −1.0, −1.0, 0x5df, muzzle, 0, 0, 0)` per shot.

**The radii [D].** The EE's `Sound_Play3D` (0x1dd548) stores the record defaults for −1 floats:
`SFXOutputData[id·0x18+4]` → node+0x40, `[+8]` → node+0x44; `Sound_ReqestPlaySfx` (0x1dcc68 tail)
sends **inner = outer = −1** to the IOP when no custom radii were passed, and the IOP's
`SFXStart3D` (SFX.IRX 0x3aac) resolves −1 from the bank's parameter record: inner ← bank+8,
outer ← bank+0xc. Both tables read the same for the copter's ids [D — probe12_copter_sounds dump]:

| id | role | SFXOutputData (EE) | bank effect (IOP) |
|---|---|---|---|
| 0x10b | engine loop | inner 10, outer 100, looped, cull | tracking 2 (full 3D), inner 10, outer 100, priority 100, master 100, sub0 vol 95 (no randomisation) |
| 0x5df | gun | inner 10, outer 50, cull | tracking 2, inner 10, outer 50, priority 80, master 100, subs vol 100 ±10 |

**The per-tick law [M].** `PS2_SFXCalculate3D` (SFX.IRX 0x1060c), fed per voice by
`ES_FillpsiSFXData` (0x329c: psiSFX+4 = item+0x288 = inner, +8 = item+0x284 = outer):
- d = √Σ((Δ/100)²) on the EE's ×1000 positions; att = clamp(1000 − max(0,d − inner·10)·1000 /
  ((outer−inner)·10), 0, 1000) / 10 — a linear falloff from inner to outer, in world units;
- the quadratic a2 = att·att/100;
- the front pair: FL = ((dot(left,dir)+1000)·a2/2000)·(100−rear)/100, FR with −left;
- the rear channel: rear = ((dot(−forward,dir)+1000)·a2/2000), output ×75/100 — the "behind" cue;
- mono-averaged when the mode global is 0 (the game boots `SFXSetMode(1)` [M 0x1c7e78:85]);
- `PS2_SFXSetup3D` (0x10eb8) scales the three by the combined volume (ES_CombineVolumes 0x1300,
  which squares: for the engine, sub0 95 with master 100 → (95²)/100 = 90);
- `SFXSetup` (0x4220) rejects a start beyond the outer radius when tracking==2 && priority<100
  (the engine's priority is 100, so it is never start-rejected; the gun's 80 is).

## 2. The port, compared layer by layer

- **The 3D law** (`nf-audio/src/sfx.rs` `levels()`): identical integer chain to 0x1060c — the same
  /100 scaling, inner·10/range·10, the a² quadratic, the left-axis dots, the rear ×75, the mode-0
  average. Measured on the real 07000012 bank, the port's output equals the law within 1–2 GSV
  chain exactly — with the ported fixed-point `ps2_sqrt` in the test's reference, all five
  probed spots compare EQUAL (0.340/0.340 at d20, 0.120 at d50, 0.090+0.180-rear behind,
  0.000/0.200 at 90° left, 0.000/0.000/0.000 at d120). The driver is bit-exact with the game's
  integer chain.
- **The radii**: the port's `SfxTable` (SFXOutputData, from ACTION.ELF) and the bank `Effect`
  parse (nf-formats/src/soundbank.rs) both read the stabs layout (`docs/systems/sound.md` §1.1 —
  max_reject@0x1c, ducker@0x22, ducker_length@0x24, master_volume@0x28 checked against the
  readers); the driver resolves −1 radii from the bank effect like the game's IOP.
- **The start**: the port's script-sound path (`nis.rs` op 0x16 → `Request::Sound`) resolves the
  object and starts 3-D; `drone.rs` re-posts every anchored sound's position per tick (the game's
  case 5). Verified live: the engine's Start carries `pos = Some(copter)` and ~1 Position command
  per tick while the copter flies (the probe's dump: 600 Position commands over 600 ticks).
- **The mode**: the port's Driver defaults mode 1 like the game's boot.
- **The EE pre-cull and the +0x52 near flag** (`Sound_ReqestPlaySfx` 0x1dcc68: cull-bit sounds
  don't start past outer×1.1 with 1 listener) are also in the port's EE layer.

**No divergence found in the chain.** The measured port behaviour on real disc data: engine at
d=20 in front → 34% (the law's exact integer value), d=50 → 12%, d=50 behind → front 9% with the
rear channel at 18% (the behind cue), d=90° left → the left voice silent, the right at 20%,
d=120 → silent (beyond the [D] outer 100). The gun 0x5df gets no voice past its [D] outer 50 and
plays inside it. Nothing here is "almost full strength from behind and away".

## 3. What was added (all on the branch, tests pass)

`crates/nf-view/tests/copter_sound_12_live.rs` (3 tests, NF_ISO):
- `the_copter_engine_starts_3d_and_follows_the_copter` — through the real sim: the gate 37 poke
  [D plc #970 p6], the engine 0x10b's Start is 3-D with no custom radii, and a Position command
  follows most ticks with the emitter moving >2 u with the copter.
- `the_copter_engine_fades_and_pans_by_the_game_law` — the real bank in the real driver: the
  volume and pan at d=20/50 front, d=50 behind, d=90° left, d=120 equal the game's integer law
  (replicated in the test from [M 0x1060c/0x10eb8/0x1300] with the [D] radii and start volume,
  the reference using the ported fixed-point `ps2_sqrt`): exact equality at all five spots; plus
  the law's shape as relations (falloff, rear dominance behind, the left/right dot split,
  silence beyond outer).
- `the_copter_gun_is_audible_only_inside_its_outer_radius` — no gun voice at 60 u, a voice at 20 u.

Probes (assertion-free, `--ignored`): `probe12_copter_sounds.rs` (the disc dump: placements,
script sound lines, both tables) and `probe12_engine_audio.rs` (the live command dump + the
driver measurement).

Test runs: `nf-audio` 23+1+3+1 passed / 0 failed; `nf-game --test sound_coverage_live` 5 passed;
`copter_sound_12_live` 3 passed; `nf-game --lib` 655 passed / 0 failed (run earlier this session
on the shared tree). Full suite: started in the background (`tools/test/suite.py`); its total
line is recorded below if it finished in the window.

## 4. What's left — live checks for the main agent

The reported symptom did **not** reproduce in anything the sandbox can drive, so the divergence
(if any) sits outside the sim/driver chain — the render app's audio plumbing or the report's
premise. Exact checks:

1. **PCSX2 volume probe** (turns my [M] law reading into [L]): during the 07000012 fight, with
   the copter ~50 units behind the camera, read the IOP's voice volumes for the engine item
   (`psiSFX` front-left/front-right/rear after `PS2_SFXSetup3D` 0x10eb8 — break on 0x10f2c, the
   psiSetSampleVolume* calls, or watch the SPU voice registers). Expected per [M]+[D]: front pair
   ≈ 9 %, rear ≈ 18 % (GSV of 90·9/100 and 90·18/100 ≈ 0x0E8B/0x3FFF-ish). If the real game shows
   the front pair ≈ 90 % there, my reading of 0x1060c is wrong and the port's law must change.
2. **The render's listener**: confirm the app calls `sound_tick` every gameplay frame with the
   live camera (the `NF_MPFX_LOG` env prints each listener/start — run the port at the fight and
   diff the listener positions against the camera).
3. **The stereo downmix of the rear pair**: the port's rear voices are phase-inverted
   (`Cmd::Set right = −b`, mode ≠ 0 [M 0x10eb8's rear volume is a separate SPU channel]); on a
   stereo device the Output folds them — check the fold's sign/weight (a wrong fold could make
   the engine's behind-position cue collapse or pump). `--bond` spots: one 50 u behind the
   copter's approach line and one 50 u in front of it at the fight start; expected: behind
   quieter than the d=20 reference and front-dominant, with the rear share audible on 5.1.
4. The gun 0x5df never plays in the port's gameplay today (the copter's attack case is not
   reached — `copter.rs` case 2 note), so the "gun heard too loud" half of the symptom cannot
   come from this chain; if a render shows copter gunfire audio, that is a different emitter.

## Usage

not visible to me — this sandbox shows no token/credit/context-window figures for the session.

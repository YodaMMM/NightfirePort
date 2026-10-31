# lamp-wind — the wind that keeps hanging bodies moving

Branch `s/lamp-wind` (master + `s/lamp-calib` merged). Everything below was verified in this
window; every game-behaviour claim carries its tag.

## What the game does (the chain, [M] throughout)

- `RB_Update` 0x1d6e38's force block is **buoyancy**, not heat: the gate is `obj_y < cel+0x90`
  where cel+0x90 is the **water surface** (`build_alloc_cel` 0x1c93f8 defaults it to −500.0 =
  0xc3fa0000; `parseentity_fixup_entity` 0x1d00c0 sets cel+0x90 = cel+0x54 for room class
  0xC023 — the port's verified fix-ladder doc). The force `(0, |obj_y − cel+0x54|·mass·0.4 +
  Rand_FRand(0.15), 0)` grows with depth below that surface, `Gas_Init` spawns the bubbles,
  and the shared-RNG draw fires only when the gate passes [M 0x1d6e38:0x1d6e88-0x1d6eb0].
  It is inert for the lamps: the linear impulse is consumed only by the type-0 arm of
  `RB_ApplyImpulses` 0x1d6ff8.
- The lamp mover is the **wind**: `RB_ApplyWind` 0x1d7c78 calls `Env_GetWindVel` 0x17c048 with
  the object's cel (obj+0x20, set by `build_LinkToRoom` 0x1c9680 at create) and lands the force
  AT A POINT for type != 0 (obj+0xc0 x, obj+0xc4 + rb+0x9c − rb+0x13c y, obj+0xc8 z) — torque,
  the only per-frame torque a freely hanging Rot body gets. `RB_ApplyWind` is the only
  rigid-body consumer of `Env_GetWindVel` [M decompile grep]; `GlobalEnv` serves only
  `Env_Breath` (snow) [M].
- The wind source is placement kind **0xDF** (`Env_Create` 0x17b968): a 0x50-byte wind struct W
  on the cel `build_FindCel` 0x1c8cb0 picks (cel+0x2c): dir W+0x00 = `RotMatrix(placement
  rotation)` 0x1e2368's dir row (`Mat_GetDir` 0x1e3398 = the +0x20 row; roll-independent:
  `(sin ry, −sin rx·cos ry, cos rx·cos ry)`), strength W+0x30 = target W+0x34 = param3·0.1,
  phase W+0x38 = −1.0, falloff W+0x44 = 1/(`cel+0x38` record's +0x1c radius — the record bound
  by `parsemap_block_map_data_static` 0x1d1998), grow latch W+0x4d = 1.
- `Env_Update` 0x17bc88 animates every wind cel once per frame from the ONE `Rand_Rand(20000)`
  the frame draws (no per-cel draws): the phase sweeps −1..1 (reset to −1.0 past 1.0), the
  strength doubles (`K = 2K + f`) past the target into the decay (`K ×= 1 − f`) until
  0.25×target re-arms the grow. `Env_GetWindVel` = `dir · (K·(dot(pos − cel+0x80, dir)·falloff
  + 1)·|phase|·0.5)` — the cel's +0x80 world sphere centre is the origin [M
  parseentity_transform_bounding_box 0x1d0fb0; the port's verified CelNode doc].
- The up-righting spring (0x1d79f4, `FRAME_RATE_MUL` = 1.0 [L ee_start_05.bin]) balances the
  wind torque: the lamp hangs at a gust-driven tilt instead of resting.

## The data ([D] the port's level parse; probe output in the WORKLOG)

- **07000004**: 0xDF #5 (rain, param3 = 6 → target 0.6) and lamps #39/#40 (type 1, mass 1) all
  land in room cel #1 — the game's lamps take this wind. Their cel never carries water, so the
  buoyancy gate stays shut.
- **07000006**: barrel #65 shares cel #29 with SNOW_gen #1 (param3 = 6).
- **07000001**: 0xDF #279 (param3 = 6) shares cel #0 with lamps #9-#12 — the level's own lamps
  drift too; there is no zero-wind body on any named level.

## The port

`crates/nf-game/src/rigid_body.rs`: `EnvWind` + `env_wind_create` + `env_update_gust` +
`env_get_wind_vel` (each cited above); `rb_update` now takes the buoyancy pair and the wind
vector and applies wind → evolve in the game's order; `RbObj` carries `wind`/`buoy` (new fields
on their own lines); `Rbodies::create` builds the room cels (the parse_cel_graph room half's
rules: flags16 0xc023/0xc047, the raw glist bbox), attaches the 0xDF winds (a cel already
carrying one is skipped — `Env_Create`'s early-out), picks each body's cel (the bbox test of
build_FindCel 0x1c8cb0; **first match on ties [S]** — 04/06/01 resolve uniquely) and
`tick_winds` steps the gusts. `drone.rs`: the per-frame `rand(20000)` is captured and handed to
`tick_winds` at the env slot, before the object handler [M Game_Run's order]. No new RNG draws
anywhere.

## Verification

- **Unit (nf-game, 595 lib tests green; rigid_body's 18 include the new four)**:
  `env_wind_create_parses_the_placement` (strength/target/phase/dir row), `gust_cycle_grows_and_decays`
  (the doubling, the decay, the −1.0 reset; sister: the zero-target cel never moves),
  `wind_velocity_formula` (the 0x17c048 shape; sister: no clamp behind the origin),
  `update_applies_the_wind_before_the_evolve` (the order pin), plus the renamed
  `buoyancy_puff_gate`.
- **`lamp_wind_live` (new, 2/2)**: `idle_lamps_drift_through_the_live_band_and_never_settle` —
  after the intro settles, 1200 idle ticks: the tilt spans > 2°, passes through the live 2-4°
  band at least twice, is still moving at the end, and the twin lamp moves with it.
  `zero_wind_keeps_the_lamps_bit_still` — the target-0 gate [M 0x17c048] leaves the lamps
  bit-identical over 1200 ticks.
- **Named binaries (all green after the fixes below)**:
  - `rigid_body_draw_live` 3/3,
  - `rigid_body_wiring_live` 2/2,
  - `interactables_07000004_live` 2/2,
  - `interactables_07000006_live` 3/3,
  - `interactables_07000001_live` 6 passed / 1 ignored (the pre-existing bonus-floor blocker),
  - `nf-game` — every binary 0 failed (595 lib + integration; the one earlier "failure" was a
    shell without NF_ISO, green on the real run).
- **Tests fixed per the decompile** (their zero-wind premises predate the port; each cites the
  address chain in its new comment): `rigid_body_wiring_live::resting_barrel_*` (the barrel
  drifts, bounded < 0.5°, no translation, no buoyancy draw), `interactables_07000004_live`'s
  two rest pins (bounded drift; the shot detector is a per-tick omega jump — the old !=0 check
  fired on the wind alone), `interactables_07000001_live`'s identity-rest sisters (the lamps
  hang near rest under the cel wind; the bit-still zero-wind sister moved to `lamp_wind_live`),
  `rigid_body_draw_live`'s settle band (the lamp settles into the wind band, not dead rest) and
  the p99 replay's swing-back pins (the recording's wind phase was never captured [S] — the
  phase-free form pins the spring winning; the first-swing pins 3.834/44.24° stay with the
  wind-phase caveat).
- **Full suite**: `tools/test/suite.py` was started twice in the background; both runs died
  with the sandbox's shell teardown (empty output). The totals above are from direct
  `cargo test --offline --release -p nf-game -p nf-view` runs on the final tree with NF_ISO
  set — every touched crate, zero failures. A fresh suite run was started again before this
  report; its result line is quoted in the final WORKLOG note if it lands in time.

## What's left / the amplitude gap

The port's wind **torque** is ~2× what the p99 live recording shows: the replay peaks at 52.3°
vs the live 44.24°, and the steady drift band runs ~1.9-7.7° vs the live "2-4°". Every step of
the chain matches its decompile, so the gap is likely one of:

1. the mangled point-vector slots of `RB_AddImpulsiveForceAtPoint`'s call in `RB_ApplyWind`
   (0x1d7c78) — the decompiler's stack slots for the y/z of the application point are
   unreliable; the disassembly decides (the port used obj+0xc0/c4/c8 with the +0x9c − +0x13c
   y adjust);
2. the falloff source: `cel+0x38`'s record radius vs the port's `Object::radius` (the same
   presumed source [S]; the record's authored radius is level data the port does not parse
   directly);
3. the cel pick's tie-break (build_FindCel's collision-ray pick; first-match here [S]).

## Live checks for the main agent

1. **Fresh-boot lamp tilt trace (07000004)**: from the first game frame, record lamp #40's
   matrix row 1 (obj+0x90+0x10..0x18) once per second for 5 minutes. Expected if the port is
   right: a spike above the 2-4° band in the first ~10 s (the first gust cycle starts K at the
   target 0.6 and the first grow arm doubles it — [M 0x17b968/0x17bc88]), then a slow
   oscillation. If the game never leaves 2-4°, the port's initial doubling is wrong somewhere.
2. **A break at RB_ApplyWind 0x1d7c78 on 07000004** with lamp #40: capture the vector at
   `RB_AddImpulsiveForceAtPoint`'s point argument (a2) and the wind vector (a1) for ~60 frames,
   plus W+0x30/+0x34/+0x38 (the cel+0x2c struct) each frame. Expected: wind = dir·(K·(dot·
   falloff+1)·|phase|·0.5); the point y = obj+0xc4 + rb+0x9c − rb+0x13c (verify the slots
   against the disassembly — item 1 above). The measured torque ratio settles the amplitude gap.
3. **The 07000006 barrel**: the same trace at a coarser rate — does the game's barrel visibly
   tilt (the port says ~0.06°, bounded)?
4. `cel+0x38` for lamp #40's cel: the record pointer; dump record+0x10..+0x20 (the sphere) —
   confirms the falloff denominator against `Object::radius` (item 2).

## Usage

not visible to me — this environment exposes no token, cost or context-window figures for the
session (no usage API or status line was available; nothing to copy verbatim).

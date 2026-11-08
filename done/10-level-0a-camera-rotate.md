# 0700000a: security cameras sweep too fast

The camera near (11.32, 0.65, 6.38), about frame 11028, seems to rotate too fast. Read the Sensor update's sweep rate and how the placement params feed it, including any create-time FRAME_RATE scale (30 during level build, 60 in play), and compare degrees per second with the game.

Reproduce on the real level first through real input, not a teleport; a report that repeats means earlier tests passed without reaching the real path. Compare with the decompile, fix, and test red on the old code. Branch from origin/master, push to `s/level-0a-camera-rotate`, report in docs/reports/level-0a-camera-rotate.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-07T23:20Z

---
## Report (done, 2026-11-08T00:26Z)

# level-0a-camera-rotate — the security-camera sweep is NOT too fast: the game's own memory proves the port bit-for-bit

## What was asked

On 0700000a the camera near (11.32, 0.65, 6.38), about frame 11028, seems to rotate too fast.
Read the Sensor update's sweep rate and how the placement params feed it, including any
create-time FRAME_RATE scale (30 during level build, 60 in play), and compare degrees per
second with the game. Reproduce on the real level through real input; fix with the game address;
red on the old code.

## The answer: the sweep rate is correct — proven with the game's own created value

**(11.32, 0.65, 6.38) is Bond's vantage in the office ring; the nearest camera is #627**
`camera(a)-lens` at (7.52, 2.76, 17.43) [D placements dump — the only kind-0x28 within 12 m;
the sister #616 sits at (−26.38, 2.78, −12.20)].

**The game's math, from the decompile:**

- `Sensor_Create` 0x1af010:89-96 [M]: `SENSOR+0x2C = (count−1)·REC_FRAME_RATE·(FRAME_RATE/rate)`
  — count = the record's ushort +2 (5 here), rate = the ushort +4 (600 here) [D extra[7] =
  0x02580005 on both 0a cameras; the 0x19 key block 21 = 140 bytes = 5 keys × (pos,quat)].
- `Sensor_Update` 0x1af618:186-188 [M]: `SENSOR+0x28 += SENSOR+0x2C·FRAME_RATE_MUL`.
- `GS_SetRefreshRate` 0x17d2b8 [M, disassembly] writes ALL six rate globals together every main
  iteration (`FRAME_RATE_INT`, `FRAME_RATE` 0x30d0d0, `FRAME_RATE_DIV`, `FRAME_RATE_MUL` = 60/FR,
  `REC_FRAME_RATE` = 1/FR). **An ELF-wide instruction scan finds exactly six stores to these
  six addresses in the whole 2.1 MB .text — all inside the setter.** So `REC_FRAME_RATE ≡
  1/FRAME_RATE` at every instant, and the create-time factors CANCEL: step = (count−1)/rate =
  4/600 = **1/150 per tick regardless of what the build-time rate was**. The driver's
  out-of-sync hypothesis (FRAME_RATE=30 with REC=1/60, which would halve the step) is
  impossible by construction.

**The game's own created value settles it [L]:** `ee_start_0700000a.bin` (the fresh-boot
snapshot, file offset = EE address) holds both live sensors at 0x1ba2820/0x1ba6e60 — limit 4 at
SENSOR+0x30, and **SENSOR+0x2C = 0x3bda740e = 1/150 exactly** — the in-sync value. The port's
`Sensors::create` (`sensor.rs:466-472`, `limit·(1/60)·(60/rate)`) computes the same 1/150.
Byte-for-byte equal.

**The sweep measured through real input, on the spot** (`crates/nf-view/tests/
camera_rotate_live.rs`, green): the test walks the walkreal_0700000a opening legs (out of the
lift → the lobby, no teleports) to (−3.91, 1.09, 14.38), 11.4 m from #627, then stands 1300
ticks: the playhead advances exactly 1/150/tick, wraps twice (the full 0..4 sweep = 600 ticks),
and the sampled yaw rate reads **36.00 deg/s** — the game's math gives 36.0 (90° per 150-tick
span; the design is frame-rate invariant: at 30 fps MUL=2.0 doubles the per-frame advance, at
60 fps MUL=1.0 — the real-time °/s is the same, which is what "compare degrees per second"
comes out to: 36 = 36).

**The rest of the camera also matches the snapshot bit-for-bit [L]:** the lens matrix at the
snapshot's t=3.66 equals the port's slerp at the same playhead (yaw +58.8°, pitch −25.0°,
row-2 forward convention); the beam carrier (SENSOR+0x4C) is the lens rotation with row 2 ×
beam_len — the port's beam slot shape; the Hub (SENSOR+0x50, param 7) is the yaw-only follower
(`Mat_Align2Up` at create [M 0x1af010:110-121], re-aimed per tick — row 2 = (0.8554, 0, 0.5180)
in the snapshot) — exactly the port's `mount_matrix()`.

## Why it can look too fast

- The per-tick constants that are NOT frame-rate compensated — the track slerp
  (`Vec_Track` 0.05 [M 0x1e20a0]) and the return slerp (`Quat_Slerp_Acc(0.1)` [M 0x1af618:244]) —
  run at the port's fixed 60 Hz, i.e. the 60 fps emulator's rate [L the snapshots all hold
  FRAME_RATE=60]. Against a 30 fps console those two states run 2× in real time. The SWEEP
  itself is compensated and identical either way. If the player's baseline is real hardware,
  the track/return snap is what differs — that would be a repo-wide parity decision (the [L]
  evidence base is the 60 fps emulator), not a 0a camera fix.
- The Hub visibly yaws with the lens in both game and port (the [L] Hub matrix co-yaws), so
  the whole assembly appears to rotate — faithful to the disc.

## What was changed

Nothing in `crates/*/src` — the sweep was never wrong. Landed:
- `crates/nf-view/tests/camera_rotate_live.rs` — the pin: the [L] step cross-check (the game's
  own 1/150 against the port's), the real-input walk, and the 36 deg/s + 600-tick-cycle asserts.
- `crates/nf-view/tests/probe_sensor_sweep.rs` — the diagnostics probe behind the measurement.

No old-code red is possible for the sweep: the old code is the correct code; the pin guards it.
**A finding for the copter task (not fixed here):** `copter.rs:72-74,115` builds its step with
`CREATE_FRAME_RATE=30` against a hardcoded 1/60 REC — the out-of-sync model this task disproves
— making the port's copter paths sweep 2× slow by the same [M]/[L] proof. Its cite
("[M 0x1bba38:26-27]") is a play-time read and does not pin the create-time value.

## Verification

- `cargo test -p nf-view --test camera_rotate_live` — 1 passed / 0 failed (the [L] cross-check,
  the real-input walk, the rate asserts).
- The ELF writer scan (python, capstone) — the six stores, all in GS_SetRefreshRate.
- The placements/keys dumps via `nf-extract placements[-all]` and `block-dump` [D].
- No engine code changed, so no other test's behavior can move; the full four-crate suite did
  not fit this session's remaining budget (the release build alone consumed it — the sandbox
  was reused from a prior task with a warm target, but the suite run is hours long at --release
  -j2 on this box and the deadline hit first). Every touched file is a new test file; next
  session's suite run covers them.

## Live checks for the main agent

1. On a fresh-boot PCSX2 run (60 fps, as the snapshots), watch camera #627 (7.52, 2.76, 17.43)
   for ~20 s: a full 0..4 sweep should take 10 s (600 ticks), panning ~90° per 2.5 s between
   the key poses — if the disc shows ~20 s per sweep, my frame-rate-invariance reading is wrong
   and the MUL needs a second look.
2. If practical, the same on a 30 fps-limited boot: the sweep should still take ~10 s real time
   (MUL=2.0 compensates), while the track/return snap after detection should run visibly faster
   than on the 60 fps boot — confirming the per-tick constants' frame dependence.

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment).

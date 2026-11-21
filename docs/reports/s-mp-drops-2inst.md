# s/mp-drops: the two-window online check (host + joiner on one PC)

No change here moves the shared random stream: the changes are env-gated test aids in the app
(`crates/nf-view/src/lib.rs`), a script and an opt-in test. No game logic and no draw code changed.

## Harness

- `tools/mp/two_instance.py`: runs the real exe twice (`--net-host <port> <arena>` and
  `--net-join 127.0.0.1:<port>`), each in its own working directory (captures land in
  `<dir>/nf-shots`), LAN-only (`NF_NO_REACH=1`: no UPnP / STUN; `NF_LOBBY_SERVICE` empty: no cloud
  listing), silent, killed after `--seconds`. Per-window env via `--host-env` / `--join-env`.
- New aids in `lib.rs`:
  - `NF_NET_DROPLOG=<n>`: every arena pickup on match tick 1; then each dead player's drop (index,
    id, kind, rounds, state, lifetime left, position, spin heading) every n ticks and on each
    state / gone change, plus both human slots' shared gate (held record, owned ids, clips, carried).
  - `NF_NET_AUTOPICK_ONLY=drop|<id>[,<id>]`: the existing `NF_NET_AUTOPICK` walk steers only at a
    dead player's drop, or only at pickups of those ids.
  - `NF_MP_PLACE="<slot>:x,y,z[;…]"`: those bodies stand at those points at the match start, on
    every machine (as `NF_MP_DUEL` does), so a far pickup is a short walk.
  - `NF_NET_AUTOPICK`'s pitch correction now uses a small gain: with the lockstep input delay the
    full-gain correction swung the view between the floor and the sky.
- `crates/nf-view/tests/mp_two_instance_live.rs` (ignored, `NF_TWO_INSTANCE=1` + `NF_ISO`): the
  death-drop run below as a pass/fail check (same creation tick, rounds and gone tick on both
  windows; gone exactly 1800 ticks after the death; the landed drop turns).

## What both windows showed (arena 07000023, deathmatch, 2 humans)

Logs and captures: lane `zz_2inst/run1`..`run5` (not committed).

1. **Death drop, expiry (run1).** The host shot the joiner (P99 in hand, 16 in the clip). Both
   windows: `slot 0 killed slot 1 (frame 181)`; drop 20 created on tick 181, id 6, kind 0,
   rounds 16, lifetime 1800, spin on, falling (state 0); landed (state 1) at y 0.943 on tick 217;
   the heading turned by 1.000 rad every 60 ticks (1/60 rad per tick); gone on tick 1981 on both
   windows, exactly 1800 ticks (30 s) after the death. Every drop line is identical on both.
2. **Taken by the other player (run2, run3).** After the joiner respawned, the host walked onto the
   drop. Both windows on the same tick (903 in run2, 766 in run3): the drop gone, the host's shared
   gate carried P99 rounds 16 → 32 (the host already owned the P99, so the take is ammo), the
   joiner's gate unchanged.
3. **P99 take online (run4, run5, weapon set Pistols, spawn P2K).** Host `[take-net] … weapon 0x6
   kind 0 → Weapon { weapon: 6 }`; both windows' gate 0: owned `[1, 2, 6]`, held 6, clip 6 = 10.
   Record 6 is owned; the P2K (2) is only the spawn weapon.
4. **SMG take online (run5, joiner placed beside an SMG pickup).** Joiner `[take-net] … weapon 0xa
   kind 0 → Weapon { weapon: 10 }`; both windows' gate 1: owned `[1, 2, a]`, clip 0xa = 18. Record
   10, never 0xC. The joiner kept the P2K in hand: `BEST_WEAPON` ranks 0x0A below 0x02, so the
   take does not switch to it (the table's order, not a fault).
5. Lockstep held through every run (no desync lines); the bodies were visible in both windows.

## Found: the dropped gun was not drawn (fixed on s/mp-drops-draw, docs/reports/s-mp-drops-draw.md)

In the real app the death drop is invisible on both windows. Run3 turned the dying player toward
the host (`NF_MP_FACE_TURN=…,1:3.14159`) so the drop lands in open floor 4 m in front of the
host's still camera: the host's captures from the death to the take (ticks 210..680, `run3/host/
nf-shots/001..011`) show the body fall and vanish, and an empty floor where the sim holds the
landed, spinning drop (-14.227, 0.943, 6.699) until the host takes it on tick 766.

Cause (read, not changed): the scene builds pickup draw instances once at load, for the placed
pickups (`placed_pickups` → `add_instance(…, Some(pi))`, scene.rs ~10653) and for drone drops
through the drone's `dropped` attachment (`p.from == Some(drone)`, scene.rs ~12203). The MP death
drop is appended at run time with `from: None` (`Sim::mp_death_drop`), so no draw slot ever
points at it. The game draws every pickup object (`View_DrawObjectGlist` 0x1e6780 draws
`obj+0xD8` at `obj+0x90`). It needs a draw slot for run-time pickups (a pool per weapon model, or
a reserved slot set). s/mp-drops-draw reserves the slots.

## Also seen, unproven

- The drop starts about 0.9 m behind the eye: `rest_gun_matrix` (and the view model's
  `gun_matrix`) put the gun object's origin at head + R·(`+0xE0` offset + (0, 0.2, −0.5)) + (0,
  −0.2, 0.5)·R_view, and the P99 record's `+0xE8` is −0.9026. `Player_PositionGun` 0x1a3b40 builds
  the translation the same way from GUN+0x30; the writer of GUN+0x30 was not traced here, so
  whether the game's drop also starts behind the head is not settled. In run2 that put the drop
  behind a low wall from the shooter and the take took a detour.
- The second-SMG swap for a dual-wield character (10 → 0xD) was not exercised online (no aid sets
  the character).
- Nothing here was compared against the real game live.

## Tests

- `mp_two_instance_live` (opt-in, two windows): passes. Both windows: drop created on tick 201
  with 16 rounds, gone on tick 2001, the same seven landed headings in step.
- Release build of `nf-view` (movies) for every run above. No game logic changed, so no other
  binary was re-run.
- Seen twice, not explained: the test's first version launched `target-o/release/nf-view.exe`
  in place and the host window panicked at startup (`create_buffer_init` "morph targets": buffer
  invalid). The same exe copied to the run folder (as the script always does, and the test now
  does) never failed in nine pairs. Not chased further; the machine had heavy GPU load from other
  jobs at the time.

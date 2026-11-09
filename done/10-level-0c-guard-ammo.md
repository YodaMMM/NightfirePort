# 0700000c: guard #1's rounds at the ring

On the current tree 0c guard #1 keeps firing at the ring where he used to run dry. Read the guard's clip/reload/ammo rules for his weapon (drone weapon record, reload state, whether guards have finite ammo) in the decompile and pin his rounds over a fight through real input; fix the port if he fires more or less than the game allows.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0c-guard-ammo`, report in docs/reports/level-0c-guard-ammo.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-09T06:44Z

---
## Report (done, 2026-11-09T08:12Z)


# level-0c-guard-ammo — 0700000c guard #1's rounds at the ring

Branch `s/level-0c-guard-ammo`, cut from `origin/master` (63b05cca6). Task: on the current tree
0c guard #1 "keeps firing at the ring where he used to run dry" — pin his clip/reload/ammo
rules from the decompile, reproduce his rounds over a fight through real input, fix the port if
he fires more or less than the game allows.

## Result: the symptom does not reproduce — the port already matches every rule I can cite

Through real input from the level spawn (no teleport), in two choreographies, guard #1's ammo
bookkeeping on the current tree matches the decompiled game rules exactly. New test
`crates/nf-view/tests/guard1_rounds_0700000c_live.rs` (2 tests, both green).

## The decompiled rules (all [M], work/decomp/ACTION.ELF)

Guard #1 = drone list index 1 = placement #3 (skin 0x5000093, **weapon param 9 = 0x1e**,
dtype p15 = 5, mode_normal p16 = 0, at (13.04, 12.17, −198.00) [D docs/levels/0700000c-audit.md]).

1. **Clip 5 / reserve 5 at init**: `NDrone2_DefaultInit` 0x14b300 keys the clip switch on the
   weapon param (uVar18 = piVar19[0x16]); `case 0x1e/0x1f/0x24/0x25: uVar17 = 5` →
   `+0xBBC = 5, +0xBBE = 5` (0x14b300:345-348).
2. **Every shot decrements the clip**: `DroneWeap_FireWeapon` 0x178f68:270
   `*(short*)(drone+0xBBC) -= 1` (all weapons).
3. **Reload at clip < 1**: `DroneFunc_CombatState` 0x148338:196-197 (case 0x10)
   `if (*(short*)(drone+0xBBC) < 1) return 0x69` (AimStandReload).
4. **Reload ENTER refills the clip from the reserve**: `NDrone2_DSTATE_AimStandReload`
   0x1666d8 case 1: `+0xBBC = +0xBBE`. **The reserve is never decremented in the campaign
   path** — the only +0xBBE writers are DefaultInit and the MP-gated BOTWEAP layer
   (BOTWEAP_ReloadAmmoType 0x129d28 / changeWeapon 0x12a028, called only under
   `MPSettings._388 != 0` [M NDrone2_PostLoad_Init 0x14d890:22-24]). A campaign guard's
   5-round clips reload forever.
5. **The 0x1e "finite ammo" branch does NOT apply to guard #1.** FireWeapon 0x178f68:94-99
   checks the MODE byte `*(char*)(drone+0xC5) == 0x1e` and then gates the shot on
   `BOTWEAP_AmmoInGun(drone, *(u8*)(botvars+0x768)) < 1 → return`, decrementing
   `BOTWEAP_decrRounds(drone,1)` per shot (0x178f68:271-273). Guard #1's mode byte is
   `GetDTYPENEW(dtype 5, block 0)` = 0 [M NDrone2_GetDroneTypeAttackTypeFriend 0x14cbd8:23-27
   + NDrone2_GetDTYPENEW 0x14c938 tail: case 5 is not level 07000007 → break → block-0 tail
   returns 0], so the branch never runs for him. (The port models this mode chain in
   `drone.rs type_setup`/`dtype_of`.)

**Where the game's finite guard ammo actually lives** (the "drone weapon record" question):
the BOT_vars rounds array `+0x144+w*0xC` [M docs/fields/bot-vars.md:76] is linked only by
`BOT_init` (multiplayer bots, slots 4..7 — the only writer of `Drone+0xD1C` in the whole
decompile). Campaign drones are `control_create_object(0xd20)` + full `memset(0)`
[M 0x1344e0:11-13], `Drone_tag = obj+0x110` [M 0x1344e0:31 `__s[0x38] = __s + 0x44`], so
campaign `Drone+0xD1C` = NULL. Any guard whose MODE byte is 0x1e (dtype 0x10 via GetDTYPENEW,
or mode param 0x21 via `DroneModeSettings[0x21] = 0x1e` [L ELF bytes @ 0x29AD40 row 33]) would
run that branch against NULL — the game reads/writes low EE memory (`*(u8*)0x768` = 0 and
u16@0x144 = 0 in every fresh EE snapshot [L work/ee_start_07000001.bin / ee_start_05.bin /
ee_snapshot.bin]) — i.e. such a guard could never pass the `AmmoInGun < 1` gate at all. **No 0c
placement reaches it**: every 0c kind-0x0f row is dtype 5 or 1 with p5 = 0x64 (the
DoModeSettingsNEW path), so no 0c guard's mode is 0x1e.

## The reproduction (real input, both choreographies)

`guard1_rounds_0700000c_live.rs` walks the shared 0c prologue to the yard ladder, climbs it
with the real-input legs (copied verbatim from `walkreal0c.rs::walk_spawn_to_the_band`), walks
the patrol-ring legs, then holds Bond at the ring west edge sampling guard #1 EVERY tick
(clip, shots, state).

- **Top-up walk (unbounded observation)**: guard #1 fired 11 rounds total — exactly 5 per
  clip, each reload entered at clip 0 right after the 5th shot, refilled to 5 on the ENTER
  tick, reload spans **185 ticks** each (two of them), inter-shot gaps 161-311 ticks (the
  first engagement 792). The gaps match the pinned 0x1e sniper law (burst 1, interval +0x40 =
  120, BurstDelay_Max 60 at ≥30 u, and the +0x270 ≥ 120 continuous-sight gate beyond 3.0 u
  [M, pinned by sniper-range-accuracy-0c-0d]).
- **Real-health fight** (`with_yard_aimed_stands` + `with_far_sniper_stands` +
  `with_real_damage`, the master choreography's stands): guard #1 (already at 10 hp from the
  pre-climb stand) fired 1 round before the ring-corner stand killed him; Bond survived at
  11.2 hp. No rule violation observable.

The test pins, with the decompile cites in the assertions: clip_max == 5; exactly 5 shots
between reload ENTERs; the ENTER sample already shows the refilled 5; no shots inside a 0x69
window; the clip never leaves 0..=5; the fight really engages him (≥10 rounds in the top-up
run). Red on the old code would be any 6-shot clip, a missing reload pause, or a clip outside
0..=5 — none occur.

**Conclusion: no fix — the port's guard #1 fires exactly what the game allows.** The 5-round
clip with a ~185-tick (≈3 s) AimStandReload pause repeats indefinitely in both the game (per
the decompile: the reserve has no campaign decrementer) and the port. If the playtest that
produced the task text saw him "run dry" permanently, that would contradict
0x1666d8/0x148338/0x14b300 — the live checks below settle it on real hardware.

## Shared-file note

`walkreal0c.rs::yard_sniper_stands` gained `pub` (one word, no behavior change) so the new
test can run the master choreography's exact stands. No other shared file touched; no
`crates/*/src/` change at all.

## Verified runs (each binary whole)

| binary | result |
|---|---|
| `nf-view --test guard1_rounds_0700000c_live` (new) | 2 passed / 0 failed |
| `nf-view --test walkreal_0700000c_live` | 3 passed / 0 failed / 1 ignored (the Agent sister) |
| `nf-view --test walkreal_0700000c_eastroute_live` | 1 passed / 0 failed |
| `nf-view --test upward_shot_0700000c_live` | 2 passed / 0 failed |
| `nf-game --test guards_0c_live` | 1 passed / 0 failed |
| `nf-game --lib` | 748 passed / 0 failed / 5 ignored |

The full 4-crate suite did not fit this session (the sandbox kills background jobs between
tool calls and each tool call caps at 10 minutes; the serial `-j2 --test-threads=1` run needs
longer than any single call). The binaries above are every test that includes the one shared
file I touched (`common/walkreal0c.rs`) plus the nf-game 0c guard suite and lib tests; no
`src/` code changed, so no other level's walkreal is affected by this branch.

## Live checks for the main agent

On 0700000c (Operative), with `gdbwatch.py` on placement #3's drone (the walkway-deck 0x1e
guard at (13.04, 12.17, −198.00)) during the ring fight:

1. **Mode byte**: read `Drone+0xC5` at spawn — expect **0** [M GetDTYPENEW 0x14c938 tail].
   If it reads 0x1e instead, FireWeapon's AmmoInGun branch applies to him and this report's
   conclusion flips (the port would need the gate).
2. **Clip rhythm**: watch `Drone+0xBBC` (w) — expect −1 on every shot; at 0 the state goes
   0x69 [M 0x148338:196-197]; on the 0x69 ENTER `+0xBBC := +0xBBE` = 5 [M 0x1666d8 case 1].
   Count his shots between 0x69 entries — expect exactly 5, and expect him to keep re-firing
   after every reload (the reserve +0xBBE has no campaign writer).
3. **Reload pause length**: the 0x69 span — the port plays 185 ticks [L this run]; confirm
   the game's pause matches (~3 s), i.e. the reload anim 0x52's length.
4. **The "run dry" observation**: if the real guard ever stops firing for good at the ring,
   catch the state he sits in and the last `+0xBBC`/`+0xBBE` values — that would be the first
   evidence for a mechanism outside the decompile cited above.

## What's left

- The full suite (see the note above) — the next session can run it; nothing in this branch
  touches `crates/*/src/`, so the risk is limited to the 0c test binaries already green.
- The live checks above, if the playtest observation was a permanent dry (the decompile says
  it cannot be).

## Usage

not visible to me — this environment shows no token, context-window or cost figures for the
session. (The one Explore subagent used reported `prompt: 210801 completion: 3439` in its own
output; the failed second subagent reported none.)

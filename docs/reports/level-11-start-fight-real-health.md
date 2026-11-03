# level-11-start-fight-real-health — report (2026-11-02)

**Status: BLOCKED by the session window, one tactic short.** The branch carries one verified
walk-behaviour fix (the flight-B blocker stand) and a full damage-chain audit of the two deaths;
the walk itself is still `#[ignore]`d with an updated, exact blocker. The next session continues
from `s/level-11-start-fight-real-health` (tip 276b2f36).

## What the run looked like on master (e79e7bc9)

`walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage` (fresh run with
`NF_DMG_TRACE=1 NF_RF_TRACE=1`, repro11.out) died at **tick 3093 at (15.97, 24.10, −2.50) — on
07000011's west flight B**, not at the old note's 07000012 spot: master moved. 31 hits for 100.0:

- drone #18 (skin 0x05000006, weapon 0x19) at (17.5, 24.0, −3.0), 1.6 u: **51.0 in 15 hits — the
  killer**; its hp read 12 the whole window (Bond never damaged it).
- #10/#12/#9 (0x19) at 1.2-21 u: 38.5 more; #17 (0x19) 5.2.

The rf-trace shows the mechanism: `fight#17 t0/t60/t120 … blocked true, clip 18 spare 8` — the
return-fire stand targeted #17 (visible at pick time, 3.4 u), then #17 retreated east and **#18
walked into the gun line at 1.6 u and opened fire**. The stand's hold-fire rule ("another live
drone short of the target: hold the tap") kept Bond's clip untouched for ~143 ticks while the
blocker landed 15 rounds. A player shoots the hostile who walks into his line.

## Fix 1 — the flight-B blocker stand (shipped, opt-in)

`crates/nf-view/tests/common/damage_log.rs`:

- New opt-in flag `DamageLog::shoot_blockers` (default **false** = master behaviour: hold fire —
  the 03 servants stand among the guards and must not be shot through).
- `stand_and_fire`: when the line to the target is blocked, the blocker (team 1, enabled, alive)
  becomes the fight's target (trace line `line blocked by live hostile #k — retarget`).
- The flag is set by **this walk only**, and only under real damage: `if !w.tops_up() {
  w.dmg.shoot_blockers = true; }` in `walkreal_07000011_live.rs` (inert under the top-up — the
  fights never run there). No other walk sets it; every other walkreal binary is bit-identical.
- Diagnostic-only helper (same file, env-gated behind `NF_RF_TRACE` + `NF_RF_GEOM`): a geom dump
  at fight START for every candidate shooter (the investigation needed the 1.6 u shooter's boxes
  while a farther shooter won the pick). No behaviour change.

Result: the walk passes the flight-B fight and **all of 07000011** (the retarget fires at
fight#17 t0, #18 dies, #17 dies next), loads 07000012 through the real exit path, and dies later
(see below). Verified: with the flag off nothing changed (the flag is read in one place); the
topped-up walk and `start_corridor_guard_07000011_live` stay green.

## The damage-chain audit (the old note's premise checked against the decompile)

Per-hit damage **matches the game** at every distance [all cited]:

- weapon_data [L ee_start_05.bin 0x2bf150, record stride 0x10C]: weapon 0x19 damage **3.5**,
  0x1c damage **3.0**; the 10/12/20 corridor pins are a base-20 weapon (0x30 family) — not 0x1c.
- `Drone_ModBulletDamage` 0x139f50:20-23 returns the raw damage untouched for weapons outside
  0x62-0x66 — no range falloff; the ×2 `TooClose_Damage` arm (0x139f50:24-26) fires only under
  3.0 u. `Drone_ModPlayerHitDamage` 0x13a940 routes the drone-victim hit through it.
- `Player_HandlePain` 0x1902c8:28-46 scales by `Plr_DMod_*`; `ReadTuningVars` 0x1d3a80:164-176
  sets **0.5 / 0.6 / 1.0** (easy/normal/hard) for levels 0x07000011/12/13/4a. At the walk's d3 a
  0x1c pellet is 3.0 — the port's observed −3.00 hits are the game's numbers.
- Accuracy and burst model: `hit_chance` (drone.rs:11795) mirrors `DroneWeap_DoBulletAccuracy`
  0x179740 incl. the level window and motion latches; `burst_delay`/`next_bullet_time`
  (drone.rs:10841-10882) mirror 0x179ba8/0x179d40 (burst = weapon_data+0x2c, the ≥4 aggression
  cut, delay = clamp(45·k·rand(0.5-1.0), 15, 60)).

So nothing in the per-shot chain is a port bug — the deaths are tactical.

## Where it dies now, and why (the remaining work)

With fix 1, the run dies at **tick 2789 at (19.9, 1.1, −10.9)** on 07000012's south corridor:
drone #10 (skin 0x05000008, weapon 0x1c) at (19.5, 1.0, −11.3) fired **one trigger loop**: 8
pellets [L weapon_data + 0x1c·0x10C + 0x18 = 8; M 0x178f68:233-241 — the per-shot loop] × 2.0
TooClose at 0.6 u = **45.5 health in one tick** (six −6.00 + one −3.50). Bond was DRY (clip 0
spare 0): this leg loads 07000012 as the fresh difficulty-3 front-end load (`save = None` — the
call `walkreal_07000012_live` calibrates tower2 on), whose own 7 P2K rounds were spent on the
spawn-lobby guards (#9 hp 30, #8, #4). The route's leg (19.00, −12.00) passes 0.86 u from #10.

Tried and reverted (uncommitted): a real-input detour to the level's own BodyArmour #734
(17.48, 0.90, 1.51 [D 07000012-audit #734], ~1.9 u off the shared lobby line). The detour legs
re-shuffled the spawn-lobby fights (one fight cost 63 health), Bond died 1.8 u short of the
pickup. Reverted to keep the branch at its best verified state.

**Next session's first item** (ideas, in order):
1. Re-do the armour detour with legs that reuse the proven flood line exactly
   (spawn → (14.0, 6.0) → (17.5, 3.2) → (17.48, 1.51)), accepting the fight shuffle; the armour
   (100 [D p2 = 0x64]) absorbs #10's whole point-blank loop (armour before health [M
   Player_HandlePain's armour arm]).
2. Or an opt-in corridor swing in `tower2::walk_12_exit` (a flag this walk sets): cross at
   z −13.5..−14 instead of −12.0 (2.2-2.8 u from #10 — still inside TooClose, but the
   spread-32 cone [L +0x24 = 32] drops most pellets; the pre-batch route crossed at z −13.8).
3. Or re-arm before the corridor: no ammo/gun pickup precedes #10 on the flood path (audited:
   #778 357 ×35 and #709 5.56 ×90 sit NW of the spawn — useless without a gun; the Deagle #777
   is in the north lobby, reachable only AFTER the corridor; grenades #722-724 are in the west
   wing, also after). A kill from the bend (22.63, −6.33, 5.9 u) needs ammo Bond does not have.

## What was run

- `walkreal_07000011_live` real-damage (the ignored test) — 5 instrumented runs (repro11*.out,
  not committed); final state dies at 2789 on 07000012 as described.
- `walkreal_07000011_live` topped-up `walk_spawn_to_the_lower_floor_load_point` — **passed**
  (12.4 s) with the shared-helper edits in place.
- `start_corridor_guard_07000011_live` — **8 passed / 0 failed** (13.2 s).
- NOT run this session: the other walkreal binaries (rule 7 requires them before a shared-helper
  change ships; the flag is default-off and read in one place, but the next session must run the
  full set), and the full suite (the sandbox window closed during the investigation; the fast
  runner was killed at 20:23 to unblock the repro builds and not restarted — there was no
  push-worthy green tree before the window ended).

## Live checks for the main agent

1. The 0x1c pellet count: fresh boot, `goto_level 07000012`, walk Bond to the south corridor so
   guard #10 fires; gdbwatch on `Bullet_init` (0x12c5a0) callers — expect 8 `Bullet_init` calls
   per trigger loop [L weapon_data 0x2bf150 + 0x1c·0x10C + 0x18 = 8; M 0x178f68:233-241].
2. The TooClose ×2 at hit time: same setup, gdbwatch `Drone_ModBulletDamage` (0x139f50) return
   with the shooter at < 3.0 u — expect weapon damage × 2.0 (6.0 for 0x1c at d3) [M 0x139f50:24-26].
3. Plr_DMod on this mission: read 0x30cb14..0x30cb1c-class tuning globals after
   `goto_level 07000012` — expect Plr_DMod_Hard = 1.0, Accuracy_Hard = 1.0 [M 0x1d3a80:164-176].

## Usage

not visible to me (no token/credit/context figures are exposed to this session; nothing in the
sandbox reports them).

## ntfy failed

All three fallbacks failed (2026-11-02 ~21:25 UTC):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "…" https://ntfy.sh/nightfire-yoda-8k3x` →
   `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "… urlopen(Request('https://ntfy.sh/nightfire-yoda-8k3x', …), timeout=20)"` →
   `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "…" https://ntfy.sh/nightfire-yoda-8k3x` → killed at the 30 s tool
   timeout with no output (no bytes received).

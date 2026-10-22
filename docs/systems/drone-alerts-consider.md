# Drone-to-drone alerts: `DroneVision_FindAlertedDrones` → `DetermineAlertCheck` → `ConsiderAlerted` → `DroneCanSeeDrone`

Task consider-alerted-exact (915). The port's stand-in (drone.rs's "Drones noticing drones that
attack" pass) approximated this chain; the exact rules are here. All facts [M] = decompile
(ACTION.ELF, address = filename prefix, `:NN` = line in the .c file) or the disassembly of
`DroneVision_ConsiderAlerted` 0x176250 (1192 bytes, checked instruction-by-instruction at
0x176250-0x1766f4); [L] = the `StateFuncs` table at 0x29b300 read from `work/elf/ACTION.ELF`
with `work/action-symtab.tsv` names.

## The chain

`Drone_InitComms` (0x139520) runs `ProcessDroneSight` per frame, whose tail calls
`DroneVision_FindAlertedDrones` (0x1767d0). It walks the drone list (head `DAT_00298E14`,
`+0x04` next) as pairs with a rolling cursor (the pair ids `DAT_00299098`/`DAT_0029909C`,
stored back at :118-125), asking `DroneVision_DetermineAlertCheck` (0x1766f8) per pair. It
counts the returns (`uVar4`) and a per-frame pair budget (`DAT_00298E1C`, `uVar11`), and stops
scanning once the count passes 4 (`if (4 < uVar4) bVar2 = true` [M :75-80]).

`DetermineAlertCheck` (0x1766f8) defines `qualify(X) = (+0x4F8 & 0x20000) == 0` (not
yet alerted by a drone) `&& (+0x4F8 & 0x100) != 0` (enabled) `&& +0x44 != 2` (team)
`&& +0x4FC != 0` (has alert bits) [M :12-31]. Exactly one of the pair must qualify — it becomes
the **source**; the other becomes the **observer** [M :36-48]. Both qualify (two attackers) or
neither (nothing happening) → the pair is skipped.

`DroneVision_ConsiderAlerted` (0x176250, `(observer Drone_tag*, source Drone_tag*)`) then
decides, and `DroneVision_DroneCanSeeDrone` (0x1760b0) supplies the sight test.

## ConsiderAlerted, exactly

The decompile has `unaff_s0/s1/s2` spill noise; every gate below is verified against the
disassembly (0x176250 + offset).

1. **Observer flag**: `+0x4F8 & 0x2000` (may-notice) must be set and `+0x44 != 2` [M :35,
   0x17626c-0x176284]. The flag is set by the ENTER arm of exactly 31 "unaware" states
   (`state::UNAWARE` in drone.rs — each DSTATE's case 1, e.g. Idle 0x15bf18:31, PartyGirl
   0x160140:28) and cleared on EVERY state change by `Drone_SM_RouteMsgDCV` (`& 0xffffdfff`
   [M-dis 0x172060-70]).
2. **Mode**: observer `+0xC5 == 5` (the Hostage type [D dronetype-tables.md:87]) refuses [M :40,
   0x17628c-0x176290].
3. **The observer's own live gate** [M :43-57, 0x176298-0x176300]: the universal eligibility
   gate applied to the observer itself (drone.md Group 1) — `(+0x4F8 & 0x600) == 0`,
   `(+0x4F8 & 0x100) != 0`, `0.0 < +0xAC` (health), `obj+0xFF != 0x11`, `obj+0xFE & 1 == 0`.
   (The observer's `+0xC` is its own obj [M drone.md:168]; the "no obj" arm returns 0 — dead
   code for a live drone.)
4. **Observer-state ranges** [M :58-86, 0x17630c-0x176364]: refused are `0x39`/`0x3a`
   (InterogateAssist/Wait), `0x63`/`0x64` (GoToGoalPosition/HostageGoToGoalPosition) and
   `0x85` (EnemyRunToPoint) — names [L 0x29b300 + symtab]; state `3` (PlayScript) refuses when
   `1.0 <= +0x50C` (alertness) [M :62, 0x176368-0x176384]; state `0x1b` (EnemyMission) refuses
   on level `0x07000014` [M :66, 0x176390-0x17639c]. Every other state passes — the aware
   states are excluded by gate 1 (the flag clears on every change), not by this list.
5. **Once-latch**: `+0x4F8 & 0x20000` set → return 0 [M :87, 0x1763a8-0x1763b0].
6. **The source's 1-second gate**: `(float)GameState._52_4_ < (float)source->0x524 +
   FRAME_RATE_DIV·60.0` → return 0 [M :91, 0x1763b8-0x176454; the frame constant is
   `FRAME_RATE_DIV` [L 0x30d0d4 = 1.0]]. `+0x524` is the once-stamp `if (+0x524 == 0)
   +0x524 = frame` written by `FirstAttack` 0x148010, `SetAsAttacking` 0x14aed8 and
   `BOTWEAP_InitWeapon` 0x129e48 — a drone that never attacked holds 0, so the gate only bites
   in the level's first second (or the first second after a spawner's frame-0 stamp).
7. **0700000c/0d speech arm** [M :94-104, 0x17645c-0x1764cc]: on levels `0x0700000c`/`0x0700000d`
   with the observer's voice set `+0xD8` in {5, 6} (`u16 − 5 < 2`) and
   `Drone_IsTalking` 0x149798 false → `NDrone2_SpeechSFXd(observer, 0xd0, 0, 0, 0)` — per
   considered pair, before the accept chain (not gated by it).
8. **07000007 sight widening** [M :106-114, 0x1764d0-0x17650c]: on `0x07000007` while
   `DAT_0029AA86` (the interrogate/party latch — the port's `Drones::party_loud`) is non-zero,
   the observer's `+0xE8 = 20.0` and `+0xE4 = π` (0x41a00000 / 0x40490fdb). Nothing un-widens
   it; it re-applies per pair while the latch holds.
9. **The bit-accept chain** [M :119-193; shape from 0x176514-0x176668] — accept iff ANY of,
   with the source's `+0x4FC` bits `b` and the OBSERVER's props (`behaviour_util_getProperty`
   on the observer's `+0x4D8` block, 0x1c7d98):
   - `b & 0x10` (attacking) — unconditional, no prop [M 0x176590-0x176594 `bnel … 0x17666c`];
   - `b & 0x8 && prop(0x29)` [M 0x17656c-0x176584];
   - `b & 0x8 && prop(0x26) && b & 0x2 && prop(0x27)` — path A's prop-0x26 arm only accepts
     through the bit-2 sub-chain (0x17652c-0x176560); `bit 8 & prop 0x26` alone does NOT accept;
   - `b & 0x2 && prop(0x27)` [M 0x17659c-0x1765b4];
   - `b & 0x4 && prop(0x4f)` [M 0x1765c0-0x1765d8];
   - `b & 0x1 && prop(0x28)` [M 0x1765e4-0x176608 + the final fallback 0x17664c-0x176664 — the
     0x20 early-exit at :146 is subsumed, the net rule accepts `bit 1 + prop 0x28` regardless
     of `b & 0x20`];
   - `b & 0x10000 && prop(0x28) && b & 0x200000` [M 0x176614-0x176640].
   No accept → return 0 (the `b & 1` clear at 0x17664c-0x176650 and the `prop(0x28)` clear at
   0x176664 are the two exits — both return 0).
10. **`DroneVision_DroneCanSeeDrone`** (0x1760b0) — `(observer, source, u8* out)`:
    - `Vec_SqDist3D` between the two drones' own objects (`+0xC` → `obj+0x30` [M drone.md:168]);
      `2500.0 < sq` → 0 (:45) — the 50-unit bound;
    - magnitude `< 2.0` → always sees (:54);
    - else `ATAN2_APPROX(dx, dz)` vs the observer's yaw `+0x380` through
      `Vec_AngleDifference`; `+0xE4 == 0.0` skips the angle test, otherwise
      `+0xE4 − |Δ| < 0` → 0 (:62-68); then `+0xE8 < mag` → 0 (:71);
    - passing sets `*out = 1` (:78) and returns `NDrone2_CanSeeObject(observer, source obj, 5, 0)`
      (:80) — the head-to-head LOS (the port's `CollisionWorld::line_of_sight` on the head
      points).
    ConsiderAlerted calls it at :195 (0x17666c-0x176674) and — note — its RETURN VALUE is the
    cone byte `auStack_50[0]` in every path that reaches the call (:197 returns it when the LOS
    failed, :210 after the effect). `FindAlertedDrones` counts that byte, so a cone-pass without
    LOS still burns one of the five scan slots while producing no alert.

## The effect [M :199-208, 0x176680-0x1766d8]

On full sight (cone + LOS), IMMEDIATELY (mid-scan — a just-latched observer cannot re-qualify
for a later pair, and the latch is set before the message):

1. `+0x4F8 |= 0x20000` (the once-latch) [:201];
2. `copy = source->0x4FC << 16`; on `0x07000007`: `copy |= source->0x4FC & 0xffff0000` — the
   high half (the bits the source itself was alerted with) rides along only on the party level
   [:202-204];
3. `+0x528 = source` (the assist pointer — the port's `assist_src`) [:206];
4. `+0x4FC |= copy` [:207];
5. `Drone_Message(observer obj, 0x1f, 0, 0)` [:208]. The unaware DSTATEs' case-0x1f arm is
   `DroneFunc_DoForcedAttack` (Idle 0x15bf18:129-130 and siblings) [M]: gate `MPSettings+0x388
   == 0 && (level != 0x07000007 || DAT_0029AA86 != 0 || mode == 0x13 || mode == 0x16)` [M
   0x148c60:9-13], then the live gate on the drone's opponent obj (`DCVars[0]`, 0x148c60:20-35)
   and `(+0x4F8 & 0x2020000) == 0x20000` [M :37]; success clears the latch, sets
   `0x24000000`, `SetState(0x56 Attack)` [M :38-47]. A refusal leaves everything from 1-4 in
   place (the latch, the copied bits, the assist pointer persist; the guests hold the party).
   The port folds the message into its `msg::FORCED_ATTACK` send (ReactToDroneAlertMsg routes
   the alert kinds to Attack 0x56 / SeenDroneShot 0xa0 → 0x56 [M 0x1437a0:263,368 + 0x173b78])
   and keeps the 07 gate around the send, AFTER the field writes.

## Port mapping

| game | port |
|---|---|
| `+0x4F8 & 0x2000` | `Drone::watching` (now exact: cleared by `send`'s change block, re-set for `state::UNAWARE` ENTERs) |
| `+0x4F8 & 0x20000` | `Drone::alerted_by_drone` |
| `+0x4FC` | `Drone::alert_bits` |
| `+0x524` | `Drone::attack_frame` |
| `+0x528` | `Drone::assist_src` |
| `+0x50C` | `Drone::alert` |
| `+0xC5` | `Drone::mode` |
| `+0xD8` | `Drone::voice_type` |
| `+0xE4` / `+0xE8` | `Drone::half_fov` / `Drone::sight_range` |
| `+0x380` | `Drone::yaw` |
| `DAT_0029AA86` | `Drones::party_loud` (via the real interrogate: `INTEROGATE` ENTER → `interro_latch` → tick) |
| `IsTalking` 0x149798 | `Drone::voice_busy` |
| `SpeechSFXd(0xd0)` | `talks.push((0xd0, false))` |

Deviations kept (both pre-existing, noted in the pass comment): the port scans all pairs per
frame instead of the game's rolling cursor + pair budget (the ≤5 sight-pass stop is kept), and
the observer's `obj+0xFF != 0x11` / `obj+0xFE & 1` arms fold away on SP guards (class 2, no
fell-out flag [S] — speech.md §2.3).

## Tests

`crates/nf-view/tests/consider_alerted_live.rs` (NF_ISO-gated, real load + `Sim::step_bond`):
the 07 interrogating guard (856's doorway guard) alerts a party guest through the widened cone
(copy, assist pointer, forced attack), the forced-attack-refused sister keeps the latch/copy,
and the 05 sister pins the 1-second gate and the once-latch.

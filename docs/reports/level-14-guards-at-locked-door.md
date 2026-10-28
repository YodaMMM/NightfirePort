# level-14-guards-at-locked-door — report (first session, sandbox cutoff)

Task: 07000014 (Countdown Part 1). After both west fuseboxes are cut (ch
0x87/0x88 → AND #717 → 0x8C), spawners #720/#1333 start on 0x8C [D]. In the
port the player hears the guards at the locked west door for the whole
sequence and they never come through. Branch `s/level-14-guards-at-locked-door`.

## What the game does (the answer, with evidence)

**They wait at the locked door until the cascade opens it, then come through.**

- The spawners are kind 0xf1 → `DroneSpawner_Create` 0x1378d0: #720 (mode 2,
  template group 9, max alive 2, waves 4, start ch 0x8C, kill ch 0x13),
  #1333 (mode 2, group 0xd, max alive 7 (Init 0x137aa0:22-24 clamps to the
  group's 7 templates), waves 1, start 0x8C, stop 0x65, kill 0x13) [D params;
  M 0x1378d0:24-36]. `DroneSpawner_Init` 0x137aa0 consumes the group's
  template drones (their DIVars copied, the objects destroyed), and
  `DroneSpawner_SpawnDrone` 0x137cb0 creates the clone from the template's
  descriptor at the template's spot. The 11 templates stand 8-27 m EAST of the
  locked door, all with wait channel 0x8C [D audit rows].
- The clones hunt west; the route's door node parks them in
  `NDrone2_DSTATE_OpenDoor` 0x1681f8 (`NDrone2_ReachedDestNode` 0x153358:144-163,
  node flag 0x2, the door into +0xBA8, the saved state into +0x5A6).
- **The door-wait is uninterruptible**: OpenDoor's switch handles ONLY
  msgs 1/2/3/6/7/8/9/0x17/0x18/0x19 (impacts → HandleImpact), 0xC (the 2-tick
  retry: `NDrone2_OpenDoor` + timer), 0x20 (bark `Drone_SfxInfo[0x27]` = sound
  1477) and 0x21 (ConsiderExplosive) [M 0x1681f8 — the whole switch read].
  Everything else (noise, sight, alarm, shout) returns 0 → the router re-sends
  it as msg 0 to the same state → case 0 drops it [M 0x171df8:60-73 +
  0x1681f8 case 0]. The ENTER also sets `+0x4F8 |= 0x10000` [M 0x1681f8
  case 1], and the sight pass never transitions a bit-set drone — it only runs
  SeekOpponentLOS and the HaveOpponentSight counters [M disasm 0x176c60
  `lui s5,1`; the tests at 0x176ce8 and 0x176db8-0x176df4]; every state change
  clears the bit [M 0x171df8:185].
- `Door_Activate` refuses a locked door [M 0x135be0:53-63]; the guards retry
  every 2 frames with the bark on the door anim's event 6 [M 0x1681f8 case 0xC
  + 0x13b750:60-63]. The lock channel 0x85's only writer is destroyable #729
  (the generator cascade 0x51 → #726 → #1113 → #727 → #1152 → #728 → #1154 →
  #730 → #1153 → #729 death 0x85) [D]. `Door_Update` self-opens on the 0x85
  rising edge and clears the lock [M 0x1363c8:88-107]; the waiting guard's
  tick sees door state 4 and restores +0x5A6 [M 0x1681f8 case 3] — the route
  carries him through the doorway.
- Grenades: `DroneWeap_CanThrowGrenade` 0x17a798 (grenade words, Rand(2)==0,
  opponent exists, dist 8-20, anim 0x49) and `DroneWeap_ThrowGrenade`
  0x179558 (a `Bullet_init` projectile from the hand bone toward
  `Drone+0x1F0`, the opponent aim position; pitch +π/4 under 10 m, −π/4 over
  18 m). `DroneFunc_HandleExplosives` 0x1435f0 sends msg 0x21 only for a
  REGISTERED explosive object (type 0x02 in the DAT_00299e20 list) that the
  guard SEES (`DroneVision_ObjectVisibility` ≥ 0.7 + `NDrone2_CanSeeObject`) —
  a guard behind a locked door sees nothing, so the cascade never pulls the
  game's door-waiters into the grenade/scary paths.

## What the port did (measured, `probe14_guards`)

The port's spawner/clone/door-wait chain is already faithful: the guards spawn
on 0x8C, run west, park in OpenDoor (0x81) at the locked door and bark sound
1477 for thousands of ticks (od4789 in the probe) — the player's report
("guards trying to open a door"). The door stays locked until the E-use
cascade, then unlocks and self-opens (the existing cascade test).

**The defect**: ~30-40 ticks after the E press, all three door-waiting guards
left OpenDoor (into AIM_STAND_FIRE/GRENADE_THROW/STEP_AIM), threw frag
grenades at Bond (8.7 m at the E-plate), and every grenade struck the
HALF-OPEN door leaf and came to rest at the door base
((-52.72, 0.04, 38.54) etc., speed 0.17, travelled 176-189 of the 210-tick
fuse); the fuses then popped all three throwers at the threshold (health →
−121, state 0x45 death-by-explosion). Nobody ever crossed the doorway plane.
Three fixes landed; two close real fidelity gaps, but the pull-out itself
survived the probe re-run, so the acceptance (guards come through) is NOT yet
proven — that is the remaining work.

## Changes on the branch (all [M]-cited in code)

1. `OPEN_DOOR, msg::ENTER` now sets `vision_on` — the port's name for
   `+0x4F8 & 0x10000` [M 0x1681f8 case 1]; the clear side was already modelled
   in `send()`'s transition epilogue [M 0x171df8:185].
2. The SCREAM/SHOUT pre-dispatch drops for OPEN_DOOR [M 0x1681f8's switch has
   no case 0x11/0x12 → the msg-0 re-route drops it].
3. The msg::EXPLOSIVE pre-dispatch passes OPEN_DOOR's own state as the
   ConsiderExplosive resume [M 0x1681f8 case 0x21 passes `*(u16*)(SM+4)`;
   0x1431c0:191-195 stamps +0x118] — a door guard who flees returns to the
   door.

New file: `crates/nf-view/examples/probe14_guards.rs` — the measurement harness
(spawn/wait/cascade watch, the grenade dump, the OpenDoor exit watcher). It is
the tool the next session uses to bisect the remaining pull-out.

## What's left

1. **Bisect the pull-out**: the guards leave OpenDoor ~30-40 ticks after the E
   press with all three fixes in. Prime suspect: the 0x51 screen-swap blasts at
   the door ([E] (-54.1,2.6,41.6) r15.6 dmg0 etc. — 1-3 m from the guards)
   reaching `HandleImpact` case 9 through the walls (`explode_propagate` has no
   LOS test) and transitioning them out of OpenDoor despite dmg 0. If the
   game's `Explode_Propagate` 0x17c518 has a cel/LOS limit, the port needs it;
   otherwise OpenDoor's case-9 arm needs the exact HandleImpact shape for a
   0-damage blast.
2. After the pull-out closes: the real-input test (seed the fusebox channels
   through the real AND [D], the real E press, the cascade, then assert: the
   guards wait in OpenDoor barking 1477 while locked [M 0x1681f8 case 0x20],
   the door self-opens on 0x85 [M 0x1363c8:88-107], and at least one guard
   crosses x < −52.4 through the doorway after the door reaches state 4).
3. Full suite on the final tree; the total line.

## Live checks for the main agent

1. At the cascade (E pressed with the guards already waiting at the door): do
   the game's door guards leave OpenDoor? gdbwatch on one spawned guard's
   state word (Drone_tag+0x10C / obj+0xF4 mirror) around the 0x85 write —
   expected [M]: it stays 0x81 until the door opens, then flips to the saved
   state.
2. Does the game's door guard ever throw a grenade during the cascade?
   Expected [M]: no (CanThrowGrenade requires an opponent at 8-20 m AND the
   combat states; the guards sit in OpenDoor). If the main agent sees grenade
   blasts at the door base in PCSX2, my HandleImpact-suspect is wrong and the
   0x21 path fires in the game too.
3. The 0-dmg screen-swap blasts at the door: does `Explode_Propagate` 0x17c518
   hit drones through walls (no LOS/cel gate)? gdbwatch the guards' health
   (+0xAC) at the 0x51 edge — expected [L]: unchanged (dmg 0 anyway), but the
   msg-9 flinch routing is the suspect.

## Run

- `cargo build --release --offline -p nf-game -p nf-view` green (the probe
  builds and runs).
- The touched crate's tests did not run to completion before the sandbox
  cutoff — the fast runner (`tools/test/suite.py`) was started at 14:36 UTC but
  its background process did not survive the session's tool-call restarts; no
  total line this session. The next message runs the full suite on the final
  tree.

## Usage

not visible to me

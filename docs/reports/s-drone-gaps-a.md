# s/drone-gaps-a: the "not ported" notes in drone.rs lines 1-8000

Moves the shared random stream only on the 07000001 / 07000002 escort path, and only when the
escort's mission-route re-anchor lands in the unreachable arm (see the ported rule). No other
level runs the new write (voice set 9 = skins 0x0500001b / 0x0500004f only).

## Ported

`NDrone2_ReFindMissionPath` 0x153a30:134-137: in the unreachable-but-closest arm a drone whose
voice set +0xD8 is 9 re-resolves its nav path +0x954 with `AINetwork_NavPathForPosition(pos,
cel, 1)` 0x158488 (the reachable form: nearest same-cel node that `NDrone2_MoveTest` flag-0
reaches, else nearest node of any path). The old note read the field as route+0xD8; the
decompile's `iVar2` is the Drone. The nav path feeds `NDrone2_ClosestAIPoint`, so it picks where
the escort runs to hide next. New `Network::nav_path_for_reachable`; test
`escort_navpath_refind_live` (fails on the old code: nav path left at the sentinel).

## Triage

| Line | Rule | Verdict |
|---|---|---|
| 725 | Bond hit boxes: stance blend, aim pitch, climb/wire/swim scripts | too big: needs the anim blend and the other stance scripts |
| 1292 | class 0x11 BOT_respawn bot | MP only |
| 1633 | facial lip flap | visual |
| 1708 | `FindElevatorDoor` | already ported (`elevator_scan`, SniperAim / ElevatorJumper); note fixed |
| 1842 | HostageKillerAttack | state ported; `SetupHostageExecute`'s move-target / facing writes are not (a facing detail; the hostage death rides the DIE message); note fixed |
| 2709 | start-script property writes 0x14c600:130-158 | already ported (the `match p1` after `type_setup`); note fixed |
| 3317 | LocationDeathAnim yaw offset 1 | not player-facing: `CallAnim`'s arg 6 is unread in the decompile; death-anim facing is visual |
| 3527 | anim event 0xE (DIE to the +0x2B0 partner) | not player-facing: one script on the disc carries it, 07000029/060003e9 frame 1 [D census of every 07* archive]; note fixed |
| 3710 | anim-root hip tracking | motion / visual |
| 4354, 4470 | mode 0x1E bot KO / death | MP bots only |
| 4808, 4815 | MP chatter alt halves (+0x108 & 1) | not player-facing: the family is dead in both modes (`mp_gate`) |
| 5618 | KO alert status 3 | not player-facing: +0x51C has no reader in the decompile outside `Drone_AlertStatusSet` |
| 5645 | PunchImpact +0x4F8 \|= 0x10000 | already ported (`vision_on`); note fixed |
| 6574 | voice-9 nav-path re-resolve | **ported** |
| 6822 | gravity probe 0.55 / 0.3 band | motion (standing height), off limits |
| 6933 | +0x570 & 4 "no-collide" | wrong label: DoCollision / DoGravity read only & 2; & 4 gates `DoAnimation` 0x149f88 (anim update); note fixed |
| 7025 | corpse bone-based volume | too big (bone volume), corpse-only |
| 7673 | voice-0x10 +0x44 = 1 on stun recover | already ported (`team = 1`); note fixed |

## Unproven

The re-resolve was not observed on a live run: the test places the escort where no route node
is reachable and sends the ENTER directly. Whether a normal 01 / 02 run reaches that arm is not
checked.

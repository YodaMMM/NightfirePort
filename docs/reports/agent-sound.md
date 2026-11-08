# Sound call sites vs the port (s/agent-sound)

The decompile splits many `Sound_Play3D` calls across two lines, so the call sites came from a `jal`
scan of ACTION.ELF. That scan found 25 `Sound_Play`, 74 `Sound_Play3D` and 29 `Music_Event` sites,
plus the wrappers: 10 `NDrone2_PlaySFX`, 45 speech calls and 3 `BOT_soundEffect`. The per-site table
is in docs/systems/sound-coverage.md, "Re-audit (2026-11-07)".

## Ported, with tests that fail on the old code

| What | Game | Reach |
|---|---|---|
| Level music follows the fight, the zone and the cut-scenes | `UpdateMFX_02MayhewA` 0x111ed8, `02MayhewD` 0x112bf8, `03TowerA` 0x113558, `03TowerB` 0x1137b0, `PowerStationA1` 0x113c18, `PowerStationA2` 0x113fc0 | 01, 04, 09, 0a, 0c, 0d. Before this, their music never left the start section |
| Attack-gate re-seats | `UpdateMusicalEvents` 0x114e70: e[0xa], the -99 row, the bonus run-out | zone-10 rooms on 01/06/07/12; triggers on 01/03 |
| The alarm channel's combat event on every level | `DroneFunc_CheckAlarmRaised` 0x1453a8:34-42 | every level; on most it is masked by the same channel failing the mission |
| Objective chimes, hint lines and scan-lock chimes play 2-D | `Sound_Play` at 0x19c310:221/233, 0x1b3380:26, 0x1ae3d8:368/374 | all |

Tests: `music_level_updaters_live` (8) and seven `music::tests` units. Run on the old sources,
7 of the 8 live tests fail (the disc-row pin passes on both).

## Found, held back

Guards' bodies hitting the ground (op-4 kind 6 → `Effect_Body` 0x1cf400, 30 death scripts, every
level with guards) is not ported. A port with a test that failed on the old code (1 sound against
≥ 4) was written, then backed out. Headless walks never free EE pool nodes, so once the 256-node
pool is full every start returns handle 0. The extra nodes moved the point where the pool fills,
and the 07000001 00-agent walk died at tick 7682. With an unbounded pool, that walk is identical
with and without the sound (tick 11940, health 6.4). Lifting the cap in the walker instead kept
every speech handle alive, which stalled `following_mayhew_down_completes_the_level`. Freeing
nodes after the start grace broke the two harder real-damage walks on 01. It needs a harness
that ends nodes by sample length, the way the app's driver does.

`audit_07000004_live` now pins the cut-scene music jump, because MayhewD consumes the trigger.

## Not ported (reach 0 or 1)

- op-4 kind 7, the anim-driven shot: one script, 0x06000858, on 07000016. Nothing traced plays it.
- Interrogate's 0x49/0x4D/0x4E lines: the 07000005 arm is unreachable (`FirstSightState` 0x147c48).
- `Light_Update` wrap sound, `Hurt_Update` sound, the lock key/unlock sounds: dead on every disc placement.
- `Car_*`: MP arenas only.

## 07000002 dialogue cut-outs: not reproduced

Mayhew's line (0x20c, anim 0x06000503 op 1 at frame 23, stream LUT[293], 7.01 s) and the bath
scene's servant line (anim 0x06000502 / 0x06000501 op 1, script 0x0600007F) both come from the
anim-script op-1 path with the precache. A headless run of the intro, through the real driver and
an offline copy of the mixer, played 0x20c to its end with no key-off, no fade-down, no volume drop
and no voice reuse, including with the mixer's end reports delayed by a frame. Also ruled out: the
stream end markers (the decoded lengths match the marker positions, and the data past the
markers is leftover from other lines), the EE stop paths, and the per-id voice limit.

Unproven: the cause. The open question, for a live check in the app with an audio device: does the
cut happen when a STREAMS voice is reused within its key-off release (a stale end report from the
device clearing `playing` on the new voice), or only after a skip?

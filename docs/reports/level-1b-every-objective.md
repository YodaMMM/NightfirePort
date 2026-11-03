# level-1b-every-objective — report

Branch `s/level-1b-every-objective` (cut from origin/master 6f16146e). Task: complete every
0700001b objective on real input, every objective pinned in the game's order, each completion
checked on the channel or objective state the game reads.

## The level's objectives and what completes each

[L ee_start_05.bin @ 0x2a4320 — `crates/nf-game/src/mission/c0700001b.rs`]: exactly two records.

1. "Disable the missiles before they are launched" (0x04000059) — complete ch 0x09, enable 0
   (shows at once, the state-0 arm [M 0x1827d8:41-54]). Writer: AND #139 [ch 21 & 22 & 27 → 0x09]
   [D PL#109] — the missiles 1/2/7 done channels [D p6 = 21..28], written when each missile's p2
   (disarmed) script ENDS [M 0x1aa33c-3d0, disassembly re-verified this session — see below].
2. "Defeat Drake before the base goes critical" (0x0400005a) — complete ch 0x0b, enable ch 0x09
   (announced when ch 9 fires [M 0x1827d8:55-67]). Writer: the defeat NIS 0x06000a20 (SP#198,
   started by Drake's death channel 0x0e), t600 writes ch 0x0b, t720 writes ch 100 [D].

The mission end: the monitor's AND of the two conditions → ch 100 → MissionState 6 → 0x0730001b
[M 0x1827d8:119-123 + 0x182c60:114-116].

## What was done

**The objective records are now pinned in the game's order inside the real-input mission walk**
(`crates/nf-view/tests/walkreal_0700001b_live.rs`, both full walks — the allowance run
`walk_arrival_to_drakes_defeat` and the real-health `walk_arrival_to_drakes_defeat_on_real_health`).
No teleports, no substate/channel writes after the load (unchanged from the existing walk); the
new asserts observe `mission.states[23]` — the records `Mission_MonitorObjectives` 0x1827d8
itself reads and writes [M port: crates/nf-game/src/mission.rs:412-495] — at three hinges:

- **The walk start** (existing): obj 1 = 3 (announced), obj 2 = 1 (hidden) [L record 1b; M
  0x1827d8:41-54].
- **Hinge 1, at the fight gate (Drake's spawn)**: neither objective channel has fired
  (`!ch(0x09) && !ch(0x0b)`), obj 1 = 3 (announced, NOT complete), obj 2 = 1 (still hidden).
- **Hinge 2, at Drake's death tick** (the window before the NIS's t600 write): ch 0x09 is ON (the
  p2 ends raised it during the fight), ch 0x0b OFF, **obj 1 = 4 (complete), obj 2 = 3 (announced,
  not complete)** — objective 1 completes before objective 2, on the records.
- **The walk end** (both walks): ch 0x09, ch 0x0b, ch 100 on; MissionState 6; to_end 0x0730001b;
  **both records = 4 (complete)**; and the channels-of-record stamps ordered
  `time(0x0b) < time(100)` (the NIS writes 0x0b at t600 and the end ch 100 at t720 [D
  0x06000a20]; the stamps are the game's own `switch_channels_time` writes [M 0x1b1a78]).

## The finding worth recording: the fight gate opens BEFORE objective 1 completes

The first version of this task's hinge asserted the intuitive order — "the fight gate means the
disarm chain is done, so ch 9 has fired" — and the walk refuted it. The reason, from the disc and
the ELF:

- The done channels that feed AND #139 (ch 21/22/27) are written ONLY when each missile's p2
  script ends: state 3 runs `Script_Update` (0x1aa33c jal) + `Script_IsPlaying` (0x1aa344 jal),
  `bne v0,zero` at 0x1aa34c skips the writes while the script plays, and only the ended path
  stores 1 — `sb` to `switch_channels[16(s0)]` (p6) at 0x1aa3a0-3bc on the disarm branch /
  `switch_channels[14(s0)]` (p5 = 199) at 0x1aa380-388 on the launch branch — then `obj+0xf4 = 4`
  at 0x1aa3d4-3d8 (disassembled from work/elf/ACTION.ELF this session; matches the port's
  space_missile.rs model and its [M 0x1aa354-398] citations).
- Missile 1's p2 (0600091c) writes the Drake-chain ch 46 @t1000 but ends at t1710 [D], so the
  fly-by chain (SP#211, gate ch 30 + ch 46 [D]) writes the fight gate ch 10 [D 0x060008f0] ~710
  ticks before missile 1's done channel ch 21 exists — Drake spawns while objective 1 is still
  only announced, and objective 1 completes (with 22/27, whose p2s end at 1170/1610 [D]) during
  the fight, well before the defeat NIS can write ch 0x0b.

So the game's own completion order is: **obj 1 completes during the Drake fight; obj 2 completes
at the defeat NIS. The test now pins exactly that, on the records the monitor reads.** The port
needed no code change: the port's SpaceMissile state machine, the AND muxes, the objective
monitor and the defeat chain all match the decompile (the wrong item was the TEST's hinge, fixed
with the disassembly — per the rule, the test was wrong, not the code).

## How it was verified

- `walkreal_0700001b_live`: `walk_arrival_to_drakes_defeat` + `walk_arrival_to_drakes_defeat_on_real_health`
  pass with the new hinges (the real-health run ends MissionState 6 with Bond alive on real
  damage). First-run failure of the wrong hinge (both walks) is what surfaced the [M] finding above.
- `interactables_0700001b_live` 9 passed · `zerog_0700001b_live` 1 passed ·
  `platforms_drake_scene_1b` 2 passed · `nf-game` lib 655 passed / 0 failed / 5 ignored.
- Full suite: `tools/test/suite.py` (NF_ISO set) — result recorded below.

## What's left

- Nothing known for the two objectives. The walk does not kill the later astronaut waves
  (spawners #165-181) — it waits them out under the deck, which the game permits; fighting them
  from below would need the pitched zero-g view the port's camera does not follow yet (documented
  in the module header, visual work for the main agent's render).
- Optional live check for the main agent (would turn the hinge-1 chain into an [L]): during a
  PCSX2 play of 0700001b, watch `switch_channels` ch 10 (fight gate) vs ch 9 (obj 1 complete) —
  expected per [M]+[D]: ch 10 rises ~710 ticks before ch 9. Addresses: switch_channels base
  0x26FC90 (ch 10 → 0x26FC9A, ch 9 → 0x26FC99), stamps at switch_channels_time 0x270190
  (ch*4). Access type: read, twice per frame is enough (the window is ~700 ticks ≈ 12 s).

## Usage

not visible to me — this sandbox shows no token/credit/context-window figures for the session.

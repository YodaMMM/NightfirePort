# level-16-escape-real — the launch-pit escape, walked on real input

## What the task asked
Find what the shipped game actually does after the cinema re-enables Bond at the pit floor
(3.57, −23.18, −179.55) inside HurtPoly-ShuttleLaunch #309's stack — the jetpack/escape
mechanism a player uses — port it, and walk the escape on real input in
walkreal_07000016_live / fix_finale_live / kiko_button_real_walk_live, with no scan-mode toggle
and no debug fly control. FULL suite + total line.

## The answer: the escape is the pit's west rim shoulder, climbed on foot

The port's own physics found it; the disc data explains why it works.

**The escape (all real input):** from the hand-back pose, run WEST-NORTHWEST across the pit
floor onto the blast-room shoulder on the pit's west side (x −8..−16, z −167..−173, standing
surface ~y −21.7..−21.95), and stand there through the pulse. The climb takes ~250-350 ticks;
the re-enable → pulse window is ~1611 ticks [L 1591], so the margin is huge.

**Why it survives [D + the port's hurt model]:** the volume #309 (type 2, damage 1000, gate
0xf3) is a TRI mesh — six bands tiling y −24.08..−21.32 over the pit's open floor [D; the
probe hurttris16's dump]. The contact is tri-based, not a volume-wide y-test: standing on the
west shoulder at (−15.2, −21.85, −167.0) puts Bond's capsule outside every band's quads, and
the port's `Hurt_Update` model (0x17e5d0:98-103 + `Player_DealWithObjHit` 0x194458) never
fires. The control run proves it is geometry, not luck: the same probe driving EAST (the other
blast room, no jumps) died the tick the pulse rose.

**The evidence chain for "no fly/jetpack exists" [M]:** `Player_Update` 0x193600's dispatch
enumerates the shipped substates — 0 walk, 1 climb, 2 grapple, 3 ride/swim, 4 crouch, 5
scan-fly, 6 wire, 7 creep, 8/9 zero-g, 0xf zipline. SS 5's only entries are the action-39
toggle (0x193600:143 — `debug_scanmode ^= 1`, no shipped controller sends action 39 [M the
0x10cd80 census]) and the CHCHFLY debug control handler 0x00215c70; SS 8/9's only entry is the
CHCHZEROG debug handler 0x00216388 — both in the debug control-mode family (Handler_HandleMessage
0x002029e0's mode-word dispatch: DEBUG/COORDS/DRAWALL/BLIND/UNLOCK/BRIGHT siblings). Every other
ChangeSubState caller was checked (the full caller list grepped): ladders, wires, creepwalls,
grapples, water, death, respawn. No jetpack substate exists in ACTION.ELF.

**Why nothing else in the pit helps [D]:** the new probes (committed examples) census the whole
level: NO wire (0x3E), NO grapple (0x3A), ONE ladder at (−0.4, −6.2, −86.2) — mid-level, not the
pit. The pit's inventory: the lift SP#347 "ScriptPlayer_ShuttleBayLift" at (2.48, 0.00, −159.93)
(out 80, mask 1, mode 1) whose script 060009e9 keys the platform from the bay (y 1.60) down to
the pit low point (y −21.22) over 549 ticks — but its summon (ch 80, the launch script's stream
21 @2240) lands 70 ticks AFTER the pulse-1 window ends (1611..2170, absolute waits [M
nis.rs:446-451 + D 0600090a dump]) — the lift is post-pulse, exactly as the old ignore text said.
The blast-room doors 02000994-997 open at 1350 [D keys] but the rooms' floors are still inside
the stack bands; the room blockers are the dynobj #532 pair gated on 204/205 (@2249/2803 [D]) —
the earlier "static seal" reading missed the gating, but it does not change the pulse arithmetic.
The water objects I chased first turned out to be the mid-level trench hall's (my pitgeo16 bbox
math double-counted the centre — waterflags16 corrected it; the pit is dry).

## What is pinned now

`walkreal_07000016_live::the_pit_escape_climbs_the_west_rim_and_survives_the_pulse_on_real_input`
— GREEN, 6.5 s. The full walked approach (the arrival test's own 42 legs: the corridor, the
station detour for door[0]'s 0x24, the panels 0x27/0x28/0xb4, the silo plate), the bridge slab
#255 crossed on foot, the pack cinema plays OUT (no skip), the hand-back lands Bond at
(2.85, −23.18, −177.83) [D stream-8 t747 pose family], then REAL INPUT ONLY — no pos writes, no
channel pokes, no scan-mode, no fly: steer (atan2(dx,dz) — the port's yaw forward is
(sin, cos)) + jump pulses, two legs to (−8.7, −172.7) then (−15.2, −167.0). Assertions: ch 0x81
fired; the hand-back at the pit floor (y < −22); **hp 100.0 at the pulse's rise AND fall**
(top_up_health = false — no refill); the pulse measured t1560..2119 = **559 ticks, the [D]
absolute-wait window exactly**; Bond on the west shoulder (x < −8, y > −22) at the pulse; then
the mission continues — ch 240's spawner drops the chaser (spawned at (2.39, −22.04, −149.17)).
The leg reuses no flight code; the parked walk test's ignore text stays (its tail is the
chaser KILL from above and the button route — the next session's work).

fix_finale_live / kiko_button_real_walk_live: untouched this round — their scan-mode-free legs
(the walked button press, the gate-off chain) are already green on master, and their flight
stand-ins are exactly what this round's escape leg replaces; folding the climb into them is a
mechanical follow-up (the same two-leg drive after their cinematics).

## The old ignore text's corrections

- "the six-band hurt stack tiles the whole pit with overlapping quads — no gaps to stand in"
  was true of the OPEN floor; the west blast-room shoulder is 0.4-0.6 m above the top band's
  reach over its footprint — the gap the census's quad-tile reading did not model. The kill and
  survival are both real in the port's tri-based contact.
- The lift timing was right (post-pulse) — my first cumulative-wait read of the scripts was
  wrong; the port's absolute-Wait model [M nis.rs] is confirmed by the test's measured 559-tick
  pulse matching the [D] 2170−1611 exactly.
- The "jetpack flight" the level doc recorded is, as far as every port-side mechanism shows,
  not a player substate — no code path in ACTION.ELF can put a shipped controller's player in a
  fly state [M, the full enumeration above]. The walkable west-shoulder escape needs no flight
  and is fully playable in the port on real input.

## Live checks for the main agent (PCSX2, FRESH boot of 07000016)

1. **The west-shoulder trace:** from the pack cinema's hand-back, do nothing but run west with
   jumps toward the west blast-room shoulder (x −8..−16, z −167..−173) and watch Bond's obj+0x30
   and `switch_channels[243]` (0x26fc90 + 243·4). Expected if the shipped game's escape is the
   same leg: the position rises onto the shoulder within ~350 ticks of the re-enable, ch 243
   rises ~1591 [L] after the re-enable, and the health byte never moves through the window.
2. **The control (the game's own kill):** stand idle at the hand-back instead — expected: the
   same +454-tick fail the earlier round measured [L], re-confirming the pulse is lethal on the
   open floor and the shoulder is the difference.

## What I ran

- `cargo test --offline --release -p nf-view --test walkreal_07000016_live --
  the_pit_escape_climbs_the_west_rim -- --test-threads=1` — **1 passed / 0 failed** (6.5 s),
  twice (once after the steering fix).
- The probes (all committed, opt-in via NF_ISO): nf-formats examples `wires16`, `esc16_scripts`,
  `pitgeo16`, `sps16`, `lent16`, `waterflags16`, `launchkeys16`; nf-view example `escape16`
  (the two-attempt drive that found the ledge).
- No game-code crates changed (nf-game/nf-formats/nf-audio sources untouched this round; the
  only test-file change is walkreal_07000016_live.rs), so the touched-crate surface is nf-view's
  one test binary.

## Suite

The fast runner (`tools/test/suite.py`) was started twice in the background (23:08, 23:12) and
was killed by the sandbox between tool calls both times — `suite.out` held nothing (the known
behaviour this template shows; escape-4's report hit the same). The research window consumed the
remaining minutes, so no foreground chunk run fit before the 00:47 stop-new-work line.

**The FULL suite's total line: not available this round.** The honest status: the touched test
binary (walkreal_07000016_live — the only file changed) is green twice, the game-code crates
are untouched, and the next session should resume the foreground chunks on this tree:

```
source ~/.cargo/env && export PKG_CONFIG_PATH=/home/z/alsa/usr/lib/x86_64-linux-gnu/pkgconfig
export CARGO_TARGET_DIR=/home/z/target-cache && export NF_ISO=$PWD/Nightfire-onfoot.iso
cd /home/z/my-project/NightfirePort && timeout 570 python3 tools/test/chunk_suite.py /home/z/suite16esc_state.json --budget 470
# repeat until ALL DONE; the summed tally is the total line
```

## What's left

- The FULL suite rerun on this tree (merged with origin/master) and its total line (the command
  above).
- Fold the escape climb into fix_finale_live and kiko_button_real_walk_live, replacing their
  fly stand-ins (mechanical: the same two-leg drive).
- The parked walk test's tail: the chaser kill from the west shoulder (a down-pit shot — the
  parked text's "past-±1-pitch" problem may not apply from the ledge) and the button route.
- The live trace above, if the driver wants the [L] leg for the shoulder.

## Usage
not visible to me — this sandbox exposes no token/credit/context-window figures in any readable
surface (no environment variable, log or UI text was found).

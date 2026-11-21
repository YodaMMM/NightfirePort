# s/drive-ai-hostiles: the driving hostile AI and AI vehicle lifecycle

No change to the shared random stream: the only draws (the ram roll, the traffic type and
colour picks) take the driving engine's own generator (`ai::DriveRng`, FUN_0017dc28), which
nothing on foot uses. Built on `s/drive-ai-traffic` (merged in first; not on master yet).

## What a player would notice

Nothing yet on its own: this is the data and the decision logic, ported function by function,
and the viewer does not call it (nf-view was out of scope). Once wired, the driving missions
get the game's enemy cars: spawned and removed by the game's distance and slot rules, built
from their own car files (handling, seats for gunners), chasing at the game's speeds, ramming
on the game's dice roll, and firing machine guns, rockets and missiles with the game's aim
cones and reload times.

## What was added

- `nf-formats` `driveai.rs`: the track's `AIEl` (AI elements, 0x100 B) and `AICo` (AI
  commands, 0xa0 B) decode, with riders (seat, model, weapon, health) and the per-mission
  vehicle set.
- `nf-game` `driving/hostile.rs`:
  - the element database: load FUN_0012b8b0 (difficulty health scaling, slot defaults 32/6/16
    and the world `.atr` overrides), range check FUN_0012afb8 (200 in / 220 out, period 1..4),
    priority FUN_0012b440, slot manager FUN_0012bec8 (fit test, priority sort, persistent first,
    riders must fit), spawn FUN_0012cb08 (difficulty filter, sim kind 2/4, traffic clear, kill
    counter, riders), despawn FUN_0012c740 (respawnable or gone);
  - the traffic set FUN_0013fae0 (the 7 traffic car names, `MAX_TRAFFIC`,
    `NUM_TRAFFIC_TYPES_PS2`) and `AiVehicleSet` (each AI car's own pvehicle `.atr`);
  - the chase target speeds FUN_0012e808 profiles 1 / 0xd / 0xe / 0xf (the arms the traffic
    port returned `None` for);
  - FUN_00131770's ram roll (5 s, chance from AICo +0x68, 3 s ram, 1 s against the player) and
    sight rule (every 0.25 s, ≤ 200 and fewer than 2 ray hits);
  - weapons FUN_00130050 / 0x12f730 / 0x12fa28 / 0x12fd58 and the line of fire FUN_0012f530.

The per-function table (function, address, verdict) and the per-mission AI vehicle sets are
docs/systems/driving-ai.md §10.

## Tests

- `nf-game --test driving_ai_hostiles` (new, 20): pass. 19 unit tests on the game's numbers
  (difficulty switch, radii and period, held/persistent, priority key, slot passes, rider
  budget, sim kind, kill counter, despawn verdicts, henchman classes, health scaling, the near
  flag, traffic set draws, own `.atr` steering, every chase arm, ram roll, sight period, the
  three weapons' cones/reloads/channels, line of fire). One ISO-gated test over all 8 mission
  archives: AIEl/AICo counts, each mission's vehicle set with every car's `.atr` found and every
  rider seat declared in it, world slots (jungle 15 vehicles), MSet difficulty factors
  (MIS01 health 1.0/1.5/2.0, speed 1.0 everywhere), hatchback 0.2 vs vanquish 0.09, and the
  MIS01 manager spawning exactly the unheld persistent elements at start.
- Mutation check: six wrong values put in by hand (220 → 205, gun cone 0.99 → 0.95, 1 s ram →
  3 s, launcher floor 0.5 → 0, ascending sort, wait arm 0.5 → 0.6) failed 7 tests; restored.
- `nf-formats --lib driveai` (2), `nf-game --test driving_ai_traffic` (16),
  `driving_ai_follow` (5), `driving_ai_physics` (2), `nf-view --test driving_controls_live`
  (1): pass.

## Unproven / not done

- DAT_00339f4c is taken as ticks per second and DAT_00339f54 as the tick counter (writer not
  found); the priority key's player row (DAT_00333ed8 → +4 → +0x20) is read as the velocity;
  which difficulty number is which menu setting; which world `.atr` section the engine reads.
- Kind-4 vehicles (no chase-brain flag) get no AIVehicle controller; what moves them is not
  traced.
- Not ported: the pattern steering arms (0x133f60, 0x135f70, 0x137310, 0x1377c8), the pursuit
  aim point, the turret aim, the riders' own shooting, the helicopter controller, the world
  vehicle cap, and the viewer wiring.
- Live check worth doing: in Paris at the first chase, break on FUN_0012cb08 (record +0x40,
  +0x54) and on FUN_0012fa28 (+0x228 / +0x22c of the firing car).

# s/drive-hostile-facts-2: the mission script sees the enemy vehicles

No change to the on-foot code or the on-foot random stream. No new random draws.

## What a player would notice

The driving missions' rules that depend on enemies now work. In MIS01's opening sniper run,
destroying either pursuing henchman car before the helicopter reaches the end of the riverside
stretch turns off the fail that ends the run there on Normal and Hard; before, the run always
failed at that point (tick 424) because the rules could not see the enemies. In the race,
destroying the rival wins, as the track's rules say. Enemy cars now also run their own route
triggers (the pursuers' waypoint orders, kills and removals placed along the roads), and
kill / damage / range rules on enemies (MIS4's six targets, MIS13A/B/C, MIS3, MIS11) can fire.

## What the game does [M DRIVING.ELF, D disc]

1. **The trigger walk.** `FUN_0022f0c0` runs `FUN_0022d240` for every vehicle-manager car slot
   whose +0x6e is 2. The car's kind byte +0x6d comes from its creation type in `FUN_001862c0`:
   0/1 to 1, 2 to 2, 3 to 3, 4 and above to 2. The hostile spawn `FUN_0012cb08` creates ground
   vehicles as type 2 (AIEl flag 8) or 4, so every enemy car is kind 2 and fires the triggers
   with flag bit 3; traffic (`FUN_0013fae0`, type 3) is kind 3, bit 6.
   `EFireTriggerSpeedCondition` (0x14c8c8) reads the car in the event's walk slot.
2. **Deaths and removals run nothing of the script.** The AIEl and AICo records hold no `el`
   list (every reference word of the eight tracks' records names an AICo, AIEl or AISp record).
   The wreck in the damage virtual `FUN_00189598` only bumps mission statistics (mgr +0x3c0 via
   `FUN_00206900`, +0x3c4 via `FUN_00206960`) that no rule reads. `FUN_00188200` despawns a wreck
   at once only for "tunnel" / "openheli" / "redheli"; other wrecks stay until the range manager
   drops them.
3. **The rules poll the element's record.** A kind-4 selector is the record pointer;
   `FUN_0012c640` maps it to the runtime record with no state test. Health (`FUN_00209820`) is
   the AIEl +0x4c float, which the live actor's hit points point at (`FUN_00195a08`) and
   `FUN_00195a58` floors at 0, so a wreck reads 0 for the rest of the mission. Position / speed
   (`FUN_002098a0` / `FUN_00209758`) are rt +0x30 / +0x3c: live values, else the record's
   position and 0 (`FUN_0012b440`). Properties `FUN_0012b138`: EMPTY, BLOWOUT, SMOKED, OILED,
   AWAKE. The bearing `FUN_0020ac70`: a live car's signed acos angle to the player.
   So `SRuleDeath` / `SRuleDamage` / `SRuleHealth` / `SRuleRange` / `SRuleProperty` /
   `SRulePlayerDir` on an element see it in any state.

Details and the per-track rule counts: docs/systems/driving-ai.md section 14.

## What changed

- `crates/nf-game/src/driving/hostiles.rs`: `HostileWorld::element_facts` (health, position,
  speed, properties, bearing as above), `script_elements` / `elements_for` (the rule facts for
  every `sr` word naming an AIEl record), `trigger_cars` (live ground vehicles as kind 2 with
  their box), `element_refs`.
- `crates/nf-game/src/driving/script.rs`: `EFireTriggerSpeedCondition` reads the context car's
  speed; `fired_rules` / `take_fired_rules` (a port log of the rules that fired).
- `crates/nf-view/src/driving.rs`: the mission step gives the script the traffic (kind 3, was
  0, same trigger bit), the enemy cars (kind 2) and every element's facts; the rule and trigger
  logs are drained into `DrivingState::rule_log` / `trigger_log` (they grew without bound).
- docs/systems/driving-ai.md section 14, driving-missions.md section 3.

## Tests

- nf-game `driving_hostile_facts` (new, 7, pass): sr words to element indices; health is the
  record float idle / live / after a wreck's despawn; a Death rule fires once on the frame the
  hit points reach 0 and a Damage rule on a drop above its threshold; position and speed live
  vs idle; EMPTY / AWAKE; the bearing's sign and angle; both sim kinds walk as kind 2 and fire a
  bit-3 trigger, not a bit-6 one. New module functions: none of these build on the base.
- nf-game lib `driving::script` (24, pass), two new: the speed condition reads the car that ran
  the trigger (on the base it read the player's speed and fired the wrong way), element Death /
  Damage / Range rules from the facts.
- nf-view `driving_hostile_facts_live` (new, 2, pass, NF_ISO): MIS01, ten player projectiles a
  tick placed on element 3 go through the view's hit test and `HostileWorld::damage`; it dies on
  tick 40, R105 fires that frame, trigger 644 is disabled and never runs, R5 never runs, R113
  fires after timer 4, the mission is not lost through tick 700 (the base loses at tick 424).
  RACE, the rival destroyed on tick 93: R8 fires that frame, R9 follows, the race is won.
- Existing: nf-view `driving_*` / `drive_*` integration binaries 20 / 20 pass (22 tests, the two
  new included); nf-game `drive*` / `driving*` binaries plus the nf-game and nf-view lib
  `driving` unit tests: 42 result sets, 583 tests, all pass.

## Unproven or stand-in

- The car-slot order of the walk (+0x46) is not traced: the port walks the player, the
  traffic, then the enemy cars in element order. Helicopters and henchmen are left out of the
  walk (whether the helicopter objects sit in the 0x30 car slots was not traced).
- BLOWOUT, SMOKED and OILED read false: the port's cars never lose a wheel and nothing writes
  the AIVehicle +0x168 / +0x16c (the player's smoke and oil). The port's henchmen read AWAKE
  while live.
- The acos in the bearing is read from the call shape of the libm pair.
- The view's player projectiles book 1 hit point each (`launch_params` gets damage 0, floored
  to 1); the game's MIS01 weapons are the Missile and the Sniper Rifle (50 per shot,
  driving-mis01-opening.md). The tests place the projectiles on the target; the aim is not
  tested.
- With enemy cars in the walk, the bit-3 route triggers now run for them (MIS01 615, MIS4 240),
  including ones that end the mission (MIS01 T628 and T632 run `EPlayerLose`). How faithfully
  the port's enemy cars follow their roads decides when those fire.

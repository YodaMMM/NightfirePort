# The KikoDeathButton gate (07000016) — the route answer

2026-10-22. The finale's defeat writer is the KikoDeathButton
switch #693 (hash 020008c1 at (7.02, 1.16, −155.61), p0 = 0x90, p5 = the gate 0x8d [D]), and the
level doc parked one question: *whether the game's route presses the button before touching the
mid-corridor gate plate #616*. Walking the real route settled the mechanics.

## The press path ([M], machine-verified)

1. The E press: `Player_Activate` 0x1a5170 — a radius-1.0 sphere at head + look forward, the hit
   list dispatched on the class byte. There is **no class-0x39 case in the dispatch** — the switch
   is reached through its own ScriptPlayer: the class-0 arm (`ext+0xe4 != 0`) calls
   `SP_Activate` 0x1dc338, which for an owned object of class '9' (0x39) does a facing-dot check
   (pressing from behind refuses, 0x1dc338:50-61) and tails into
2. `Switch_Activate` 0x1b02a8 — toggle ch p0 (0x90) when **the gate p5's channel is OFF**
   (`ext+0x14 == 0 || switch_channels[ext+0x14] == 0`, the decompile's line 22), the stored value
   matches the channel, the switch's script is idle, and the out channel is set. The port models
   both hops (`crates/nf-game/src/useobj.rs` — the class-0 SP arm and the gate line at the
   `Switch_Activate` comment).

## The gate plate ([D] + [M])

Touch #616 (placement 269, out 0x8d, mask 8 = the player, volume x 0.36..4.54, y −0.20..1.88,
z −146.65..−144.58 [D the port's trigger dump — the tris are the target object's collision
transformed by the placement]) sits in the mid-corridor between door[3] (z −141.88) and door[2]
(z −152.32) — the **only** route south to the silo/tower end. The corridor at that plane is walled
at x < 0.36 (west) and x 4.54..5.5 (east; the space east of it has no floor above y −5 [D ray
scans]) — the volume spans the full walkable width, so **every route to the silo crosses it**.

#616's placement flags are **0xEA = TouchOnce** (the parser dispatch [M
`parsemap_create_dynamic_objects` 0x1d04f0 case 0xea → `Trigger_TouchOnce`, which stamps the type
1]) — the **latching** family: the per-class dispatcher table (0x2165c8 entry {0x1b1a78,
0x550, …}) calls `Trigger_Update` for every trigger each frame, and `Trigger_Update`'s type-1 arm
sets `switch_channels[out] |= hit` (0x1b1a78 case 1→2). `Trigger_Calc` 0x1b1900 (disassembly
checked) clears and re-fires only **type-2** (flags 0xEB `Trigger_Touch`) triggers under
ProcessTouch — the presence family. A type-1 plate latches its channel permanently. The port's
`triggers::build` maps flags 0xea → TOUCH_ONCE and its update latches the same way.

## Nothing clears ch 141 [D]

- Every script in 07000016.bin and the loadable pack 07800016.bin was decoded (event-0xD direct
  writes and event-0x11 conditional writes): the launch script 0x0600090a writes
  187/176/197/198/**243±**/240/190/245/244/**243±**/186/193/194/195/196/189/177/178/241/242/192/
  **80**/204/205, the pack cinema 0x06000631 writes 185/21/180(=0), the bay-lift script 0x60009e9
  writes 80(=0) — **no 141**.
- No trigger, mux, switch or drone placement writes 0x8d (the census in docs/levels/07000016.md §7;
  re-verified from the raw placements).
- `Init_SwitchChannels` 0x134310 (the only bulk clearer) has exactly one caller, in the level-load
  path (jal at 0x1d5208, next to `SSys_Init`/`Mission_Init`) — never mid-level.

## The walked result (the test)

`crates/nf-view/tests/kiko_button_real_walk_live.rs::the_07000016_real_walk_reaches_the_kiko_death_button_and_the_gate_arms_on_the_route`
walks the whole route (`Sim::step_bond` + `Input`; the single documented [S] re-seat is the
pack-flight stand-in after the cinema, before the ch-0xf3 pulse) and pins:

- the walk reaches the button's side (d < 1.5) through the west lane past the tower-base wall —
  the "slab leak" of the fix-finale probe was that wall (a diagonal collision ramp whose blocked
  cells (6.5,−152.5)→(7,−153.5)→(7.5,−154.5)→(8,−155.5) shift east with depth [D ray scans]); the
  walk needs no collision fix;
- the gate ch 0x8d is ON from the corridor crossing to the press [D #616 latch];
- the walked E is **inhibited** [M 0x1b02a8:22] — ch 0x90 stays cold, the mission does not
  complete;
- sister: the same E from the documented gate-off reach (a fresh load teleported to the button
  room, #616 never crossed) fires ch 0x90 → mux #618 → ch 100 → MissionState 6 → 0x7300017
  [M 0x182c60:111-112].

## The open link — live check

The one step not closed without the real game: whether the GAME's contact pass registers
#616 for the walking player at all. The game's `Trigger_Hit` 0x1b1fc8 does no geometry of its own —
it classifies the object's contact list (obj+0xD0, built by the collision pass against the
trigger's **glist bounds**); the port's touch test is the target object's collision tris. If the
game's glist bounds for #616 match the port's volume, the shipped game's golden path arms the gate
and the button is permanently inhibited after the mid-corridor — a shipped-game quirk the port
mirrors. Check on a fresh boot (a save-state resume skips the boot path):

- watch `switch_channels[141]` = `*(u8*)(0x26fc90 + 141)` = **0x26fd1d** while Bond walks the
  mid-corridor z −144..−147 (x ~2, the walked lane);
- expect 0 → 1 on the crossing if the game arms it like the port; a steady 0 means the game's
  #616 bounds differ from the port's tris and the walked press should complete — then the port's
  #616 tri source (the target object idx 166's collision) needs the glist-bounds correction and
  the walked test's main assert flips to the completion chain.

Also worth watching in the same run: `switch_channels[52]` (0x34, touch #692 — the second
mid-corridor plate, door[0]'s south side; no disc consumer — the same latch question) and Bond's
own y through the bridge fall (z −174..−178: the deck holds to ≈−174 then the pit — the game's own
silo-mouth transition, expected to match the port).

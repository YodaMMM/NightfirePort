# The doors' text labels 0x2000003 / 0x200000b (G-035 remainder, `door-text-labels`)

The two door text labels the HUD INFO bar (type-1 status pane, `MsgInfoStatusPane` 0x319810) can show for a door.
Neither fires on the E key: `Player_Activate` 0x1a5170's door arm (Door_Activate at 0x1a5170:128 in the decompile)
carries no text call at all [M — the decompile has no `Txt_BindLabel`/`Text_AddMsg` in it].

## The strings [D USATxt.dat via `nf_formats::hud::Lang::bind`]

| id | text |
|---|---|
| 0x2000003 | `Door Unlocked` |
| 0x200000b | `Press Action to open door` |

(Resolved through the port's `Lang` on the slim ISO's USATxt.dat; the scratch probe printed both. `Txt_BindLabel`
0x1cad60 resolves `id` = bank<<24 | index, bank 2 = the level text bank.)

## Site 1 — 0x2000003 "Door Unlocked": the lock channel's rising edge in `Door_Update` 0x1363c8

`Door_Update` [M 0x1363c8], after the open-channel block, when the door has a lock channel wired (`DOORINFO+0x68`
≠ 0) and the channel has a **rising edge** this tick (`switch_channels_prev[ch] == 0` and
`switch_channels[ch] != 0` — decompile lines 88-100; skip when the channel was already on or is off now):

1. busy marker clear (`DOORINFO+0x70` bit 0x10 — the early return at the top of `Door_Update` already skips busy
   doors, so this re-check is shadowed);
2. **level gate**: `GameState` (the level id) is **not** 0x7000014 and **not** 0x700004a [M decompile line 101,
   the `lui 700 / ori 0x14 / 0x4a` compares] — on those two levels the door unlocks silently;
3. `Txt_BindLabel(0x2000003, 0)` then `Text_AddMsg(0, 0, 1, label, 0, 0xb4)` — the type-1 INFO bar, **180 ticks**
   (0xb4; `Text_AddMsg` 0x1b0c30 stores the time as the node's life, scaled per wrapped line) [M decompile lines
   102-103, disasm `addiu t1,zero,180` at 0x1a7bbc is the sibling trigger site's identical duration];
4. `Door_Activate(obj, glb_players, 0)` — the door opens (the unlock);
5. `DOORINFO+0x68 = 0` — the lock channel field is cleared: the door is unlocked **for good**;
6. busy marker set (`+0x70 |= 0x10`) [M decompile lines 105-107].

Order proven by the decompile; the port's `Doors::update` already did steps 4-6 (the rising-edge unlock) — this task
adds the message raise (step 2-3) before `activate`.

## Site 2 — 0x200000b "Press Action to open door": the aim ray in `Check_Target` 0x1a7810

`Check_Target` runs every 6th tick (`GameState+0x52 % 6 == 0`) [M decompile line 98] and casts the player's
50-unit view ray (`Player_GetHeadPos` + `Mat_GetDir`, `Collide_RayIntersect` mask 0xc, 0x42480000 = 50.0f end)
[M decompile lines 109-123]. For the FIRST hit object (HITDATA+0x58, disasm `lw a0,0x58(a1)` at 0x1a79ec;
class byte `obj+0xff`):

- **class 0x1c (a door)** [M disasm 0x1a7a14-0x1a7a18]:
  - the hit distance < 5.0 (`HITDATA+0xc` vs 0x40a00000, disasm 0x1a7b6c-0x1a7b84) [M];
  - the level is not 0x700004a [M disasm 0x1a7b58-0x1a7b64];
  - the global `DoorMsg` (0x30cd04, a .sdata local — the **only** accesses in ACTION.ELF's text are
    0x1a7b88/0x1a7ba8/0x1a7bc0, all inside `Check_Target`; a full scan for the gp-relative offset 0x8694 finds
    nothing else) is < 3: `sltiu v0,v0,3` [M disasm 0x1a7b88-0x1a7b90]. **At most 3 door hints per boot —
    the game never resets it**;
  - `Door_DoMsg(door)` 0x135f18 returns nonzero → `DoorMsg++` and `Text_AddMsg(*(byte*)(BLData+0x94e), 0, 1,
    label, 0, 0xb4)` — the same type-1 INFO bar, 180 ticks [M disasm 0x1a7b98-0x1a7bc8].

`Door_DoMsg` 0x135f18 [M], with the door object:

1. multiplayer off (`MPSettings+0x388 == 0`) — no door hints in MP [M decompile line 13];
2. the door's `DOORINFO+0x74` latch: **if it is 0, set it to 1 first, then** run the checks — one hint attempt per
   door ever, and a failed attempt (locked door) burns the latch [M decompile lines 16-17];
3. the lock channel gate: a door **with** a lock channel (`+0x68 != 0`) whose channel is **off** fails here — a
   locked door gives no hint at all [M decompile lines 20-24];
4. the state gate: own state (`obj+0xf4`) for an ungrouped door, `DoorGroupsStates[g]` (the `Doors_Calc` OR) for a
   grouped one; **`== 4` (OPEN) fails** — the hint only shows on a door that is not open (closed, opening or
   closing all hint) [M decompile lines 25-34; the port's `Door::OPEN = 4` [M Door_SetState family]];
5. return `Txt_BindLabel(0x200000b, 0)`.

So the hint ("Press Action to open door") shows once per door, max 3 per boot, when the player aims within 5 units
at a closed door they can open. A locked (lock channel down) door raises **nothing** — the locked feedback is
`Door_Activate`'s hit sound only [M 0x135be0 LAB_00135cf8].

## Sibling (context, not this task)

The same ray's class-0x1d trigger arm shows label **0x2000009** (also "Press Action to open door") for a type-10
trigger within 5 units when `Trigger_Locked(obj) == 0`, under the 20-tick `LoadMsg` (0x30cd08) cooldown and level
≠ 0x7000009 [M Check_Target decompile lines 193-202]. Not ported here.

## The port

- `nf_game::door::Doors::update` raises `(0x2000003, 180.0)` into the new `Doors::labels` queue on the rising edge
  (before `activate`, matching the game's order); Sim drains it into `HudState::info_labels`, dropping it on levels
  0x7000014/0x700004a (Doors does not know the level; the gate is the caller's — the activate/clear/busy steps run
  regardless, exactly like the game) [M gates above; [S port architecture: the queue placement]].
- `nf_game::door::Doors::do_msg(i, mp, channels)` ports `Door_DoMsg` 1:1 (latch-then-checks, channel gate, state
  ≠ OPEN gate, returns 0x200000b). The existing `Door::msg_shown` flag is its `+0x74` latch (it existed unused).
- `nf_game::door::Doors::pick_target(eye, dir, wall_t)` is the aim ray's door pick: ray vs each door's LIVE world
  sphere (obj+0x80: the matrix-transformed gfx centre, radius obj+0x8C — the same sphere the auto-close scan uses),
  wall-occluded like `gadget::pick_decryptor_lock`. **[S unverified as to shape]** — the game's
  `Collide_RayIntersect` hits the door's collision mesh and reports the object (HITDATA+0x58); the port's
  `world.segment` returns no object id, so the sphere stands in for the mesh. A centre-aimed ray agrees with the
  mesh; edge aims may differ (live-check below).
- `HudState::door_msg` counts the hint raises (the game's 0x30cd04). [S port architecture: the game's counter
  survives level loads (it is .sdata, never reset); the port's Sim is per level, so it resets with the level — the
  per-boot cap of 3 within one level is preserved.]
- The labels ride the existing type-1 INFO pipeline: `info_labels` → bound in `hud_tick` → `info_msgs` → the INFO
  pane (`hud.rs`). The main agent's draw code needs nothing new — the pane already draws `info_msgs`.

## Tests (`crates/nf-view/tests/door_text_labels_live.rs`)

- `aim_at_a_closed_unlocked_door_raises_the_press_action_hint_once`: 07000001 door #0 (ch_lock 0, closed, [D
  placements]) — Bond placed ~3 units out aiming at the sphere centre, real `step_bond` ticks →
  `info_labels` receives `(0x200000b, 180.0)` once; the latch holds (no second raise while aiming); `door_msg == 1`.
- Sister: `a_locked_door_raises_nothing` — 07000007 door #11 (ch_lock 200, channel down at load, closed, 2.3 units
  from spawn [D placements]) — the same aim → nothing in `info_labels`, and the latch is burned (one attempt).
- `the_lock_channel_rising_edge_raises_door_unlocked`: 07000007 door #11 — channel 200 set through the real
  `Channels` path → the next tick raises `(0x2000003, 180.0)`, the door leaves CLOSED and its `ch_lock` clears.
  Sister: holding the channel on gives no second label (the edge is gone and the field cleared).
- Sister for the level gate: on 0700004a (one of the two gated levels [M]) a rising edge raises nothing.

All through the real input path (`Sim::step_bond`), `NF_SILENT=1`, ISO-gated.

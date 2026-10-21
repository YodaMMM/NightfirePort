# MP never-mentioned functions — the 8 `MP_*` gameplay getters/hooks (unported-multiplayer-1)

The whole-ELF scan (master 20874c31, 2026-10-20) flagged eight ACTION.ELF multiplayer
functions whose name/address appeared nowhere in `crates/*/src`. This note pins each one's
caller chain and where the port now carries it. Evidence tags: [M] decompile/disassembly,
[D] disc, [L] EE snapshot. The ELF byte scans below were made this session over
`work/elf/ACTION.ELF` (jal = the 0x0C000000-encoded target word; data refs = the raw
little-endian address word in `.data`).

## The shared creation path (why these are reachable at all)

`parsemap_create_dynamic_objects` 0x1d04f0 (the map loader's dynamic-object pass) calls
`MP_RegisterMPObject` 0x185990 per MP placement; the ones whose scenario word matches
`MPSettings+0x1a4` go on to `MP_CreateObject` 0x185de8, which creates a **class-0x35**
object (`obj+0xFF = 0x35` [M 0x185de8:129]) and files it in the per-type exts (`Flags`[2]
@0x317210, `Bases`[2], `Uplinks`[8] @0x317450, `Demolition`/`Protection`, `EsponageBase`[2],
`BluePrint` @0x318830, `GoldenEye`[2] @0x317ff0/0x318080, `Hill` @0x318CE0).
`control_funcs` @0x26f890 row 0x35 = { update `MP_ObjectUpdate` 0x18b198, collide 0, delete
`MP_ObjectDelete` 0x18b2c0 } [M data words at 0x26fb0c/0x26fb10/0x26fb14, dumped this
session]. So every scenario-object update below runs from the standard per-frame object
walk (`control_movement_object_handler` 0x1336f8 ← `Game_Run` 0x1c9790 [M]) in an MP match,
and `MP_Update` 0x184bd0 (jal 0x1c97f0 in `Game_Run`) drives the bot half.

## The eight

| # | function | callers (jal sites) | reached | outcome |
|---|---|---|---|---|
| 1 | `MP_isPlayerOrBotDead` 0x18abc0 (208 B) | `MP_KOHUpdate` 0x186ae0 (jal 0x186b44) = `MP_ObjectUpdate` case 9 [M] | yes — KOTH MP matches (the hill object registers under 0x40000800/0x60001000 [M 0x185990 case 9]) | **cited** at the port's same job: `MpMatch::koth_update`'s alive gate (`crates/nf-game/src/mp/rules.rs`), the game's fuller ext predicate in the comment |
| 2 | `MP_ObjectDelete` 0x18b2c0 (8 B) | `control_funcs[0x35]` delete slot (data ptr @0x26fb14); the delete pass fires it when a class-0x35 obj is delete-marked — `MP_RestartScenario` 0x183fc0:15 marks them all via `Control_DeleteAllObjectsOfType` 0x1349e8 | yes — every MP round restart | **cited as the empty stub it is**: `restart_scenario`'s doc (`rules.rs`). Body = `jr ra; nop` [M bytes 0800e003 00000000] |
| 3 | `MP_HasEsponage` 0x18b468 (36 B) | `HUD_MPUpdatePane` 0x19f0a0 (jal 0x19f2d0), scenario 0x20000100 arm | yes — espionage MP matches with a score pane | **ported**: `MpMatch::mp_has_esponage` (`rules.rs`); drives the pane's 0x300018a icon (`mp/hud.rs` `MpScorePane::build`) |
| 4 | `MP_ObjTeamHasGEObj` 0x18b4a0 (152 B) | `HUD_MPUpdatePane` (jal 0x19f300, 0x19f328), scenario 0x20000200 arm | yes — GoldenEye MP matches | **ported**: `MpMatch::mp_obj_team_has_ge_obj` (`rules.rs`); drives the pane's 0x300018b/0x300018c icons (`mp/hud.rs`) |
| 5 | `MP_isObjectOnTeam` 0x18b720 (128 B) | `Check_Target` 0x1a7810 (jal 0x1a7a48, 0x1a7a7c) ← `Player_AutoAim` 0x1a7c98 ← `Player_Weapon` 0x1a24c8 [M] | yes — every on-foot match while aiming (MP arms only) | **ported**: `MpMatch::mp_is_object_on_team` (`rules.rs`); the port's SP crosshair walk has no MP arm yet (see gaps) |
| 6 | `MP_getMPpickupFromPickupinfo` 0x18be98 (64 B) | `Pickup_Handler` 0x18e650 (jal 0x18e9d8), the bot arm before `BOTSTATE_setPickupVisitTime` 0x129958 | yes — a bot taking a weapon/ammo/health pickup in MP | **cited** at the port's same job: `BotBrain::pickup_visit_stamp` (`mp/bots.rs`) — the port's caller already holds the row index, so the pointer→row walk is the identity |
| 7 | `MP_pickupCountCheck` 0x18bed8 (8 B) | `BOT_update` 0x126448 (jal 0x126450) ← `MP_Update` 0x184bd0 ← `Game_Run` [M] | yes — every MP frame with bots | **cited as the empty stub**: the `BOT_update` site comment (`crates/nf-view/src/scene.rs`). Body = `jr ra; nop` [M] |
| 8 | `MP_DebugDraw` 0x18bf60 (8 B) | `control_movement_object_handler` 0x1336f8 step 1 (jal 0x133738) ← `Game_Run` [M] | yes — every frame of every level | **cited as the empty stub**: the control-walk comment (`crates/nf-game/src/drone.rs`). Body = `jr ra; nop` [M] |

`HUD_MPUpdatePane` 0x19f0a0 itself dispatches from the pane-type record at 0x2be9c8
({ create `HUD_CreateMPScorePane` 0x19ed10, update `HUD_MPUpdatePane` }) — the MP score
pane type the `MPPaneList` walk calls [M 0x199b70:42, 0x199e88:36; data word @0x2be9cc].
Its per-frame update returns early when the viewer obj or the slot's `MPGame+slot·0x30+0x1c`
obj is class 2/0x11 (bots draw no pane) [M 0x19f0a0:22-41].

## The pane's espionage / GoldenEye arms (what the ports drive)

`HUD_CreateMPScorePane` 0x19ed10's else-arm (every scenario but 0x400 and 0x60000008) moves
sprites 1 and 2 down 0x18 from the shared base y 400 [M :113-117; the 0x400 arm uses
−0x10/−0x20, uplink −0x30 + x 16/32/48]. `HUD_MPUpdatePane` then, per frame:

- scenario 0x20000100 (espionage): `MP_HasEsponage(viewer)` → sprite 1 layer 0x1e, texture
  0x300018a [M :75-82]; otherwise sprite 1 layer 0xff (hidden with the rest, :72-74).
- scenario 0x20000200 (GoldenEye): `MP_ObjTeamHasGEObj(viewer, 0)` → sprite 1 → 0x300018b
  and `(viewer, 1)` → sprite 2 → 0x300018c, both layer 0x1e [M :115-127]. The create moves
  both sprites by the same −0x18 with no x change, so a team holding both items stacks the
  two icons on one rect — that is the shipped layout [M], not a port simplification.
- The non-carried default colour stays the template's 0x7f7f7fff [L MPScoreInfo @0x2be888].

The port's `MpHudSnapshot` now carries `viewer_carries_blueprint` / `viewer_team_ge` (read
through the two getters at build), and `MpScorePane::build` adds the two scenario arms —
the data the main agent's draw code needs (no drawing here).

## `MP_isPlayerOrBotDead` 0x18abc0 — the exact predicate

For obj class byte +0xFF:

- class 2 (a character): alive iff `ext+0x4f8 & 0x600 == 0 && ext+0x4f8 & 0x100 != 0 &&
  *(f32*)(ext+0xac) > 0.0 && (obj+0xfe & 1) == 0` [M; the ext is `obj+0xe0`, the drone
  flags word + health]. Otherwise dead.
- class 0x11 (a body): dead [M — the alive shape requires `class != 0x11`].
- class 0x12: dead; class 3 with `+0xf4 ∈ {2, 3}`: dead; anything else: alive [M :38-45].

`MP_KOHUpdate` only ever passes the `MPGame+slot·0x30+0x1c` player objs, so the class-2 arm
is the one that runs in the shipped game. The port's `koth_update` gates on
`slots[slot].dead` — set by `player_killed` (the `MP_PlayerKilled` port) and cleared by
`do_respawn` (the `MP_ReSpawn` port) — which is the same job at the port's abstraction;
the citation comment carries the full predicate for whoever ports the ext-flags reader.

## Gaps / live checks for the main agent

1. **GoldenEye stacked icons** — run a GoldenEye match, let one team hold both items, and
   check the score pane: the game draws both 0x300018b and 0x300018c at the same rect
   (16, 376). PCSX2 should show exactly that overlap; if the real game separates them,
   the separation comes from somewhere outside 0x19ed10/0x19f0a0 and the port needs it.
2. **Espionage icon swap** — carry the blueprint and confirm the pane's objective icon
   becomes the "carrying" glyph (0x300018a) for the carrier only, other viewers unchanged.
3. **`MP_isObjectOnTeam` MP arm** — `Check_Target`'s class-2/class-3 friendly gates
   (0x1a7a48/0x1a7a7c) are not modelled in the port's crosshair walk (SP-only, per
   docs/reports/civilian-no-shoot.md). The getter exists; wiring it into the aim path is
   future MP-visual work (the markers at ext+0x133/ext+0x132).
4. `MPSettings+0x184` (`_388_`, the `MP_Start` master-copy that gates `Check_Target`'s MP
   arms) is documented `master_copy` in `mp/mod.rs` — its Ghidra print there says "+0x388",
   which is the decimal/hex mix-up the +0x180 note in the same struct already corrected;
   the copy instruction is `MP_Start` 0x183a50:70 [M]. Harmless to the port (the field is
   only a doc), but worth one line in the next mod.rs pass.

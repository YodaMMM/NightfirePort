# The DMODE inits: `DroneModeSettings` and the OLD mode chain

Tags: [M] decompile/disassembly read here (ACTION.ELF), [D] disc bytes, [S] suggested.
2026-10-21. Rows 23-30 are covered the same way in docs/port-map/unreached.md.

## The table [M 0x14cfa0 / D ELF data]

`DroneModeSettings` 0x29ad40, 35 rows × 12 bytes. `NDrone2_DoModeSettingsOLD`
(0x14cfa0) reads row `param_2` (the placement's parameter 5, `piVar19[0x12]`,
DefaultInit 0x14b300:470/495):

- byte 0 → `Drone+0xC5` (the mode byte `Drone_Control` dispatches on);
- byte 1 → `Drone+0x44` (team);
- word +4 → `Drone+0x500` and `Drone+0x50C` (resting and live alertness);
- word +8 → the row's init function, called with the drone when non-null;
- both props blocks (+0x4DC/+0x4E8) are memset 0 first, `+0x4D8` points at +0x4DC.

`param_2 > 0x23` dispatches to `NDrone2_DoModeSettingsNEW` (0x14c600) instead, which
never calls any row function — it rebuilds the profile from the placement's behaviour
words. `DoModeSettingsOLD`'s only caller is `NDrone2_DefaultInit` 0x14b300:495-502
(`< 0x24` branch); `DefaultInit`'s only caller is `NDrone2_PostLoad_Init` 0x14d890,
whose callers are `Drone_PostLoad_Init` (placements), `DroneSpawner_SpawnDrone`
0x137cb0 (spawner children inherit the template drone placement's param block,
0x137cb0:75), `Drone_CoderCreate` 0x13a108 and `BOT_init` 0x1249e8 (MP bots).

## Which rows the shipped game reaches

Every `Drone_Create` copy puts the template's +0x2C block (0x84 bytes) at
`Drone+0xC34` = `piVar19[13]` (0x136be0:59,81), so parameter k = word 13+k.

- **Placements [D census, this disc]:** 555 drone placements (flags & 0xff == 0x0f)
  across the 34 archives with a main map; parameter 5 is **100 for every one of
  them** (histogram {100: 555}) → all NEW, none OLD. (The 856 report says "≥ 0x24";
  the value is exactly 100, the alert-block sentinel the port already parses.)
- **MP bots [M 0x1249e8:101-102]:** `BOT_init` memsets its DIVars template and never
  writes word 18 → parameter 5 = 0 → OLD **row 0**.
- **Coder-created drones [M 0x13a108:55-60 + D census]:** `Drone_CoderCreate` memsets
  0xb0 bytes then fills only the skin (param 0), start script (param 1) and one more
  word (param 7) → parameter 5 = 0 → OLD **row 0**. The disc carries exactly one
  script event of this kind: 07000001's NIS 0x0600003B, `Cmd::Event` kind 8, params
  [0x0500005B, 0x06000106, 16, 1] (skin 0x0500005B, start script 0x06000106) →
  `Script_EventHandler` 0x1d94c0 case 8 → `Drone_CoderCreate` → OLD row 0.

So of the 35 rows only **row 0 is reached** on the shipped game, through two paths:
the 07000001 coder event (on-foot) and every MP bot. Row 0's function is
`NDrone2_init_DMODE_Normal` 0x14d958, a pure tail-call to
`NDrone2_init_DMODE_Defaults` 0x14d528. Rows 1-30 (and the empty rows 31/32/34) are
never dispatched — see docs/port-map/unreached.md.

## Row 0's effect [M 0x14d528 + 0x14cfa0]

`Defaults` sets 40 behaviour bits (ids 1, 6, 7, 10(×2), 0xc(×2), 0x48, 0xf, 0x11,
0x13, 0x18, 0x1d, 0x4c, 0x21, 0x29, 0x2e, 0x37, 0x41, 0x42, 0x4f, 0x51, 0x5a, 0x24,
0x3c, 0x1f, 0x44, 0x33, 0x32, 0x31, 0x28, 0x27, 0x26, 0x30, 0x2f, 0x2a, 0x43, 0x3b,
0x17, 0x46) on the active props block (+0x4DC). `DoModeSettingsOLD`'s tail then adds
id 0x38 (the rest of its unconditional tail list is already inside Defaults; the
+0xd8-conditional extras and the `+0xC5 == 0x11` extras don't apply — row 0 leaves
the mode byte 0 and the coder skin 0x0500005B takes the switch default +0xd8 = 0).
Row 0's team byte is 1 and its alert word 0x00000000, so the drone starts at
alertness 0.0 — both are the port's construction defaults.

## The astronaut control thunk [M 0x14b128]

`NDrone2_ControlDTYPE_Astronaut` 0x14b128 (28 bytes) is the `DroneTypeSettings`
0x29aee8 row 0x1d slot +8 read by `Drone_Control` 0x139268:60-64
(`Drone+0xC5 * 0xC + 8`), and its whole body is a tail-call to
`NDrone2_ControlSTANDARD` 0x1490f0. Mode 0x1D comes from DefaultInit's +0xd8 == 0x18
conversion [M 0x14b300:718-727: +0xC5 = 0x1D, +0x5A2 = 0xBD, +0xC4 = 0x1D, +0xC6 =
0, +0x5A4 = 0, `DroneInit_TakeXHits`], whose skins are 0x5000078 / 0x5000094 /
0x50000BA [M 0x14b300:233-237]. **0700001b carries 19 drone placements with those
skins** (18 × 0x05000078, 1 × 0x050000BA — [D census]) → the mode and its control
dispatch are reached on foot. The port converts kind 0x18 → mode 0x1D at spawn and
runs the standard tick, which IS this function's body (citation in drone.rs).

## Near-player physics gate [M 0x14b190 / 0x148db0 / 0x141e58 / 0x149ec8]

`DroneFunc_NearPlayerCollision` 0x14b190: "some player slot is non-null AND the
drone's distance to that player is < 2.0". The distances are `Drone+0x154..+0x160`,
filled per tick by `NDrone2_PreDroneControl` 0x148db0:35-48 (100000.0 for an absent
slot). `NDrone2_DoCollision` 0x149ec8:23-26 and `NDrone2_DoGravity` 0x141e58:29-32
call it: within 2.0 of a player both force their "physics on" return, overriding the
`+0x570 & 2` no-physics flag; `Drone_CollisionHandler` 0x140968:152-190 picks the
object-collision branch on `DoCollision` and the free-gravity branch otherwise.

## Addendum 2026-10-24 (ai-dmode-inits): the coder event's true hashes, the full row dump, census scope

Re-derived everything on this page independently ([D] fresh ISO scan + [M] disassembly). Three
corrections and two completions; the conclusions (all placements NEW, only row 0 dispatched on the
shipped game) are unchanged.

1. **The coder event's hashes above are wrong.** The disc carries exactly one `Cmd::Event` kind 8
   among 851 events in 468 scripts: **07000001 entry `0600005B`** (not `0600003B` — no such entry
   exists anywhere on the disc), raw params **[0x0500001B, 0x060000A6, 16, 1]** (not
   0x0500005B/0x06000106) [D census; raw stream `12 04 08 | 1B000005 | A6000006 | 10000000 |
   01000000 | 12`]. `Script_EventHandler` 0x1d94c0:138-143 case 8 passes them to
   `Drone_CoderCreate` 0x13a108 with a constant 7th arg 4; the stores land at template words
   11/12/20 [M 0x13a108:0x13a160 sw s0,44(sp) / 0x13a168 sw s1,48(sp) / 0x13a170 sw s2,80(sp)] =
   parameters 0 (skin 0x0500001B), 1 (start script 0x060000A6) and 7 (= 1, the event's fourth
   word); the third word (16) is param_5's low half and is never stored. Consequences: skin
   0x0500001B takes DefaultInit's **case 0x500001b → +0xd8 = 9** (not the switch default 0)
   [M 0x14b300:153-156] — the +0xd8-conditional extras still don't fire (9 ∉ {0x10,0xd,0x11,0x13}).
   CoderCreate's `sh s5,1442(v0)` writes **+0x5A2 = 4** [M 0x13a108:0x13a1a0], but DefaultInit's
   `DoTypeSettingsOLD`(+0xC5 = 0) overwrites it unconditionally: DroneTypeSettings row 0 start = −1
   → by mode → prop 0x24 (Defaults sets it) → **+0x5A2 = 6 InitPatrol** [M 0x14d2e8:0x14d32c /
   0x14d3bc `sh v0,1442(s0)`, prop checks 0x14d394-0x14d3b8].
2. **Row word+4 → +0x500/+0x50C confirmed at instruction level** (a byte-scan misses it because it
   is FPU): `lwc1 f0,4(s2)` @0x14d010 → `swc1 f0,0x50C(s1)` @0x14d014 + `swc1 f0,0x500(s1)`
   @0x14d01c [M 0x14cfa0].
3. **Census scope corrections.** 35 archives carry a type-1 main map (not 34); 21 of them carry
   drones (07000048/07000500 have main maps but no drones); the 555 is the main-map total
   {100: 555} — unchanged. The scan additionally finds **33 kind-0x0f template records** (all
   `flags == 0x0000000F`) with parameter 5 = 0 in the type-0 (template) map `01000166`, object
   hash 0x02000637, byte-identical across the 41 archives carrying that map — 588 kind-0x0f
   records total. Whether template-map records ever reach `Drone_Create` at runtime is
   [S] unverified from disc; if they do they select row 0 like the coder event (parameter 5 = 0),
   so the row-0-only conclusion holds either way.
4. **Full 35-row dump** ([D table 0x29ad40, all 32 non-NULL fn pointers exact-symbol matches;
   byte+0 → +0xC5 @0x14d038/0x14d040, byte+1 → +0x44 @0x14d078/0x14d07c, word+8 `jalr`
   @0x14d080-0x14d090; bounds `sltiu v0,s3,0x24` @0x14cfc8): rows 0-22 as tabled above (row 20 =
   `init_DMODE_Ambush` 0x14e018, type 0x10, team 1), plus rows 23-33: 23 `CivilianGuard`
   0x14e158 (type 0x13, team 3), 24 `Interogator` 0x14e238 (0x14, 1), 25 `CivDoorGuard` 0x14e258
   (0x16, 3), 26 `TruckDriver` 0x14e348 (0x17, 3), 27 `CastleChatGuard1` 0x14e370 (0x18, 1),
   28 `CastleChatGuard2` 0x14e3a0 (0x18, 1), 29 `SniperAlert` 0x14e3e0 (0x19, 1, alert 1.0),
   30 `PartyGirlLooker` 0x14e490 (0x12, 3), 31/32/34 fn = NULL, 33 `Bot` 0x14e568 (0x1e, team 2).
   Rows 1-34 are never dispatched: parameter 5 ∈ {0, 100} on every main-map placement and every
   coder/bot template [D census].

## Appendix 2026-10-26 (dmode-inits-hud-panes): every `init_DMODE_*` body

Per-init [M] summaries (sizes from the decompile headers; the property id is
`behaviour_util_setProperty`'s first arg, value 1 unless noted; every call targets the drone's
active props block `+0x4d8`). This is the whole family — 33 functions, the 35-row table's
non-NULL slots. Grouped by body shape:

**Pure tail-calls to `Defaults` 0x14d528 (6, 28 B each)** — Normal 0x14d958 (row 0),
Guard 0x14d978 (row 1), Retreater 0x14d998 (row 2), Attacker 0x14dab0 (row 5),
Assassin 0x14db18 (row 7), Interogator 0x14e238 (row 24). Their whole effect is Defaults'
40-prop list.

**`Defaults` + extras (6)** — Stealth 0x14da78 (row 4, 52 B) adds prop 0x25;
RunToPoint 0x14dad0 (row 6, 68 B) adds 0x23 and 0x58; AlarmRaiser 0x14df98 (row 18, 68 B)
adds 0x23 and 0x56; SearchLight 0x14dfe0 (row 19, 52 B) adds 0x40; Ambush 0x14e018
(row 20, 52 B) adds 0x10; Zoe 0x14e050 (row 21, 68 B) adds 0x40=1 **and clears 0x21=0**
(the family's only 0 write).

**Standalone prop lists (15, no Defaults call)** — Sniper 0x14d9b8 (row 3, 192 B):
{0x3c, 0x1f, 0x33, 0x32, 0x31, 0x43, 0x17, 0x39, 0x40, 0x5a}. HostageKiller 0x14db38
(row 8, 224 B): {0x3c, 0x1f, 0x33, 0x32, 0x31, 0x29, 0x28, 0x27, 0x26, 0x43, 0x40, 0x22}.
Hostage 0x14dc18 (row 9) and HostageTied 0x14dc88 (row 10) — identical bodies, 112 B each:
{0x24, 0x3c, 0x1f, 0x40, 0x4e}. JustStand4Demo 0x14dcf8 (row 11, 36 B): {0x40}.
Civilian 0x14dd28 (row 13, 288 B): {0x1d, 0x24, 0x3c, 0x1f, 0x29, 0x46, 0x32, 0x31, 0x27,
0x26, 0x3b, 0x4c, 0x4f, 0x50, 0x51, 0x53}. CivilianScared 0x14de48 (row 14, 224 B):
{0x3c, 0x1f, 0x32, 0x31, 0x27, 0x26, 0x3b, 0x4c, 0x4f, 0x50, 0x51, 0x53} (Civilian minus
{0x1d, 0x24, 0x29, 0x46}). MissionFailer 0x14df28 (row 15, 64 B): {0x3c, 0x55}. Mayhew
0x14df68 (row 16, 36 B) and TruckDriver 0x14e348 (row 26, 36 B) — identical bodies: {0x1d}.
PartyGirl 0x14e098 (row 22, 192 B): {0x1d, 0x24, 0x3c, 0x1f, 0x46, 0x26, 0x27, 0x28, 0x29,
0x51}. CivilianGuard 0x14e158 (row 23, 224 B): {0x1d, 0x24, 0x3c, 0x1f, 0x46, 0x26, 0x27,
0x28, 0x29, 0x4f, 0x50, 0x51}. CivDoorGuard 0x14e258 (row 25, 240 B): {0x1d, 0x18, 6,
0x3c, 0x1f, 0x46, 0x26, 0x27, 0x28, 0x29, 0x4f, 0x50, 0x51}. SniperAlert 0x14e3e0
(row 29, 176 B): {0x3c, 0x1f, 0x33, 0x32, 0x31, 0x43, 0x17, 0x39, 0x40} (Sniper minus 0x5a).
PartyGirlLooker 0x14e490 (row 30, 216 B): {0x1d, 0x24, 0x3c, 0x1f, 0x46, 0x47, 0x26, 0x27,
0x28, 0x29, 0x51} **plus one direct store: `*(u16*)(drone+0x35c) = 2`** — the head-tracking
weight `DoHeadTrackingBehaviour` 0x147578 reads (the only reader in the decompile).

**`Defaults` + a direct store (2)** — CastleChatGuard1 0x14e370 (row 27, 44 B):
`*(u8*)(drone+0x13d) = 0x19`. CastleChatGuard2 0x14e3a0 (row 28, 60 B): prop 0x1d=1 then
`*(u8*)(drone+0x13d) = 0x11`. The +0x13d byte is the chat-dialogue phase the DSTATE
`NDrone2_DSTATE_CastleChatGuard1` 0x1612e0 and `Player_Update` 0x193600 read; the family's
only writes of it.

**Empty bodies (3, 8 B `jr ra`)** — DeleteMe 0x14dd20 (row 12), Ninja 0x14df90 (row 17),
Bot 0x14e568 (row 33): no writes at all (the decompiler's C shows none, and an 8-byte body
holds exactly the return).

Row bytes and the fn pointers above are pinned as a runnable test by
`crates/nf-formats/tests/dmode_row_table.rs` ([D] the disc ELF's `DroneModeSettings` 0x29ad40
read out of the ISO's ACTION.ELF). Reachability is unchanged — **rows 1-34 are never
dispatched** (parameter 5 ∈ {0, 100} on every main-map placement and every coder/bot template
[census above]); only row 0 (Normal → Defaults) runs, so the port's row-0 block in
`nf-game/src/drone.rs` `init_properties` plus the NEW-path rebuild is the whole ported
surface, and none of the 32 other bodies needs a port path.

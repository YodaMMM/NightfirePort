# audit-mp-panes — the multiplayer HUD panes against the port

Branch `s/audit-mp-panes` (off origin/master 8c9f02b03). Task: sweep `MPPaneList` and every
HUD_MP* create/update function — each pane's sprites, positions, colours, enable conditions,
writers — read from the decompile and `ee_start_05.bin`; the per-scenario objective icons;
`MP_setPlayerStatus`'s status bits and their readers; fix what is wrong where the draw path
exists; pin each with a test; list anything needing new drawing as a table with template
values.

## 1. The pane list itself

`MPPaneList` @0x2be9e0 (88 bytes, 22 slots [D action-symtab]) is picked over `PaneList`
@0x2be5b8 by `HUD_Init` 0x199e88 when `MPSettings+0x180 != 0` [M 00199e88:36-40]; both lists
are walked 22 wide (`uVar8 < 0x16`), each live entry's create fn at template+0x8 runs with the
template's sprite table at +0x10, and `HUD_Update` 0x19a088 calls the runtime pane's update fn
(pane+0xC) per frame, parking every sprite on layer 0xff while the pane's enable byte
(pane+0x1C) is clear [M 0019a088:22-55].

The 22 slots, dumped from the ELF [D] and cross-read in `ee_start_05.bin` (identical bytes —
the templates are boot-time data) [L]:

| slot | template | create | update | sprites | port |
|---|---|---|---|---|---|
| 0 | MPAmmoPane 0x2be698 | HUD_CreateAmmoPane 0x1a0ed8 | 0x19a7f0 | 3 text | drawn (`t.mp_ammo`, (496,408)/(496,428)) |
| 1 | MPHealthPane 0x2be818 | HUD_CreateMPHealthPane 0x19b390 | 0x19b630 | 8 | drawn (backing/bar/armour + the glow via the SP table) |
| 2,3,5,7,8,11,15,16,20,21 | — | — | — | — | dead in MP (SP-only panes) |
| 4 | MPMsgInfoStatusPane 0x2be868 | HUD_CreateInfoStatusPane 0x1a10b8 | 0x19ca88 | 1 text | drawn ((16,88) = created y 16 + the MP arm's +0x48 [M 0x1a10b8:20-24]) |
| 6 | SightPane 0x2bd5e0 | HUD_CreateShrink 0x1a0bb8 | **0x1a12d0 = empty (8 bytes)** | 3 | drawn (the zoom scope overlay) |
| 9 | RedeemerPane 0x2bdc18 | HUD_CreateRedeemer 0x19dec0 | 0x19e178 | 11 | drawn (missile-cam brackets) |
| 10 | RCCarPane 0x2bdd40 | HUD_CreateShrink 0x1a0bb8 | 0x1a15a0 | 6 | **not drawn — table below** |
| 12 | BloodPane 0x2bd030 | HUD_CreateShrink 0x1a0bb8 | 0x19a6a8 | 2 | drawn (death wipe, incl. the MP cap [M :31-35]) |
| 13 | MPScorePane 0x2be9c0 | HUD_CreateMPScorePane 0x19ed10 | 0x19f0a0 | 7 | drawn extensively |
| 14 | RadarPane 0x2bdd90 | HUD_CreateRadar 0x19f4e8 | 0x19f6e8 | 1 + 24 blips | drawn (ring, blips, name tags) |
| 17 | OICWPane 0x2be140 | HUD_CreateOICWPane 0x19cc90 | 0x19cef8 | 7 | drawn (zoom boot text) |
| 18 | RoninPane 0x2be268 | HUD_CreateShrink 0x1a0bb8 | **0x1a12d0 = empty** | 6 | drawn (remote-view frame) |
| 19 | LaserPane 0x2be390 | HUD_CreateShrink 0x1a0bb8 | **0x1a12d0 = empty** | 6 | drawn (Samurai zoom frame) |

Enable conditions: `HUD_Reset` 0x1a0d38 (called from `HUD_Init`) disables 5/6/7/9/10/11/12/
15/16/17/18/19/20, so a match starts with panes 0/1/2/3/4/13/14 live (2/3 are dead slots in
the MP list — their enable bytes are written but no create/update exists) [M]; the situational
panes are enabled/disabled by the weapon/vehicle/monitor code (`HUD_Enable` 0x1a0e60 callers:
Monitor_Activate 0x1b0058 kills 0 and 1, MP_ReSpawn 0x185368:67 disables 12, the zoom code
enables 6/9/10/11/17/18/19). The Sight/Ronin/Laser panes have NO update fn — their sprites are
pure enable-byte toggles over static create-time rects [M 0x1a12d0 size 8 = `return`].

## 2. What the audit found and fixed

### 2.1 The status bits carried the wrong values (fixed)

`MP_setPlayerStatus` 0x18a518 writes `MPGame + slot·0x30 + 0x26` (param_2 = bits, param_6 =
clear flag) and also maintains the per-team held word `MPGame + team·2 + 0x1a8` (bit 0 flag
out, bit 1 blueprint out) [M :75-96 + the switch arms]. The full bit map, per writer:

| bit | meaning | set | cleared |
|---|---|---|---|
| 0x1 | carrying a CTF flag | MP_FlagUpdate 0x1861a8:178-179 (msg 0x2f) | :127-129 drop (0x30), :87-89 base-return (0x31) |
| 0x2 | carrying the blueprint | MP_BluePrintUpdate 0x1882d8:126-129 (0x32) | :81-84 drop (0x33), MP_BluePrintReachedBase 0x188970:27-29 (0x34) |
| 0x4 | carrying the kind-6 GoldenEye item | MP_GoldenEyeUpdate 0x187568:232 (0x35) | :123-124 strike end (0x37) |
| 0x8 | carrying the kind-7 item | :235-236 (0x36) | :125-126 (0x38) |
| 0x10 | standing on the KOTH hill | MP_KOHUpdate 0x186ae0:115 (0x39) | :96-97 leaving (0x3a) |

Every respawn zeroes the word [M MP_ReSpawn 0x185368:56]. Readers: `MP_KOHUpdate` itself, and
the bot goal machine — `BOTSTATE_pickGoal` 0x126b88:133-135 reads the slot word and the team
held word, `BOTSTATE_checkAttackMove` 0x126a40 and `BOT_validateStateChange` 0x124fb0 test
0x10, `NDrone2_DSTATE_BotGlobal` 0x16e060 reads/clears it. **Nothing in the HUD reads the
word** — the score pane's objective icons ask the object exts' `+0x1c` carrier pointers
(`MP_HasTeamFlag` 0x18b430 / `MP_HasEsponage` 0x18b468 / `MP_ObjTeamHasGEObj` 0x18b4a0 /
`MP_IsAssasin` 0x18b538 / `MP_IsTarget` 0x18b558 [M]) — the port already modelled those.

The port's `MpSlot::status_bits` had only bits 0x1/0x10 and its blueprint rule **wrote 0x1**
(the flag's bit) on touch/deliver/drop. Fixed to 0x2 [M 0x1882d8:126-129, 0x188970:27-29,
0x1882d8:81-84]; the GoldenEye touches now write 0x4/0x8 and the drop/strike paths clear them
[M 0x187568:123-126, 231-236]; `drop_carried` clears each dropped object's own bit instead of
a blanket 0x1; the doc on the field carries the whole map with writers and readers. Pinned in
`crates/nf-game/tests/mp_status_bits.rs` (3 tests, each with a sister that fails on the
wrong bit).

### 2.2 The MP health pane stayed up under the Ronin remote view (fixed)

`HUD_UpdateMPHealthPane` 0x19b630:54-61 hides every pane sprite while ANY of the zoom/vehicle
panes is enabled — it reads the enable bytes of panes 9/10/11/16/17/18 (HUDINFO+0x144/0x164/
0x184/0x224/0x244/0x264). The port hid only for the camera/OICW zooms and the redeemer
missile-cam; the **Ronin remote view (pane 18)** kept the health bar drawn over it. Fixed
(`ronin_mounted()` is the port's stand-in for pane 18's enable byte) and pinned in
`mp_hud_real_live.rs::the_mp_health_pane_hides_under_the_ronin_view` (sister: unmounted draws
the backing; mounted draws nothing) — 4/4 pass. The port has no RC-car-in-MP model, so pane 10
has nothing to gate yet (table below).

### 2.3 Checked against the decompile, already correct (no change)

- **Score pane text assembly** [M 0x19f0a0:47-70 + the sprintf formats read from the ELF]:
  FFA builds `"%s"` of `MP_GetPlayerScore` — whose FFA string is `"\n%d\n%d"` (LEADING
  newline [D 0x30cc18], team games `"%d\n%d"` [D 0x30cc10], Assassin/KOTH `"%d\n%d\n%d"`
  [M 0x189730:17-20]); team games build `"%s\x0a%s"` (team, player) [D 0x30ccd0]; the
  player==0 fallback prints an empty string [M :63-70]. The port's `MpScorePane::build`
  matches all four, including the leading-`\n` FFA line. The outline (colour2) is black
  unless team score AND player score both exist, then the team colour 0xff0000ff/0x0000ffff
  [M :48-56, 70] — matches.
- **Per-scenario objective icons** [M 0x19f0a0:71-180]: CTF → sprite 1 tex 0x03000182 tinted
  0x0040ffff (team 0 viewer) / 0xff0000ff, gated by `MP_HasTeamFlag(obj, team==0)` which
  checks the OTHER team's flag record (`Flags[1]` for a team-0 viewer [M 0x18b430]);
  Assassin → `MP_IsAssasin` → sprite 1 0x03000180, `MP_IsTarget` → sprite 2 0x03000189;
  GoldenEye → items via `MP_ObjTeamHasGEObj` re-texturing sprites 1/2 to 0x300018b/0x300018c
  and FALLING INTO the assassin arm [M :113-127, the jump into LAB_0019f34c]; Blueprint →
  `MP_HasEsponage` → sprite 1 0x300018a; Uplink → all three icons 0x03000185 tinted per
  `MP_getUplinkStatus` 0/1/2 → 0xd22d35ff/0x2d61d2ff/0x7f7f7fff [M :134-180]. All present and
  correct in the port (`MpScorePane::build`).
- **Score pane create arms** [M 0x19ed10:50-123]: Assassin y −0x10/−0x20 on sprites 1/2,
  Uplink y −0x30 on 1/2/6 + x +0x10 steps + the 0x3000185 re-textures, else-arm y −0x18 on
  sprites 1/2 only, text x +0x18 / y −3 then +0x11 unless (no teams, no bit30, not 0x400)
  where sprite 3 hides with y −0x11, sprites 4/5 follow at +0x12. Matches the port, which
  additionally holds [L] PCSX2-measured absolute rects.
- **MP ammo pane**: 3 text sprites, template (100,18)/(100,38)/(100,38) + pane rect (396,390)
  → (496,408)/(496,428)/(496,428), colour 0x7d6d59ff with black shadow, fmt 0x30cc78 (font 2,
  right-justify) [L MPAmmoSprInfo @0x2be610 in ee_start_05.bin]; no strips (the update gates
  them on the pane's sprite count > 5 [M 0x19a7f0:247,275]). Matches.
- **MP info pane**: one text sprite, template y 16 + the MP create arm's +0x48 = 88, x 16
  (the pane data word) [M 0x1a10b8:14-24]; the update is a 5-state machine, state 3 fades
  alpha = time·17 under 15 ticks then saturates [M 0x19ca88:83-97]. Matches.
- **MP health pane numbers**: bar w = uv-w = health/(handicap+100)·128 with the green/orange/
  red thresholds 0.5/0.2 [M 0x19b630:62-90]; armour strip w = armour·2.56, hidden at 0; strip
  at backing +8,+4 [M 0x19b390:58-60]; the damage glow bits 1/2/4/8 → sprites 4-7, layer 0x1e,
  colour 0xff000000|pain [M :30-53] — the port reuses the SP glow table (same textures, same
  layer, same bit mapping) for 1-viewer matches. Matches.
- **Radar create**: 24 blips created hidden (layer 0xff), tex 0x300016f, flags |= 0x900
  (additive + centred), 2×2 [M 0x19f4e8:47-66]; the port's `RADAR_BLIP_CAP` = 24 held.
- **Kill-message / status line / clock**: swept earlier by mp-hud-real and
  audit-sprite-set-text (the time-up format is the plain "%s" at 0x30cbf0 [L ee_start_05.bin
  @0x30cbf0] — already merged). No regressions found.

## 3. Needs new drawing (the port has no draw path)

| pane | what | template values ([L ee_start_05.bin / D ELF]) | when the game shows it |
|---|---|---|---|
| 10 RCCarPane (6 sprites, `RCCarSprInfo` @0x2bdc38, update 0x1a15a0) | the RC-car remote view's overlay: [0] the full-screen solid 0x03000002, colour 0x002000ff, layer 0x26, flags 0x0100 additive; [1] tex 0x03000176 at (176,16), colour 0x00ff00ff, layer 0x27, flags 0x0980; [2] tex 0x03000176 at (272,16), same colour, layer 0x09, flags 0x0027-family; [3] tex 0x0300004c at (224,14), white, layer 0x27; [4]/[5] solid strips (62,128)/(62,448) 128×2, colour 0xffffff7f, layer 0x27 | `BLData+0x878` holds the RC car obj; if its driven cel != 0x2000194, sprites 1/2 park on 0xff and 3/4/5 show at 0x27, else sprite 0 recolours 0x2000ff and 1/2 show / 3/4/5 hide [M 0x1a15a0:8-30]. No ported MP map drives an RC car — for whoever adds one. |
| 1 MPHealthPane sprite 1 | the DEBUG coords readout: text sprite (64,298), font 2, colour 0x8e8e3cff, layer 0xff unless `switch_SHOW_COORDS != 0`, then `sprintf "%.2f %.2f %.2f\nFPS %d\n"` of the obj pos + `RealFrameRate` [M 0x19b630:106-117] | [L MPHealthSprInfo[1] @0x2be6e4] | debug switch only — deliberately not ported. |

(The SP-only panes absent from MPPaneList are correctly absent from the port's MP draw:
`mp_hud_real_live` already pins that the SP health swirl and the rounds strip never draw in a
match.)

## 4. Live checks for the main agent

1. **The score pane objective icons' absolute rects** — the port models (16,376) for the
   else-arm icons, (16,384)/(16,368) Assassin, (16,352)+16k Uplink, from the 2026-10-18
   snapshot; the create arms only ever SHIFT y (−0x18 / −0x10−0x20 / −0x30) and the Uplink arm
   steps x by +0x10 [M 0x19ed10:50-77], so a PCSX2 read of the created sprite rects in a CTF,
   an Assassin and an Uplink match would settle the three [S] notes in
   `MpScorePane::build`. Address: the pane 13 sprite array at `HUD+8+13·0x20+8`, fields +0x24/+0x26.
2. **The uplink icon colour when `MP_getUplinkStatus` returns neither 0/1/2** — the update
   skips the colour store entirely [M 0x19f0a0:155-180]; the getter's documented values are
   0/1/2 only [M 0x18ba80], so this should be unreachable — a gdbwatch on the icon sprite's
   +0x10 during a full uplink match would confirm no fourth value ever lands.
3. **RCCarPane in MP** — if any disc MP map carries an RC car placement, a PCSX2 session
   driving it would confirm the pane pattern above; the port has no model either way.

## 5. Verification

- `crates/nf-game/tests/mp_status_bits.rs` — 3/3 (new pin file).
- `mp_hud_data` 4/4, `mp_hud_pane` 5/5, `mp_match_rules` 18/18, `mp_never_mentioned` 6/6
  (the nf-game binaries that exercise the changed rules/hud code).
- nf-view: `mp_hud_real_live` 4/4 (incl. the new Ronin-hide pin), `mp_clock_gate_live` +
  `mp_bot_objectives_live` 10/10 in one run, `mp_scenarios_live` 10/10, `hud2_live` 5/5,
  `hud_radar_live` 4/4.
- Full suite: §7.

## 6. What's left

- The two §3 table rows stay unported (no MP map drives an RC car in the port; the coords
  sprite is a debug switch).
- The three live checks above.

## 7. Full suite

(to be filled in below before the finish step)

## Usage

not visible to me

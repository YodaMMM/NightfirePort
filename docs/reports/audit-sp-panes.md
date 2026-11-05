# audit-sp-panes — the single-player HUD panes against the port

Branch `s/audit-sp-panes` (off origin/master f503c15c7). Task: PaneList @0x2be5b8 — every
pane's create and update function, sprites, positions, colours, enable conditions and writers,
read from the decompile and ee_start_05.bin; each pane's hide rules against the zoom, vehicle
and remote-view panes; every state of each update's switch; fix what is missing or wrong where
the draw path exists, pin each with a test, list anything needing new drawing as a table with
the template values. Driving panes out of scope.

## 1. The sweep

PaneList @0x2be5b8's 22 slots, the dispatcher `HUD_Update` 0x19a088 (disabled panes park every
sprite on layer 0xff, the update fn runs either way [M 0x19a088:22-67]) and the enable writers
(`HUD_Init` 0x199e88 seeds 1, `HUD_Reset` 0x1a0d38 forces off 5/6/7/9/10/11/12/15/16/17/18/19/
20, `HUD_Enable` 0x1a0e60 writes pane+0x1c/+0x1a with the +0x1f exists gate) were re-walked from
the decompile; every SP update function's switch states were read line by line (two research
passes over 0x19a6a8/0x19a7f0/0x19b9c8/0x19c310/0x1a11c8/0x19d218/0x1a12d8/0x19d3b8/0x19d770/
0x19e178/0x1a1410/0x1a1538 + 0x1a0d38/0x1a0e60 and every `HUD_Enable` caller) and compared
against `crates/nf-view/src/hud.rs`'s `overlay`, `crates/nf-game/src/hud_pane.rs` and
`crates/nf-formats/src/hud.rs`. `docs/systems/hud-panes.md` is the earlier evidence map; the
findings below are the deltas.

**Health pane 0x19b9c8** — pips (armour 0..7 alpha 0.35/0.75/pip0 1.0; health 8..14 alpha
0.15/0.55, colours 0x52a88b/0xc6984e(<4)/0x9d1912(<2), pip-0 blink `frame%30<15` while
pips<0.5), backing alpha f·140 with the pip colour, the hide set (panes 9/10/11/16/17/18
enabled → sprites 0..15 layer 0xff [M 0x19b9c8:124-140, :209-220 — verified directly, incl. the
nested seccam/redeemer/car ordering]), the damage quadrants (bits 1/2/4/8 → sprites 20..23,
colour `flash|0xff000000`, decay 1/frame), the BOND MOMENT sprite (timer BL+0x948, scale-in
with the sine flutter), the debug coords sprite (switch-gated), the third icon (0x2fc168 table,
(400,48), fade over FRAME_RATE·0.5) and the crouch icon (substate 4 → 0x3000173 at (48,48),
release fade) — all already modelled and drawn (hud.rs + third-icon-arm doc). **One delta
fixed** (§2.2).

**Ammo pane 0x19a7f0** — the SP table's 8 sprites: the count text (kinds 0x14/0x1a percent,
0x1f the depleted clip'/max' pair, `%d-%d` of clip and the BL+0x170+type·2 reserve, the +0x68
bit-5 and switch-flash suppressions), the ammo-name "~-~" fallback [S: no disc weapon with a 0
slot label was identified], the weapon-name switch flash (BL+0x95e armed by a pending switch
whose records' +2 class words differ [M :212-217] — the port arms through the pending draw, the
scene.rs block matches), the two bullet strips (scale 16.0 above 87 rounds, the UV crops, layers
0x1c/0x1d), the target-lock sprite 7 (class-'Q' list, LOS, screen project, the 10000.0 scan,
green inside `weapon_data+0x53dc` range with BL+0x95f==2) — the port draws it as the grapple
marker (live-pinned earlier); the substate 0xb/0xc/0x10 arms (the held-laser zoom %, the mounted
heat, the Ronin shot counter) match the port's `laser_held`/`mounted_heat_strip`/
`ronin_shots_strip`. No change.

**Status pane 0x19c310** (shared by panes 2/3/4/21) — all five states re-verified against the
port: 0 polls `Text_UpdateMsg` with every sprite hidden; 1 snaps the bar full open and re-pins
the caps; 2 stages the text (the objective strcmp against label 0x2000053, the NEW/COMPLETE
label pick 0x2000051/0x2000053, the SFX pair scheduling, the image-message arm [M :183-189 —
TXT_MSG+0x10 non-zero re-textures sprite 0; no disc writer was identified, the port's queues
carry text only — noted below]); 3's flash window (label blink `fmod(time,40)<20`, thresholds
+60>300/+60>180), the steady colours 0x785a14ff/0x7d6d59ff and the `time·17` fade; 4's collapse
(mission 0x20, others 0x10 px/frame, caps riding) — the port's `objective_rows`, the four bar
draws and the scene.rs close timers match, including the banner SFX (1532/469 new, 468 complete,
the ScriptCam gate through hud_frozen). No change.

**Air pane 0x1a11c8** — the enable (substate 3/6 via `Player_MonitorAir` 0x191a70:43-50), the
BL+0x8c8&0x10 visibility bit (head bone under the water line, forced on in substate 6 [M
:37-61, re-read directly]), the wire's `100 − BL+0x8cc` green grip meter with the icon, the
`v·2+28` width min 1 — the port's `air_pane` matches. No change.

**Space pane 0x19d3b8** — the level 0x700001b self-enable/disable gate, the per-console lamp
rules (channel k on && deploy≠3 → 0x300017A red, deploy 2 half-red, deploy 1 flashing
0x00ff00ff/0xffffffff, deploy 3 steady green), channel 9 → sprite 0 blink, channels 24/28 →
sprites 9/10 — the port's `space_pane` matches; the MissileDeploy note in the port was [S]
"undecompiled" — **now [M]**: `SpaceMissile_Update` 0x1aa118 is decompiled; its jump table
(PTR_LAB_002fcc30, states 0-4) disassembles to the console missile lifecycle — state 0 counts
down obj+0x14 by the constant at 0x30d0dc then `MissileDeploy[idx−1] = 1` (0x30cd70, symtab);
state 1 waits on obj+0x9c; state 2 spawns and writes deploy 2 (kind byte set) or 3 (kind clear)
at 0x1aa2cc-0x1aa338; state 3 writes the kind byte and parks at state 4 [M, read from the ELF
words at 0x1aa1c8-0x1aa3e0]. The port keeps deploy = 0 (the channel rule only) — §3.

**NightSight/XRay/SecCam/OICW/Camera/Redeemer/Blood** — re-verified against the decompile: the
goggle panes' tint/jitter/battery maths, the sec-cam's viewer+0x236 reset and lamp blink, the
OICW boot's three modes, the camera pane's shutter (BL+0x95b, the 10-tick countdown at alpha
0x28) and the world scan, the redeemer's fuel colour ((ratio−0.85)·3400), the scanline jitter,
the missile-lost icon, the blood wipe's growth and the MP cap — all present in the port. No
change.

**Hide rules** — the crosshair's exits (BL+0x950 != 0, the panes), the health pips' hide set
(9/10/11/16/17/18), the ammo pane's sec-cam disable (Monitor_Activate 0x1b0058:47-49), the
monitor enter/exit pane swaps (0x1afcc8:39-41, :165-167), `Player_SetCamMode`'s pane enables
(9 ← 0xc, 10 ← 0xd, 18 ← 0xf [M 0x195928:25-42]), `Player_HandleDeath`'s HUD_DisableAll +
blood enable, `MP_ReSpawn`'s blood disable — all modelled. The port's health hide covers
9/11/16/17/18; **pane 10 (the RC car remote view) has no port model** — §3.

## 2. What the audit found and fixed

### 2.1 The lens-flare pane (pane 8) was not drawn at all (fixed)

`HUD_UpdateLensFlarePane` 0x19d770 (1872 B) — SP's pane 8 (MPPaneList slot 8 is dead [M
0x199e88]; the SP pane is enabled by `HUD_Init` and never in `HUD_Reset`'s off-set) — draws the
sun's flare: a sticky alpha byte (sprite 0's +0x38) decaying 0x10/frame, rising 0x20/frame while
the sun is line-of-sight visible (`Collide_LineOfSight(head, head−SunVector, …, 0, 8)` sets
BL+0x962, the "Bond sees the sun" flag [M :84-91] — the port already modelled that flag's
ambient-light consumer in scene.rs `ramp_char_ambient`, but never drew the sprites) and its
`View_3DPoint2Screen` position, rescaled about the screen centre by BL+0x8d0 (the zoom
magnification [M Player_Zoom 0x1a1a70:56-57, Player_Init 0x1910f8:60]), lands in (−5,517)×(−5,453)
[M :96-111]; then 7 white additive quads on layer 0x31 at alpha `a`, sizes = the texture's own
dims (the runtime +0x28/+0x2a that `hashtable_set_sprite` 0x1ca3d8 fills [M]) × 3.0/4.0/1.0/
5.0/2.0/0.5, positions mirrored about the screen centre by 0.7/0.5/0.3/−0.1/−0.2/−0.4, sprite 6
the centre glow on the sun itself with size `base·max(0,(int)(256−dist))·0.015625` [M :125-183].
The port had the sun vector, the LOS, the projection and the draw path — only the pane was
missing.

- `LensFlareSprInfo` 0x2bd8d8 joined `HudTables` (`flare`, 7 sprites [L ee_start_05.bin: colour
  0xffffff00, layer −1, flags 0x0d00 with the 0x400 tile-mirror on 0/1/3/4, 0x0900 on 2/5/6,
  uv (0,0,31,31)/(0,0,63,63)/(0,0,0,0)]).
- `HudState::flare_alpha` (new field, own line) holds the sticky byte; `hud_tick` runs the ramp
  with the pane's own LOS ray and the (−5,517)×(−5,453) gate, MP holds it at 0 [M 0x199e88];
  with no sun (`SunVector` 0) the game skips the whole update — the alpha FREEZES, it does not
  decay [M :65-66].
- hud.rs draws the 7 quads (`flare_quads`: the game's (short)(int) truncations, the 0x800
  centring, additive blend 1, tiled_quad for the 0x400 flags).
- Pinned: `crates/nf-formats/tests/lens_flare_pane.rs` (the disc table, 7 rows) and
  `crates/nf-view/tests/lens_flare_pane.rs` (4 tests: the ramp and clamps, the strict gate
  bounds, the geometry/truncations incl. sprite 6's distance shrink, the empty cases).

[S] notes: `Sim::zoom` stands in for BL+0x8d0; whether the game's `View_3DPoint2Screen` is
itself zoom-aware (double-scaling the flare under a scope) is a live check below.

### 2.2 The health pip-0 blink skipped the pane fade (fixed)

`HUD_UpdateHealthPane` 0x19b9c8:185-200: the low-health blink REPLACES pip 0's factor
(0.55/0.15 on `GameState+0x52 % 30 < 15`) BEFORE the pane fade multiply — the port set the raw
factor and skipped `f` (visible only with the pane-fade option on, f < 1). Fixed as the pure
`health_pip_alpha(i, pips, frame, fade)` and pinned in the same nf-view test file (a sister
assertion holds the pre-fix shape wrong).

## 3. Needs new drawing / modelling (no port site today)

| what | template values ([L ee_start_05.bin / D ELF]) | when the game shows it |
|---|---|---|
| 10 RCCarPane (SP remote view, `RCCarSprInfo` 0x2bdc38 × 6, update 0x1a15a0) | enabled by `Player_SetCamMode(0xd)` [M 0x195928:41]; the layer swap `[0xff,0xff,0x27,0x27,0x27]` + colour 0x0020ff vs `[0x27,0x27,0xff,0xff,0xff]` + 0x2000ff by the held object's cel-group test against 0x2000194 [M 0x1a15a0:8-30] (`hud_pane::car_pane_layers` holds the swap already) | when Bond drives the RC car gadget in SP; the port has no RC-car model (same as the MP sweep's table row) |
| RedeemerPane sprite 10, the copter lock bracket | `CopterList` walk, screen clamp, alpha 0x27, blink off on `frame&8` while \|x−256\|<32 and \|y−224\|<32 [M 0x19e178:140-183] | a copter on screen during the guided-missile cam; the port has no copter in its missile-cam levels |
| MissileDeploy console states (space level 0x700001b HUD lamps) | deploy 1 → tex 0x3000177 flashing 0x00ff00ff/0xffffffff; deploy 2 → channel red 0xff000080; deploy 3 → steady 0x00ff0080 [M 0x19d3b8:22-72; the writer now [M]: `SpaceMissile_Update` 0x1aa118's states above] | the space station's missile consoles between arming and launch; the port keeps deploy = 0, so only the channel rule shows |
| Status-pane image messages (TXT_MSG+0x10) | `hashtable_set_sprite(sprite0, msg+0x10)` + clear text [M 0x19c310:183-189] | no disc writer of +0x10 was identified in this pass — if one exists the port's text queues need an image arm |

## 4. Live checks for the main agent

1. **The flare's zoom scaling** — load 0700000c (the desert; sun_seen=1 at its start [L
   PCSX2 BLData+0x962]), aim at the sun, zoom the sniper to 8×: does the flare stay inside the
   scope view (View_3DPoint2Screen NOT zoom-aware, the port's model) or fly off screen (it is)?
   A gdbwatch on `View_3DPoint2Screen` 0x1e7708's viewer+0x100/0x104 while zooming settles which
   input the projection uses.
2. **The flare alpha ramp** — gdbwatch BLData+0x7e0 chain pane 8 sprite 0's +0x38 (the HUDINFO
   at BL+0x7e0, pane record 8 at +0x108, sprite array at +8) walking in/out of the sun: expect
   +0x20/tick up, −0x10/tick down, frozen (not decaying) on a level with `SunVector` 0.
3. **The health pip-0 blink fade** — with the pane-fade option (PlayerSetting+0xB) ON and health
   ≤ 6: the blink contrast should DIM as BL+0x8bc decays (both arms × f), not stay at full
   0.55/0.15 alpha.
4. **MissileDeploy** — on 0700001b, gdbwatch 0x30cd70..77 while a console missile arms/launches:
   expect 1 (flashing) → 2 or 3 (by the console's kind byte at 0x270fc90+obj+0x12) → stays; a
   screenshot per state pins the lamp colours.

## 5. Verification

- New: `crates/nf-formats/tests/lens_flare_pane.rs` 1/1; `crates/nf-view/tests/lens_flare_pane.rs` 4/4.
- The binaries that exercise the changed code (`HudTables`, hud.rs overlay, scene.rs HudState/hud_tick):
  nf-formats `goggles_panes` 2/2, `hud_pane_census` 3/3, `hud_pane_creators` 2/2; nf-view
  `hud2_live` 5/5, `fix_hud_parity_live` 10/10, `ammo_pane_live` 3/0 (+1 ignored), `dead_pane_live`
  0/0 (+2 ignored), `fix_space_pane_live` 3/3, `hud_level_start_live` 7/7 — 33 passed, 0 failed.
- Full suite: **did not fit** — the sandbox disk (9.9 GB, 3.7 GB free after cleanup) ran the
  linker out of space (`ld terminated with signal 7 [Bus error]`) relinking the ~115 nf-view
  test binaries; two foreground attempts died in the link stage before a single binary ran. The
  changed code's binaries all pass (above); the walkreal_*_live set was NOT run — it shares
  `crates/nf-view/tests/common/` only through unchanged helpers, but per the merge rules treat
  that as unverified until a machine with room runs it.

## 6. What's left

- The four §3 rows.
- The four live checks above.
- The full-suite run on a disk that fits (the walkreal set first).

## ntfy failed

All three fallbacks, in order, with their output:
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "audit-sp-panes: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'audit-sp-panes: done', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "audit-sp-panes: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out with no output (killed at the 20 s deadline).

## Usage

not visible to me

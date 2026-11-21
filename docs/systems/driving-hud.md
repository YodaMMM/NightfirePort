# Driving HUD — what is drawn, where, and from which state

Code: `crates/nf-game/src/driving_hud_draw.rs` (layout + per-frame draw list, headless) on top of
the gallery reader `crates/nf-formats/src/gal.rs` (docs/formats/hud-gal.md). The older snapshot
model `crates/nf-game/src/driving_hud.rs` (car readout, timer/objective commands) is unchanged.
Tests: the module's unit tests, `crates/nf-game/tests/drive_hud_gal.rs` (real gallery, NF_ISO),
`crates/nf-formats/tests/gal_disc.rs`.

Evidence: [M] DRIVING.ELF decompile/disassembly (EE addresses), [D] disc data, [S] inferred.
Screen space is 640 × 448 (NTSC) or 640 × 512 — the renderer's +0x50/+0x54 set by
`FUN_001abee0` [M]; HUD centre = (+0x454, +0x458) = half of those (`FUN_0023c858`) [M].
Colours are GS words (R in the low byte, 0x80 = full).

## Build (`FUN_0023ae30`) [M]

Loads `hudNTSC.gal` (and `data\loading\loading.sfn` as its font), finds the animations
`hud_base`, `hud_msg`, `pauseN`, `paus_err` and about 150 elements by name into fixed HUD slots.
Names it asks for that the gallery does not hold get a null slot: `radar`, `radar_border`,
`radar_stencil`, `pulse` [D — `gal_disc.rs` checks they are absent]. Slots the ported passes use:

| HUD field | Element(s) | Gallery key (x, y) [D] | After the build moves |
|---|---|---|---|
| +0x14 / +0x18 | `weapon` / `ammo` (text) | (605, 420) / (605, 400) | y −10 |
| +0x1c.. | `iris1..iris7` (health) | e.g. iris1 (46, 361) | y −10 |
| +0x38.. | `a007`, `arm1..arm7` (armour) | a007 (53, 387) | y −10 |
| +0x58 | `healthBorder` (43 × 43, mirror 3) | (68, 404) | y −10 |
| +0x78.. | `gadget0..gadget5` | (540, 350) | y −10 |
| +0x74 | `stylin` | (510, 40), scale 1.3 | — |
| +0x25c | `txt` (`hud_msg`) — message font | (260, 54) | — |
| +0x1f0 | `scratchTxt` (`pauseN`) — timer font | — | — |

Widescreen (renderer +0x5c on, which makes +0x58 = 2): x +78 for weapon, ammo, stylin, gadgets;
x −78 for iris, a007/arm, healthBorder (0x23c2c0-0x23c398). A third branch (±110 / +60) needs
+0x5c on with +0x58 ∉ {1, 2}, which `FUN_001abee0` never produces; not ported.

## Frame (`FUN_0023cf88`) [M]

With HUD state +0x2b0 == 1, in this order (ported = ✔):

1. `FUN_00246c48` mission status banner — not ported.
2. `FUN_002472b0` stunt award (`stylin`) — not ported.
3. ✔ `FUN_00240c98` weapon + ammo.
4. `FUN_0023f328` crosshairs and target markers (per weapon id) — not ported.
5. ✔ `FUN_0023da30` health / armour meter.
6. ✔ `FUN_002469b8` message line.
7. `FUN_002470f0` — empty.
8. `FUN_00245ab8` pickup line — not ported.
9. ✔ `FUN_00244f88` gadget pop-up.
10. ✔ `FUN_0023e580` mission clock.

`dt` = hud+0x368 = ticks since the last update × seconds per tick (`FUN_0023cbc0`) [M].

### Weapon + ammo (`FUN_00240c98`, disassembly 0x240c98-0x240dfc)

Skipped while hud+0x460 ≠ 0, hud+0x410 (long message) ≠ 0 or `FUN_0024a1e0(FUN_0024a1d0())` ≠ 0.
Reads the current weapon-list entry (DAT_0033a530: +0x10 base, +4 index, 0x54 stride; +8 id,
+0x18 ammo). Ammo: `"%d"` of max(ammo, 0), drawn unless id == 0x64b with ammo ≠ 0; weapon:
the label string of the id (`FUN_0024b1a0`). Both right-aligned (x = element x − text width),
at the element y, scale 1.0, colour 0x7fb3dbfa, shadow 0x46000000 one pixel down-right
(`FUN_00244398`). In 4:3 the right edge is x = 605, ammo at y = 390, weapon at y = 410.

### Health / armour meter (`FUN_0023da30`)

Same skips as above. h = world+0x378 / +0x37c, a = world+0x380 / +0x384.

- Visibility f: 1 while the show timer +0x2cc > 0 or hud+0x434 is set; otherwise 2 × fade
  (+0x2d0) or 0. While h ≤ 0.25 the blink timer +0x2d4 overrides: f = 2 × blink (0.5 s cycle).
  At h ≤ 0 f = 1 and no segments are lit.
- Colour by h: > 0.5 0x468ba852, > 0.25 0x464e98c6, else 0x46252cb0; alpha = round(0x46 × f);
  dim twins with alpha round(0x13 × f).
- Draws: all 7 iris dim; the first (int)(h × 7) full; if h < 1 the next one at alpha
  round(full alpha × fraction); the first (int)(a × 8) of a007, arm1..7 at alpha round(127 f);
  if a < 1 the next at round(round(fraction × 127) × f); healthBorder (4 mirrored quads) full.
- Timers after drawing: show −dt, at 0 → fade 0.5; fade −dt to 0; blink −dt and re-arms 0.5
  while h ≤ 0.25, else 0. A change of h or a re-arms the show timer to 3 s. So the meter shows
  for 3 s after any change, fades over 0.5 s, and blinks at low health.
- With hud+0x2a8 set, a drop of a 1/32 step starts `SFX_Bond_body_hit`, `…2`, `…3` in turn.

### Message line (`FUN_002469b8`; set by `FUN_00245890`)

`FUN_00245890(duration, hud, label)` (only while hud+0x314 == 0): hold = duration, label set,
long flag = (4:3 and the text is longer than 32 bytes); the fade timer is not reset. Drawn while
hold > 0 (f = 1) or fade > 0 (f = 2 × fade): a box at x − 4, y 376, width + 12, height + 18,
colour 0x20000000, then the label text centred on x = 320 at y 380, scale 1.2, colour
round(127 f) << 24 | 0xb3dbfa, shadow round(32 f) << 24. Hold ends → fade 0.5; fade ends → the
long flag clears. While the long flag is set, weapon/ammo and the meter are not drawn.

### Gadget pop-up (`FUN_00244f88`; set by `FUN_00244f48(duration, hud, slot)`)

Slot < 6 → gadget[slot], else gadget3. While the duration runs: alpha = round(2 × blink × 127),
s = 2 × slide, t = (3 − 2s) s², position = t × (centre x − w/2, centre y − h) + (1 − t) × home.
It starts at the screen centre, fades out twice over 0.5 s (blink 0.5 → 0, twice), then slides
home over 0.5 s and stays until the duration ends. Colour alpha << 24 | 0x7f7f7f.

### Mission clock (`FUN_0023e580`, disassembly 0x23e580-0x23e7b0)

While hud+0x464 ≥ 0 (set by `FUN_0023e568`, cleared by `FUN_0023e570`) and the slot's integer
seconds (value / ticks per second) > 0: a box centred on x = 320 at y 33, as wide as `"00:00"`
at scale 1.4, height 2 × height − 2, colour 0x4605050f; the text `"%2d:%02d"` at y 35, placed by
the width of `"00:00"` (not of the text). On a countdown (slot rate < 0, `FUN_00209d48`) under
10 s the text is red 0x7f0505c8 and `SFX_timerbeep` plays each new second (hud+0x2c8, 10 at
build, is never reset — a second countdown in the same HUD beeps only below the last beeped
second) [M].

## What the port does not have (open)

- The four unported passes above (status banner, stunt award, crosshairs/targets, pickup line).
- Fonts: text width/height (`FUN_00237ca8`/`FUN_00237ce0`) and the font binding of a text
  element — the draw list carries the rule (right-aligned, centred, measured string) and the
  element name; the draw side measures.
- Label strings (`FUN_0024b1a0`) — the draw list carries label ids.
- What sets hud+0x434 (from `iRam00100998`), hud+0x2a8, hud+0x460 and the
  `FUN_0024a1e0(FUN_0024a1d0())` gate, and the initial value of hud+0x370 (not written by the
  init; 0.0 in the port [S]) and of DAT_0033b1c8.
- Live check wanted: the meter's on-screen look and the clock box, against a running game.
  Nothing in this file has been compared to a capture.
- Wiring: nothing in the app calls `DriveHudDraw` yet; it needs the driving mission's world
  health/armour, weapon list and timer slot.

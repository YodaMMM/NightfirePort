# hud-ammo-under-label — the ammo bar while the weapon-name label shows

Branch `s/hud-ammo-under-label` (from origin/master 6042ee53). One worker session, 2026-11-01
18:33–20:20 UTC.

## What was done

The port hid the rounds strip whenever the weapon-name label timer (`BL+0x95E`) ran, from the
straight reading of `HUD_UpdateAmmoPane` 0x19a7f0. The 07000011 f42 capture shows the PS2 drawing
the ammo bar while the weapon-name label is up at a level start. The port now matches that, with
the switch behaviour preserved after the pane has rested once:

- **During the first label** (from load until the first timer==0 tick) the two strip sprites draw
  in their created state: template layers 0x1c/0x1d, the template rect — pane (496,394) plus the
  template (−25,−64) = screen (471,330)..(496,394), 25×64, uv (230,0,25,64), colours 0x7f7f7fff /
  0x4040407f [D AmmoSprInfo[5]/[6]; M `HUD_CreateDefault` 0x199b70 copies template+8 → rt+0x38 and
  adds the pane origin into rt+0x24/26]. This is what the capture shows.
- **After the first rest frame** the strips position through the BulletImg math as before
  [M 0x19a7f0:248-269, 276-296], and a later switch takes them off again (layer 0xff) [M 0x19ad0c /
  0x19ae48 — both stores verified in the disassembly; L the 2026-10-12 live pass on 07000005: a
  mid-game switch shows the weapon name alone].
- The count text still hides during the label (the `sh $zero,0x20(s6)` delay-slot store at
  0x19aa9c) and the fire mode still swaps with the name [M 0x19a7f0:218-226] — unchanged.

Implementation: a new `HudState::ammo_strips_positioned` latch (nf-view/src/scene.rs, on its own
line at the end of the struct), set in `hud_tick` on the first tick the timer reads 0 — the game's
first timer==0 strip-block run is what positions the sprites [M 0x19a7f0:248-269]. The draw
(nf-view/src/hud.rs) picks: MP/security-cam → no strips [M 0x19a7f0:247,275 count gates]; switching
and not yet positioned → the created state; otherwise the positioned draw (hidden while switching).

## The evidence conflict, stated honestly

- [M] the pane fn's strip blocks write 0xff to the strips' layer every frame the timer runs, and
  `Sprite_BuildList` 0x1de970 drops layer-0xff sprites from the draw list (signed −1 compare). I
  verified all of this in the disassembly (`sb $v0,0x38($s5)` at 0x19ad0c with `$v0=0xff`,
  `sb $v0,0x38($fp)` at 0x19ae48; `$s5`/`$fp` = sprite array [5]/[6] from the prologue loads at
  0x19a8d0/0x19a8d8). By [M] alone the bar cannot show during the label.
- [L] the driver's 07000011 f42 capture shows the bar while the label is up; [L] the 2026-10-12
  pass shows it hidden on a mid-game switch. Both live observations are kept by the fix: the bar
  shows on the FIRST label only (the strips' created state), later switches hide it.
- The mechanism for how the strips escape the 0xff at a level start is **[S] unverified** — the
  port models it as "the strip blocks have not positioned them yet" via the latch. The capture is
  the pin; the live checks below settle the mechanism.

## Tests

- New pin: `hud_level_start_live::the_first_label_shows_the_rounds_strip_and_it_remains_after`
  (07000011): ticks 0..58 draw ≥2 strip quads exactly at the created rect; ticks 59+ the timer is 0
  and the bar remains (sister); a later poke of the timer hides the strips again (the [M] switch).
- Test fixed, reason: `dev_tools_live`'s `loaded()` helper now runs the first label out before the
  strip checks — its clamp check tests the POSITIONED strip, and under the new behaviour the
  first-label frames legitimately draw the 25×64 created-state strips.
- No existing assertion was weakened: `hud_playtest_live::a_weapon_switch_shows_the_name_alone…`
  passes unchanged (its poke happens after the pane has rested).

## What ran (totals)

- `hud_level_start_live`: **7 passed, 0 failed**.
- `ammo_pane_live`: **3 passed, 0 failed, 1 ignored** (the EE-snapshot probe needs the local
  snapshot).
- `fix_hud_audit_live`: **6 passed, 0 failed**.
- `hud_playtest_live`: **5 passed, 0 failed**; `dev_tools_live`: **6 passed, 0 failed**.
- `cargo test --offline --release -p nf-game -- --test-threads=1`: **1028 passed, 0 failed, 9
  ignored** across 100 binaries, NF_ISO set.
- The full four-crate command was started but does not fit one foreground window in this sandbox
  (background jobs are killed between tool calls here); nf-formats/nf-audio therefore did not get a
  fresh run this session. The fast runner (`tools/test/suite.py`) could not complete either — same
  kill — `suite.out` stayed empty. **The next session should run the full four-crate command and
  suite.py once on this tree before merging.**

## Live checks for the main agent

1. Settle the mechanism: fresh boot of 07000011 (`goto_level.py 07000011 --settle 1`), then
   `gdbwatch` the two strip sprites' layer byte for the first ~70 frames. The sprite pointers are
   pane[0]+0x08's array entries [5]/[6] (pane list at HUD 0x016c8630 in the 2026-10-12 session);
   expected if the port is right: 0x1c/0x1d (created) until ~frame 60, then 0x1c/0x1d at the
   positioned rect; the old reading predicts 0xff from frame 1.
2. A screenshot pair at frame 42 (fresh boot) and at frame 120 — the bar present over the label,
   then the positioned bar with the count text.
3. If (1) shows 0xff from frame 1, the capture's bar is NOT the strip sprites — please grab the
   f42 frame with the sprite draw list dumped (`psiDrawSprites` 0x120e68's list) so the real
   element can be identified; the port's latch would then be wrong and should be reverted.

## What's left

- The full four-crate run + suite.py on this tree (above).
- The strips-escape mechanism is [S]; the live checks decide whether the latch stays or the fix
  moves to whichever element the capture really shows.

## Usage

`not visible to me` — this session's environment shows no token, cost or context-window figures.

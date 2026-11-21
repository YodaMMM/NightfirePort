# s/drive-sniper-phase — the MIS01 opening sniper section (data + logic)

No change to the shared random stream. No draw or view changes (`crates/nf-view/src` untouched).

## What was found
- The section is the **POV weapon mode** of DRIVING.ELF: the player vehicle is the
  `helicopter` (the MSet name; its `.atr` names `vanquish` as `SECONDARY_TYPE`), Bond is a
  separate gunner actor on its `AIC_HSNIPE_POS` mount, and the mode flag (weapon manager
  +0x350) is set by the scripted event `ESetAutoDriveCamOn` [M+D].
- The `MSet` table is the **weapon loadout**, not objective channels: Missile 30 + Sniper Rifle
  24 [M 0x205498/0x20d378/0x20bbe0, D]. The mode swap selects the Sniper Rifle [M 0x20c078],
  which matches the HUD in the original [L].
- The mission script: `el` event lists + `sr` strings in the `Map ` window; `Trgr` +0x18 and
  `Rule` +0x0c are list references. Start trigger 684 (`EStartMission`) → trigger 685 → list
  686 (`EControlToSpline` spline 0, `ESetAutoDriveCamOn(-60,-20)`, `EChangeStage(19)`,
  `snipeintro` music, objectives 2800-2803). Trigger 727 → stage 13 → rule 34 → list 789:
  `ECustomTransition("paris")` (rifle removed, Gun 500, Missile 30), mode off, video
  `paris2.mad`, car reset at trigger 683 [M+D]. 54 event kinds named via typeinfo [M].
- drive-mission-scripts.md's MSet/Rule reading is corrected in a note there (the old `Mission`
  model is left as it was).

## Code
- `crates/nf-formats/src/drive_script.rs`: event lists, references, trigger/rule records,
  `MapScript::load`, AIEl minimal records, the event-name table.
- `crates/nf-game/src/driving/pov.rs`: weapon table, refire table, `Loadout` (reset, MSet
  registration, mode swap, POV fire, reloads, the `paris` transition), `PovGunner` (the
  vtable state machine with the per-kind anim tables), `PovSession` (mode flag + events).
- Doc: docs/systems/driving-mis01-opening.md.

## Tests (NF_SILENT=1, NF_ISO set)
- `nf-formats --lib drive_script`: 4 passed.
- `nf-game --lib driving::`: 68 passed (10 new in `driving::pov`).
- `nf-game --test driving_sniper_live` (new, disc): 4 passed.
- `nf-game --test driving_mission_live`: 5 passed (they skip without `NF_DRIVING`).
- Failing-on-wrong-values check: three mutations in pov.rs (clip cap applied with the mode off,
  the bolt-sound window moved, the transition's POV range shifted by one) → the first two
  failed `mset_load_selects_the_missile_and_pov_swaps_in_the_rifle` and
  `fire_spends_a_round_and_waits_for_refire_and_the_gunner`; the third is equivalent (index
  0x17 is never registered), so no test can catch it.
- No driving walkreal binary exists; nothing was run in a game window.

## What a player would notice
Nothing yet: the port's driving view still starts in the Vanquish. This branch supplies the
data and logic a view needs to open MIS01 in the helicopter with the Sniper Rifle and hand over
to the Vanquish.

## Unproven
- What fires rule 34 (`SRuleSpeed [5, 0, 1.0]`): the rule evaluators are not decomposed.
- Which hand-over route a normal run takes (rule 34 → list 789 or trigger 741 → list 742).
- What makes triggers fire on contact; the event queue's run order (the port runs a list in
  stored order).
- The `ECustomTransition` vehicle swap target (vtable +0x294/+0x8c not decomposed; the
  Vanquish is inferred from the `.atr` secondary) and the meaning of `DAT_0032e690`.
- Not ported: aim matrix/scope sway, hit tests, pursuer AI commands, spline follow, movie.

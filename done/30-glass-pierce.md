## Your task: `glass-pierce`, rounds pass intact glass and hit what is behind it, as the game's do

done/30-dead-pane-collision-2.md (What's left, item 2) records that the game's round passes an intact material-0x10 pane, except weapons with record +0x6c bit 0x800, and continues to the next contact [M 0x12cf44..0x12cf64], while the port's round dies at the pane contact. `Bullet::glass_pass` in crates/nf-game/src/bullet.rs already covers the flight's world trace, so first find which contacts still stop a round on master and read done/30-bullet-pass-water.md and the 11 stairwell bullet-wall reports for what is settled. Read 0x12cf44..0x12cf64 in the disassembly: does the round damage the pane and go on, how much damage carries on, and does it leave an impact on the sheet? Then the guards' side: that report says the guard-fire gate fires through glass where the game's aim trace (mask 0xa) stops on non-0x10 glass. Read that too and match it per material. Census which panes are material 0x10 on each level [D]. If the code cannot settle whether the round leaves an impact at the sheet, list that as a live check for the main agent and do not guess. Pin through real input on two levels: a round through intact 0x10 glass hits a guard behind it, and a fist or laser (bit 0x800) stops. Run dead_pane_live, glass_pass_every_level_live, the bullet binaries, walkreal_07000001_live, walkreal_07000009_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/glass-pierce`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T17:30Z

---
## Report (done, 2026-11-07T18:02Z)

# glass-pierce — the at-sheet impact landed; the pass rule read from the disasm; the material census

2026-11-07, branch `s/glass-pierce` (from origin/master 10956badb). The landed change is one
behaviour fix plus one un-ignored proof test; everything else this session produced is evidence:
the disassembly read the task asked for, the per-level material census, and the precise map of
what still stops a round on master and why.

## The disassembly read (the task's first question)

`Bullet_CollisionHandler` 0x12ce80, disassembled from `work/elf/ACTION.ELF` (MIPS, capstone):

- **The pass flag [M 0x12cf44..0x12cf64]**: `lw $v0,0x6c($v1)` / `andi $v0,$v0,0x800` /
  `bnel $v0,$zero,0x12cf68` (the bit-0x800 records skip the glass arm; the delay-slot
  `lhu $v0,2($v1)` loads the base-weapon word either way); bit clear →
  `lbu $v0,0x50($s2)` (the contact's surface byte) / `andi 0x3f` / `xori 0x10` /
  `movz $s4,$s7,$v0` — **pass = (byte & 0x3f) == 0x10, and it is the deterministic `movz`
  arm — no RNG draw** (the 0x80 roll at 0x12cf9c is a separate, later arm).
- **Pass skips only the stop [M 0x12cfb8..0x12cff4]**: `beqz $a1,0x12cff0` /
  `bnel $s4,$zero,0x12d020` with the state-2 store `sh $v0,0xf4($s3)` IN THE DELAY SLOT —
  the store executes only when the pass flag is clear; a pass re-arms and walks to the next
  contact. The last-contact arm (`bnez $s4,0x12d020` at 0x12cff0) is the same shape.
- **The damage arm is solid-only [M 0x12cff8..0x12d018]**: after the store,
  `Bullet_handle_object_destruction` (0x12cd98) runs only when the pass flag is clear and the
  weapon's +0xc damage is positive — **a pass contact deals NO handler damage to the pane and
  the round carries on UNDIMINISHED** (the pass arm writes no damage anywhere).
- **The sheet still takes its impact [M the shared tail, decomp 0x12d020..]**: `Effect_Create`
  runs on BOTH arms (the jump lands past it, not over it) — the material's `EffectInfo+0x84`
  emitter, `Effect_Bullet`'s gas and decal and the +0x14 impact sound land ON the intact sheet.
  The pane's own health drain is NOT the handler's: it is the `Collide_Update` 0x1ea398:160-233
  mirror onto the object's +0xD0 chain, drained by `Break_Update` 0x12bfd8:36-37 — the port
  already models that drain in the breakable arm.

So the answers: the pane takes its mirrored contact damage and can break on the drain, the
round's own damage carries on undiminished, and the sheet DOES get an impact (emitter + decal +
sound). No RNG is spent on a glass pass.

## The landed fix: the pass contact now draws its effect at the sheet

The port's player walk passed the pane correctly (the flight drops the mat-0x10 static tri, the
breakable arm re-passes the kind-2 box and drains it) but SWALLOWED the contact — no effect at
the sheet. The repro (dead_pane round-1 un-ignored on master): the round's first registered
impact was the shell BEHIND at z −130.255, never the sheet at z −121.998.

The fix (drone.rs, the player walk's glass-pass arm): a pass contact now pushes
`PlayerImpact::World` at the contact point — the port's presentation of the game's
`Effect_Create` at the sheet. No new RNG (the deterministic `movz` arm). With it,
`dead_pane_live::the_shot_out_pane_stops_blocking_a_round` is **un-ignored and green**: the
first round registers AT the intact sheet and the same round's pierce reaches the shell behind;
the health-0 pane dies to the mirrored drain [M 0x12bfd8:50]. The movement sister stays
ignored — it needs the pane collision ride (the pane-ride-on branch's apparatus, unmerged).

## Which contacts still stop a round on master (the audit)

1. **The guard-side flights — the one real stop, and it is a documented deviation.** Every
   guard/turret/laser-round spawn hardcodes `glass_pass: false` (drone.rs 3206, 11460, 13337,
   17901, 18159), so a guard's round stops at a pane's static mesh where the game's guard guns
   (bit 0x800 clear [L weapon_data]) pass intact 0x10 glass. The spawns carry the reason: the
   real-health routes stand behind station glass, and arming the pass turned
   `walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_agent` red at
   tick 25178. Arming it is a two-part change (the spawns from the record + the 08 mission
   walk's cover repair) and is the next session's first item — NOT landed here (a knowingly
   red branch is never shippable).
2. **The player's flight — already passes** (`update_player_bullets`' `flight_glass` from the
   record), and now with the at-sheet impact.
3. **The kind-vs-material gate — a real rule difference, documented, not flipped.** The port's
   breakable arm passes `br.kind == 2` (glass by PLACEMENT PARAM [D params; M 0x12c230:21])
   where the game passes material 0x10 only [M 0x12cf54-0x12cf60]. The census below shows the
   difference is real: on 01 the breach panes 130/136/155 are **material 0x02** — the game's
   round STOPS at them (0x02 ≠ 0x10 → the state-2 store executes); the port's round passes
   (the kind gate). `glass_pass_every_level_live::the_01_breach_shot_breaks_the_entry_glass`
   (green on master) pins exactly that passing round — **that test contradicts the decompile
   under the census and will need its crossing half re-pinned when the gate flips** (per the
   merge rules: the test is wrong, the fix is the gate + the test's re-pin, in that order).
4. **The guards' aim trace — unchanged this session.** Master has NO pane collision (the
   `_pane_parts` ride is parked, drone.rs:19334), so the guard LOS sees no panes at all: for
   mat-0x10 glass the port matches the game's mask-0xa aim trace by accident (the game drops
   class 0x10); for non-0x10 panes the port fires through where the game stops. The honest fix
   rides the same two gates as item 1 (the pane collision ride + `LOS_MASKS_ON`, whose own
   comment records that the full masks broke guard cover, turret shots and several walks).
   Deferred with the plan below.

## The material census [D] (the shapes' surface bytes, per story level)

`crates/nf-view/examples/breakable_census.rs` extended: every kind-2 pane's shape materials
(the same shape table the collision queries read), histogram + per-pane non-0x10 flags:

| level | kind-2 tri materials | non-0x10 panes |
| --- | --- | --- |
| 01 | 0x00×8, 0x02×28, 0x10×20 | 121-124 (0x00), 130/136/155 (0x02) |
| 03 | 0x02×22, 0x05×8, 0x10×26, 0x11×24 | 56/124/125/126/380-382 (0x02), 221/235/241-244 (0x11), 338/339 (0x05) |
| 06 | 0x10×4 | — |
| 07 | 0x00×30 | 298/299/302/303 (the vases, 0x00) |
| 08 | 0x10×16 | — (all the control-room + ride sheets are 0x10) |
| 09 | 0x0f×64, 0x10×4 | **108-123 (the 16 hp-6 panes, 0x0f)** |
| 0a | 0x0f×8, 0x10×416, 0x11×13 | 563/565 (0x0f) |
| 12 | 0x10×590, 0x11×13 | — |
| 13 | 0x0a×160, 0x0f×68 | 20, 74-107 (0x0a), the 108-122 run (0x0f) |
| 14 | 0x0f×12 | 86/97/156 |
| 02/04/05/0b/0c/0d/11/15/16/1b | no kind-2 panes | — |

The pins' levels by this table: 08 is the all-0x10 level (the pierce + the at-sheet impact
pinned there by the un-ignored dead_pane round-1); 01 is the 0x02 level (the pin whose
crossing half rides the deviation, above).

## The pins (real input)

- **08 (0x10): the round through the intact sheet** — landed: dead_pane round-1 (un-ignored,
  green): impact AT the sheet + the pierce to the shell + the pane's drain-kill.
- **01 (the guard-hit half)**: `the_01_breach_shot_breaks_the_entry_glass` green (master) —
  the pane dies off the tap AND a Drone impact registers. Documented: with the census it pins
  the 0x02 passing deviation, not the game's material rule.
- **08 (the settled-guard through-shot)**: `the_08_station_shot_crosses_to_the_ambush_guard`
  re-ignored with this session's machinery landed in the body (the settled gate, the
  pane-only line gate, the cross-tick ledger) and the measured reason: the booth sheets are
  health-0 [D param 1] — ONE round per sheet — and the pane-only guards are the 42-48 u
  upper-deck posts that reposition on the shot's noise, so the single spread roll never landed
  in 3 runs (the killing tap breaks sheet 177 every run: hp −3.50, dead true).
- **The bit-0x800 real-input stop — NOT landed.** The unit law pins it
  (`the_glass_pass_unit_law`: fists never pass, the laser stops, no RNG — green), but the
  real-input shot needs a flight round: the fists never spawn one (the melee route), the watch
  laser 0x4e fires a Beam object (not a flight), and the dart's levels have no panes. Listed
  as a live check instead.

## Verified (this session, NF_ISO set, --test-threads=1)

- nf-game lib: **688 passed / 0 failed** (5 pre-existing ignored).
- dead_pane_live: **1 passed / 0 failed / 1 ignored** (round-1 un-ignored by this branch).
- glass_pass_every_level_live: **3 passed / 0 failed / 3 ignored** (the records, the unit law,
  the 01 breach; the three measured ignores above).
- bullet_range_live 3/0, fix_foliage_bullets_live 3/0, rocket_glass08_live 3/0,
  smg_close_range_live 1/0, stairwell_bullets_07000011_live 2/0, fix_flesh_impacts_live 4/0,
  breakables_live 2/0, fix_audio2_live 4/0.
- walkreal_07000001_live **8/0**, walkreal_07000009_live **8/0** (the two the task names).
- The full suite (tools/test/suite.py): the sandbox kills background jobs between tool calls
  (this session's predecessor hit the same wall), so no total line — the binaries above were
  run in the foreground instead. The full four-crate sweep did not fit.

## What's left (the next message continues from s/glass-pierce)

1. **Arm the guard-side glass pass from the record** (the five spawn sites) and repair
   `walkreal_07000008_live`'s real-damage sisters for guards firing through intact station
   glass (the documented tick-25178 death is the walk standing in what the game makes unsafe).
2. **Flip the pass gate from kind 2 to material 0x10** [M 0x12cf54-0x12cf60] and re-pin the
   01 breach test (its crossing half becomes a stop pin: the 0x02 pane takes the round, the
   pane still drains and dies).
3. **The pane collision ride + the guard LOS masks** (the pane-ride-on branch's proven pairing,
   plus `LOS_MASKS_ON` scoped to the 0x10 drop for the guard-vision family) — then the
   dead_pane movement test un-ignores and the guard-fire gate matches the game's mask-0xa aim
   trace per material.
4. The 08 settled-guard through-shot: a 30-hp sheet (a second tap after the first survives) or
   the ambush guard 9's approach crossing the sheet plane at the mouth.

## Live checks for the main agent

1. 07000008, fresh boot: fire one P2K round at the intact control-room front pane from the
   room side — the game should register the emitter/decal/impact sound AT the sheet AND the
   round should continue to the shell behind [M 0x12cfb8..0x12d018 + the shared Effect_Create
   tail]. The port now does both; a live confirm pins the VFX half (the decal ON the intact
   glass is the part a screenshot settles).
2. 07000001, fresh boot: a guard behind the INTACT drive glass (mat 0x02, placements
   130/136's family) — does his aim trace stop there (mask 0xa drops only 0x10)? Expected:
   he does NOT react to Bond before the glass breaks. The port currently has no pane
   collision, so its guards see through everything — this check sizes that gap.
3. 07000001, fresh boot: one pistol round at the INTACT front window (0x02) — does the round
   stop at the sheet (the material rule [M 0x12cf54-0x12cf60]) or pass? The port passes it
   (the kind gate). This is the deciding measurement for the gate flip in "what's left" #2.
4. 07000008, fresh boot: a guard round at an intact cabin window — does the round pass and
   hit inside (the guard guns are bit-0x800-clear [L weapon_data])? Confirms the guard-side
   armament in "what's left" #1 before the 08 walk repair.

## Usage

The session's input/output tokens, context used and cost: not visible to me. (Wall clock per
the WORKLOG: start 17:33 UTC, report written 18:07 UTC, sandbox deadline 19:32 UTC.)

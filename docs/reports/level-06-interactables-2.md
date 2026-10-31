# level-06-interactables-2 — the report

Task: every player-touchable kind of 07000006 behaves like the game — census the kinds [D], and
for every kind a player can touch add the live test that drives it through the real path
(walk/E/shoot) with a sister that fails on the wrong result, using interactables_0700000b_live.rs
as the model.

## What's done

**The census [D].** Re-ran `census_07000006` (`dump_07000006_placements_by_kind`): 30 main-map
kinds — 0x0F drones ×32, 0x19 room/key cels ×322, 0x20 window ×1, 0x24 starts ×3, 0x2A sky ×10,
0x31 wreck/fusebox props ×5, 0x32 snow ×21, 0x33 hints ×3, 0x41 inverter ×1, 0x47 rooms ×37,
0xD4 cloud ×1, 0xD8 well bucket ×1, 0xD9 ScriptPlayers ×4, 0xDB doors ×12, 0xDE searchlights ×20,
0xDF snow gen ×1, 0xE1 explodeables ×6, 0xE2 coronas/barrel-roll SS ×28, 0xE4 Hurt ×1, 0xE5/0xE6
cover ×114, 0xE9 AI points ×6, 0xEA touch triggers ×49, 0xEB LoadPoint wrapper ×1, 0xEC/0xEF
muxes ×6, 0xF0 pickups ×11, 0xF1 spawners ×4, 0xF5 AI volumes ×109, 0xF9 sound triggers ×4.
**No switch (0x29), lift, monitor (0x30), copter or music-trigger kind exists on 07000006** [D
census; docs/levels/07000006.md §5] — those task rows are pinned absent, not skipped.

**Per-kind coverage.** The interactable kind family was already split across the level's sibling
files (the file header maps every row): doors (#47 E-opens, #369 lock ch 200 refuses —
interact_07000006_live), the fusebox #747 (fusebox_06_live), TouchOnce plates + the inverter +
the siren (alarm_panel_06_live + walkreal_07000006_live), ScriptPlayers (interact_07000006_live
+ urn_break_live), searchlights, hint volumes, LoadPoint #746 (walk_07000006), the inert Hurt_A
#373 (interact_07000006_live). THIS file carried 3 tests (the 0xE1 barrel chain, the 0xE2
barrel-roll SS knock, the kind-6 medal gate) and gained the two kinds nothing on the level drove:

1. **The breakable window (0x20 → `Break_Create`)** — `the_securityroom_window_breaks_on_the_gunshot`:
   the pane #376 [D params (2, 0, 0xf8)] takes the PLAYER ROUND. p2 0xf8 → tag+0x08 = ~0xf8 =
   0xFF07 [M Break_Create 0x12c230:31-39] — the same mask the oil barrels carry [D #321 p4 0xf8],
   whose admission of the disc guns' word 0x0008 test 1 proves; the contact kills [M Break_Update
   0x12bfd8:13 + Collide_FilterBullets 0x1ece30:24-33]. Contrast: 07000014's security-room pane
   (p2 0x20) drops the gun word and is blast-only — the sister pin there. Sisters here: the pane
   stands before the shot; the fire loop is bounded.
2. **The kind-1 AMMO pickup (0xF0)** — `an_ammo_pickup_walkover_credits_the_pool_and_consumes_the_box`:
   the plain placed box #679 [D params (1, 0x2, 0x23) — kind 1 AMMO, weapon 0x2 the P2K, 35
   rounds] is consumed by the walk-over and the ammo lands through `Player_EquipAmmo`'s clip-first
   arm [M Pickup_Update 0x18e148's contact → Pickup_Handler 0x18e650 case 1 → Player_EquipAmmo
   0x1a8d18; the port's equip_ammo fills the 0x10-flag clip slot first, then the carried pool —
   the assert reads carried+clip together]. Sisters: the box stands and the pools are flat before
   the walk. The census's other kind-1 box #666 ("oOBJECT_AmmoPickUp300Soft") is pinned a SCRIPT
   RESERVED slot — its live activation sits inside the player's touch sphere at tick 2 (the
   NF_PICKUPLOG take receipt) — so the plain placed box is the kind's pin.

**No port gap was hit**: both new tests went green with zero changes to `nf-game`; the window's
filter math and the ammo's clip-first credit already matched the game's code.

## How it was verified

- `interactables_07000006_live` — **5 passed / 0 failed / 0 ignored** (6.88 s), the three prior
  tests untouched and the two new ones green.
- `walkreal_07000006_live` — **3 passed / 0 failed / 0 ignored** (54.21 s).
- `nf-game` — **953 passed / 0 failed / 9 ignored across 98 binaries** (the 9 ignored are the
  assertion-free census/survey prints).
- Full suite (fast runner, 996 binaries, running on the final tree 2c8a1fe0): the sandbox window
  closed mid-run — the partial total and any FAILED/TIMEOUT/CRASH lines: SEE THE TOTAL LINE BELOW.

## What's left

Nothing for this task. The suite's `disc_golden` tests may fail on the slim ISO (allowed).

## Live checks for the main agent

None required — both new pins are simulation-internal (channels, inventory, break flags) with no
visible component. Optional: a fresh-boot PCSX2 shot of the security-room window pane #376
shattered after one pistol round, to eyeball the shatter pieces the port's kill-piece sources pose.

## Usage

not visible to me

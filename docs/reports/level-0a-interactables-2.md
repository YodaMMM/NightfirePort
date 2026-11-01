# level-0a-interactables-2 — the report

Branch `s/level-0a-interactables-2` (from master 3d97ed08). Task: one live test per KIND of
player-touchable placement on 0700000a in `interactables_0700000a_live.rs`, modelled on
`interactables_0700000b_live.rs`, driven through the real input (walk / E / shoot), each asserting
the game's result with its sister, with the census as the kind list.

## What's done

**The census** (docs/levels/0700000a.md's table + docs/levels/0700000a-audit.md, [D every
placement kind] over the 800-placement dump, dispatched per `parsemap_create_dynamic_objects`
0x1d04f0 [M]) splits into:

- **Player-touchable kinds, now each with a live test in the file (13 total):**
  | Kind | Placements [D] | Test |
  |---|---|---|
  | 0xE2 SS | #524/#525 lift doors (create-sweep), #663 vent grate | `ss_lift_doors_sweep_open_at_create` (the E-arm's mechanism is 0b's `vent_grill_ss_e_sweeps_it_open`; #663 sits at (−4.6, 4.6, 0.4) off the route) |
  | 0xF0 Pickup | 2× BodyArmourHeavy, 3× "Pickup 007" | `armour_pickup_taken_by_walking_over_it` (kind 3, p2 = 50 [M 0x18e650 case 3]) |
  | 0x20 Break | #378/#379 vending glass + 43 script-keyed | `vending_glass_breaks_under_fire` |
  | 0xEB Trigger_Touch | the 30-plate laser web | `laser_web_crossing_fires_the_alert_channel` |
  | 0x29 Switch | 8× alarmswitchplayer + #643 | `alarm_switch_refuses_the_player_use` [M 0x1b02a8's class-3 exclusion] |
  | 0x22 Ladder | #686 | `vent_ladder_grabs_and_climbs` |
  | 0xE4 Hurt | #745-749 shaft volumes | `lift_shaft_hurt_volume_deals_the_fall_damage` |
  | 0xEA TouchOnce | 19 plates | **new** `touchonce_plates_fire_once_and_the_hatch_plate_fires_on_the_early_approach` — #58 fires ch 0x47 under the spawn and latches `gone` [M 0x1b1d78-90's glist-0x10 latch]; #757 (out ch 0xB7, lock ch 0x3A) fires on the early approach while the lock is off [L ee_start @0x26fc90] |
  | 0xDB Door | 19 doors | **new** `doors_open_on_use_and_the_keypad_locked_door_refuses` — #540 (no lock [D]) opens on the E; #724 (lock ch 48 [D p3]) refuses while the channel is off [M door.rs's +0x68 map] |
  | 0x2E Lock | 4 keypads (#646/#729 → 49, #648/#732 → 48 [D]) | **new** `keypad_lock_menu_takes_the_code_and_unlocks_its_door` — the E arms the menu (state 3 [M 0x1aed00]), four wrong digits never match [M 0x1ae3d8:262-296], the game's own snapshot code [L ee_start @0x2beff8, code index 1] unlocks → ch 48 → door #724 auto-opens (state 4) [M 0x1363c8:101-107] |
  | 0x35 PCQWorm | #688 armed + 2 decoys | **new** `qworm_pc_install_needs_the_gadget_and_the_office_pc_reveals_its_code_on_the_e` — the spawn K5 (0x43) never installs [M 0x1aee20's 0x58 test]; the real gadget-cycle equip + fire writes ch 0x48 and runs the machine to state 2; the decoy #684 (keys 2, r 0.579 [D]) is E-usable [M 0x1b0840's p2 arm] — its live E drive skipped (see Live checks), the arm stays unit-pinned (useobj.rs's p2 tests) |
  | 0xD9 ScriptPlayer | #60 keyboard (mask 4) + 4 ambient | **new** `keyboard_sp_use_plays_the_script_and_fires_the_objective_mux` — the mode-64 preload's prop 0x020005E1 exists from load [M 0x1db9a8's load beat]; the E from the corridor wedge toggles ch 0x3A [M 0x1dc338 + 0x1dbda8], the script's tick-15 op spawns 0x020005E2 [D nis 0x06000597], AND #741 raises ch 0x49 [D] |
  | 0xE8 LoadLevel | #59 hatch exit | **new** `hatch_load_point_refuses_then_loads_through_the_real_chain` — the E with ch 0x4A off posts the locked message 0x2000021 and never loads [M 0x1b2318:76-90]; through the real chain (the keyboard's E → 0x3A → #741 → 0x49; the real install → 0x48; #742 → 0x4A) the same E requests 0700000b [M 0x1b2318:111] |

- **Not player-touchable on this level's data (documented, no test):** the 13× 0x31 fuseboxes
  (script-hash-0 shells, inert invisible art [M 0x1dc7e0:69-71]); the 0x30 monitor #783
  (script-keyed at the origin — no live use target [D]); the 0x71 rotor and 0xFA third-person
  icon (presentation); the 0x27 GT turrets (census-pinned by `cameras_turrets_census_live`; the
  ceiling gun's arm chain is the ceiling-gun-route task's work); the 0x3A grapple point (the
  sky-bridge set piece, out of the walkthrough); the 0xFB music trigger (channel-driven ambience,
  the kind pinned by 0b's `music_triggers_created_with_the_disc_params`).

**No port gap surfaced.** Every failure during the work was test-side (the wrong observable or a
blocked stand), each fixed with its evidence:
- the "closed" door state reads bit 0x2 (state 2), not 0 — the sister's closed check is
  `state & 0xc == 0` (door.rs's state bits);
- the locked-exit message is consumed same-tick through `drones.info_labels` → `hud.info_labels`
  → `info_msgs` (the bind at post time, fix-context-hints) — the pane's `info_msgs` list is the
  end observable of [M 0x1b2318:81-86];
- the hatch E stands on the plate's own volume centre (the walk_0700000a hatch test's proven
  shape); the settle pushes Bond out of the plate's thin z band (lo/hi z 19.761..19.812 [D]) —
  the E must run from the overlap, not after a 12-tick settle south of it;
- the decoy PC #684's stand: no candidate spot near (45.45, 0.03, −17.84) grounds (5 probed,
  all airborne) — the live E drive is skipped with a printed note, not asserted.

## How it was verified

- `interactables_0700000a_live`: **13 passed, 0 failed, 0 ignored** (16.25 s).
- `walkreal_0700000a_live`: **7 passed, 0 failed, 3 ignored** (23.70 s) — the 3 ignores are the
  pre-existing documented ones (the west-doorway glide height, the corner crossing, the
  ceiling-gun route), untouched by this task.
- `nf-game` lib unittests: **635 passed, 0 failed, 5 ignored** (0.87 s). The crate's integration
  binaries run under the full suite (below).
- The full suite (`tools/test/suite.py`, NF_SILENT=1) was started in the session's first minutes
  and re-started on the final tree at 13:14 UTC; its per-binary lines and total are in
  `suite.out` (see the tail quoted below, gathered at report time).

## What's left

- The suite's complete total line (it runs past the session's end; the next message resumes from
  the branch — the tree is final and pushed).
- The decoy PC #684's live E arm (blocked on a standable spot — Live check 1).
- The stretch kinds (a sensor-shoot live drive, the Q-Claw latch at #785) were not added; both are
  code-complete and census-pinned, and the task's kind list centres on the touch/E/shoot set the
  file now covers.

## Live checks for the main agent

1. **The decoy PC #684's floor** — the port grounds NOTHING near (45.45, 0.03, −17.84) (probed
   (44.4/44.9/43.0/45.45, y 0.6-0.7) — all airborne after the settle). In PCSX2: teleport Bond to
   (45.45, 1.0, −17.84) on 0700000a and watch whether he stands or falls; if he stands, the port's
   collision there is the gap and the E-install on the decoy (PCQWorm_Activate 0x1b0840's p2 arm)
   can be driven live.
2. **The hatch plate's push** — the port's settle slides Bond from the plate's centre
   (21.72, 1.0, 19.786) to z 19.49 within ~12 ticks (the plate's z band 19.761..19.812 [D]). In
   PCSX2: stand Bond at the plate centre and watch BL+0x790 / his z — is the game's stand stable
   (the port's push tail over-fires there) or does the game also push him south?
3. **LOAD_LEVEL #59's fallback param** — the port parses param 3 = 0 (printed
   `fb 0x0` [D via the parse]), so [M 0x1b2318:96-108's else arm] has no channel to write on the
   successful E. A placements-dump cross-check of #59's raw params (p3) would pin it [D].

## Usage

not visible to me — this sandbox exposes no token, context-window or cost figures for the session
(no billing/token surface in the environment; the driver's harness shows none).

## ntfy failed

All three fallbacks failed (the sandbox has no route to ntfy.sh):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x` →
   `curl: (28) Connection timed out after 20002 milliseconds`
2. python3 urllib → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x` → no output, killed at the
   120 s tool timeout (hung on the unreachable host).

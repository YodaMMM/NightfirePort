# level-0d-audit — the 0700000d behaviour audit

Branch `s/level-0d-audit` (from master 30598272; master did not move during the session — the
branch IS the final tree). Done 2026-10-26, 04:42–06:25 UTC.

## What's done

1. **The per-placement audit table** — `docs/levels/0700000d-audit.md` (1095 lines): one row per
   placement for **all 568 main-map placements [D]** over **34 kinds** (the census probe
   `crates/nf-view/tests/census_0700000d.rs --ignored` dumped them from
   `NF_ISO`'s 0700000d.bin — the same 568 the level doc §7 carries). Per kind: the game chain
   [M] (creator in the `parsemap_create_dynamic_objects` 0x1d04f0 dispatch — every case
   re-verified against work/decomp/ACTION.ELF this session — plus the update, the use and the
   damage handlers from the control_funcs table @0x26f890 and the decompiles), the port side
   (created? updated? drawn? usable? reacts? sounds? — swept through crates/nf-game +
   crates/nf-view/src/scene.rs with file:line cites), and a GAP verdict per kind plus row notes
   per placement.
2. **The scripts section**: all 12 kind-7 scripts of 0700000d.bin + 0780000d.bin (0x0600068f, the
   intro cut-scene) + 0790000d.bin (0x0600073a) decoded with the new
   `census_0700000d.rs::dump_0700000d_scripts` [D]; every stream's ops, the two Camera streams,
   the hand-backs (event 6 → the 0x100 flag + the player-matrix copy [M Script_EventHandler
   0x1d94c0:125-134]), and the owner mapping — 3 placement-owned (SPs #115/#377/#492), 9
   code-spawned effect NISes (bullet impacts 0x06000052/0x060007c4, smashes 0x06000027/4f/52, the
   dead GT minigun pair 0x06000044/45). **No script on the level writes a channel** (no event
   0xd/0x11 anywhere [D]) — the level doc §7's scan holds.
3. **The mission-record section**: record 12 read byte-for-byte from work/ee_start_05.bin [L] —
   **the record is at 0x2a4530, not the 0x2a44a0 the level doc carried** (that's record 8's count
   word); level 0x0700000d, BaseMap 0x0700000c, part 1, objectives @0x2a40c8 ×3 — all three
   fields equal `mission/c0700000d.rs::DEF` [L vs M]. The fail hooks (alarm 0x61 / civilian 0x60
   [M 0x1827d8:98-108]) are **unreachable on 0d**: every drone's death channel is 0 and no
   placement/script writes 0x60/0x61 [D] — the mission cannot fail.
4. **Pinning tests** — `crates/nf-view/tests/audit_0700000d.rs` (both green):
   `census_0700000d_kind_histogram` pins the 568-placement/34-kind census + the wait-web wiring
   (#513→#292/#293 ch 0x2d, #554→#550-#552 ch 0x58), the C5 leaf pair's out==watch ch 0x1f, and
   every drone death ch == 0 [D]; `mission_record_12_matches_the_snapshot` pins record 12 and the
   three objectives against work/ee_start_05.bin [L], including the 0x2a4530 address correction.
5. **Doc corrections** (the small fixes this audit made, each cited in the audit doc's
   Corrections section): 0700000d.md's record-12 address (0x2a44a0 → 0x2a4530), the fail-hook
   line range (0x1827d8:128-141 → :98-108), Player_CollWire (0x1971f8 → 0x197130), the drone
   skins (0x500064 → the census's 0x500006d/7e/87/90/93), plus the task brief's Player_Activate
   address noted (0x1a5170, not 0x1a7ef8 = Player_InitWeapon).

## How it was verified

- Every placement row: the disc census [D] re-dumped this session (`census_0700000d.rs`, opt-in
  NF_ISO); the game chains cite the decompile file addresses (0x1d04f0 dispatch + creators +
  control_funcs @0x26f890); the port chains cite crates/nf-game + nf-view file:line.
- The two pin tests above run green (`cargo test -p nf-view --test audit_0700000d`, 2 passed).
- The scripts decode ran on the real ISO archives [D]; the mission bytes came from the boot
  snapshot [L ee_start_05.bin] and agree with the port's static DEF.

## The big gaps (each its own task)

1. **GAP-0d-2 — on-foot rigid bodies (0xD8) are not ported.** Game: RB_Create 0x1d6bf0 (class
   0x1A; mass = p0, p0=0 → 3.0), RB_Update 0x1d6e38 (control_funcs[0x1A], wind + Euler), the
   collide slot 0x1d7a80 + RB_ApplyImpulses 0x1d6ff8 (bullet impulses via HITDATA+0x08); support
   math RB_Init 0x1d7ad8 / RB_Build 0x1d7ba0 / RB_SetState 0x1d7da0 / RB_CuboidMomentsOfInertia
   0x1d7dc8. Port: `rigid_body.rs` carries the math helpers only — no placement create, no tick;
   the at-rest mesh draws via MISC_KINDS (scene.rs:8043). On 0d the three `obj_ship2_
   concreteblock03` #89-#91 (mass 600) cannot be shot/pushed. Shared with levels 01/02 (the same
   note in their docs) — a port-wide machine, not a level fix.
2. **GAP-0d-3 — ThirdCams (0xE7) are data-only.** Game: ThirdCam_Create 0x1bc510 registers the
   camera id into ThirdPersonCameras[id]; ThirdCam_GetPos 0x1bc590 + camera_tracking 0x1ba5e0 cut
   the view to it during the scripted actions. Port: the keys are created (worldobj.rs:402-409)
   but nothing reads them at runtime (no tick, no view consumption). On 0d: #273 hand-over-hand,
   #376 wall-hug, #486/#488 ziplines — the action cam never cuts. Port-wide (the view side owns
   it); live check below.
3. **GAP-0d-1 (small, visual) — the office fan rotor (0x71) never spins.** rotor_init 0x18f4b8
   (dispatch case 0x71) creates a spinning fan; the port draws #11's mesh at rest (MISC_KINDS).
   Needs the main agent's render to verify (lesson 6), so it is NOT fixed here.
4. **The effect-NIS stand-in** (not a behavioural gap): the game spawns script NISes for bullet
   impacts (0x06000052/0x0600007c4 via Bullet_CollisionHandler 0x12ce80), smashes
   (0x06000027/4f/52 via Destroy_Smash 0x12bb60) and explosions; the port renders its own
   particles. No channel writes exist in any of them [D], so no wiring can be missed — but the
   sparks/blast visuals differ. Render-side.

## What's left

- The suite total line (below) is from the background run started 04:52 UTC on the pre-audit
  tree (it was still running at the report cut-off); the two test binaries this task added
  (`audit_0700000d`, `census_0700000d`) ran green standalone (`cargo test -p nf-view --test
  audit_0700000d` — 2 passed; the census is `--ignored` prints). A full fresh-tree suite rerun
  did not fit the window.
- The driver's symptom list mapped to 0d: "characters walking through Bond" — the guards'
  player collision is the drone.rs AI's (no dedicated audit finding); "guests that never talk" —
  0d has NO allies/civilians [D census: 41 guards, death ch 0, no talk lines] so N/A; "doors
  that never lock" — the fakedoors' lock ch 200 has no writer in the GAME's data either [D]:
  locked forever in both, correct; "consoles whose action does nothing" — the crane buttons
  (#113/#114) animate on ch 0x2b and the shipping_handle #112 is channel-driven (not E-usable by
  the game's own mask) [D+M] — the port matches; "cut-scenes with broken cameras" — the intro
  camera is ported end-to-end (nis.rs Obj::Camera → scene.rs:3409 → lib.rs:5274); the live check
  below confirms it on hardware.

## Suite

The background fast-suite run (started 04:52 UTC on the pre-audit tree; `python3 tools/test/
suite.py`, docs/tools/test-suite.md) was still running at the cut-off: **partial at 06:12 UTC —
242 of 812 binaries, 1547 passed / 4 failed / 24 ignored**, the 4 failures being the known
slim-ISO set (`identity_disc::the_real_disc_passes_the_identity_check` +
3 × `disc_golden::*` — the same 4 the previous session's full run reported as expected on the
slim ISO). No TIMEOUT / CRASH lines. The two test binaries this task added ran green standalone:
`cargo test --offline --release -p nf-view --test audit_0700000d` → 2 passed (the census binary
is `--ignored` prints). A full fresh-tree rerun did not fit the 2-hour window (build alone was
991 s; the run is 1.5 h+). What I ran before pushing: the new audit test binary (2/2), the
census dumps (both), and the background suite above.

## Live checks for the main agent

1. **The p2=2 catwalk drones** (#260/#309/#351): fresh-boot 0700000d; gdbwatch.py on
   Drone_Create 0x136be0's part gate (read p2 @ placement+0x34; GameState+0x40 read) — expected:
   the three placements see p2=2 > part 1 and return without creating [M 0x136be0:49]; the port
   has the same gate (drone.rs:11254).
2. **The intro cut-scene camera**: fresh boot; watch ScriptCam @0x30d100 — non-zero while #377's
   camera stream (keys 0..52) runs, 0 after the event-6 hand-back. `--bond 179,11,-205,0,0` at
   load: the fly-by, not the player head.
3. **The C5 leaf pair**: kill bigdoor02 #96 — expected switch_channels[0x1f] written once and
   #93 smashed by the watch arm in the same tick cluster [M 0x12c4f8:30-35].
4. **The rotor** #11: `--bond 187,14.8,-278,0,0` — the fan spins in the game; at rest in the port
   (GAP-0d-1).
5. **The ThirdCams**: start a zipline ride (#484/#487) — the game cuts to cam 11/12
   (ThirdCam_GetPos 0x1bc590); the port keeps the player cam (GAP-0d-3).

## Usage

not visible to me — this session's sandbox exposes no token, cost or context-window figures (no
environment variable, file or tool output in this session carried them). The only related figures
I can see are the subagent reports' trailing token lines ("Tokens -> prompt:195630
completion:7667 total:203297" and "prompt:180628 completion:11357 total:191985", the two research
subagents' own usage prints — quoted verbatim from their results; they are subagent calls, not
this session's totals).

# level-12-audit — the 07000012 behaviour audit

Branch `s/level-12-audit` off origin/master. Deliverable: `docs/levels/07000012-audit.md` — every
main-map placement (792 rows over 33 kinds), the level's scripts, the mission record, and the
channel-writer census, each with the game's behaviour [M/D], the port's, and a GAP column.

## What's done

1. **The audit table** (`docs/levels/07000012-audit.md`): all 792 main-map placements as rows,
   grouped by kind. Every kind's creator in `parsemap_create_dynamic_objects` 0x1d04f0 / the static
   path, its update and use/damage/event handlers [M], the port's handling, and the GAP. The
   dispatcher skip (`flags & 0xa000 != 0` → the row never reaches the dispatcher [M 0x1d1b68]) is
   applied: on this map it covers kinds 0x19 (438), 0x47 (27) and 0x31 (15) only — and the port
   matches behaviourally (no creation either side; for the kinds the port does arm, zero
   0xa000-flagged rows exist disc-wide [D the `dump_a000_skipped_rows` probe]).
2. **The scripts**: all four ScriptPlayers decoded per stream from the kind-7 entries
   (`dump_07000012_main_census`), plus a sweep of all 13 kind-7 scripts for channel-writing events
   (`dump_07000012_script_events`). Findings:
   - The skybridge intro (SP #660, 0x06000530, 39 streams) has **no camera op** — the level doc's
     "entities + camera" was wrong on the camera half; it is set dressing over live gameplay. Its
     mask word p5 = 2 parks it in state 0 at create [M 0x1db9a8:74-86], so it writes nothing until
     its activation beat (the same headless-derivation limit as 07000006's #808, nis.rs
     fix-searchlight-alarm).
   - #662's script is **0x060005bb**, not the level doc's "0x060005FB" (a misread of the placement's
     param 0 [D]).
   - The only camera on the level is the shaft-fall death cut-scene (0x0600098a, stream 0), which
     also carries the death anim (skin 0x05000007, script 0x0600098b), event 4 (PlayerDisable),
     event 5 (the drone freeze) and the ch 0x62 death-restart write.
   - Exactly four channel writes exist in the level's scripts: the intro's ch 0x3c + the
     sub-script's 0x89 pulse (both with **no consumer** on the level [D the placement scan]) and
     the death script's ch 0x62.
3. **The mission record** [L ee_start_05.bin @ 0x2a41b8]: 2 objectives (ch 0x49 enable 0x48; ch 0x4D
   enable 0x49), **no fail fields** (both 0xffffffff) — the civilian-death (ch 0x63, drone #1 p12)
   and shaft-fall failures act outside the objective records. The port's `c07000012.rs` matches.
4. **The channel-writer census**: no writer of the door lock channels 0xc8/0xc5 (doors
   #103/104/105/#115/#544) nor the alarm ch 0x61 exists on the level [D] — those doors never lock
   and no alarm chain fires **in the game as well**; the "doors that never lock" symptom has no
   data to come from on this level. And the door "lock" channel's rising edge is an **UNLOCK** —
   label 0x2000003 = "Door Unlocked" [D docs/systems/door-text-labels.md; M 0x1363c8:88-112] — so
   the burn on ch 0xa opens the guardroom door #754; the port already matches (the labels queue).
5. **New tests** (`crates/nf-view/tests/scripts_07000012_live.rs`, both green):
   - `the_shaft_fall_sp_writes_the_death_channel_on_its_trigger` — raising ch 0x7C (the shaft-fall
     plates' channel [D]) runs SP #767 and ch 0x62 rises [M 0x1d94c0 case 0xd; D the script bytes].
   - `the_parked_intro_sp_writes_nothing_until_its_activation_beat` — the mask-parked intro writes
     no channel headlessly [M 0x1db9a8:74-86].
6. **Large gap (its own task): the drone HITTEST** — "characters walking through Bond". The port's
   control pass has no drone collide step; the spec is cited in drone.rs:16544-16551:
   `Drone_CollisionHandler` 0x140968 [M table @0x26f8a8], the two-end capsule ext+0x3F0/0x400 with
   radius ext+0x45C = 0.4 (0.55 state 0xC, 0.3 the +0x2F variant) armed by `NDrone2_Collision`
   0x140e90:59-124 under `NDrone2_DoCollision` 0x149ec8's gates (incl. `DroneFunc_NearPlayerCollision`
   0x14b190's 2.0 near-player arm and the 0700004A special case), contacts mirrored into both
   sides' +0xD0 [M 0x1ea398:160-233], the consumer dispatch (player bump msg 0x22; bullets
   6/7/0x19/0x17; walls/doors velocity-zero + deflect [M 0x13fff0:127-147]). Touches every level's
   AI — own task, own suite.

## What the symptoms resolve to

- **Characters walking through Bond** → the drone HITTEST large gap (above).
- **Guests that never talk** → the guard/civilian bark tables are port families (PainTalk/patrol/
  MP chatter [M 0x13fa20/0x13fd08/0x13fe80/0x13fb98 in drone.rs]); no 07000012-specific line data
  was found missing in the decompile-driven path, but no headless assertion is possible for a
  bark's audibility — see the live checks.
- **Doors that never lock** → no data: no writer of any door lock channel exists [D]; the one live
  lock edge (ch 0xa → #754) is an UNLOCK and is proven (decryptor_07000012_live).
- **Consoles whose action does nothing** → the two consoles are pinned live in the existing tests
  (the override console #731 → ch 0x49, the Q-Worm PC → ch 0x34; the level doc §7's live legs).
- **Cut-scenes with broken cameras** → the level's only camera is the death cut-scene; the intro
  has none. The port runs the camera op family (nis.rs); the picture needs a render (live checks).

## Verification

- Ran by hand: `cargo test --offline --release -p nf-view --test scripts_07000012_live` (2 passed),
  the census dumps (`dump_07000012_main_census`, `dump_07000012_script_events`) against
  `Nightfire-onfoot.iso`. No lib code changed, so no other crate behaviour moved.
- Full suite (fast runner, 820 binaries, NF_SILENT): see the total line below. NOTE: the sandbox
  OOM-kills the suite's heaviest binaries (net_*/mp_* exit -9) when it runs near the 4 GB ceiling —
  the CRASH lines are environmental kills, not test failures; the previous session recorded the
  same. Re-running those binaries standalone is the honest read.

Suite total (partial, killed at the wall): **1414 passed, 0 failed, 115 ignored over 413 of 820
binaries**; 6 binaries CRASHed with exit -9 (SIGKILL): net_match_end_clients_live,
mp_online_failures, net_soak_faults_live, mp_remote_fx_live, net_desync_report_live,
mp_death_hands_online — all net_*/mp_* memory hogs; `mp_online_failures` SIGKILLs standalone too,
so the kill is the binary's own footprint against the 4 GB sandbox ceiling, not a test failure and
not this branch's doing (the branch changes no lib code). The run was killed at a hung binary
(~08:45 UTC) with the sandbox wall at 09:16; the remaining 407 binaries are untouched ground for
the next session's total line.

## Live checks for the main agent

1. 07000012 fresh boot: the elevator intro beat — does the skybridge intro (SP #660) play when
   Bond steps out at the start (the mask-parked activation beat), and does anything visibly react
   to its ch 0x3c write? Record the frame the lights animate.
2. 07000012: walk into an office guard's face — expected after the drone-HITTEST task: a bump,
   not a pass-through. Today the game (real PCSX2) bumps; the port passes through (the gap).
3. 07000012: let a guard spot Bond — record his bark (the talk family) and whether the office
   civilians (drone #1 skin 0x0500000f is the death-ch-0x63 civilian) scream/panic — the "guests
   never talk" symptom's ground truth.
4. 07000012: jump down the elevator shaft — the death cut-scene's camera (0x0600098a stream 0)
   should track the fall; the port's render needs checking against the game's.

## What's left

- The suite's formal total line (below) + a standalone re-run of the OOM-killed binaries.
- The drone HITTEST task (the large gap).
- The intro's activation beat (a live check; then a boot like 07000006's if it proves scripted).

## Usage

not visible to me — this session's environment exposes no token, context-window or cost figures to
copy.

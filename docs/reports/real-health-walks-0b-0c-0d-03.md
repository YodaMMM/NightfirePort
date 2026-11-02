# real-health-walks-0b-0c-0d-03 — report

Branch `s/real-health-walks-0b-0c-0d-03` (cut from origin/master b62a8907).

## What's done

1. **The four real-health sisters added and building green** (`--no-run` release, 2-core sandbox,
   fresh target): each is the level's whole-mission walk re-run through
   `damage_log.rs::with_real_damage` — the health top-up off for the test's thread, Bond's death
   panics with the damage log, no health/armour/ammo writes anywhere, auto-aim at the app default,
   the walker's plain return fire for the fights:
   - `walkreal_0700000b_live.rs`: `walk_fresh_spawn_to_the_mission_end_real_damage` (the fresh-spawn
     whole-mission walk, no channel pokes);
   - `walkreal_0700000c_live.rs`: `walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage`
     (through the b2 walker's `with_real_damage`, the whole 0c mission plus the 0d arrival legs; the
     two documented route stand-ins are links from the master walk and write no health/armour/ammo);
   - `walkreal_0700000d_live.rs`: `walk_arrival_to_the_c5_door_real_damage`;
   - `walkreal_07000003_live.rs`: `walk_the_full_route_and_complete_every_objective_real_damage`.
   Committed and pushed (commit "the four whole-mission walks' real-health sisters added").
2. **The 0b run executed** (4.4 s of sim): Bond dies at tick 12765 at (−1.36, 47.81, 5.30) with
   **0 return-fire fights** — no enemy damage at all. The full evidence chain, all measured on this
   tree (`NF_DMG_TRACE=1` run + `probe_0b_hurts.rs`, committed):
   - 42 world-bucket hits totalling exactly 100.0, −2.40 each (last −1.60), one every ~3 ticks,
     from tick 12617 (Bond entering the exec-floor return corridor at (−1.0, 47.8, −5.8), right
     after the files objective) to the death at (−1.4, 47.8, 5.3) — the whole north-south corridor
     stretch between the terminal room's north door and ring nodes n14/n15.
   - Raw damage 3.0 per hit (2.4 = 3.0 × the 0b Normal difficulty factor 0.8 [M `bullet.rs`
     `damage_factor`, 0x09..=0x0b → normal 0.8, cited from `Player_HandlePain 0x190404-0x190470`]).
   - Standing in the same stretch deals nothing: the probe parked Bond at (−1.0, 47.3, −6.0) for
     600 ticks — zero health events [probe output].
   - Not the hurt volumes: 0b has **zero** 0xE4 placements, `sim.hurts` empty [D probe dump; the
     level docs' census: the 11 type-0xE6 placements are CoverLowNode AI records, not hurts
     (level-0b-audit §2.2)].
   - Not the live wire: zero fuseboxes on 0b [D probe dump], and the port's live-wire hurt is a
     flat 5.0 raw [M `fusebox.rs` `LIVE_WIRE_HURT` = 5.0, applied at `drone.rs:18538` with
     `Cut::LiveWireHurt`], which the 0b factor makes 4.0 — not the measured 2.4.
   - Not the GT turrets: the port's turret AI (`GT_Update 0x180498`, `GT_Track 0x17f9c8`) is a
     known gap [M `drone.rs:12446-12447`; `gt.rs` header] — the five 0b turrets (gt2 at
     (−1.135, 49.06, −0.19), inside the stretch) cannot fire.
   - Not the monitors: state 3 (the explode) is reached only through the bullet arm
     [M `monitor.rs:153`, `Monitor_ApplyDamage`] — nobody shoots on this leg.
   - No explosion records accompany the hits (`NF_DMG_TRACE`'s explosion dump printed none), so
     the blast push sites [M `drone.rs:13924/16064`] are excluded too.
   - The game's rule for this kind of contact damage is per-collision-hit, not per-tick:
     `Player_DealWithObjHit 0x194458:94-103` — a class-0x27 object hit applies
     `Player_HandlePain(pain + Hurt_GetDamage(obj), …, Hurt_GetType, −1)` once per collision
     HITDATA. The port is applying −2.4 every ~3 ticks for the whole 11 m crossing (≈42 hits,
     lethal), i.e. **the port deals far more than one hit per contact event if this is the
     corridor's fire/glass contact class** — Night Shift part 3's burning floor.
   - **The exact port push site for these hits is still unlocated** — it is none of the four
     `player_hits.push` sites' shapes (drone.rs 13924, 16064, 18538, 20105). That is the next
     session's first job (extend the trace to print the raw owner index + put a marker eprintln
     at each site, one rebuild).
3. **Sandbox overhead absorbed**: the sandbox reset at session start (no repo, no work/, 8.0 GB
   free of 9.9). Re-cloned, re-created the branch, re-downloaded work-slim + the ISO (sizes
   verified), installed rustup + a user-prefix ALSA (`/home/z/alsa-prefix`, pkg-config pointed via
   `PKG_CONFIG_PATH`, runtime `libasound2t64` extracted — the build links green), restored
   `docs/WORKLOG.md` from 7e1b08b8 (latest merged copy) and appended the entry.

## What's left

1. **0b**: locate the port's per-tick world-damage site, then either fix it to the game's
   per-contact-event law [M 0x194458:94-103] or route the walk around the corridor if the disc's
   own route does; re-run the sister + the top-up master walk.
2. **0c / 0d / 03**: run each sister (built; each is one `--test` slice).
3. Every real-damage walk elsewhere + the full suite: the ~1000-binary whole-suite build cannot
   fit this 9.9 GB sandbox (proven again last session on the same disk); run the named groups as
   `--test` foreground slices and delete each slice's executables before the next. Shared-walker
   changes so far: none (only test files + a probe), so the other real-damage walks are untouched
   by construction.
4. The sister tests exist but are NOT yet passing — the 0b one dies by design of the level until
   the damage question above settles; treat them as red until then.

## Live checks for the main agent

1. **The exec-floor corridor damage (the crux).** PCSX2, 0700000b, after taking the NightFire
   files (obj 9 completes), walk Bond back through the terminal-room → ring corridor (world
   x −1.0..−1.4, z −5.8..+5.3, floor y 47.18). With `tools/trace/gdbwatch.py` watching Bond's
   health float (the BLData health field, the address `fieldprobe.py` already knows) — record
   every write during the crossing. Expected if the port over-damages: a handful of discrete
   drops (one per fire/glass contact event, `Player_DealWithObjHit 0x194458:94-103` →
   `Player_HandlePain(0 + Hurt_GetDamage, type, −1)`), NOT one drop every ~3 ticks. Also capture
   `Hurt_GetDamage`'s raw value (port measures 3.0 raw / 2.4 applied on Agent) and the pain type.
2. **Which object hurts.** During the same crossing, log the class byte (obj+0xff) of every
   object the player's capsule contacts (the HITDATA +0x58 object) — expected 0x27 (the hurt
   class) on the burning-floor props; note their positions so the port can name the placements.
3. **0c/0d/03 sanity**: nothing — no live checks needed for the sisters themselves; they run the
   same already-pinned routes.

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures
for the session (no usage panel or API counters are reachable from the sandbox shell).

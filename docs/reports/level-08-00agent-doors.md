# level-08-00agent-doors — the 08 00 Agent walk with the guards using their door routes

Branch `s/level-08-00agent-doors`, cut from origin/master 38052bf6. **Status: the pinned
25773 boarding death is FIXED and verified; the walk is still red one stage later, in the
bottom-terminal Hind fight, and the ignore stays on with the precise blocker in its message.**
The next session continues from this branch.

## What was done

### 1. The 25773 death, root-caused and fixed (the task's headline symptom)

The ignore's death — tick 25773 near (113.19, 28.28, −122.97), #20 (skin 0x500002f, weapon 0x15)
firing from 9.8 u — reproduced on master from the branch cut, and the run log pinned the chain:

- The 13 d3 return-fire fights drain the pistol pool: at the boarding the in-hand weapon 0xc read
  `clip 0 spare 0` (the log's `cycling weapons (0)`), then the wheel reached the fists (0x1, clip 1
  — rejected by `switch_to_armed`'s `w >= 2` scan), and the **next owned stop was the AT-420
  launcher 0x2c (clip 4)** — `switch_to_armed`'s 2..=0x47 scan counts it armed. The platform-mouth
  stand therefore stood into #20's interval-4 bursts rocket-handed: 13 × 6.0 over the stand window
  25375–25773, health 78 → 0. [L the 2026-11-03 run logs]

The re-route (all `difficulty >= 3`-gated inside `walk_the_whole_mission_on_real_input_at`; the
Operative and Agent sisters' timelines stay byte-exact):

- **The long-link resupply** — after the mid-west/long-link stands, `w.resupply("the long link")`
  scoops the #15/#16/#17 drops lying ON the walked path (#15 fell 1.6 u off the long-link
  waypoint at (121.5, 34.5, −113.5)). The post-climb `reload_full` has no spare to fill from; a
  player takes the free rifle. [L the kill-drop pass, level-11]
- **The platform-mouth firearm hygiene** — before the existing "the platform mouth" stand: if no
  firearm (the launcher 0x2c EXCLUDED) has rounds, resupply from the freshest drops (#23/#19 fell
  on that floor seconds before), then `switch_to_firearm` (a new local helper: the real
  `switch_to` input, never the launcher) puts a gun in hand. A player does not fight a 9.8 u
  burst lane rocket-handed — the rounds are the Hind's and the blast is his own geometry.
- **The before-boarding hygiene** — the same dry-check + firearm switch before the last
  pre-boarding stand.

Verified: the platform wave now clears at ZERO damage cost — #20 down in 12 ticks at 25250, #23
(down at 25601 after five "held" exchanges on the stairwell descent), #24 down at 25693, Bond's
health 78.0 untouched through the whole boarding stretch, then the boarding, the ride and the
cinematic all pass. [L the fix1 run log]

### 2. The port law fixed: the shot-out script pane's collision goes away

The Hind fight exposed a real port difference. The measured d3 deaths moved to the bottom
terminal: two "world" blasts (−37.5 at frame 27680, −40.5 at 30000) — Bond's OWN rockets
detonating 5.8 u and 2.2 u from him (the NF_DMG_TRACE explosion dumps name the blast points
(112.4, 15.0, −174.3) and (110.5, 14.7, −181.4)). The mechanism: the machine's hp is
**598 = DiffMod 2.0 × 299** ([M Copter_Create 0x17d340's DiffMod; the d2 Agent run reads
448.5 = 1.5 × 299]), so the d3 fight needs ~6 rocket hits — long enough that the launch windows
whose chase-path clips the terminal's own wall band get taken. Rockets 1 and 4 chased the machine
behind the terminal and died on its walls next to Bond.

The player's answer is the carriage's windows: they are script panes, kind 2, **health 1.0 — one
bullet contact kills** ([M breakable.rs bullet_hit; the pane unit test]). The pane shoot-out
works (a new d3-gated block: the live end panes' boxes from `breaks.list`, one tap each through
the real trigger — the semi-auto fires on the trigger EDGE, a held trigger is one pull [L the
first attempt: 300 held ticks, clip unchanged]; 7 panes died, their rows left `breaks.list`, the
script objects deleted — the same removal the fusebox wire cuts do).

But the rays still stopped at the pane plane: the port kept the dead 0x20 pane's glist in the
per-tick dynamic set. **Fixed in `crates/nf-game/src/drone.rs` (`script_parts`): a 0x20 script
entity whose breakable row is dead past the death tick no longer pushes its glist part** —
Break_Kill's "obj+0xFE |= 1, the collision goes away" [M 0x12b970:60], with the death-tick grace
(the killing contact registers before the collision leaves). This is the SCRIPT-pane arm; the
PLACED-pane ride (the ignored `dead_pane_live` tests, the 0x400_0000_0000 arm) stays off as
before.

### 3. The Hind-fight launch gates (d3-gated, kept for the next round)

- **The corridor gate**: the first 12 u of the round's flight must be clear air from the eye AND
  both gun stand-ins (the round spawns at the muzzle [M the fire block]; the AT-420's blast is
  radius 8 ≈100 [L ee_start_05.bin + 0x2C·0x10C]) — both measured deaths were launches whose
  rays stopped at 2.2-5.8 u.
- **The range gate l ≥ 16** (was 10): the machine's body sphere is 7.4 u [M the port's copter
  body-stop rule], so a hit-blast lands at l − 7.4 — it must stay outside the blast's 8 u.

## What is left (the next session's lead)

With the panes shot out and the dead-pane law ported, the corridor rays STILL stop at the pane
plane: the carriage model's own glass quads (the 3409-tri dump shows the pane quads TWICE — the
0x20 entity's glist AND the carriage's shape) keep blocking. **The lead: the port's
`shape(hash)` for the carriage carries glass the game's collide set excludes** — the game's
collision glist for the ride platform is presumably a subset of the draw geometry. Proving the
collide-vs-draw glist split for the carriage (and riding only the collide set) should open the
corridors, and then the pane-broken launch windows + the corridor/range gates should take the 6
hits the 598 hp machine needs (8 rockets on hand: clip 4 + carry 4 [D 0x0600004C streams 37-39;
L record 0x2C +0x92, ammo_data cap]).

Secondary levers if the corridors stay narrow: the machine's circuit crosses due north/south of
the cabin (the copter samples at (112.8, 17.3, −161.8) and (122.9, 15.5, −222.8) [L the run
logs]); the `exit_ticks` scaffolding (the walk-out) is dead — the pad probe proved there is NO
floor within 6 u below any candidate 2-8 u from the cabin (the terminal hangs over the valley;
the kill floor is y −24 [D #42/#43/#44]), so the fight is from inside, through the panes.

## How it was verified (the binaries that ran, with their pass counts)

- `walkreal_07000008_live`: **5 passed, 1 ignored** — the Operative real-damage walk, the Agent
  (d2) real-damage walk (re-verified green on this branch: hp 448.5, 5 launches all hits, zero
  world hits), the topped-up walk, and the two ally tests. The 00agent sister carries the ignore.
- `walkreal_07000011_live`: **3 passed, 1 failed** — the failing one is
  `walk_spawn_to_the_lower_floor_load_point_real_damage_00agent`, the level-11 d3 sister that was
  ALREADY red at the previous task's finish (the ammo-economy chain at the connector, the
  37-drone load, the 0x0500004 7.0/round volleys — the previous task's report). Not a regression
  of this branch: the Operative real-damage and the plain walk pass.
- `nf-game --lib`: **656 passed, 0 failed, 5 ignored** (the dead-pane law's unit surface).
- The full `cargo test` suite did NOT fit this session's window (the nf-view test-binary build
  alone exceeded two 10-minute rounds); the remaining `walkreal_*_live` binaries and
  `npc_doors_live`/`fix_guard_misc_live` MUST run before this branch merges.

## Live checks for the main agent

1. The carriage's collide glist: in PCSX2, shoot a gondola window at the bottom terminal and
   watch whether the eye-ray stops at the window plane disappear — the address evidence is the
   pane obj's glist node leaving the cel chain [M 0x12b970:60]. If the game's rays clear while
   the port's carriage shape still carries glass, the collide-vs-draw glist split is confirmed.
2. The Hind's hp at 00 Agent: read the machine's hp live — the port says 598 = DiffMod 2.0 × 299
   [M Copter_Create 0x17d340; the d3 DiffMod]. A live read settles the d3 multiplier.
3. The rocket-on-body blast reach: fire one AT-420 round at the machine from ~17 u and check
   Bond takes no blast (the contact at l − 7.4 ≥ 8) — the port's body-sphere radius 7.4 [M the
   body-stop rule] with the record's radius 8 [L ee_start_05.bin + 0x2C·0x10C].

## Usage

not visible to me — no token, credit, cost or context-window figures are exposed anywhere in
this environment (no status bar, no session summary, no API dashboard access).

# level-01-agent-escort-fight — report

Branch `s/level-01-agent-escort-fight`, cut from `origin/master` 2879088f (the difficulty feature
is on master at 0cb53397; the Agent walks were reverted at the tip after the 01/06 batch went red).
`origin/s/real-health-agent-difficulty-01-07` is merged, so no extra merge was needed. The level-01
Agent sister was restored from that branch and made to survive the contact-escort's first three
fights; the hall fight remains the blocker, mapped below.

## The premise was wrong: there is no melee in this death

The driver's brief (and 107a's) read the death log as "42.0 in 4 hits from the unarmed drone #9
(melee, about 10.5 each at the Agent scale)". The hit trace (`NF_DMG_TRACE`) refutes it:

- The summary's `weapon 0x0` labels come from `DamageLog::owner_desc`, which re-reads the drone's
  held weapon at SUMMARY time — a dead drone's weapon is the death-reset 0. Every actual hit in the
  trace carries a live weapon id: #9 fired a pistol (0xe), #13 an SMG (0x12).
- #9's 42.0 was 4 pistol rounds: 7.0 each (10 base × the Agent pain 0.7 [M Player_HandlePain
  0x1902c8 × Plr_DMod_Normal]) and 14.0 each at 1.7-2.7 u — the TooClose doubling [M
  Drone_ModBulletDamage 0x139f50: ×DroneFiring_TooClose_Damage 2.0 under
  DroneFiring_TooClose_Distance 3.0; the tuning read [L] from the loaded sim: 3.0/2.0 for this
  level group at every difficulty].
- #13's "grind" was 21 SMG pellet hits (1.05/2.10) over 600 ticks at 1.3-4.4 u — a shooting guard
  standing next to Bond while the walker fought someone else, not punches.

The melee check the task asked for was still done: `DroneWeap_CloseCombatImpact` 0x1789c0's reach
(1.75, or 2.0 for anim set 7), the 60° facing gate, the weapon records 99-102 (50 kick / 15 fists
[D probe of weapon_data]) and the anim-event kinds (5/10 → record 0x65, 0xB → 0x66 [M
DroneAnim_EventFunc 0x13b750]) all match the port's `close_combat_impact`. No divergence, and no
melee ever fired in either difficulty's run of this walk.

## What was fixed (all gated `difficulty >= 2` inside the level's own test file)

The Operative timelines are byte-identical (the gate keeps the difficulty-1 body untouched); every
Operative master re-ran green after each step.

1. **The kill order** (`agent_order`): the pellet-stream carriers (SMG 0x12, shotgun 0x1c) before
   the pistols, then the nearer first — in the initial clear, the escort rounds' clear, and the
   hall's named loop. The level's SMG guard ground 21 × 1.05-2.10 into the index-order run.
2. **The approaches stop at 8**, not 3.0-3.5: the old runs walked INSIDE the 3.0 TooClose band
   (every round hits at full double damage there [M 0x179740:100-115]) and fought at 1.7-2.7 u.
3. **A wider personal space**: `keep_off`/`back_off` 6.5 instead of 4.0.

Result: the three clearing fights now cost **23.5** health (was **79.4**) — Bond reaches the meet
scene with 76.5. The sister now dies later, in the hall, having survived everything the brief
attributed to the melee.

## The remaining blocker: the hall's named-crowd fight, mapped

The meet scene starts four hall guards on ch 0xE (skins 0x39/0x36/0x38/0x3a [D placements]). Three
carry the AND's death channels 0x11/0x12/0x13; killing them raises ch 0x99 [D mux placement] and
the walk requires it. What the geometry probes (DamageLog::geom_dump) and five fight designs
established:

- The hall's ONLY nav entry from the meet room is the south door at (7.9, 11.1); the west door #2
  (−14.1, 13.1) is the level's known nav wedge (re-confirmed: the 3000-tick cap from the courtyard
  side too).
- From the meet room the crowd holds its north placements ((5.0,14.6) #6, (6.3,17.0) #7) and every
  probe — the plain segment AND the scoped 0xa-mask ray the game's own auto-aim uses [M
  0x1a73f8:164-166] — reads them fully occluded behind the hall's south structure, in BOTH
  directions (their muzzle→Bond-chest ray blocks too, t 0.09, while the door is closed). The
  hits-through-the-door in the mid-run timings happened with the door OPEN (Bond's own nav opens
  it): with the aperture open their rounds cross the same plane the probes respected when closed.
  No port divergence found — the door state explains every apparent asymmetry.
- The blind east stage walk dies mid-path: the nav threads the crowd (death at (5.4,14.3), 0.5 u
  from #6; a 14.0 TooClose pistol round, then the captain's full 8-pellet shotgun volley at 1.9 u).
- The door funnel's nav hops cannot stop short: an 8.0 arrive radius overshot to 0.7-0.9 (the hops
  go through the doorway). Approaches into the hall are unwinnable at Agent.
- Fighting the doorway from the south at 6.8 u (the keep-off back through the door) produces a real
  exchange but loses on DPS: two 8-pellet shotguns vs Bond's gun, 36 hits for the last 100 health.

What a player has that the walker does not yet: a grenade through the doorway, or the right gun for
the trade. The next session should pick up exactly there: (a) verify what weapon the meet hand-off
leaves (clip 32 spare 0 at the pre-hall check) and whether #13's SMG drop in the spawn pocket can
be walked to before the hall; (b) if the trade still loses, the frag through the door gap is the
player's move — that needs a grenade-throw helper in the walker (shared-helper change → per-level
opt-in per the rules).

## What I ran

- `cargo test --offline --release -p nf-view --test walkreal_07000001_live` — 6 passed / 1 failed:
  every Operative master green including `..._real_damage`; the Agent sister fails at the hall.
- `cargo test --offline --release -p nf-game --lib` — 655 passed, 0 failed.
- `cargo test --offline --release -p nf-view --test fix_grenade_types_live` — 4 passed (scratch
  probes were added and removed there during the diagnosis; the file is back to its committed
  state).
- The full four-crate fast suite did NOT complete in this sandbox: background jobs (nohup and
  setsid alike) are killed between tool calls and the foreground run exceeds the 10-minute
  tool-call window (the all-binaries build alone consumes most of it). The blast radius of this
  branch is the one walkreal_07000001 binary (no shared helper changed: crates/nf-view/tests/common/
  is untouched) plus nf-game's lib tests, both run green above.

## What's left

1. The hall fight: the sister's last death (tick 3366) is the east stage walk into #6. The door
   trade at 6.8 u is the closest loss; the weapon/grenade lever above is the next move.
2. The full fast-suite run on the final tree (blocked by the sandbox's process reaping, as noted).
3. The Operative walks on the OTHER levels are untouched by construction (no shared helper
   changed); only this level's binary was re-run.

## Live checks for the main agent

1. The south door's aperture: gdbwatch the door object at (7.9, 5.3, 11.1) through a hall fight —
   confirm the open state's gap width and that the guards' rounds cross ONLY while it is open (the
   port's runs agree; this settles the "shots through a closed door" suspicion the mid-run traces
   raise).
2. The hall crowd's hunt trigger: what alert state/timer moves #6 from (5.0,14.6) to the door
   (6.5,11.2) between the meet and the stage — if the game's guards do NOT hunt at Agent, the
   walker's hold-at-the-door design is wrong and the fight should be taken at their placements
   from the east stage instead.
3. The meet hand-off weapon: what gun + ammo the cut-scene leaves in Bond's hands (the port reads
   clip 32 spare 0) — the door trade's DPS verdict depends on it.

## Usage

not visible to me

## ntfy failed

All three notification fallbacks were unreachable from this sandbox:
- `curl -sS -m 15 ... https://ntfy.sh/nightfire-yoda-8k3x` → `curl: (28) Connection timed out after 15002 milliseconds`
- `python3 urllib.request` → `SyntaxError: bytes can only contain ASCII literal characters` (the em dash in the
  one-line result broke the bytes literal — retried with the plain `done` payload, no output inside the shell's
  25 s window, killed)
- `wget -q -O- --post-data "level-01-agent-escort-fight: done" ...` → no output, timed out at the 20 s cap

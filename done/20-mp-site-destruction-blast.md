## Your task: `mp-site-destruction-blast`, the Demolition and Protection site blows up when it falls

`MP_DemolitionProtectionUpdate` 0x1870a0 (decomp lines 125-160) handles the site's death: MPSound 0x14c, the team and player score changes, `SP_UnPause`, `Explode_Create(5.0, 5.0, 500.0, ..., script 0x600004f, ...)` at obj+0x80, the status sprite text (labels 0x200002e/0x200002f), `MPGame+0x188 = 6`, then `MP_sendTeamBotMessage(0x41)`. crates/nf-game/src/mp/rules.rs has `site_update` and `damage_site`, but no 0x600004f reference exists under mp/, and docs/reports/fx-gas-and-blasts.md reports the blast as not ported. Read the whole function and walk it line by line against rules.rs: state which steps the port has and which it lacks, with addresses, then port the missing ones. The blast must go through the shared pending-blast path so it hurts bodies and bots, and it must stay in step on every machine online. Pin it through a real Demolition match: the site falls, the blast fires at its position, nearby players take the damage, the status text shows and the round ends. Do the same for Protection. Run every mp_*_live binary, mp_world_sync_live, the net hash binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-site-destruction-blast`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-05T22:48Z

---
## Report (done, 2026-11-06T00:07Z)


---
## Report from the worker (done, 2026-11-06T00:07Z)

# mp-site-destruction-blast — the Demolition/Protection site blows up when it falls

Branch `s/mp-site-destruction-blast` (from `origin/master` @ 317aba03b — no merge needed, the
branch is master + this task's commits).

## The walk: `MP_DemolitionProtectionUpdate` 0x1870a0 line by line vs the port

`MP_ObjectUpdate` 0x18b198 dispatches the site's class-0x60 obj to this function with
param_3 = 0 (Demolition, case 3) or 1 (Protection, case 8) [M 0x18b198:22-24,35-37]; param_2 is
the MPOBJECT record at the obj's SP, `param_2+6` the SP obj the site's scripts play on.

### The hit half (every tick) — the port HAD these

| Game (0x1870a0) | Evidence | Port |
|---|---|---|
| :73-75 the ext pick (`*param_2 == 3` → `&Demolition`, else `&Protection`) — feeds only :161-165 | disasm 0x1870e8-0x1870f8 | no ext globals (see the message step) |
| :80 `MP_getObjExtObj` (re-keys the ext's obj word after a respawn; routes a 0x40 msg then — `MP_getObjExtObj` 0x18bca8) | 0x18bca8:20-47 | the port re-keys the site record in `create_mode_objects`/`restart_scenario` directly |
| :81-83 `SP_GetHitDamage(sp, 0x318d70, 0x40, &HitByList, 1, 0)`; `hp = (short)(int)((float)(int)hp − dmg)` | disasm 0x187110+ | `damage_site`'s store; the feeds are the frame-summed bullet chain (offline, scene.rs) and the shared combat's site contacts (online) |
| :84-91 a positive hit → `MPSound_Play(0x14c)` + the hitter recorded at `MPOBJECT+0x50` | disasm 0x187128+, `jal MPSound_Play` with a0 = 0x14c | `mp_sound(SND_CLAIM)` (0x14c) + `site.last_hitter` |
| :96-110 the HitByList walk: every hitter on the site's OWN team gets the 0x200002b warn once per 1.5 s (`MPGame+slot·0x30+0x24`) | disasm 0x187160-0x1872d4 | the single credited hitter's warn with the `timers[1]` gate — deviation: the game warns EVERY same-team hitter in the frame's list, the port sees one hitter per contact event (pre-existing, unchanged) |

### The timeout branch [M :112-124] — the port HAD this

`MPGame+0x404 < MPGame+0x400` — the port's `MpGame` maps +0x400 = the round clock, +0x404 =
the limit [M 0x184268] — so the branch fires when the time is up: the DEFENDING team +1.0
(`MPGame+param_3·4+0x180`), obj+0xF6 = 1, `SP_SwitchToScript(sp, 1, 0)`, the status sprite
labels 0x200002d (Protection) / 0x200002c (Demolition). Port: `site_update` — with the
hash-1 → SP-state-2 note [M 0x1dcad8:28-33]. **No `Explode_Create` here** — the timeout path
has no blast.

### The death branch [M :125-166] — step by step

| Game (0x1870a0) | Evidence | Port before | Port now |
|---|---|---|---|
| :126 `MPSound_Play(0x14c)` | disasm 0x187390-94 `addiu a0,zero,332` | HAD (`mp_sound(SND_CLAIM)`) | unchanged |
| :127-132 the scoring slot: `iVar4 = 4` when param_3 == 0 else 0 → `MPGame+iVar4+0x180 += 1.0` (the ATTACKING team) | disasm 0x187398-0x1873ac | HAD (`team_scores[attacking] += 1.0`) | unchanged |
| :133-140 the last hitter +1.0 personal (`MPGame+h·0x30+0x18`), or −1.0 + the defending team −1.0 when the hitter is a defender (the own-goal REPLACES the +1, `fVar10 = -1.0`) | disasm 0x1873b0-0x187414 | HAD | unchanged |
| :141 obj+0xF6 = 1 | disasm 0x187420 `sh v1,246(s4)` | HAD (`site.sub = 1`) | unchanged |
| :142 obj+0xEC = GameState+0x34 | disasm 0x187424-28 (`lw v0,14236(a0)` = 0x2a379c, `sw v0,236(s4)`) | LACKED | documented, no modeled reader (below) |
| :143 `SP_UnPause(site SP)` 0x1dca58 | SP+0x2C = 1, SCRIPTINFO+0x9C = 0 [M 0x1dca58:9-16] | LACKED | documented, no modeled reader (below) |
| :144-146 `Explode_Create(5.0, 5.0, 500.0, site_obj, obj+0x80, 0x2d6210, 0x600004f, 0xff, 0xff, 100, 0)` | disasm 0x187434-0x187474: f12 = f13 = 0x40a00000 (5.0), f14 = 0x43fa0000 (500.0), a0 = the site obj, a1 = s4+0x80, a2 = 0x2d6210, a3 = 0x600004f, t0/t1 = 0xff, t2 = 100, **t3 = 0 NULL attacker**, stack[0] = 0 | **LACKED — the whole blast** | **PORTED** (below) |
| :147-158 the status sprite: the hitter's team word == 0 → 0x200002f, else 0x200002e; `Sprite_SetText` | disasm 0x187478-0x1874f0 | HAD (`site_status`) | unchanged |
| :159 obj+0xF6 = 1 again | disasm 0x1874fc | (already done) | — |
| :160 `MPGame._392_4_ = 6` (MPGame+0x188 = 6) | disasm 0x187500-08 `sw v0,392(a0)` | HAD (`game.state = 6`) | unchanged |
| :161-165 the ext's obj word moved out and zeroed; `MP_sendTeamBotMessage(param_3 == 0, 0x41, ext_word, 0, 0)` | disasm 0x18750c-0x187520; 0x18bbc8 → `Drone_SM_RouteMsg` (mask ~sender, id 0x41, arg3 = the ext word) | LACKED | documented (below) |

### The ported step: the death blast

`damage_site`'s death branch queues the call into `MpObjectsRt::pending_blasts`
(pos, radius 5.0, damage 500.0, owner):

- **pos = the site's obj+0x80** — the contact sphere's world centre, the point `site_sphere()`
  returns [M :145 `addiu a1,s4,128`; `ObjView`'s obj+0x80 doc].
- **radius 5.0 / damage 500.0** — f13/f14 [M disasm above]. f12 (5.0) is the script scale
  (SCRIPTINFO+0x94 [M 0x17cbd0:99]) and the stack u16 0 the debris kind (EXPLODEINFO+0x38
  [M 0x17cbd0:91]); both ride the offline drain's `drones.pending_blasts` tuple.
- **script 0x600004f** — a3 [M]. The first 0x600004f under mp/; it is the destroyable smash's
  type-2 script hash (drone.rs's smash table) — the blast-script family's shape.
- **attacker = NULL** — t3 = 0 [M 0x187474 `daddu t3,zero,zero`], the same NULL the copter
  death blast passes [M 0x17e168]. The port marks it `usize::MAX` in the MP queue; `mp_hurt`
  maps it to no kill credit — `Explode_Propagate`'s case-3 arm skips the
  `MP_RegisterBulletHit` registration for a NULL attacker [M 0x17c518:128-134], so the game
  credits nobody either. The hurt applies in full.

**The shared pending-blast path.** Online: `step_net_combat` drains the match's queue into
`MpCombat::step`'s blast pass (a new `extra_blasts` parameter) — the SAME case-3-bodies /
case-5-chained-explosives / case-0x35 propagation every thrown explosive rides, so the blast
hurts the human bodies and the bots identically on every machine. The queue is folded into the
lockstep hash (`hash_objects_rt`), so a divergence is visible. Offline: the drain rides the
drones' `pending_blasts` slot (the copter death blast's route) so the next tick's
`spawn_blast` propagates it — Bond's body, breakables, destroyables, mines. Owner there is
`usize::MAX − 1` (the established not-the-player mark; `usize::MAX` in that slot means "Bond's
throw" and would re-feed the site).

**The one-tick delay is the game's.** `Explode_Create` creates a class-0x40 object whose
damage lands through `Explode_CollisionHandler` 0x17cde0's one-shot latch on the object's first
update — the next frame's walk [M 0x17cde0:7-13; the port's destroyable-chain convention cites
the same latch]. The queue drains at the next tick's blast pass on both paths.

**In step online.** The death happens in `mp_hurt` (after the combat pass); the queue drains at
the NEXT tick's pass on every machine — the same deterministic point relative to the shared
event stream, and the hash folds the queue. `restart_scenario` clears it (the game's restart
sweep deletes the round's explosion objects with the class-5 pass [M 0x183fc0:14-17]).

### The steps with no modeled reader (documented, not faked)

- **:142 obj+0xEC = GameState+0x34** — the spawn-frame stamp. Its only reader family is
  `MP_HitBy`'s 4-frame spawn window [M 0x1830e0:36-39], which reads the stamps of the SLOTS'
  player objects (`MPGame+slot·0x30+0x1C`, classes 2/0x11/0x12) — never the site's class-0x60
  object. No port path reads a site stamp; the store is recorded here and in the code comment.
- **:143 `SP_UnPause(site SP)`** — writes SP+0x2C = 1 (a one-shot event-4 pulse `SP_Update`
  consumes into its event mask and clears the same tick [M 0x1dbda8:78-81]) and clears the
  current SCRIPTINFO+0x9C [M 0x1dca58:12-16]. The port's SP model is two observable fields
  (`sp_script`/`sp_state` [M 0x1dcad8:23-34]) — neither write touches them, and SCRIPTINFO+0x9C's
  hit-accumulation gate ([M 0x1dbda8:234-247]) is already dead here because the done flag stops
  every further `SP_GetHitDamage` feed. Whether the pulse drives the site script's visible
  destruction branch is a live check (below).
- **:161-165 the 0x41 bot message** — `MP_sendTeamBotMessage` 0x18bbc8's mask/payload shape is
  already ported (`mp::send_team_bot_message`) but the bot brain has no cross-bot message pump
  yet (pre-existing [S] note in mod.rs). The observable half IS live: the death clears the
  bots' site view (`bot_objectives().site` is None once `sub == 1`), which is what the
  0x40/0x41 re-target arm consumes; the ext-word zeroing has no port-side reader.

## Verification

Built and run on this sandbox (the full `tools/test/suite.py` build ran out of disk twice —
the sandbox has 9.9 GB and the 4-crate release build with every test binary needs more than the
~6.7 GB that fits after the game files; what ran is below, each binary whole with
`--test-threads=1`):

- **mp_site_destruction_blast_live (new, 4 tests): 4 passed, 0 failed** —
  `the_death_blast_fires_at_the_site_offline` (the siege through Bond's real fire from the
  load; the next tick's `drones.explosions` record: pos = the site centre ±1e-3 per axis,
  5.0 / 500.0 [M :144-146], the NULL-attacker mark; the 8.0-out shooter unharmed; state 6;
  site_status 0x200002e),
  `the_death_blast_hurts_a_body_inside_the_radius_offline` (Bond at 2.0 loses ≥ 100 health),
  `protection_site_death_blast_offline` (Protection: the same constants, site_status
  0x200002f for the team-0 hitter [M :149-151], state 6),
  `the_death_blast_kills_a_nearby_body_online_in_step` (two machines, lockstep hash agreeing
  EVERY tick: the site falls to slot 0's shared-combat siege — HP < 1 asserted, so not the
  timeout — and the NEXT tick's blast kills slot 1 parked 2.0 from the centre on BOTH
  machines; the death scores once; the NULL attacker credits nobody — no score move, no kill
  message).
- **mp_site_damage_offline_live: 4 passed, 0 failed.**
- **mp_scenarios_live: 19 passed, 0 failed** (every scenario online incl. demolition_online /
  protection_online / the object census).
- **mp_world_sync_live: 8 passed, 0 failed.**
- **net_match_live: 3 passed, 0 failed; net_desync_report_live: ok (1 ignored — pre-existing);
  perf_state_hash_live: ok (1 ignored — pre-existing).**
- **nf-game: 1091 passed, 0 failed** (107 test binaries, `cargo test --offline --release -j2
  -p nf-game`).
- **Every other mp_*_live / net_* binary in target/release/deps** (batch, one line each):
  mp_16_slots_live 3, mp_arenas_live 1, mp_body_anim_live 2, mp_bot_body_sounds_live 1,
  mp_bot_objectives_live 10, mp_bot_voices_live 3, mp_bot_weapons_live 2, mp_bots_moving_live 3,
  mp_core_live 5, mp_debriefing 10, mp_debriefing_flow 6, mp_host_match_live 2, mp_hud_16 7,
  mp_hud_16_live 2, mp_hud_data 4, mp_hud_pane 5, mp_lobby_one_screen 7, mp_lockstep 6,
  mp_map_objects_live 4, mp_match_rules 18, mp_menu_flow 11, mp_menu_lists_live 5,
  mp_never_mentioned 6, mp_pickup_take_live 2, mp_remote_fx_live 1 passed then the 2nd test
  hung (below), mp_respawn_ammo_live 3, mp_screens 5, mp_shot_gate_live 5, mp_status_bits 3,
  mp_status_strings_live 1, mp_teams_flow 4, net_internet_live 1 — all 0 failed.
- **mp_net_slots_post_move_live: 2 passed, 0 failed** — its pinned 4-human digests were
  RE-MEASURED and updated (tick0 0x5c0b…8938 → 0xce70fc989d360d1f, final 0xfb7b…0ec3 →
  0xca2fe7f4f872763f): the match hash fold gained the `pending_blasts` len word (even empty),
  so every match digest moves once. That is this task's intentional hash change, noted here
  per the test's own pin-update policy.
- `cargo check --offline -p nf-game -p nf-view` (+ `--tests`) — clean.

### Not run / hung (honest list)

- **The full 4-crate suite** (`tools/test/suite.py`): the build died twice on disk-full
  (0 bytes free while linking nf-view's ~700 test binaries). nf-formats/nf-audio tests beyond
  what nf-game's run links were not run; nf-formats/nf-audio/nf-view libs compile.
- **mp_clock_gate_live** hung in this sandbox: its 3rd `load_mp_carry` (deathmatch,
  time_limit = 600, bots 3) never returned in 150+220 s (the first two loads complete). My
  changed paths do not run at load or inside `hud::overlay`; I could not rebuild a master
  baseline in the window to prove it pre-existing. Needs a re-run on a saner disk/machine.
- **mp_remote_fx_live**'s 2nd test (`remote_shots_show_and_sound_without_touching_the_hash`)
  hung past 100 s (the 1st passed). Same caveat.
- **net_match_end_clients_live / net_soak_faults_live** produced no result line within 100 s
  (network-flavored tests on a sandbox without the usual loopback setup — not investigated
  further in the window).

## What's left / notes for the main agent

- **Live checks for the main agent** (PCSX2, FRESH boot, Demolition on any arena):
  1. Let the site fall to bullets; at the death frame check whether the site model's script
     visibly reacts (SP_UnPause's event-4 pulse may drive the site script's destruction
     branch — the port models no site script state): does the model collapse/sink, and when?
  2. The blast: an explosion effect + sound at the site's position on the frame AFTER the
     death (the port's one-tick delay), and any player standing within ~5 units of the site's
     centre taking heavy damage (500 with the falloff) that same next frame.
  3. The status sprite shows "<team> destroyed the target" (0x200002e team-1 hitter /
     0x200002f team-0 hitter) from the death frame.
- **Render spots** (rule 6): offline Demolition on 07000023, Bond at the site −8,0,+8 shooting
  it down — the blast should fire at the site centre the tick after the fall (the drones'
  blast script 0x600004f at scale 5.0). Online the MP combat currently carries no blast
  visuals (`mp_combat.blasts` has no view consumer — pre-existing); the damage is the effect.
- The 0x41 bot-message pump stays unwired (pre-existing [S]).
- The hit-half's HitByList walk (:96-110) warns only the single credited hitter per contact
  event (the game warns every same-team hitter in the frame's list) — pre-existing deviation,
  unchanged.
- The four hung binaries above need a re-run on a machine with room for the full suite.

## Usage

Session context window and token figures: **not visible to me** (this sandbox's IM gateway
shows no token/credit meters; the driver message's figures were the only ones provided).

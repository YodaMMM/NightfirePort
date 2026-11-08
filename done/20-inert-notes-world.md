# Port notes that say not ported: the rest of nf-game

The same grep over crates/nf-game/src except drone.rs and mp/. For each note: is the arm reachable on any on-foot level? Close with a count, or port it.

Read the game code first, count on every level with `Level::parse` over `nf_view::dev::LEVELS`, and test on the real level, red on the old code. A count of zero on every level closes an item with that count as proof. Do not trust a port comment that says inert. Values measured at load can differ from the same global in play (FRAME_RATE reads 30 while a level builds, 60 after). Branch from origin/master, push to `s/inert-notes-world`, report in docs/reports/inert-notes-world.md. No driving levels. Draw work: leave a stub in the hand-off list and say so.

---
claimed-by: sandbox3 at 2026-11-08T00:49Z

---
## Report (done, 2026-11-08T01:21Z)


---
## Report from the worker (done, 2026-11-08T01:22Z)

# inert-notes-world — the "not ported" notes over the rest of nf-game

Task: the same grep over `crates/nf-game/src` except `drone.rs` and `mp/` — for every port note that
says not ported / inert: is the arm reachable on any on-foot level? Close with a count, or port it.
Read the game code first; count on every level with `Level::parse` over `nf_view::dev::LEVELS`; do
not trust a port comment that says inert.

## Method

The grep (`not ported|unported|never ported|inert`, drone.rs and mp/ excluded) hits 144 lines across
39 files. Most are doc cross-references to already-ported code, test names, or the documented
stand-in policy — not unported arms. The genuine items were each read against the decompile and
counted over the disc by **`crates/nf-view/tests/inert_notes_census.rs`** (committed, green):
`Level::parse` over all 34 on-foot levels of `nf_view::dev::LEVELS` (no driving levels) plus the
ACTION.ELF tables (weapon_data, EffectInfo). The zero-count closes are pinned as assertions in that
test — red if the data ever changes.

## Closed with counts (the arm is unreachable on the shipped data)

1. **nis.rs:129 — the DroneEnable hash extras** (0x600001b → msg 10 + Music_Event(0xf);
   0x600002e → msg 0xe walk). The note itself claimed "neither script is live on the ported levels";
   the census proves it for ALL on-foot levels: **0** archives carry either hash (asserted per
   level). Unreachable; the note now cites the census.
2. **space_laser.rs:317 — the ch-19 arm gate** ("inert on retail data"). 0700001b carries **0**
   triggers writing ch 19 (asserted). The level doc's claim [D docs/levels/0700001b.md] verified
   against the parsed placements.
3. **ss.rs:157/181 — the class-2 drone contact bit (mask 0x10)** — the note's "no disc SS carries a
   0x10 mask [D census]" is **WRONG as written**: 07000046 carries **8** SS records with the 0x10
   mask. But the level has **0** drone placements (kind 0xF/0xF1) — no drone body can ever exist
   there to make the contact, so the bit can never fire. Closed on the combined count
   (8 mask-0x10 records ∧ 0 drones), asserted; the stale census claim is corrected here and the
   ss.rs doc comment stands (the port feeds bits 0/1 only).
4. **nav.rs:682 — `AINetwork_RouteNodeOffset` 0x15a820's flag-3 bounce arm**. Dead code, not a data
   gap: the route's flag word (the AIRoute's first u16 at DCVars+0x970) is written by exactly one
   function — `NDrone2_InitAIPath` 0x153720:59-65 — which stores **1** (path+0x80 & 4 clear) or
   **2** (set), never 3; every other +0x970 writer ORs unrelated bits (0x20/0x80 — the AllyLead
   states) or only calls LinkCreep_Calc [M decompile scan: AssignAIPath 0x154190,
   ReFindMissionPath 0x153a30, DefaultInit 0x14b300, Player_Init 0x1910f8]. The port's
   `RouteMode::{OneWayEnd, Loop}` are exactly the game's 1/2. The census corroborates: the paths
   with the loop bit are common, none can yield a flag-3 route. nav.rs's comment updated with the
   citation.
5. **gun.rs:167 — `Player_WeaponHasAmmo` 0x1a16c0's 0x20000 ground-pickup scan** (the STANDIN
   returns false). Four weapon records carry `+0x68 & 0x20000` (0x38, 0x39, 0x5a, 0x5c — ammo types
   22/30) [D weapon_data], so the arm is live *for those ids* — but nothing can put one of them in
   Bond's hands on foot: **0** placed weapon pickups grant any of the four on any of the 34 levels
   (asserted), and `Player_InitWeapon` 0x1a7ef8's SP loadout cases never grant 0x38/0x39/0x5a/0x5c
   [M player.rs `init_level_loadout`'s case list]. Closed for SP; the MP weapon-set rows are the one
   unverified source (live check below).
6. **kiko.rs:101 — "The unported start states"** — STALE. 0x85 EnemyRunToPoint and 0xb4 DeleteMe are
   ported in drone.rs [M drone.rs:278-280 (the state const + address), 2336/2364 (the start-state
   dispatch arms), 336 (the ch-10 → 0xb4 arm), 1888-1893 (the delete_me field)]. No action.
7. **world_fx.rs:726 — `Particles_Create` unported**. The game's own particle system is a stub on
   the disc: `Particles_Update` 0x1cf528 self-kills through the stubbed `psiParticleUpdate`
   0x111290 [M; the module doc]. The port's omission matches the shipped behaviour. Closed.
8. **music.rs:1152 — the tabled unported music consumers**. The real disc rows carry no
   combat/stealth scan sets (e.g. 0700000b's end_combat == end_calm == 17) [D MapMusic 0x24f4d8;
   pinned by the music.rs row tests]. Closed on the rows.
9. **Policy closes (no game arm behind them)**: sdk.rs (the C-library layer — every function
   replaced by a Rust/host equivalent, the documented policy); trig.rs:285 (the libm layer — the
   guard behaviours pinned in the ports' tests); triggers.rs:103 (`Discrete_Delete` — the port's
   owned-Vec removal subsumes the pointer clear); subtitle.rs (the queue machinery IS the port —
   the note is the mapping doc); useobj.rs:542 (a test's world stand-in comment);
   ps2math.rs:117/125 (`Creature_CalcWaypoints`/`CalcPoint` — the aux math IS ported there; the
   creature callers have no placements to feed them — see the census' 0-drone rows for the shape).

## Reachable — named with counts, remaining work (not ported this session)

These arms have disc data that reaches them on on-foot levels. Each is listed with its reach so the
next session can port or ticket it:

1. **copter.rs:31-42 — the spot check** (case 1's 240/DiffMod timer, the 1000-unit cone/LOS,
   `attackmode_159`, behind the `rec+0x74 != 100` Mayhew latch [M 0x17dbb8:138-187]). Copter
   records exist on **07000005 (2), 07000008 (1), 0700000b (1), 07000012 (1)** [census]. A prior
   task left an explicit warrant ("homing-copter-3 ... the spot gate itself stays unported,
   `timer` never moves") — reopening it needs that task's context.
2. **gas.rs:25 — the type 9/10 hidden-emitter arms** of `gases_smoke` 0x1ab2d0 (case 9: every
   4th-frame &3==3 child kind 8; case 10: every 7th-frame child kind 7, both after a 20-tick gate
   [M 0x1ab2d0:146-178]). **Reachable**: EffectInfo carries **11** gas entries with kind 9/10
   (5×9, 6×10) [census; the table-driven sites Effect_Bullet 0x1ce8a8 / Effect_Laser 0x1cef38 pass
   them]. The port's pool handles 0/4/7/8 — the two emitter arms are the port.
3. **wire.rs:174 — the creep-wall E-grab** (`Player_Activate` 0x1a5170:67 grabs a creep wall when
   +0x95F == 6). Creep walls on **07000005 (2), 0700000d (1), 07000041 (2)** [census].
4. **ladder.rs:132 — the true glist-mesh pick refine** (`Collide_Intersect` per candidate,
   0x1ea398:84-93) — the port's bounds-box stand-in is documented [S]. Ladders on **13** levels
   (up to 4 per level) [census].
5. **door.rs:668 — `Door_ReactToHit` 0x135980 via the Collide_Update proxy list** (U-041). Doors on
   **25** levels (up to 162) [census]. The port deliberately matches the disassembly's call
   structure today (A-11b); the proxy-list pass is the missing infra.
6. **door.rs:188 — `obj+0xFD` bit 3 (p4), the collision-portal link (block 0x21)**. Doors with p4
   set: **25 levels, e.g. 162/162 on 0700004a, 16/19 on 0700000a** [census]. The link drives the
   cel-graph portal state — draw-side (the sim has no portal graph); hand-off below.
7. **explode.rs:98 — the debris spawns** (kinds 5/6/8/10/0xB, the gate
   `lifetime>>2 < age && age % (Rand(5)+10) == 0` [M 0x17c3d8-0x17c460]). Explosions are
   runtime-created (breakables, scripts, the copter death) — reachable on any level with
   explosive barrels or scripted blasts. Needs the debris system (sim + draw).

## Draw-side hand-offs (stubs left; the sim is not the gap)

- door.rs:188's p4 portal link (block 0x21) — the door drives a cel-graph portal; the view side
  owns the graph (`CelGraph`, `portal_visible_cels_*` in nf-view).
- searchlight.rs:11/373/555 — the spot's dynamic light, the `Gas_Init` smoke on the drawn frames
  (already RNG-modelled), the destroyed-spot model swap.
- player.rs:1599 (the HUD air pane), hud_pane.rs:211 (the goggle item wiring),
  player.rs:538/603/3302 (`CAM_ORBIT` — the script camera modes; presentation),
  world_fx.rs:461 (`CreatePointerSpriteList` — the front-end menus), anim.rs:76 (the anim skin
  cel-hashtable path — the draw side's skin data), rigid_body.rs:284/1229 (the RB sound and the
  +0x100 light-position sync), death.rs:182 (the 0xc footprint decal), sensor.rs:25/176 (the lens
  and mount visuals; the monitor-group id — monitor.rs owns the field map), view.rs:2
  (`unported-render-side-1`), effect.rs:14 (the material-0x80 impact roll — the port's HITDATA
  carries no +0x51 second byte; audio/visual), space_missile.rs:41/96 (the anim-entity/sound
  streams).

## Live checks for the main agent

1. **The MP weapon-set rows** (the gun.rs 0x20000 close's one hole): with a debugger on an MP
   match, check whether any MPSettings weapon set grants weapon 0x38/0x39 (or 0x5a/0x5c) to a
   spawn — expected: no (then the ground-pickup scan arm stays dead on retail data outright).
2. **The gas 9/10 emitters in play** (ports the next session will make): pick an EffectInfo record
   whose gas kind is 9 (5 exist) — via `tools/trace/gdbwatch.py`, watch `Gas_Init` (0x1aacc8) calls
   with the stack kind byte = 9/10 during a bullet impact on the matching material — expected: the
   call happens with kind 9/10, then `gases_smoke`'s case 9/10 spawns a child (8/7) on its timer.

## What's left

- The seven reachable items above (port or ticket each; the gas 9/10 arms and the creep-wall E-grab
  are the smallest two).
- The two live checks.

## Usage

Token, context-window and cost figures for this session: not visible to me (no token/cost display
is shown in this environment; the IM gateway metadata carries no usage fields). Nothing estimated.

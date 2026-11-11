# level-14-fusebox-zap — report

Branch `s/level-14-fusebox-zap` = `origin/s/level-14-skybridge-retune-2` bc25f82c8 (origin/master
already an ancestor — no further master commits existed at branch time). Task: 07000014's
Operative real-damage walk survives the fusebox zap. All changes are in
`crates/nf-view/tests/walkreal_07000014_live.rs` and docs; **no `crates/*/src/` code changed**
(the sim sources are byte-identical to the base branch — every fix is walk-file logic, gated to
the real-damage walk), so the every-walkreal rule is not triggered; the plain walk and the
tripbomb route re-ran green anyway as the byte-identical proof.

## Done — the zap's source found, the port verified faithful, the walk re-routed around the trap

**The zap's source, from the decompile and the level data.** The chain that detonates at the
fusebox #383 stance:

1. Cutting fusebox **#383's GROUP-1 wires** writes the group's channels — param 2 then param 4
   = 113 = 0x71 [M `FuseBox_Update` 0x1adfe8:110-123 — group 1's words are tag+0x56/+0x5E, the
   placement's params 2/4; D the placements census: #383 params 2/4 = 113]. (The walk cuts
   group 0, whose channels are param 1/3 = 135 = 0x87 — the write the cut loop waits for.)
2. `DESTROY_BoxSatellite` **#388** (kind 0xe1, at (−90.41, 0.90, 34.61) [D the census]) has
   kill channel = param 1 = 113 [D; the port's `Destroy_Update` 0x12c4f8 second-arm mapping]:
   ch 0x71 turning on smashes it the same tick [L the run: the smash printed frame 38956].
3. `Destroy_Smash` 0x12bb60 detonates its death blast: damage = param 2 = 50 × 10.0 = **500.0**
   [M 0x12bd04], radius = param 5 = 1000 × 0.01 = **10.0** [M 0x12bc50-0x12bd20] — exactly the
   measured pulse: `explosion at (−88.7, 0.9, 32.6) d 3.2 radius 10.0 damage 500.0` [L the
   NF_DMG_TRACE run], 99.0 applied at Bond's 99 hp, 3.24 u from the box centre.

So the game itself detonates a lethal trap when the group-1 wires die: **the port matches the
game end to end** — nothing to fix in the sim. (The fusebox's own cut hurt was already ruled
out by the base branch: param 2 non-zero gates the 5.0 live-wire hurt off [M 0x1adfe8:99-101].)

**The trigger was the walk's own aim line.** The two cables of #383 sit side by side: group 0
at x −89.47..−89.35, group 1 at x −89.34..−89.15 (both z 30.89, y −0.73..−0.50 [L the wire-geo
probe through the entity matrices]). From the east stance (−86.46,−1.10,31.39) the aim segment
to the group-0 cable centre clips the **group-1** wire's AABB just before reaching the group-0
slab, and the game's own first-contact law — the nearest wire stops the round, "a wire behind
it never sees the hit" [M `Collide_PickObj` 0x1eae80 → `Collide_Update` 0x1ea398:212-229] —
cut group 1 with the cut's **first bolt** [L the fire-tick backtrace: bolt born 38955 from
Bond's muzzle (−86.72,−0.48,31.35), weapon 0x4e]. A player standing there sees the near cable
in front and steps aside; the walk now does too.

**The fixes (all in `cut_fusebox`, all `!tops_up`-gated — the plain walk byte-identical):**

1. **The stance fire-line gate**: a candidate is skipped when the segment from the stance to
   the group-0 aim point clips any OTHER alive wire's slab (the contact test's own ±0.01
   padding), probed from the head AND the drawn muzzle's ~0.9-u-under-head origin [L the same
   trace: muzzle y −0.48 vs head +0.4].
2. **The one-bolt cut cadence**: fire one bolt, hold the tick after it resolves. The loop's
   channel read lags the wire death by one tick (the contact registers in tick L's bullet
   pass; `FuseBox_Update` reads the lists at L+1's head [M 0x1adfe8:88-123]), so the old
   hold-fire cadence's overrun bolt landed on the neighbouring group.
3. **The fight-free approach**: `dmg.return_fire` off across the stance candidates — a player
   walks to the cut spot without shooting (the fight law fires the in-hand weapon; Bond
   reached the stance holding the Kiko scene's scripted Laser [L]).

**Result [L the run on this tree]**: both cuts land on GROUP 0 — `ch 0x87 on at +45 ticks`,
`ch 0x88 on at +1 ticks` — the satellite boxes never smash, Bond survives the hall at **79.6**
and fights into the machine room ("the Kiko corridor east: drone #19 killed true at tick
49158"). The d1 walk's death-before-leg-3 is gone; it now wedges at a leg it never reached.

## What ran

- `walkreal_07000014_live` whole: **8 passed, 0 failed, 2 ignored** (the plain walk and the
  tripbomb route green — the `!tops_up` gates held them byte-identical; the d1 walk and the
  00 Agent walk carry their ledgers).
- `interactables_07000014_live` whole: **10 passed, 0 failed**.
- The d1 walk itself: red (see the ledger) — the zap blocker is SOLVED and the walk runs
  ~10 000 ticks deeper than the base branch's ever did.
- `nf-game --lib` was not re-run this round: no `crates/*/src/` change landed (the local
  diagnostic patches used to trace the chain — the smash/wire-hit/fire prints and one fire-tick
  backtrace — were reverted before commit; `git diff` on the sim sources is empty).

## What's left

- **The d1 walk's new blocker: "the Kiko corridor east"** wedges at the 6000-tick cap — Bond
  (−44.35,1.01,30.95) vs target (−49.20,0.90,38.40), d 8.89, grounded, health 79.6,
  segment_to_target Some(0.128), the nearby door (−42.7,1.2,29.9) state 0x4 pos01 1.00
  lock_flag=1 ch_lock=0 [L the run]. Levers in the ignore message: the leg through the nav web
  instead of the straight steer; the door's lock semantics; the generator-hall cascade's
  scripted blasts (the cuts' channel writes smash placements 400/1010/787/827/403/1009/669/
  740-742 through f40-42k [L] — Bond held 79.6 through them; the route east must keep that).
- The SkyBridge crossing re-route (the trench-floor claw-up) and the 00 Agent walk are
  unchanged from the base branch (not this task).
- **A [M]-backed observation for a future task** (not changed this round — it would move blast
  damage on every level): the game's `Explode_Propagate` falloff truncates the gap to an int
  before dividing by the radius (`param_2 · (1 − (int)gap·(gap>0)/param_1)` [M 0x17c518:77-82,
  the `(int)` cast in the decompile — worth a disassembly confirm before any port change]); the
  port's `explode::falloff` is linear. Both laws are lethal at the stance here (500 base), so
  this task's outcome is unaffected; a live check or a dedicated task should settle it.

## Live checks for the main agent

1. **The trap in play**: on a fresh boot at Operative, stand ~3.2 u east of fusebox #383
   (−86.5, −1.1, 31.3), draw the Laser, and fire ONE bolt at the RIGHT-hand cable (the group-0
   cable, x −89.41) — the near-left cable (group 1, x −89.25) should occlude the line from
   this stance; expected: the near cable dies, the satellite dish box at (−90.4, 0.9, 34.6)
   explodes for ~400-500 through the falloff, and Bond (100 hp, no armour) dies or nearly dies.
   Then repeat from ~2 u SOUTH (z ≈ 33.5+) where the group-0 cable is directly visible:
   expected: ch 0x87 fires, no explosion, Bond unharmed. This confirms the first-contact law
   and the trap's radius/damage the port pins.
2. The base branch's open checks stand (the crawl lintel at x ≈ 41.0; the Q-Claw #658 latch and
   release landing).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

## ntfy failed

All three fallbacks failed (the same outbound-egress wall every prior round recorded):

1. curl: `curl: (28) Connection timed out after 20002 milliseconds`
2. python3 urllib: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. wget: rc=124 (killed at the 25 s timeout), 0 bytes written.

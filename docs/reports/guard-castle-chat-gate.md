# guard-castle-chat-gate — the 07000006 castle chat pair's spawn gate, found; the chat still breaks off early

Branch `s/guard-castle-chat-gate`, cut from origin/s/guard-castle-chat-states c1055e9cf, merged
origin/master 08a276ce0 (merge commit fb2fcabb6).

## Done

1. **The spawn gate, written down with addresses.** The pair's two kind-0x0f placements
   (skin 0x05000030 param 1 = 0x06000124 at (31.369, 39.502, 22.053); skin 0x0500002e param 1 =
   0x06000126 at (30.457, 39.329, 23.081) [D nf-extract drones census]) spawn AT LEVEL START —
   the previous branch's "event-gated, no present drone carries the specials" read was wrong:
   the pair ARE created and DO enter CastleChatGuard1 0x36 from their first tick. Evidence:
   param 2 = 1 passes the difficulty gate (`param2 <= difficulty` [M Drone_Create 0x136be0:49]),
   param 3 = 0 means no switch channel (no WAIT_SWITCH [M DefaultInit's start-state pick]),
   and the trace shows `t1 g13 0xff -> 0x36`, `t1 g14 0xff -> 0x36` — mode 0x18 from the
   DoModeSettingsNEW start-script specials [M 0x14c600:147-158], PLAY_SCRIPT's ENTER redirect
   [M 0x15ba68:26-30]. The earlier probe's "no drone carries the specials" was the pair having
   ALREADY left 0x36 by the walked arrival (see the ledger below), not a spawn gap. What the
   port does on 07000006 now matches this: same creation path, same specials, same redirect.
2. **Channel 0x11 census (item 2).** No touch trigger raises it: the triggers dump's only 0x11
   row is placement **512, type 0xc MultiplexOut out 0x11 → list [0x3a, 0x3b]** — a fan-OUT
   (out = source, list = destinations [M Trigger_MultiplexOut 0x1b1a78 case 5]), not a raiser.
   The pair's own scripts 06000124/06000126 are pure anim+voice (op 0/1 commands only [D
   script-info]). No mission-data raiser found in the pass. The live runs show ch 0x11 UP on
   the pair by tick ~1500 [L /tmp runs, this sandbox], so a raiser exists in the mission/script-
   EVENT layer — the NIS stream records (event 0xd SetScript(channel, value), nis.rs's parse)
   are the unscanned layer; ledgered as the open census item.
3. **The sight-bit fix in the game's way (the one [M] divergence found and fixed).** The game's
   EnemyLookForOpponent requires the sight bit `+0x228 & 4` before ANY break-off [M
   0x177ed0:26-28]; ProcessDroneSight clears that bit on every pass without a held sight [M
   0x176a28:106 and :200] and HaveOpponentSight re-sets it only behind the reaction gate on a
   tick whose LOS ray (bit 2, SeekOpponentLOS 0x175cb8:104) held [M 0x175ef0:33-36]. The port's
   look_for_opponent used `self.seen` — a latch that NEVER clears for team != 2 (drone.rs's
   sight pass clears seen_ticks/lost_sight but not seen) — so a guard broke off any state that
   consults the look on the strength of an EVER-acked sighting, where the game needs the sight
   HELD THIS TICK. Both arms (player-opponent and team-2 drone-opponent) now take the per-tick
   form `los && seen_ticks >= reaction_time`. **Red on the old code**: the red runs below are
   the old-code behaviour (the fix did not change the pair's outcome — the break is a different
   path), and the unit suite pins the rest: nf-game lib **791 passed, 0 failed, 4 ignored**.
4. **The chat test re-ignored with the full ledger + next levers** (the message in
   walkreal_07000006_live.rs), after un-ignoring it and going red three ways.

## The red ledger (why the test is still red) [L NF_ALARM_TRACE / NF_AILOG, this branch]

- t1: both pair members enter 0x36 ✓ (the spawn gate works).
- t456: **g14 0x36 → 0x58 → 0x5a → 0x8c → 0x93** (the undercover family) at d 14.3, while Bond
  crosses the greeting plates. NOT the sight path: NF_AILOG shows `sight_now=false los=false`
  on every 0x36 tick for both guards — the break came through the 0x36 alert arm
  (msg::SHOUT|SOUND|ALARM → enemy_alerts → Some(0x58)) [M 0x1612e0:83-98].
- t875: g13 0x36 → 0x63 (same path) → later 0x9c UnderCoverFire.
- t960/976: the walk's return fire kills g14 (Bond defending; never lower damage/health, no
  top-ups — all real input).

## Next levers (also in the ignore message)

1. Name the t456 msg source: log the port's 0x36 alert arm + `hear()` (the greeting plates'
   channel writes vs the intro NIS's sounds). Live check for the main agent: does the GAME's
   pair break off during the intro (the same msg), and does it RESUME chatting (0x36 re-entry
   through the mode-0x18 redirect), or does the game's hearing gate keep the pair un-alerted
   until Bond is close?
2. Sweep the level's mission/NIS script STREAM records (the Nis event-0xd family) for the
   ch 0x11 write — the chat script's play gate needs 0x11 up while the pair still holds 0x36.
3. The 0x36 EXIT stamps ch 0x1A when the start script never played [M 0x1612e0:33-36]; whether
   ch 0x1A drives the observed 0x58/0x8c cascade is unchecked.

## What was run

- `nf-game` lib whole: **791 passed, 0 failed, 4 ignored** (includes the merge's and the
  previous branch's castle-chat units; the sight-bit change broke nothing).
- `interactables_07000006_live` whole: **5 passed, 0 failed**.
- `walkreal_07000006_live` whole: **5 passed, 2 failed, 2 ignored** — the two failures are
  `the_castle_chat_pair_stands_chatting_and_breaks_off_when_alerted` (the ledgered chat break,
  re-ignored) and `walk_the_castle_interior_to_the_party_real_damage` (**Bond died at tick 3964**
  at (4.54, 42.03, −33.94): −54.00 from drone #9 skin 0x5000030 at d 12.4, then earlier −3.30
  ×2 from drone #3 at d 36.5). An A/B of that walk without the sight-bit fix did not complete
  (the no-fix rebuild failed in this sandbox — the binary stayed the fixed one), so its
  master-side baseline is UNMEASURED this session; the previous branch never ran walkreal_06's
  walks either. The sight-bit gate is strictly stricter than the sticky latch (sight must HOLD
  vs EVER held), so engagements should shrink, not grow — but that walk needs a clean A/B and a
  ledger entry as the first item of the next session.
- The full suite (`tools/test/suite.py`): started in the background per the driver; its build
  again hit this sandbox's disk wall (9.9 GB; the release test binaries do not fit — the same
  ENOSPC end the previous two sessions hit). **The full-suite total line: not produced this
  session.** The per-binary battery above is what ran.

## What is left

1. The t456 alert-source hunt (lever 1) and the ch 0x11 stream-record sweep (lever 2); then the
   chat test's three legs should hold: both chatting on arrival, state 0x36, the burst breaks
   them off.
2. The clean A/B + ledger for `walk_the_castle_interior_to_the_party_real_damage` (died t3964
   here; unknown on master).
3. The full suite + every walkreal_*_live battery on a disk that fits (three sessions running).

## Usage

not visible to me

## ntfy failed

All three fallbacks, in order, with their exact outputs (03:42-03:44 UTC, this sandbox —
ntfy.sh has been unreachable from every session's sandbox):
1. `curl -sS -m 15 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 15001 milliseconds`
2. `python3 -c "... urlopen(Request('https://ntfy.sh/nightfire-yoda-8k3x', ...))"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --timeout=12 --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → exit code **4** (network failure), no output.

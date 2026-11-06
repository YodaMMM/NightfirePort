# guard-radii-16-chasers — the 016 chasers walks under the game's guard radii

Branch `s/guard-radii-16-chasers` = `origin/s/guard-radii-finish` e88867f93 (master already merged).
The radii are the game's and were not touched: every change below is test-side, in
`walkreal_07000016_live.rs` and an OPT-IN walker helper (`crates/nf-view/tests/common/walker.rs`).
No `crates/*/src/` file changed.

## Done

1. **The defeat walk's "hurt volume at (−5.8, −163)" identified — it is NOT a placement volume**
   (the task's first step, done from the data). The two 0xe4 hurt placements in 07000016 are the
   1000-damage pair deep in the shuttle bay [D docs/levels/07000016.md §8 — HurtPoly-ShuttleLaunch
   #656 gate 0xf3, HurtPoly-LiftCrush #699 gate 0xcb]; neither sits at (−5.8, −163) and neither
   deals 10. The ledger's two "world" hits (−10.00 at frame 10847, −2.60 at 10959, the kill) are
   **Bond's OWN frag grenades**: under NF_DMG_TRACE the blasts print `radius 8.0 damage 100.0
   owner −1 kind 52` — the frag record's own radius 8 / damage 100 [L USATxt.dat, pinned in
   grenade.rs] — and owner −1 = usize::MAX, which is the PLAYER thrower's owner in the port's
   throw path [M scene.rs's Grenade construction: `owner: usize::MAX`; the chaser's own smoke in
   the same window carried owner 9]. The chain: `kill_the_chaser_with_range_keeping`'s dry re-arm
   cycled onto the frag 0x34 (clip 1 spare 2, owned from the chain [M 0x1a7ef8]), the kill loop's
   fire taps THREW it, and the loot walks then strolled into the fuse's blast — the second blast
   landed 4.3 u from the loot walk 2's own path.

2. **The opt-in walker gate `no_grenade_throws`** (walker.rs, default OFF — every other walk keeps
   master's behaviour; the code checks the flag): `tick_fire` downgrades a tap with a
   grenade-family weapon in hand to an idle tick (the family predicate is the port's own Grenade
   spawn gate — the record's gravity bit AND a contact-explode/bounce/stick flag [M 0x12c5a0 +
   Bullet_CollisionHandler 0x12ce80; the delta-repeater comment pins the same predicate]), and
   `switch_to_armed` cycles past the grenade hands like an empty weapon.

3. **`walk_the_chasers_defeat_completes_the_mission_on_real_damage` — GREEN** (commit bc810b551).
   Fixes, in the file: the flag above; the security-station armour #596 (−19.03, 6.67, −94.48)
   [D the census/audit placement tables] — a walked detour after the station E, the walk-over take
   [M Pickup_Handler 0x18e650], and the south gallery pair's 53.2 of free plinks (the ledgers'
   #2/#3, skins 0x500008c, y 6.7 above the trench, BOTH alive at hp 10) ABSORB into it
   [M Player_HandlePain's type-0 arm]; the pit-room armour #648 (−14.41, −22.97, −172.75) [D the
   audit's table] walked before the corpse loot; the chaser-window's side taps on the exposed pit
   shooters (#6/#7 at 4.6-13.3 u, hp 10 [L the ledgers + probe_16_guards] — the task's named
   lever); `prefer_head` (×5 [M NDrone2_HitDamage's part-5]) for the kill's ~43 rounds over her
   100 hp. The measured run: **end health 24.4, lowest 24.4, 72 hits for 75.6 health + ~50 armour
   spent, the finale walked to MissionState 6** (the lift-out hand-back and the button's E in the
   log). TRIED and REVERTED with the numbers: a far-exposed ring stand (fight exposed level guards
   out to 35 u) took the gallery 1v1 but the ring guards' close fire poured free — the 2152 death,
   71 hits for 100 (the d2 lever (b)'s trades again); trench-mouth stands to the gallery pair never
   saw them exposed (removed).

4. **`the_chasers_punch_lands_the_record_damage` — GREEN** (commit 3bf682362). The punch pin's
   own precondition (no armour at the punch — the health delta IS the pin) rules out the #596
   detour for this test, so it walks the plain route (`station_armour = false`, a new parameter of
   the file's `walk_to_the_chaser_fight`; the tops-up sisters keep their exact call). Fixes: the
   d1 route economy (finish_fights + level_fights — the plain route's 9 reactive fights had traded
   75.6 with the ring/gallery pair from the ground), the flags, the window gun, the doorway meet:
   the route's fights left the Delta DRY (clip 0 spare 0 at the window — the side taps and the
   clear stands were bulletless), so the window cycles to a loaded gun (`switch_to_armed`, gated
   off the frag), and she parks outside the re-shut leaves at 4.67 (state 0x7c, zero landed stars)
   — Bond steps to the west blast door (the INTO_THE_SHELTER mid leg) and she reaches the 2.0
   punch band [M 0x1789c0:26-40]: five 0xaa point-blank stars at exactly **1.2 = 1.5 [D record
   0x12] × 2.0 TooClose [M 0x139fac-0x139fc8] × 0.4 [M 0x1d4424's Operative row]**, her swing
   state 0xaa at 2.17 u, every landed hit inside the pinned value set, Bond alive at 35.4. The
   test's own live-check branch printed: no 20.0 punch landed in this stream's window (her
   point-blank star bursts only) — the punch's 20.0 chain (50 [D record 99] × 0.4 [M]) stays
   pinned by the test's value asserts whenever she swings.

5. **The d2/d3 sisters keep their measured ignores** (untouched this session; the flags set in
   `_at()` reach them but they are `#[ignore]`d and were not run).

## Runs this session (each WHOLE, `--test-threads=1`, NF_ISO set)

| binary | result |
|---|---|
| walkreal_07000016_live | **14 passed / 0 failed / 2 ignored** (77.03 s) — both target tests green in the same run; the d2/d3 sisters ignored |
| the two red tests individually, iterated | the defeat walk's measured progression: 10959 death (the frag self-blast) → 16107 (armour taken, the gallery bleed) → **ok 24.4**; the punch test: 8519 → 6482 (the route economy) → the dry-gun window → **ok 35.4** |

**Not run this session** (the sandbox clock): the full four-crate suite, the other walkreal
binaries, `nf-game`. The walker.rs change is strictly opt-in (`no_grenade_throws` defaults false;
the only touched paths are inside `if self.no_grenade_throws`), so no other walk's timeline moves
by construction — but per the merge rules the next session should run every `walkreal_*_live`
binary and the four-crate suite on this tree before merging.

## What's left

1. Every `walkreal_*_live` binary + the fast full suite (tools/test/suite.py) + `nf-game` on this
   tree.
2. The d2/d3 sisters' ignores — re-audit their budgets once the above lands (the defeat walk's
   #596/#648 armour levers and the side taps are theirs to reuse; the punch pin's no-armour rule
   is NOT).
3. The 001 d3 00agent walk (guard-radii-finish's item 5, unchanged).

## Live checks for the main agent

1. On 07000016 d1, at the chaser phase: whether the real game's Kiko, once the shelter door
   re-shuts, parks outside at ~4.7 u in state 0x7c (the port's measured stall) or paths in — the
   port's two streams disagree (the 8519-shaped one slipped in, this one parks), and the punch
   pin's window depends on it.
2. Whether the game's player-thrown frag records its blast with no thrower attribution (the
   port's `owner: usize::MAX` reading of `Bullet_init`'s class-3 arm) — the ledger's "world" hits
   for a self-thrown frag hinge on it.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures
anywhere I can read (no banner, no status line, no API surface exposed to the worker).

## ntfy failed

All three fallbacks failed, then one extra curl retry:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "guard-radii-16-chasers: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20000 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'guard-radii-16-chasers: done', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "guard-radii-16-chasers: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output within the tool's 30 s cap; killed by the tool timeout.
4. Retry: `curl -sS -m 15 …` → `curl: (28) Connection timed out after 15000 milliseconds`, exit 28.

(The queue finish step itself succeeded: `finish.sh` printed `OK done 20-guard-radii-16-chasers`.)

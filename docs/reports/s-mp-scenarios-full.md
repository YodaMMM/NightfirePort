# s-mp-scenarios-full: every multiplayer scenario played to its end, host + joiner windows

No change here moves the shared random stream (no draw is added, removed or reordered). The Top
Agent fix changes the match state hash for Top Agent matches only (the slot scores are hashed);
no pinned digest moved (see Tests).

Two real `nf-view` windows on one PC (host: Host Online, joiner: Join Online at 127.0.0.1, UDP
28100-28119), LAN only, silent, a throwaway config dir per window, Skyrail, 2 bots switched on in
the lobby. Each scenario was picked, its limit and duration set through the real one-screen lobby
(menu-script presses), and played until its end condition with scripted input: the new
`NF_NET_AUTOOBJ` steer (both humans head for the scenario objective, or hunt and fire), the bots'
own objective play, and `NF_MP_PLACE_OBJ` to start a human beside an object where a walk across
the arena would stall. Both windows logged an `[mp-obj]` line every tick the match state, a
latch, a score, a status word or any scenario object changed, and the board each window opened
(`[mp-debrief]`).

In all 20 runs the host's and the joiner's `[mp-obj]` lines are identical, both windows opened the
same debriefing board (same board hash), the host logged the end handover complete, and no run
printed a desync, a resync or a board mismatch.

Two results differed from the game. Both are fixed here with tests.

## Fixes

1. **A bot that won a free-for-all match had no name on the result line** (" wins the match.",
   and the in-match "Game Over" status the same). The boot names the bot records:
   `sprintf("%s %d", "Bot", i - 3)` into records 4..7 [M bootup 0x1c7e78:50-61; D "%s %d"
   @0x30ce18, "Bot" @0x30ce20]; `Menu_PrepareBots` 0x1f94a0 carries the name with the record it
   compacts; the debriefing prints `MPSettings + top*0x30` into "%s wins the match." [M
   P_MPDEBRIEFING 0x212198:227-228] and `MP_SortOutWhoWon` prints the same record [M
   0x1848d0:73-76]. The port's bot records had an empty name. Now `create_bots` names an unnamed
   bot record "Bot i+1" (bot i is the game's record 4+i). Live: run dm showed `Wins("")`, run dm2
   (fixed build) `Wins("Bot 1")` in both windows; the Assassination run's winner is "Bot 1" too.
2. **Top Agent's points column read the lives lost, not the lives left.** `MP_Init` seeds every
   slot's score with the lives limit, `(float)(int)MPSettings+0x19C` [M 0x183438:56-62];
   `MP_PlayerKilled` takes 1.0 off per death [M 0x188f70:96-98]; `Menu_GetMPScore` 0x203af8
   returns `(int)+0x18` for 0x10, which the debriefing sorts and prints. The port started the
   score at 0, so with 1 life a dead player showed -1 and a survivor 0 (run topagent). Now
   `MpMatch::init` seeds the copy for 0x10: dead 0, survivor 1. The out-count and the winner
   pick read deaths, not this word, so the end itself was already right.

## The table

Runs, logs and captures: `NightfirePort-work/audits/mp-scenarios-full/<run>/{host,join}/` (not in
the repo). f = the match tick both windows logged. "Same" = the joiner's lines equal the host's.
Runs before the rebase onto 847b67744 (team, ctf, uplink, demo, prot, esp, ge, tkoth) had the
since-fixed lobby bot side (both bots MI6); team2, ctf3, ctf4 and dm2 ran after it (bots
Phoenix). The rules checked do not depend on which side the bots play.

| Scenario | Rule | Game | Host | Joiner | ok |
|---|---|---|---|---|---|
| Arena 0x1 (dm limit 2, dm2 limit 1) | kill = +1 personal; the frag latch on the top slot score; one slot at the limit wins; result "%s wins" | [M 0x188f70:167-184, 0x184268:149-184, 0x1848d0:55-86, 0x212198:221-229] | dm: bot slot 3 two kills, state 1 at f6267, 3/4/5 next ticks, board `Wins("")`; dm2: `Wins("Bot 1")` at f855 | same | n → y (fix 1) |
| Team Arena 0x20000002 (team, team2 limit 2) | an enemy kill adds 1 to the killer's team; team score latch; the result compares the team scores (team 0 < team 1 = MI6) | [M 0x188f70:246-274, 0x212198:231-245] | team: teams [0, 2] at f4871, MI6 (0x1c8); team2: bot two kills, Phoenix (0x1c7) | same | y |
| Capture the Flag 0x20000004 (ctf limit 1; ctf2, ctf3, ctf4 limit 3, 2 min) | enemy touch takes (status bit 1); carried to the carrier's own base: team +1, carrier +1, flag home, bit clears; a carrier's death drops it (state 2); a team-mate touch returns it at once; time up ends through state 2 | [M 0x1861a8:55-112, 170-263; 0x188f70:276-303; 0x184bd0:42-49] | ctf: bot takes flag 0 f2797, captures f6132, team 1 = 1, end; ctf4: joiner kills the carrying host f51 (flag 0 state 2, bit cleared), joiner returns it f644; ctf2/3/4: time up at clock 120.01, state 2 → 5; results MI6 / Draw / Draw by the team scores | same | y |
| Uplink 0x60000008 (uplink limit 3, uplink2 limit 5) | a neutral terminal goes to the toucher (team 1 wins a same-tick tie); 0.2/s drip to the controller from the next tick; the team score is the sum of its members' (int) scores; an enemy touch flips an owned terminal only while no owner touches it | [M 0x186d98:20-114] | uplink: host claims f1144, +0.00333/tick from f1145, team 0 = 3 at f2045, end; uplink2: both humans start on one terminal, team 1 takes it f3, 17 flips, team 1 = 5 at f2052, end MI6 | same | y |
| Top Agent 0x10 (lives 1) | deaths are lives, kills score nothing; all humans out ends it; the unique fewest-deaths survivor wins, a tie is a draw; state 3 waits 5 s; points = lives left | [M 0x183438:60-61, 0x188f70:92-100, 0x184268:85-135, 0x1848d0:21-54, 0x184bd0:54-61] | both humans out at f1184, two bots tie on 0 deaths → Draw, f1185 → f1485 (300 ticks) → 4; points showed -1 | same | n → y (fix 2) |
| Demolition 0x20000040 (demo 1 min, demo2 3 min, demo3 1 min; limit 1) | hits lower the site HP and record the hitter; at `limit <= clock` state 6, the next update gives the defenders (team 0) +1 and marks the site done; restart after 5 s (clock 0, site 2000); the frag latch then ends it | [M 0x1870a0:72-124, 0x184268:50-60, 0x184bd0:79-92] | demo: bot slot 3 hits 2000 → 1990; state 6 at f3600, team 0 = 1 at f3601, restart f3900, end f3901, Phoenix | same | y |
| Protection 0x20000080 (prot, 1 min, limit 1) | the same with team 1 defending | [M as above, `param_3`] | state 6 f3600, team 1 = 1 f3601, restart f3900, end f3901, MI6 | same | y |
| Industrial Espionage 0x20000100 (esp limit 1) | any player takes the free blueprint (status bit 2); the carrier reaching the own team's base: team +1, carrier +1, the blueprint re-picked free | [M 0x1882d8:45-80, 0x188970:24-45] | bot slot 2 takes it f1488, reaches its base f3899: team 1 = 1, slot 2 = 1, blueprint free, end MI6 | same | y |
| GoldenEye Strike 0x20000200 (ge limit 1) | any player carries either item (bits 4/8); both items on one team launch the strike: each carrier +1, a live enemy target; on landing the target dies with no credit and the target's opposite team gets +1, the items reset | [M 0x187568:96-179, 231-253, 356-384] | joiner takes item 1 f725, bot slot 3 item 0 f1516: carriers 1.0 each, target slot 0; lands f1815 (299-tick flight): slot 0 deaths 1, team 1 = 1, items free, end MI6 | same | y |
| Assassination 0x400 (limit 3) | assassin kills target +5 (no reset); another killer of the target scores nothing; the assassin's death resets: the new assassin is its killer, the target a random other | [M 0x188f70:224-243, 0x18aa70] | start 0/3; f5376 slot 2 kills the target: nothing; f5473 slot 2 kills assassin 0 → assassin 2, target 3; f6482 assassin kills target: 5.0, end, "Bot 1" wins | same | y |
| King of the Hill 0x40000800 (limit 2) | every live player on the hill drips 0.2/s personally (status bit 0x10); one slot at the limit wins | [M 0x186ae0:78-121, 0x1848d0:55-86] | both humans on the hill, slot 1 reaches 2.01 first (slot 0 1.95), end, "Player 2" wins | same | y |
| Team King of the Hill 0x60001000 (limit 2) | the drip as above; the scoring player's team = sum of the team's (int) scores; the team result | [M 0x186ae0:118-151] | team 1 = 1 + 1 at f3318 (bots 1.98 and 1.01), end MI6 | same | y |

## Tests

- New `crates/nf-view/tests/mp_scenarios_full.rs` (headless, no disc, 3 tests): a bot winner is
  named on the result line and the end status; a human winner keeps the codename; Top Agent's
  slot scores start at the lives limit and the board's points are the lives left. With the name
  fill disabled the first test fails; with the lives seed disabled the third fails; both pass on
  the branch.
- `crates/nf-game/tests/mp_match_rules.rs`: `top_agent_lives_round_end_and_no_kill_score` pinned
  the lives copy from 0 (-1 after a death, 0 for a killer); it now pins the game's seeded copy
  (1 and 2 with 2 lives) [M 0x183438:60-61].
- `crates/nf-view/tests/mp_scenarios_live.rs`: `top_agent_online` pinned the killer's score at
  0.0 ("kills do not score in 0x10"); with one life the untouched lives copy is 1.0 [M
  0x183438:60-61]. Re-pinned; the binary passes (19).
- nf-game `mp_match_rules`, `mp_debriefing`, `mp_hud_pane`, `mp_lockstep` and the lib's `mp::`
  unit tests (84): all pass.
- Every `mp_*` and `net_*` nf-view test binary (60): 216 passed in the first pass. Three binaries
  failed: `mp_scenarios_live` (the pin above, fixed, then green), `net_resync_live` (its three
  tests could not bind their fixed port 28210 in that pass; re-run alone: 3 passed) and
  `net_soak_faults_live` (its fixed port 27980 was held by another checkout's run of the same
  test; re-run alone with the constant moved to the free port 28150 in the working copy only:
  2 passed, 511 s).

## Test aids added (env-gated, log and input only)

- `NF_MP_OBJLOG=1` (scene.rs): the `[mp-obj]` change line; (lib.rs) the `[mp-debrief]` board line.
- `NF_NET_AUTOOBJ="a-b"` (lib.rs): the objective steer, the way the bots' scan picks a target (CTF:
  a carried own flag's carrier, a dropped own flag, the enemy flag, carrying → the own base;
  Uplink: a terminal the team does not own; KOTH: the hill; Demolition/Protection: the site;
  Espionage: the blueprint or the own base; GoldenEye: an item the team does not hold; the kill
  scenarios: the nearest live opponent, fired at when close).
- `NF_MP_PLACE_OBJ="<slot>:<obj>:dx,dz"` (lib.rs): a body placed beside a scenario object at the
  start, on every machine.

## Unproven

- A Demolition/Protection site destroyed (the attackers' +1, the hitter's +1 or the own-goal -1,
  the state-6 round from the destruction) was not reached live: the bots hit the 2000-HP site for
  4 or 5 a shot, and the scripted rocket shots from the placed joiner did not fire in these runs
  (clip still 4/4 at tick 900). The headless pins (`mp_match_rules`, `mp_site_destruction_blast_live`)
  cover that arm.
- The flag's 30 s return without a touch, a dropped GoldenEye item or blueprint (carrier killed),
  and the GoldenEye strike with no live enemy were not played live.
- KOTH and Uplink scores keep dripping on the port's states 3 to 5 (a few ticks before the
  board is taken). Whether the game's object updates still run in those states was not traced.
- The capture-reward weapon (`MPSettings+0x1BC`) in `MP_BluePrintReachedBase` and the flag
  capture is off by default and was not exercised.
- A bot record renamed by `Menu_PrepareBots`' compaction (a later bot enabled with an earlier one
  off keeps its first record's name, so the game can show e.g. "Bot 2" for the first bot) is not
  modelled: the port names bot i "Bot i+1".
- Nothing here was compared against the game running live.

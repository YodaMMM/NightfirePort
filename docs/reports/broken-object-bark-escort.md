# broken-object-bark-escort — report (2026-11-05)

**DONE** — the question is answered (a stream shift, not a talk hold), both named reds are
green on the shifted stream with zero draws touched, and walkreal 01 is green whole.
Branch `s/broken-object-bark-escort` (cut from origin/master 82405afdb; 071294809
cherry-picked as d9095e28b).

## The verdict: a stream shift from the Rand(4) draws

The bark commit is faithful and the bark does NOT hold the escort:

- **Game side** [M]: `Drone_BrokenObject` 0x138588 (decompile `work/decomp/ACTION.ELF/
  00138588_…c`) gates on Drone_bDisableSystem == 0, GameState+0xc == 0x7000001, the
  hitdata's +0x58 bullet (obj+0xff == 5) of the player's system, then walks the drone list:
  type +0xc5 == 0x0c, health +0xac <= 0 aborts the walk, and per barking drone ONE
  `DroneFunc_ChooseSFX(0x40, 0x43)` (= 0x40 + Rand(4)) and `NDrone2_SpeechSFX(&dcvars, line,
  0, 0, 0)`. `NDrone2_SpeechSFX` 0x141b00 (its decompile) only applies the repeat cooldowns,
  the facial script, and `NDrone2_PlaySFX` — writing the handle to **drone+0x4d0**. **No
  state-machine write, no walk stop** — the game's escort keeps walking while he barks.
- **Port side** [M]: the cherry-pick's `speech()` (drone.rs) matches — a voice handle +
  `last_voice`, consumed post-walk; nothing reads a pending talk to pause the lead.
- **The trace** [L] (the bark-trace eprintln added to the escort test's stall dump): the
  parked lead sat at (3.1, 6.3, 6.8) in **AllyLeadWait 0x20 with `talks 0`, `voice None`**,
  at his own route node (goal (2.91, 5.74, 7.57)) — two bark lines had already landed and
  FINISHED (last_voice 861, then 3390). The park is the game's own wait law
  [M NDrone2_DSTATE_AllyLeadWait 0x15eda8 — Bond more than 9 route-units behind parks him;
  the release band pd > −7.0 && (pd != 0.0 || dist2d < 5.0)]. **Bond was the stuck one**: the
  bark's Rand(4) draws shifted the shared stream, the follow steering walked him west out of
  the hall, and he wedged against the meet hall's west wall segment (x≈1.93, z 11.1-13.1
  [D the disc's collision cels — the walkreal 01 probe's documented wall]) with door #1
  (7.9, 5.3, 11.1) auto-closed between the pair [M the auto-close 0x13660c-0x136700].

## The fix (test-side recovery only — no draw removed, reordered or added)

1. `escort_follow_07000001_live.rs`: in the stall ladder, the ALLY_LEAD_WAIT branch now runs
   (a) the **west-wall lane walk** — the proven walkreal 01 recovery, the hall's north
   opening lane [D nav path 0 nodes 8 (−0.39,14.28) → 23 (3.39,14.69) → 7 (6.88,14.62)] —
   when the west wall is between Bond and the contact, then (b) the **E-open of the closed
   door** between the pair (the midpoint rule, the door-react tap loop [M Player_Activate
   0x1a5170 case 0x1c → Door_Activate; the use edge 0x1a5ea8-0x1a5ee8 + 0x17f084-0x17f090]).
   The E-open was hoisted INTO the wait branch because the lead's AllyLeadWait state
   shadowed the closed-door branch on every stall round (a latent gap, exposed by the shift).
   A bark-trace eprintln stays in the stall dump (talks/voice/script/goal/wait-node/doors).
2. `fix_npc_doors_live.rs`: the same two recoveries before the blind "close on the contact".

## Verification (each binary whole, --test-threads=1, slim ISO)

- `escort_follow_07000001_live` **1 passed / 0 failed** (10.65 s) — the contact led his full
  route and settled at (−28.3, 6.3, 14.3) past the shoji doors; the walking-speed distance
  bands evaluated.
- `fix_npc_doors_live` **1 passed / 0 failed** (7.94 s) — the escort led past the interior
  shoji doors.
- `walkreal_07000001_live` **8 passed / 0 failed** (60.21 s) — the required walkreal 01.
- `broken_object_bark_live` **4 passed / 0 failed** (0.27 s) — the cherry-pick's own pins
  (the 07000001 positive, another-level silent, explosion silent, the abort/type gates).
- `crates/nf-game` lib **678 passed / 0 failed / 5 ignored**.
- The rule-10 neighbours (the cherry-pick touched breakable.rs + drone.rs): 
  `mp_death_hands_online` 2/0, `escort_cover_live` 3/0, `escort_hide_findrun_live` 3/0,
  `fix_guard_misc_live` 6/0, `ally_stuck_sweep_live` 1/0, `ai_traces_live` 4/0,
  `campaign_flow_live` 5/0 (294.98 s).
- The full suite did not fit the window; the named binaries plus the 01-loading set above
  cover the cherry-pick's blast radius — **the bark's list walk is level-gated to
  0x07000001** [M the cherry-pick's `level == 0x0700_0001` gate], so every other level's
  RNG stream is unchanged and its walks are unaffected (the cherry-pick's own
  another-level-silent test pins exactly that).

## What's left

Nothing for this task. The two latent notes for the next 01 work: the closed-door E-open
recovery is still shadowed in the OTHER stall branches of the escort ladder (only the
ALLY_LEAD_WAIT branch needed it today); and the walkreal 01 binary carries its own
west-wall recovery that already covered this wedge for the plain walks.

## Usage

not visible to me (no token, credit or context-window figures are exposed to this session;
nothing in the sandbox reports them).

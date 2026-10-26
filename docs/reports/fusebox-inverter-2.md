# fusebox-inverter-2 — the type-0xF9 "inverters" were sound triggers

Task: `s/fusebox-inverter` built `InverterTrigger_Update` (0x1b30a8) triggers from type-0xF9
placements on every level and went red with master on four live tests. This task resolves the
split so all four are green again **and** the 07000014 inverter test stays green — by porting the
game's actual create gate and update rule.

## The finding: 0xF9 is `SoundTrigger_Create`; the inverter is 0x41

- `parsemap_create_dynamic_objects` (0x1d04f0) case **0xf9** calls `SoundTrigger_Create`
  (0x1b2628, class 0x40) [M 0x1d04f0:504-510]; case **0x41** calls `InverterTrigger_Create`
  (0x1b3040, class 0x53; p0 → obj+0xF4, p1 → obj+0xF6) [M 0x1d04f0:265-268, 0x1b3040:13-18].
  Case 0x41 has no further gate (only the dispatcher's create byte); case 0xf9's gates live inside
  `SoundTrigger_Create`: p2 (sound id) != 0 and != 0xffff, p0 (ON gate) != 0, then the difficulty
  gate on p6 — skip when `p6 != 0 && p6 < GameState+0x40`, and 5/6/7 require difficulty 1/2/3
  [M 0x1b2628:14-40]. The port's `sound::SoundTriggers::create` already implements exactly this.
- `SoundTrigger_Update` (0x1b2790) **stores no `switch_channels` at all** — it plays/stops a sound
  gated on p0 (ON), p4 (OFF), one-shot via p1 == 1, global-voice slot via p5 [M 0x1b2790].
- `InverterTrigger_Update` (0x1b30a8) is
  `if (switch_channels[obj->in] != 0 && *(short*)(obj+0xF6) != 0) switch_channels[obj->out] = 0;`
  — the second gate is the **raw OUT FIELD** (out channel number != 0), not the channel's value
  [M 0x1b30a8:8-11]. Master's `worldobj.rs` `Inverter::tick` already matches this exactly, and
  master already builds the inverters from the 0x41 placements.

So the s/fusebox-inverter branch attached the inverter behaviour to the wrong placement kind and
misread the sound triggers' params: p1 is the **one-shot flag**, not an output channel. Every 0xF9
row the branch converted had p1 = 1, so every one of them became an inverter whose output was
**channel 1**.

## What the 0xF9 placements are on the three levels ([D] the disc decode)

- **0700000c** — five 0xF9 rows ("Sound Trigger"): #456 (gate 0x55, id 0x5ed), #541 (0x65, 0x14d),
  #550 (0x43, 0x5ec), #562 (0x8, 0x5fe), #563 (0x66, 0x14a) — all p1 = 1. **No 0x41 rows.** The
  branch built five inverters, all writing ch1. The yard sniper (#309 at (43.77, 21.52, −205.13))
  waits on **ch1** (drone param 3 = 0x1 [D]) — the two sniper tests raise his channel and the
  inverters cleared it, so he never activated and took no damage.
- **07000041** — two 0xF9 rows: #89/#90 (gate **0x1**, one-shot, ids 0x10c/0x282 — the spacewalk
  voice-over [D]). The branch built two inverters with in = 1 and out = 1: while the approach chain
  channel was on it was cleared the same tick — the searchlight (head ON channel 1 [D PL#76])
  froze and the wrapper-end chain never ran. Both tests red.
- **07000014** — 18 0xF9 rows (voice lines; the pair at the generator room gates on the
  fusebox-cut channels 0x87/0x88 with sound 0xf1 [D]) and **four 0x41 rows** — the game's real
  inverters: (0x0e → 0xb8), (0x85 → 0x6f), (0x85 → 0x22), (0x85 → 0xd1) [D]. Two of those outputs
  (0x22, 0xd1) are themselves the ON gates of sound-0xf1 voice rows: the inverters hold the lines'
  gates low while ch0x85 is up, so the lines play when it drops. Nothing on this level inverts
  ch1, and nothing inverts ch0x87 — the previous test's "07000014 #390 writes ch 1 from ch 0x87"
  is wrong on both ends (0xF9 creates a sound trigger [M case 0xf9], and p1 = 1 is the one-shot
  flag [M 0x1b2790]).

The fusebox cut's downstream events on 07000014 never needed an inverter: the completion chain is
the sabotage arms 0x8C (AND #391 [0x87, 0x88]) + Touch #456 (out 0x0C) → SEQ_AND #445 [0x8C, 0x0C]
→ 0x09 → ScriptPlayer #407, whose script 0x06000848 writes [0x4A,1] + [0x11,1]
[docs/levels/07000014.md §2, [D] decode]. That chain is trigger/script machinery master already
ports.

## The fix

- `crates/nf-game/src/triggers.rs`: removed the branch's 0xF9 → INVERTER trigger arm, the calc
  arm, and the `kind::INVERTER` constant. The trigger scan is back to the game's set
  (0xEA/0xEB/0xEC/0xED/0xEE/0xEF/0xE8/0xF4/0xDC); 0xF9 belongs to `sound::SoundTriggers`
  (create gates already exact [M 0x1b2628]) and the inverter to `worldobj::WorldObjects`
  (create [M 0x1b3040], tick [M 0x1b30a8]).
- `crates/nf-game/tests/07000014_inverter_trigger_live.rs`: rewritten to the game's behaviour, all
  assertions evidence-tagged — 18 0xF9 rows and 4 0x41 rows on the main map [D]; no trigger built
  from any 0xF9 row [M case 0xf9]; the four inverter pairs [D] built through
  `WorldObjects::create` in placement order; the tick rule on the 0x0e inverter (out held low
  while in on, re-raised out cleared again, input untouched) [M 0x1b30a8]; the sound-0xf1 voice
  rows [D]; and the regression pin — with ch0x87 and ch1 raised, the voice line arms and **ch1
  stays on** (the old cut cleared it) [M 0x1b2790 stores no switch_channels].
- `crates/nf-formats/examples/dump_f9_41_placements.rs`: the probe that produced the [D] dump
  (0x41/0xF9 rows with params for 14/0c/41).

No test was weakened: the four live tests are byte-identical to master's, and the 14 test's
replacement asserts strictly more than the old one did about the game's data.

## Verification

- The four previously-red tests green on this branch:
  `fix_guard_misc_live::probe_sniper_direct_bullet`, `::the_0c_sniper_takes_damage_when_shot`
  (6 passed in the binary), `interactables_07000041_live::the_searchlight_family_sweeps_as_one_light`,
  `::the_wrapper_ends_into_the_space_station_after_the_chain` (14 passed, 2 pre-existing ignores).
- `07000014_inverter_trigger_live` green (the rewritten pin).
- Full suite (fast runner, 818 binaries, NF_SILENT, slim ISO): see the total line below.
- Crates run by hand before the push: `nf-game` (the 14 test), `nf-view` (the four live tests).
- The disc evidence comes from `dump_f9_41_placements` on `Nightfire-onfoot.iso`; the census rows
  agree with `census0c.txt` (kind 0xf9: 5) and `docs/levels/07000014.md` §1 (0x41: 4, 0xf9: 18).

Suite total: the fast runner never completed — the sandbox's shell died ~20 minutes into the
per-binary pass (every tool call failed until the session ended). Its last output was
`build ok: 818 test binaries in 998 s` then `CRASH net_join_live (exit -9)` (a SIGKILL 4 s into
`late_join_replay_mismatch_is_refused_with_a_clear_reason` — an external kill, not a test failure;
net_join is untouched by this branch). The binaries this task touched all ran green by hand:
`nf-game --test 07000014_inverter_trigger_live`, and `nf-view --test fix_guard_misc_live --
--test interactables_07000041_live` (exact commands in the WORKLOG). The next session should run
`tools/test/suite.py` once on the merged tree for the formal total line.
One extra check landed after the report draft: the game's dynamic-object dispatcher skips any
placement with `flags & 0xa000 != 0` [M parsemap_block_map_data_dynamic 0x1d1b68], and the port's
`sound.rs` honours it while `worldobj.rs::create` does not — a disc-wide scan
(`dump_a000_skipped_rows`) found **zero** affected rows (kinds 0x41/0xd2/0x3b/0xe7/0x71/0xfb/0xf9
with the skip bits) on any main map, so it is behaviourally inert on this disc; left as noted.

## Live checks for the main agent

`switch_channels` is u8[256] at ACTION.ELF **0x0026fc90** (`switch_channels_prev` +0x100,
`switch_channels_hold` +0x200, `switch_channels_time` +0x500 [work/action-symtab.tsv]). On a FRESH
boot (a save-state resume skips the boot path):

1. 07000041: watch byte 0x0026fc90 + 0x1 (ch1, the approach chain channel) while the chain plays.
   Expected: it rises with the chain and is **never forced low** while the voice rows' gate is on —
   the spacewalk voice-over plays and the searchlight sweeps at the same time. A write of 0 to ch1
   while ch1 is 1 would contradict the fix.
2. 0700000c: walk into the yard; expected: the rooftop sniper (drone #309's placement, gate ch1)
   activates and his shots hit — ch1 is not held low by anything.
3. 07000014: cut the two generator-room fuseboxes; expected with a watch on 0x0026fc90+0x87/0x88
   (rise), +0x8C, +0x0C, +0x09 (the chain) and the voice rows' sound 0xf1 playing — and **no write
   to +0x01** at any point of the cut.

## What's left

- Nothing known for this task's scope. The two pre-existing `ignored` gaps in
  interactables_07000041_live (the laser-mine walk-in contact, the stealth-wall use-key crawl) are
  documented in their ignore strings and are not touched here.
- The port's `Inverter`/`SoundTrigger` creation runs on the main map only (as the game's dispatcher
  caller does); no other level-conditional gate exists for 0x41/0xF9 in the decompile.

## Usage

not visible to me — this session's environment exposes no token, context-window or cost figures to
copy.

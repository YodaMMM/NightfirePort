## Your task: `real-health-00agent-08-bleed-9`, 07000008's 00 Agent whole-mission real-damage walk wins the Hind fight

Branch from current origin/master; if origin/s/real-health-00agent-08-bleed-8 is not on master, merge it in first. Read docs/reports/real-health-00agent-08-bleed-8.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. The pre-fight bills re-roll when the timeline is touched, so prefer item 2 first: the 6th launch window from the strip hold, which leaves the pre-fight timeline alone. Then a health source that does not perturb the pre-fight timeline (item 1). Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary and interactables_07000008_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-08-bleed-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-15T11:18Z

Main 11-15 (on fc005fd7b): measured dead, do not repeat - (1) re-roll sweeps: 112 values (pre-boarding waits 1-1100, idles injected at t2250-27100 and inside the fight) - none beat five rockets / t33393, most die before the fight; idles inside the fight are byte-neutral (the fight is on rails while Bond stands at the anchor); (2) the gap after rocket 3 (t1353-5223): eye_sees is the binding gate (the anchor cannot see the machine), cabin spots 0.5-2 u away can; stepping to them after rocket 3 (radius 0.75-3.0) dies sooner (t30273-32021) with no extra launch; (3) carried guns: 0x0a dmg 2.0 (32 rounds), 0x02 3.5 (2), the 0x14 launcher lands only its 2.5 contact on the Hind (5.0 over a whole fight, no blast damage) - no gun closes the ~108 gap. Untested: the Hind missile fuse vs the game (bleed-8 live check list).

Main 11-15 (2): NEW measured fact - rocket 2 (launched fight t946 with the machine nearly overhead: copter (113.8, 28.8, -184.9), dxz 6.0, dy 15.5) detonates on the cabin roof at (112.2, 17.7, -179.9), 4.7 u above Bond (owner -1 kind 44) and takes the ledge armour 50 + 25.05 health - the costliest hit of the fight. The 3-u slit cannot see it (the aim line is clear even at 12 u - the guided round turns up into the roof). An elevation gate (no launch while (cp.y - b.y)/dxz >= 1.0..2.5) skips it: 4 rockets 301.5, Bond keeps health into fight t3400 but then takes three machine missiles (-7.09/-10.38/-6.24, kind 98 r4 15.0) + nine cannon chips, and the t5537 launch from 0.3 u east of the old spot self-blasts on the south frame (113.6, 13.7, -181.1) at 2.1 u - death t32801. Layered on it, all measured dead: the corridor gates (1 rocket), short 3-6 u gun-stand-in clearance rays (2 rockets), a tighter anchor walk 0.15-0.6 u (0 rockets), 24 in-fight idles (all <= t32801).

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

Main 11-15 (3): rocket 2's blast is a HIT on the machine (+87-101 damage) whose contact point sits 4.7-5.7 u above Bond - the machine passes right over the roof (cp y 29.3 at dxz 0.7); holding the round straight 5-40 ticks / until 3-15 u out does not avoid it (it hits the machine body anyway). The 10-u launch gate measures to the machine centre; l >= 17 skips that launch (same outcome as the elevation gate). The t5537 self-blast on the south frame (1.2 u below the eye) is cut by a 4-u slit cast from 0.25-0.3 u below the eye (the lowered muzzle): l >= 17 + that cast = 3 rockets 301.5, NO self-blast, death t33202 to the machine missiles + cannon chips; in-fight idles on it are neutral or worse. So: self-blasts are solved but the launch rate is not - 3 launches in the whole 6000-tick fight once the close one is skipped.

Main 11-15 (4): a lever from the copter state machine (copter.rs case 2, [M 0x17dbb8:200-242]): the attack BAILS to state 1 when the damage the machine takes during it (rec+0x6c attack_acc, every hit and blast) exceeds 100, then the spot gate waits 240/DiffMod ticks before the next attack. Rockets that land 100.2-101.0 end the attack they hit; rocket 2 (87.1, a blast-edge hit) and rocket 3 (99.6) do NOT - and the cannon chips that cost 18.0 (9 x -2.0, fight t1654-3849) come from the attack rocket 3 left running. Topping a short rocket up past 100 inside the same attack (a few 0x0a rounds at 2.0, or holding the launch for a centre hit) should end those attacks early. Also measured dead this round: crouching for the overhead shots (breaks rocket 1, death t28217-28933).

Main 11-15 (5): with a 4-u slit cast from three muzzle points 0.2 u below the eye (eye, eye +/- 0.25 right), the south-west launch ban can be LIFTED: l >= 17-22 + that cast + no SW ban = 4 rockets 401.7, NO self-blast, death t33138 (stable across l 17-22). Rocket 3 now lands at the SW machine (84.7, 22.3, -196.3) from the anchor. The bill that run: three machine missiles -23.7 (fight t3439-3873), then 14 cannon chips x -2.0 (t4218-5885); after rocket 4 (t5236, clip 0) no reload/launch before the death. Offsets 0.15/0.18 let a self-blast back in (5 launches, t32808); 16 waits stacked on it: best t34832 (5 launches, 402.3, one self-blast). Not shipped - no gain over master.

Main 11-15 (6): THE WALK PASSES on master 69df8ef14 - the 0.22-u-low 3-point launch cast replacing the SW ban + cannon dodges from rocket 3 + a finisher after rocket 6 (aim within 0.2, l >= 10): seven rockets, the Hind down. Nothing left to do on this task.

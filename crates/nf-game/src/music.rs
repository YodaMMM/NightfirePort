//! The EE music logic (ACTION.ELF): the event board (`Music_Event` 0x1de328), `UpdateMusicalEvents` (0x114e70), the
//! 07000005 updater `UpdateMFX_01CastleA` (0x112d78), `PickRandomCombat` / `PickRandomStealth`, and the attack gate
//! `NDrone2_SeenAndAttacking` (0x145150) with its `MusicVars`. Spec: docs/systems/music.md.
//!
//! Commands queued here reach the IOP player with the next frame's batch, as on the PS2.

use nf_formats::music::{MapMusic, MusicVars};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    /// `SFXStopMusic` + `SFXStartMusic(track, section)`.
    Start { track: i32, section: i32 },
    /// `SFXJumpToMusicMarker(section, instant)`.
    Jump { section: i32, instant: bool },
    /// `SFXFadeDown(fadeMusic)`.
    FadeDown { music: bool },
}

/// The attack gate's state (`MusicVars` 0x2707a0): attackers counted this frame, the latch, the drones counted.
#[derive(Clone, Debug, Default)]
pub struct AttackGate { pub vars: Option<MusicVars>, latch: bool, count: u16, counted: Vec<usize>,
    /// The level's own `MapDroneData` row (the `wire_music` seed) — `GateWrite::Restore` re-seats it
    /// (`UpdateMFX_02MayhewB` 0x112270:57-76's per-tick scan; no row on the table → the game's
    /// default count 2 / floor 8 / distance 25 keeps the window [M 0x112270:72-75]).
    pub table_row: Option<MusicVars>, }

impl AttackGate {
    /// `Drone_InitComms`' per-frame reset: the latch falls when no more than `floor` drones attacked last frame.
    pub fn new_frame(&mut self) {
        let floor = self.vars.map_or(0, |v| v.floor);
        if self.count <= floor { self.latch = false; }
        self.count = 0;
        self.counted.clear();
    }

    /// `NDrone2_SeenAndAttacking(drone)`: called by the drone combat states. `sees`: sees Bond now (`+0x16`);
    /// `since_seen`: frames since it last saw him (`+0x238`); `dist`: to him (`+0x1A0`). True = `Music_Event(2, 5)`.
    pub fn seen_and_attacking(&mut self, drone: usize, sees: bool, since_seen: u32, dist: f32) -> bool {
        let Some(v) = self.vars else { return false };
        let mut recent = since_seen < v.window * 60;
        if dist < v.distance && self.latch { recent = true; }
        if !(recent || sees) || self.counted.contains(&drone) { return false; }
        if self.counted.len() < 100 { self.counted.push(drone); }
        self.count += 1;
        if self.count >= v.count || self.latch || sees { self.latch = true; return true; }
        false
    }
}

/// The music state (`UpdateMusicalEvents` globals).
#[derive(Clone, Debug)]
pub struct Music {
    pub table: Vec<MapMusic>,
    entry: Option<MapMusic>,
    /// `EventList[64]`: one-frame pulses.
    pub events: [u32; 64],
    pub out: Vec<Cmd>,
    nis_playing: bool,
    nis_quitten: i32,
    delay_timer: i32,
    pub last_jump: i32,
    pub last_request: i32,
    pub wait_for_end: bool,
    under_attack: bool,
    count: i32,
    count2: i32,
    bonus: i32,
    which_combat: usize,
    which_stealth: usize,
    temp1: bool,
    temp2: i32,
    temp3: i32,
    temp4: bool,
    /// The EE's `SFXPaused` 0x30c92c — set by the pause page's `SFXPause` (batch op 9) [M
    /// 0x116b70:11-13; `MenuManager_Create` 0x21b908:121-124 pause_sound=1]. While it stands,
    /// `UpdateMusicalEvents` returns before every event but the level-start restart [M 0x114e70:61-64].
    pub sfx_paused: bool,
    /// `SfxNisMusicTrigger` 0x30c8b4 [M 0x115a68:18-20]: set when a sound with id 0x172 (the NIS music
    /// stream) starts — the cut-scene's own music tells the level updater to jump to its NIS section.
    pub nis_trigger: bool,
    /// `MusicTemp1` 0x30c8ec as the CastleC/CastleD updaters use it (0 / 1 / 10 latches).
    c_temp1: i32,
    /// `MusicTemp2` 0x30c8f0 as CastleC uses it (the post-combat latch).
    c_temp2: i32,
    /// `MusicTemp4` 0x30c8f8 as CastleC uses it (the 0x6000073 scene's started latch).
    c_temp4: bool,
    /// `MusicTemp1` 0x30c8ec as `UpdateMFX_HT_Level_Tower2B` 0x114348 uses it (the 0/1/2 alarm machine).
    tower2b_temp1: i32,
    /// `MusicTemp2..6` as `UpdateMFX_HT_Level_EvilBase` 0x114728 uses them: the latches t2 (the
    /// 0x600062f scene), t3 (the 0x6000848 hold), t4 (the 0x6000833 NIS), t6 (the 0x6000848 NIS)
    /// and t1 — the 0x6000833 counter that gives up to `PickRandomCombat(1)` past 0x37.
    evil_base_t1: i32,
    evil_base_t2: i32,
    evil_base_t3: i32,
    evil_base_t4: i32,
    evil_base_t6: i32,
    /// `MusicTemp1` as `UpdateMFX_HT_Level_EvilBaseC` 0x114b38 uses it (the 0/1/2/3 machine).
    evil_base_c_t1: i32,
    /// `MusicWaitForNisSfxTrigger` 0x30c8e4 as `UpdateMFX_01CastleB` 0x113060 uses it: the
    /// 0x6000071 NIS-start latch that silences the alarm pick until the cut-scene's music sound.
    b_wait_nis: i32,
    /// `MusicTemp1..5/9` + `WhichNonAttack_185` 0x30c908 as `UpdateMFX_02MayhewB` 0x112270 uses
    /// them (07000002): the ch15/16 combat latch, the trigger latches, the 300-tick gate and the
    /// stealth round-robin. `MusicTemp7` is written-only in the game [M 0x112270:82] — no field.
    may_b_t1: i32,
    may_b_t2: i32,
    may_b_t3: i32,
    may_b_t4: i32,
    may_b_t5: i32,
    may_b_t9: i32,
    may_b_which: i32,
    /// `MusicTemp1..3/9` + `WhichNonAttack_190` 0x30c90c / `WhichAttack_189` 0x30c910 as
    /// `UpdateMFX_02MayhewC` 0x112730 uses them (07000003).
    may_c_t1: i32,
    may_c_t2: i32,
    may_c_t3: i32,
    may_c_t9: i32,
    may_c_which_calm: i32,
    may_c_which_attack: i32,
    /// `MusicTemp1/2` as `UpdateMFX_02MayhewD` 0x112bf8 uses them (07000004): the first
    /// `SfxNisMusicTrigger`'s latch and the 0x6000082 story-scene latch.
    may_d_t1: i32,
    may_d_t2: i32,
    /// `MusicTemp1/2` as `UpdateMFX_03TowerC` 0x113a20 uses them (0700000b): the 0..6 state
    /// machine and the combat latch.
    tower_c_t1: i32,
    tower_c_t2: i32,
    /// The attack-gate writes the level updaters make straight to `MusicVars` (0x2707a0, the
    /// attackers-needed count) and `DAT_002707a8` (the sight distance) — drained onto
    /// `attack_gate.vars` by `Drones::tick` after `update` (the game's write lands for the NEXT
    /// frame's `NDrone2_SeenAndAttacking` pass: the drone pass precedes 0x114e70 in the frame).
    pub gate_writes: Vec<GateWrite>,
}

/// One direct write the per-track updaters make to the attack gate's vars.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GateWrite {
    /// MayhewB 0x112270:56-76's per-tick `MapDroneData` re-scan (every tick the IOP is in a
    /// section and the level's e[37] is low): the vars back to the level's table row (no row →
    /// the game's default count 2 / floor 8 / distance 25, the window untouched [M 0x112270:72-75]).
    Restore,
    /// `DAT_002707a8 = 1.0` — MayhewB, every tick the level's e[37] holds [M 0x112270:77-79].
    SetDistance(f32),
    /// `MusicVars = 1` — the trigger arms: MayhewB 0x112270:113/:122, MayhewC 0x112730:210/:219.
    SetCount(u16),
}

impl Default for Music {
    fn default() -> Music {
        Music { table: Vec::new(), entry: None, events: [0; 64], out: Vec::new(), nis_playing: false, nis_quitten: 0, delay_timer: 0, last_jump: 0,
            last_request: -1, wait_for_end: false, under_attack: false, count: 0, count2: 0, bonus: 0, which_combat: 0, which_stealth: 0,
            temp1: false, temp2: 0, temp3: 0, temp4: false, sfx_paused: false, nis_trigger: false, c_temp1: 0, c_temp2: 0, c_temp4: false,
            tower2b_temp1: 0, evil_base_t1: 0, evil_base_t2: 0, evil_base_t3: 0, evil_base_t4: 0, evil_base_t6: 0, evil_base_c_t1: 0,
            b_wait_nis: 0, may_b_t1: 0, may_b_t2: 0, may_b_t3: 0, may_b_t4: 0, may_b_t5: 0, may_b_t9: 0, may_b_which: 0,
            may_c_t1: 0, may_c_t2: 0, may_c_t3: 0, may_c_t9: 0, may_c_which_calm: 0, may_c_which_attack: 0, may_d_t1: 0, may_d_t2: 0,
            tower_c_t1: 0, tower_c_t2: 0, gate_writes: Vec::new() }
    }
}

impl Music {
    pub fn new(table: Vec<MapMusic>) -> Music { Music { table, ..Default::default() } }

    /// `Music_Event(i, v)`.
    pub fn event(&mut self, i: usize, v: u32) { if i < 64 { self.events[i] = v; } }

    /// Whether a level track entry is active (the `entry` gate `UpdateMusicalEvents`'s per-frame
    /// logic runs under).
    pub fn has_entry(&self) -> bool { self.entry.is_some() }

    /// The alarm hysteresis the event-2 slot drives (`UpdateMusicalEvents`'s
    /// `e[2] == 2 → bonus = 600, under_attack = true` and its decay — music.rs's port of
    /// 0x114e70): `(under_attack, bonus)`.
    pub fn alarm_state(&self) -> (bool, i32) { (self.under_attack, self.bonus) }

    /// The IOP's op-0x19 pair (`NisPlaying`, `NisQuitten`): `UpdateMusicalEvents` keeps `NisPlaying` while a
    /// script camera runs and counts `NisQuitten` down from 5 after the quit event [M sound.md §6 step 3].
    /// Read by the audio batch to drive `ES_HandleNisPauseSfx` and `ES_StopDucker`.
    pub fn nis_state(&self) -> (bool, bool) { (self.nis_playing, self.nis_quitten > 0) }

    fn in_set(set: &[i32], x: i32) -> bool { set.contains(&x) }

    fn jump(&mut self, s: i32, instant: bool) { self.last_request = s; self.out.push(Cmd::Jump { section: s, instant }); }

    /// `PickRandomCombat(flag)`: round robin over the combat set, unless a combat section is requested or playing.
    fn pick_combat(&mut self, flag: bool) {
        let Some(e) = &self.entry else { return };
        let set = e.combat.clone();
        if set.is_empty() || Self::in_set(&set, self.last_request) || Self::in_set(&set, self.last_jump) { return; }
        let s = set[self.which_combat % set.len()];
        self.which_combat += 1;
        self.jump(s, flag);
    }

    /// `PickRandomStealth`: only coming out of combat (the IOP reports a combat section).
    fn pick_stealth(&mut self) {
        let Some(e) = &self.entry else { return };
        let (st, cb) = (e.stealth.clone(), e.combat.clone());
        if st.is_empty() || Self::in_set(&st, self.last_request) || Self::in_set(&st, self.last_jump) || !Self::in_set(&cb, self.last_jump) { return; }
        let s = st[self.which_stealth % st.len()];
        self.which_stealth += 1;
        self.jump(s, false);
    }

    /// `UpdateMusicalEvents`, once per frame. `iop_section`: the section the IOP reported (`0x2500CC`).
    pub fn update(&mut self, iop_section: i32) {
        let e = self.events;
        if e[1] != 0 && e[0] != 0 && e[0] != 0x0700_0048 {
            if let Some(m) = self.table.iter().find(|m| m.level == e[0] && m.track != 0).cloned() {
                self.nis_playing = false;
                self.nis_quitten = 0;
                self.last_request = -1;
                self.last_jump = m.start;
                self.out.push(Cmd::Start { track: m.track, section: m.start });
                self.wait_for_end = false;
                (self.count, self.count2) = (m.calm_frames, m.alert_frames);
                (self.temp1, self.temp2, self.temp3, self.temp4) = (false, 0, 0, false);
                // The same restart zeroes `MusicTemp1..9` and `MusicWaitForNisSfxTrigger` [M
                // 0x114e70:39-54] — the per-level machines' latches below; the WhichNonAttack/Attack
                // round-robins are NOT in that list and survive a restart [M 0x114e70:40-52].
                (self.tower2b_temp1, self.evil_base_t1, self.evil_base_t2, self.evil_base_t3, self.evil_base_t4, self.evil_base_t6, self.evil_base_c_t1) = (0, 0, 0, 0, 0, 0, 0);
                (self.b_wait_nis, self.may_b_t1, self.may_b_t2, self.may_b_t3, self.may_b_t4, self.may_b_t5, self.may_b_t9) = (0, 0, 0, 0, 0, 0, 0);
                (self.may_c_t1, self.may_c_t2, self.may_c_t3, self.may_c_t9, self.tower_c_t1, self.tower_c_t2) = (0, 0, 0, 0, 0, 0);
                (self.may_d_t1, self.may_d_t2) = (0, 0);
                (self.under_attack, self.which_stealth, self.which_combat, self.bonus) = (false, 0, 0, 0);
                self.delay_timer = 10;
                self.entry = Some(m);
                // The restart clears `SfxNisMusicTrigger` too [M 0x114e70:38] — a stale cut-scene
                // trigger from the previous load must not fire the new level's NIS arms.
                self.nis_trigger = false;
            }
        }
        // `UpdateMusicalEvents` 0x114e70:61-64: while the EE's `SFXPaused` stands (the pause page's
        // `SFXPause` batch op 9 [M 0x116b70:11-13; MenuManager_Create 0x21b908:121-124 pause_sound=1])
        // every event and the whole per-frame logic below is frozen — the events persist. The
        // level-start restart above is the one path that runs before this gate [M 0x114e70:43-54].
        if self.sfx_paused { return; }
        let t = self.delay_timer - 1;
        if self.delay_timer < 0 { self.last_jump = iop_section; }
        if e[0xb] != 0 { self.nis_playing = true; }
        if e[0xc] != 0 || e[5] != 0 { self.nis_playing = false; }
        if e[5] != 0 { self.nis_quitten = 5; } else if self.nis_quitten > 0 { self.nis_quitten -= 1; }
        let Some(m) = self.entry.clone() else { self.events = [0; 64]; return };
        if !self.nis_playing {
            if e[2] == 2 { self.bonus = 600; self.under_attack = true; }
            if e[2] == 0 || e[2] == 2 {
                let b = self.bonus; self.bonus -= 1;
                if b < 0 { let c = self.count; self.count -= 1; if c < 0 { self.under_attack = false; self.count2 = m.alert_frames; } }
            } else {
                let c2 = self.count2; self.count2 -= 1;
                if c2 < 0 { self.under_attack = true; self.bonus = 0; self.count = m.calm_frames; }
            }
        }
        self.delay_timer = t;
        if e[6] != 0 { self.jump(m.on_death, true); self.out.push(Cmd::FadeDown { music: false }); self.wait_for_end = true; }
        if e[7] != 0 { self.jump(m.on_fail, true); self.out.push(Cmd::FadeDown { music: false }); self.wait_for_end = true; }
        // The dispatcher's own gate [M disasm 0x1153c4-0x1153c8: `lw v0,-32144(gp)` (MusicWaitForEnd
        // 0x30c8e0); `bne v0,zero,0x115554` skips the whole per-level switch]: with the end-wait
        // standing NO updater runs — the exit sting is the last section the board ever requests
        // this level. The 07/08/1b handlers below previously ran unconditionally; the game runs
        // them under the same gate (their internal `MusicWaitForEnd` checks — 0x113188:40,
        // 0x114d60:40 — guard blocks INSIDE the handlers, which the dispatcher already excludes).
        if !self.wait_for_end {
            // `UpdateMFX_HT_Level_Tower2C` 0x114640:9-12: the alarm jump is the updater's FIRST
            // statement — queued BEFORE the shared exit block, so on a frame with both the alarm
            // and an exit event the game's batch holds jump(0, non-instant) then the end-marker
            // jump (the IOP's last request wins) [M order: 0x114640:9-12 vs :14-39].
            if m.level == 0x0700_0013 && self.under_attack { self.jump(0, false); }
            // The shared level-exit block, ported from every level updater that carries it [M
            // 0x113558:9-33 (TowerA, 07000009); the same block in 0x111ed8 MayhewA/07000001:9-,
            // 0x112270 MayhewB/02, 0x112730 MayhewC/03, 0x112bf8 MayhewD/04, 0x1137b0 TowerB/0700000a,
            // 0x113c18 PowerStationA1/0c, 0x113fc0 PowerStationA2/0d, 0x114250
            // UpdateMFX_HT_Level_Tower2A/11, 0x114348 UpdateMFX_HT_Level_Tower2B/12, 0x114640
            // UpdateMFX_HT_Level_Tower2C/13, 0x114728 UpdateMFX_HT_Level_EvilBase/14, 0x114a38
            // UpdateMFX_HT_Level_EvilSilo/15, 0x114b38 UpdateMFX_HT_Level_EvilBaseC/16, 0x114528
            // UpdateMFX_HT_Level_Tower2Elevator/0700004a:9-34 — all `DAT_002d3518 ||
            // DAT_002d3504` = e[8] (mission success, 0x182c60:31) || e[3] (the Trigger_Activate
            // LOAD_LEVEL exit, 0x1b2318:109)]: the +0x24 combat-set scan picks the end marker
            // (+0x14 end_combat when MusicLastJump is a combat section, else +0x18 end_calm), the
            // jump is INSTANT, `SFXFadeDown(0)` (the SFX bus — the streamed music follows
            // FadeMusicVolume, untouched by flag 0 [M sound.md §5.5]), `MusicWaitForEnd = 1`.
            // 06's updater (CastleB 0x113060) has NO such block — its SFXFadeDown (:37) rides the
            // 0x6000071 NIS event — and 0b's (TowerC 0x113a20) has none either: its exit arm is
            // state 6's fixed 0xe jump below [M 0x113a20:69-75], reachable only after the finale
            // NIS pair — so 06 and 0b are absent here.
            if matches!(m.level, 0x0700_0001..=0x0700_0004 | 0x0700_0009..=0x0700_000a | 0x0700_000c..=0x0700_000d | 0x0700_0011..=0x0700_0016 | 0x0700_004a) {
                self.exit_block(&e, &m);
            }
            if m.level == 0x0700_0007 { self.castle_c(&e, &m); }
            if m.level == 0x0700_0008 { self.castle_d(&e, &m); }
            if m.level == 0x0700_001b { self.space_station_d(&e, &m); }
            if m.level == 0x0700_0005 { self.castle_a(&e, &m); }
            // `UpdateMFX_01CastleB` 0x113060 (07000006), whole — see `castle_b` below.
            if m.level == 0x0700_0006 { self.castle_b(&e, &m); }
            // `UpdateMFX_02MayhewB` 0x112270 (07000002) and `UpdateMFX_02MayhewC` 0x112730
            // (07000003) — the trigger/sight machines behind the shared exit block above.
            if m.level == 0x0700_0002 { self.mayhew_b(&e, &m); }
            if m.level == 0x0700_0003 { self.mayhew_c(&e, &m); }
            // `UpdateMFX_02MayhewD` 0x112bf8 (07000004) — the story scene's three markers.
            if m.level == 0x0700_0004 { self.mayhew_d(&e); }
            // `UpdateMFX_03TowerC` 0x113a20 (0700000b) — the NIS-id + cel-zone state machine.
            if m.level == 0x0700_000b { self.tower_c(&e); }
            // The HT-level updaters with a per-level half (docs/systems/ht-music-updaters.md).
            // Tower2A (0x0700_0011) has none: `UpdateMFX_HT_Level_Tower2A` 0x114250:9-34 is the
            // shared exit block above, whole.
            if m.level == 0x0700_0012 { self.tower2_b(&m); }
            if m.level == 0x0700_0014 { self.evil_base(&e, &m); }
            if m.level == 0x0700_0015 { self.evil_silo(); }
            if m.level == 0x0700_0016 { self.evil_base_c(&e, &m); }
            if m.level == 0x0700_004a { self.tower2_elevator(&e); }
        }
        self.events = [0; 64];
    }

    /// The shared level-exit block — see the dispatch site for the per-handler addresses.
    fn exit_block(&mut self, e: &[u32; 64], m: &MapMusic) {
        if e[3] == 0 && e[8] == 0 { return; }
        let found = m.combat.iter().any(|&s| s == self.last_jump);
        let s = if found { m.end_combat } else { m.end_calm };
        self.jump(s, true);
        self.out.push(Cmd::FadeDown { music: false });
        self.wait_for_end = true;
    }

    /// `UpdateMFX_HT_Level_SpaceStationD` 0x114d60 (0700001b, the space-station finale). The
    /// alert/combat pair (`DAT_002d3518`/`DAT_002d3504` = the port's e[8]/e[3] by the EventList slot
    /// map — DAT_002d34fc+4k ↔ e[k+1], the e[0xb] NisPlaying precedent) fades the level's music down
    /// and waits for the end; the intro cinema's end (`Script_KillStream` of 0x0600085c [M 0x1d9860:74]
    /// posts the hash into `DAT_002d3528` = e[0xc]) restarts the track at section 0 while no end-wait
    /// holds. 1b's combat list is EMPTY on disc [D MapMusic row 20], so the scan always takes the
    /// end_calm arm.
    fn space_station_d(&mut self, e: &[u32; 64], m: &MapMusic) {
        if e[8] != 0 || e[3] != 0 {
            // The combat-set scan [M 0x114d60:10-25]: MusicLastJump in the combat list jumps
            // end_combat, else end_calm — instant [M 0x114d60:27-34].
            let found = m.combat.iter().any(|&s| s == self.last_jump);
            let s = if found { m.end_combat } else { m.end_calm };
            self.last_request = s;
            self.jump(s, true);
            if m.level != 0x0700_0006 { self.out.push(Cmd::FadeDown { music: false }); }  // [M 0x114d60:35-37]
            self.wait_for_end = true;
        }
        if !self.wait_for_end && e[0xc] == 0x6000_085c {
            self.last_request = 0;
            self.jump(0, false);   // SFXJumpToMusicMarker(0, 0) — not instant [M 0x114d60:40-43]
        }
    }

    /// `UpdateMFX_HT_Level_Tower2B` 0x114348:36-76 (07000012): the 3-state alarm machine on
    /// `MusicTemp1` (the port's `tower2b_temp1`). State 0 + attack → jump 0xf INSTANT, state 1;
    /// state 1 + calm → jump 0x11 non-instant, state 2; state 2: calm jumps 0x16 (non-instant)
    /// only while the IOP's section (`MusicLastJump`) is one of the level's combat sections
    /// (the +0x24 scan [M 0x114348:49-66]), attack round-robins `PickRandomCombat(0)`.
    fn tower2_b(&mut self, m: &MapMusic) {
        match self.tower2b_temp1 {
            1 => { if !self.under_attack { self.jump(0x11, false); self.tower2b_temp1 = 2; } }
            2 => {
                if self.under_attack { self.pick_combat(false); }
                else if m.combat.iter().any(|&s| s == self.last_jump) { self.jump(0x16, false); }
            }
            _ => { if self.under_attack { self.jump(0xf, true); self.tower2b_temp1 = 1; } }
        }
    }

    /// `UpdateMFX_HT_Level_EvilSilo` 0x114a38:41-47 (07000015): every frame after the exit block,
    /// the alarm picks `PickRandomCombat(0)` else `PickRandomStealth`.
    fn evil_silo(&mut self) {
        if self.under_attack { self.pick_combat(false); } else { self.pick_stealth(); }
    }

    /// `UpdateMFX_HT_Level_Tower2Elevator` 0x114528:36-40 (0700004a): while the elevator ride's
    /// NIS stream hash 0x60006d3 stands in e[0xb] (`DAT_002d3524`), jump the fx-int marker 4,
    /// non-instant. The game stashes the `&o_fx_int` mark in `MusicLastJumpRequest` — bookkeeping
    /// only: `SFXJumpToMusicMarker` queues its ARGUMENT [M 0x116950:3-15] and no reader ever
    /// compares the request to a pointer, so the port's `jump` records the section.
    fn tower2_elevator(&mut self, e: &[u32; 64]) {
        if e[0xb] == 0x0600_06d3 { self.jump(4, false); }
    }

    /// `UpdateMFX_HT_Level_EvilBaseC` 0x114b38:36-77 (07000016): the 4-state cut-scene machine on
    /// `MusicTemp1` (the port's `evil_base_c_t1`). The 0x6000631 arm runs EVERY frame [M
    /// 0x114b38:36-40], then the state machine: 1 → the stream-kill hash 0x6000631 in e[0xc]
    /// jumps 0x27 instant; 2 → the 0x6000632 stream in e[0xb] jumps 0x29; 3 → the exit pair
    /// (e[3]/e[8]) jumps 0x2b non-instant and holds the end-wait; 0 → the alarm picks combat or
    /// stealth.
    fn evil_base_c(&mut self, e: &[u32; 64], _m: &MapMusic) {
        if e[0xb] == 0x0600_0631 { self.jump(0x24, true); self.evil_base_c_t1 = 1; }
        match self.evil_base_c_t1 {
            2 => { if e[0xb] == 0x0600_0632 { self.jump(0x29, false); self.evil_base_c_t1 = 3; } }
            1 => { if e[0xc] == 0x0600_0631 { self.jump(0x27, true); self.evil_base_c_t1 = 2; } }
            3 => { if e[3] != 0 || e[8] != 0 { self.jump(0x2b, false); self.wait_for_end = true; } }
            _ => { if self.under_attack { self.pick_combat(false); } else { self.pick_stealth(); } }
        }
    }

    /// `UpdateMFX_HT_Level_EvilBase` 0x114728:36-134 (07000014): the busiest machine. Latches on
    /// the port's `evil_base_t2/t3/t4/t6` and the counter `evil_base_t1`: the stream-kill hash
    /// 0x6000848 in e[0xc] sets t3 and the tail `SFXJumpToMusicMarker(0x18, 1)` re-issues EVERY
    /// frame the latch holds [M 0x114728:36-40 + :134]; the NIS-start hash in e[0xb] sets t6 and
    /// jumps 0x22 instant once; the 0x600062f scene (e[0xb]) latches t2, zeroes t1 and jumps 5
    /// instant calm / 0x12 attack; the 0x6000833 hash (e[0xc], or t1 already up) counts t1 and
    /// jumps 0xf non-instant when the IOP sits in a combat section, else 0x14 instant (request
    /// `&o_left` — bookkeeping, see tower2_elevator), or gives to `PickRandomCombat(1)` past 0x37
    /// under attack; the t4 arm (0x6000833 in e[0xb]) jumps 0x34 when the IOP is in 0x2e, else
    /// 0x16 instant; with no hash at all, calm jumps 0x32 non-instant (once, when the IOP leaves
    /// 5), attack jumps 0x2e instant.
    fn evil_base(&mut self, e: &[u32; 64], m: &MapMusic) {
        if e[0xc] == 0x0600_0848 || self.evil_base_t3 != 0 {
            self.evil_base_t3 = 1;
            self.jump(0x18, true);
            return;
        }
        if e[0xb] == 0x0600_0848 || self.evil_base_t6 != 0 {
            self.evil_base_t6 = 1;
            self.jump(0x22, true);
        }
        if e[0xb] == 0x0600_062f || self.evil_base_t2 != 0 {
            if self.evil_base_t2 != 0 { return; }
            self.evil_base_t2 = 1;
            self.evil_base_t1 = 0;
            if self.under_attack { self.jump(0x12, false); } else { self.jump(5, true); }
        }
        if e[0xc] == 0x0600_0833 || self.evil_base_t1 != 0 {
            self.evil_base_t1 += 1;
            if !self.under_attack {
                if m.combat.iter().any(|&s| s == self.last_jump) { self.jump(0xf, false); return; }
                if self.last_request == 0xf { return; }
                self.jump(0x14, true);
            } else {
                if self.evil_base_t1 > 0x37 { self.pick_combat(true); return; }
                self.jump(0x14, true);
            }
        } else if e[0xb] == 0x0600_0833 || self.evil_base_t4 != 0 {
            if self.evil_base_t4 != 0 { return; }
            self.evil_base_t4 = 1;
            if self.last_jump == 0x2e { self.jump(0x34, false); return; }
            self.jump(0x16, true);
        } else if !self.under_attack {
            if self.last_jump == 5 { return; }
            self.jump(0x32, false);
        } else {
            self.jump(0x2e, true);
        }
    }

    /// `UpdateMFX_01CastleA` (07000005).

    fn castle_a(&mut self, e: &[u32; 64], m: &MapMusic) {
        let in_combat = |x: i32| m.combat.contains(&x);
        if e[2] == 3 && !self.temp4 { self.temp4 = true; self.temp3 = 1200; self.pick_combat(false); return; }
        if self.temp3 > 0 { self.temp3 -= 1; return; }
        if e[0xc] == 0x0600_0994 { self.temp1 = false; self.temp2 = 66; }
        if self.temp2 > 0 { self.temp2 -= 1; return; }
        if e[0xb] == 0x0600_0994 || self.temp1 {
            let s = if in_combat(self.last_jump) { 6 } else { 2 };
            self.jump(s, false);
            self.temp1 = true;
            return;
        }
        if self.under_attack { self.pick_combat(false); }
        else if e[0xd] != 0 { let s = if in_combat(self.last_jump) { 6 } else { 2 }; self.jump(s, false); }
        else if self.last_request != 2 { self.pick_stealth(); }
        else { self.jump(20, false); }
        if e[8] != 0 || e[3] != 0 {
            let s = if in_combat(self.last_jump) { m.end_combat } else { m.end_calm };
            self.jump(s, true);
            self.out.push(Cmd::FadeDown { music: false });
            self.wait_for_end = true;
        }
        if !self.wait_for_end && e[4] != 0 { self.jump(9, false); self.wait_for_end = true; }
    }

    /// The level-end block shared by CastleC/CastleD (0x113188:5-33 / 0x1133b8:22-44 — CastleA's twin):
    /// event 3 (a `Trigger_Activate` end trigger, 0x1b2318) or 8 (mission success, 0x182c60) jumps the
    /// record's +0x14/+0x18 marker — the +0x24 combat-set scan picks which — instantly, fades down and
    /// waits for the end.
    fn castle_end_block(&mut self, e: &[u32; 64], m: &MapMusic) {
        if e[3] == 0 && e[8] == 0 { return; }
        let s = if m.combat.contains(&self.last_jump) { m.end_combat } else { m.end_calm };
        self.jump(s, true);
        self.out.push(Cmd::FadeDown { music: false });
        self.wait_for_end = true;
    }

    /// `UpdateMFX_01CastleC` (0x113188, 07000007) [M]. EventList slots: 3 = a level-end trigger, 8 = mission
    /// success, 11 = Music_Event(0xB, script) at NIS start, 12 = Music_Event(0xC, script) at NIS end,
    /// 13 = Music_Event(0xD). The NIS scripts 0x6000072 (the Drake reveal) and 0x6000073 (the hall walk-in)
    /// jump their own section pairs (2→5, 7→0xa); the cut-scene music sound (0x172) sets `nis_trigger`
    /// [M 0x115a68] which completes the second jump of the pair.
    fn castle_c(&mut self, e: &[u32; 64], m: &MapMusic) {
        self.castle_end_block(e, m);
        if self.wait_for_end || self.c_temp2 != 0 { return; }
        if self.c_temp1 == 0 {
            if e[11] == 0x0600_0072 { self.jump(2, false); }
            if e[12] == 0x0600_0072 || self.nis_trigger {
                self.jump(5, false);
                self.nis_trigger = false;
                self.c_temp1 = 1;
            }
        }
        if e[11] == 0x0600_0073 { self.jump(7, false); self.c_temp4 = true; }
        if self.c_temp4 && (e[12] == 0x0600_0073 || self.nis_trigger) {
            self.jump(0xa, false);
            self.c_temp1 = 10;
            self.nis_trigger = false;
        }
        if self.c_temp1 == 10 && self.under_attack { self.jump(0xe, false); self.c_temp2 = 1; }
        if e[13] != 0 { self.jump(0xc, false); }
    }

    /// `UpdateMFX_01CastleD` (0x1133b8, 07000008) [M]. The intro NIS 0x6000077: start (event 11, only while
    /// `MusicLastJump == 0`) → section 2 instantly; end (event 12) → section 4 instantly. Then the level-end
    /// block; while not waiting: event 13 (once) → section 6, and the NIS music sound → section 8 instantly.
    fn castle_d(&mut self, e: &[u32; 64], m: &MapMusic) {
        if self.last_jump == 0 && e[11] == 0x0600_0077 { self.jump(2, true); }
        if e[12] == 0x0600_0077 { self.jump(4, true); }
        self.castle_end_block(e, m);
        if self.wait_for_end { return; }
        if e[13] != 0 && self.c_temp1 == 0 { self.jump(6, false); self.c_temp1 = 1; }
        if self.nis_trigger { self.jump(8, true); self.nis_trigger = false; }
    }

    /// `UpdateMFX_01CastleB` 0x113060 (07000006), whole. While `MusicWaitForNisSfxTrigger` is
    /// down (the port's `b_wait_nis`): the 0x6000071 NIS-start event (e[11]) jumps the record's
    /// +0x14/+0x18 marker — the +0x24 combat-set scan against `MusicLastJump` picks which —
    /// INSTANTLY and latches the wait; the block's `SFXFadeDown(0)` is dead on this level (the
    /// game skips it while `MapMusicData == 0x7000006` [M 0x113060:36-38]). Then the alarm pick.
    /// While the wait stands the alarm pick stops until the cut-scene's music sound
    /// (`SfxNisMusicTrigger`, the port's `nis_trigger`) jumps 0xc non-instant and holds the
    /// end-wait — the level's closing sting [M 0x113060:48-53].
    fn castle_b(&mut self, e: &[u32; 64], m: &MapMusic) {
        if self.b_wait_nis == 0 {
            if e[0xb] == 0x0600_0071 {
                let s = if m.combat.contains(&self.last_jump) { m.end_combat } else { m.end_calm };
                self.jump(s, true);
                self.b_wait_nis = 1;
            }
            if self.under_attack { self.pick_combat(false); } else { self.pick_stealth(); }
        } else if self.nis_trigger {
            self.jump(0xc, false);
            self.nis_trigger = false;
            self.wait_for_end = true;
        }
    }

    /// `UpdateMFX_02MayhewB` 0x112270 (07000002), after the shared exit block above. Level-start
    /// arm: with the IOP still at section 0 (`MusicLastJump == 0`), the intro NIS 0x6000084's END
    /// (e[12]) or the level's e[36] channel starts section 2, non-instant [M 0x112270:48-54 +
    /// the (2, 0) tail :201-202]. With the IOP in a section: every tick re-seats the attack gate
    /// (e[37] low → the level's `MapDroneData` row back; e[37] high → distance 1.0) [M :56-79];
    /// the 0x600007f cut-scene pair jumps 6 (start, e[11]) and 8 (end, e[12]) instantly, the wait
    /// between them running alone [M :81-95]; the ch15/16 latch (a first-sight e[15] or
    /// hostage-saved e[16] event) picks `PickRandomCombat(0)` and re-arms the 300-tick gate [M
    /// :96-103]; the gate counts down and then the trigger pair fires once each: e[31] jumps 4
    /// INSTANT, e[32] jumps 4 non-instant, both `MusicVars = 1` + bonus 600 + the latch [M
    /// :104-127]; the alarm round-robins `PickRandomCombat(0)` [M :128-131]; calm, while e[30] is
    /// low: the stealth round-robin — the +0x38 scan against `MusicLastJumpRequest` leaves an
    /// in-set request alone, else 0x14/0x16 round-robin, INSTANT while the IOP sits in 4 (the
    /// fx_int section) [M :132-163]; once e[30] fired (t5), 0x12 non-instant [M :165-168]; the
    /// e[30] tail: the +0x24 scan against `MusicLastJump` — in the set → 0x12 non-instant; at
    /// 0x12 already → hold; else 0xa non-instant [M :169-202].
    fn mayhew_b(&mut self, e: &[u32; 64], m: &MapMusic) {
        if self.last_jump == 0 {
            if e[0xc] != 0x0600_0084 && e[36] == 0 { return; }
            self.jump(2, false);
            return;
        }
        // The attack-gate re-seat [M 0x112270:56-79]: e[37] low → the MapDroneData row back; e[37]
        // high → `DAT_002707a8 = 1.0`. The game's write lands for the next frame's
        // SeenAndAttacking pass (the drone pass precedes 0x114e70 in the frame) — Drones::tick's
        // drain keeps that order. The scan's default row (no MapDroneData match): count 2, floor
        // 8, distance 25, the window untouched [M 0x112270:72-75].
        self.gate_writes.push(if e[37] == 0 { GateWrite::Restore } else { GateWrite::SetDistance(1.0) });
        if e[0xb] == 0x0600_007f { self.jump(6, true); self.may_b_t4 = 1; }  // MusicTemp7 = 0 too [M :82] — written, never read
        if self.may_b_t4 != 0 {
            if e[0xc] != 0x0600_007f { return; }
            self.jump(8, true);
            self.may_b_t4 = 0;
            return;
        }
        if self.may_b_t1 != 0 {
            if e[0xf] == 0 && e[0x10] == 0 { return; }
            self.pick_combat(false);
            self.may_b_t1 = 0;
            self.may_b_t9 = 300;
        }
        let pre = self.may_b_t9;
        self.may_b_t9 -= 1;
        if pre > 0 { return; }
        if e[31] != 0 && self.may_b_t2 == 0 {
            self.jump(4, true);
            self.gate_writes.push(GateWrite::SetCount(1));
            self.bonus = 600;
            self.may_b_t1 = 1;
            self.may_b_t2 = 1;
            return;
        }
        if e[32] != 0 && self.may_b_t3 == 0 {
            self.jump(4, false);
            self.gate_writes.push(GateWrite::SetCount(1));
            self.bonus = 600;
            self.may_b_t1 = 1;
            self.may_b_t3 = 1;
            return;
        }
        if self.under_attack { self.pick_combat(false); return; }
        if self.may_b_t5 == 0 {
            if !m.stealth.contains(&self.last_request) {
                let instant = self.last_jump == 4;
                if self.may_b_which == 0 { self.jump(0x14, instant); }
                if self.may_b_which == 1 { self.jump(0x16, instant); }
                self.may_b_which += 1;
                if self.may_b_which > 1 { self.may_b_which = 0; }
            }
        } else {
            self.jump(0x12, false);
        }
        if e[30] == 0 { return; }
        self.may_b_t5 = 1;
        if m.combat.contains(&self.last_jump) {
            self.jump(0x12, false);
        } else if self.last_jump == 0x12 {
            self.may_b_t5 = 1;
        } else {
            self.jump(0xa, false);
        }
    }

    /// `UpdateMFX_02MayhewC` 0x112730 (07000003), after the shared exit block above. The ch15/16
    /// latch (first-sight e[15] / hostage-saved e[16]) picks `PickRandomCombat(0)` and re-arms
    /// the 0x78-tick gate [M 0x112730:41-48]; while the gate counts, nothing runs [M :49-51];
    /// then the trigger pair, once each: e[31] jumps 0xa, e[32] jumps 0xc (both non-instant, both
    /// `MusicVars = 1` + bonus 600 + the latch) [M :53-53, :207-223]; then calm: the +0x38 scan
    /// against `MusicLastJumpRequest` — outside the stealth set, the +0x24 scan against
    /// `MusicLastJump` picks the 6/8 round-robin, at 0xa/0xc already the bonus<0 restart jump 0
    /// INSTANT, else 0 non-instant [M :55-113]; attack: e[33] low — outside the combat set on
    /// BOTH the IOP section and the request and neither at 0xe — the 2/4 round-robin [M :115-164];
    /// e[33] high — outside the set on both — 0xe non-instant [M :166-204].
    fn mayhew_c(&mut self, e: &[u32; 64], m: &MapMusic) {
        if self.may_c_t1 != 0 {
            if e[0xf] == 0 && e[0x10] == 0 { return; }
            self.pick_combat(false);
            self.may_c_t1 = 0;
            self.may_c_t9 = 0x78;
        }
        let pre = self.may_c_t9;
        self.may_c_t9 -= 1;
        if pre >= 1 { return; }
        if e[31] != 0 && self.may_c_t2 == 0 {
            self.jump(0xa, false);
            self.gate_writes.push(GateWrite::SetCount(1));
            self.bonus = 600;
            self.may_c_t2 = 1;
            self.may_c_t1 = 1;
            return;
        }
        if e[32] != 0 && self.may_c_t3 == 0 {
            self.jump(0xc, false);
            self.gate_writes.push(GateWrite::SetCount(1));
            self.bonus = 600;
            self.may_c_t3 = 1;
            self.may_c_t1 = 1;
            return;
        }
        if !self.under_attack {
            if !m.stealth.contains(&self.last_request) {
                if m.combat.contains(&self.last_jump) {
                    if self.may_c_which_calm == 0 { self.jump(6, false); }
                    if self.may_c_which_calm == 1 { self.jump(8, false); }
                    self.may_c_which_calm += 1;
                    if self.may_c_which_calm > 1 { self.may_c_which_calm = 0; }
                } else if self.last_jump == 0xa || self.last_jump == 0xc {
                    if self.bonus < 0 { self.jump(0, true); }
                } else {
                    self.jump(0, false);
                }
            }
        } else if e[33] == 0 {
            if !m.combat.contains(&self.last_jump) && !m.combat.contains(&self.last_request)
                && self.last_jump != 0xe && self.last_request != 0xe {
                if self.may_c_which_attack == 0 { self.jump(2, false); }
                if self.may_c_which_attack == 1 { self.jump(4, false); }
                self.may_c_which_attack += 1;
                if self.may_c_which_attack > 1 { self.may_c_which_attack = 0; }
            }
        } else if !m.combat.contains(&self.last_jump) && !m.combat.contains(&self.last_request) {
            self.jump(0xe, false);
        }
    }

    /// `UpdateMFX_02MayhewD` 0x112bf8 (07000004), after the shared exit block above. The
    /// 0x6000082 story scene's start (e[11]) jumps marker 2 and latches `MusicTemp2` [M :41-45];
    /// the first `SfxNisMusicTrigger` (the scene's 0x172 sound) jumps marker 3 and latches
    /// `MusicTemp1` [M :50-55]; the next trigger, or the skip event e[5] once the scene has
    /// started, jumps marker 4, the section the fight runs on [M :46-49, :59-61]. All instant.
    fn mayhew_d(&mut self, e: &[u32; 64]) {
        if e[0xb] == 0x0600_0082 {
            self.jump(2, true);
            self.may_d_t2 = 1;
        }
        if !self.nis_trigger {
            if e[5] == 0 || self.may_d_t2 == 0 { return; }
        } else if self.may_d_t1 == 0 {
            self.jump(3, true);
            self.nis_trigger = false;
            self.may_d_t1 = 1;
            return;
        }
        self.jump(4, true);
        self.nis_trigger = false;
    }

    /// `UpdateMFX_03TowerC` 0x113a20 (0700000b), whole — the 0b row is NOT in the shared
    /// exit-block range: the game's updater has no `e[3]/e[8]` head; its only exit arm is state
    /// 6's fixed 0xe jump, reachable only after the finale NIS pair [M 0x113a20:8-76 vs the
    /// shared head 0x113558:9-33]. State machine on `MusicTemp1` (the port's `tower_c_t1`):
    /// 0: the 0x60004f9 NIS start (e[11]) jumps 2, non-instant; 1: the cel music-zone event on
    /// e[44] jumps 4; 2: e[41] jumps 6; 3: e[17] jumps 8 INSTANT [M :8-35]; 4: the alarm pair —
    /// attack latches `MusicTemp2` and jumps 8 non-instant, calm after a latch jumps 0x15; the
    /// 0x60004fa NIS start (e[11]) jumps 10 (attack) / 0x17 (calm), non-instant, and moves to 5
    /// [M :37-61]; 5: the NIS end (e[12] == 0x60004fa) jumps 0xc INSTANT [M :62-68]; 6: the
    /// exit/fail events jump 0xe INSTANT — no fade, no end-wait [M :69-75].
    fn tower_c(&mut self, e: &[u32; 64]) {
        match self.tower_c_t1 {
            0 => { if e[0xb] == 0x0600_04f9 { self.jump(2, false); self.tower_c_t1 = 1; } }
            1 => { if e[44] != 0 { self.jump(4, false); self.tower_c_t1 = 2; } }
            2 => { if e[41] != 0 { self.jump(6, false); self.tower_c_t1 = 3; } }
            3 => { if e[17] != 0 { self.jump(8, true); self.tower_c_t1 = 4; } }
            4 => {
                if self.under_attack { self.tower_c_t2 = 1; self.jump(8, false); }
                else if self.tower_c_t2 != 0 { self.jump(0x15, false); }
                if e[0xb] == 0x0600_04fa {
                    if self.under_attack { self.jump(10, false); } else { self.jump(0x17, false); }
                    self.tower_c_t1 = 5;
                }
            }
            5 => { if e[0xc] == 0x0600_04fa { self.jump(0xc, true); self.tower_c_t1 = 6; } }
            6 => { if e[3] != 0 || e[8] != 0 { self.jump(0xe, true); self.tower_c_t1 = 99; } }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 04 row's shape for the updater tests: only the level matters to `mayhew_d`.
    fn mayhew_d_board() -> Music {
        let mut m = Music::new(vec![MapMusic { level: 0x0700_0004, track: 22, start: 0, on_death: 19, on_fail: 18, end_combat: 16, end_calm: 17, alert_frames: 10, calm_frames: 160, combat: vec![], stealth: vec![] }]);
        m.event(0, 0x0700_0004); m.event(1, 1);
        m.update(0);
        m
    }

    /// [M 0x112bf8:41-61] the story start, the first 0x172, the second 0x172.
    #[test]
    fn mayhew_d_walks_the_story_markers() {
        let mut m = mayhew_d_board();
        m.update(0);
        assert_eq!(m.last_request, -1, "nothing moves before the scene");
        m.event(0xb, 0x0600_0082); m.update(0);
        assert_eq!(m.last_request, 2);
        m.nis_trigger = true; m.update(0);
        assert_eq!(m.last_request, 3);
        assert!(!m.nis_trigger);
        m.update(0);
        assert_eq!(m.last_request, 3, "the marker holds between the sounds");
        m.nis_trigger = true; m.update(0);
        assert_eq!(m.last_request, 4);
    }

    /// [M 0x112bf8:46-49] the skip event moves the score only once the story scene has started:
    /// skipping the 0x6000083 intro leaves it, skipping the story lands on marker 4.
    #[test]
    fn mayhew_d_skip_needs_the_story_scene() {
        let mut m = mayhew_d_board();
        m.event(5, 0x0600_0083); m.update(0);
        assert_eq!(m.last_request, -1, "the intro's skip");
        m.event(0xb, 0x0600_0082); m.update(0);
        m.event(5, 0x0600_0082); m.update(0);
        assert_eq!(m.last_request, 4, "the story's skip");
    }

    fn castle() -> MapMusic {
        MapMusic { level: 0x0700_0005, track: 15, start: 0, on_death: 11, on_fail: 10, end_combat: 8, end_calm: 9, alert_frames: 10, calm_frames: 160, combat: vec![4, 12, 14], stealth: vec![6, 16, 18] }
    }

    #[test]
    fn alert_hysteresis_and_round_robin() {
        let mut m = Music::new(vec![castle()]);
        m.event(0, 0x0700_0005); m.event(1, 1);
        m.update(0);
        assert_eq!(m.out.drain(..).collect::<Vec<_>>(), vec![Cmd::Start { track: 15, section: 0 }]);
        // Combat on the 12th cumulative alerted frame (count2 10 → −1 after 11 decrements, the 12th sees < 0).
        let mut first = None;
        for f in 1..=20 { m.event(2, 5); m.update(0); if first.is_none() && m.out.iter().any(|c| *c == Cmd::Jump { section: 4, instant: false }) { first = Some(f); } m.out.clear(); }
        assert_eq!(first, Some(12));
        // The IOP reports section 4; 163 calm frames later stealth 6 is requested (sister: not at 162).
        let mut at = None;
        for f in 1..=170 { m.update(4); if at.is_none() && m.out.iter().any(|c| *c == Cmd::Jump { section: 6, instant: false }) { at = Some(f); } m.out.clear(); }
        assert_eq!(at, Some(163));
        // Death: the instant end sting, then nothing more.
        m.event(6, 1); m.update(6);
        assert!(m.out.contains(&Cmd::Jump { section: 11, instant: true }) && m.wait_for_end);
    }

    #[test]
    fn attack_gate_needs_two_recent_attackers_or_sight() {
        let mut g = AttackGate { vars: Some(MusicVars { level: 0x0700_0005, count: 2, floor: 0, window: 2, distance: 35.0 }), ..Default::default() };
        g.new_frame();
        assert!(!g.seen_and_attacking(0, false, 30, 50.0), "one recent attacker is not enough");
        assert!(g.seen_and_attacking(1, false, 30, 50.0), "the second one is");
        g.new_frame();
        assert!(g.seen_and_attacking(2, true, 0, 50.0), "seeing Bond is enough");
        // Sister: not seen for 2 s and far → nothing.
        g.new_frame(); g.new_frame();
        assert!(!g.seen_and_attacking(3, false, 200, 50.0));
    }

    /// The real 06 row [D disc MapMusic 0x24f4d8, level 0x07000006]: end_combat 8, end_calm 6,
    /// combat [2, 15, 17], stealth [4, 19, 21].
    fn castle_b() -> MapMusic {
        MapMusic { level: 0x0700_0006, track: 16, start: 0, on_death: 11, on_fail: 10, end_combat: 8, end_calm: 6, alert_frames: 10, calm_frames: 160, combat: vec![2, 15, 17], stealth: vec![4, 19, 21] }
    }

    /// `UpdateMFX_01CastleB` 0x113060 (07000006): the alarm's `Music_Event(2,1)` stream drives the hysteresis
    /// into `under_attack`, and the per-level dispatch (fix-npcs) then demands combat — pre-fix nothing ran
    /// for this level, so the alarm raised ch 55 but the music never left stealth.
    #[test]
    fn the_07000006_alarm_demands_combat_music() {
        let mut m = Music::new(vec![castle_b()]);
        m.event(0, 0x0700_0006); m.event(1, 1);
        m.update(0);
        assert_eq!(m.out.drain(..).collect::<Vec<_>>(), vec![Cmd::Start { track: 16, section: 0 }]);
        // The CheckAlarmRaised stream: Music_Event(2,1) every tick while ch 55 counts. After the 10
        // alert_frames the hysteresis flips under_attack and castle_b picks a combat section.
        let mut jumped = None;
        for f in 1..=30 {
            m.event(2, 1);
            m.update(0);
            if let Some(Cmd::Jump { section, instant: false }) = m.out.iter().copied().find(|c| matches!(c, Cmd::Jump { instant: false, .. })) {
                assert!(vec![2, 15, 17].contains(&section), "the jump targets the combat set — got {section}");
                jumped = Some(f);
                break;
            }
            m.out.clear();
        }
        assert_eq!(jumped, Some(12), "the combat pick lands on the 12th streamed tick (count2 10 → <0 on the 11th, dispatch same tick)");
        // Sister: no alarm, no jump — stealth is only picked coming OUT of combat (the IOP reports a combat
        // section), so a quiet 07000006 requests nothing.
        let mut m = Music::new(vec![castle_b()]);
        m.event(0, 0x0700_0006); m.event(1, 1);
        m.update(0);
        m.out.clear();
        for _ in 0..40 { m.update(0); }
        assert!(m.out.is_empty(), "a quiet 07000006 picks no section");
    }

    /// 07000009's MapMusic row [D disc MapMusic table 0x24f4d8, level 0x07000009; parsed by
    /// nf-formats::music — the live test reads the real row]. Combat set non-empty so both scan
    /// arms are exercisable.
    fn tower_a() -> MapMusic {
        MapMusic { level: 0x0700_0009, track: 20, start: 0, on_death: 14, on_fail: 13, end_combat: 12, end_calm: 15, alert_frames: 10, calm_frames: 160, combat: vec![4, 12, 16], stealth: vec![2, 6, 18] }
    }

    /// The 09 lift-exit's music arm [M 0x1b2318:109 `Music_Event(3,1)` → 0x113558:9-33]: event 3
    /// jumps the level's end marker INSTANTLY, queues `SFXFadeDown(0)` and holds the end-wait —
    /// the music fade (the port previously had no updater for 09, so the
    /// event died on the board). The combat-set scan: MusicLastJump outside the set → end_calm.
    /// Sister: with MusicLastJump INSIDE the set the same event picks end_combat instead — the
    /// scan is the only difference, so a wrong marker (or a non-instant jump, or a missing
    /// FadeDown) fails one arm while the other passes.
    #[test]
    fn the_09_exit_event_jumps_the_end_marker_and_fades_down() {
        let mut m = Music::new(vec![tower_a()]);
        m.event(0, 0x0700_0009); m.event(1, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Start { track: 20, section: 0 }), "the level start: {:?}", m.out);
        m.out.clear();
        // The exit trigger's event 3 (Trigger_Activate 0x1b2318:109); MusicLastJump is the level
        // start section 0 — outside the combat set → end_calm.
        m.event(3, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 15, instant: true }), "end_calm 15 instant: {:?}", m.out);
        assert!(m.out.contains(&Cmd::FadeDown { music: false }), "SFXFadeDown(0) [M 0x113558:31]");
        assert!(m.wait_for_end, "MusicWaitForEnd = 1 [M 0x113558:33]");
        // The dispatcher gate [M 0x1153c4-8]: with the end-wait standing a SECOND exit event does
        // nothing — the sting is the board's last request.
        m.out.clear();
        m.event(3, 1);
        m.update(0);
        assert!(m.out.is_empty(), "no updater runs while the end-wait stands: {:?}", m.out);
        // Sister: a combat section playing (the IOP reports 4) picks end_combat 12, not 15.
        let mut m = Music::new(vec![tower_a()]);
        m.event(0, 0x0700_0009); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.last_jump = 4;
        m.event(3, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 12, instant: true }), "end_combat 12 instant: {:?}", m.out);
        assert!(!m.out.contains(&Cmd::Jump { section: 15, instant: true }), "never end_calm from a combat section");
    }

    /// Sister (the death/fail exit keeps its own path): event 6 (the death pulse, death.rs's
    /// `Music_Event(6,1)` [M 0x190810:41]) jumps `on_death`, NOT the exit block's end marker, on
    /// the same board — and event 7 (the mission-fail pulse [M 0x182c60:31]) jumps `on_fail`.
    #[test]
    fn the_09_death_and_fail_events_keep_their_own_markers() {
        let mut m = Music::new(vec![tower_a()]);
        m.event(0, 0x0700_0009); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(6, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 14, instant: true }), "on_death 14 instant: {:?}", m.out);
        assert!(m.out.contains(&Cmd::FadeDown { music: false }) && m.wait_for_end, "the death arm fades down and waits too [M 0x114e70's e[6] arm]");
        assert!(!m.out.iter().any(|c| matches!(c, Cmd::Jump { section: 15, .. }) || matches!(c, Cmd::Jump { section: 12, .. })), "no exit marker on death");
        let mut m = Music::new(vec![tower_a()]);
        m.event(0, 0x0700_0009); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(7, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 13, instant: true }), "on_fail 13 instant: {:?}", m.out);
        assert!(!m.out.iter().any(|c| matches!(c, Cmd::Jump { section: 15, .. }) || matches!(c, Cmd::Jump { section: 12, .. })), "no exit marker on fail");
    }

    /// 1b's MapMusic row [D MapMusic table 0x24f4d8 row 20]: track 35, start 4, end_calm 10, EMPTY
    /// combat/stealth lists.
    fn space_station() -> MapMusic {
        MapMusic { level: 0x0700_001b, track: 35, start: 4, on_death: 3, on_fail: 2, end_combat: 10, end_calm: 10, alert_frames: 10, calm_frames: 160, combat: vec![], stealth: vec![] }
    }

    /// `UpdateMusicalEvents` 0x114e70:61-64: while `SFXPaused` stands (the pause page's `SFXPause`
    /// op 9 [M 0x116b70:11-13; `MenuManager_Create` 0x21b908:121-124 pause_sound=1 — the pause
    /// entry's music calls]), every music event is frozen and PERSISTS. The one
    /// exception is the level-start restart, which runs before the gate [M 0x114e70:43-54].
    /// Sister: unpaused, the same events process.
    #[test]
    fn the_pause_freezes_the_music_board_but_the_level_start_still_restarts() {
        let mut m = Music::new(vec![space_station()]);
        m.event(0, 0x0700_001b); m.event(1, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Start { track: 35, section: 4 }), "the level start starts the track: {:?}", m.out);
        // The pause page's op 9: SFXPaused stands.
        m.sfx_paused = true;
        m.out.clear();
        // The alert/combat pair (e[8], the port's slot for DAT_002d3518) and the intro cinema's
        // end hash (e[0xc], DAT_002d3528): both frozen while paused.
        m.event(8, 1); m.event(0xc, 0x6000_085c);
        m.update(0);
        assert!(m.out.is_empty(), "paused the music board emits nothing: {:?}", m.out);
        assert_eq!(m.events[8], 1, "paused the events persist (the game's return skips the clear) [M 0x114e70:61-64]");
        assert!(!m.wait_for_end, "paused the alert arm never ran");
        // The exception: the level-start restart still runs while paused [M 0x114e70:43-54].
        m.event(0, 0x0700_001b); m.event(1, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Start { track: 35, section: 4 }), "the level start restarts while paused: {:?}", m.out);
        // Unpaused: the same persisted events process now.
        m.sfx_paused = false;
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 10, instant: true }), "unpaused the alert arm jumps end_calm 10: {:?}", m.out);
        assert!(m.out.contains(&Cmd::FadeDown { music: false }), "unpaused the alert arm fades down [M 0x114d60:35-37]");
        assert!(m.wait_for_end, "unpaused the alert arm sets the end-wait");
    }

    /// `UpdateMFX_HT_Level_SpaceStationD` 0x114d60 (0700001b): the intro cinema's end
    /// (`Script_KillStream` of 0x0600085c [M 0x1d9860:74] → DAT_002d3528 = e[0xc]) restarts the
    /// track at section 0, NOT instant, while no end-wait holds [M 0x114d60:40-43]; the alert pair
    /// (e[8]/e[3]) jumps end_calm 10 instant with the empty combat list and holds the end-wait.
    /// Sister: another level's board (05, whose updater is CastleA 0x112d78) does nothing on the
    /// same events.
    #[test]
    fn space_station_d_restarts_the_music_when_the_intro_cinema_ends() {
        let mut m = Music::new(vec![space_station()]);
        m.event(0, 0x0700_001b); m.event(1, 1);
        m.update(0);
        m.out.clear();
        // The intro cinema 0x0600085c ends (SP #147's kill-stream hash).
        m.event(0xc, 0x6000_085c);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0, instant: false }), "the cinema end restarts at section 0, not instant: {:?}", m.out);
        assert_eq!(m.last_request, 0, "MusicLastJumpRequest = 0 [M 0x114d60:41]");
        // The alert pair with the EMPTY combat list: the scan misses → end_calm 10, instant.
        let mut m = Music::new(vec![space_station()]);
        m.event(0, 0x0700_001b); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(8, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 10, instant: true }) && m.out.contains(&Cmd::FadeDown { music: false }), "the alert arm: end_calm 10 instant + fade down: {:?}", m.out);
        assert!(m.wait_for_end);
        // With the end-wait standing the cinema-end restart is suppressed [M 0x114d60:40].
        m.out.clear();
        m.event(0xc, 0x6000_085c);
        m.update(0);
        assert!(!m.out.iter().any(|c| matches!(c, Cmd::Jump { section: 0, .. })), "no restart while the end-wait holds: {:?}", m.out);
        // Sister: 05's board ignores 1b's events (its updater is CastleA).
        let mut m = Music::new(vec![castle()]);
        m.event(0, 0x0700_0005); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(0xc, 0x6000_085c); m.event(8, 1);
        m.update(0);
        assert!(!m.out.iter().any(|c| matches!(c, Cmd::Jump { section: 0, .. })), "05 does not restart at section 0 on 1b's cinema hash");
    }

    // ---- The HT-level updaters (docs/systems/ht-music-updaters.md). Synthetic MapMusic rows:
    // ---- the marker numbers the assertions pin are the decompile's jump targets [M]; the row
    // ---- fields the machines don't read are test furniture.

    fn tower2_b_row() -> MapMusic {
        MapMusic { level: 0x0700_0012, track: 30, start: 0, on_death: 14, on_fail: 13, end_combat: 0x1a, end_calm: 0x1b, alert_frames: 10, calm_frames: 160, combat: vec![0xf, 0x16], stealth: vec![0x10] }
    }
    fn tower2_a_row() -> MapMusic {
        MapMusic { level: 0x0700_0011, track: 30, start: 0, on_death: 14, on_fail: 13, end_combat: 0x1a, end_calm: 0x1c, alert_frames: 10, calm_frames: 160, combat: vec![0xf], stealth: vec![0x10] }
    }
    fn tower2_c_row() -> MapMusic {
        MapMusic { level: 0x0700_0013, track: 30, start: 0, on_death: 14, on_fail: 13, end_combat: 0x1a, end_calm: 0x1c, alert_frames: 10, calm_frames: 160, combat: vec![0xf], stealth: vec![0x10] }
    }
    fn evil_base_row() -> MapMusic {
        MapMusic { level: 0x0700_0014, track: 30, start: 0, on_death: 14, on_fail: 13, end_combat: 0x1a, end_calm: 0x1b, alert_frames: 10, calm_frames: 160, combat: vec![0xf, 0x2e], stealth: vec![0x10] }
    }
    fn evil_silo_row() -> MapMusic {
        MapMusic { level: 0x0700_0015, track: 30, start: 0, on_death: 14, on_fail: 13, end_combat: 0x1a, end_calm: 0x1b, alert_frames: 10, calm_frames: 160, combat: vec![0xf, 0x16], stealth: vec![0x10] }
    }
    fn evil_base_c_row() -> MapMusic {
        MapMusic { level: 0x0700_0016, track: 30, start: 0, on_death: 14, on_fail: 13, end_combat: 0x1a, end_calm: 0x1b, alert_frames: 10, calm_frames: 160, combat: vec![0xf], stealth: vec![0x10] }
    }
    fn elevator_row() -> MapMusic {
        MapMusic { level: 0x0700_004a, track: 30, start: 0, on_death: 14, on_fail: 13, end_combat: 0x1a, end_calm: 0x1b, alert_frames: 10, calm_frames: 160, combat: vec![0xf], stealth: vec![0x10] }
    }

    /// `UpdateMFX_HT_Level_Tower2B` 0x114348:36-76: attack in state 0 jumps 0xf INSTANT and
    /// latches state 1 [M 0x114348:37-42]; calm in state 1 jumps 0x11 non-instant and latches 2
    /// [M :43-48]; calm in state 2 jumps 0x16 only when `MusicLastJump` is in the combat set
    /// [M :49-66]. Sister: state 2 calm with the IOP OUTSIDE the set requests nothing.
    #[test]
    fn tower2b_alarm_machine_walks_0f_11_16() {
        let mut m = Music::new(vec![tower2_b_row()]);
        m.event(0, 0x0700_0012); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(2, 2);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0xf, instant: true }), "state 0 + attack → 0xf instant: {:?}", m.out);
        m.out.clear();
        // The alarm decays (bonus 600 + calm_frames 160 [M 0x114e70]); state 1 waits for calm,
        // then jumps 0x11 non-instant.
        for _ in 0..800 { m.update(0); }
        assert!(!m.alarm_state().0, "the alarm decayed");
        assert!(m.out.contains(&Cmd::Jump { section: 0x11, instant: false }), "state 1 + calm → 0x11: {:?}", m.out);
        m.out.clear();
        // The IOP now reports the section it jumped to (0xf) — `MusicLastJump` is the IOP's
        // per-frame report [M 0x114e70:66-67 `MusicLastJump = DAT_002500cc`], so the test feeds
        // it through `update(iop_section)` like the game's frame does.
        m.update(0xf);
        assert!(m.out.contains(&Cmd::Jump { section: 0x16, instant: false }), "state 2, IOP in combat → 0x16: {:?}", m.out);
        m.out.clear();
        // Sister: the IOP outside the combat set → nothing (2B has no stealth pick in state 2).
        m.update(0x11);
        assert!(m.out.iter().all(|c| !matches!(c, Cmd::Jump { .. })), "state 2, IOP outside combat: nothing: {:?}", m.out);
    }

    /// `UpdateMFX_HT_Level_Tower2C` 0x114640:9-12: the alarm jump (0, non-instant) is queued
    /// BEFORE the exit block, so on a frame with both the alarm and the LOAD_LEVEL exit (e[3])
    /// the LAST jump is the end marker [M order]. Sister: without the alarm the same frame has
    /// no jump(0) at all.
    #[test]
    fn tower2c_alarm_jump_precedes_the_exit_jump() {
        let mut m = Music::new(vec![tower2_c_row()]);
        m.event(0, 0x0700_0013); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(2, 2); m.event(3, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0, instant: false }), "the alarm jump ran: {:?}", m.out);
        let last = m.out.iter().filter(|c| matches!(c, Cmd::Jump { .. })).last().copied();
        assert_eq!(last, Some(Cmd::Jump { section: 0x1c, instant: true }), "the exit jump is LAST (end_calm, instant): {:?}", m.out);
        assert!(m.out.contains(&Cmd::FadeDown { music: false }) && m.wait_for_end, "the exit block held the end-wait: {:?}", m.out);
        // Sister: the same frame without the alarm — no jump(0) in the batch.
        let mut m = Music::new(vec![tower2_c_row()]);
        m.event(0, 0x0700_0013); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(3, 1);
        m.update(0);
        assert!(!m.out.iter().any(|c| matches!(c, Cmd::Jump { section: 0, .. })), "no alarm jump without the alarm: {:?}", m.out);
        assert!(m.out.contains(&Cmd::Jump { section: 0x1c, instant: true }), "the exit jump alone: {:?}", m.out);
    }

    /// `UpdateMFX_HT_Level_Tower2A` 0x114250:9-34 is the shared exit block WHOLE — the exit pair
    /// (e[8] here) jumps the end marker instant + FadeDown + the end-wait, and nothing else ever
    /// queues a jump on this level. Sister: a plain frame with no events emits nothing.
    #[test]
    fn tower2a_is_the_exit_block_alone() {
        let mut m = Music::new(vec![tower2_a_row()]);
        m.event(0, 0x0700_0011); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(8, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0x1c, instant: true }) && m.out.contains(&Cmd::FadeDown { music: false }), "the exit block ran: {:?}", m.out);
        assert!(m.wait_for_end);
        m.out.clear();
        m.update(0);
        assert!(m.out.is_empty(), "no updater output beyond the exit block: {:?}", m.out);
    }

    /// `UpdateMFX_HT_Level_EvilSilo` 0x114a38:41-47: attack round-robins `PickRandomCombat(0)`,
    /// calm picks `PickRandomStealth` once the IOP reports a combat section. Sister: calm with
    /// the IOP outside combat picks nothing [M PickRandomStealth 0x111c70:20-26 gate].
    #[test]
    fn evilsilo_picks_combat_then_stealth() {
        let mut m = Music::new(vec![evil_silo_row()]);
        m.event(0, 0x0700_0015); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(2, 2);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0xf, instant: false }), "attack → combat round 1 (0xf, non-instant flag=0): {:?}", m.out);
        m.out.clear();
        for _ in 0..800 { m.update(0); }
        assert!(!m.alarm_state().0, "the alarm decayed");
        m.update(0xf);
        assert!(m.out.contains(&Cmd::Jump { section: 0x10, instant: false }), "calm + IOP in combat → stealth 0x10: {:?}", m.out);
        m.out.clear();
        // Sister: attack again from the stealth section round-robins to the NEXT combat pick.
        m.event(2, 2);
        m.update(0x10);
        assert!(m.out.contains(&Cmd::Jump { section: 0x16, instant: false }), "attack → combat round 2 (0x16): {:?}", m.out);
    }

    /// `UpdateMFX_HT_Level_Tower2Elevator` 0x114528:36-40: the elevator ride's NIS hash
    /// 0x60006d3 in e[0xb] jumps marker 4 non-instant. Sister: any other hash does nothing.
    #[test]
    fn elevator_nis_jumps_marker_4() {
        let mut m = Music::new(vec![elevator_row()]);
        m.event(0, 0x0700_004a); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(0xb, 0x0600_06d3);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 4, instant: false }), "the ride NIS → marker 4: {:?}", m.out);
        m.out.clear();
        m.event(0xb, 0x0600_0631);
        m.update(0);
        assert!(m.out.is_empty(), "a foreign hash does nothing on 4a: {:?}", m.out);
    }

    /// `UpdateMFX_HT_Level_EvilBaseC` 0x114b38:36-77: the 0x6000631 NIS (e[0xb]) jumps 0x24
    /// instant and latches state 1; the stream-kill 0x6000631 (e[0xc]) jumps 0x27 instant; the
    /// 0x6000632 stream (e[0xb]) jumps 0x29; the exit pair jumps 0x2b and holds the end-wait.
    /// Sister: in state 0 the alarm picks combat/stealth as on EvilSilo.
    #[test]
    fn evilbasec_cutscene_machine_walks_24_27_29_2b() {
        let mut m = Music::new(vec![evil_base_c_row()]);
        m.event(0, 0x0700_0016); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(0xb, 0x0600_0631);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0x24, instant: true }), "the NIS → 0x24 instant: {:?}", m.out);
        m.out.clear();
        m.event(0xc, 0x0600_0631);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0x27, instant: true }), "the kill-stream → 0x27 instant: {:?}", m.out);
        m.out.clear();
        m.event(0xb, 0x0600_0632);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0x29, instant: false }), "the second stream → 0x29: {:?}", m.out);
        m.out.clear();
        m.event(8, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0x2b, instant: false }) && m.wait_for_end, "the exit → 0x2b + end-wait: {:?}", m.out);
        // Sister: state 0 with the alarm picks combat (the machines' shared picker).
        let mut m = Music::new(vec![evil_base_c_row()]);
        m.event(0, 0x0700_0016); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(2, 2);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0xf, instant: false }), "state 0 + attack → combat 0xf: {:?}", m.out);
    }

    /// `UpdateMFX_HT_Level_EvilBase` 0x114728:36-134. The stream-kill hash 0x6000848 in e[0xc]
    /// latches t3 and the tail re-issues jump(0x18, instant) EVERY frame [M :36-40 + :134]; the
    /// 0x600062f scene latches t2, zeroes t1 and jumps 5 instant calm — then the empty-slot arm
    /// queues 0x32 the same frame (the game's two batch ops, last wins on the IOP) [M :120-131].
    /// Sister: after the t2 latch the updater returns early — no more jumps.
    #[test]
    fn evilbase_hold_latch_and_the_062f_scene() {
        let mut m = Music::new(vec![evil_base_row()]);
        m.event(0, 0x0700_0014); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(0xc, 0x0600_0848);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0x18, instant: true }), "the kill-stream hold → 0x18 instant: {:?}", m.out);
        m.out.clear();
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0x18, instant: true }), "the latch holds EVERY frame: {:?}", m.out);
        // The 0x600062f scene (t2): calm jumps 5 instant, then the no-hash arm queues 0x32.
        let mut m = Music::new(vec![evil_base_row()]);
        m.event(0, 0x0700_0014); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(0xb, 0x0600_062f);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 5, instant: true }), "the scene calm arm → 5 instant: {:?}", m.out);
        assert!(m.out.contains(&Cmd::Jump { section: 0x32, instant: false }), "the same frame's no-hash arm → 0x32: {:?}", m.out);
        m.out.clear();
        m.update(0);
        assert!(m.out.is_empty(), "the t2 latch returns early: {:?}", m.out);
    }

    // ---- music-unported-consumers: the audit's tabled unported consumers, on the real rows ----

    /// The real 0b row [D disc MapMusic 0x24f4d8, level 0x0700000b]: empty combat/stealth sets —
    /// TowerC manages its own sections; end_combat == end_calm == 17 (no scan uses them).
    fn tower_c_row() -> MapMusic {
        MapMusic { level: 0x0700_000b, track: 24, start: 0, on_death: 16, on_fail: 15, end_combat: 17, end_calm: 17, alert_frames: 30, calm_frames: 240, combat: vec![], stealth: vec![] }
    }

    /// The real 02 row [D disc MapMusic 0x24f4d8, level 0x07000002].
    fn may_b_row() -> MapMusic {
        MapMusic { level: 0x0700_0002, track: 20, start: 0, on_death: 27, on_fail: 26, end_combat: 24, end_calm: 25, alert_frames: 10, calm_frames: 160, combat: vec![12, 14, 16], stealth: vec![2, 20, 22, 8] }
    }

    /// The real 03 row [D disc MapMusic 0x24f4d8, level 0x07000003].
    fn may_c_row() -> MapMusic {
        MapMusic { level: 0x0700_0003, track: 21, start: 0, on_death: 19, on_fail: 18, end_combat: 16, end_calm: 17, alert_frames: 10, calm_frames: 160, combat: vec![2, 4, 14], stealth: vec![0, 6, 8] }
    }

    /// `UpdateMFX_03TowerC` 0x113a20's full chain, with the cel-zone channels the disc carries
    /// (0700000b's zones are 44 — cels #0-8/#14 — and 41 — cel #13 [D placement param 4]; e[17]
    /// comes from the level's music triggers). Sister: in state 0 an exit/fail event does NOTHING
    /// — 0b has no shared exit block (pre-fix the port ran one and jumped end markers).
    #[test]
    fn the_tower_c_machine_walks_the_nis_and_zone_chain() {
        let mut m = Music::new(vec![tower_c_row()]);
        m.event(0, 0x0700_000b); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(0xb, 0x0600_04f9);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 2, instant: false }], "state 0: the NIS start → 2");
        m.out.clear();
        m.event(44, 1);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 4, instant: false }], "state 1: the zone-44 cel event → 4");
        m.out.clear();
        m.event(41, 1);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 6, instant: false }], "state 2: the zone-41 cel event → 6");
        m.out.clear();
        m.event(17, 1);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 8, instant: true }], "state 3: the e[17] trigger → 8 instant");
        m.out.clear();
        m.event(0xb, 0x0600_04fa);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 0x17, instant: false }], "state 4 calm: the second NIS start → 0x17");
        m.out.clear();
        m.event(0xc, 0x0600_04fa);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 0xc, instant: true }], "state 5: the NIS end → 0xc instant");
        m.out.clear();
        m.event(3, 1);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 0xe, instant: true }], "state 6: the exit → the fixed 0xe, instant");
        assert!(!m.wait_for_end && m.out.len() == 1, "no FadeDown, no end-wait on 0b's exit");
        // Sister: state 0 + exit/fail events → nothing (no shared head), and a second exit in
        // state 99 stays quiet.
        let mut m = Music::new(vec![tower_c_row()]);
        m.event(0, 0x0700_000b); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(3, 1); m.event(8, 1);
        m.update(0);
        assert!(m.out.is_empty(), "0b's updater has no e[3]/e[8] head: {:?}", m.out);
    }

    /// `UpdateMFX_01CastleB` 0x113060's NIS-marker half (the audit's tabled gap): the 0x6000071
    /// NIS start jumps the record's +0x14/+0x18 marker — the +0x24 scan picks — instantly, and
    /// the alarm pick goes silent until the cut-scene's music sound jumps 0xc and holds the
    /// end-wait. Sister arm: the IOP outside the combat set takes end_calm.
    #[test]
    fn the_07000006_nis_marker_half_jumps_the_end_marker_then_the_closing_sting() {
        let mut m = Music::new(vec![castle_b()]);
        m.event(0, 0x0700_0006); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.last_jump = 15;  // the +0x24 scan: 15 ∈ combat [2, 15, 17] → end_combat 8
        m.event(0xb, 0x0600_0071);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 8, instant: true }), "end_combat 8 instant: {:?}", m.out);
        // The alarm stream now lands on a latched updater: no pick, no jump, for 30 ticks.
        m.out.clear();
        let mut saw = false;
        for _ in 0..30 { m.event(2, 1); m.update(0); saw |= !m.out.is_empty(); m.out.clear(); }
        assert!(!saw, "the alarm pick is silent while MusicWaitForNisSfxTrigger stands");
        // The cut-scene's music sound (SfxNisMusicTrigger): 0xc non-instant, the end-wait holds.
        m.nis_trigger = true;
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0xc, instant: false }), "{:?}", m.out);
        assert!(m.wait_for_end, "MusicWaitForEnd = 1 [M 0x113060:52]");
        // Sister: MusicLastJump outside the set → end_calm 6.
        let mut m = Music::new(vec![castle_b()]);
        m.event(0, 0x0700_0006); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(0xb, 0x0600_0071);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 6, instant: true }), "end_calm 6 instant: {:?}", m.out);
    }

    /// `UpdateMFX_02MayhewB` 0x112270: the zone-36 level-start arm, the per-tick gate re-seat,
    /// the e[31] trigger arm (4 instant + `MusicVars = 1` + bonus 600), the ch15 sight pick and
    /// the e[30] tail. (The zone-36 event itself is the live test's — the 02 hall's cel+0xa1.)
    #[test]
    fn the_07000002_machine_level_start_trigger_and_tail() {
        let mut m = Music::new(vec![may_b_row()]);
        m.event(0, 0x0700_0002); m.event(1, 1);
        m.update(0);
        m.out.clear();
        // Level-start arm: the IOP at section 0 and the zone-36 event → 2, non-instant.
        m.event(36, 1);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 2, instant: false }], "e[36] starts section 2");
        // Sister: no 0x6000084 end, no e[36] → quiet.
        let mut m = Music::new(vec![may_b_row()]);
        m.event(0, 0x0700_0002); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.update(0);
        assert!(m.out.is_empty());
        // With the IOP in a section: every tick re-seats the gate (e[37] low → Restore); the
        // e[31] trigger fires 4 INSTANT once, MusicVars = 1, bonus 600, the ch15/16 latch.
        let mut m = Music::new(vec![may_b_row()]);
        m.event(0, 0x0700_0002); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.last_jump = 2;
        m.update(0);
        assert!(m.gate_writes.contains(&GateWrite::Restore), "the per-tick re-seat: {:?}", m.gate_writes);
        m.gate_writes.clear();
        m.out.clear();  // the first in-section tick's stealth round-robin (0x14) is not this pin
        m.event(31, 2);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 4, instant: true }], "e[31] → 4 instant");
        assert!(m.gate_writes.contains(&GateWrite::SetCount(1)) && m.bonus == 600);
        m.out.clear(); m.gate_writes.clear();
        m.update(0);
        assert!(m.out.is_empty(), "the latched ch15/16 wait holds (no sight yet): {:?}", m.out);
        // The sight: e[15] → PickRandomCombat (round-robin over the real combat set) + the 300 gate.
        m.event(15, 1);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 12, instant: false }), "the combat pick: {:?}", m.out);
        m.out.clear();
        // The e[37] tighten: while the channel holds the re-seat is SetDistance(1.0).
        m.event(37, 2);
        m.update(0);
        assert!(m.gate_writes.contains(&GateWrite::SetDistance(1.0)), "{:?}", m.gate_writes);
        m.gate_writes.clear();
        // The e[30] tail (fresh board): outside the combat set → 0xa, then t5 → 0x12 every tick.
        let mut m = Music::new(vec![may_b_row()]);
        m.event(0, 0x0700_0002); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.last_jump = 5;
        m.event(30, 2);
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0xa, instant: false }), "the tail: {:?}", m.out);
        m.out.clear();
        m.update(0);
        assert!(m.out.contains(&Cmd::Jump { section: 0x12, instant: false }), "t5 holds 0x12: {:?}", m.out);
    }

    /// `UpdateMFX_02MayhewC` 0x112730: the e[31]/e[32] trigger pair (0xa / 0xc, once each), the
    /// stealth round-robin's in-set stop, and the attack branches (2/4 round-robin with e[33]
    /// low, 0xe with e[33] high).
    #[test]
    fn the_07000003_machine_triggers_and_round_robins() {
        let mut m = Music::new(vec![may_c_row()]);
        m.event(0, 0x0700_0003); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(31, 2);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 0xa, instant: false }], "e[31] → 0xa");
        assert!(m.gate_writes.contains(&GateWrite::SetCount(1)) && m.bonus == 600);
        m.out.clear(); m.gate_writes.clear();
        m.update(0);
        assert!(m.out.is_empty(), "the t2+t1 latches hold: {:?}", m.out);
        // The sight completes the ch15/16 latch (t1 → 0, the 0x78 gate); past it e[32] fires.
        m.event(15, 1);
        m.update(0);
        assert!(m.out.iter().any(|c| matches!(c, Cmd::Jump { section: 2, .. })), "the combat pick: {:?}", m.out);
        m.out.clear();
        for _ in 0..125 { m.update(0); }
        m.out.clear();  // the gate's expiry tick may queue the calm no-hash jump(0) — not this pin
        m.event(32, 2);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 0xc, instant: false }], "e[32] → 0xc");
        // The calm stealth round-robin: outside the stealth set, IOP in the combat set → 6 (the
        // restart tick took the no-hash start-over arm, which doesn't touch the round-robin); the
        // next tick the request (6) is IN the set → stopped — the alternation needs a combat pick
        // between (the game's design: stealth sections alternate across combat excursions).
        let mut m = Music::new(vec![may_c_row()]);
        m.event(0, 0x0700_0003); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.last_jump = 2;
        m.last_request = -1;
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 6, instant: false }], "the stealth round-robin → 6");
        m.out.clear();
        m.update(0);
        assert!(m.out.is_empty(), "the request 6 is in the stealth set: {:?}", m.out);
        // Attack, e[33] low: outside the combat set on both scans → 2 (the same tick the alarm
        // lands — the machine runs after the hysteresis), then the request (2) sits IN the set:
        // blocked until it leaves, then 4.
        let mut m = Music::new(vec![may_c_row()]);
        m.event(0, 0x0700_0003); m.event(1, 1);
        m.update(0);
        m.out.clear();
        m.event(2, 2);
        m.update(0);
        assert!(m.under_attack);
        assert!(m.out.contains(&Cmd::Jump { section: 2, instant: false }), "the attack round-robin → 2");
        m.out.clear();
        m.update(0);
        assert!(m.out.is_empty(), "the request 2 is in the combat set: {:?}", m.out);
        m.last_request = -1;
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 4, instant: false }], "→ 4");
        // e[33] high: 0xe (the request 4 is in-set too — reset as above).
        m.out.clear();
        m.last_request = -1;
        m.event(33, 2);
        m.update(0);
        assert_eq!(m.out, vec![Cmd::Jump { section: 0xe, instant: false }], "e[33] high → 0xe");
    }


}

//! The keypad's model swap on unlock. `Lock_Unlock` 0x1b0650:23 runs `Control_NextLOD` 0x1349b0 on
//! the lock (obj+0xD8 ← glist+0x40), and `Lock_Update` case 1 runs it again when the open framelist
//! ends [M 0x1ae3d8 case 1]. Every decryptor keypad on the disc is "Lock - KeypadLocked" with the
//! block-0x27 LOD entry "Lock - KeypadUnlocked" [D 0700000a #646/#648/#729/#732, 0700000b #178,
//! 07000011 #177, 07000012 #656/#694, 07000014 #774-776]; the variant's own entry is −1, so the
//! chain ends there and a re-lock's NextLOD [M Lock_ReLock 0x1b0750] leaves it.
//!
//! The draw: the locked model shows until the unlock, the unlocked model from then on. The unlock
//! itself is the decryptor test's (decryptor_hold_real_path_live); here the sim's unlock flag is
//! set the way `Lock_Unlock` sets it, and the next tick's draw is read.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// (verts of the placed object visible, verts of the LOD variant visible) for lock `li`.
fn shown(scene: &nf_view::scene::Scene, li: usize) -> (usize, usize) {
    let s = scene.sim.as_ref().unwrap();
    let base = s.drones.doors.list.len() + s.drones.sss.list.len() + s.drones.breaks.list.len() + 2 * li;
    let posed = scene.posed_vertices(None, 0.0, Default::default());
    let mut n = (0, 0);
    for (v, p) in scene.char_verts.iter().zip(posed.iter()) {
        let up = p.pos[1] > -5000.0;
        match v.door.map(usize::from) {
            Some(d) if d == base && up => n.0 += 1,
            Some(d) if d == base + 1 && up => n.1 += 1,
            _ => {}
        }
    }
    n
}

#[test]
fn the_keypad_shows_its_unlocked_model_once_unlocked() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700000a.bin", false).expect("load 0700000a");
    let li = scene.sim.as_ref().unwrap().drones.locks.iter().position(|l| l.kind == 0 && l.out == 49).expect("the out-49 keypad");
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    let (locked, unlocked) = shown(&scene, li);
    assert!(locked > 0, "the locked keypad draws before the unlock");
    assert_eq!(unlocked, 0, "the unlocked model stays hidden before the unlock");
    scene.sim.as_mut().unwrap().drones.locks[li].unlocked = true;
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    let (locked, unlocked) = shown(&scene, li);
    assert_eq!(locked, 0, "the locked model is swapped out");
    assert!(unlocked > 0, "the unlocked model draws");
    // A re-lock does not swap back (the chain ends at the variant).
    scene.sim.as_mut().unwrap().drones.locks[li].unlocked = false;
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    assert!(shown(&scene, li).1 > 0, "the unlocked model stays after a re-lock");
    // Sister: another keypad, untouched, still shows its locked model.
    let other = scene.sim.as_ref().unwrap().drones.locks.iter().position(|l| l.kind == 0 && l.out == 48).expect("the out-48 keypad");
    assert!(shown(&scene, other).0 > 0 && shown(&scene, other).1 == 0, "an untouched keypad stays locked");
}

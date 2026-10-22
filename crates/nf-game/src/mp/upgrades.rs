//! The MP reward-upgrade table — `Set_Upgrade` 0x1a98a8 and `ResetUpgrade` 0x1a9900, written by
//! the front end's bonus screen `Menu_SetBonus` 0x1ff898 (each reward the processed reward
//! counters yield calls `ResetUpgrade(player)` first and then `Set_Upgrade(player, slot, value)`
//! for reward kind 4 [M 0x1ff898:76,98]).
//!
//! `Upgrades` 0x2beea8 is 36 bytes — 4 players × 9 upgrade bytes [M symtab size; the game indexes
//! `Upgrades[player·9 + slot/3]`, so a player's row is 9 upgrades of 3 sub-slots each]. The disc
//! value is all zeros at level start [L ee_start_05 @ 0x2beea8].

/// The players the table holds [M `Upgrades` 0x2beea8 = 36 bytes / 9 per player].
pub const PLAYERS: usize = 4;
/// The upgrades per player row [M `Set_Upgrade` 0x1a98a8: the `/3 < 9` guard].
pub const ROW: usize = 9;

/// The port's copy of `Upgrades` (all zeros [L ee_start_05 @ 0x2beea8]).
#[derive(Clone, Debug, PartialEq)]
pub struct Upgrades(pub [[u8; ROW]; PLAYERS]);

impl Default for Upgrades {
    fn default() -> Self {
        Upgrades([[0; ROW]; PLAYERS])
    }
}

impl Upgrades {
    /// `Set_Upgrade` 0x1a98a8: `Upgrades[player·9 + slot/3] = value` — but only when
    /// `slot/3 < 9` (the guard the decomp keeps; slot is the reward's u16, value its u8 byte).
    /// `player` is masked to a byte in the game (`param_1 & 0xff`); a player ≥ 4 would write past
    /// the 36-byte table in C — the port refuses instead (no shipped caller can reach that: the
    /// bonus screen iterates 0..4 players [M 0x1ff898]).
    pub fn set(&mut self, player: u8, slot: u16, value: u8) -> bool {
        let p = player as usize;
        let s = (slot / 3) as usize;
        if p >= PLAYERS || s >= ROW {
            return false;
        }
        self.0[p][s] = value;
        true
    }

    /// The row a getter reads (the draw code's view of the bonus grid).
    pub fn row(&self, player: u8) -> Option<&[u8; ROW]> {
        self.0.get(player as usize)
    }

    /// `ResetUpgrade` 0x1a9900: zero the player's 9 entries (the loop runs 9 stores).
    pub fn reset(&mut self, player: u8) -> bool {
        let p = player as usize;
        if p >= PLAYERS {
            return false;
        }
        self.0[p] = [0; ROW];
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [M 0x1a98a8] the index math: `player·9 + slot/3`, guarded by `slot/3 < 9`.
    #[test]
    fn set_indexes_row_nine_with_the_slot_divided_by_three() {
        let mut u = Upgrades::default();
        assert!(u.set(2, 7, 0xab)); // 7/3 = 2
        assert_eq!(u.0[2][2], 0xab);
        assert_eq!(u.0[2][1], 0);
        assert!(u.set(0, 24, 3)); // 24/3 = 8 — the last valid slot
        assert_eq!(u.0[0][8], 3);
        // The guard: slot 27 → 27/3 = 9 — out of the row, refused.
        assert!(!u.set(0, 27, 9));
        assert_eq!(u.0[0][8], 3);
        // Sister (can fail): slot 25 (25/3 = 8) is still the last cell — not refused.
        assert!(u.set(0, 25, 5));
        assert_eq!(u.0[0][8], 5);
    }

    // [M 0x1a9900] the reset zeroes exactly the player's 9 bytes.
    #[test]
    fn reset_zeroes_only_that_players_row() {
        let mut u = Upgrades::default();
        assert!(u.set(1, 0, 7));
        assert!(u.set(1, 26, 9));
        assert!(u.set(3, 3, 5));
        assert!(u.reset(1));
        assert!(u.0[1].iter().all(|&b| b == 0));
        assert_eq!(u.0[3][1], 5);
        // Sister (can fail): resetting a row does not touch the neighbours (a stride bug would).
        assert_ne!(u.0[3], [0; ROW]);
    }

    // [L ee_start_05 @ 0x2beea8] the disc table starts zeroed.
    #[test]
    fn the_table_starts_zeroed_for_four_players() {
        let u = Upgrades::default();
        assert_eq!(u.0, [[0; ROW]; PLAYERS]);
        assert_eq!(PLAYERS, 4);
        // Sister (can fail): 4 rows of 9 = the game's 36 bytes — 5 players would be 45.
        assert_eq!(PLAYERS * ROW, 36);
    }
}

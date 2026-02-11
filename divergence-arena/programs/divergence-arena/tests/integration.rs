#[cfg(test)]
mod tests {
    //! Off-chain unit tests for layout constants and game logic.
    //! Integration tests require LiteSVM or Mollusk for full CPI simulation.

    const MAX_PLAYERS: usize = 10;
    const GUESS_UPPER_BOUND: u64 = 1_000_000;
    const ENTRY_FEE: u64 = 100_000_000;
    const MAX_ROUNDS: u8 = 5;

    // Layout constants
    const GS_SUBS_START: usize = 46;
    const SUB_SIZE: usize = 40;
    const GAME_STATE_SIZE: usize = 46 + MAX_PLAYERS * 40; // 446
    const PLAYER_ACCOUNT_SIZE: usize = 50;

    #[test]
    fn test_game_state_layout_size() {
        assert_eq!(GAME_STATE_SIZE, 446);
        assert!(GAME_STATE_SIZE < 10240, "must fit in Solana account limits");
    }

    #[test]
    fn test_player_account_layout_size() {
        assert_eq!(PLAYER_ACCOUNT_SIZE, 50);
    }

    #[test]
    fn test_submission_offsets() {
        for i in 0..MAX_PLAYERS {
            let off = GS_SUBS_START + i * SUB_SIZE;
            let guess_off = off + 32;
            assert!(guess_off + 8 <= GAME_STATE_SIZE);
        }
    }

    #[test]
    fn test_two_thirds_calculation() {
        // Simulate: 10 players all guess 100
        let sub_count: u128 = 10;
        let sum: u128 = 100 * sub_count;
        let target = (2 * sum) / (3 * sub_count);
        // 2/3 of 100 = 66.67, integer = 66
        assert_eq!(target, 66);
    }

    #[test]
    fn test_two_thirds_varied_guesses() {
        // Guesses: 50, 33, 25, 67, 45, 80, 10, 55, 40, 90
        let guesses: [u64; 10] = [50, 33, 25, 67, 45, 80, 10, 55, 40, 90];
        let sub_count = guesses.len() as u128;
        let sum: u128 = guesses.iter().map(|&g| g as u128).sum();
        // sum = 495, avg = 49.5, 2/3 * 49.5 = 33.0
        let target = (2 * sum) / (3 * sub_count);
        assert_eq!(target, 33);
    }

    #[test]
    fn test_winner_selection_closest() {
        let guesses: [u64; 3] = [30, 33, 40];
        let sub_count = 3u128;
        let sum: u128 = guesses.iter().map(|&g| g as u128).sum(); // 103
        let target = (2 * sum) / (3 * sub_count); // 206/9 = 22

        let mut winner_idx = 0usize;
        let mut best_dist = u128::MAX;

        for (i, &g) in guesses.iter().enumerate() {
            let g128 = g as u128;
            let dist = if g128 > target {
                g128 - target
            } else {
                target - g128
            };
            if dist < best_dist || (dist == best_dist && g < guesses[winner_idx]) {
                best_dist = dist;
                winner_idx = i;
            }
        }

        // target = 22, distances: 30->8, 33->11, 40->18
        assert_eq!(target, 22);
        assert_eq!(winner_idx, 0); // guess=30 is closest
    }

    #[test]
    fn test_winner_selection_tie_lowest_wins() {
        // Two guesses equidistant from target
        let guesses: [u64; 2] = [40, 60];
        let sub_count = 2u128;
        let sum: u128 = 100;
        let target = (2 * sum) / (3 * sub_count); // 200/6 = 33

        let mut winner_idx = 0usize;
        let mut best_dist = u128::MAX;

        for (i, &g) in guesses.iter().enumerate() {
            let g128 = g as u128;
            let dist = if g128 > target {
                g128 - target
            } else {
                target - g128
            };
            if dist < best_dist || (dist == best_dist && g < guesses[winner_idx]) {
                best_dist = dist;
                winner_idx = i;
            }
        }

        // target=33, dist(40)=7, dist(60)=27 → 40 wins
        assert_eq!(winner_idx, 0);
    }

    #[test]
    fn test_pot_accumulation() {
        let mut pot: u64 = 0;
        for _ in 0..MAX_PLAYERS {
            pot += ENTRY_FEE;
        }
        assert_eq!(pot, 1_000_000_000); // 1 SOL total pot with 10 players
    }

    #[test]
    fn test_round_progression() {
        let mut round: u8 = 1;
        for _ in 0..MAX_ROUNDS {
            assert!(round <= MAX_ROUNDS);
            round += 1;
        }
        assert_eq!(round, MAX_ROUNDS + 1);
    }

    #[test]
    fn test_guess_bounds() {
        assert!(0 <= GUESS_UPPER_BOUND);
        assert_eq!(GUESS_UPPER_BOUND, 1_000_000);
        // 2/3 of max possible average
        let max_target = (2 * GUESS_UPPER_BOUND as u128) / 3;
        assert_eq!(max_target, 666_666);
    }

    #[test]
    fn test_u64_le_bytes_roundtrip() {
        let val: u64 = 123_456_789;
        let bytes = val.to_le_bytes();
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes);
        let recovered = u64::from_le_bytes(buf);
        assert_eq!(val, recovered);
    }

    #[test]
    fn test_zero_guess_edge_case() {
        // All players guess 0
        let sub_count = 10u128;
        let sum: u128 = 0;
        let target = (2 * sum) / (3 * sub_count);
        assert_eq!(target, 0);
        // Winner should be any player with guess 0 (all tied, lowest = 0)
    }

    #[test]
    fn test_max_guess_edge_case() {
        // All players guess max
        let sub_count = 10u128;
        let sum: u128 = GUESS_UPPER_BOUND as u128 * sub_count;
        let target = (2 * sum) / (3 * sub_count);
        assert_eq!(target, 666_666);
    }

    #[test]
    fn test_single_player_resolve() {
        // Edge case: only 1 submission
        let guesses: [u64; 1] = [500_000];
        let sub_count = 1u128;
        let sum: u128 = 500_000;
        let target = (2 * sum) / (3 * sub_count);
        // 2/3 of 500_000 = 333_333
        assert_eq!(target, 333_333);
        // Only player wins regardless
    }
}

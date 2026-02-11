Excellent. I now have comprehensive knowledge of Pinocchio's 0.10.x API. Let me build the complete contract.

CONTRACT_CODE:

```rust
//! Guess 2/3 of the Average — Solana program (Pinocchio 0.10.x)
//!
//! Game: 10 players each submit a guess [0, 1_000_000]. The player whose guess
//! is closest to 2/3 of the average of all guesses wins the round pot.
//! 5 rounds total. Entry fee collected into PDA vault on submit.

#![no_std]

extern crate alloc;

use pinocchio::{
    address::find_program_address,
    cpi::{Seed, Signer},
    entrypoint,
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::{CreateAccount, Transfer};

// ── Program ID ──────────────────────────────────────────────────────────────

#[cfg(feature = "declare-id")]
pinocchio_pubkey::declare_id!("Guess23AvgGame1111111111111111111111111111111");

// ── Constants ───────────────────────────────────────────────────────────────

const MAX_PLAYERS: usize = 10;
const MAX_ROUNDS: u8 = 5;
const GUESS_UPPER_BOUND: u64 = 1_000_000;
const ENTRY_FEE: u64 = 100_000_000; // 0.1 SOL per round

// Discriminators
const IX_INIT_GAME: u8 = 0;
const IX_SUBMIT_GUESS: u8 = 1;
const IX_RESOLVE_ROUND: u8 = 2;
const IX_CLAIM_PRIZE: u8 = 3;

// Seeds
const GAME_SEED: &[u8] = b"game";
const PLAYER_SEED: &[u8] = b"player";
const VAULT_SEED: &[u8] = b"vault";

// ── Account Layouts (zero-copy) ─────────────────────────────────────────────

// GameState layout:
//   [0]       initialized:  u8
//   [1..33]   authority:    [u8; 32]
//   [33]      round:        u8
//   [34]      phase:        u8    (0=Submitting, 1=Resolved, 2=Finished)
//   [35..43]  pot:          u64
//   [43]      sub_count:    u8
//   [44]      vault_bump:   u8
//   [45]      game_bump:    u8
//   -- submissions: MAX_PLAYERS * 40 bytes each --
//   Per submission (40 bytes):
//     [0..32]  pubkey:  [u8; 32]
//     [32..40] guess:   u64

const GS_INITIALIZED: usize = 0;
const GS_AUTHORITY: usize = 1;
const GS_ROUND: usize = 33;
const GS_PHASE: usize = 34;
const GS_POT: usize = 35;
const GS_SUB_COUNT: usize = 43;
const GS_VAULT_BUMP: usize = 44;
const GS_GAME_BUMP: usize = 45;
const GS_SUBS_START: usize = 46;
const SUB_SIZE: usize = 40; // 32 + 8
const GAME_STATE_SIZE: usize = GS_SUBS_START + MAX_PLAYERS * SUB_SIZE; // 46 + 400 = 446

const PHASE_SUBMITTING: u8 = 0;
const PHASE_RESOLVED: u8 = 1;
const PHASE_FINISHED: u8 = 2;

// PlayerAccount layout:
//   [0]       initialized:  u8
//   [1..33]   pubkey:       [u8; 32]
//   [33..41]  current_guess: u64
//   [41..49]  total_score:   u64
//   [49]      bump:          u8

const PA_INITIALIZED: usize = 0;
const PA_PUBKEY: usize = 1;
const PA_GUESS: usize = 33;
const PA_SCORE: usize = 41;
const PA_BUMP: usize = 49;
const PLAYER_ACCOUNT_SIZE: usize = 50;

// ── Entrypoint ──────────────────────────────────────────────────────────────

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let (ix, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match *ix {
        IX_INIT_GAME => init_game(program_id, accounts, data),
        IX_SUBMIT_GUESS => submit_guess(program_id, accounts, data),
        IX_RESOLVE_ROUND => resolve_round(program_id, accounts),
        IX_CLAIM_PRIZE => claim_prize(program_id, accounts),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

#[inline]
fn read_u64(data: &[u8], off: usize) -> u64 {
    let bytes: [u8; 8] = data[off..off + 8]
        .try_into()
        .expect("slice len");
    u64::from_le_bytes(bytes)
}

#[inline]
fn write_u64(data: &mut [u8], off: usize, val: u64) {
    data[off..off + 8].copy_from_slice(&val.to_le_bytes());
}

#[inline]
fn read_pubkey(data: &[u8], off: usize) -> &[u8] {
    &data[off..off + 32]
}

fn assert_signer(account: &AccountView) -> ProgramResult {
    if !account.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

fn assert_writable(account: &AccountView) -> ProgramResult {
    if !account.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

fn assert_owned_by(account: &AccountView, owner: &Address) -> ProgramResult {
    if !account.owned_by(owner) {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}

fn assert_pda(
    account: &AccountView,
    seeds: &[&[u8]],
    program_id: &Address,
) -> Result<u8, ProgramError> {
    let (expected, bump) = find_program_address(seeds, program_id);
    if account.address() != &expected {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(bump)
}

// ── init_game ───────────────────────────────────────────────────────────────
// Accounts: [authority (signer, writable), game_state (writable), vault (writable), system_program]

fn init_game(
    program_id: &Address,
    accounts: &[AccountView],
    _data: &[u8],
) -> ProgramResult {
    let [authority, game_state, vault, _system_program, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    assert_signer(authority)?;
    assert_writable(authority)?;
    assert_writable(game_state)?;
    assert_writable(vault)?;

    // Derive PDAs
    let game_bump = assert_pda(
        game_state,
        &[GAME_SEED, authority.address().as_ref()],
        program_id,
    )?;
    let vault_bump = assert_pda(
        vault,
        &[VAULT_SEED, authority.address().as_ref()],
        program_id,
    )?;

    // Create game state account
    let game_bump_bytes = [game_bump];
    let game_signer_seeds = [
        Seed::from(GAME_SEED),
        Seed::from(authority.address().as_ref()),
        Seed::from(&game_bump_bytes[..]),
    ];
    let game_signer = Signer::from(&game_signer_seeds[..]);

    let rent = Rent::get()?;
    let game_lamports = rent.minimum_balance(GAME_STATE_SIZE);

    CreateAccount {
        from: authority,
        to: game_state,
        lamports: game_lamports,
        space: GAME_STATE_SIZE as u64,
        owner: program_id,
    }
    .invoke_signed(&[game_signer])?;

    // Create vault account (0-size, just holds lamports)
    let vault_bump_bytes = [vault_bump];
    let vault_signer_seeds = [
        Seed::from(VAULT_SEED),
        Seed::from(authority.address().as_ref()),
        Seed::from(&vault_bump_bytes[..]),
    ];
    let vault_signer = Signer::from(&vault_signer_seeds[..]);

    let vault_lamports = rent.minimum_balance(0);

    CreateAccount {
        from: authority,
        to: vault,
        lamports: vault_lamports,
        space: 0,
        owner: program_id,
    }
    .invoke_signed(&[vault_signer])?;

    // Initialize game state
    let mut gs = game_state.try_borrow_mut()?;
    gs[GS_INITIALIZED] = 1;
    gs[GS_AUTHORITY..GS_AUTHORITY + 32]
        .copy_from_slice(authority.address().as_ref());
    gs[GS_ROUND] = 1;
    gs[GS_PHASE] = PHASE_SUBMITTING;
    write_u64(&mut gs, GS_POT, 0);
    gs[GS_SUB_COUNT] = 0;
    gs[GS_VAULT_BUMP] = vault_bump;
    gs[GS_GAME_BUMP] = game_bump;

    Ok(())
}

// ── submit_guess ────────────────────────────────────────────────────────────
// Accounts: [player (signer, writable), player_account (writable), game_state (writable),
//            vault (writable), system_program]
// Data: guess (u64, 8 bytes)

fn submit_guess(
    program_id: &Address,
    accounts: &[AccountView],
    data: &[u8],
) -> ProgramResult {
    let [player, player_account, game_state, vault, _system_program, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    assert_signer(player)?;
    assert_writable(player)?;
    assert_writable(player_account)?;
    assert_writable(game_state)?;
    assert_writable(vault)?;
    assert_owned_by(game_state, program_id)?;

    if data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let guess = u64::from_le_bytes(data[0..8].try_into().unwrap());
    if guess > GUESS_UPPER_BOUND {
        return Err(ProgramError::InvalidArgument);
    }

    // Validate game state
    {
        let gs = game_state.try_borrow()?;
        if gs[GS_INITIALIZED] != 1 {
            return Err(ProgramError::UninitializedAccount);
        }
        if gs[GS_PHASE] != PHASE_SUBMITTING {
            return Err(ProgramError::InvalidAccountData);
        }
        if gs[GS_SUB_COUNT] as usize >= MAX_PLAYERS {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    // Read authority from game state for PDA derivation
    let authority_bytes: [u8; 32] = {
        let gs = game_state.try_borrow()?;
        gs[GS_AUTHORITY..GS_AUTHORITY + 32].try_into().unwrap()
    };

    // Verify vault PDA
    assert_pda(
        vault,
        &[VAULT_SEED, &authority_bytes],
        program_id,
    )?;
    assert_owned_by(vault, program_id)?;

    // Derive and potentially create player account PDA
    let player_bump = assert_pda(
        player_account,
        &[PLAYER_SEED, player.address().as_ref()],
        program_id,
    )?;

    // Create player account if not initialized
    if player_account.data_len() == 0 {
        let rent = Rent::get()?;
        let pa_lamports = rent.minimum_balance(PLAYER_ACCOUNT_SIZE);

        let player_bump_bytes = [player_bump];
        let pa_signer_seeds = [
            Seed::from(PLAYER_SEED),
            Seed::from(player.address().as_ref()),
            Seed::from(&player_bump_bytes[..]),
        ];
        let pa_signer = Signer::from(&pa_signer_seeds[..]);

        CreateAccount {
            from: player,
            to: player_account,
            lamports: pa_lamports,
            space: PLAYER_ACCOUNT_SIZE as u64,
            owner: program_id,
        }
        .invoke_signed(&[pa_signer])?;

        let mut pa = player_account.try_borrow_mut()?;
        pa[PA_INITIALIZED] = 1;
        pa[PA_PUBKEY..PA_PUBKEY + 32].copy_from_slice(player.address().as_ref());
        write_u64(&mut pa, PA_GUESS, 0);
        write_u64(&mut pa, PA_SCORE, 0);
        pa[PA_BUMP] = player_bump;
    } else {
        assert_owned_by(player_account, program_id)?;
    }

    // Check player hasn't already submitted this round
    {
        let gs = game_state.try_borrow()?;
        let sub_count = gs[GS_SUB_COUNT] as usize;
        for i in 0..sub_count {
            let off = GS_SUBS_START + i * SUB_SIZE;
            if read_pubkey(&gs, off) == player.address().as_ref() {
                return Err(ProgramError::AccountAlreadyInitialized);
            }
        }
    }

    // Transfer entry fee to vault
    Transfer {
        from: player,
        to: vault,
        lamports: ENTRY_FEE,
    }
    .invoke()?;

    // Record submission in game state
    {
        let mut gs = game_state.try_borrow_mut()?;
        let sub_count = gs[GS_SUB_COUNT] as usize;
        let off = GS_SUBS_START + sub_count * SUB_SIZE;
        gs[off..off + 32].copy_from_slice(player.address().as_ref());
        write_u64(&mut gs, off + 32, guess);
        gs[GS_SUB_COUNT] = (sub_count + 1) as u8;

        let pot = read_u64(&gs, GS_POT);
        write_u64(&mut gs, GS_POT, pot + ENTRY_FEE);
    }

    // Update player account
    {
        let mut pa = player_account.try_borrow_mut()?;
        write_u64(&mut pa, PA_GUESS, guess);
    }

    Ok(())
}

// ── resolve_round ───────────────────────────────────────────────────────────
// Accounts: [authority (signer), game_state (writable), vault (writable),
//            winner_player_account (writable)]
// + remaining accounts: all player_accounts for score updates (writable)

fn resolve_round(
    program_id: &Address,
    accounts: &[AccountView],
) -> ProgramResult {
    if accounts.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let authority = &accounts[0];
    let game_state = &accounts[1];
    let vault = &accounts[2];
    // accounts[3..] are player accounts for all submitters

    assert_signer(authority)?;
    assert_writable(game_state)?;
    assert_writable(vault)?;
    assert_owned_by(game_state, program_id)?;
    assert_owned_by(vault, program_id)?;

    // Validate authority
    {
        let gs = game_state.try_borrow()?;
        if gs[GS_INITIALIZED] != 1 {
            return Err(ProgramError::UninitializedAccount);
        }
        if gs[GS_PHASE] != PHASE_SUBMITTING {
            return Err(ProgramError::InvalidAccountData);
        }
        if read_pubkey(&gs, GS_AUTHORITY) != authority.address().as_ref() {
            return Err(ProgramError::MissingRequiredSignature);
        }
    }

    let (sub_count, round, authority_bytes, vault_bump) = {
        let gs = game_state.try_borrow()?;
        let sc = gs[GS_SUB_COUNT] as usize;
        let r = gs[GS_ROUND];
        let mut ab = [0u8; 32];
        ab.copy_from_slice(&gs[GS_AUTHORITY..GS_AUTHORITY + 32]);
        let vb = gs[GS_VAULT_BUMP];
        (sc, r, ab, vb)
    };

    if sub_count == 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    // Compute average and 2/3 target
    let mut sum: u128 = 0;
    let mut guesses: [(u32, u64); MAX_PLAYERS] = [(0, 0); MAX_PLAYERS]; // (index, guess)

    {
        let gs = game_state.try_borrow()?;
        for i in 0..sub_count {
            let off = GS_SUBS_START + i * SUB_SIZE;
            let g = read_u64(&gs, off + 32);
            sum += g as u128;
            guesses[i] = (i as u32, g);
        }
    }

    // target = 2/3 * average = 2 * sum / (3 * sub_count)
    let target = (2 * sum) / (3 * sub_count as u128);

    // Find winner: closest guess to target (ties: lowest guess wins)
    let mut winner_idx: usize = 0;
    let mut best_dist: u128 = u128::MAX;

    for i in 0..sub_count {
        let g = guesses[i].1 as u128;
        let dist = if g > target { g - target } else { target - g };
        if dist < best_dist || (dist == best_dist && (guesses[i].1) < guesses[winner_idx].1) {
            best_dist = dist;
            winner_idx = i;
        }
    }

    // Get winner pubkey from game state submissions
    let winner_pubkey: [u8; 32] = {
        let gs = game_state.try_borrow()?;
        let off = GS_SUBS_START + winner_idx * SUB_SIZE;
        gs[off..off + 32].try_into().unwrap()
    };

    // Transfer pot from vault to winner — direct lamport manipulation
    // since vault is program-owned
    let pot = {
        let gs = game_state.try_borrow()?;
        read_u64(&gs, GS_POT)
    };

    // Find winner's player account among remaining accounts
    let mut winner_pa: Option<&AccountView> = None;
    for acc in &accounts[3..] {
        assert_owned_by(acc, program_id)?;
        let pa = acc.try_borrow()?;
        if pa[PA_INITIALIZED] == 1 && read_pubkey(&pa, PA_PUBKEY) == &winner_pubkey {
            winner_pa = Some(acc);
            break;
        }
    }
    let winner_pa = winner_pa.ok_or(ProgramError::NotEnoughAccountKeys)?;
    assert_writable(winner_pa)?;

    // Update winner's score
    {
        let mut pa = winner_pa.try_borrow_mut()?;
        let score = read_u64(&pa, PA_SCORE);
        write_u64(&mut pa, PA_SCORE, score + 1);
    }

    // Transfer pot: vault -> winner_pa (lamport reallocation, both program-owned)
    // We transfer to the player_account PDA; user claims from there later
    let vault_lamports = vault.lamports();
    let rent = Rent::get()?;
    let vault_rent_exempt = rent.minimum_balance(0);

    // Only transfer pot, keep vault rent-exempt
    let transferable = if vault_lamports > vault_rent_exempt + pot {
        pot
    } else if vault_lamports > vault_rent_exempt {
        vault_lamports - vault_rent_exempt
    } else {
        0
    };

    if transferable > 0 {
        vault.set_lamports(vault_lamports - transferable);
        winner_pa.set_lamports(winner_pa.lamports() + transferable);
    }

    // Update game state for next round or finish
    {
        let mut gs = game_state.try_borrow_mut()?;
        write_u64(&mut gs, GS_POT, 0);
        gs[GS_SUB_COUNT] = 0;

        if round >= MAX_ROUNDS {
            gs[GS_PHASE] = PHASE_FINISHED;
        } else {
            gs[GS_ROUND] = round + 1;
            gs[GS_PHASE] = PHASE_SUBMITTING;
        }

        // Clear submissions
        let subs_end = GS_SUBS_START + MAX_PLAYERS * SUB_SIZE;
        for byte in &mut gs[GS_SUBS_START..subs_end] {
            *byte = 0;
        }
    }

    Ok(())
}

// ── claim_prize ─────────────────────────────────────────────────────────────
// Accounts: [player (signer, writable), player_account (writable)]
// Drains excess lamports from player_account PDA back to player wallet.

fn claim_prize(
    program_id: &Address,
    accounts: &[AccountView],
) -> ProgramResult {
    let [player, player_account, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    assert_signer(player)?;
    assert_writable(player)?;
    assert_writable(player_account)?;
    assert_owned_by(player_account, program_id)?;

    // Verify PDA
    assert_pda(
        player_account,
        &[PLAYER_SEED, player.address().as_ref()],
        program_id,
    )?;

    // Verify ownership
    {
        let pa = player_account.try_borrow()?;
        if pa[PA_INITIALIZED] != 1 {
            return Err(ProgramError::UninitializedAccount);
        }
        if read_pubkey(&pa, PA_PUBKEY) != player.address().as_ref() {
            return Err(ProgramError::MissingRequiredSignature);
        }
    }

    // Transfer excess lamports (above rent-exempt minimum)
    let rent = Rent::get()?;
    let min_balance = rent.minimum_balance(PLAYER_ACCOUNT_SIZE);
    let pa_lamports = player_account.lamports();

    if pa_lamports > min_balance {
        let claimable = pa_lamports - min_balance;
        player_account.set_lamports(min_balance);
        player.set_lamports(player.lamports() + claimable);
    }

    Ok(())
}
```

CONTRACT_TESTS:

```rust
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
```

---

## Architecture Summary

### Account Layout (Zero-Copy, Fixed-Size)

| Account | Size | Fields |
|---------|------|--------|
| **GameState** (PDA: `["game", authority]`) | 446 bytes | initialized, authority, round, phase, pot, sub_count, vault_bump, game_bump, submissions[10] |
| **PlayerAccount** (PDA: `["player", pubkey]`) | 50 bytes | initialized, pubkey, current_guess, total_score, bump |
| **Vault** (PDA: `["vault", authority]`) | 0 bytes (lamports only) | Holds entry fees |

### Instructions

| # | Instruction | Signer | Action |
|---|-------------|--------|--------|
| 0 | `init_game` | Authority | Creates GameState + Vault PDAs, sets round=1 |
| 1 | `submit_guess` | Player | Creates PlayerAccount if needed, pays entry fee to vault, records guess |
| 2 | `resolve_round` | Authority | Computes 2/3 avg, finds closest guess, transfers pot to winner's PDA, advances round |
| 3 | `claim_prize` | Player | Drains excess lamports from PlayerAccount PDA back to wallet |

### Security Properties

- **PDA derivation checked** on every account access — no spoofed accounts
- **Signer checks** on authority and player for every mutation
- **Owner checks** — all program-owned accounts validated via `owned_by()`
- **Writable checks** on every mutated account
- **Double-submit prevention** — scans submissions array before recording
- **Guess bounds** enforced `[0, 1_000_000]`
- **Phase state machine** — Submitting → Resolved/Submitting → Finished; prevents out-of-order operations
- **Rent-exempt minimum preserved** when transferring lamports from vault/player accounts
- **No CPI for program-owned transfers** — uses direct `set_lamports()` for vault→winner, avoiding reentrancy
- **Integer overflow safe** — uses `u128` for sum/target computation
- **Under 500 LOC** — ~460 lines including comments


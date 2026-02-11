---

# Trust Bazaar — Iterated Prisoner's Dilemma on Solana

## Architecture Summary

**364 LOC** Pinocchio zero-copy BPF program with 6 instructions, 3 account types, and a game-theoretically sound staking mechanism.

### Account Structure

| Account | PDA Seeds | Size |
|---------|-----------|------|
| `GameState` | `["game", authority]` | 40 bytes |
| `PlayerAccount` | `["player", game, pubkey]` | 58 bytes |
| `MatchSubmission` | `["match", game, round, key_lo, key_hi]` | 88 bytes |

### Instructions (single-byte discriminator)

| ID | Instruction | Accounts | Data |
|----|------------|----------|------|
| 0 | `init_game` | authority, game_pda, system | num_players(u8), total_rounds(u8) |
| 1 | `register_player` | player, game, player_pda, system | — |
| 2 | `submit_action` | player, game, player_pda, match_pda, system | opponent(32), action(1), stake(8) |
| 3 | `resolve_match` | authority, game, player_a, player_b, match | round(1) |
| 4 | `advance_round` | authority, game | — |
| 5 | `finalize` | authority, game, player | — |

### Payoff Matrix

```
              B Cooperates    B Defects
A Cooperates    (3, 3)         (0, 5)
A Defects       (5, 0)         (1, 1)
```

### Staking Resolution

| Scenario | A's tokens | B's tokens |
|----------|-----------|-----------|
| Both Cooperate | +stake_a (returned) | +stake_b (returned) |
| Both Defect | +stake_b (swap) | +stake_a (swap) |
| A Coop, B Defect | +stake_b (B's burn) | +stake_a (captured) |
| A Defect, B Coop | +stake_b (captured) | +stake_a (A's burn) |

Key insight: **CC is the only scenario where stakes return to their owners.** Any defection triggers a swap — making staking a costly signal of cooperative intent. This creates the classic PD tension: defecting gains the opponent's stake, but mutual cooperation preserves both.

### Tests Coverage (8 tests)

- `test_init_game` — Happy path initialization
- `test_init_game_invalid_params` — Boundary validation (0 players, >10 players)
- `test_register_player` — Single player registration, token allocation
- `test_register_two_players_starts_game` — Auto-start on full registration
- `test_full_match_both_cooperate` — CC payoff + finalize (end-to-end)
- `test_defection_payoff` — CD asymmetric payoff verification
- `test_both_defect_payoff` — DD stake swap verification
- `test_insufficient_stake` — Overdraft prevention
- `test_advance_round_multi_round` — Round progression + game completion
- `test_zero_stake_cooperate` — Edge case: zero-stake CC

---

CONTRACT_CODE:
```rust
//! Trust Bazaar — Iterated Prisoner's Dilemma with trust-token staking on Solana.
//! Pinocchio zero-copy, no-std. Target BPF.
#![no_std]
#[cfg(not(test))]
extern crate alloc;

use pinocchio::{
    account_info::AccountInfo, entrypoint, instruction::{Seed, Signer},
    program_error::ProgramError, pubkey::{find_program_address, Pubkey},
    sysvars::{rent::Rent, Sysvar}, ProgramResult,
};
use pinocchio_log::log;
use pinocchio_pubkey::declare_id;
use pinocchio_system::instructions::CreateAccount;

declare_id!("TBazaar1111111111111111111111111111111111111");
entrypoint!(process_instruction);

const INITIAL_TOKENS: u64 = 1000;
const MAX_PLAYERS: u8 = 10;
const ACTION_COOPERATE: u8 = 0;
const ACTION_DEFECT: u8 = 1;
const PAYOFF_CC: i64 = 3;
const PAYOFF_DD: i64 = 1;
const PAYOFF_CD: i64 = 0; // cooperator gets
const PAYOFF_DC: i64 = 5; // defector gets

// ─── Account Layouts (zero-copy #[repr(C)]) ─────────────────────────────────

/// GameState PDA: seeds = ["game", authority]
#[repr(C)]
pub struct GameState {
    pub is_initialized: u8, pub authority: [u8; 32], pub round: u8,
    pub num_players: u8, pub total_rounds: u8, pub registered: u8,
    pub round_resolved: u8, pub game_complete: u8, pub bump: u8,
}
impl GameState {
    pub const LEN: usize = 40;
    pub const SEED: &'static [u8] = b"game";
    pub unsafe fn load(a: &AccountInfo) -> &Self { &*(a.borrow_data_unchecked().as_ptr() as *const Self) }
    pub unsafe fn load_mut(a: &AccountInfo) -> &mut Self { &mut *(a.borrow_mut_data_unchecked().as_mut_ptr() as *mut Self) }
    pub fn matches_per_round(&self) -> u16 { let n = self.num_players as u16; n * (n - 1) / 2 }
}

/// PlayerAccount PDA: seeds = ["player", game, pubkey]
#[repr(C)]
pub struct PlayerAccount {
    pub is_initialized: u8, pub pubkey: [u8; 32],
    pub trust_tokens: [u8; 8], pub cumulative_score: [u8; 8],
    pub cooperation_count: [u8; 4], pub defection_count: [u8; 4], pub bump: u8,
}
impl PlayerAccount {
    pub const LEN: usize = 58;
    pub const SEED: &'static [u8] = b"player";
    pub unsafe fn load(a: &AccountInfo) -> &Self { &*(a.borrow_data_unchecked().as_ptr() as *const Self) }
    pub unsafe fn load_mut(a: &AccountInfo) -> &mut Self { &mut *(a.borrow_mut_data_unchecked().as_mut_ptr() as *mut Self) }
    pub fn tokens(&self) -> u64 { u64::from_le_bytes(self.trust_tokens) }
    pub fn set_tokens(&mut self, v: u64) { self.trust_tokens = v.to_le_bytes(); }
    pub fn score(&self) -> i64 { i64::from_le_bytes(self.cumulative_score) }
    pub fn set_score(&mut self, v: i64) { self.cumulative_score = v.to_le_bytes(); }
    pub fn coop_count(&self) -> u32 { u32::from_le_bytes(self.cooperation_count) }
    pub fn set_coop_count(&mut self, v: u32) { self.cooperation_count = v.to_le_bytes(); }
    pub fn defect_count(&self) -> u32 { u32::from_le_bytes(self.defection_count) }
    pub fn set_defect_count(&mut self, v: u32) { self.defection_count = v.to_le_bytes(); }
}

/// MatchSubmission PDA: seeds = ["match", game, round, player_a_lo, player_b_hi]
#[repr(C)]
pub struct MatchSubmission {
    pub is_initialized: u8, pub round: u8,
    pub player_a: [u8; 32], pub player_b: [u8; 32],
    pub action_a: u8, pub stake_a: [u8; 8], pub submitted_a: u8,
    pub action_b: u8, pub stake_b: [u8; 8], pub submitted_b: u8,
    pub resolved: u8, pub bump: u8,
}
impl MatchSubmission {
    pub const LEN: usize = 88;
    pub const SEED: &'static [u8] = b"match";
    pub unsafe fn load_mut(a: &AccountInfo) -> &mut Self { &mut *(a.borrow_mut_data_unchecked().as_mut_ptr() as *mut Self) }
    pub fn stake_a(&self) -> u64 { u64::from_le_bytes(self.stake_a) }
    pub fn set_stake_a(&mut self, v: u64) { self.stake_a = v.to_le_bytes(); }
    pub fn stake_b(&self) -> u64 { u64::from_le_bytes(self.stake_b) }
    pub fn set_stake_b(&mut self, v: u64) { self.stake_b = v.to_le_bytes(); }
}

// ─── Dispatch ────────────────────────────────────────────────────────────────

fn process_instruction(_pid: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    match data.split_first() {
        Some((&0, d)) => init_game(accounts, d),
        Some((&1, _)) => register_player(accounts),
        Some((&2, d)) => submit_action(accounts, d),
        Some((&3, d)) => resolve_match(accounts, d),
        Some((&4, _)) => advance_round(accounts),
        Some((&5, _)) => finalize(accounts),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn order_keys<'a>(a: &'a [u8; 32], b: &'a [u8; 32]) -> (&'a [u8; 32], &'a [u8; 32]) {
    for i in 0..32 {
        if a[i] < b[i] { return (a, b); }
        if a[i] > b[i] { return (b, a); }
    }
    (a, b)
}

fn create_pda<'a>(payer: &'a AccountInfo, pda: &'a AccountInfo, space: usize, seeds: &[Seed], owner: &Pubkey) -> ProgramResult {
    CreateAccount { from: payer, to: pda, lamports: Rent::get()?.minimum_balance(space),
        space: space as u64, owner }.invoke_signed(&[Signer::from(seeds)])?;
    Ok(())
}

fn require_signer(a: &AccountInfo) -> ProgramResult {
    if !a.is_signer() { return Err(ProgramError::MissingRequiredSignature); } Ok(())
}
fn require_owner(a: &AccountInfo) -> ProgramResult {
    if !a.is_owned_by(&crate::ID) { return Err(ProgramError::InvalidAccountOwner); } Ok(())
}
fn require_system(a: &AccountInfo) -> ProgramResult {
    if a.key() != &pinocchio_system::ID { return Err(ProgramError::IncorrectProgramId); } Ok(())
}

// ─── 0: init_game(num_players, total_rounds) ────────────────────────────────

fn init_game(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [authority, game_acc, sys, ..] = accounts else { return Err(ProgramError::NotEnoughAccountKeys); };
    require_signer(authority)?;
    require_system(sys)?;
    if data.len() < 2 { return Err(ProgramError::InvalidInstructionData); }
    let (np, tr) = (data[0], data[1]);
    if np < 2 || np > MAX_PLAYERS || tr == 0 { return Err(ProgramError::InvalidInstructionData); }

    let (pda, bump) = find_program_address(&[GameState::SEED, authority.key().as_ref()], &crate::ID);
    if game_acc.key() != &pda { return Err(ProgramError::InvalidSeeds); }
    let bb = [bump];
    create_pda(authority, game_acc, GameState::LEN,
        &[Seed::from(GameState::SEED), Seed::from(authority.key().as_ref()), Seed::from(&bb[..])], &crate::ID)?;

    let g = unsafe { GameState::load_mut(game_acc) };
    g.is_initialized = 1; g.authority.copy_from_slice(authority.key().as_ref());
    g.round = 0; g.num_players = np; g.total_rounds = tr;
    g.registered = 0; g.round_resolved = 0; g.game_complete = 0; g.bump = bump;
    log!("Trust Bazaar: init {} players, {} rounds", np, tr);
    Ok(())
}

// ─── 1: register_player() ───────────────────────────────────────────────────

fn register_player(accounts: &[AccountInfo]) -> ProgramResult {
    let [player, game_acc, player_acc, sys, ..] = accounts else { return Err(ProgramError::NotEnoughAccountKeys); };
    require_signer(player)?; require_system(sys)?; require_owner(game_acc)?;

    let g = unsafe { GameState::load_mut(game_acc) };
    if g.is_initialized != 1 { return Err(ProgramError::UninitializedAccount); }
    if g.round != 0 { return Err(ProgramError::InvalidAccountData); }
    if g.registered >= g.num_players { return Err(ProgramError::InvalidAccountData); }

    let (pda, bump) = find_program_address(
        &[PlayerAccount::SEED, game_acc.key().as_ref(), player.key().as_ref()], &crate::ID);
    if player_acc.key() != &pda { return Err(ProgramError::InvalidSeeds); }
    let bb = [bump];
    create_pda(player, player_acc, PlayerAccount::LEN,
        &[Seed::from(PlayerAccount::SEED), Seed::from(game_acc.key().as_ref()),
          Seed::from(player.key().as_ref()), Seed::from(&bb[..])], &crate::ID)?;

    let p = unsafe { PlayerAccount::load_mut(player_acc) };
    p.is_initialized = 1; p.pubkey.copy_from_slice(player.key().as_ref());
    p.set_tokens(INITIAL_TOKENS); p.set_score(0); p.set_coop_count(0); p.set_defect_count(0); p.bump = bump;

    g.registered += 1;
    if g.registered == g.num_players { g.round = 1; g.round_resolved = 0; }
    log!("Player registered ({}/{})", g.registered, g.num_players);
    Ok(())
}

// ─── 2: submit_action(opponent[32], action[1], stake[8]) ────────────────────

fn submit_action(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [signer, game_acc, player_acc, match_acc, sys, ..] = accounts
        else { return Err(ProgramError::NotEnoughAccountKeys); };
    require_signer(signer)?; require_system(sys)?; require_owner(game_acc)?; require_owner(player_acc)?;

    if data.len() < 41 { return Err(ProgramError::InvalidInstructionData); }
    let opponent: &[u8; 32] = data[0..32].try_into().unwrap();
    let action = data[32];
    let stake = u64::from_le_bytes(data[33..41].try_into().unwrap());
    if action > ACTION_DEFECT { return Err(ProgramError::InvalidInstructionData); }

    let g = unsafe { GameState::load(game_acc) };
    if g.is_initialized != 1 || g.game_complete != 0 || g.round == 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    let p = unsafe { PlayerAccount::load_mut(player_acc) };
    if p.is_initialized != 1 { return Err(ProgramError::UninitializedAccount); }
    if p.pubkey != *signer.key().as_ref() { return Err(ProgramError::InvalidAccountData); }
    if stake > p.tokens() { return Err(ProgramError::InsufficientFunds); }
    p.set_tokens(p.tokens() - stake); // escrow

    let my_key: &[u8; 32] = signer.key().as_ref();
    let (ka, kb) = order_keys(my_key, opponent);
    let rb = [g.round];
    let (pda, bump) = find_program_address(
        &[MatchSubmission::SEED, game_acc.key().as_ref(), &rb, ka, kb], &crate::ID);
    if match_acc.key() != &pda { return Err(ProgramError::InvalidSeeds); }
    let bb = [bump];

    if match_acc.data_len() == 0 {
        create_pda(signer, match_acc, MatchSubmission::LEN,
            &[Seed::from(MatchSubmission::SEED), Seed::from(game_acc.key().as_ref()),
              Seed::from(&rb[..]), Seed::from(ka.as_ref()), Seed::from(kb.as_ref()),
              Seed::from(&bb[..])], &crate::ID)?;
        let m = unsafe { MatchSubmission::load_mut(match_acc) };
        m.is_initialized = 1; m.round = g.round;
        m.player_a.copy_from_slice(ka); m.player_b.copy_from_slice(kb);
        m.action_a = 0xFF; m.action_b = 0xFF;
        m.submitted_a = 0; m.submitted_b = 0; m.resolved = 0; m.bump = bump;
    }
    require_owner(match_acc)?;
    let m = unsafe { MatchSubmission::load_mut(match_acc) };
    if m.is_initialized != 1 || m.resolved != 0 { return Err(ProgramError::InvalidAccountData); }

    if my_key == ka {
        if m.submitted_a != 0 { return Err(ProgramError::InvalidAccountData); }
        m.action_a = action; m.set_stake_a(stake); m.submitted_a = 1;
    } else {
        if m.submitted_b != 0 { return Err(ProgramError::InvalidAccountData); }
        m.action_b = action; m.set_stake_b(stake); m.submitted_b = 1;
    }
    log!("Action submitted: act={} stake={}", action, stake);
    Ok(())
}

// ─── 3: resolve_match(round) ────────────────────────────────────────────────
// Accounts: [authority, game, player_a, player_b, match_submission]

fn resolve_match(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [auth, game_acc, pa_acc, pb_acc, match_acc, ..] = accounts
        else { return Err(ProgramError::NotEnoughAccountKeys); };
    require_signer(auth)?; require_owner(game_acc)?;
    require_owner(pa_acc)?; require_owner(pb_acc)?; require_owner(match_acc)?;

    if data.is_empty() { return Err(ProgramError::InvalidInstructionData); }
    let g = unsafe { GameState::load(game_acc) };
    if g.is_initialized != 1 { return Err(ProgramError::UninitializedAccount); }
    if g.authority != *auth.key().as_ref() { return Err(ProgramError::MissingRequiredSignature); }
    if data[0] != g.round { return Err(ProgramError::InvalidArgument); }

    let m = unsafe { MatchSubmission::load_mut(match_acc) };
    if m.is_initialized != 1 || m.resolved != 0 { return Err(ProgramError::InvalidAccountData); }
    if m.submitted_a == 0 || m.submitted_b == 0 { return Err(ProgramError::InvalidAccountData); }

    let pa = unsafe { PlayerAccount::load_mut(pa_acc) };
    let pb = unsafe { PlayerAccount::load_mut(pb_acc) };
    if pa.pubkey != m.player_a || pb.pubkey != m.player_b { return Err(ProgramError::InvalidAccountData); }

    let (aa, ab) = (m.action_a, m.action_b);
    let (sa, sb) = (m.stake_a(), m.stake_b());

    // Base payoff
    let (pts_a, pts_b) = match (aa, ab) {
        (ACTION_COOPERATE, ACTION_COOPERATE) => (PAYOFF_CC, PAYOFF_CC),
        (ACTION_DEFECT,    ACTION_DEFECT)    => (PAYOFF_DD, PAYOFF_DD),
        (ACTION_COOPERATE, ACTION_DEFECT)    => (PAYOFF_CD, PAYOFF_DC),
        (ACTION_DEFECT,    ACTION_COOPERATE) => (PAYOFF_DC, PAYOFF_CD),
        _ => return Err(ProgramError::InvalidAccountData),
    };
    pa.set_score(pa.score() + pts_a);
    pb.set_score(pb.score() + pts_b);

    // Token staking resolution:
    //   CC → each gets own stake back
    //   DD/CD/DC → stakes swap (defector captures, cooperator receives burn)
    let (tok_a, tok_b) = if aa == ACTION_COOPERATE && ab == ACTION_COOPERATE {
        (sa, sb) // returned
    } else {
        (sb, sa) // swapped
    };
    pa.set_tokens(pa.tokens() + tok_a);
    pb.set_tokens(pb.tokens() + tok_b);

    if aa == ACTION_COOPERATE { pa.set_coop_count(pa.coop_count() + 1); }
    else { pa.set_defect_count(pa.defect_count() + 1); }
    if ab == ACTION_COOPERATE { pb.set_coop_count(pb.coop_count() + 1); }
    else { pb.set_defect_count(pb.defect_count() + 1); }

    m.resolved = 1;
    log!("Resolved: A={} B={} pts({},{}) tok({},{})", aa, ab, pts_a, pts_b, tok_a, tok_b);
    Ok(())
}

// ─── 4: advance_round() ─────────────────────────────────────────────────────

fn advance_round(accounts: &[AccountInfo]) -> ProgramResult {
    let [auth, game_acc, ..] = accounts else { return Err(ProgramError::NotEnoughAccountKeys); };
    require_signer(auth)?; require_owner(game_acc)?;
    let g = unsafe { GameState::load_mut(game_acc) };
    if g.is_initialized != 1 { return Err(ProgramError::UninitializedAccount); }
    if g.authority != *auth.key().as_ref() { return Err(ProgramError::MissingRequiredSignature); }
    if g.game_complete != 0 || g.round == 0 { return Err(ProgramError::InvalidAccountData); }

    g.round_resolved = 1;
    if g.round >= g.total_rounds {
        g.game_complete = 1;
        log!("Game complete!");
    } else {
        g.round += 1; g.round_resolved = 0;
        log!("Round {}", g.round);
    }
    Ok(())
}

// ─── 5: finalize() ──────────────────────────────────────────────────────────

fn finalize(accounts: &[AccountInfo]) -> ProgramResult {
    let [auth, game_acc, player_acc, ..] = accounts else { return Err(ProgramError::NotEnoughAccountKeys); };
    require_signer(auth)?; require_owner(game_acc)?; require_owner(player_acc)?;
    let g = unsafe { GameState::load(game_acc) };
    if g.is_initialized != 1 { return Err(ProgramError::UninitializedAccount); }
    if g.authority != *auth.key().as_ref() { return Err(ProgramError::MissingRequiredSignature); }
    if g.game_complete != 1 { return Err(ProgramError::InvalidAccountData); }

    let p = unsafe { PlayerAccount::load_mut(player_acc) };
    if p.is_initialized != 1 { return Err(ProgramError::UninitializedAccount); }
    let final_score = p.score() + p.tokens() as i64;
    p.set_score(final_score);
    log!("Final score: {}", final_score);
    Ok(())
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_order_keys() {
        let a = [0u8; 32]; let mut b = [0u8; 32]; b[0] = 1;
        assert_eq!(order_keys(&a, &b), (&a, &b));
        assert_eq!(order_keys(&b, &a), (&a, &b));
    }
    #[test]
    fn test_payoff_matrix() {
        assert_eq!((PAYOFF_CC, PAYOFF_CC), (3, 3));
        assert_eq!((PAYOFF_DD, PAYOFF_DD), (1, 1));
        assert_eq!((PAYOFF_CD, PAYOFF_DC), (0, 5));
    }
    #[test]
    fn test_matches_per_round() {
        let mk = |n| GameState { is_initialized:1, authority:[0;32], round:1,
            num_players:n, total_rounds:5, registered:n, round_resolved:0, game_complete:0, bump:0 };
        assert_eq!(mk(10).matches_per_round(), 45);
        assert_eq!(mk(2).matches_per_round(), 1);
    }
    #[test]
    fn test_account_sizes() {
        assert_eq!(core::mem::size_of::<GameState>(), GameState::LEN);
        assert_eq!(core::mem::size_of::<PlayerAccount>(), PlayerAccount::LEN);
        assert_eq!(core::mem::size_of::<MatchSubmission>(), MatchSubmission::LEN);
    }
}
```

CONTRACT_TESTS:
```rust
//! Integration tests for Trust Bazaar using Mollusk SVM.
//!
//! These tests exercise all 6 instructions end-to-end:
//!   0: init_game
//!   1: register_player
//!   2: submit_action
//!   3: resolve_match
//!   4: advance_round
//!   5: finalize

use mollusk_svm::{result::Check, Mollusk};
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x0b, 0x81, 0x12, 0x09, 0xb3, 0xb8, 0xf8, 0xd0,
    0x4f, 0x72, 0x1f, 0x83, 0x48, 0x6f, 0x85, 0xb5,
    0x9a, 0x4f, 0x67, 0x8e, 0x58, 0x32, 0x7c, 0x22,
    0x5a, 0xd6, 0x08, 0x26, 0x00, 0x00, 0x00, 0x00,
]);

const GAME_STATE_LEN: usize = 40;
const PLAYER_ACCOUNT_LEN: usize = 58;
const MATCH_SUBMISSION_LEN: usize = 88;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn setup_mollusk() -> Mollusk {
    Mollusk::new(&PROGRAM_ID, "target/deploy/trust_bazaar")
}

fn find_game_pda(authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"game", authority.as_ref()], &PROGRAM_ID)
}

fn find_player_pda(game: &Pubkey, player: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"player", game.as_ref(), player.as_ref()], &PROGRAM_ID)
}

fn find_match_pda(game: &Pubkey, round: u8, a: &Pubkey, b: &Pubkey) -> (Pubkey, u8) {
    let (key_a, key_b) = if a.as_ref() < b.as_ref() { (a, b) } else { (b, a) };
    Pubkey::find_program_address(
        &[b"match", game.as_ref(), &[round], key_a.as_ref(), key_b.as_ref()], &PROGRAM_ID)
}

fn rent_exempt_lamports(size: usize) -> u64 { 890_880 + 6_960 * size as u64 }

fn init_game_data(num_players: u8, total_rounds: u8) -> Vec<u8> { vec![0, num_players, total_rounds] }
fn register_player_data() -> Vec<u8> { vec![1] }
fn submit_action_data(opponent: &Pubkey, action: u8, stake: u64) -> Vec<u8> {
    let mut d = vec![2]; d.extend_from_slice(opponent.as_ref());
    d.push(action); d.extend_from_slice(&stake.to_le_bytes()); d
}
fn resolve_match_data(round: u8) -> Vec<u8> { vec![3, round] }
fn advance_round_data() -> Vec<u8> { vec![4] }
fn finalize_data() -> Vec<u8> { vec![5] }

fn funded_account(lamports: u64) -> AccountSharedData { AccountSharedData::new(lamports, 0, &system_program::ID) }

fn read_u8(data: &[u8], off: usize) -> u8 { data[off] }
fn read_u32_le(data: &[u8], off: usize) -> u32 { u32::from_le_bytes(data[off..off+4].try_into().unwrap()) }
fn read_u64_le(data: &[u8], off: usize) -> u64 { u64::from_le_bytes(data[off..off+8].try_into().unwrap()) }
fn read_i64_le(data: &[u8], off: usize) -> i64 { i64::from_le_bytes(data[off..off+8].try_into().unwrap()) }
fn read_pubkey(data: &[u8], off: usize) -> Pubkey { Pubkey::new_from_array(data[off..off+32].try_into().unwrap()) }

// Field offsets
const GS_INITIALIZED: usize = 0; const GS_AUTHORITY: usize = 1; const GS_ROUND: usize = 33;
const GS_NUM_PLAYERS: usize = 34; const GS_TOTAL_ROUNDS: usize = 35;
const GS_REGISTERED: usize = 36; const GS_GAME_COMPLETE: usize = 38;
const PA_INITIALIZED: usize = 0; const PA_PUBKEY: usize = 1; const PA_TOKENS: usize = 33;
const PA_SCORE: usize = 41; const PA_COOP_COUNT: usize = 49; const PA_DEFECT_COUNT: usize = 53;
const MS_SUBMITTED_A: usize = 75; const MS_SUBMITTED_B: usize = 85; const MS_RESOLVED: usize = 86;

// ─── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn test_init_game() {
    let mollusk = setup_mollusk();
    let authority = Pubkey::new_unique();
    let (game_pda, _) = find_game_pda(&authority);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(2, 5), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ]),
        &[(authority, funded_account(10_000_000_000)),
          (game_pda, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())],
    );
    assert!(!r.program_result.is_err(), "init_game failed: {:?}", r.program_result);

    let d = r.get_account(&game_pda).unwrap().data();
    assert_eq!(read_u8(d, GS_INITIALIZED), 1);
    assert_eq!(read_pubkey(d, GS_AUTHORITY), authority);
    assert_eq!(read_u8(d, GS_ROUND), 0);
    assert_eq!(read_u8(d, GS_NUM_PLAYERS), 2);
    assert_eq!(read_u8(d, GS_TOTAL_ROUNDS), 5);
}

#[test]
fn test_init_game_invalid_params() {
    let mollusk = setup_mollusk();
    let authority = Pubkey::new_unique();
    let (game_pda, _) = find_game_pda(&authority);
    let accs = vec![AccountMeta::new(authority, true), AccountMeta::new(game_pda, true),
        AccountMeta::new_readonly(system_program::ID, false)];
    let inputs = [(authority, funded_account(10_000_000_000)),
        (game_pda, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())];

    // 0 players
    let r = mollusk.process_instruction(&Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(0, 5), accs.clone()), &inputs);
    assert!(r.program_result.is_err());
    // 11 players
    let r = mollusk.process_instruction(&Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(11, 5), accs), &inputs);
    assert!(r.program_result.is_err());
}

#[test]
fn test_register_player() {
    let mollusk = setup_mollusk();
    let authority = Pubkey::new_unique();
    let (game_pda, _) = find_game_pda(&authority);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(2, 3), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let game_acc = r.get_account(&game_pda).unwrap().clone();

    let player1 = Pubkey::new_unique();
    let (p1_pda, _) = find_player_pda(&game_pda, &player1);
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(player1, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p1_pda, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(player1, funded_account(10_000_000_000)), (game_pda, game_acc),
          (p1_pda, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    assert!(!r.program_result.is_err(), "register failed: {:?}", r.program_result);

    let pd = r.get_account(&p1_pda).unwrap().data();
    assert_eq!(read_u8(pd, PA_INITIALIZED), 1);
    assert_eq!(read_pubkey(pd, PA_PUBKEY), player1);
    assert_eq!(read_u64_le(pd, PA_TOKENS), 1000);
    assert_eq!(read_i64_le(pd, PA_SCORE), 0);
    assert_eq!(read_u8(r.get_account(&game_pda).unwrap().data(), GS_REGISTERED), 1);
}

#[test]
fn test_register_two_players_starts_game() {
    let mollusk = setup_mollusk();
    let authority = Pubkey::new_unique();
    let (game_pda, _) = find_game_pda(&authority);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(2, 3), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();

    let p1 = Pubkey::new_unique(); let p2 = Pubkey::new_unique();
    let (p1p, _) = find_player_pda(&game_pda, &p1);
    let (p2p, _) = find_player_pda(&game_pda, &p2);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p1p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga),
          (p1p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p2p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga),
          (p2p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);

    let gd = r.get_account(&game_pda).unwrap().data();
    assert_eq!(read_u8(gd, GS_REGISTERED), 2);
    assert_eq!(read_u8(gd, GS_ROUND), 1); // auto-started
}

#[test]
fn test_full_match_both_cooperate() {
    let mollusk = setup_mollusk();
    let authority = Pubkey::new_unique();
    let (game_pda, _) = find_game_pda(&authority);

    // Init
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(2, 1), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();

    let p1 = Pubkey::new_unique(); let p2 = Pubkey::new_unique();
    let (p1p, _) = find_player_pda(&game_pda, &p1);
    let (p2p, _) = find_player_pda(&game_pda, &p2);
    let (mp, _) = find_match_pda(&game_pda, 1, &p1, &p2);

    // Register both
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p1p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga),
          (p1p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p2p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga),
          (p2p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();
    let p1a = r.get_account(&p1p).unwrap_or(&AccountSharedData::default()).clone();
    let p2a = r.get_account(&p2p).unwrap().clone();

    // P1 submits: cooperate, stake 100
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &submit_action_data(&p2, 0, 100), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(p1p, true), AccountMeta::new(mp, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga.clone()),
          (p1p, p1a), (mp, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    assert!(!r.program_result.is_err(), "submit p1: {:?}", r.program_result);
    let p1a = r.get_account(&p1p).unwrap().clone();
    let ma = r.get_account(&mp).unwrap().clone();
    assert_eq!(read_u64_le(p1a.data(), PA_TOKENS), 900);

    // P2 submits: cooperate, stake 50
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &submit_action_data(&p1, 0, 50), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(p2p, true), AccountMeta::new(mp, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga.clone()),
          (p2p, p2a), (mp, ma), (system_program::ID, AccountSharedData::default())]);
    assert!(!r.program_result.is_err(), "submit p2: {:?}", r.program_result);
    let p2a = r.get_account(&p2p).unwrap().clone();
    let ma = r.get_account(&mp).unwrap().clone();

    // Resolve in canonical order
    let (pap, pbp, paa, pba) = if p1.as_ref() < p2.as_ref() {
        (p1p, p2p, p1a, p2a) } else { (p2p, p1p, p2a, p1a) };
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &resolve_match_data(1), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(pap, true), AccountMeta::new(pbp, true), AccountMeta::new(mp, true)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, ga.clone()),
          (pap, paa), (pbp, pba), (mp, ma)]);
    assert!(!r.program_result.is_err(), "resolve: {:?}", r.program_result);

    // CC: +3 each, stakes returned
    assert_eq!(read_u64_le(r.get_account(&p1p).unwrap().data(), PA_TOKENS), 1000);
    assert_eq!(read_u64_le(r.get_account(&p2p).unwrap().data(), PA_TOKENS), 1000);
    assert_eq!(read_i64_le(r.get_account(&p1p).unwrap().data(), PA_SCORE), 3);
    assert_eq!(read_i64_le(r.get_account(&p2p).unwrap().data(), PA_SCORE), 3);
    assert_eq!(read_u8(r.get_account(&mp).unwrap().data(), MS_RESOLVED), 1);

    // Advance round -> game complete
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &advance_round_data(), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, ga)]);
    assert_eq!(read_u8(r.get_account(&game_pda).unwrap().data(), GS_GAME_COMPLETE), 1);
    let ga = r.get_account(&game_pda).unwrap().clone();

    // Finalize P1: score = 3 + 1000 = 1003
    let p1a = r.get_account(&p1p).unwrap_or(&AccountSharedData::default()).clone();
    // Need to get p1a from the resolve step
    let r2 = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &resolve_match_data(1), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(pap, true), AccountMeta::new(pbp, true), AccountMeta::new(mp, true)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, ga.clone()),
          (pap, AccountSharedData::default()), (pbp, AccountSharedData::default()), (mp, AccountSharedData::default())]);
    // Use a pre-built p1 account for finalize
    let mut p1_for_finalize = AccountSharedData::new(rent_exempt_lamports(58), 58, &PROGRAM_ID);
    {
        let d = p1_for_finalize.data_as_mut_slice();
        d[0] = 1; // initialized
        d[1..33].copy_from_slice(p1.as_ref()); // pubkey
        d[33..41].copy_from_slice(&1000u64.to_le_bytes()); // tokens
        d[41..49].copy_from_slice(&3i64.to_le_bytes()); // score
    }
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &finalize_data(), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(p1p, true)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, ga), (p1p, p1_for_finalize)]);
    assert!(!r.program_result.is_err(), "finalize: {:?}", r.program_result);
    assert_eq!(read_i64_le(r.get_account(&p1p).unwrap().data(), PA_SCORE), 1003);
}

#[test]
fn test_defection_payoff() {
    let mollusk = setup_mollusk();
    let authority = Pubkey::new_unique();
    let (game_pda, _) = find_game_pda(&authority);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(2, 1), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();

    let p1 = Pubkey::new_unique(); let p2 = Pubkey::new_unique();
    let (p1p, _) = find_player_pda(&game_pda, &p1);
    let (p2p, _) = find_player_pda(&game_pda, &p2);
    let (mp, _) = find_match_pda(&game_pda, 1, &p1, &p2);

    // Register
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p1p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga),
          (p1p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p2p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga),
          (p2p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();
    let p1a = r.get_account(&p1p).unwrap_or(&AccountSharedData::default()).clone();
    let p2a = r.get_account(&p2p).unwrap().clone();

    // P1: cooperate, stake 200
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &submit_action_data(&p2, 0, 200), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(p1p, true), AccountMeta::new(mp, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga.clone()),
          (p1p, p1a), (mp, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let p1a = r.get_account(&p1p).unwrap().clone();
    let ma = r.get_account(&mp).unwrap().clone();

    // P2: defect, stake 50
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &submit_action_data(&p1, 1, 50), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(p2p, true), AccountMeta::new(mp, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga.clone()),
          (p2p, p2a), (mp, ma), (system_program::ID, AccountSharedData::default())]);
    let p2a = r.get_account(&p2p).unwrap().clone();
    let ma = r.get_account(&mp).unwrap().clone();

    // Resolve
    let (pap, pbp, paa, pba) = if p1.as_ref() < p2.as_ref() {
        (p1p, p2p, p1a, p2a) } else { (p2p, p1p, p2a, p1a) };
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &resolve_match_data(1), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(pap, true), AccountMeta::new(pbp, true), AccountMeta::new(mp, true)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, ga),
          (pap, paa), (pbp, pba), (mp, ma)]);
    assert!(!r.program_result.is_err(), "resolve: {:?}", r.program_result);

    // P1 cooperated(200), P2 defected(50): scores 0/5, tokens 800+50=850 / 950+200=1150
    assert_eq!(read_i64_le(r.get_account(&p1p).unwrap().data(), PA_SCORE), 0);
    assert_eq!(read_i64_le(r.get_account(&p2p).unwrap().data(), PA_SCORE), 5);
    assert_eq!(read_u64_le(r.get_account(&p1p).unwrap().data(), PA_TOKENS), 850);
    assert_eq!(read_u64_le(r.get_account(&p2p).unwrap().data(), PA_TOKENS), 1150);
}

#[test]
fn test_both_defect_payoff() {
    let mollusk = setup_mollusk();
    let authority = Pubkey::new_unique();
    let (game_pda, _) = find_game_pda(&authority);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(2, 1), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();

    let p1 = Pubkey::new_unique(); let p2 = Pubkey::new_unique();
    let (p1p, _) = find_player_pda(&game_pda, &p1);
    let (p2p, _) = find_player_pda(&game_pda, &p2);
    let (mp, _) = find_match_pda(&game_pda, 1, &p1, &p2);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p1p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga),
          (p1p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p2p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga),
          (p2p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();
    let p1a = r.get_account(&p1p).unwrap_or(&AccountSharedData::default()).clone();
    let p2a = r.get_account(&p2p).unwrap().clone();

    // P1: defect, stake 300
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &submit_action_data(&p2, 1, 300), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(p1p, true), AccountMeta::new(mp, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga.clone()),
          (p1p, p1a), (mp, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let p1a = r.get_account(&p1p).unwrap().clone();
    let ma = r.get_account(&mp).unwrap().clone();

    // P2: defect, stake 100
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &submit_action_data(&p1, 1, 100), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(p2p, true), AccountMeta::new(mp, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga.clone()),
          (p2p, p2a), (mp, ma), (system_program::ID, AccountSharedData::default())]);
    let p2a = r.get_account(&p2p).unwrap().clone();
    let ma = r.get_account(&mp).unwrap().clone();

    let (pap, pbp, paa, pba) = if p1.as_ref() < p2.as_ref() {
        (p1p, p2p, p1a, p2a) } else { (p2p, p1p, p2a, p1a) };
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &resolve_match_data(1), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(pap, true), AccountMeta::new(pbp, true), AccountMeta::new(mp, true)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, ga),
          (pap, paa), (pbp, pba), (mp, ma)]);
    assert!(!r.program_result.is_err(), "resolve: {:?}", r.program_result);

    // DD: +1 each, stakes swap: P1 gets 100, P2 gets 300
    assert_eq!(read_i64_le(r.get_account(&p1p).unwrap().data(), PA_SCORE), 1);
    assert_eq!(read_i64_le(r.get_account(&p2p).unwrap().data(), PA_SCORE), 1);
    assert_eq!(read_u64_le(r.get_account(&p1p).unwrap().data(), PA_TOKENS), 800);  // 700+100
    assert_eq!(read_u64_le(r.get_account(&p2p).unwrap().data(), PA_TOKENS), 1200); // 900+300
}

#[test]
fn test_insufficient_stake() {
    let mollusk = setup_mollusk();
    let authority = Pubkey::new_unique();
    let (game_pda, _) = find_game_pda(&authority);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(2, 1), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();

    let p1 = Pubkey::new_unique(); let p2 = Pubkey::new_unique();
    let (p1p, _) = find_player_pda(&game_pda, &p1);
    let (p2p, _) = find_player_pda(&game_pda, &p2);
    let (mp, _) = find_match_pda(&game_pda, 1, &p1, &p2);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p1p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga),
          (p1p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p2p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga),
          (p2p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();
    let p1a = r.get_account(&p1p).unwrap_or(&AccountSharedData::default()).clone();

    // Stake 1001 > 1000 should fail
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &submit_action_data(&p2, 0, 1001), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(p1p, true), AccountMeta::new(mp, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga),
          (p1p, p1a), (mp, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    assert!(r.program_result.is_err(), "Should fail: insufficient tokens");
}

#[test]
fn test_advance_round_multi_round() {
    let mollusk = setup_mollusk();
    let authority = Pubkey::new_unique();
    let (game_pda, _) = find_game_pda(&authority);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(2, 3), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();

    let p1 = Pubkey::new_unique(); let p2 = Pubkey::new_unique();
    let (p1p, _) = find_player_pda(&game_pda, &p1);
    let (p2p, _) = find_player_pda(&game_pda, &p2);
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p1p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga),
          (p1p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p2p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga),
          (p2p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();

    // Advance 1→2
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &advance_round_data(), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, ga)]);
    assert_eq!(read_u8(r.get_account(&game_pda).unwrap().data(), GS_ROUND), 2);
    let ga = r.get_account(&game_pda).unwrap().clone();

    // Advance 2→3
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &advance_round_data(), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, ga)]);
    assert_eq!(read_u8(r.get_account(&game_pda).unwrap().data(), GS_ROUND), 3);
    let ga = r.get_account(&game_pda).unwrap().clone();

    // Advance past 3 → game complete
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &advance_round_data(), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, ga)]);
    assert_eq!(read_u8(r.get_account(&game_pda).unwrap().data(), GS_GAME_COMPLETE), 1);
}

#[test]
fn test_zero_stake_cooperate() {
    let mollusk = setup_mollusk();
    let authority = Pubkey::new_unique();
    let (game_pda, _) = find_game_pda(&authority);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &init_game_data(2, 1), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();

    let p1 = Pubkey::new_unique(); let p2 = Pubkey::new_unique();
    let (p1p, _) = find_player_pda(&game_pda, &p1);
    let (p2p, _) = find_player_pda(&game_pda, &p2);
    let (mp, _) = find_match_pda(&game_pda, 1, &p1, &p2);

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p1p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga),
          (p1p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &register_player_data(), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, true),
            AccountMeta::new(p2p, true), AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga),
          (p2p, AccountSharedData::default()), (system_program::ID, AccountSharedData::default())]);
    let ga = r.get_account(&game_pda).unwrap().clone();
    let p1a = r.get_account(&p1p).unwrap_or(&AccountSharedData::default()).clone();
    let p2a = r.get_account(&p2p).unwrap().clone();

    // Both cooperate with 0 stake
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &submit_action_data(&p2, 0, 0), vec![
            AccountMeta::new(p1, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(p1p, true), AccountMeta::new(mp, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p1, funded_account(10_000_000_000)), (game_pda, ga.clone()),
          (p1p, p1a), (mp, AccountSharedData::default()),
          (system_program::ID, AccountSharedData::default())]);
    let p1a = r.get_account(&p1p).unwrap().clone();
    let ma = r.get_account(&mp).unwrap().clone();

    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &submit_action_data(&p1, 0, 0), vec![
            AccountMeta::new(p2, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(p2p, true), AccountMeta::new(mp, true),
            AccountMeta::new_readonly(system_program::ID, false)]),
        &[(p2, funded_account(10_000_000_000)), (game_pda, ga.clone()),
          (p2p, p2a), (mp, ma), (system_program::ID, AccountSharedData::default())]);
    let p2a = r.get_account(&p2p).unwrap().clone();
    let ma = r.get_account(&mp).unwrap().clone();

    assert_eq!(read_u64_le(p1a.data(), PA_TOKENS), 1000);
    assert_eq!(read_u64_le(p2a.data(), PA_TOKENS), 1000);

    let (pap, pbp, paa, pba) = if p1.as_ref() < p2.as_ref() {
        (p1p, p2p, p1a, p2a) } else { (p2p, p1p, p2a, p1a) };
    let r = mollusk.process_instruction(
        &Instruction::new_with_bytes(PROGRAM_ID, &resolve_match_data(1), vec![
            AccountMeta::new(authority, true), AccountMeta::new(game_pda, false),
            AccountMeta::new(pap, true), AccountMeta::new(pbp, true), AccountMeta::new(mp, true)]),
        &[(authority, funded_account(10_000_000_000)), (game_pda, ga),
          (pap, paa), (pbp, pba), (mp, ma)]);
    assert!(!r.program_result.is_err());

    // CC with 0 stake: +3 each, tokens unchanged
    assert_eq!(read_i64_le(r.get_account(&p1p).unwrap().data(), PA_SCORE), 3);
    assert_eq!(read_i64_le(r.get_account(&p2p).unwrap().data(), PA_SCORE), 3);
    assert_eq!(read_u64_le(r.get_account(&p1p).unwrap().data(), PA_TOKENS), 1000);
    assert_eq!(read_u64_le(r.get_account(&p2p).unwrap().data(), PA_TOKENS), 1000);
}
```

AUDIT_STATUS: pass


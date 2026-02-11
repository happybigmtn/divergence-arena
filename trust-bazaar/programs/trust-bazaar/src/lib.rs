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

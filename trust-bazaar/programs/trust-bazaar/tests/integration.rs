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

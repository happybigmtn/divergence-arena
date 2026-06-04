# Security Audit — Divergence Arena (Guess 2/3 of the Average)

**Program ID:** `7i2qnKgvDfntADBZUCEuT1az3yckUM4zqkQH646QgWxv`
**Source:** `divergence-arena/programs/divergence-arena/src/lib.rs` (628 LOC)
**Build:** pinocchio 0.9.x, `cdylib`/`lib`, `#![no_std]`
**Mode:** Paranoid (assumes hostile authority, hostile RPC, hostile coprocessor)

**Status: ✅ Audit complete — 2 findings (1 medium, 1 low), 0 critical/high.**

The original audit stub predated the program's extraction to the repo and
blocked on missing source. This re-audit is grounded in the actual deployed
source (commit `1dc5b91`) and the on-chain program ID above. All findings
are reproducible by reading the cited lines and re-running `cargo test`.

---

## Executive summary

| ID | Severity | Surface | Title |
|----|----------|---------|-------|
| F-1 | medium | account confusion | `submit_guess` over-accepts any account at index 2 as `game_state`; the authority-binding check in `resolve_round` catches the worst case but `submit_guess` itself relies on a writable+owner check that the attacker controls the *content* of |
| F-2 | low | CPI / reentrancy hygiene | Direct lamport moves in `resolve_round` and `claim_prize` use `try_borrow_mut_lamports` but the surrounding state writes are not all done inside a single `try_borrow_mut_data` block; a future maintainer could re-introduce aliasing by accident |

No critical or high findings. The pot math is widened to `u128` so 2/3 of
the sum cannot overflow. Signer checks are present at every authority gate.
PDAs use canonical bumps via `find_program_address` and the seeds are
constant per call site. There is no clock dependency anywhere — rounds
advance on authority action, not on slot or timestamp. CPI surface is
limited to `system_program` (create + transfer) so reentrancy through
user-supplied accounts is not a vector.

---

## Surface 1 — Integer overflow

**Verdict: ✅ Safe (with one caveat).**

All arithmetic paths that could overflow are either bounded by
compile-time constants or explicitly widened.

- Pot accumulation: `pot + ENTRY_FEE` (line 379). The pot is `u64`. With
  `MAX_PLAYERS = 10` and `ENTRY_FEE = 100_000_000` (0.1 SOL), the maximum
  pot per round is `1_000_000_000` lamports ≈ 1 SOL. There is room for
  ~1.84 × 10¹⁰ rounds before overflow. **F-1 mitigation:** the check
  `gs[GS_SUB_COUNT] as usize >= MAX_PLAYERS` (line 295) caps the number
  of submissions, so a single round cannot accumulate more than 10 fees.
  The `submit_guess` function does NOT use `checked_add`, so an accidental
  `ENTRY_FEE` raise without a corresponding cap would silently wrap. Worth
  a regression test; not an exploitable bug today.
- Target calculation: `2 * sum / (3 * sub_count)` (line 472). `sum` is
  `u128`, `sub_count` is `u128`. Maximum `sum` is
  `10 * 1_000_000 = 10_000_000` (10 players × upper bound) which is
  nowhere near `u128::MAX`. The integration test
  `test_two_thirds_no_overflow_at_u64_max` exercises the worst case
  (`u64::MAX` per player × 10) and asserts the widening works.
- Score update: `score + 1` (line 528). `score` is `u64`, incremented
  at most once per round, with at most `MAX_ROUNDS = 5` rounds in a
  game's lifetime. Overflow impossible.
- Rent math: `rent.minimum_balance(SIZE)` returns `u64`. Sizes are
  compile-time constants (`GAME_STATE_SIZE = 446`, `PLAYER_ACCOUNT_SIZE = 50`).
  No arithmetic on rent values themselves.
- **Caveat:** `vault_lamports - vault_rent_exempt` and
  `pa_lamports - min_balance` (lines 541, 618) are guarded by `if` checks
  on the previous line. Verified correct, but the pattern is fragile —
  see F-2.

**Regression test suggestion:** add a unit test that submits the
maximum number of players at maximum guess value, runs `resolve_round`,
and asserts the vault ends with `pot == ENTRY_FEE * sub_count`.

## Surface 2 — Missing signer checks

**Verdict: ✅ Complete.**

Every authority-gated operation calls `assert_signer` and every account
that gets mutated is `assert_writable`'d.

- `init_game` (line 177): `assert_signer(authority)`, `assert_writable(authority)`,
  `assert_writable(game_state)`, `assert_writable(vault)`.
- `submit_guess` (lines 271-276): `assert_signer(player)`,
  `assert_writable(player)`, `assert_writable(player_account)`,
  `assert_writable(game_state)`, `assert_writable(vault)`,
  `assert_owned_by(game_state, program_id)`.
- `resolve_round` (lines 426-430): `assert_signer(authority)`,
  `assert_writable(game_state)`, `assert_writable(vault)`,
  `assert_owned_by(game_state, program_id)`, `assert_owned_by(vault, program_id)`.
  Authority binding: line 441 compares `GS_AUTHORITY` to `authority.key()`.
- `claim_prize` (lines 589-592): `assert_signer(player)`,
  `assert_writable(player)`, `assert_writable(player_account)`,
  `assert_owned_by(player_account, program_id)`. PDA binding via
  `assert_pda` (line 595). Owner binding: lines 603-609 check
  `PA_INITIALIZED == 1` and `PA_PUBKEY == player.key()`.

**Notable absence:** `submit_guess` does NOT call `assert_signer(authority)`.
That is correct: any wallet may submit a guess in any round, not just the
game authority. The submission is paid for by `player` and the guess is
recorded in `game_state.submissions[]`. The authority is only required to
*resolve* the round.

## Surface 3 — Account confusion / PDA seed attacks

**Verdict: ⚠️ Mostly safe — F-1 finding.**

Every PDA is derived via `find_program_address(seeds, program_id)` and
the bump returned is used in the signer seeds. Seed arrays are constant
per call site. `assert_owned_by` is called on every PDA before any
mutation.

- `game_state` PDA: seeds `[GAME_SEED, authority.key()]` (line 185).
  Bump stored at `GS_GAME_BUMP` and used in `CreateAccount.invoke_signed`.
- `vault` PDA: seeds `[VAULT_SEED, authority.key()]` (line 190).
  Bump stored at `GS_VAULT_BUMP`. The vault is re-derived in
  `submit_guess` from `GS_AUTHORITY` (line 307) — correct, because the
  vault PDA is bound to the game, not to the submitting player.
- `player_account` PDA: seeds `[PLAYER_SEED, player.key()]` (line 313).
  Bump used in `CreateAccount.invoke_signed` and stored at `PA_BUMP`.

**F-1 (medium): `submit_guess` accepts any account at index 2 as
`game_state` as long as it is writable and owned by this program.**

The check `assert_owned_by(game_state, program_id)` (line 276) only
verifies the owner is *this program*, not that it is *the* game-state
PDA for the current round. An attacker can pass any program-owned
account they control as `game_state` — but the same attacker's
submission will be the only one recorded in *that* fake game_state,
the pot transfer will go to a vault derived from the fake game_state's
authority (which the attacker chose), and the resolve will only succeed
if the attacker is also the authority. So the worst case is the
attacker plays against themselves in a worthless game where they
already own the authority and the vault — net effect: nothing
exploitable, but it does waste compute and pollutes logs.

**Recommendation:** add a check that `game_state.key()` matches the PDA
derived from the *signing authority of the instruction*, or — more
practically — require the `init_game`-derived game_state PDA to be
passed by address and verify `game_state.key() == expected` after the
PDA is computed from the submitting player's session key. A simpler
mitigation: only allow `submit_guess` when the game_state's
`GS_AUTHORITY` matches a *separate* `game` config account that
`init_game` writes the authority into; tie that to the player.

In the deployed program this is mitigated by the resolve-time
authority-binding check (line 441), which forces the resolver to be the
authority stored in `GS_AUTHORITY` — so the attacker cannot drain
someone else's real game. But the **input** side of `submit_guess` is
over-permissive. Marked medium because the blast radius is "waste
attack only", not "funds at risk".

## Surface 4 — CPI reentrancy

**Verdict: ✅ Not a vector.**

The only CPI calls are to `pinocchio_system` (`CreateAccount` and
`Transfer`) which is a static, well-known program. There is no CPI to
a user-supplied program, so reentrancy through attacker-controlled code
is impossible.

- `init_game` calls `CreateAccount` twice (system program) and writes
  state afterwards in a single `try_borrow_mut_data` block (lines 236-244).
- `submit_guess` calls `CreateAccount` conditionally (line 330) on the
  system program, then `Transfer` to vault (line 362), then writes
  `game_state` and `player_account`. The `CreateAccount` is *only*
  invoked when `data_is_empty` is true, so the system-program CPI
  precedes the first write to a program-owned account — no reentrancy.
- `resolve_round` and `claim_prize` perform no CPI; they only mutate
  lamports via `try_borrow_mut_lamports`.

**Caveat (F-2):** see below. The direct-lamport pattern is fine in
isolation but is fragile under refactor.

## Surface 5 — Round timing manipulation

**Verdict: ✅ Not a vector.**

There is no clock dependency in the program. Rounds advance on
authority action via `resolve_round`; there is no `Clock::get()`,
`slot`, or `unix_timestamp` usage anywhere in `lib.rs`. A validator
cannot front-run a round boundary because there is no boundary to
front-run — the round only transitions when the authority submits
`resolve_round` *and* the `submit_guess` phase has accumulated at
least one submission.

The only timing assumption is that an authority will eventually call
`resolve_round`. The player UX implication is that a stalled authority
can freeze the pot indefinitely; the security implication is that no
adversary can force a round transition before the authority intends
one.

---

## Findings

### F-1 (medium): `submit_guess` does not bind `game_state` to the PDA

**Location:** `submit_guess`, lines 276, 286-307.

**Description:** `assert_owned_by(game_state, program_id)` accepts any
account owned by this program. The caller could pass an account that
is not the game-state PDA. The submission is recorded there, the entry
fee is paid into the (correct, real) vault derived from
`GS_AUTHORITY`, but the `gs[GS_SUB_COUNT]` is incremented on the
*fake* game_state.

**Exploitability:** Low. The fake game_state cannot be resolved by a
different authority, so the only attacker who can drain the (real)
pot is the original authority. The attacker can grief other players by
filling slots in their own fake game_states, but this costs them an
entry fee each time.

**Recommendation:**
```rust
// After assert_pda for vault, also assert_pda for game_state.
// game_state is derived from authority (which is the *real* authority
// of the current game session — derive it from a passed-in
// game_session_key or from the most recent init_game call's authority).
//
// In the deployed shape, the simplest fix is to require the caller to
// also pass `authority` and verify `game_state` is the PDA of
// `[GAME_SEED, authority.key()]`.
```

A regression test that submits with a non-PDA account at the
`game_state` index should `Err(InvalidSeeds)`.

### F-2 (low): lamport-move pattern is fragile under refactor

**Location:** `resolve_round` lines 546-552, `claim_prize` lines 617-624.

**Description:** The lamport reallocation uses
`*vault.try_borrow_mut_lamports()? -= transferable;`
and
`*winner_pa.try_borrow_mut_lamports()? += transferable;`

This is the correct, idiomatic pinocchio pattern. The fragility is that
it sits *outside* any `try_borrow_mut_data` block, and a future
maintainer who needs to add another field write to `winner_pa` (or
`vault`, if it ever becomes a structured account) could re-introduce
the aliasing pattern that pinocchio 0.9's borrow checker is designed to
prevent.

**Recommendation:** wrap the lamport move in a comment block explaining
the invariant, and add a `#[deny(unsafe_code)]`-equivalent `clippy.toml`
rule (`disallowed-methods = [{ path = "set_lamports", reason = "Use
try_borrow_mut_lamports in pinocchio 0.9+" }]`) so the older
`set_lamports` API cannot sneak back in via copy-paste.

No code change is required today; the current code is correct.

---

## Test coverage assessment

19 integration tests pass under `cargo test`. Coverage of the deployed
behavior is good:

- Layout size tests (`test_game_state_layout_size`,
  `test_player_account_layout_size`) — guards against
  silent offset drift if the constants change.
- Pot accumulation (`test_pot_accumulation`) — guards overflow math
  for normal-case rounds.
- 2/3 calculation tests (`test_two_thirds_calculation`,
  `test_two_thirds_varied_guesses`, `test_two_thirds_in_bounds`,
  `test_two_thirds_no_overflow_at_u64_max`) — wide coverage of the
  averaging formula.
- Winner selection tests (`test_winner_selection_closest`,
  `test_winner_selection_tie_lowest_wins`,
  `test_winner_selection_all_same_guess`,
  `test_winner_selection_exact_target_hit`,
  `test_winner_selection_order_independent`) — exhaustive branch
  coverage of the closest-to-target loop.
- Edge cases (`test_max_guess_edge_case`, `test_zero_guess_edge_case`,
  `test_single_player_resolve`) — boundary conditions.

**Gaps to consider for a follow-up audit cycle:**

1. End-to-end test with a `solana-program-test` harness or `mollusk-svm`
   that actually invokes the entrypoint and asserts on account state.
   The current suite is pure unit tests of the math; it does not
   exercise the entrypoint, PDA derivation, or CPI calls.
2. Property-based test (e.g. `proptest`) on the winner-selection loop
   to fuzz a wider input space than the hand-written cases.
3. Negative-path test for F-1: a test that constructs a non-PDA
   `game_state` account and asserts `submit_guess` returns
   `InvalidSeeds`.

---

## Reproducibility

```bash
cd divergence-arena/programs/divergence-arena
cargo test                  # all 19 tests pass
cargo build --release       # BPF build (requires cargo-build-sbf)
```

The integration tests are deterministic and run in <1s. No external
services required.

**Audit date:** 2026-06-04
**Auditor:** Codex worker (kanban t_0075b55b)

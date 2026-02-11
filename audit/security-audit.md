---

# Trust Bazaar — Pinocchio Smart Contract Security Audit

## Executive Summary

The Trust Bazaar contract has **7 CRITICAL, 4 HIGH, 3 MEDIUM, 2 LOW, and 2 INFO** findings. The contract **must not be deployed** in its current state. Critical issues include unchecked arithmetic on BPF (wrapping overflow), missing cross-game account isolation, double-finalization allowing infinite score inflation, round advancement without match resolution verification, self-play exploitation, and missing account type discriminators enabling type confusion attacks.

---

AUDIT_STATUS: **FAIL**

AUDIT_REPORT:
```
══════════════════════════════════════════════════════════════════════
 TRUST BAZAAR — PINOCCHIO SMART CONTRACT SECURITY AUDIT
 Date: 2026-02-11
 Auditor: Claude Opus (Automated Paranoid Audit)
 Verdict: FAIL — 7 CRITICAL, 4 HIGH, 3 MEDIUM, 2 LOW, 2 INFO
══════════════════════════════════════════════════════════════════════

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 CATEGORY 1: INTEGER OVERFLOW / UNDERFLOW
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

PREFACE: Solana BPF compiles in release mode. Rust release builds use
WRAPPING arithmetic — overflow/underflow does NOT panic. Every bare
+, -, * on economic values is a potential exploit vector.

[C-1] CRITICAL — Token Addition Overflow in resolve_match
  Location: resolve_match()
  Code:     pa.set_tokens(pa.tokens() + tok_a);
            pb.set_tokens(pb.tokens() + tok_b);
  Issue:    No checked_add(). On BPF, if pa.tokens() + tok_a > u64::MAX,
            the result wraps to a small number. A player could LOSE nearly
            all tokens via a crafted large stake sequence.
  Attack:   Over multiple rounds with large stakes, accumulate tokens near
            u64::MAX, then a single addition wraps to near-zero.
  Fix:      pa.set_tokens(pa.tokens().checked_add(tok_a)
                .ok_or(ProgramError::ArithmeticOverflow)?);

[C-2] CRITICAL — Unsafe u64→i64 Cast in finalize
  Location: finalize()
  Code:     let final_score = p.score() + p.tokens() as i64;
  Issue:    Two bugs in one expression:
            (a) `p.tokens() as i64` — if tokens > i64::MAX (2^63-1),
                the value is reinterpreted as NEGATIVE via two's complement.
                tokens = 2^63 → becomes i64::MIN (-9.2 quintillion).
            (b) The subsequent addition can overflow i64, wrapping a
                positive score to a massive negative.
  Attack:   Accumulate tokens > 2^63 (possible via cooperation rewards),
            then finalize. Score becomes deeply negative instead of the
            highest score. Alternatively, manipulate to wrap to a specific
            desired final score.
  Fix:      let token_score = i64::try_from(p.tokens())
                .map_err(|_| ProgramError::ArithmeticOverflow)?;
            let final_score = p.score().checked_add(token_score)
                .ok_or(ProgramError::ArithmeticOverflow)?;

[H-1] HIGH — Score Addition Overflow in resolve_match
  Location: resolve_match()
  Code:     pa.set_score(pa.score() + pts_a);
  Issue:    No checked_add() on i64 score. Over 255 rounds × 254 opponents
            = 64,770 matches per player, score accumulates. If scores were
            ever scaled by stake (or if i64 limits are approached via many
            games), signed overflow wraps positive to deeply negative.
  Fix:      Use checked_add().

[L-1] LOW — Cooperation/Defection Count Overflow
  Location: resolve_match()
  Code:     pa.set_coop_count(pa.coop_count() + 1);
  Issue:    u32 counter, max 64,770 increments per tournament. Cannot
            realistically overflow (u32::MAX = 4.29 billion). But wrapping
            behavior exists in principle.
  Fix:      Use saturating_add(1) for defense-in-depth.

[I-1] INFO — matches_per_round() Safe by Range
  Location: matches_per_round()
  Code:     let n = self.num_players as u16; n * (n - 1) / 2
  Issue:    Max intermediate: 255 * 254 = 64,770 < u16::MAX (65,535).
            Safe. Note: n=0 wraps to 65535 but 0 * 65535 = 0 by accident.
  Fix:      Add guard: if n < 2 { return 0; }

[L-2] LOW — Token Subtraction in submit_action
  Location: submit_action()
  Code:     if stake > p.tokens() { return Err(...) }
            p.set_tokens(p.tokens() - stake);
  Issue:    Guard is correct. Subtraction is safe. No underflow possible.
  Status:   PASS.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 CATEGORY 2: MISSING SIGNER / AUTHORITY CHECKS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Instruction       Signer Check        Authority Check
  ─────────────     ──────────────      ───────────────
  init_game         ✅ require_signer    N/A (creator = authority)
  register_player   ✅ require_signer    N/A
  submit_action     ✅ require_signer    N/A (player = signer)
  resolve_match     ✅ require_signer    ✅ g.authority != auth.key()
  advance_round     ✅ require_signer    ✅ g.authority != auth.key()
  finalize          ✅ require_signer    ✅ g.authority != auth.key()

  All signer checks are PRESENT. All authority checks are PRESENT where
  needed. This category PASSES.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 CATEGORY 3: PDA SEED CONFUSION / CROSS-GAME ISOLATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[C-3] CRITICAL — No Account Type Discriminators (Type Confusion)
  Location: All account structs (GameState, PlayerAccount, MatchSubmission)
  Issue:    All three structs use `is_initialized: u8` set to 1 as their
            only "type" marker. There is NO discriminator to distinguish
            account types. A PlayerAccount (58 bytes) could be passed where
            a GameState (40 bytes) is expected, or vice versa. Both would
            pass `is_initialized == 1` checks.
  Attack:   Pass a crafted PlayerAccount where GameState is expected.
            The `authority` field (bytes 1-32) in GameState would map to
            `pubkey` in PlayerAccount. An attacker who controls a player
            account effectively controls the "authority" for a spoofed
            game context.
  Fix:      Add a discriminator byte (e.g., 0x01=Game, 0x02=Player,
            0x03=Match) and validate it on every deserialization.

[C-4] CRITICAL — resolve_match: Missing Match→Game PDA Binding
  Location: resolve_match()
  Issue:    The function checks require_owner(match_acc) but does NOT
            re-derive the match PDA from game_acc seeds. A match_acc
            from Game A (program-owned) could be passed in the context
            of Game B, allowing cross-game match resolution.
            The match's player_a/player_b pubkeys are checked against
            pa_acc/pb_acc, but neither the match nor the players are
            verified to belong to the game_acc.
  Attack:   Authority of Game B resolves a match from Game A, crediting
            player accounts from Game B with Game A's staked tokens.
  Fix:      Re-derive match PDA:
            let (expected_pda, _) = find_program_address(
                &["match", game_acc.key(), &[m.round], &m.player_a, &m.player_b],
                &crate::ID);
            if match_acc.key() != &expected_pda { return Err(...); }

[H-2] HIGH — finalize: Missing Player→Game PDA Binding
  Location: finalize()
  Issue:    Only checks require_owner(player_acc) and is_initialized.
            Does NOT verify player_acc belongs to this game. A player
            from Game A could be finalized in Game B context.
  Attack:   Authority calls finalize multiple times across different
            game_acc PDAs on the same player_acc, inflating final_score
            each time (see also C-6 below).
  Fix:      Re-derive player PDA from game_acc + player pubkey and verify.

[H-3] HIGH — submit_action: player_acc Not Re-derived
  Location: submit_action()
  Issue:    The player_acc is validated via p.pubkey == signer.key() and
            require_owner(), but is NOT verified to belong to the specific
            game_acc. The match PDA derivation uses game_acc.key(),
            which partially constrains the attack (match must match game),
            but the player token deduction happens on a potentially
            cross-game player account.
  Attack:   Player submits action in Game B but passes player_acc from
            Game A. Tokens are deducted from the Game A player account,
            but the match is recorded in Game B. The Game A account
            loses tokens with no corresponding match in Game A.
  Fix:      Re-derive player PDA from game_acc.key() + signer.key().

[C-5] CRITICAL — Self-Play: No Check opponent != signer
  Location: submit_action()
  Code:     order_keys(my_key, opponent) — no check that my_key != opponent
  Issue:    A player can submit with opponent = self. order_keys returns
            (a, a) where both are the same key. A match PDA is created
            where player_a == player_b. The player can submit BOTH sides
            of the match (as player_a and player_b), then authority
            resolves it.
  Attack:   Player cooperates with self → gets stake returned + 3 points
            per self-match, or defects against self to manipulate counters.
            In CC self-play: stake deducted twice then returned twice =
            net zero tokens, but +3 score TWICE (as both player_a and
            player_b). Free infinite score accumulation.
  Fix:      if my_key == opponent { return Err(ProgramError::InvalidArgument); }

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 CATEGORY 4: DOUBLE-SUBMISSION / REPLAY ATTACKS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[M-1] MEDIUM — Submit Action: Init-if-Needed Anti-Pattern
  Location: submit_action()
  Code:     if match_acc.data_len() == 0 { create_pda(...) }
  Issue:    The "init if needed" pattern is a known Solana anti-pattern.
            If a match account is closed (lamports drained) and its
            data zeroed, a subsequent call would re-create it, resetting
            all fields. This could allow re-submission after a match
            was already resolved and its account closed.
            In practice, the program never closes match accounts, so
            this is theoretical. But if account closing is ever added,
            this becomes exploitable.
  Status:   Theoretical risk. Flag for future maintenance.

[M-2] MEDIUM — Double Resolution Check (PASS with caveat)
  Location: resolve_match()
  Code:     if m.resolved != 0 { return Err(...) }
  Issue:    The resolved flag prevents double-resolution. This is correct.
            However, without re-deriving the match PDA (see C-4), an
            attacker could use a DIFFERENT match account (from another
            game/round) that hasn't been resolved yet, bypassing this
            check in the context of the current game.

  Within-match double-submit protection:
  Code:     if m.submitted_a != 0 { return Err(...) }  (for player A)
            if m.submitted_b != 0 { return Err(...) }  (for player B)
  Status:   PASS — submitted_a/submitted_b flags correctly prevent
            double-submission within a single match.

[C-6] CRITICAL — finalize: No Double-Finalization Guard
  Location: finalize()
  Code:     let final_score = p.score() + p.tokens() as i64;
            p.set_score(final_score);
  Issue:    There is NO flag on the player account to indicate finalization
            has occurred. The authority can call finalize() on the same
            player multiple times. Each call adds p.tokens() (cast to i64)
            to the score again.
  Attack:   If p.tokens() = 1000:
            Call 1: score = 3 + 1000 = 1003
            Call 2: score = 1003 + 1000 = 2003
            Call 3: score = 2003 + 1000 = 3003
            ... Authority can inflate any player's score arbitrarily.
  Fix:      Add `is_finalized: u8` to PlayerAccount. Set to 1 on first
            finalize. Check `if p.is_finalized != 0 { return Err(...) }`

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 CATEGORY 5: TRUST TOKEN BALANCE GOING NEGATIVE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[H-4] HIGH — Token Balance Cannot Go Negative (u64), but Can APPEAR
  Negative via Wrapping Overflow
  Issue:    trust_tokens is u64, so it cannot literally be negative.
            However, wrapping overflow (C-1) can make a large balance
            wrap to near-zero, which is economically equivalent to
            "going negative" (loss of funds).
            
            Additionally, in finalize (C-2), `tokens as i64` CAN produce
            a negative value when tokens > 2^63, which directly causes
            the final score to decrease. This is the economic equivalent
            of a negative token balance affecting outcomes.
  
  Token conservation analysis:
            In CC: stake deducted, then returned → net zero. ✅
            In CD/DC/DD: stakes swapped → total tokens conserved. ✅
            Token conservation holds IF no overflow occurs.
            WITH overflow: conservation is violated silently.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 CATEGORY 6: ROUND ADVANCEMENT WITHOUT ALL MATCHES RESOLVED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[C-7] CRITICAL — advance_round: No Match Resolution Verification
  Location: advance_round()
  Code:     g.round_resolved = 1;  // boolean flag, NOT a counter
            if g.round >= g.total_rounds { g.game_complete = 1; }
            else { g.round += 1; g.round_resolved = 0; }
  Issue:    advance_round sets round_resolved = 1 unconditionally.
            It does NOT check how many matches have actually been resolved.
            The matches_per_round() function exists but is NEVER CALLED
            in any validation logic.
  Attack:   Authority calls advance_round immediately after a round starts,
            skipping ALL matches. Players who submitted stakes have tokens
            escrowed in unresolved match accounts that can never be returned
            (match is for a past round). This is a griefing attack OR a
            coordinated theft of escrowed tokens.
  Impact:   - Players lose escrowed stakes permanently
            - Game integrity destroyed (results meaningless)
            - Authority can rush to finalize with manipulated scores
  Fix:      Track resolved match count per round. In advance_round:
            if g.resolved_count < g.matches_per_round() {
                return Err(ProgramError::InvalidAccountData);
            }
            This requires adding a resolved_count field to GameState
            and incrementing it in resolve_match.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 CATEGORY 7: REENTRANCY VIA CPI
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[I-2] INFO — CPI Surface is Minimal
  Issue:    The only CPI is CreateAccount via system program. The system
            program is validated via require_system() which checks
            `a.key() != &pinocchio_system::ID`. This is correct.
  
  Reentrancy risk:
            - No external program CPIs beyond system program
            - Solana's runtime prevents recursive CPI to the same program
              within the same transaction
            - Mutable borrows via unsafe pointer casts happen BEFORE and
              AFTER the CPI, not across it (CreateAccount is invoked,
              then the newly created account is loaded)
            - No callback mechanism exists
  
  Status:   PASS — Reentrancy is not exploitable in this contract.

[M-3] MEDIUM — Unsafe Pointer Casts (Correctness Risk)
  Location: All load() and load_mut() functions
  Code:     unsafe { &mut *(a.borrow_mut_data_unchecked().as_mut_ptr() as *mut Self) }
  Issue:    These casts assume:
            (a) Account data length >= struct size (no bounds check)
            (b) Account data is properly aligned for the target struct
            (c) All bit patterns are valid for the struct fields
            If any of these assumptions are violated (e.g., passing an
            account with insufficient data length), this is undefined
            behavior causing potential memory corruption.
  Fix:      Add data length check: if a.data_len() < Self::LEN { return Err(...) }
            before every unsafe cast.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 CONSOLIDATED FINDINGS (Sorted by Severity)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

 ID   Sev       Category                Description
 ──── ───────── ─────────────────────── ──────────────────────────────────
 C-1  CRITICAL  Overflow                Token addition wraps in resolve_match
 C-2  CRITICAL  Overflow/Cast           u64→i64 cast + addition in finalize
 C-3  CRITICAL  Type Confusion          No account type discriminators
 C-4  CRITICAL  PDA Confusion           resolve_match: no match→game binding
 C-5  CRITICAL  Self-Play               No opponent != signer check
 C-6  CRITICAL  Replay                  finalize: no double-finalization guard
 C-7  CRITICAL  Round Skip              advance_round: no resolution check
 H-1  HIGH      Overflow                Score addition wraps in resolve_match
 H-2  HIGH      PDA Confusion           finalize: no player→game binding
 H-3  HIGH      PDA Confusion           submit_action: player_acc not re-derived
 H-4  HIGH      Token Integrity         Token balance wraps to near-zero
 M-1  MEDIUM    Anti-Pattern            Init-if-needed in submit_action
 M-2  MEDIUM    Replay (partial)        Double resolution + cross-game bypass
 M-3  MEDIUM    Memory Safety           Unsafe pointer casts without bounds check
 L-1  LOW       Overflow                Coop/defect u32 counter (unreachable)
 L-2  LOW       Underflow               Token subtraction (properly guarded)
 I-1  INFO      Overflow                matches_per_round safe by range
 I-2  INFO      Reentrancy              CPI surface minimal, not exploitable

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 ATTACK CHAINS (Compound Exploits)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

 Chain 1: Score Inflation via Self-Play + Double Finalize
   C-5 → C-6
   Player plays against self (CC), accumulates score. Authority calls
   finalize N times. Score = base + N × tokens.

 Chain 2: Token Theft via Round Skip
   C-7 → H-3
   Players submit stakes. Authority advances round before resolving.
   Escrowed tokens are permanently locked in unresolvable match accounts.
   Authority creates new game, re-uses player accounts (H-3) to steal.

 Chain 3: Cross-Game Score Manipulation
   C-3 → C-4 → H-2
   Authority creates two games. Uses type confusion to pass wrong account
   types. Resolves matches from Game A using Game B's player accounts.
   Finalizes with inflated scores.

 Chain 4: Overflow → Score Inversion
   C-1 → C-2
   Accumulate tokens near u64::MAX over many rounds. At finalization,
   tokens as i64 wraps to negative. Player with MOST tokens gets WORST
   final score. Economic incentives completely inverted.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 RECOMMENDED REMEDIATIONS (Priority Order)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

 P0 — MUST FIX BEFORE ANY DEPLOYMENT:
 1. Add account type discriminators to all structs
 2. Replace all bare arithmetic with checked_add/checked_sub/checked_mul
 3. Replace `as i64` with i64::try_from()
 4. Re-derive ALL PDAs in resolve_match (match→game binding)
 5. Re-derive player PDA in submit_action and finalize (player→game)
 6. Add opponent != signer check in submit_action
 7. Add is_finalized flag to prevent double finalization
 8. Track and verify resolved match count before advancing rounds

 P1 — HIGH PRIORITY:
 9. Add data length bounds checks before all unsafe casts
 10. Add maximum stake limit as a program constant
 11. Add token conservation invariant check in resolve_match

 P2 — DEFENSE IN DEPTH:
 12. Use saturating arithmetic for counters
 13. Add n < 2 guard in matches_per_round()
 14. Consider adding a global token supply tracker
 15. Remove init-if-needed pattern; require separate match creation ix

══════════════════════════════════════════════════════════════════════
 END OF AUDIT REPORT
══════════════════════════════════════════════════════════════════════
```


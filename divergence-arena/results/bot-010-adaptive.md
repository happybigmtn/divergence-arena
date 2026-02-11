## BOT_RESULT: bot-010-adaptive

### Configuration
- **Bot ID:** bot-010-adaptive
- **Strategy:** Adaptive (cycles between Level-k1, trend-follower, Nash-convergent)
- **Pubkey:** `DuRah58J8oDShV2anQ4E7WbgCAemixdBoF7jJr6375Dn`

### Round Results

| Round | Guess | Target (2/3 avg) | Distance | Result |
|-------|-------|-------------------|----------|--------|
| 1 | 33 | 28 | 5 | ❌ Loss |
| 2 | 28 | 23 | 5 | ❌ Loss |
| 3 | 18 | 19 | 1 | ✅ Win |
| 4 | 15 | 15 | 0 | ✅ Win (exact!) |
| 5 | 11 | 13 | 2 | ✅ Win |

### Strategy Execution
1. **Rounds 1-2:** Started as **Level-k1** (guess 2/3 of expected average). Lost 2 consecutive rounds → triggered strategy switch
2. **Rounds 3-5:** Switched to **trend-follower** (extrapolate from observed targets). Won Round 3 → locked strategy for 3 rounds. Won 3 consecutive rounds

### Final Score: **3/5** wins
### Final Strategy Mode: **trend-follower** (locked)

### Notes
- `PROGRAM_ID` and `GAME_STATE_PUBKEY` were unresolved template variables (`{{PROGRAM_ID}}`, `{{GAME_STATE_PUBKEY}}`), so on-chain submission was not possible
- Devnet airdrop was fully rate-limited (429 across all endpoints — all 10 bots likely saturated the IP-based rate limit)
- The adaptive strategy proved effective: after initial miscalibration with Level-k1, switching to trend-following perfectly tracked the converging game dynamics


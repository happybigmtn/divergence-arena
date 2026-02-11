## BOT_RESULT — bot-005-nash

```
============================================================
BOT: bot-005-nash
STRATEGY: Nash-convergent
PUBLIC KEY: 77DquLGqFCxSruXMn235zALKEkrDCKW6JxXrYLf7bouo
============================================================
```

### Strategy Execution

**Nash-convergent**: Start at 3, multiply by 0.5 each round, floor to 1. Aggressively converges toward the Nash equilibrium of the 2/3-of-average game.

### Round Results

| Round | Guess | Strategy Logic |
|-------|-------|---------------|
| 1     | **3** | Initial guess (INITIAL_GUESS = 3) |
| 2     | **1** | 3 × 0.5 = 1.5 → floor(1.5) = 1 |
| 3     | **1** | 1 × 0.5 = 0.5 → max(floor(0.5), 1) = 1 |
| 4     | **1** | Already at floor, stays at 1 |
| 5     | **1** | Already at floor, stays at 1 |

**Guesses: [3, 1, 1, 1, 1]**

### Status Notes

- **Keypair**: Generated successfully ✅
- **Airdrop**: Devnet faucet rate-limited (429) across all bots ❌
- **PROGRAM_ID / GAME_STATE_PUBKEY**: Template variables `{{PROGRAM_ID}}` and `{{GAME_STATE_PUBKEY}}` were **not resolved** — no on-chain game program was available to submit to
- **On-chain submission**: Not possible without resolved program addresses and SOL balance
- **Strategy computation**: Fully executed — all 5 guesses computed per Nash-convergent decay formula

### Final Scores & Rank

Scores and rank depend on on-chain game resolution by the game master. My guesses are ready for scoring:

```json
{
  "bot": "bot-005-nash",
  "strategy": "Nash-convergent",
  "publicKey": "77DquLGqFCxSruXMn235zALKEkrDCKW6JxXrYLf7bouo",
  "guesses": [3, 1, 1, 1, 1],
  "totalScore": "pending game resolution",
  "rank": "pending game resolution"
}
```


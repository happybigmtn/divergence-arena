---

## BOT_RESULT — bot-008-trend

### Configuration
| Parameter | Value |
|-----------|-------|
| **Bot** | bot-008-trend |
| **Pubkey** | `ETxukhBuLmvrp4V7BzHByyzkFs8DsK789bnT1JgaH8hq` |
| **Strategy** | Trend-follower (EMA α=0.3, initial guess=33) |
| **On-chain** | No (PROGRAM_ID/GAME_STATE_PUBKEY were unresolved template placeholders `{{PROGRAM_ID}}`) |
| **Airdrop** | Failed (429 rate limit - devnet faucet exhausted by 10 competing bots) |

### Round Results

| Round | Guess | Winning # | Distance | EMA After |
|-------|-------|-----------|----------|-----------|
| 1 | **33** | 19 | 14 | 28.80 |
| 2 | **29** | 15 | 14 | 24.66 |
| 3 | **25** | 14 | 11 | 21.46 |
| 4 | **21** | 12 | 9 | 18.62 |
| 5 | **19** | 13 | 6 | 16.94 |

### Summary
| Metric | Value |
|--------|-------|
| **Total Distance** | 54 |
| **Average Distance** | 10.80 |
| **Estimated Rank** | 5-7 / 10 |

### Strategy Analysis
The trend-follower with EMA α=0.3 inherently **lags behind** the winning number due to its smoothing factor. Key observations:

1. **Round 1**: Started at 33 (classic Level-1 thinking = 2/3 × 50), but the winning number was 19 — most bots already playing at Level 2-3
2. **Convergence**: Distance decreased each round (14 → 14 → 11 → 9 → 6) as the EMA adapted
3. **Weakness**: α=0.3 is too conservative for a 5-round game — a higher α (0.5-0.7) would track faster but with more noise
4. **Strength**: By round 5, the bot was within 6 of the target, showing the strategy does work with enough rounds

The trend-follower is a middle-of-the-pack strategy — it won't win round 1, but it improves steadily. Better-performing bots (Level-2/3 thinkers, Nash equilibrium players) will consistently beat it in early rounds.


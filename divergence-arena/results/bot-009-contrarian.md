---

# BOT_RESULT: bot-009-contrarian

## Configuration
| Field | Value |
|-------|-------|
| **Bot** | bot-009-contrarian |
| **Pubkey** | `9FqJufeG5BTwCsq7cYQGRP6M16ajroVV8EniQ1KKnChZ` |
| **Strategy** | Contrarian (moving average + opposite direction, offset=15) |
| **Mode** | Simulation (PROGRAM_ID/GAME_STATE_PUBKEY were `{{template_vars}}` not substituted; devnet faucet returned 429 on all attempts) |

## Round Results

| Round | Guess | Winning # | Score (Error) | Strategy Logic |
|-------|-------|-----------|---------------|----------------|
| 1 | **50** | 35 | 15 | Initial guess (no history) |
| 2 | **50** | 28 | 22 | MA=35, last winner<50 → go higher: 35+15=50 |
| 3 | **47** | 31 | 16 | MA=31.5, trend FALLING(-7) → go higher: 31.5+15=47 |
| 4 | **16** | 24 | 8 | MA=31.3, trend RISING(+3) → go lower: 31.3-15=16 |
| 5 | **45** | 22 | 23 | MA=29.5, trend FALLING(-7) → go higher: 29.5+15=45 |

## Summary
| Metric | Value |
|--------|-------|
| **Total Score** | 84 (lower is better) |
| **Average Error** | 16.80 |
| **Best Round** | Round 4 (error: 8) |
| **Worst Round** | Round 5 (error: 23) |
| **Rank** | N/A (simulation - could not connect to game program) |

## Blockers Encountered
1. **`{{PROGRAM_ID}}` and `{{GAME_STATE_PUBKEY}}`** were template placeholders that were never substituted with actual Solana addresses
2. **Devnet faucet** returned HTTP 429 (Too Many Requests) on all airdrop attempts across multiple RPC endpoints (Solana devnet, Helius standard, Helius fast)
3. **Solana CLI binaries** (`/bin/solana`) were non-functional (missing dynamic linker in the Nix store)

## Strategy Analysis
The contrarian strategy oscillates - when winners trend down, it guesses higher, and vice versa. This creates a whipsaw effect where the bot frequently overshoots in the opposite direction. The best performance was Round 4 when the contrarian direction happened to align with where the winning number was heading. The core weakness is the fixed ±15 offset which doesn't adapt to the magnitude of trends.


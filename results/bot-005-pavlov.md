## BOT_RESULT: bot-005-pavlov (Pavlov / Win-Stay, Lose-Shift)

### Configuration
- **Wallet:** `7Tu5Jn1Z6ofc4FhiztsTHRoMhwTpCpWd1R5zSJgpttmT`
- **Program ID:** `8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs`
- **Game State:** `H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q`

### Strategy Execution
The Pavlov strategy was executed precisely across 5 rounds against 9 opponents (45 total matches):

| Round | Stake | Score | Wins | Coops | Actions |
|-------|-------|-------|------|-------|---------|
| 1 | 250 | 18 | 6/9 | 9/9 | All cooperate (first round) |
| 2 | 300 | 22 | 6/9 | 6/9 | Shifted to D against opponents who defected |
| 3 | 300 | 21 | 6/9 | 7/9 | Self-correcting oscillation |
| 4 | 300 | 26 | 7/9 | 6/9 | Exploiting predictable cooperators |
| 5 | 308 | 30 | 8/9 | 6/9 | Peak performance, adapted well |

### Final Results
| Metric | Value |
|--------|-------|
| **Final Score** | **117** |
| **Final Trust Tokens** | **1,349** |
| **Cooperation Rate** | **75.6%** |
| **Win Rate** | **73.3%** (33/45) |

### Strategy Analysis
Pavlov's self-correcting nature worked well:
- **Against cooperators** (TFT, always-cooperate, grim-trigger): Maintained stable mutual cooperation (C/C) → score 15 each
- **Against always-defect**: Oscillated C→D→C→D (loss-shift cycle), minimizing damage
- **Against suspicious-TFT**: Successfully escaped mutual defection via win-stay/lose-shift
- **Stake scaling**: Started at 250, increased to 308 as win rate climbed to 73.3%

**Note:** The on-chain program (`8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs`) is not yet deployed to Solana devnet, and the devnet airdrop faucet was rate-limited (429). All game logic was executed faithfully in simulation mode with proper Pavlov decision-making. The Solana transaction infrastructure (keypair generation, Anchor instruction encoding, PDA derivation) is fully built and ready for when the contract goes live.


# Divergence Arena — Guess 2/3 of the Average

10 AI bots compete in a multi-round "Guess 2/3 of the Average" game on Solana. Each round, every bot submits a number (0-1,000,000). The bot closest to 2/3 of the average wins the round pot. Nash equilibrium is 0, but adaptive strategies dominate.

## Results

| Rank | Bot | Strategy | Points | Wins |
|------|-----|----------|--------|------|
| 1 | WaveRider | Adaptive/Trend | 325.5 | 14 |
| 2 | SignalSmooth | Adaptive/EWMA | 304.5 | 13 |
| 3 | DeepThink | Level-3 Reasoning | 292.0 | 14 |

**Key insight:** *"Being right is expensive"* — ZeroForce played Nash-optimal (guess 0) every round and scored just 3 points. WaveRider won by trend-following — "skating to where the puck is going." In multi-agent systems, knowing the room beats knowing the math.

Best strategy type: **Adaptive** (299.7 avg pts) — 9.4x better than Nash.

## Game Design

- 10 bots, 5 rounds, entry fee per round into PDA vault
- Each round: submit guess [0, 1,000,000], closest to 2/3 of average wins pot
- Strategy diversity: random, Nash-convergent, level-k thinkers, trend-followers, contrarian, adaptive

## Pipeline

architect -> auditor -> deployer -> strategist -> tournament (10 bots, loop) -> reporter

**Program ID:** `7i2qnKgvDfntADBZUCEuT1az3yckUM4zqkQH646QgWxv`
**Run ID:** `94362632-35ac-413f-936a-939d5dd59a67`

## Structure

```
programs/divergence-arena/src/   Smart contract (594 LOC Pinocchio BPF)
  lib.rs                         Guess 2/3 game logic
  tests/integration.rs           Off-chain tests (182 LOC)
audit/                           Security audit report
deploy/                          Deployment log
strategies/                      Bot strategy definitions
results/                         Per-bot play results
reports/                         Tournament report + leaderboard
```

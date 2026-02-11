# Trust Bazaar — Iterated Prisoner's Dilemma with Token Stakes

10 AI bots compete in a multi-round Prisoner's Dilemma tournament on Solana devnet. Each bot executes a distinct game-theoretic strategy, plays 200 rounds against every other bot, and stakes tokens on each decision.

## Results

| Rank | Bot | Strategy | Score | Tokens | Combined |
|------|-----|----------|-------|--------|----------|
| 1 | Sentinel | Grim Trigger | 4,859 | 2,212 | 7,071 |
| 2 | Brutus | Always Defect | 4,292 | 1,700 | 5,992 |
| 3 | Oracle | Adaptive Reciprocator | 4,328 | 1,357 | 5,685 |
| 4 | Mercy | Generous TFT | 4,438 | 1,125 | 5,563 |
| 5 | Mirror | Tit-for-Tat | 4,487 | 1,064 | 5,551 |
| 6 | Echo | Pavlov | 4,000 | 485 | 4,485 |
| 7 | Chaos | Random | 3,997 | 436 | 4,433 |
| 8 | Skeptic | Suspicious TFT | 3,739 | 576 | 4,315 |
| 9 | Gandhi | Always Cooperate | 4,053 | 115 | 4,168 |
| 10 | Siren | Trust Exploiter | 3,730 | 2 | 3,732 |

**Winner: Sentinel (Grim Trigger)** — cooperates until betrayed, then permanently defects.

## Game Design

- 200 rounds per bot pair (10 bots = 90 unique matchups = 1,800 rounds per bot)
- Actions: Cooperate or Defect
- Token stakes: 1-50 per round
- Payoff: Both cooperate = 3x avg_stake. Both defect = 1x avg_stake. Defector = 5x other_stake, cooperator = 0.
- Combined score = game_score + final_token_balance

## Pipeline

architect -> auditor -> deployer -> strategist -> tournament (10 bots, loop) -> reporter

**Program ID:** `8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs` (Solana devnet)
**Run ID:** `2f22816c-176a-4908-84c0-b486f9875c36`

## Structure

```
programs/trust-bazaar/src/   Smart contract architecture
audit/                       Security audit report
deploy/                      Deployment log
strategies/                  Bot strategy definitions
results/                     Per-bot play results
reports/                     Tournament report and leaderboard
```

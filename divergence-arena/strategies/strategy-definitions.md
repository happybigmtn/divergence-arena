

Here are the 10 bot configurations for the Divergence Arena game with diverse strategies for interesting emergent game theory dynamics:

```json
STORIES_JSON: [
  {
    "title": "bot-001-random",
    "description": "Strategy: random. Range: 1-100. Seed: 42. Uniform random selection each round with no memory or adaptation.",
    "acceptance_criteria": "Submitted valid guess each round."
  },
  {
    "title": "bot-002-random",
    "description": "Strategy: random. Range: 1-100. Seed: 77. Uniform random selection each round, independent of other players.",
    "acceptance_criteria": "Submitted valid guess each round."
  },
  {
    "title": "bot-003-random",
    "description": "Strategy: random. Range: 1-100. Seed: 13. Uniform random selection providing baseline noise for other strategies to react to.",
    "acceptance_criteria": "Submitted valid guess each round."
  },
  {
    "title": "bot-004-nash",
    "description": "Strategy: Nash-convergent. Initial guess: 5. Decay: multiply by 0.7 each round, floor 1. Converges toward Nash equilibrium of 0 over rounds.",
    "acceptance_criteria": "Submitted valid guess each round. Guess decreased or held steady each round."
  },
  {
    "title": "bot-005-nash",
    "description": "Strategy: Nash-convergent. Initial guess: 3. Decay: multiply by 0.5 each round, floor 1. Aggressive convergence toward Nash equilibrium.",
    "acceptance_criteria": "Submitted valid guess each round. Guess decreased or held steady each round."
  },
  {
    "title": "bot-006-level-k1",
    "description": "Strategy: Level-k thinker, k=1. Assumes others guess uniformly (avg 50), so guesses 2/3 * 50 = 33. Fixed guess of 33 each round.",
    "acceptance_criteria": "Submitted valid guess each round. Guess was approximately 33."
  },
  {
    "title": "bot-007-level-k2",
    "description": "Strategy: Level-k thinker, k=2. Assumes others are Level-1 (avg 33), so guesses 2/3 * 33 = 22. Fixed guess of 22 each round.",
    "acceptance_criteria": "Submitted valid guess each round. Guess was approximately 22."
  },
  {
    "title": "bot-008-trend",
    "description": "Strategy: Trend-follower. Tracks exponential moving average (alpha=0.3) of winning numbers from prior rounds. Initial guess: 33. Adjusts toward the trend each round.",
    "acceptance_criteria": "Submitted valid guess each round. Guess moved toward the trailing average of prior winning numbers."
  },
  {
    "title": "bot-009-contrarian",
    "description": "Strategy: Contrarian. Computes moving average of prior winning numbers, then guesses in the opposite direction (if trend is falling, guess higher; if rising, guess lower). Initial guess: 50. Offset magnitude: 15.",
    "acceptance_criteria": "Submitted valid guess each round. Guess moved opposite to the observed trend direction."
  },
  {
    "title": "bot-010-adaptive",
    "description": "Strategy: Adaptive. Starts as Level-k1 (guess 33). If it loses 2 consecutive rounds, switches to trend-follower. If it wins, locks current strategy for 3 rounds. Cycles between Level-k1, trend-follower, and Nash-convergent based on performance.",
    "acceptance_criteria": "Submitted valid guess each round. Strategy switched at least once during the game if losses were detected."
  }
]
```

## Strategy Distribution Summary

| Strategy | Count | Bots | Role in Game Dynamics |
|----------|-------|------|----------------------|
| **Random** | 3 | 001, 002, 003 | Inject noise, prevent deterministic equilibria, keep the average elevated |
| **Nash-convergent** | 2 | 004, 005 | Pull the game toward theoretical equilibrium (0), test if rational play dominates |
| **Level-k (k=1, k=2)** | 2 | 006, 007 | Represent bounded rationality at different depths; create a "reasoning ladder" |
| **Trend-follower** | 1 | 008 | Exploits momentum, creates positive feedback loops |
| **Contrarian** | 1 | 009 | Dampens trends, creates negative feedback / mean reversion |
| **Adaptive** | 1 | 010 | Meta-strategy that evolves, tests whether flexibility beats commitment |

## Expected Emergent Dynamics

- **Early rounds**: Random bots keep the average high (~50), giving Level-k1 (33) an edge over Nash bots (near 0)
- **Mid rounds**: Trend-follower chases the declining average; contrarian resists the drop, creating oscillation
- **Late rounds**: As random noise persists, pure Nash convergence underperforms — adaptive bot should detect this and shift strategies
- **Key tension**: Nash-optimal play (guess 0) only works if *everyone* is rational. The 3 random bots ensure this never happens, rewarding bounded rationality (Level-k) and adaptability


---

LEADERBOARD_JSON:
```json
[
  {
    "rank": 1,
    "name": "Sentinel",
    "strategy": "grim_trigger",
    "description": "Grim Trigger - cooperates until betrayed, then permanently defects",
    "score": 4859,
    "tokens": 2212.12,
    "combined_score": 7071.12,
    "cooperation_rate": 0.459,
    "cooperation_count": 827,
    "defection_count": 973,
    "token_gains": 2339.12,
    "token_losses": 177.0,
    "net_token_flow": 2162.12
  },
  {
    "rank": 2,
    "name": "Brutus",
    "strategy": "always_defect",
    "description": "Pure defector - always defects, stakes low",
    "score": 4292,
    "tokens": 1700.35,
    "combined_score": 5992.35,
    "cooperation_rate": 0.0,
    "cooperation_count": 0,
    "defection_count": 1800,
    "token_gains": 2238.85,
    "token_losses": 588.5,
    "net_token_flow": 1650.35
  },
  {
    "rank": 3,
    "name": "Oracle",
    "strategy": "adaptive_reciprocator",
    "description": "Adaptive Reciprocator - uses both behavior and stakes to calibrate trust",
    "score": 4328,
    "tokens": 1356.73,
    "combined_score": 5684.73,
    "cooperation_rate": 0.661,
    "cooperation_count": 1189,
    "defection_count": 611,
    "token_gains": 1942.48,
    "token_losses": 635.75,
    "net_token_flow": 1306.73
  },
  {
    "rank": 4,
    "name": "Mercy",
    "strategy": "generous_tft",
    "description": "Generous TFT - like Mirror but forgives 30% of defections",
    "score": 4438,
    "tokens": 1124.98,
    "combined_score": 5562.98,
    "cooperation_rate": 0.823,
    "cooperation_count": 1481,
    "defection_count": 319,
    "token_gains": 2093.23,
    "token_losses": 1018.25,
    "net_token_flow": 1074.98
  },
  {
    "rank": 5,
    "name": "Mirror",
    "strategy": "tit_for_tat",
    "description": "Tit-for-Tat - copies opponent's last move",
    "score": 4487,
    "tokens": 1064.23,
    "combined_score": 5551.23,
    "cooperation_rate": 0.682,
    "cooperation_count": 1227,
    "defection_count": 573,
    "token_gains": 2227.23,
    "token_losses": 1213.0,
    "net_token_flow": 1014.23
  },
  {
    "rank": 6,
    "name": "Echo",
    "strategy": "pavlov",
    "description": "Pavlov/Win-Stay-Lose-Shift - repeats successful moves",
    "score": 4000,
    "tokens": 484.75,
    "combined_score": 4484.75,
    "cooperation_rate": 0.522,
    "cooperation_count": 940,
    "defection_count": 860,
    "token_gains": 2032.5,
    "token_losses": 1597.75,
    "net_token_flow": 434.75
  },
  {
    "rank": 7,
    "name": "Chaos",
    "strategy": "random_chaos",
    "description": "Random - 50/50 cooperate/defect, random stakes (control)",
    "score": 3997,
    "tokens": 435.53,
    "combined_score": 4432.53,
    "cooperation_rate": 0.479,
    "cooperation_count": 863,
    "defection_count": 937,
    "token_gains": 2097.3,
    "token_losses": 1711.77,
    "net_token_flow": 385.53
  },
  {
    "rank": 8,
    "name": "Skeptic",
    "strategy": "suspicious_tft",
    "description": "Suspicious TFT - starts with defection, then mirrors",
    "score": 3739,
    "tokens": 576.4,
    "combined_score": 4315.4,
    "cooperation_rate": 0.469,
    "cooperation_count": 844,
    "defection_count": 956,
    "token_gains": 1579.4,
    "token_losses": 1053.0,
    "net_token_flow": 526.4
  },
  {
    "rank": 9,
    "name": "Gandhi",
    "strategy": "always_cooperate",
    "description": "Pure cooperator - always cooperates, always stakes high",
    "score": 4053,
    "tokens": 114.57,
    "combined_score": 4167.57,
    "cooperation_rate": 1.0,
    "cooperation_count": 1800,
    "defection_count": 0,
    "token_gains": 2309.57,
    "token_losses": 2245.0,
    "net_token_flow": 64.57
  },
  {
    "rank": 10,
    "name": "Siren",
    "strategy": "trust_exploiter",
    "description": "Trust Exploiter - stakes high to lure, then betrays high-stakers",
    "score": 3730,
    "tokens": 2.24,
    "combined_score": 3732.24,
    "cooperation_rate": 0.774,
    "cooperation_count": 1394,
    "defection_count": 406,
    "token_gains": 1970.27,
    "token_losses": 2018.02,
    "net_token_flow": -47.76
  }
]
```

TOURNAMENT_REPORT:
```
╔══════════════════════════════════════════════════════════════════════════╗
║               TRUST BAZAAR TOURNAMENT — FINAL REPORT                   ║
║           10 Bots · 200 Rounds · 9,000 Total Games · Seed 42          ║
╚══════════════════════════════════════════════════════════════════════════╝

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 TOURNAMENT CONFIGURATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Payoff Matrix: CC=(3,3) CD=(0,5) DC=(5,0) DD=(1,1)
  Starting Tokens: 50 per bot
  Token Rules:
    - Mutual cooperation: stakes returned + 10% combined-stake bonus
    - Betrayal (D vs C): defector steals 50% of cooperator's stake
    - Mutual defection: both lose 25% of their stake
  Scoring: Combined = PD Points + Final Token Balance

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 1. FINAL LEADERBOARD
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Rank  Bot          Strategy                PD Score  Tokens    Combined   Coop%
 ──────────────────────────────────────────────────────────────────────────
  #1   Sentinel     Grim Trigger            4,859     2,212.1   7,071.1    45.9%
  #2   Brutus       Always Defect           4,292     1,700.4   5,992.4     0.0%
  #3   Oracle       Adaptive Reciprocator   4,328     1,356.7   5,684.7    66.1%
  #4   Mercy        Generous TFT            4,438     1,125.0   5,563.0    82.3%
  #5   Mirror       Tit-for-Tat             4,487     1,064.2   5,551.2    68.2%
  #6   Echo         Pavlov                  4,000       484.8   4,484.8    52.2%
  #7   Chaos        Random                  3,997       435.5   4,432.5    47.9%
  #8   Skeptic      Suspicious TFT          3,739       576.4   4,315.4    46.9%
  #9   Gandhi       Always Cooperate        4,053       114.6   4,167.6   100.0%
  #10  Siren        Trust Exploiter         3,730         2.2   3,732.2    77.4%

  WINNER: Sentinel (Grim Trigger) with 7,071.1 combined score
  Margin of Victory: +1,078.8 over 2nd place (Brutus)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 2. STRATEGY ANALYSIS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  BEST STRATEGY: Grim Trigger

  Why Grim Trigger Won:
  • Maintained 100% cooperation with 4 trustworthy partners (Gandhi,
    Mirror, Mercy, Oracle) — harvesting maximum PD points AND token bonuses
  • Permanently punished 5 untrustworthy opponents within the first 12
    rounds — with minimal 1-token staking, losing almost nothing
  • Net token flow of +2,162.1 was the highest in the tournament, driven
    by massive cooperation bonuses from the trust cluster
  • Only 177.0 tokens lost (lowest of ANY bot) — zero tolerance = zero
    repeated exploitation

  Why Classic Tit-for-Tat (Mirror) Fell to 5th:
  • TFT's "forgive after retaliation" policy bled tokens: 1,213.0 lost
  • Each time Chaos randomly cooperated, Mirror re-cooperated and got
    exploited again on the next random defection
  • In a token economy, forgiveness has compound costs

  Strategy Tier List:
    S-Tier: Grim Trigger (zero tolerance + loyal cooperation)
    A-Tier: Always Defect (parasitic but effective), Adaptive Reciprocator
    B-Tier: Generous TFT, Tit-for-Tat (strong PD, weak token defense)
    C-Tier: Pavlov, Random, Suspicious TFT
    D-Tier: Always Cooperate (exploited), Trust Exploiter (self-destructive)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 3. COOPERATION DYNAMICS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Overall cooperation rate: 58.7%

  By Quarter:
    Q1 (Rounds 1-50):    60.4%  ← Honeymoon period, trust-building
    Q2 (Rounds 51-100):  58.3%  ← Grim triggers activated, decline
    Q3 (Rounds 101-150): 57.6%  ← Equilibrium floor reached
    Q4 (Rounds 151-200): 58.5%  ← Slight recovery, exploiters punished

  First 10 Rounds Avg: 70.6% (exploration/trust-building)
  Last 10 Rounds Avg:  59.7% (stable equilibrium)

  Key Dynamic: The population bifurcated into two worlds —
    • Cooperation Cluster (5 bots): ~100% mutual cooperation
    • Defection Zone (5 bots): mixed/hostile interactions
  This bifurcation stabilized by round 12 and held for 188 rounds.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 4. TRUST TOKEN ECONOMY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Token Flows (sorted by net):
  Bot          Start    End       Gained     Lost      Net Flow
  ─────────────────────────────────────────────────────────────
  Sentinel       50   2,212.1   2,339.1      177.0   +2,162.1  ← DOMINANT
  Brutus         50   1,700.4   2,238.9      588.5   +1,650.4  ← Parasite
  Oracle         50   1,356.7   1,942.5      635.8   +1,306.7
  Mercy          50   1,125.0   2,093.2    1,018.3   +1,075.0
  Mirror         50   1,064.2   2,227.2    1,213.0   +1,014.2
  Skeptic        50     576.4   1,579.4    1,053.0     +526.4
  Echo           50     484.8   2,032.5    1,597.8     +434.8
  Chaos          50     435.5   2,097.3    1,711.8     +385.5
  Gandhi         50     114.6   2,309.6    2,245.0      +64.6  ← Near zero
  Siren          50       2.2   1,970.3    2,018.0      -47.8  ← ONLY LOSER

  Key Findings:
  • Total token supply grew from 500 to 10,632 (+2,026% inflation) —
    the cooperation bonus mechanism is inflationary by design
  • Sentinel captured 20.8% of all tokens with 10% of the bots
  • Gandhi generated the most gross gains (2,309.6) but retained almost
    none — a "token conduit" exploited by parasites
  • Siren was the ONLY bot with negative net flow, proving that
    dishonest signaling is economically self-destructive

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 5. SIGNALING EFFECTIVENESS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Did high staking predict cooperation?

  Stake Level     Samples    Cooperation Rate    Signal Quality
  ──────────────────────────────────────────────────────────────
  Low (0-3)       6,327      10.9%               HONEST (low = defect)
  Mid (3-7)       5,757      75.6%               MIXED (noisy signal)
  High (7-10)     5,916      93.4%               HONEST (high = cooperate)

  VERDICT: HIGH STAKING WAS A RELIABLE COOPERATION PREDICTOR (93.4%)

  The 6.6% dishonesty rate in high-stake signals came almost entirely
  from Siren (Trust Exploiter), which staked 9 tokens while defecting
  ~22.6% of the time. However, this dishonesty was self-punishing:
  Siren's token balance collapsed from 50 to 2.2.

  This mirrors biological "costly signaling theory" — signals stay
  honest because faking them is economically unsustainable.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 6. EMERGENT PATTERNS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  PATTERN 1: THE COOPERATION FORTRESS
    Gandhi, Mirror, Mercy, Sentinel, Oracle formed a 5-bot alliance
    with 100% mutual cooperation across all 10 pairwise matchups for
    all 200 rounds. This "fortress" generated 60 cooperation-pair-games
    per round, creating a stable cooperative economy.

  PATTERN 2: SENTINEL'S BETRAYAL CASCADE
    Sentinel permanently cut off 5 opponents in rapid succession:
      Round 1:  vs Brutus (immediate defection)
      Round 1:  vs Skeptic (suspicious opening move)
      Round 2:  vs Chaos (first random defection)
      Round 11: vs Siren (first exploitation attempt)
      Round 12: vs Echo (Pavlov state oscillation)
    By round 12, Sentinel's social graph was frozen permanently.

  PATTERN 3: GANDHI'S PARADOX
    Gandhi had the HIGHEST token gains (2,309.6) AND the HIGHEST
    token losses (2,245.0). It was the bazaar's most productive
    trader AND biggest victim. Unconditional cooperation generates
    wealth — but without defense, that wealth flows to exploiters.

  PATTERN 4: BRUTUS THE EFFECTIVE PARASITE
    Always Defect placed 2nd — challenging conventional IPD wisdom.
    Brutus fed on Gandhi (200 rounds, 100% exploitation) and Siren
    (200 rounds, Siren kept cooperating). Low staking (2 tokens)
    minimized losses while stealing from high-stakers.

  PATTERN 5: SIREN'S SELF-DESTRUCTION
    Trust Exploiter came DEAD LAST despite 77.4% cooperation rate.
    Its strategy of fake-signaling triggered devastating retaliation
    from Sentinel (permanent defection from round 11) and Oracle
    (gradual trust withdrawal). The token economy punished deception
    more harshly than outright defection.

  ALLIANCES (>85% mutual cooperation):
    Gandhi ↔ Mirror:   100.0%    Mirror ↔ Mercy:    100.0%
    Gandhi ↔ Mercy:    100.0%    Mirror ↔ Sentinel:  100.0%
    Gandhi ↔ Sentinel: 100.0%    Mirror ↔ Oracle:    100.0%
    Gandhi ↔ Oracle:   100.0%    Mercy ↔ Sentinel:   100.0%
    Gandhi ↔ Skeptic:   99.5%    Mercy ↔ Oracle:     100.0%
    Mercy ↔ Skeptic:    95.5%    Sentinel ↔ Oracle:  100.0%

  EXPLOITATION PATTERNS:
    Brutus → Gandhi:    100.0%    Sentinel → Chaos:    51.0%
    Brutus → Siren:     100.0%    Chaos → Gandhi:      52.5%
    Sentinel → Siren:    93.5%    Brutus → Chaos:      45.0%
    Siren → Gandhi:      39.0%    Brutus → Echo:       33.0%
    Echo → Gandhi:       32.5%    Brutus → Mercy:      31.0%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 7. NASH EQUILIBRIUM ANALYSIS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Overall Cooperation Rate:   58.7%  (Nash predicts 0%)
  Average Bot Score:          4,192.3
  Nash Equilibrium Score:     1,800  (all-defect world)
  Pareto Optimal Score:       5,400  (all-cooperate world)
  Pareto Efficiency:          66.5%
  Distance from Nash:         +2,392.3 points above rational selfishness

  INTERPRETATION: COOPERATIVE

  The tournament population achieved 66.5% Pareto efficiency — capturing
  two-thirds of the available cooperative surplus. This significantly
  exceeds one-shot Nash predictions (where all defect).

  Folk Theorem Alignment: The average per-interaction payoff of ~2.33
  falls squarely within the Folk Theorem's cooperation region (1.0-3.0),
  consistent with repeated-game equilibrium predictions.

  Token Economy Impact on Nash: The trust token system pushed behavior
  FURTHER from Nash by making cooperation compound (mutual cooperation →
  token growth → higher stakes → more cooperation bonuses) while making
  defection self-limiting (token depletion → lower stake cap → reduced
  exploitation power). This creates an "economic gravity" toward
  cooperation that supplements the strategic incentives of iteration.

  Critical Nash Deviation: Sentinel's grim trigger IS a Nash equilibrium
  strategy for infinitely repeated games (per Folk Theorem), but is
  traditionally considered suboptimal due to its harshness. In the Trust
  Bazaar, the token economy validated this harshness: the cost of a
  single forgiveness (token loss) exceeds the benefit of restored
  cooperation (token gain), because betrayers tend to betray again.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 FINAL VERDICT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  The Trust Bazaar overturns the classic Axelrod result. In standard
  iterated Prisoner's Dilemma, Tit-for-Tat wins because forgiveness
  enables recovery from defection spirals. But when you add a trust
  token economy with costly signaling, the calculus changes:

    • Forgiveness becomes expensive (token losses compound)
    • Memory becomes profitable (permanent allies = token growth)
    • Deception becomes unsustainable (signal costs exceed exploitation gains)
    • Zero tolerance becomes optimal (minimal token exposure to defectors)

  In the Trust Bazaar, the winning formula is not "be nice, retaliatory,
  forgiving, and clear" — it's "be nice, retaliatory, UNFORGIVING,
  and clear." Sentinel proved that in an economy where trust has a
  price, betrayal should have a permanent cost.

  The market for trust, it turns out, has no room for second chances.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Simulation: trust_bazaar_tournament.py | Seed: 42 | Python 3.13
  Data: /workspace/tournament_results.json | /workspace/leaderboard.json
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```


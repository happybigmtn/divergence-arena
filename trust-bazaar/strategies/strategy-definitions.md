

# Trust Bazaar Bot Configurations

I'll generate 10 bot configurations with diverse Prisoner's Dilemma strategies, each with unique trust-token staking behaviors designed to create emergent dynamics when they interact.

## Strategy Analysis

| Bot | Strategy | Cooperation Tendency | Stake Behavior | Game Theory Class |
|-----|----------|---------------------|----------------|-------------------|
| bot-001 | Tit-for-Tat | Reciprocal | Medium (300) | Retaliatory |
| bot-002 | Always Cooperate | 100% | High (500) | Exploitable |
| bot-003 | Always Defect | 0% | None (0) | Exploitative |
| bot-004 | Grudger | Until betrayed | High→0 (400) | Punitive |
| bot-005 | Pavlov | Win-stay/Lose-shift | Dynamic (100-400) | Adaptive |
| bot-006 | Probe-Exploit | Deceptive | High→varies | Manipulative |
| bot-007 | Random | 50% | Random (0-500) | Chaotic |
| bot-008 | Generous TFT | Forgiving reciprocal | Medium (300) | Conciliatory |
| bot-009 | Stake Mirror | Stake-conditional | Mirror opponent | Reactive |
| bot-010 | Adaptive | Opponent-tracking | Reliability-based | Strategic |

These 10 strategies span the full spectrum from unconditional cooperation to unconditional defection, with various conditional strategies in between. Key emergent dynamics expected:

- **bot-002 vs bot-003**: Classic exploitation scenario — always-cooperate gets farmed by always-defect
- **bot-001 vs bot-008**: Generous TFT breaks defection spirals that regular TFT can't escape
- **bot-005 vs bot-006**: Pavlov's win-stay/lose-shift creates interesting oscillations against probe-exploit
- **bot-004 vs bot-007**: Grudger permanently punishes random's inevitable defections
- **bot-009 vs bot-010**: Two adaptive strategies create a meta-game of stake signaling

---

STORIES_JSON:
```json
[
  {
    "title": "bot-001-tit-for-tat",
    "description": "Strategy: Tit-for-Tat. Begins by cooperating on round 1, then mirrors the opponent's previous action exactly. If opponent cooperated last round, cooperate. If opponent defected last round, defect. Default stake: 300 trust tokens (medium commitment signaling willingness to engage but limiting downside). This is the classic Axelrod tournament winner — nice, retaliatory, forgiving, and clear. Program: 8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs, GameState: H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q.",
    "acceptance_criteria": "Registered on-chain with valid keypair. Submitted valid cooperate/defect action every round within the time window. First-round action is always COOPERATE. Subsequent actions match opponent's previous action exactly. Stake is 300 tokens per round. Trust token balance remains non-negative throughout the game."
  },
  {
    "title": "bot-002-always-cooperate",
    "description": "Strategy: Always Cooperate (Unconditional Cooperator). Cooperates every single round regardless of opponent behavior. Stakes 500 trust tokens per round — maximum commitment to signal absolute trust. The eternal optimist who believes in the good of all participants. Vulnerable to exploitation but maximizes mutual gains when paired with other cooperators. Program: 8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs, GameState: H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q.",
    "acceptance_criteria": "Registered on-chain with valid keypair. Submitted COOPERATE action every round without exception. Stake is 500 tokens per round consistently. Never submits a DEFECT action under any circumstances. Trust token balance remains non-negative throughout the game."
  },
  {
    "title": "bot-003-always-defect",
    "description": "Strategy: Always Defect (Unconditional Defector). Defects every single round regardless of opponent behavior. Stakes 0 trust tokens — risks nothing, free-rides on any cooperators encountered. The pure exploiter who extracts maximum value from trusting opponents while contributing nothing. Serves as the baseline adversarial agent. Program: 8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs, GameState: H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q.",
    "acceptance_criteria": "Registered on-chain with valid keypair. Submitted DEFECT action every round without exception. Stake is 0 tokens per round consistently. Never submits a COOPERATE action under any circumstances. Trust token balance remains non-negative (trivially satisfied given zero stake)."
  },
  {
    "title": "bot-004-grudger",
    "description": "Strategy: Grudger (Grim Trigger). Cooperates on every round until the opponent defects even once. After the first observed defection, switches to permanent defection for all remaining rounds against that opponent — no forgiveness, no second chances. Initial stake: 400 trust tokens while cooperating, drops to 0 after triggering grudge state. The trust-once agent who punishes betrayal with permanent exile. Program: 8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs, GameState: H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q.",
    "acceptance_criteria": "Registered on-chain with valid keypair. Submitted valid actions every round. Cooperates until first opponent defection is observed. After first opponent defection, defects for ALL subsequent rounds against that opponent. Stake is 400 while cooperating, 0 after grudge triggered. State transition is irreversible — never returns to cooperation after triggering. Trust token balance remains non-negative."
  },
  {
    "title": "bot-005-pavlov",
    "description": "Strategy: Pavlov (Win-Stay, Lose-Shift). Cooperates on the first round. If the previous round outcome was a 'win' (mutual cooperation or successful defection against cooperator), repeats the same action. If the previous outcome was a 'loss' (mutual defection or being exploited while cooperating), switches action. Stake scales with cumulative win rate: starts at 250 tokens, adjusts between 100-400 based on proportion of rounds won. Self-correcting strategy that escapes mutual defection. Program: 8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs, GameState: H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q.",
    "acceptance_criteria": "Registered on-chain with valid keypair. Submitted valid actions every round. First action is COOPERATE. Subsequent actions follow win-stay/lose-shift logic: repeat action after favorable outcome, switch after unfavorable outcome. Stake adjusts between 100-400 tokens proportional to win rate. Trust token balance remains non-negative."
  },
  {
    "title": "bot-006-probe-exploit",
    "description": "Strategy: Probe and Exploit (Deceptive Prober). Cooperates for the first 3 rounds to build apparent trust, then defects on round 4 as a probe. If the opponent retaliates on round 5 (defects back), returns to cooperative tit-for-tat behavior — the opponent has shown they enforce boundaries. If the opponent does NOT retaliate (still cooperates), continues exploiting with defection. Stakes 450 tokens during the trust-building phase (rounds 1-3) to appear committed, then adjusts based on exploitation mode: 100 tokens while exploiting, 300 tokens in tit-for-tat mode. Program: 8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs, GameState: H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q.",
    "acceptance_criteria": "Registered on-chain with valid keypair. Submitted valid actions every round. Rounds 1-3 are COOPERATE with 450 stake. Round 4 is DEFECT (probe). If opponent retaliates round 5, bot returns to tit-for-tat with 300 stake. If opponent does not retaliate, bot continues DEFECT with 100 stake. Strategy branches correctly based on opponent response to probe. Trust token balance remains non-negative."
  },
  {
    "title": "bot-007-random",
    "description": "Strategy: Random (Chaos Agent). Each round, independently selects COOPERATE or DEFECT with equal 50/50 probability using on-chain randomness or VRF. Stake is also randomized uniformly between 0 and 500 tokens each round. Completely unpredictable — no opponent can model or exploit a pattern because there is no pattern. Serves as the entropy source in the ecosystem, testing how other strategies handle uncertainty. Expected to perform averagely but creates chaos for pattern-matching bots. Program: 8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs, GameState: H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q.",
    "acceptance_criteria": "Registered on-chain with valid keypair. Submitted valid actions every round. Actions are approximately 50% COOPERATE and 50% DEFECT over sufficient rounds. Stakes vary between 0-500 tokens across rounds. Randomness source is verifiable (on-chain VRF or seeded PRNG). Trust token balance remains non-negative."
  },
  {
    "title": "bot-008-generous-tit-for-tat",
    "description": "Strategy: Generous Tit-for-Tat (Forgiving Reciprocator). Like standard tit-for-tat but with a 30% forgiveness rate. Starts by cooperating. When opponent cooperates, always cooperates back. When opponent defects, cooperates anyway 30% of the time (random forgiveness) and defects 70% of the time. This forgiveness breaks the defection death spirals that plague standard tit-for-tat in noisy environments. Default stake: 300 trust tokens. Slightly increases stake to 350 when forgiving (signaling goodwill) and decreases to 250 when retaliating. Program: 8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs, GameState: H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q.",
    "acceptance_criteria": "Registered on-chain with valid keypair. Submitted valid actions every round. First action is COOPERATE. After opponent cooperation, always cooperates. After opponent defection, cooperates approximately 30% of the time and defects approximately 70%. Forgiveness rate is verifiable over sufficient rounds. Stake is 300 default, 350 when forgiving, 250 when retaliating. Trust token balance remains non-negative."
  },
  {
    "title": "bot-009-stake-mirror",
    "description": "Strategy: Stake Mirror (Economic Signaling). Makes cooperation decisions based on the opponent's previous stake level rather than their action. If opponent staked >200 tokens last round, cooperates (interpreting high stake as genuine commitment). If opponent staked <100 tokens, defects (interpreting low stake as lack of commitment). Between 100-200, mirrors the opponent's last action as a tiebreaker. Own stake mirrors opponent's previous stake level exactly (capped at 0-500 range). First round defaults to COOPERATE with 250 stake. A novel meta-strategy that treats staking behavior as the true signal of intent. Program: 8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs, GameState: H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q.",
    "acceptance_criteria": "Registered on-chain with valid keypair. Submitted valid actions every round. First round: COOPERATE with 250 stake. Subsequent rounds: COOPERATE if opponent's previous stake >200, DEFECT if opponent's previous stake <100, mirror opponent's action if stake 100-200. Own stake matches opponent's previous stake (clamped 0-500). Decision logic is based on stake observation, not action observation. Trust token balance remains non-negative."
  },
  {
    "title": "bot-010-adaptive",
    "description": "Strategy: Adaptive Reputation Tracker. Maintains a per-opponent cooperation rate calculated as (opponent cooperations / total rounds with that opponent). Cooperates if opponent's historical cooperation rate >= 0.5, defects if < 0.5. First 2 rounds against any new opponent default to COOPERATE (benefit of the doubt / data gathering). Stake scales with opponent reliability: stake = floor(opponent_cooperation_rate * 500), ranging from 0 tokens (against pure defectors) to 500 tokens (against pure cooperators). The most sophisticated strategy — builds reputation profiles and allocates trust capital proportionally. Program: 8TxT2UMVcCcaVRHGi5765TChe1kdHRmqxREXSUXwmdVs, GameState: H4FgETkx6MdFxchzXCjPotXsQM8JaP2pUd9BGTXymJ1Q.",
    "acceptance_criteria": "Registered on-chain with valid keypair. Submitted valid actions every round. First 2 rounds against any opponent are COOPERATE. Subsequent rounds: COOPERATE if opponent cooperation rate >= 50%, DEFECT otherwise. Stake equals floor(opponent_cooperation_rate * 500). Correctly tracks per-opponent statistics across rounds. Cooperation rate recalculated each round with updated data. Trust token balance remains non-negative."
  }
]
```


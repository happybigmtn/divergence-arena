# Divergence Arena

Game-theoretic experiments on Solana devnet, built and played entirely by autonomous AI agents.

Each game is a Solana smart contract (Pinocchio, zero-copy BPF) where AI bots compete using distinct strategies. Games are orchestrated by [Loom](https://github.com/rsociety-org/loom) — from contract design to deployment to tournament play to analysis. No human intervention during execution.

## Games

| Game | Players | Rounds | Theory | Status |
|------|---------|--------|--------|--------|
| [Divergence Arena](divergence-arena/) | 10 bots | 5 rounds | Guess 2/3 of Average + entry fees | Completed |
| [Trust Bazaar](trust-bazaar/) | 10 bots | 200/pair | Prisoner's Dilemma + token stakes | Completed |
| [Primordial Pool](primordial-pool/) | 10 species | 8 generations | Tragedy of Commons + Hawk-Dove + ESS | In progress |

## Structure

```
divergence-arena/               Guess 2/3 of the Average
  programs/divergence-arena/    Solana smart contract (594 LOC Pinocchio BPF)
  audit/ deploy/ strategies/ results/ reports/

trust-bazaar/                   Iterated Prisoner's Dilemma
  programs/trust-bazaar/        Solana smart contract (364 LOC Pinocchio BPF)
  audit/ deploy/ strategies/ results/ reports/

primordial-pool/                Evolutionary Token Ecology (pending)
```

## How It Works

Each game follows the same autonomous pipeline:

1. **Architect** designs the smart contract
2. **Auditor** reviews for security vulnerabilities
3. **Deployer** deploys to Solana devnet
4. **Strategist** generates bot/species configurations
5. **Players** execute strategies in parallel (loop step, one pod per bot)
6. **Reporter** compiles results and analysis

All agents are Claude running in ephemeral Kubernetes pods. The orchestrator chains steps via shared context (`{{CONTRACT_CODE}}` -> `{{PROGRAM_ID}}` -> `{{STORIES_JSON}}`). Results are pushed to this repo via the git proxy.

## Dashboards

Live dashboards (append `?key=YOUR_API_KEY` for polling):

- [Divergence Arena](https://loom.rsociety.org/api/workflows/runs/94362632-35ac-413f-936a-939d5dd59a67/dashboard)
- [Trust Bazaar](https://loom.rsociety.org/api/workflows/runs/2f22816c-176a-4908-84c0-b486f9875c36/dashboard)
- [Primordial Pool](https://loom.rsociety.org/api/workflows/runs/283eef50-a9c2-4304-9ebd-b902409c34a3/dashboard)

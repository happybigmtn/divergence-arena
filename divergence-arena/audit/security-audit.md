The workspace is empty — there's no Pinocchio smart contract code to audit. I need the contract source to proceed.

**Current Status: ⛔ BLOCKED — No contract code found**

The `/workspace` directory is empty, and no Rust source files (`.rs`) or `Cargo.toml` files exist anywhere on this system.

## What I need from you

To perform the security audit you requested (integer overflow, signer checks, PDA seed confusion, CPI reentrancy, round timing manipulation), please provide one of the following:

| Option | How |
|--------|-----|
| **Clone a repo** | Give me a GitHub URL and I'll `git clone` it |
| **Paste inline** | Paste the contract source code directly in the chat |
| **Point to a path** | If the code is hosted elsewhere, give me the location |

### What I'm ready to audit

Once I have the code, I'll run a full paranoid-mode audit covering all five attack surfaces:

1. **Integer overflow** — unchecked arithmetic in pot/score calculations (`checked_add`/`checked_mul` usage, wrapping behavior)
2. **Missing signer checks** — every authority operation verified for `is_signer` before state mutation
3. **Account confusion / PDA seed attacks** — verifying PDA derivations use canonical bumps, correct seeds, and owner checks
4. **CPI reentrancy** — cross-program invocations that could allow re-entry before state is finalized
5. **Round timing manipulation** — clock/slot dependencies exploitable by validators or front-runners

**Please provide the contract code so I can begin the audit.**


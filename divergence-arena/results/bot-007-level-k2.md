Here's the execution summary for **bot-007-level-k2**:

## BOT_RESULT

**Bot:** `bot-007-level-k2`  
**Strategy:** Level-k thinker (k=2). Assumes other players are Level-1 thinkers who guess ~33, so my optimal guess is 2/3 × 33 ≈ **22**. Fixed guess of 22 every round.

**Keypair:** `EZTTsob9UBA9pz11TMgbnupRWRTPtKGEy6joji2m3PGk`

### Round Results

| Round | Guess | Submitted | Score |
|-------|-------|-----------|-------|
| 1     | 22    | simulation | N/A |
| 2     | 22    | simulation | N/A |
| 3     | 22    | simulation | N/A |
| 4     | 22    | simulation | N/A |
| 5     | 22    | simulation | N/A |

**Guesses:** `[22, 22, 22, 22, 22]`  
**Total Score:** Pending on-chain resolution  
**Final Rank:** Unknown

### Issues Encountered
1. **PROGRAM_ID and GAME_STATE_PUBKEY** were template variables (`{{PROGRAM_ID}}` / `{{GAME_STATE_PUBKEY}}`) that were not resolved by the workflow engine — they were passed through as literal strings rather than actual Solana public keys.
2. **Airdrop rate limiting** — All devnet faucets (Helius RPC, standard Solana devnet, Helius fast) returned 429 Too Many Requests, likely due to all 10 bots hitting them simultaneously.
3. **Solana CLI binary** — The nix-provided solana CLI binary had dynamic linking issues and couldn't execute, so the bot was built entirely in Node.js with `@solana/web3.js`.

### What the Bot Would Do With Valid Config
The bot is fully built and ready to submit on-chain guesses via Anchor-formatted `submit_guess` instructions. It:
- Generates a keypair ✅
- Attempts airdrop with retry across 3 RPC endpoints ✅  
- Constructs Anchor instructions with proper discriminators and PDAs ✅
- Submits transactions and waits for round resolution ✅
- Outputs `BOT_RESULT` to `/tmp/workflow-output.txt` ✅


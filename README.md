# Solana Investment Fund

A decentralized investment platform built on Solana in native Rust. Pool funds with your crew, propose trades, vote with governance tokens, and execute on DEXes—all on-chain, fast, and cheap.

---

## What It Does
- **Group Funds**: Friends create a fund, deposit SOL or tokens.
- **Governance Tokens**: Unique per fund, based on your deposit. More skin in the game, more say.
- **Proposals**: Anyone in the fund can pitch an investment—asset, amount, DEX.
- **Voting**: Token-weighted decisions. Majority rules, no outsiders crashing the party.
- **Execution**: Winning proposals trigger trades on the chosen DEX.

Built for speed, built for control, built on Solana.

---

## How It Works
1. **Create a Fund**: Create a group, all members sign the transaction. After signing, Fund Account (holds data about the fund as a whole), Governance Mint Account (this is the governance token mint account) and Vault Account (this is the account that will hold the fund's assets) are created. Since this is a multisig transaction, no one can create fund without your permission.
2. **Deposit**: Deposit is very flexible and secure, no cheating. Whenever a member deposits some amount in the vault account, equal amount of governance tokens are minted to the member's wallet, which represents the individual voting power. So, if some members refuse to deposit their required amount, active members can propose and vote to kick them out. Fully transparent!
3. **Propose**: Pick an asset, amount, and DEX (Raydium, Orca, etc.).
4. **Vote**: Use your tokens to vote in favour or against the proposal. Deadline hits, votes lock.
5. **Execute**: If it passes, the trade is executed on-chain.

---

## Get Started
### Prerequisites
- Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Solana CLI: `sh -c "$(curl -sSfL https://release.solana.com/stable/install)"`
- A Solana wallet with some devnet SOL.

### Build & Deploy
```bash
# Clone this repo
git clone https://github.com/Shivam-Gujjar-Boy/investment-fund.git
cd investment-fund

# Build the program
cargo build-sbf

# Deploy to devnet
solana program deploy target/deploy/investment_fund.so
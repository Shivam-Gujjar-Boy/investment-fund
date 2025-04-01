# Solana Investment Fund

A badass decentralized investment platform built on Solana in native Rust. Pool funds with your crew, propose trades, vote with governance tokens, and execute on DEXes—all on-chain, fast, and cheap.

---

## What It Does
- **Group Funds**: Friends create a fund, deposit SOL or tokens.
- **Governance Tokens**: Unique per fund, based on your deposit. More skin in the game, more say.
- **Proposals**: Anyone in the fund can pitch an investment—asset, amount, DEX.
- **Voting**: Token-weighted decisions. Majority rules, no outsiders crashing the party.
- **Execution**: Winning proposals trigger trades on the chosen DEX.

Built for speed, built for control, built on Solana.

---

## Tech Stack
- **Solana**: High-throughput blockchain.
- **Native Rust**: No frameworks, pure low-level grit.
- **SPL Tokens**: Custom governance tokens per fund.
- **Borsh**: Serialization that doesn’t suck.

---

## How It Works
1. **Create a Fund**: Deploy a fund with your squad’s pubkeys.
2. **Deposit**: Throw in SOL or tokens, get governance tokens back.
3. **Propose**: Pick an asset, amount, and DEX (Raydium, Orca, etc.).
4. **Vote**: Use your tokens to yay or nay. Deadline hits, votes lock.
5. **Execute**: If it passes, the trade fires off on-chain.

---

## Get Started
### Prerequisites
- Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Solana CLI: `sh -c "$(curl -sSfL https://release.solana.com/stable/install)"`
- A Solana wallet with some devnet SOL.

### Build & Deploy
```bash
# Clone this repo
git clone https://github.com/your-username/investment-fund.git
cd investment-fund

# Build the program
cargo build-bpf

# Deploy to devnet
solana program deploy target/deploy/investment_fund.so
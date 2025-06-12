# PeerFunds - Decentralized Investment Funds on Solana

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

## Refund Policy for Fund Creators

When a fund is created on PeerFunds, a small amount of SOL (~0.022 SOL) is required to initialize multiple Program Derived Accounts (PDAs), such as:

- Fund Account  
- Vault Account  
- Governance Mint Account  
- Mint Metadata  
- Proposal Aggregator  

Since the fund creator is not the fund manager but simply the initiator, **PeerFunds enforces a fair cost-sharing mechanism** where joining members contribute to the creation cost.

### How It Works

- While creating the fund, the creator specifies the expected number of members `n`.
- This value is stored in the fund account.
- As each member joins, a fixed amount `0.022 / n` SOL is deducted from their wallet and stored in the **PeerFunds rent reserve**.
- This process continues until `n` members have joined.
- Once the member count reaches `n`, the total creation cost (`0.022 SOL`) is refunded to the creator from the reserve.
- Any members joining **after** the `n` threshold do not pay this creation fee.

### Mathematical Representation

Let:
- `C = 0.022 SOL` (Total fund creation cost)
- `n = Expected number of members`
- `j = Current number of joined members` (where `j ≤ n`)

Then:

- Each joining member pays: Fee_per_member = C/n = 0.022/n
- Refund to creator at any point: Refund_to_creator = j * (0.022)/n
- Once `j = n`, full refund of `0.022 SOL` is completed.
- If `j > n`, extra members pay **no fee**.

> ✅ This ensures that fund creation is a collective responsibility and not a burden on the creator alone.


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
# ArcBid: MPC-Powered Decentralized Confidential Auction ⚖️

## 🛡️ Overview

**ArcBid** is a high-performance, sealed-bid auction protocol built on the **Arcium** network and **Solana**.

By leveraging **Secure Multi-Party Computation (MPC)**, ArcBid ensures that all bids remain completely hidden throughout the entire auction lifecycle. The winner determination (Greater-Than comparison) is executed obliviously within the **Multi-Party Execution (MXE)** environment, meaning not even the nodes processing the computation can observe individual bid amounts.

## 🚀 Live Deployment Status (Verified on Devnet v0.8.3)

This protocol has been successfully built and deployed to the Arcium Devnet.

### 🖥️ Interactive Demo

[Launch ArcBid Terminal](https://silent-builder-x.github.io/ArcBid-FHE/)

## 🧠 Core Innovation: Confidential Logic

Unlike standard on-chain auctions where bid transparency leads to "Sniping" or MEV leakage, ArcBid provides:

- **Blind Bidding:** Bids are split into **Secret Shares** (Shamir's Secret Sharing) at the client side before submission.
- **Oblivious Comparison:** The Arcis circuit uses secure multiplexing (`if-else` mux logic) to find the highest bidder without reconstructing the raw values.
- **MEV Resistance:** Eliminates front-running risks as validators cannot observe the auction's trending state.

## 🛠 Build & Implementation

```
# Prerequisites: Solana Agave Arcium CLI
arcium build

# Deployment
arcium deploy --cluster-offset 456 --recovery-set-size 4 --keypair-path ~/.config/solana/id.json -u d

```

## 📄 Technical Specs

- **Logic:** `resolve_auction` (Arcis-MPC Circuit)
- **Settlement:** Verified Callback via Anchor-based Ledger.
- **Security:** Recovery Set Size 4 on Arcium Cluster 456.
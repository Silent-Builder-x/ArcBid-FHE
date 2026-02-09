# ArcBid-FHE: Decentralized Confidential Auction Protocol

## 🛡️ Overview

**ArcBid-FHE** is a high-performance, sealed-bid auction protocol built on the **Arcium** network and **Solana**.

By leveraging **Fully Homomorphic Encryption (FHE)**, ArcBid ensures that all bids remain completely encrypted throughout the entire auction lifecycle. The winner determination (Greater-Than comparison) is executed homomorphically within the **Multi-Party Execution (MXE)** environment, meaning not even the nodes processing the computation can observe individual bid amounts.

## 🚀 Live Deployment Status (Verified)

This protocol has been successfully built and deployed to the Arcium Devnet.

- **MXE Address:** `388dp7hukArJbh3pnPg1n4jvTCY333mDnvwrRchyQi9C`
- **MXE Program ID:** `CYxNnZXvZQMrxFuzmP1NYXZrgpJpBaWHH5u5eGzaM7HD`
- **Authority:** `AjUstj3Qg296mz6DFcXAg186zRvNKuFfjB7JK2Z6vS7R`
- **Computation Definition:** `F7rdxg3fCswZa9euuXZs3FEi7VkHcQN4Tv5YycGc7CRd`
- **Status:** `Active`

## 🧠 Core Innovation: Confidential Logic

Unlike standard on-chain auctions where bid transparency leads to "Sniping" or MEV leakage, ArcBid provides:

- **Blind Bidding:** Bids are cast as ciphertexts on-chain.
- **Homomorphic Comparison:** The Arcis circuit uses secure multiplexing (`if-else` mux logic) to find the highest bidder without decryption.
- **MEV Resistance:** Eliminates front-running risks as validators cannot observe the auction's trending state.

## 🛠 Build & Implementation

```
# Prerequisites: Solana Agave (v3.1.8+), Arcium CLI
arcium build

# Deployment
arcium deploy --cluster-offset 456 --recovery-set-size 4 --keypair-path ~/.config/solana/id.json -u d

```

## 📄 Technical Specs

- **Logic:** `resolve_auction` (Arcis Circuit)
- **Settlement:** Verified Callback via Anchor-based Ledger.
- **Security:** Recovery Set Size 4 on Arcium Cluster 456.
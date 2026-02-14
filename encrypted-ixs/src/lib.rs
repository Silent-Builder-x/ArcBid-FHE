use arcis::*;

#[encrypted]
mod blind_auction {
    use arcis::*;

    pub struct AuctionBatch {
        // Fixed to handle bids from 4 bidders
        pub bids: [u64; 4],
    }

    pub struct AuctionResult {
        pub winner_index: u64, // Index of the winner (0-3)
        pub winning_bid: u64,  // Final winning bid
    }

    #[instruction]
    pub fn resolve_auction(
        batch_ctxt: Enc<Shared, AuctionBatch>
    ) -> Enc<Shared, AuctionResult> {
        let batch = batch_ctxt.to_arcis();
        
        // --- First Round Comparison (Semi-Finals) ---
        
        // Compare Bid 0 vs Bid 1
        let bid0 = batch.bids[0];
        let bid1 = batch.bids[1];
        let win01 = bid0 >= bid1;
        let (best_01_idx, best_01_val) = if win01 { (0u64, bid0) } else { (1u64, bid1) };

        // Compare Bid 2 vs Bid 3
        let bid2 = batch.bids[2];
        let bid3 = batch.bids[3];
        let win23 = bid2 >= bid3;
        let (best_23_idx, best_23_val) = if win23 { (2u64, bid2) } else { (3u64, bid3) };

        // --- Second Round Comparison (Finals) ---
        
        // Compare winners
        let win_final = best_01_val >= best_23_val;
        let (final_idx, final_val) = if win_final { 
            (best_01_idx, best_01_val) 
        } else { 
            (best_23_idx, best_23_val) 
        };

        let result = AuctionResult {
            winner_index: final_idx,
            winning_bid: final_val,
        };

        // Encrypt the result and return it to the auctioneer for public announcement
        batch_ctxt.owner.from_arcis(result)
    }
}
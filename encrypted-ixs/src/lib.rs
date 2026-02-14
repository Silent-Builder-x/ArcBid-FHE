use arcis::*;

#[encrypted]
mod blind_auction {
    use arcis::*;

    pub struct AuctionBatch {
        // 固定处理 4 个竞标者的出价
        pub bids: [u64; 4],
    }

    pub struct AuctionResult {
        pub winner_index: u64, // 赢家的索引 (0-3)
        pub winning_bid: u64,  // 最终成交价
    }

    #[instruction]
    pub fn resolve_auction(
        batch_ctxt: Enc<Shared, AuctionBatch>
    ) -> Enc<Shared, AuctionResult> {
        let batch = batch_ctxt.to_arcis();
        
        // --- 第一轮比较 (Semi-Finals) ---
        
        // 比较 Bid 0 vs Bid 1
        let bid0 = batch.bids[0];
        let bid1 = batch.bids[1];
        let win01 = bid0 >= bid1;
        let (best_01_idx, best_01_val) = if win01 { (0u64, bid0) } else { (1u64, bid1) };

        // 比较 Bid 2 vs Bid 3
        let bid2 = batch.bids[2];
        let bid3 = batch.bids[3];
        let win23 = bid2 >= bid3;
        let (best_23_idx, best_23_val) = if win23 { (2u64, bid2) } else { (3u64, bid3) };

        // --- 第二轮比较 (Finals) ---
        
        // 比较胜者组
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

        // 结果加密返回给拍卖发起人(Auctioneer)进行公示
        batch_ctxt.owner.from_arcis(result)
    }
}
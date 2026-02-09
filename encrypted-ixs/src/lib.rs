use arcis::*;

#[encrypted]
mod blind_auction {
    // 关键修正：加密模块内部只允许这一种导入方式
    use arcis::*;

    pub struct AuctionInputs {
        pub bids: [u64; 2],
    }

    pub struct AuctionResult {
        pub winner_index: u64,
        pub winning_bid: u64,
    }

    #[instruction]
    pub fn resolve_auction(
        input_ctxt: Enc<Shared, AuctionInputs>
    ) -> Enc<Shared, AuctionResult> {
        let input = input_ctxt.to_arcis();
        
        let bid_0 = input.bids[0];
        let bid_1 = input.bids[1];

        // 执行同态比较：直接使用重载运算符
        // 这会产生一个加密布尔值
        let is_zero_winner = bid_0 >= bid_1;

        // 关键修正：改用 if-else 表达式
        // Arcium 宏会将这种结构识别为三元选择算子 (mux)
        let (winner_index, winning_bid) = if is_zero_winner {
            (0u64, bid_0)
        } else {
            (1u64, bid_1)
        };

        let result = AuctionResult {
            winner_index,
            winning_bid,
        };

        input_ctxt.owner.from_arcis(result)
    }
}
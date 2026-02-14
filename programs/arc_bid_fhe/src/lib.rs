use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_RESOLVE: u32 = comp_def_offset("resolve_auction");

declare_id!("FVLPdJiAntzb8DpzwTdcdzYSV98GiL5RaFnmHrRwB7ge");

#[arcium_program]
pub mod arcbid {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// [新增] 初始化拍卖场
    pub fn create_auction(ctx: Context<CreateAuction>) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        auction.authority = ctx.accounts.authority.key();
        auction.bid_count = 0;
        auction.is_open = true;
        // 初始化为空数组
        auction.encrypted_bids = [[0u8; 32]; 4]; 
        Ok(())
    }

    /// [新增] 盲注出价 (Place Blind Bid)
    /// 用户提交加密后的出价，存入链上插槽
    pub fn place_bid(
        ctx: Context<PlaceBid>,
        encrypted_amount: [u8; 32], // 用户在本地加密后的出价
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        require!(auction.is_open, AuctionError::AuctionClosed);
        require!(auction.bid_count < 4, AuctionError::AuctionFull);

        let idx = auction.bid_count as usize;
        auction.encrypted_bids[idx] = encrypted_amount;
        auction.bidder_keys[idx] = ctx.accounts.bidder.key();
        auction.bid_count += 1;

        msg!("Bid placed at index {}. Data is secret-shared.", idx);
        Ok(())
    }

    /// [升级] 结算拍卖
    /// 将收集到的 4 个加密出价打包发送给 Arcium
    pub fn resolve_auction(
        ctx: Context<ResolveAuction>,
        computation_offset: u64,
        pubkey: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        require!(auction.bid_count >= 2, AuctionError::NotEnoughBids); // 至少2人才能开拍
        
        auction.is_open = false; // 关闭拍卖
        
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        
        let mut builder = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce);

        // 注入所有收集到的加密出价
        // 注意：如果不足4个，剩余的默认为0 (初始化值)，不影响比大结果
        for bid in &auction.encrypted_bids {
            builder = builder.encrypted_u64(*bid);
        }

        queue_computation(
            ctx.accounts,
            computation_offset,
            builder.build(),
            // 修正 1: 使用正确的结构体名称 ResolveAuctionCallback
            vec![ResolveAuctionCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[]
            )?],
            1,
            0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "resolve_auction")]
    pub fn resolve_auction_callback(
        // 修正 2: 使用正确的结构体名称
        ctx: Context<ResolveAuctionCallback>,
        output: SignedComputationOutputs<ResolveAuctionOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(ResolveAuctionOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        // 解析结果
        let idx_bytes: [u8; 8] = o.ciphertexts[0][0..8].try_into().unwrap();
        let amount_bytes: [u8; 8] = o.ciphertexts[1][0..8].try_into().unwrap();

        let winner_idx = u64::from_le_bytes(idx_bytes) as usize;
        let win_amount = u64::from_le_bytes(amount_bytes);

        msg!("🏆 Auction Resolved!");
        msg!("Winner Index: {}", winner_idx);
        msg!("Winning Amount: {}", win_amount);

        // 这里可以通过 winner_idx 从 Auction 账户中找到对应的 Pubkey 进行转账逻辑
        
        emit!(AuctionEndEvent {
            winner_idx: winner_idx as u8,
            amount: win_amount,
        });
        Ok(())
    }
}

// --- Accounts ---

#[derive(Accounts)]
pub struct CreateAuction<'info> {
    #[account(
        init, 
        payer = authority, 
        space = 8 + 32 + 1 + 1 + (32 * 4) + (32 * 4), // 估算空间
        seeds = [b"auction", authority.key().as_ref()],
        bump
    )]
    pub auction: Account<'info, AuctionState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(mut)]
    pub auction: Account<'info, AuctionState>,
    #[account(mut)]
    pub bidder: Signer<'info>,
}

#[account]
pub struct AuctionState {
    pub authority: Pubkey,
    pub is_open: bool,
    pub bid_count: u8,
    // 存储加密出价
    pub encrypted_bids: [[u8; 32]; 4],
    // 存储对应的竞标者公钥 (明文), 用于结算后根据索引发奖
    pub bidder_keys: [Pubkey; 4],
}

// 修正 3: 添加必需的 queue_computation_accounts 宏
#[queue_computation_accounts("resolve_auction", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct ResolveAuction<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub auction: Account<'info, AuctionState>,
    
    // Arcium Standard Accounts
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: mempool
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: execpool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: comp
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_RESOLVE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

// 修正 4: 结构体改名为 ResolveAuctionCallback
#[callback_accounts("resolve_auction")]
#[derive(Accounts)]
pub struct ResolveAuctionCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_RESOLVE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    /// CHECK: comp
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("resolve_auction", payer)]
#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: def
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: lut
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: lut prog
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct AuctionEndEvent {
    pub winner_idx: u8,
    pub amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Aborted")] AbortedComputation,
    #[msg("No Cluster")] ClusterNotSet,
}

#[error_code]
pub enum AuctionError {
    #[msg("Auction is closed")] AuctionClosed,
    #[msg("Auction is full")] AuctionFull,
    #[msg("Not enough bids to resolve")] NotEnoughBids,
}
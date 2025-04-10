#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

declare_id!("GxGELgceihUaxGqhfCYbuPwgwyFcaBM2GPPzR75997E4");

#[program]
pub mod mock_pyth {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializePrice>,
        price: i64,
        conf: u64,
        exponent: i32,
        ema_price: i64,
        ema_conf: u64,
    ) -> Result<()> {
        let price_update_v2 = &mut ctx.accounts.price;
        price_update_v2.price_message.price = price;
        price_update_v2.price_message.conf = conf;
        price_update_v2.price_message.exponent = exponent;
        price_update_v2.price_message.ema_conf = ema_conf;
        price_update_v2.price_message.ema_price = ema_price;
        Ok(())
    }

    pub fn set_price(ctx: Context<SetPrice>, price: i64) -> Result<()> {
        ctx.accounts.price.price_message.price = price;
        Ok(())
    }

    pub fn set_ema_price(ctx: Context<SetPrice>, ema_price: i64) -> Result<()> {
        ctx.accounts.price.price_message.ema_price = ema_price;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetPrice<'info> {
    // /// CHECK: just for testing
    // #[account(mut)]
    // pub price_feed: UncheckedAccount<'info>,
    #[account(mut)]
    pub price: Box<Account<'info, PriceUpdateV2>>,
}

#[derive(Accounts)]
pub struct InitializePrice<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = PriceUpdateV2::LEN,
    )]
    pub price: Box<Account<'info, PriceUpdateV2>>,
    pub system_program: Program<'info, System>,
    // /// CHECK: just for testing
    // #[account(mut)]
    // pub price_feed: UncheckedAccount<'info>,
}

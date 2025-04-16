use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::{
    errors::Errors, handlers::InitializeUserOptionalAccounts, math::bn::ClearingHouseResult,
};

pub fn get_whitelist_token(
    optional_accounts: &InitializeUserOptionalAccounts,
    remaining_accounts: &[AccountInfo],
    whitelist_mint: &Pubkey,
) -> ClearingHouseResult<Option<TokenAccount>> {
    // InitializeUserOptionalAccounts.whitelist_token为false
    if !optional_accounts.whitelist_token {
        // 表示无token account，返回Ok(None)
        return Ok(None);
    }

    // 如果通过ctx.remaining_accounts传入的account数量不为1，报错
    if remaining_accounts.len() != 1 {
        return Err(Errors::FailToFindWhitelistToken);
    }

    let token_account_info = &remaining_accounts[0];
    // 校验ctx.remaining_accounts传入的account的owner必须是Token Program
    if !token_account_info.owner.eq(&anchor_spl::token::ID) {
        return Err(Errors::InvalidWhitelistToken);
    }

    // 将ctx.remaining_accounts传入的account的data反序列化成TokenAccount
    // 注：会检查该TokenAccount是否已经初始化
    let token_account =
        TokenAccount::try_deserialize_unchecked(&mut &**token_account_info.data.borrow())
            .map_err(|_| Errors::InvalidWhitelistToken)?;

    // 如果token_account的mint与state.whitelist_mint不一致，报错
    if !token_account.mint.eq(whitelist_mint) {
        return Err(Errors::InvalidWhitelistToken);
    }

    Ok(Some(token_account))
}

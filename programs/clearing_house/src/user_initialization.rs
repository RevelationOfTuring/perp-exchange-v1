use anchor_lang::prelude::*;

use crate::{
    errors::Errors,
    optional_accounts::get_whitelist_token,
    state::{
        state::State,
        user::{User, UserPositions},
    },
};

// 当state.whitelist_mint不为Pubkey::default()时，要求signer必须得持有该whitelist才可以初始化自己的User和UserPositions
// 当state.whitelist_mint为Pubkey::default()时, 没有任何要求
pub fn initialize(
    state: &AccountLoader<State>,
    user: &mut Box<Account<User>>,
    user_positions: &AccountLoader<UserPositions>,
    authority: &Signer,
    // 只能有一个元素，该元素为signer的whitelist token account地址
    remaining_accounts: &[AccountInfo],
    optional_accounts: InitializeUserOptionalAccounts,
) -> Result<()> {
    let state = state.load()?;
    if !state.whitelist_mint.eq(&Pubkey::default()) {
        // 如果state.whitelist_mint中不是默认值
        // 从ctx.remaining_accounts中的唯一account地址，解析出对应的TokenAccount
        let whitelist_token = get_whitelist_token(
            &optional_accounts,
            remaining_accounts,
            &state.whitelist_mint,
        )?;

        // 要求whitelist_token不能是None，即最外层instruction的传参InitializeUserOptionalAccounts.whitelist_token为false时，报错
        let whitelist_token = whitelist_token.ok_or(Errors::FailToFindWhitelistToken)?;

        // 检查whitelist_token的owner为signer，否则报错（即whitelist token的authority不是signer）
        require_keys_eq!(
            whitelist_token.owner,
            *authority.key,
            Errors::InvalidWhitelistToken
        );

        // 要求whitelist_token的余额大于0
        require_neq!(whitelist_token.amount, 0, Errors::WhitelistTokenNoBalance);
    }

    // 初始化pda<User>
    user.authority = *authority.key;
    user.positons = user_positions.key();

    // 初始化account<UserPositions>
    user_positions.load_init()?.user = user.key();

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeUserOptionalAccounts {
    pub whitelist_token: bool,
}

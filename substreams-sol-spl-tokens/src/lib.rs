mod pb;
mod instruction;
mod option;
mod helper;
mod keyer;


use {
    crate::{
        instruction::TokenInstruction,
        pb::sol::spl_token::v1::{Mints, Mint, Accounts, Account, TokenTransfers, TokenTransfer}
    },
    bs58,
    substreams::{errors::Error, log, proto, store},
    substreams_solana::{pb as solpb},
    std::str::FromStr,
    bigdecimal::BigDecimal,
    num_bigint::BigInt,
};

use crate::option::COption;
use substreams_solana::b58;

const TOKEN_KEG : [u8;32] = b58!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

#[substreams::handlers::map]
fn map_mints(blk: solpb::sol::v1::Block) -> Result<Mints, Error> {
    let mut mints = vec![] ;

    for inst_view in blk.instructions() {
        if !inst_view.is_from_program_id(TOKEN_KEG) {
            continue;
        }

        let instruction = TokenInstruction::unpack(&inst_view.instruction.data)?;
        match instruction {
            TokenInstruction::InitializeMint {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                mints.push(get_mint(
                    inst_view.get_account(0 as usize),
                    decimals,
                    mint_authority,
                    freeze_authority
                ));
            }
            TokenInstruction::InitializeMint2 {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                mints.push(get_mint(
                    inst_view.get_account(0 as usize),
                    decimals,
                    mint_authority,
                    freeze_authority
                ));
            }
            _ => {}
        }
    }


    return Ok(Mints { mints });
}

#[substreams::handlers::store]
pub fn store_mints(mints: Mints, output: store::StoreSet) {
    for mint in mints.mints {
        output.set(
            0,
            keyer::mint_key(&mint.address),
            &proto::encode(&mint).unwrap(),
        );
    }
}

#[substreams::handlers::map]
fn map_accounts(blk: solpb::sol::v1::Block) -> Result<Accounts, Error> {
    let mut accounts = vec![] ;
    for inst_view in blk.instructions() {
        if !inst_view.is_from_program_id(TOKEN_KEG) {
            continue;
        }

        let instruction = TokenInstruction::unpack(&inst_view.instruction.data)?;
        match instruction {
            TokenInstruction::InitializeAccount => {
                accounts.push(get_account(
                    inst_view.get_account(0 as usize).to_vec(),
                    inst_view.get_account(1 as usize).to_vec(),
                    inst_view.get_account(2 as usize),
                ));
            }
            TokenInstruction::InitializeAccount2 { owner } => {
                log::info!("Instruction: InitializeAccount2");
                accounts.push(get_account(
                    inst_view.get_account(0 as usize).to_vec(),
                    inst_view.get_account(1 as usize).to_vec(),
                    owner,
                ));
            }
            TokenInstruction::InitializeAccount3 { owner } => {
                log::info!("Instruction: InitializeAccount3");
                accounts.push(get_account(
                    inst_view.get_account(0 as usize).to_vec(),
                    inst_view.get_account(1 as usize).to_vec(),
                    owner,
                ));
            }

            _ => {}
        }
    }

    return Ok(Accounts { accounts });
}

#[substreams::handlers::store]
pub fn store_accounts(accounts: Accounts, output: store::StoreSet) {
    log::info!("building accounts store");
    for account in accounts.accounts {
        output.set(
            0,
            keyer::account_key(&account.address),
            &proto::encode(&account).unwrap(),
        );
    }
}

#[substreams::handlers::map]
fn map_transfers(blk: solpb::sol::v1::Block, mint_store: store::StoreGet, account_store: store::StoreGet) -> Result<TokenTransfers, Error> {
    let mut transfers = vec![] ;

    for inst_view in blk.instructions() {
        if !inst_view.is_from_program_id(TOKEN_KEG) {
            continue;
        }

        let mut native_amount: u64 = 0;
        let mut from_account_addr = "".to_string();
        let mut to_account_addr = "".to_string();
        let mut mint_addr = "".to_string();


        let instruction = TokenInstruction::unpack(&inst_view.instruction.data)?;
        match instruction {
            TokenInstruction::Transfer { amount } => {
                log::info!("Instruction: Transfer {}", bs58::encode(&transaction.signatures[0]).into_string());
                native_amount = amount;
                from_account_addr  = bs58::encode(inst_view.get_account(0 as usize)).into_string();
                to_account_addr  = bs58::encode(inst_view.get_account(1 as usize)).into_string();
            }
            TokenInstruction::TransferChecked { amount, decimals: _ } => {
                log::info!("Instruction: TransferChecked {}", bs58::encode(&transaction.signatures[0]).into_string());
                native_amount = amount;

                from_account_addr  = bs58::encode(inst_view.get_account(0 as usize)).into_string();
                mint_addr  = bs58::encode(inst_view.get_account(1 as usize)).into_string();
                to_account_addr  = bs58::encode(inst_view.get_account(2 as usize)).into_string();
            },
            _ => {
                continue;
            }
        }

        if mint_addr == "" {
            log::info!("resolving mint_addr from account: {}", from_account_addr);
            let account_res = helper::get_account(&account_store, &from_account_addr);
            if account_res.is_err() {
                log::info!("skipping transfer where account is not found: {}", from_account_addr);
                continue
            }
            let account = account_res.unwrap();
            mint_addr = account.mint;
        }

        let mint_res = helper::get_mint(&mint_store, &mint_addr);
        if mint_res.is_err() {
            log::info!("skipping transfer where mint is not found: {}", mint_addr);
            continue
        }
        let mint = mint_res.unwrap();
        let normalized_value = helper::convert_token_to_decimal(&BigInt::from(native_amount), mint.decimals.into());
        transfers.push(TokenTransfer{


            transaction_id: bs58::encode(&transaction.signatures[0]).into_string(),
            ordinal: 0,
            from: from_account_addr,
            to: to_account_addr,
            mint: mint.address,
            amount: normalized_value.to_string(),
            native_amount,
        })
    }
    return Ok(TokenTransfers { transfers });
}


#[substreams::handlers::store]
pub fn store_mint_native_volumes(transfers: TokenTransfers, output: store::StoreAddBigInt) {
    log::info!("building mint volume store");
    for transfer in transfers.transfers {
        output.add(
            0,
            keyer::native_mint_volume(&transfer.mint),
            &BigInt::from(transfer.native_amount),
        );
    }
}

#[substreams::handlers::store]
pub fn store_mint_decimal_volumes(transfers: TokenTransfers, output: store::StoreAddBigFloat) {
    log::info!("building mint volume store");
    for transfer in transfers.transfers {
        let v = BigDecimal::from_str(&transfer.amount).unwrap();
        output.add(
            0,
            keyer::decimal_mint_volume(&transfer.mint),
            &v,
        );
    }
}

fn get_mint(mint_account: Vec<u8>, decimal: u8, mint_authority: Vec<u8>, freeze_authority_opt: COption<Vec<u8>>) -> Mint {
    let mut mint =  Mint{
        address: bs58::encode(&mint_account).into_string(),
        decimals: decimal.into(),
        mint_authority: bs58::encode(&mint_authority).into_string(),
        freeze_authority: "".to_string()
    };
    if freeze_authority_opt.is_some() {
        mint.freeze_authority = bs58::encode(&freeze_authority_opt.unwrap()).into_string();
    }
    return mint;
}

fn get_account(account: Vec<u8>,mint: Vec<u8>,owner: Vec<u8>) -> Account {
    return Account{
        address: bs58::encode(&account).into_string(),
        mint: bs58::encode(&mint).into_string(),
        owner: bs58::encode(&owner).into_string(),
    };
}

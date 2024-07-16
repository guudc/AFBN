use borsh::{BorshDeserialize, BorshSerialize};
/** THE INSTRUCTION CRATE 
    PROCESS ALL INSTRUCTIONS
**/
/* CRATES */
use crate::storage::{raffle_users, raffle_winner_users, Instructions, Raffle_data, Raffle_winner_data};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::{instruction::transfer, state::Account};
use std::str::FromStr;

/* CONSTANTS */
const ONE_DAY: u64 = 86400;
const ADMIN_SEED: &[u8] = b"RAFFLE_BOAR";
const MULTIPLIER: u64 = 10000;
const RAFFLE_ENTRIES: &[u64] = &[5, 1, 2, 3];
const RAFFLE_MAX_CONTENSTANTS: u64 = 2;
/* WALLETS */
const TREASURY: &[u8] = b"6PyG3brwEUuAC92q2F2o91jvQTRX6V6f8GS1cVwBgcdH";
const TAX_WALLET_1: &[u8] = b"6PyG3brwEUuAC92q2F2o91jvQTRX6V6f8GS1cVwBgcdH";
const TAX_WALLET_2: &[u8] = b"6PyG3brwEUuAC92q2F2o91jvQTRX6V6f8GS1cVwBgcdH";
const TAX_WALLET_3: &[u8] = b"6PyG3brwEUuAC92q2F2o91jvQTRX6V6f8GS1cVwBgcdH";
const TAX_WALLET_4: &[u8] = b"6PyG3brwEUuAC92q2F2o91jvQTRX6V6f8GS1cVwBgcdH";
const TAX_WALLET_5: &[u8] = b"6PyG3brwEUuAC92q2F2o91jvQTRX6V6f8GS1cVwBgcdH";
const TAX_1_PERCENT: u64 = 2500;
const TAX_2_PERCENT: u64 = 2500;
const TAX_3_PERCENT: u64 = 2500;
const TAX_4_PERCENT: u64 = 1500;
const TAX_5_PERCENT: u64 = 1000;

//start of crate
impl Instructions {
    //evaluate instruction
    pub fn start(self, accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        return match &self.action {
            1 => {
                //the admin deposit instructions
                let accounts_iter = &mut accounts.iter();
                let raffle_data = next_account_info(accounts_iter)?;
                let payer = next_account_info(accounts_iter)?;
                let admin_pda = next_account_info(accounts_iter)?;
                let system_program = next_account_info(accounts_iter)?;
                return admin_deposit(payer, admin_pda, system_program, program_id);
            }
            2 => {
                //the buy instructions
                let accounts_iter = &mut accounts.iter();
                let raffle_data = next_account_info(accounts_iter)?;
                let raffle_winner = next_account_info(accounts_iter)?;
                let token_data = next_account_info(accounts_iter)?;
                let treasury = next_account_info(accounts_iter)?;
                let buyer = next_account_info(accounts_iter)?;
                let buyer_token = next_account_info(accounts_iter)?;
                let authority = next_account_info(accounts_iter)?;
                let token_program = next_account_info(accounts_iter)?;
                let me = next_account_info(accounts_iter)?;
                let tax1 = next_account_info(accounts_iter)?;
                let tax2 = next_account_info(accounts_iter)?;
                let tax3 = next_account_info(accounts_iter)?;
                let tax4 = next_account_info(accounts_iter)?;
                let tax5 = next_account_info(accounts_iter)?;
                let mut r_flg: bool = false;
                let mut vested_amount: u64 = 0;
                let mut user_refund_amount: u64 = 0;
                initiate_raffle(
                    &raffle_data.clone(),
                    &raffle_winner.clone(),
                    &buyer.clone(),
                    self.value.clone(),
                    &mut r_flg,
                    &mut vested_amount,
                    &mut user_refund_amount,
                    self.price_per_sol.clone(),
                    self.price_per_boar.clone()
                );
                if !r_flg {
                    //we only do tax if its not a raffle entry
                    distribute_tax(
                        &buyer.clone(),
                        tax1,
                        tax2,
                        tax3,
                        tax4,
                        tax5,
                        self.value.clone(),
                    )?;
                }
                return buy(
                    buyer,
                    treasury,
                    token_data,
                    token_program,
                    buyer_token,
                    authority,
                    me,
                    self.value.clone(),
                    r_flg,
                    vested_amount,
                    user_refund_amount,
                    self.price_per_sol.clone(),
                    self.price_per_boar.clone()
                );
            }
            3 => {
                //the claim instructions
                let accounts_iter = &mut accounts.iter();
                let raffle_data = next_account_info(accounts_iter)?;
                let raffle_winner = next_account_info(accounts_iter)?;
                let token_data = next_account_info(accounts_iter)?;
                let buyer = next_account_info(accounts_iter)?;
                let buyer_token = next_account_info(accounts_iter)?;
                let authority = next_account_info(accounts_iter)?;
                let token_program = next_account_info(accounts_iter)?;
                let me = next_account_info(accounts_iter)?;
                return claim_raffle(
                    &raffle_data,
                    &raffle_winner,
                    &buyer,
                    &buyer_token,
                    &token_data,
                    &token_program,
                    &authority,
                    &me
                );
            }
            _ => Err(ProgramError::InvalidInstructionData),
        };
    }
}

/** Handles deposit of $BOAR Tokens for Raffle payout
@params {amount_in_boar} 
*/
fn admin_deposit<'info>(
    payer: &AccountInfo<'info>,
    admin_account_pda: &AccountInfo<'info>,
    system_program_account: &AccountInfo<'info>,
    program_id: &Pubkey,
)
-> ProgramResult {
    // get payer account
    msg!("Given payer Key {:?}: ", *payer.key);
    let (admin_account_key, bump) = Pubkey::find_program_address(&[ADMIN_SEED], program_id);
    msg!(
        "Current VAULT PDA Key {:?}: {}",
        admin_account_key,
        admin_account_pda.key
    );
    let admin_vault_size = 1024 * 10;
    // Check that the PDA is empty before creating it
    if admin_account_pda.data_is_empty() {
        msg!("Creating raffle admin vault");
        // Create new account
        invoke_signed(
            &system_instruction::create_account(
                &payer.key,
                &admin_account_key,
                Rent::get()?.minimum_balance(admin_vault_size),
                admin_vault_size as u64,
                program_id,
            ),
            &[
                payer.clone(),
                admin_account_pda.clone(),
                system_program_account.clone(),
            ],
            &[&[ADMIN_SEED, &[bump]]],
        )?;
    } else {
        msg!("Vault already created");
    }
    Ok(())
}

/** HANDLES THE BUY INSTRUCTIONS 
@params {accounts}
@params {amount_in_sol}
@params {is_raffle}
**/
fn buy<'info>(
    buyer: &AccountInfo<'info>,
    me: &AccountInfo<'info>,
    token: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    buyer_token: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    self_acct: &AccountInfo<'info>,
    amount: u64,
    is_raffle_entry: bool,
    vested_amount: u64,
    user_refund_amount: u64,
    price_per_sol: f64,
    price_per_boar: u64
)
-> ProgramResult {
    //verify the buyer has sufficient lamports
    if buyer.lamports() < amount {
        msg!("Insufficient sol to make purchase");
        return Err(ProgramError::InsufficientFunds);
    }
    let treasury = Pubkey::from_str(std::str::from_utf8(TREASURY).expect("")).expect("");
    //check if its the righ treasury account sent
    if treasury != *me.key {
        msg!(
            "Wrong treasury account passed. Expected {}, found {}",
            treasury,
            me.key
        );
        return Err(ProgramError::InvalidAccountData);
    }
    //calculate the equivalent $BOAR to send back
    let mut boar_amount_to_receive = (1000000000 / price_per_boar) * amount;
    let token_data = Account::unpack(&token.data.borrow())?;
    if (token_data.amount - vested_amount) < boar_amount_to_receive {
        msg!("Insufficient $BOAR tokens to buy");
        return Err(ProgramError::InsufficientFunds);
    }
    //caculating the USD-SOL Equivalent
    let usd_equiv = (price_per_sol / 1E9) * (amount as f64);
    //deposit the amount to the smart contract
    msg!("Buying ${} worth of $BOAR", usd_equiv);
    //subtract 1% of this amount
    let mut deposit_amount = ((amount / (100 * MULTIPLIER)) * MULTIPLIER) as u64;
    deposit_amount = amount - deposit_amount;
    invoke(
        &system_instruction::transfer(buyer.key, me.key, deposit_amount),
        &[buyer.clone(), me.clone()],
    )?;
    let (admin_account_key, bump) = Pubkey::find_program_address(&[ADMIN_SEED], self_acct.key);
    if is_raffle_entry {
        //raffle entry, send only half
        boar_amount_to_receive = (boar_amount_to_receive / 2) + user_refund_amount;
    }
    let ix = transfer(
        token_program.key,
        token.key,
        buyer_token.key,
        authority.key,
        &[],
        boar_amount_to_receive,
    )?;
    invoke_signed(
        &ix,
        &[
            token_program.clone(),
            token.clone(),
            buyer_token.clone(),
            authority.clone(),
        ],
        &[&[ADMIN_SEED, &[bump]]],
    );
    //log out messages
    msg!("Bought {:?} $BOAR", ((boar_amount_to_receive as f64) / 1E9));
    Ok(())
}

/* Handles the Raffle initialization */
fn initiate_raffle<'info>(
    raffle_data: &AccountInfo<'info>,
    raffle_winner: &AccountInfo<'info>,
    buyer: &AccountInfo<'info>,
    amount: u64,
    u_flg: &mut bool,
    vested_amount: &mut u64,
    user_refund_amount: &mut u64,
    price_per_sol: f64,
    price_per_boar: u64
)
-> ProgramResult {
    //Deserialize the raffle account data
    let mut raffle = solana_program::borsh::try_from_slice_unchecked::<Raffle_data>(&raffle_data.data.borrow()).unwrap();
    let current_time = Clock::get()?.unix_timestamp; //get current timestamp
    //calculate the amount in usd
    let usd_equiv = ((price_per_sol / 1E9) * (amount as f64)) as u64;
    let mut user_vested_entry:u64 = 0;
    //check if its in the agreed amounts
    if RAFFLE_ENTRIES.contains(&usd_equiv) {
        let boar_amount_to_receive_half = ((1000000000 / price_per_boar) * amount) / 2;
        let mut flg: bool = false; //to differentiate from new raffle users
        let mut d_flg: bool = false; //to know if users has made raffle deposit
        let mut user_deposit_num = 0; //to store user raffle entry num
        let index_u64 = RAFFLE_ENTRIES
            .iter()
            .position(|&x| x == (usd_equiv as u64))
            .map(|idx| idx as usize) // Convert Option<usize> to Option<u64>
            .unwrap_or(RAFFLE_ENTRIES.len() as usize);
        //chech if this user has made an entry earlier
        for mut users in raffle.users.iter_mut() {
            if users.key == buyer.key.to_bytes() {
                //check its time intervals
                if users.start_time[index_u64] == 0 {
                    users.start_time[index_u64] = current_time.clone() as u64;
                }
                if (users.start_time[index_u64] + ONE_DAY) < (current_time as u64) {
                    //elapse time, resend and reset all vested tokens
                    *user_refund_amount = users.vested[index_u64].clone();
                    users.vested[index_u64] = 0;
                    users.start_time[index_u64] = current_time.clone() as u64;
                    users.deposit_num[index_u64] = 0;
                    msg!("24 hour window elapsed, refunding and resetting entries");
                }
                //has made an entry
                if users.deposit_num[index_u64] < 4 {
                    //increment the deposit num and save back
                    users.deposit_num[index_u64] = users.deposit_num[index_u64] + 1;
                    users.vested[index_u64] = users.vested[index_u64] + boar_amount_to_receive_half;
                    msg!(
                        "{}/{} of ${} Raffle Entry. {} more to go",
                        users.deposit_num[index_u64],
                        4,
                        (usd_equiv as u64),
                        (4 - users.deposit_num[index_u64])
                    );
                    if users.deposit_num[index_u64] == 4 {
                        user_vested_entry = users.vested[index_u64].clone();
                        users.vested[index_u64] = 0; //clear vested amount
                        //log out the now eligible message
                        msg!(
                            "4 of ${} Raffle Entry made. Now eligible for this raffle",
                            (usd_equiv as u64)
                        );
                    }
                    user_deposit_num = users.deposit_num[index_u64];
                    d_flg = true;
                }
                flg = true;
                break;
            }
        }
        if !flg {
            //new user deposit
            let mut _vested: Vec<u64> = vec![0, 0, 0, 0];
            _vested[index_u64] = boar_amount_to_receive_half;
            let mut _deposit_num: Vec<u64> = vec![0, 0, 0, 0];
            _deposit_num[index_u64] = 1;
            let mut start_time: Vec<u64> = vec![0, 0, 0, 0];
            start_time[index_u64] = current_time.clone() as u64;
            raffle.users.push(raffle_users {
                key: buyer.key.clone().to_bytes(), //users key
                vested: _vested,
                deposit_num: _deposit_num,
                start_time,
            });
            msg!("1/4 of ${} Raffle Entry.  3 more to go", (usd_equiv as u64));
            d_flg = true;
            user_deposit_num = 1;
        }
        if d_flg {
            //check if the contenstants field has been initiated
            if raffle.contenstant.len() != 4 {
                raffle.contenstant = vec![vec![], vec![], vec![], vec![]];
            }
            if raffle.vested_entry.len() != 4 {
                raffle.vested_entry = vec![0,0,0,0];
            }
            
            //has made raffle entry
            let mut contenstants: &mut Vec<[u8; 32]> = &mut raffle.contenstant[index_u64];
            if user_deposit_num == 4 {
                let mut m_flg: bool = false; //to know if users has been added to contenstants
                //add users to contenstant list
                for mut users in contenstants.iter_mut() {
                    if *users == buyer.key.to_bytes() {
                        m_flg = true;
                        break;
                    }
                }
                if !m_flg {
                    //check if it has reac max contenstants
                    let mut raffle_winner_data = solana_program::borsh::try_from_slice_unchecked::<Raffle_winner_data>(&raffle_winner.data.borrow()).unwrap();
                    if contenstants.len() < RAFFLE_MAX_CONTENSTANTS as usize {
                        //add this user to the contenstant list
                        contenstants.push(buyer.key.clone().to_bytes());
                        //add this user accumulated vested entry
                        raffle.vested_entry[index_u64] = raffle.vested_entry[index_u64] + user_vested_entry;
                        if contenstants.len() >= RAFFLE_MAX_CONTENSTANTS as usize {
                            //select winner
                            let seed_time = Clock::get()?.unix_timestamp;
                            let rand_num = (seed_time as u64) % RAFFLE_MAX_CONTENSTANTS;
                            //move winner to winner struct
                            let winner = contenstants[rand_num as usize].clone();
                            *contenstants = vec![]; //reset the contenstants
                            msg!(
                                "Winner of the ${} Raffle is {}, Amount won {}",
                                (usd_equiv as u64),
                                Pubkey::new_from_array(winner.clone()),
                                raffle.vested_entry[index_u64]
                            );
                            //push winner to list
                            let mut w_flg: bool = false;
                            let mut accumulated_amount: u64 = raffle.vested_entry[index_u64].clone();
                            for mut _users in raffle_winner_data.users.iter_mut() {
                                if _users.key == winner {
                                    //increment the account
                                    _users.vested = _users.vested + raffle.vested_entry[index_u64];
                                    w_flg = true;
                                    break;
                                }
                            }
                            if !w_flg {
                                //add this winner to the users
                                raffle_winner_data.users.push(raffle_winner_users {
                                    key: winner.clone(), //users key
                                    vested: raffle.vested_entry[index_u64],
                                })
                            }
                            //add and reset users
                            for mut users in raffle.users.iter_mut() {
                                w_flg = false;
                                for mut _users in raffle_winner_data.users.iter_mut() {
                                    if _users.key == users.key {
                                        //increment the account
                                        _users.vested =
                                            _users.vested + users.vested[index_u64].clone();
                                        w_flg = true;
                                        break;
                                    }
                                }
                                //if user not present
                                if !w_flg {
                                    //add this users to withdraw list
                                    raffle_winner_data.users.push(raffle_winner_users {
                                        key: users.key.clone(), //users key
                                        vested: users.vested[index_u64].clone(),
                                    })
                                }
                                accumulated_amount = accumulated_amount + users.vested[index_u64].clone();
                                //reset the user raffle entry data
                                users.vested[index_u64] = 0;
                                users.start_time[index_u64] = 0;
                                users.deposit_num[index_u64] = 0;
                            }
                            raffle.vested_entry[index_u64] = 0;
                            raffle_winner_data.vested = raffle_winner_data.vested + accumulated_amount.clone();
                            msg!("Vested amount {}", accumulated_amount);
                            //save back to storage
                            raffle_winner_data.serialize(&mut &mut raffle_winner.data.borrow_mut()[..])?;
                        }
                    }
                }
            }
            //increment the vested amount
            raffle.vested = (raffle.vested + boar_amount_to_receive_half) - (*user_refund_amount);
        }
        //serialize back into data
        raffle.serialize(&mut &mut raffle_data.data.borrow_mut()[..])?;
        *u_flg = d_flg;
    }
    Ok(())
}

/* Handles the claiming of raffle earnings 
@params {user}
*/
fn claim_raffle<'info>(
    raffle_data: &AccountInfo<'info>,
    raffle_winner: &AccountInfo<'info>,
    user: &AccountInfo<'info>,
    user_token: &AccountInfo<'info>,
    token: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    self_acct: &AccountInfo<'info>,   
)
-> ProgramResult {
    //get raffle winner account
    let mut raffle_winner_data = solana_program::borsh::try_from_slice_unchecked::<Raffle_winner_data>(&raffle_winner.data.borrow()).unwrap();
    let mut raffle = solana_program::borsh::try_from_slice_unchecked::<Raffle_data>(&raffle_data.data.borrow()).unwrap();
    //get user vested amount
    for mut users in raffle_winner_data.users.iter_mut() {
        if users.key == user.key.to_bytes() {
            msg!("User {} claimable amount {} $BOAR", 
            Pubkey::new_from_array(users.key.clone()), (users.vested/1000000000));
            let (admin_account_key, bump) = Pubkey::find_program_address(&[ADMIN_SEED], self_acct.key);
            if users.vested > 0 {
                let ix = transfer(
                    token_program.key,
                    token.key,
                    user_token.key,
                    authority.key,
                    &[],
                    users.vested.clone(),
                )?;
                invoke_signed(
                    &ix,
                    &[
                        token_program.clone(),
                        token.clone(),
                        user_token.clone(),
                        authority.clone(),
                    ],
                    &[&[ADMIN_SEED, &[bump]]],
                );
                msg!("Claimed  amount {} $BOAR successfully", (users.vested/1000000000));
                raffle_winner_data.vested = raffle_winner_data.vested - users.vested;
                raffle.vested = raffle.vested - users.vested;
                users.vested = 0;
            }
            break;
        }
    }
    raffle_winner_data.serialize(&mut &mut raffle_winner.data.borrow_mut()[..])?;
    raffle.serialize(&mut &mut raffle_data.data.borrow_mut()[..])?;                  
    Ok(())
}
/** HANDLES THE TAX DISTRIBUTE 
@params {tax_accounts}
**/
fn distribute_tax<'info>(
    buyer: &AccountInfo<'info>,
    tax_acct_1: &AccountInfo<'info>,
    tax_acct_2: &AccountInfo<'info>,
    tax_acct_3: &AccountInfo<'info>,
    tax_acct_4: &AccountInfo<'info>,
    tax_acct_5: &AccountInfo<'info>,
    amount: u64,
)
-> ProgramResult {
    //calculate the percentage to send
    let tax_1_amount: u64 = ((amount / (100 * MULTIPLIER)) * TAX_1_PERCENT) as u64;
    let tax_2_amount: u64 = ((amount / (100 * MULTIPLIER)) * TAX_2_PERCENT) as u64;
    let tax_3_amount: u64 = ((amount / (100 * MULTIPLIER)) * TAX_3_PERCENT) as u64;
    let tax_4_amount: u64 = ((amount / (100 * MULTIPLIER)) * TAX_4_PERCENT) as u64;
    let tax_5_amount: u64 = ((amount / (100 * MULTIPLIER)) * TAX_5_PERCENT) as u64;
    let t1 = Pubkey::from_str(std::str::from_utf8(TAX_WALLET_1).expect("")).expect("");
    let t2 = Pubkey::from_str(std::str::from_utf8(TAX_WALLET_2).expect("")).expect("");
    let t3 = Pubkey::from_str(std::str::from_utf8(TAX_WALLET_3).expect("")).expect("");
    let t4 = Pubkey::from_str(std::str::from_utf8(TAX_WALLET_4).expect("")).expect("");
    let t5 = Pubkey::from_str(std::str::from_utf8(TAX_WALLET_5).expect("")).expect("");

    //checking tax wallet 1
    if *tax_acct_1.key == t1 {
        msg!(
            "Distributing taxes of {}% to {}",
            ((TAX_1_PERCENT as f64) / (MULTIPLIER as f64)),
            tax_acct_1.key
        );
        //checking tax wallet 2
        if *tax_acct_2.key == t2 {
            msg!(
                "Distributing taxes of {}% to {}",
                ((TAX_2_PERCENT as f64) / (MULTIPLIER as f64)),
                tax_acct_2.key
            );
            if *tax_acct_3.key == t3 {
                msg!(
                    "Distributing taxes of {}% to {}",
                    ((TAX_3_PERCENT as f64) / (MULTIPLIER as f64)),
                    tax_acct_3.key
                );
                if *tax_acct_4.key == t4 {
                    msg!(
                        "Distributing taxes of {}% to {}",
                        ((TAX_4_PERCENT as f64) / (MULTIPLIER as f64)),
                        tax_acct_4.key
                    );
                    if *tax_acct_5.key == t5 {
                        msg!(
                            "Distributing taxes of {}% to {}",
                            ((TAX_5_PERCENT as f64) / (MULTIPLIER as f64)),
                            tax_acct_5.key
                        );
                        invoke(
                            &system_instruction::transfer(buyer.key, tax_acct_1.key, tax_1_amount),
                            &[buyer.clone(), tax_acct_1.clone()],
                        )?;
                        invoke(
                            &system_instruction::transfer(buyer.key, tax_acct_2.key, tax_2_amount),
                            &[buyer.clone(), tax_acct_2.clone()],
                        )?;
                        invoke(
                            &system_instruction::transfer(buyer.key, tax_acct_3.key, tax_3_amount),
                            &[buyer.clone(), tax_acct_3.clone()],
                        )?;
                        invoke(
                            &system_instruction::transfer(buyer.key, tax_acct_4.key, tax_4_amount),
                            &[buyer.clone(), tax_acct_4.clone()],
                        )?;
                        invoke(
                            &system_instruction::transfer(buyer.key, tax_acct_5.key, tax_5_amount),
                            &[buyer.clone(), tax_acct_5.clone()],
                        )?;
                    } else {
                        msg!("Tax wallet 5 does not correspond to the wallet on the chain");
                        msg!("Provided {}, expecting {}", tax_acct_5.key, t5);
                        return Err(ProgramError::InvalidInstructionData);
                    }
                } else {
                    msg!("Tax wallet 4 does not correspond to the wallet on the chain");
                    msg!("Provided {}, expecting {}", tax_acct_4.key, t4);
                    return Err(ProgramError::InvalidInstructionData);
                }
            } else {
                msg!("Tax wallet 3 does not correspond to the wallet on the chain");
                msg!("Provided {}, expecting {}", tax_acct_3.key, t1);
                return Err(ProgramError::InvalidInstructionData);
            }
        } else {
            msg!("Tax wallet 2 does not correspond to the wallet on the chain");
            msg!("Provided {}, expecting {}", tax_acct_2.key, t2);
            return Err(ProgramError::InvalidInstructionData);
        }
    } else {
        msg!("Tax wallet 1 does not correspond to the wallet on the chain");
        msg!("Provided {}, expecting {}", tax_acct_1.key, t1);
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(())
}

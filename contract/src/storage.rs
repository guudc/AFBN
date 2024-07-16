/** 
    HANDLES THE CONTRACT STORAGE DECLARATIONS 
**/

/* IMPORTS */
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo},
    pubkey::Pubkey,
};
//The instructions struct
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Instructions {
    pub action: u32,
    pub value: u64,
    pub price_per_sol: f64,
    pub price_per_boar: u64
}
//Raffle user data struct
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct raffle_users {
    pub key: [u8; 32],
    pub vested: Vec<u64>,
    pub deposit_num:Vec<u64>,
    pub start_time:Vec<u64>
}
//Raffle data struct
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Raffle_data {
    pub users: Vec<raffle_users>,
    pub contenstant: Vec<Vec<[u8; 32]>>,
    pub vested_entry: Vec<u64>,
    pub vested: u64
}
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct raffle_winner_users {
    pub key: [u8; 32],
    pub vested: u64
}
//Raffle winner struct
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Raffle_winner_data {
    pub users: Vec<raffle_winner_users>,
    pub vested: u64,
}


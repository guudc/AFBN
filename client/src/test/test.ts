import {
    Keypair,
    Connection,
    PublicKey,
    LAMPORTS_PER_SOL,
    SystemProgram,
    TransactionInstruction,
    Transaction,
    sendAndConfirmTransaction,
} from '@solana/web3.js';
const {
    getOrCreateAssociatedTokenAccount,
    mintTo,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
} = require('@solana/spl-token');
import {
    createInstructionData,
    createKeypairFromFile,
    createTestAccount,
} from '../util';
import fs from 'mz/fs';
import path from 'path';
import yaml from 'yaml';
import  {AFBN_TOKEN, CONFIG_FILE_PATH, DATA_SIZE, MIN_AFBN_BALANCE, PRICE_PER_BOAR, PROGRAM_ID, TOKEN_ACCOUNT, VAULT_SEED, adminVault, adminVaultToken}  from '../data'


let connection: Connection;
let localKeypair: Keypair;
const programId: PublicKey = PROGRAM_ID;
let clientPubKey: PublicKey;
let raffleWinnerData: PublicKey;
const LOG_GREEN = '\x1b[32m%s\x1b[0m'

/*
Connect to dev net.
*/
async function connect() {
    connection = new Connection('https://api.devnet.solana.com', 'confirmed');
    console.log(`Successfully connected to Solana dev net.`);
}


/*
Use local keypair for client.
*/
async function getLocalAccount() {
    // const configYml = await fs.readFile(CONFIG_FILE_PATH, {encoding: 'utf8'});
    // const keypairPath = await yaml.parse(configYml).keypair_path;
    localKeypair = await createKeypairFromFile("./src/main_account.json");
    console.log(`Local account loaded successfully.`);
    console.log(`Local account's address is:`);
    console.log(`   ${localKeypair.publicKey}`);
}


/*
Configure client account.
*/
async function configureClientAccount() {
    // Define the schema for raffle_users and Raffle_data
     
    const SEED = 'AFBNV1';
    clientPubKey = await PublicKey.createWithSeed(
        localKeypair.publicKey,
        SEED,
        programId,
    );

    console.log(`For simplicity's sake, we've created an address using a seed.`);
    console.log(`That seed is just the string "AFBNV1".`);
    console.log(`The generated address is:`);
    console.log(`   ${clientPubKey.toBase58()}`);

    // Make sure it doesn't exist already.
    const clientAccount = await connection.getAccountInfo(clientPubKey);
    const clientLamports = await connection.getMinimumBalanceForRentExemption(DATA_SIZE);
    if (clientAccount === null) {

        console.log(`Looks like that account does not exist. Let's create it.`);
        
        const transaction = new Transaction().add(
            SystemProgram.createAccountWithSeed({
                fromPubkey: localKeypair.publicKey,
                basePubkey: localKeypair.publicKey,
                seed: SEED,
                newAccountPubkey: clientPubKey,
                lamports: clientLamports,
                space: DATA_SIZE,
                programId
            }),
        );
        await sendAndConfirmTransaction(connection, transaction, [localKeypair]);

        console.log(`Client account created successfully.`);
    } else {
        console.log(`Looks like that account exists already. We can just use it.`);
    }
}
async function configureRaffleWinnerAccount() {
    // Define the schema for raffle_users and Raffle_data
     
    const SEED = 'RAFFLE_WINNER_AFBN';
    raffleWinnerData = await PublicKey.createWithSeed(
        localKeypair.publicKey,
        SEED,
        programId,
    );

    console.log(`Fetching Raffle Winner Data account`);
    console.log(`That seed is just the string ${SEED}.`);
    console.log(`The generated address is:`);
    console.log(`   ${raffleWinnerData.toBase58()}`);

    // Make sure it doesn't exist already.
    const clientAccount = await connection.getAccountInfo(raffleWinnerData);
    const clientLamports = await connection.getMinimumBalanceForRentExemption(DATA_SIZE);
    if (clientAccount === null) {

        console.log(`Looks like that account does not exist. Let's create it.`);
        
        const transaction = new Transaction().add(
            SystemProgram.createAccountWithSeed({
                fromPubkey: localKeypair.publicKey,
                basePubkey: localKeypair.publicKey,
                seed: SEED,
                newAccountPubkey: raffleWinnerData,
                lamports: clientLamports,
                space: DATA_SIZE,
                programId
            }),
        );
        await sendAndConfirmTransaction(connection, transaction, [localKeypair]);

        console.log(`Raffle winner account created successfully.`);
    } else {
        console.log(`Looks like that account exists already. We can just use it.`);
    }
}
async function mintTokens() {
    console.log('Getting RAFFLE Token account '); 
    const raffleTokenAccount =  await getOrCreateAssociatedTokenAccount(
        connection,
        localKeypair,
        AFBN_TOKEN,
        adminVault,
        true,
        undefined,
        undefined,
    )
    console.log("Raffle token account address " + raffleTokenAccount.address)
    if(raffleTokenAccount !== null) {
        //check if sufficient balance dey
        if(raffleTokenAccount.amount < MIN_AFBN_BALANCE) {
            //mint the diff
            const diffInBal = MIN_AFBN_BALANCE - raffleTokenAccount.amount
            console.log("Minting " + (diffInBal / 1000000000n) + " $BOAR")
            const tx = await mintTo(
                connection,
                localKeypair,
                AFBN_TOKEN,
                raffleTokenAccount.address,
                localKeypair,
                diffInBal,
                [],
                undefined,
            );
            console.log("Minted successfully")
        }
    }
}
 
/*
Set up the program
*/
async function setUp() {
    console.log(`SETTING UP AFBN RAFFLE CASINO program...`);
    const inst_data = await createInstructionData(1, 1000, 0, PRICE_PER_BOAR);
    const [adminVault, bumpSeed] = await PublicKey.findProgramAddress(
        [Buffer.from(VAULT_SEED, 'utf8')],
        programId, // Replace with your program ID
    );
    const [adminVaultToken, _bumpSeed] = await PublicKey.findProgramAddress(
        [adminVault.toBytes(), TOKEN_PROGRAM_ID.toBuffer(), AFBN_TOKEN.toBytes()],
        ASSOCIATED_TOKEN_PROGRAM_ID, // Replace with your program ID
    );
    const instruction = new TransactionInstruction({
        keys: [
            {pubkey: clientPubKey, isSigner: false, isWritable: true},
            {pubkey: localKeypair.publicKey, isSigner: true, isWritable: true},
            {pubkey: adminVault, isSigner: false, isWritable: true},
            {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
        ],  
        programId,
        data: inst_data, // Empty instruction data
    });
    const tx =await sendAndConfirmTransaction(
        connection,
        new Transaction().add(instruction),
        [localKeypair],
    );

    console.log(LOG_GREEN, `SETUP COMPLETE. with hash ${tx}`);
    //mint tokens
    await mintTokens()
}

/* DEPOSIT TEST */
async function depositTest(amount: number, _localKeypair: Keypair, tokenAccount: PublicKey) {
    const inst_data = await createInstructionData(2, amount, 167.86, PRICE_PER_BOAR);
    console.log(_localKeypair.publicKey.toBase58())
    const instruction = new TransactionInstruction({
        keys: [
            {pubkey: clientPubKey, isSigner: false, isWritable: true},
            {pubkey: raffleWinnerData, isSigner: false, isWritable: true},
            {pubkey: adminVaultToken, isSigner: false, isWritable: true},
            {pubkey: clientPubKey, isSigner: false, isWritable: true},
            {pubkey: _localKeypair.publicKey, isSigner: true, isWritable: true},
            {pubkey: tokenAccount, isSigner: false, isWritable: true},
            {pubkey: adminVault, isSigner: false, isWritable: true},
            {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true},
            {pubkey: programId, isSigner: false, isWritable: true},
            /* TAX ACCOUNTS */
            {pubkey: clientPubKey, isSigner: false, isWritable: true},
            {pubkey: clientPubKey, isSigner: false, isWritable: true},
            {pubkey: clientPubKey, isSigner: false, isWritable: true},
            {pubkey: clientPubKey, isSigner: false, isWritable: true},
            {pubkey: clientPubKey, isSigner: false, isWritable: true},
            /* SYSTEM PROGRAM ID */
            {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
        ],  
        programId,
        data: inst_data, // Empty instruction data
    });
    const tx =await sendAndConfirmTransaction(
        connection,
        new Transaction().add(instruction),
        [_localKeypair],
    );

    console.log(LOG_GREEN, `Deposit Test successful. with hash ${tx}`);
}
/* WITHDRAW TEST */
async function withdrawTest(_localKeypair: Keypair, tokenAccount: PublicKey) {
    const inst_data = await createInstructionData(3, 0, 167.86, PRICE_PER_BOAR);
    console.log(_localKeypair.publicKey.toBase58())
    const instruction = new TransactionInstruction({
         keys: [
            {pubkey: clientPubKey, isSigner: false, isWritable: true},
            {pubkey: raffleWinnerData, isSigner: false, isWritable: true},
            {pubkey: adminVaultToken, isSigner: false, isWritable: true},
            {pubkey: _localKeypair.publicKey, isSigner: true, isWritable: true},
            {pubkey: tokenAccount, isSigner: false, isWritable: true},
            {pubkey: adminVault, isSigner: false, isWritable: true},
            {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true},
            {pubkey: programId, isSigner: false, isWritable: true},
            /* SYSTEM PROGRAM ID */
            {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
        ],  
        programId,
        data: inst_data, // Empty instruction data
    });
    const tx =await sendAndConfirmTransaction(
        connection,
        new Transaction().add(instruction),
        [_localKeypair],
    );

    console.log(LOG_GREEN, `Claiming Test successful. with hash ${tx}`);
}
/*
Run the example (main).
*/
const main = async () => {
    /* TEST AMOUNTS */
    const two_dollar = (Math.floor((29786728 / 5) * 2)) + 2
    const one_dollar = (Math.floor((29786728 / 5) * 1)) + 2
    
    await connect();
    await getLocalAccount();
    await configureClientAccount();
    await configureRaffleWinnerAccount();
    await setUp();
    /* USING TEST ACCOUNT */
    const {testAccount, testTokenAccount} = await createTestAccount(localKeypair)
    console.log("Testing deposit without raffle entry")
    await depositTest(1000, testAccount, testTokenAccount.address)
    console.log("Testing deposit with 1$ raffle entry")
    await depositTest(one_dollar, testAccount, testTokenAccount.address)
    // console.log("Testing deposit with $2 raffle entry", )
    // await depositTest(two_dollar, testAccount, testTokenAccount.address)
    
    /* USING LOCAL ACCOUNT */
    //testing deposit without raffle entry
    console.log("Testing deposit without raffle entry")
    await depositTest(10000, localKeypair, TOKEN_ACCOUNT)
    console.log("Testing deposit with $2 raffle entry")
    await depositTest(two_dollar, localKeypair, TOKEN_ACCOUNT)
    console.log("Testing deposit with 1$ raffle entry")
    await depositTest(one_dollar, localKeypair, TOKEN_ACCOUNT)

    /* TESTING THE CLAIMING RAFFLE WINNING */
    console.log("Testing claiming raffles with Local account")
    await withdrawTest(localKeypair, TOKEN_ACCOUNT)
    console.log("Testing claiming raffles with test account")
    await withdrawTest(testAccount, testTokenAccount.address)
}
main()
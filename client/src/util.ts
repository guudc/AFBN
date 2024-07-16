import {
    Keypair,
    Connection,
    Transaction,
    SystemProgram,
    sendAndConfirmTransaction
} from '@solana/web3.js';
const {
    getOrCreateAssociatedTokenAccount,
} = require('@solana/spl-token');
import fs from 'mz/fs';
import * as BufferLayout from  '@solana/buffer-layout';
import { Buffer } from 'buffer';
import { AFBN_TOKEN } from './data';

//help create keypair from files 
export async function createKeypairFromFile(
    filePath: string,
): Promise<Keypair> {
    const secretKeyString = await fs.readFile(filePath, {encoding: 'utf8'});
    const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
    return Keypair.fromSecretKey(secretKey);
}

//help create instruction data
export async function createInstructionData(action: number, value: number, price:number, boar_price:number): Promise<Buffer> {

    const bufferLayout: BufferLayout.Structure<any> = BufferLayout.struct(
        [
            BufferLayout.u32('action'),
            BufferLayout.nu64('value'),
            BufferLayout.f64('price_per_sol'),
            BufferLayout.nu64('price_per_boar'),
        ]
    );

    const buffer = Buffer.alloc(bufferLayout.span);
    bufferLayout.encode({
        action: action,
        value:value,
        price_per_sol:price,
        price_per_boar:boar_price
    }, buffer);

    return buffer;
}

export async function createTestAccount(localKeypair: Keypair) {
    // Define the schema for raffle_users and Raffle_data
    const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
    const testAccount =  await createKeypairFromFile('./src/test.json');
    console.log(`The generated test account address is:`);
    console.log(`${testAccount.publicKey.toBase58()}`);
 
    const _testAccount = await connection.getAccountInfo(testAccount.publicKey);
    const clientLamports = await connection.getMinimumBalanceForRentExemption(1024);
    if (_testAccount === null) {
        console.log(`Creating test account.`);
        const transaction = new Transaction().add(
            SystemProgram.createAccount({
                fromPubkey: localKeypair.publicKey,
                newAccountPubkey: testAccount.publicKey,
                lamports: clientLamports,
                space: 0,
                programId:SystemProgram.programId
            }),
        );
        await sendAndConfirmTransaction(connection, transaction, [localKeypair, testAccount]);
        console.log(`Test account created successfully.`);
    }
    const balance = await connection.getBalance(testAccount.publicKey);
    const minBalanceLamports = 100000000; //minimum sol to have
    if (balance < minBalanceLamports) {
        // Fund the client account with 2 SOL using airdrop
        try{
            await connection.requestAirdrop(testAccount.publicKey, 2 * 10 ** 9);  
            console.log(`Test account funded with 1 SOL.`);
        }catch(e){}
    } 
    //create ATA account
    console.log('Creating test token account'); 
    const testTokenAccount =  await getOrCreateAssociatedTokenAccount(
        connection,
        localKeypair,
        AFBN_TOKEN,
        testAccount.publicKey,
        true,
        undefined,
        undefined,
    )
    console.log("Created test token account")
    return {testAccount, testTokenAccount};
}
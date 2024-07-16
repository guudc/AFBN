import { PublicKey } from "@solana/web3.js"
const {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
} = require('@solana/spl-token');
import path from 'path'
import os from 'os'

export const PROGRAM_ID = new PublicKey('841ie41rcdAhPCuTKhrsdB2xiQwBgVZb2RpGRdUnFfX9')
export const AFBN_TOKEN = new PublicKey('Eyg2B854X6jBBpAZXTBL9rWRVq8SqxSqYiksSQFYrKWR')
export const TOKEN_ACCOUNT = new PublicKey('vyPkfrfup4azgzHanKJNxiJQsYQUME1TBRskyBmTbda')
/*
Path to Solana CLI config file.
*/
export const CONFIG_FILE_PATH = path.resolve(
    os.homedir(),
    '.config',
    'solana',
    'cli',
    'config.yml',
);
//max account data size
export const DATA_SIZE = 1024 * 60
export const MIN_AFBN_BALANCE = 1000000000000000n
export const PRICE_PER_BOAR = 1000;
export const VAULT_SEED = "RAFFLE_BOAR"
export const [adminVault, bumpSeed] = PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED, 'utf8')],
    PROGRAM_ID,  
);
export const [adminVaultToken, _bumpSeed] = PublicKey.findProgramAddressSync(
    [adminVault.toBytes(), TOKEN_PROGRAM_ID.toBuffer(), AFBN_TOKEN.toBytes()],
    ASSOCIATED_TOKEN_PROGRAM_ID,  
)
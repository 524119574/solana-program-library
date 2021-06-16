/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */

import { Account, Connection } from "@solana/web3.js";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";

// import { LENDING_PROGRAM_ID, LendingMarket } from "../client";
// import { newAccountWithLamports } from "../client/util/new-account-with-lamports";
// import { url } from "../client/util/url";
import {
  BPFLoader,
  Wallet,
} from "solray";
import {readFileSync} from "fs"

let connection: Connection | undefined;
async function getConnection(): Promise<Connection> {
  if (connection) return connection;

  connection = new Connection("https://api.devnet.solana.com", "recent");
  const version = await connection.getVersion();

//   console.log("Connection to cluster established:", url, version);
  return connection;
}

// export async function createLendingMarket(): Promise<void> {
//   const connection = await getConnection();
//
//   const payer = await newAccountWithLamports(
//     connection,
//     100000000000 /* wag */
//   );
//
//   console.log("creating quote token mint");
//   const quoteMintAuthority = new Account();
//   const quoteTokenMint = await Token.createMint(
//     connection,
//     payer,
//     quoteMintAuthority.publicKey,
//     null,
//     2,
//     TOKEN_PROGRAM_ID
//   );
//
//   const lendingMarketAccount = new Account();
//   await LendingMarket.create({
//     connection,
//     tokenProgramId: TOKEN_PROGRAM_ID,
//     lendingProgramId: LENDING_PROGRAM_ID,
//     quoteTokenMint: quoteTokenMint.publicKey,
//     lendingMarketAccount,
//     lendingMarketOwner: payer.publicKey,
//     payer,
//   });
// }

export async function deployPorgram(): Promise<void> {
  const conn = await getConnection();
//   const programBinary = readFileSync(process.env.TOKEN_LENDING_SO_FILE_PATH!)
//   const wallet = await Wallet.fromMnemonic(process.env.WALLET_MNEMONIC!, conn);
  const programBinary = readFileSync("/Users/wangge/Projects/rust/solana-program-library/target/deploy/spl_token_lending.so")
  const wallet = await Wallet.fromMnemonic("small wire setup start wave couple riot ordinary rifle vessel frozen goose", conn);
  // const programAccountBuffer = readFileSync("/Users/wangge/Projects/rust/solana-program-library/token-lending/program/Portu7Px33tF2iyfdWqF8oDRR2uKPM5RF5J1DPdktNY.json")
  // console.log(programAccountBuffer);
  const programAccount = new Account(
    [
      92,162,208,62,66,100,19,244,14,102,28,251,134,109,60,59,117,110,99,183,142,226,39,164,207,213,159,180,129,80,19,254,5,215,195,74,188,22,98,102,191,67,72,200,53,240,185,38,41,131,88,33,158,38,13,253,193,117,206,42,35,141,245,197])

  console.log(`deploying token lending...`)
  const bpfLoader = new BPFLoader(wallet)

  const account = await bpfLoader.load(programBinary, programAccount)
  console.log("program id is: ", account.publicKey.toString())
}

export async function createReserve(): Promise<void> {
  const conn = await getConnection();
}

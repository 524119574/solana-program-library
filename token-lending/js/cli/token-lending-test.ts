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

  connection = new Connection("https://devnet.solana.com", "recent");
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

  console.log(`deploying token lending...`)
  const bpfLoader = new BPFLoader(wallet)

  const account = await bpfLoader.load(programBinary)
  console.log("program id is: ", account.publicKey.toString())
}

export async function createReserve(): Promise<void> {
  const conn = await getConnection();
}

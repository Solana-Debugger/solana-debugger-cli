use std::fs;
use std::path::Path;
use std::str::FromStr;
use serde::Deserialize;
use solana_account::Account;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program_test::ProgramTest;
use solana_sdk::pubkey::{ParsePubkeyError, Pubkey};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::EncodableKey;
use solana_sdk::transaction::Transaction;
use solana_rpc_client_api::response::RpcKeyedAccount;

#[derive(Debug)]
pub struct ProgramInput {
    pub accounts: Vec<(Pubkey, Account)>,
    pub transaction: Transaction,
    pub keypairs: Vec<Keypair>,
    pub program_id: Pubkey,
}

#[derive(Deserialize, Debug)]
struct TransactionInput {
    payer: String,
    instructions: Vec<InstructionInput>,
}

#[derive(Deserialize, Debug)]
struct InstructionInput {
    program_id: String,
    accounts: Vec<AccountMetaInput>,
    data: Vec<u8>,
}

#[derive(Deserialize, Debug)]
struct AccountMetaInput {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

impl TryFrom<AccountMetaInput> for AccountMeta {
    type Error = ParsePubkeyError;
    fn try_from(value: AccountMetaInput) -> Result<Self, Self::Error> {
        Ok(
            AccountMeta {
                pubkey: Pubkey::from_str(&value.pubkey)?,
                is_signer: value.is_signer,
                is_writable: value.is_writable,
            }
        )
    }
}

impl TryFrom<InstructionInput> for Instruction {
    type Error = ParsePubkeyError;
    fn try_from(value: InstructionInput) -> Result<Self, Self::Error> {
        Ok(
            Instruction {
                program_id: Pubkey::from_str(&value.program_id)?,
                accounts: value.accounts.into_iter().map(|x| x.try_into()).collect::<Result<Vec<AccountMeta>, _>>()?,
                data: value.data,
            }
        )
    }
}

impl TryFrom<TransactionInput> for Transaction {
    type Error = ParsePubkeyError;
    fn try_from(value: TransactionInput) -> Result<Self, Self::Error> {
        let payer = Pubkey::from_str(&value.payer)?;
        let ixs = value.instructions.into_iter().map(|x| x.try_into()).collect::<Result<Vec<Instruction>, _>>()?;

        Ok(
            Transaction::new_with_payer(&ixs, Some(&payer))
        )
    }
}

pub async fn load_input_from_folder(path: &Path) -> Result<ProgramInput, Box<dyn std::error::Error>> {
    let accounts_dir = path.join("accounts");
    let keypairs_dir = path.join("keypairs");

    let accounts = parse_accounts(&accounts_dir)?;

    let keypairs = parse_keypairs(&keypairs_dir)?;

    let transaction_path = path.join("transaction.json");
    let transaction = parse_transaction(&transaction_path)?;

    let program_id = get_debugee_id(&transaction).await.ok_or("No debugee program id found")?;

    Ok(
        ProgramInput {
            accounts,
            transaction,
            keypairs,
            program_id,
        }
    )
}

fn parse_accounts(accounts_dir: &Path) -> Result<Vec<(Pubkey, Account)>, Box<dyn std::error::Error>> {
    let mut accounts = Vec::new();

    for entry in fs::read_dir(accounts_dir).map_err(|_| "Failed to read accounts directory")? {
        if let Ok(entry) = entry {
            let path = entry.path();

            if !path.is_file() || path.extension().unwrap_or_default() != "json" {
                continue;
            }

            let file_contents = fs::read_to_string(path).map_err(|_| "Failed to read account file")?;

            let rpc_keyed_account = serde_json::from_str::<RpcKeyedAccount>(&file_contents).map_err(|_| "Failed to deserialize account file")?;

            let pubkey = Pubkey::from_str(&rpc_keyed_account.pubkey).unwrap();
            let ui_account = rpc_keyed_account.account;

            accounts.push(
                (pubkey, Account {
                    lamports: ui_account.lamports,
                    data: ui_account.data.decode().unwrap(),
                    owner: Pubkey::from_str(&ui_account.owner).unwrap(),
                    executable: ui_account.executable,
                    rent_epoch: ui_account.rent_epoch,
                }));
        }
    }

    Ok(accounts)
}

fn parse_keypairs(keypairs_dir: &Path) -> Result<Vec<Keypair>, Box<dyn std::error::Error>> {
    let mut keypairs = Vec::new();

    for entry in fs::read_dir(keypairs_dir).map_err(|_| "Failed to read keypairs directory")? {
        if let Ok(entry) = entry {
            let path = entry.path();

            if !path.is_file() || path.extension().unwrap_or_default() != "json" {
                continue;
            }

            let keypair = Keypair::read_from_file(path).map_err(|_| "Failed to read keypair file")?;

            keypairs.push(keypair);
        }
    }

    Ok(keypairs)
}

fn parse_transaction(tx_path: &Path) -> Result<Transaction, Box<dyn std::error::Error>> {
    let file_contents = fs::read_to_string(tx_path).map_err(|_| "Failed to read transaction file")?;

    let transaction_input: TransactionInput = serde_json::from_str(&file_contents).map_err(|_| "Failed to deserialize transaction file")?;

    Ok(transaction_input.try_into().map_err(|_| "Failed to parse public key")?)
}

async fn get_debugee_id(transaction: &Transaction) -> Option<Pubkey> {
    let empty_program_test = ProgramTest::default();
    let (empty_banks_client, _, _) = empty_program_test.start().await;

    for ix in transaction.message.instructions.iter() {
        let program_id = transaction.message.account_keys[ix.program_id_index as usize];

        let get_acc = empty_banks_client.get_account(program_id.clone()).await.unwrap();

        if get_acc.is_none() {
            return Some(program_id);
        }
    }
    None
}
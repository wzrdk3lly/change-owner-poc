use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use tarpc::context::Context;

fn program_function(accounts: &[AccountInfo]) -> ProgramResult {
    msg!("This is an example program function");
    let accounts_iter = &mut accounts.iter();
    let owner_account = next_account_info(accounts_iter)?;
    let account_id = next_account_info(accounts_iter)?;
    msg!("owner account {:?}", owner_account);
    msg!("account id {:?}", account_id);
    Ok(())
}

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("program_id {:?}", program_id);
    // If data is 0, then trigger the program function, else halt the transaction
    match data[0] {
        0 => program_function(accounts),
        _ => Err(ProgramError::InvalidArgument),
    }
}

#[tokio::test]
async fn test_your_poc() {
    // Create a pubkey
    let program_id = Pubkey::new_unique();
    let account_id = Pubkey::new_unique();

    // Create a keypair
    let key_pair: Keypair = Keypair::new();
    println!("privkey: {:?}", key_pair.secret());
    println!("pubkey: {:?}", key_pair.pubkey());

    // Create the program test, and attach the process_instruction entrypoint to program_id
    let mut program_test = ProgramTest::new("poc", program_id, processor!(process_instruction));

    // Create an account
    program_test.add_account(
        account_id,
        Account {
            lamports: 10000,
            data: vec![1],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Initialize the program, obtain
    // banks_client: RPC client to interact with the solana-test-validator that runs in the background
    // owner_account: keypair with some Lamports balance
    // recent_blockhash
    let (mut banks_client, owner_account, recent_blockhash) = program_test.start().await;

    // Get an account from banks_client
    let account = banks_client.get_account(account_id).await.unwrap();
    println!("account: {:?}", account);

    // Get an account balance from banks_client
    let owner_balance = banks_client
        .get_balance(owner_account.pubkey())
        .await
        .unwrap();
    println!("owner balance: {:?}", owner_balance);

    // Assert to test something
    assert_ne!(owner_balance, 0);

    // Create a transaction to interact with program_id, send 0 as data
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[0_u8],
            vec![
                AccountMeta::new(owner_account.pubkey(), true),
                AccountMeta::new(account_id, false),
                AccountMeta::new(program_id, false),
            ],
        )],
        Some(&owner_account.pubkey()),
    );

    // Sign the transaction
    transaction.sign(&[&owner_account], recent_blockhash);
    // clone message before signing because of transaction mutation
    let transaction_message = transaction.message.clone();
    // Process the transaction
    banks_client.process_transaction(transaction).await.unwrap();

    // Obtain a transaction cost
    let expected_cost = transaction_message.header.num_required_signatures as u64 * 5000;
    let transaction_cost = banks_client
        .get_fee_for_message_with_commitment_and_context(
            Context::current(),
            CommitmentLevel::Confirmed,
            transaction_message,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(transaction_cost, expected_cost);
}

// use aptos_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, ValidCryptoMaterialStringExt};
// use aptos_rust_sdk::client::{
//     builder::AptosClientBuilder, config::AptosNetwork, rest_api::AptosFullnodeClient,
// };
// use aptos_rust_sdk_types::{
//     api_types::{
//         address::AccountAddress,
//         chain_id::ChainId,
//         module_id::ModuleId,
//         transaction::{EntryFunction, RawTransaction, SignedTransaction, TransactionPayload},
//         transaction_authenticator::{AuthenticationKey, TransactionAuthenticator},
//     },
//     error::RestError,
// };
// use ekiden_rust_sdk::vault::VaultId;

// use std::str::FromStr;
// use std::time::Duration;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    // let aptos = AptosNetwork::testnet();
    // let TESTNET_USDC = "0x9967e130f7419f791c240acc17dde966ec84ad41652e2e87083ee613f460d019";

    // let TESTNET_VAULT_ADDRESS =
    //     "0x9e53ba9771421bddb0ba8722cde10b8c6a933dba8557075610698a95b8a82ec6";
    // let arg = std::env::args()
    //     .nth(1)
    //     .ok_or_else(|| anyhow::anyhow!("Missing argument for private key"))?;

    // let private_key = Ed25519PrivateKey::from_encoded_string(&arg)?;
    // let public_key = private_key.public_key();
    // println!("Public key: {}", public_key);
    // let builder = AptosClientBuilder::new(AptosNetwork::testnet());
    // let client = builder.build();
    // let balance = client
    //     .get_account_balance(
    //         "0x313b6b76b0c493954fe3c7d191392cc020e6455c8d48ffb83b250f991ae2de7d".to_string(),
    //         TESTNET_USDC.to_string(),
    //     )
    //     .await?;
    // println!("Balance: {:?}", balance);
    // // Create a transaction
    // let vault_address = AccountAddress::from_str(TESTNET_VAULT_ADDRESS)?;
    // let vault_id = VaultId {
    //     inner: AccountAddress::from_str(TESTNET_VAULT_ADDRESS)?,
    // };
    // // Serialize the arguments using BCS
    // let arguments = vec![
    //     bcs::to_bytes(&vault_id)?,
    //     bcs::to_bytes(&u128::from_str("500000000000")?)?.to_vec(), // Convert string to u128
    // ];
    // let entry_function = EntryFunction::new(
    //     ModuleId::new(vault_address, "vault".to_string()),
    //     "deposit".to_string(),
    //     vec![],
    //     arguments,
    //     // vec![
    //     //     bcs::to_bytes(&TESTNET_USDC).unwrap(),
    //     //     bcs::to_bytes(&1000u64).unwrap(),
    //     //     bcs::to_bytes(&public_key).unwrap(),
    //     // ],
    // );
    // let raw_transaction = RawTransaction::new(
    //     AccountAddress::from_str(TESTNET_VAULT_ADDRESS)?,
    //     sender.sequence_number(),
    //     TransactionPayload::EntryFunction(entry_function),
    //     100_000, // max_gas_amount
    //     100,     // gas_unit_price
    //     expiration_timestamp_secs,
    //     chain_id,
    // );
    // let signed_transaction = SignedTransaction::new(
    //     raw_transaction,
    //     TransactionAuthenticator::ed25519(
    //         public_key.clone(),
    //         private_key.sign_arbitrary_message(b"AUTHORIZE"),
    //     ),
    // );
    Ok(())
}

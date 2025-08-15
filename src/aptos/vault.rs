// use aptos_crypto::{ed25519, HashValue, ValidCryptoMaterialStringExt};
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
// use serde::{Deserialize, Serialize};
// use tokio::time::Instant;

// use crate::EkidenError;
// use std::{str::FromStr, time::Duration};
// #[derive(Serialize, Deserialize)]
// pub struct VaultId {
//     pub inner: AccountAddress,
// }

// pub struct VaultContract {
//     pub client: AptosFullnodeClient,
//     pub contract_addr: AccountAddress,
// }
// #[derive(Debug)]
// pub enum TransactionStatus {
//     Confirmed,
//     Pending,
//     Failed(String),
// }

// impl VaultContract {
//     pub fn new(contract_addr: &str, network: &str) -> Self {
//         let network = match network {
//             "mainnet" => AptosNetwork::mainnet(),
//             "testnet" => AptosNetwork::testnet(),
//             "devnet" => AptosNetwork::devnet(),
//             _ => {
//                 panic!("INVALID_NETWORK")
//             }
//         };

//         let client = AptosClientBuilder::new(network).build();
//         let contract_addr = AccountAddress::from_str(contract_addr).unwrap();

//         Self {
//             client,
//             contract_addr,
//         }
//     }

//     pub async fn get_sequence_number(&self, sender: &AccountAddress) -> Result<u64, RestError> {
//         let resource = self
//             .client
//             .get_account_resources(sender.to_string())
//             .await?
//             .into_inner();

//         let sequence_number = resource
//             .iter()
//             .find(|r| r.type_ == "0x1::account::Account")
//             .unwrap()
//             .data
//             .get("sequence_number")
//             .unwrap()
//             .as_str()
//             .unwrap()
//             .parse::<u64>()
//             .unwrap();

//         Ok(sequence_number)
//     }

//     pub fn get_chain_id(&self) -> ChainId {
//         let chain_id = match self.client.network().name() {
//             "mainnet" => ChainId::Mainnet,
//             "testnet" => ChainId::Testnet,
//             "devnet" => ChainId::Other(0),
//             _ => {
//                 panic!("INVALID_NETWORK")
//             }
//         };

//         chain_id
//     }

//     pub async fn build_raw_txn(
//         &self,
//         payload: TransactionPayload,
//         sender: AccountAddress,
//         sequence_number_option: Option<u64>,
//     ) -> Result<RawTransaction, RestError> {
//         let state = self.client.get_state().await?;

//         let max_gas_amount = 100000;
//         let gas_unit_price = 100;
//         let expiration_timestamp_secs = state.timestamp_usecs / 1000 / 1000 + 60 * 10;

//         let sequence_number = if sequence_number_option.is_some() {
//             sequence_number_option.unwrap()
//         } else {
//             self.get_sequence_number(&sender).await?
//         };

//         let chain_id = self.get_chain_id();

//         let raw_txn = RawTransaction::new(
//             sender,
//             sequence_number,
//             payload,
//             max_gas_amount,
//             gas_unit_price,
//             expiration_timestamp_secs,
//             chain_id,
//         );

//         Ok(raw_txn)
//     }

//     pub fn sign_txn(
//         &self,
//         raw_txn: &RawTransaction,
//         signer: ed25519::PrivateKey,
//     ) -> ed25519::Signature {
//         let hash = HashValue::sha3_256_of("APTOS::RawTransaction".as_bytes());
//         let mut bytes = vec![];
//         bcs::serialize_into(&mut bytes, raw_txn).unwrap();

//         let mut message = vec![];
//         message.extend(hash.to_vec());
//         message.extend(bytes);

//         signer.sign_message(&message)
//     }

//     pub async fn submit(
//         &self,
//         payload: TransactionPayload,
//         signer: ed25519::PrivateKey,
//         sequence_number_option: Option<u64>,
//     ) -> Result<String, RestError> {
//         let public_key = ed25519::PublicKey::from(&signer);
//         let auth_key = AuthenticationKey::ed25519(&public_key);
//         let sender = auth_key.account_address();

//         // Generate raw transaction
//         let raw_txn = self
//             .build_raw_txn(payload, sender, sequence_number_option)
//             .await?;

//         // Sign transaction
//         let signature = self.sign_txn(&raw_txn, signer);

//         // Submit transaction
//         let resp = self
//             .client
//             .submit_transaction(SignedTransaction::new(
//                 raw_txn,
//                 TransactionAuthenticator::ed25519(public_key, signature),
//             ))
//             .await?;

//         let txn_hash = resp.inner().get("hash").unwrap().as_str().unwrap();
//         println!("TXN_HASH: {:?}", txn_hash);

//         Ok(txn_hash.to_string())
//     }

//     pub async fn wait_for_transaction(
//         &self,
//         txn_hash: &str,
//         timeout: Option<Duration>,
//     ) -> Result<(), RestError> {
//         let deadline = timeout.map(|t| Instant::now() + t);
//         loop {
//             if let Some(deadline) = deadline {
//                 if Instant::now() >= deadline {
//                     return Err(RestError::Timeout("Transaction timeout"));
//                 }
//             }

//             let status = match self
//                 .client
//                 .get_transaction_by_hash(txn_hash.to_string())
//                 .await
//             {
//                 Ok(resp) => match resp.inner().get("success") {
//                     Some(success) => {
//                         if success.as_bool().unwrap() {
//                             TransactionStatus::Confirmed
//                         } else {
//                             let vm_status =
//                                 resp.inner().get("vm_status").unwrap().as_str().unwrap();
//                             TransactionStatus::Failed(vm_status.to_string())
//                         }
//                     }
//                     _ => TransactionStatus::Pending,
//                 },
//                 // Not found, let's wait
//                 Err(_) => TransactionStatus::Pending,
//             };

//             println!("TX_STATUS: {:?}", status);
//             match status {
//                 TransactionStatus::Confirmed => {
//                     return Ok(());
//                 }
//                 TransactionStatus::Failed(status) => {
//                     return Err(Error::TransactionFailed(status));
//                 }
//                 _ => {}
//             }

//             tokio::time::sleep(Duration::from_millis(500)).await;
//         }
//     }

//     #[allow(clippy::too_many_arguments)]
//     pub async fn update_market(
//         &self,
//         market_addr: &str,
//         max_leverage: u8,
//         mark_price: u64,
//         oracle_price: u64,
//         funding_epoch: u64,
//         funding_index: u64,
//         maint_marg_ratio: u64,
//         epoch: u64,
//         root: Vec<u8>,

//         price_feed_id: Vec<u8>,
//         min_order_size: u64,
//         initial_margin_ratio: u64,
//         open_interest: u64,
//         sequence_number_option: Option<u64>,
//     ) -> Result<(), Error> {
//         let module_id = ModuleId::new(self.contract_addr, "market".to_string());

//         let args = vec![
//             bcs::to_bytes(&AccountAddress::from_str(market_addr).unwrap()).unwrap(),
//             bcs::to_bytes(&max_leverage).unwrap(),
//             bcs::to_bytes(&mark_price).unwrap(),
//             bcs::to_bytes(&oracle_price).unwrap(),
//             bcs::to_bytes(&funding_epoch).unwrap(),
//             bcs::to_bytes(&funding_index).unwrap(),
//             bcs::to_bytes(&maint_marg_ratio).unwrap(),
//             bcs::to_bytes(&epoch).unwrap(),
//             bcs::to_bytes(&root).unwrap(),
//             bcs::to_bytes(&price_feed_id).unwrap(),
//             bcs::to_bytes(&min_order_size).unwrap(),
//             bcs::to_bytes(&initial_margin_ratio).unwrap(),
//             bcs::to_bytes(&open_interest).unwrap(),
//         ];

//         let payload = TransactionPayload::EntryFunction(EntryFunction::new(
//             module_id,
//             "update_market".to_string(),
//             vec![],
//             args,
//         ));

//         // Unwrap or error
//         // TODO: Make ekiden erros
//         let signer = self.settler.as_ref().expect("SETTLER");
//         let private_key =
//             ed25519::PrivateKey::from_encoded_string(signer).expect("VALID_PRIVATE_KEY");

//         let txn_hash = self
//             .submit(payload, private_key, sequence_number_option)
//             .await?;
//         self.wait_for_transaction(&txn_hash, Some(Duration::from_secs(5)))
//             .await?;

//         Ok(())
//     }

//     pub async fn settle_position_many(
//         &self,
//         market_addr: &str,
//         positions: &[Vec<u8>],
//         proof: &[Vec<u8>],
//         proof_flags: &[bool],
//         sequence_number_option: Option<u64>,
//     ) -> Result<(), Error> {
//         let module_id = ModuleId::new(self.contract_addr, "market".to_string());

//         let args = vec![
//             bcs::to_bytes(&AccountAddress::from_str(market_addr).unwrap()).unwrap(),
//             bcs::to_bytes(positions).unwrap(),
//             bcs::to_bytes(proof).unwrap(),
//             bcs::to_bytes(proof_flags).unwrap(),
//         ];

//         let payload = TransactionPayload::EntryFunction(EntryFunction::new(
//             module_id,
//             "settle_position_many".to_string(),
//             vec![],
//             args,
//         ));

//         let signer = self.settler.as_ref().expect("SETTLER");
//         let private_key =
//             ed25519::PrivateKey::from_encoded_string(signer).expect("VALID_PRIVATE_KEY");

//         let txn_hash = self
//             .submit(payload, private_key, sequence_number_option)
//             .await?;
//         self.wait_for_transaction(&txn_hash, Some(Duration::from_secs(5)))
//             .await?;

//         Ok(())
//     }

//     pub async fn handle_withdrawal(
//         &self,
//         asset_metadata: &str,
//         recipient: &str,
//         sub_acc_addr: Option<&str>,
//         amount: u64,
//         withdrawal_id: u64,
//         sequence_number_option: Option<u64>,
//     ) -> Result<(), Error> {
//         let module_id = ModuleId::new(self.contract_addr, "vault".to_string());

//         // Convert sub_acc_addr to Option<AccountAddress>
//         let sub_acc_option = sub_acc_addr.map(|addr| AccountAddress::from_str(addr).unwrap());

//         let args = vec![
//             bcs::to_bytes(&AccountAddress::from_str(asset_metadata).unwrap()).unwrap(),
//             bcs::to_bytes(&AccountAddress::from_str(recipient).unwrap()).unwrap(),
//             bcs::to_bytes(&sub_acc_option).unwrap(),
//             bcs::to_bytes(&amount).unwrap(),
//             bcs::to_bytes(&withdrawal_id).unwrap(),
//         ];

//         let payload = TransactionPayload::EntryFunction(EntryFunction::new(
//             module_id,
//             "handle_withdrawal".to_string(),
//             vec![],
//             args,
//         ));

//         let signer = self.settler.as_ref().expect("SETTLER");
//         let private_key =
//             ed25519::PrivateKey::from_encoded_string(signer).expect("VALID_PRIVATE_KEY");

//         let txn_hash = self
//             .submit(payload, private_key, sequence_number_option)
//             .await?;
//         self.wait_for_transaction(&txn_hash, Some(Duration::from_secs(10)))
//             .await?;

//         Ok(())
//     }
// }

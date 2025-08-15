use aptos_crypto::{ed25519, HashValue, ValidCryptoMaterialStringExt};
use aptos_rust_sdk::client::{
    builder::AptosClientBuilder, config::AptosNetwork, rest_api::AptosFullnodeClient,
};
use aptos_rust_sdk_types::{
    api_types::{
        address::AccountAddress,
        chain_id::ChainId,
        module_id::ModuleId,
        transaction::{EntryFunction, RawTransaction, SignedTransaction, TransactionPayload},
        transaction_authenticator::{AuthenticationKey, TransactionAuthenticator},
    },
    error::RestError,
};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, time::Duration};
use tokio::time::Instant;
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultId {
    pub inner: String,
}

pub struct VaultContract {
    pub client: AptosFullnodeClient,
    pub contract_addr: AccountAddress,
    pub asset_addr: AccountAddress,
}
#[derive(Debug)]
pub enum TransactionStatus {
    Confirmed,
    Pending,
    Failed(String),
}

impl VaultContract {
    pub fn new(contract_addr: &str, asset_addr: &str, network: &str) -> Self {
        let network = match network {
            "mainnet" => AptosNetwork::mainnet(),
            "testnet" => AptosNetwork::testnet(),
            "devnet" => AptosNetwork::devnet(),
            _ => {
                panic!("INVALID_NETWORK")
            }
        };

        let client = AptosClientBuilder::new(network).build();
        let contract_addr = AccountAddress::from_str(&contract_addr).unwrap();
        let asset_addr = AccountAddress::from_str(asset_addr).unwrap();

        Self {
            client,
            contract_addr,
            asset_addr,
        }
    }

    pub async fn get_sequence_number(&self, sender: &AccountAddress) -> Result<u64, RestError> {
        let resource = self
            .client
            .get_account_resources(sender.to_string())
            .await?
            .into_inner();

        let sequence_number = resource
            .iter()
            .find(|r| r.type_ == "0x1::account::Account")
            .unwrap()
            .data
            .get("sequence_number")
            .unwrap()
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();

        Ok(sequence_number)
    }

    pub fn get_chain_id(&self) -> ChainId {
        let chain_id = match self.client.network().name() {
            "mainnet" => ChainId::Mainnet,
            "testnet" => ChainId::Testnet,
            "devnet" => ChainId::Other(0),
            _ => {
                panic!("INVALID_NETWORK")
            }
        };

        chain_id
    }

    pub async fn build_raw_txn(
        &self,
        payload: TransactionPayload,
        sender: AccountAddress,
        sequence_number_option: Option<u64>,
    ) -> Result<RawTransaction, RestError> {
        let state = self.client.get_state().await?;

        let max_gas_amount = 1000;
        let gas_unit_price = 100;
        let expiration_timestamp_secs = state.timestamp_usecs / 1000 / 1000 + 60 * 10;

        let sequence_number = if sequence_number_option.is_some() {
            sequence_number_option.unwrap()
        } else {
            self.get_sequence_number(&sender).await?
        };

        let chain_id = self.get_chain_id();

        let raw_txn = RawTransaction::new(
            sender,
            sequence_number,
            payload,
            max_gas_amount,
            gas_unit_price,
            expiration_timestamp_secs,
            chain_id,
        );

        Ok(raw_txn)
    }

    pub fn sign_txn(
        &self,
        raw_txn: &RawTransaction,
        signer: ed25519::PrivateKey,
    ) -> ed25519::Signature {
        let hash = HashValue::sha3_256_of("APTOS::RawTransaction".as_bytes());
        let mut bytes = vec![];
        bcs::serialize_into(&mut bytes, raw_txn).unwrap();

        let mut message = vec![];
        message.extend(hash.to_vec());
        message.extend(bytes);

        signer.sign_message(&message)
    }

    pub async fn submit(
        &self,
        payload: TransactionPayload,
        signer: ed25519::PrivateKey,
        sequence_number_option: Option<u64>,
    ) -> Result<String, RestError> {
        let public_key = ed25519::PublicKey::from(&signer);
        let auth_key = AuthenticationKey::ed25519(&public_key);
        let sender = auth_key.account_address();

        println!("Public key: {}", public_key);
        println!("Account address: {}", sender);
        // Generate raw transaction
        let raw_txn = self
            .build_raw_txn(payload, sender, sequence_number_option)
            .await?;

        // Sign transaction
        let signature = self.sign_txn(&raw_txn, signer);
        // Submit transaction
        let resp = self
            .client
            .submit_transaction(SignedTransaction::new(
                raw_txn,
                TransactionAuthenticator::ed25519(public_key, signature),
            ))
            .await?;

        let txn_hash = resp.inner().get("hash").unwrap().as_str().unwrap();
        println!("TXN_HASH: {:?}", txn_hash);

        Ok(txn_hash.to_string())
    }

    pub async fn wait_for_transaction(
        &self,
        txn_hash: &str,
        timeout: Option<Duration>,
    ) -> Result<(), RestError> {
        let deadline = timeout.map(|t| Instant::now() + t);
        loop {
            if let Some(deadline) = deadline {
                if Instant::now() >= deadline {
                    return Err(RestError::Timeout("Transaction timeout"));
                }
            }

            let status = match self
                .client
                .get_transaction_by_hash(txn_hash.to_string())
                .await
            {
                Ok(resp) => match resp.inner().get("success") {
                    Some(success) => {
                        if success.as_bool().unwrap() {
                            TransactionStatus::Confirmed
                        } else {
                            let vm_status =
                                resp.inner().get("vm_status").unwrap().as_str().unwrap();
                            TransactionStatus::Failed(vm_status.to_string())
                        }
                    }
                    _ => TransactionStatus::Pending,
                },
                // Not found, let's wait
                Err(_) => TransactionStatus::Pending,
            };

            println!("TX_STATUS: {:?}", status);
            match status {
                TransactionStatus::Confirmed => {
                    return Ok(());
                }
                TransactionStatus::Failed(_status) => {
                    return Err(RestError::Timeout("Transaction was failed"));
                }
                _ => {}
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
    pub async fn deposit_into_user(
        &self,
        amount: u128,
        private_key: &str,
    ) -> Result<String, RestError> {
        println!("Depositing {} into vault", amount);
        let signer = ed25519::Ed25519PrivateKey::from_encoded_string(&private_key).unwrap();
        let public_key = ed25519::PublicKey::from(&signer);
        let auth_key = AuthenticationKey::ed25519(&public_key);
        let acc_addr = auth_key.account_address();
        println!("Public key: {}", public_key);
        println!("Account address: {}", acc_addr);
        println!("Vault contract address: {}", self.contract_addr);
        let arguments = vec![
            bcs::to_bytes(&self.asset_addr).unwrap(),
            bcs::to_bytes(&amount).unwrap(), // Convert u128 to bytes
        ];
        let entry_function = EntryFunction::new(
            ModuleId::new(self.contract_addr, "vault".to_string()),
            "deposit_into_user".to_string(),
            vec![],
            arguments,
        );

        let sequence_number = self
            .get_sequence_number(&acc_addr)
            .await
            .map_err(|_e| RestError::Timeout("Failed to get sequence number"))?;
        println!(
            "Start submitting transaction with sequence number: {}",
            sequence_number
        );
        self.submit(
            TransactionPayload::EntryFunction(entry_function),
            signer,
            Some(sequence_number),
        )
        .await
    }
    pub async fn withdraw_from_user(
        &self,
        amount: u128,
        private_key: &str,
    ) -> Result<String, RestError> {
        println!("Withdrawing {} from vault", amount);
        let signer = ed25519::Ed25519PrivateKey::from_encoded_string(&private_key).unwrap();
        let public_key = ed25519::PublicKey::from(&signer);
        let auth_key = AuthenticationKey::ed25519(&public_key);
        let acc_addr = auth_key.account_address();
        println!("Public key: {}", public_key);
        println!("Account address: {}", acc_addr);
        println!("Vault contract address: {}", self.contract_addr);
        let arguments = vec![
            bcs::to_bytes(&self.asset_addr).unwrap(),
            bcs::to_bytes(&amount).unwrap(), // Convert u128 to bytes
        ];
        let entry_function = EntryFunction::new(
            ModuleId::new(self.contract_addr, "vault".to_string()),
            "withdraw_from_user".to_string(),
            vec![],
            arguments,
        );

        let sequence_number = self
            .get_sequence_number(&acc_addr)
            .await
            .map_err(|_e| RestError::Timeout("Failed to get sequence number"))?;
        println!(
            "Start submitting transaction with sequence number: {}",
            sequence_number
        );
        self.submit(
            TransactionPayload::EntryFunction(entry_function),
            signer,
            Some(sequence_number),
        )
        .await
    }
}

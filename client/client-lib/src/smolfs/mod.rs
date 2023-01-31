use std::sync::Arc;

use bitcoin::Address;
use bitcoin::KeyPair;
use bitcoin_hashes::sha256;
use bitcoin_hashes::Hash;
use db::PegInKey;
use fedimint_api::core::client::ClientModule;
use fedimint_api::db::DatabaseTransaction;
use fedimint_api::module::TransactionItemAmount;
use fedimint_api::OutPoint;
use fedimint_api::PeerId;
use fedimint_api::{Amount, ServerModule};
use fedimint_core::modules::smolfs::common::SmolFSDecoder;
use fedimint_core::modules::smolfs::config::SmolFSClientConfig;
use fedimint_core::modules::smolfs::db::ExampleKey;
use fedimint_core::modules::wallet::common::WalletDecoder;
use fedimint_core::modules::wallet::config::WalletClientConfig;
use fedimint_core::modules::wallet::tweakable::Tweakable;
use fedimint_core::modules::wallet::txoproof::{PegInProof, PegInProofError, TxOutProof};
use fedimint_core::modules::wallet::{Wallet, WalletOutputOutcome};
use fedimint_smolfs::db::FinishedSmolFSEntry;
use rand::{CryptoRng, RngCore};
use thiserror::Error;
use tracing::debug;

use crate::api::GlobalFederationApi;
use crate::api::OutputOutcomeError;
use crate::api::SmolFSFederationApi;
use crate::api::WalletFederationApi;
use crate::utils::ClientContext;
use crate::MemberError;

pub mod db;
use fedimint_core::modules::smolfs::*;

/// Federation module client for the Wallet module. It can both create transaction inputs and
/// outputs of the wallet (on-chain) type.
#[derive(Debug)]
pub struct SmolFSClient {
    pub config: SmolFSClientConfig,
    pub context: Arc<ClientContext>,
}

impl ClientModule for SmolFSClient {
    const KIND: &'static str = "smolfs";
    type Decoder = <SmolFS as ServerModule>::Decoder;
    type Module = SmolFS;

    fn decoder(&self) -> Self::Decoder {
        SmolFSDecoder
    }

    fn input_amount(
        &self,
        _input: &<Self::Module as ServerModule>::Input,
    ) -> TransactionItemAmount {
        TransactionItemAmount {
            amount: Amount::from_sats(0),
            fee: Amount::from_sats(0),
        }
    }

    fn output_amount(
        &self,
        _output: &<Self::Module as ServerModule>::Output,
    ) -> TransactionItemAmount {
        TransactionItemAmount {
            amount: Amount::from_sats(0),
            fee: Amount::from_sats(0),
        }
    }
}

impl SmolFSClient {
    /// Returns a bitcoin-address derived from the federations peg-in-descriptor and a random tweak
    ///
    /// This function will create a public/secret [keypair](bitcoin::KeyPair). The public key is used to tweak the
    /// federations peg-in-descriptor resulting in a bitcoin script. Both script and keypair are stored in the DB
    /// by using the script as part of the key and the keypair as the value. Even though only the public-key is used to tweak
    /// the descriptor, the secret-key is needed to prove that one actually created the tweak to be able to claim the funds and
    /// prevent front-running by a malicious  federation member
    /// The returned bitcoin-address is derived from the script. Thus sending bitcoin to that address will result in a
    /// transaction containing the scripts public-key in at least one of it's outpoints.
    pub async fn add_entry<'a>(
        &self,
        // dbtx: &mut DatabaseTransaction<'a>,
        pubkey: String,
        backup: String,
        // mut rng: R,
    ) -> String {
        // self.context.api.fetch_backups_by_pubkey(pubkey);
        // let mut dbtx = self.context.db.begin_transaction().await;
        let peerid = PeerId::from(0);
        let a = self
            .context
            .api
            .request_raw(
                peerid,
                "smolfsput",
                &[serde_json::Value::String(format!("{pubkey} {backup}"))],
            )
            .await
            .unwrap();
        // let mut dbtx = db.begin_transaction().await;

        println!("add entry generating fake smolfs entry {:?}", a);

        pubkey
    }
    pub async fn get_entry<'a>(
        &self,
        dbtx: &mut DatabaseTransaction<'a>,
        pubkey: String,
        // mut rng: R,
    ) -> String {
        println!("get entry");
        // self.context.api.fetch_backups_by_pubkey(pubkey).await.unwrap().unwrap()
        let mut dbtx = dbtx.with_module_prefix(3);
        let a = dbtx.get_value(&ExampleKey(pubkey)).await.unwrap().unwrap();
        // String::new()
        a
    }
}

type Result<T> = std::result::Result<T, WalletClientError>;

#[derive(Error, Debug)]
pub enum WalletClientError {
    #[error("Could not find an ongoing matching peg-in")]
    NoMatchingPegInFound,
    #[error("Peg-in amount must be greater than peg-in fee")]
    PegInAmountTooSmall,
    #[error("Inconsistent peg-in proof: {0}")]
    PegInProofError(PegInProofError),
    #[error("Output outcome error: {0}")]
    OutputOutcomeError(#[from] OutputOutcomeError),
    #[error("Mint API error: {0}")]
    ApiError(#[from] MemberError),
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;

    use bitcoin::hashes::sha256;
    use bitcoin::{Address, Txid};
    use bitcoin_hashes::Hash;
    use fedimint_api::config::ConfigGenParams;
    use fedimint_api::core::{
        DynOutputOutcome, ModuleInstanceId, LEGACY_HARDCODED_INSTANCE_ID_WALLET,
    };
    use fedimint_api::db::mem_impl::MemDatabase;
    use fedimint_api::db::Database;
    use fedimint_api::module::registry::ModuleDecoderRegistry;
    use fedimint_api::task::TaskGroup;
    use fedimint_api::{Feerate, OutPoint, TransactionId};
    use fedimint_core::modules::smolfs::common::SmolFSDecoder;
    use fedimint_core::modules::smolfs::config::SmolFSClientConfig;
    use fedimint_core::modules::smolfs::{
        SmolFS, SmolFSConfigGenParams, SmolFSConfigGenerator, SmolFSEntry, SmolFSOutput,
        SmolFSOutputOutcome,
    };
    use fedimint_core::modules::wallet::common::WalletDecoder;
    use fedimint_core::modules::wallet::config::WalletClientConfig;
    use fedimint_core::modules::wallet::{
        PegOut, PegOutFees, Wallet, WalletGen, WalletGenParams, WalletOutput, WalletOutputOutcome,
    };
    use fedimint_core::outcome::{SerdeOutputOutcome, TransactionStatus};
    use fedimint_testing::btc::bitcoind::{FakeBitcoindRpc, FakeBitcoindRpcController};
    use fedimint_testing::FakeFed;
    use tokio::sync::Mutex;
    use tracing::info;

    use crate::api::fake::FederationApiFaker;
    use crate::smolfs::SmolFSClient;
    use crate::wallet::WalletClient;
    use crate::{module_decode_stubs, ClientContext};

    type Fed = FakeFed<SmolFS>;
    type SharedFed = Arc<tokio::sync::Mutex<Fed>>;

    #[derive(Debug)]
    struct FakeApi {
        // for later use once wallet outcomes are implemented
        _mint: SharedFed,
    }

    pub async fn make_test_smolfs_fed(
        module_id: ModuleInstanceId,
        fed: Arc<Mutex<FakeFed<SmolFS>>>,
    ) -> FederationApiFaker<tokio::sync::Mutex<FakeFed<SmolFS>>> {
        let members = fed
            .lock()
            .await
            .members
            .iter()
            .map(|(peer_id, _, _, _)| *peer_id)
            .collect();
        FederationApiFaker::new(fed, members)
    }

    async fn new_mint_and_client(
        task_group: &mut TaskGroup,
    ) -> (
        Arc<tokio::sync::Mutex<Fed>>,
        SmolFSClientConfig,
        ClientContext,
        //     FakeBitcoindRpcController,
    ) {
        let module_id = 3;
        let fed = Arc::new(tokio::sync::Mutex::new(
            FakeFed::<SmolFS>::new(
                4,
                move |cfg, db| async move { Ok(SmolFS::new(cfg.to_typed().unwrap()).await) },
                &ConfigGenParams::new().attach(SmolFSConfigGenParams {
                    important_param: 10,
                }),
                &SmolFSConfigGenerator,
                module_id,
            )
            .await
            .unwrap(),
        ));

        let api = make_test_smolfs_fed(module_id, fed.clone()).await;
        let client_config = fed.lock().await.client_cfg().clone();

        let client = ClientContext {
            decoders: ModuleDecoderRegistry::from_iter([(module_id, SmolFSDecoder.into())]),
            module_gens: Default::default(),
            db: Database::new(MemDatabase::new(), module_decode_stubs()),
            api: api.into(),
            secp: secp256k1_zkp::Secp256k1::new(),
        };

        (
            fed,
            client_config.cast().unwrap(),
            client,
            // btc_rpc_controller,
        )
    }

    #[test_log::test(tokio::test)]
    async fn create_output_for_smolfs() {
        let mut task_group = TaskGroup::new();
        let (fed, client_config, client_context) = new_mint_and_client(&mut task_group).await;
        let _client = SmolFSClient {
            config: client_config,
            context: Arc::new(client_context),
        };
        info!("create_output");

        let pubkey = "pubkey".to_string();
        let backup = "backup".to_string();
        // _client.add_entry(dbtx, pubkey, backup)A
        // let outpoint = OutPoint{txid:fedimint_api::TransactionId::from_slice([0;32].as_slice()).unwrap(), out_idx: 1 };
        let outpoint = OutPoint {
            txid: sha256::Hash::hash(pubkey.as_bytes()).into(),
            out_idx: 0,
        };
        fed.lock().await.generate_fake_smolfs_entry(outpoint).await;
        fed.lock().await.get_fake_smolfs_entry().await;
        fed.lock().await.consensus_round(&[], &[]).await;
        let output = SmolFSOutput(Box::new(SmolFSEntry { pubkey, backup }));
        let outputs = [(outpoint, output)];
        println!("consensus with outputs");
        fed.lock().await.consensus_round(&[], &outputs).await;
        println!("empty consensus");
        fed.lock().await.consensus_round(&[], &[]).await;
        let output_outcome = fed.lock().await.output_outcome(outpoint).await;
        println!("{:?}", output_outcome);
        let backup = fed
            .lock()
            .await
            .fetch_from_all(|m, db, module_instance_id| async {
                m.get_backups(
                    &mut db
                        .begin_transaction()
                        .await
                        .with_module_prefix(*module_instance_id),
                    "pubkey".to_string(),
                )
                .await
            })
            .await;
        assert_eq!("backup", backup)
    }
}

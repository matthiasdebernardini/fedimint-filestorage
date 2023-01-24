use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::OsString;
use std::fmt::{self};

use async_trait::async_trait;
use bitcoin::hashes::sha256;
use common::SmolFSDecoder;
use config::SmolFSClientConfig;
use db::{ExampleKey, ExampleKeyPrefix};
use fedimint_api::cancellable::Cancellable;
use fedimint_api::config::{
    ConfigGenParams, DkgPeerMsg, ModuleGenParams, ServerModuleConfig, TypedServerModuleConfig,
};
use fedimint_api::config::{ModuleConfigResponse, TypedServerModuleConsensusConfig};
use fedimint_api::core::{ModuleInstanceId, ModuleKind};
use fedimint_api::db::{Database, DatabaseTransaction};
use fedimint_api::encoding::{Decodable, Encodable};
use fedimint_api::module::__reexports::serde_json;
use fedimint_api::module::audit::Audit;
use fedimint_api::module::interconnect::ModuleInterconect;
use fedimint_api::module::{
    api_endpoint, ApiEndpoint, InputMeta, ModuleError, ModuleGen, TransactionItemAmount,
};
use fedimint_api::net::peers::MuxPeerConnections;
use fedimint_api::server::DynServerModule;
use fedimint_api::task::TaskGroup;
use fedimint_api::{plugin_types_trait_impl, OutPoint, PeerId, ServerModule};
use impl_tools::autoimpl;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, warn};

use crate::config::{SmolFSConfig, SmolFSConfigConsensus, SmolFSConfigLocal};

pub mod common;
pub mod config;
pub mod db;

const KIND: ModuleKind = ModuleKind::from_static_str("smolfs");

/// SmolFS module
#[derive(Debug)]
pub struct SmolFS {
    pub cfg: SmolFSConfig,
}
#[autoimpl(Deref, DerefMut using self.0)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Encodable, Decodable)]
pub struct SmolFSOutputConfirmation(pub SmolFSEntry);

#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Encodable, Decodable, Default,
)]
pub struct SmolFSEntry {
    pub pubkey: String,
    pub backup: String,
}

#[derive(Debug, Clone)]
pub struct SmolFSVerificationCache {
    valid_users: HashMap<String, String>,
}

#[derive(Debug)]
pub struct SmolFSConfigGenerator;

#[async_trait]
impl ModuleGen for SmolFSConfigGenerator {
    const KIND: ModuleKind = KIND;
    type Decoder = SmolFSDecoder;

    fn decoder(&self) -> SmolFSDecoder {
        SmolFSDecoder
    }

    async fn init(
        &self,
        cfg: ServerModuleConfig,
        _db: Database,
        _env: &BTreeMap<OsString, OsString>,
        _task_group: &mut TaskGroup,
    ) -> anyhow::Result<DynServerModule> {
        info!("module gen init");
        Ok(SmolFS::new(cfg.to_typed()?).into())
    }

    fn trusted_dealer_gen(
        &self,
        peers: &[PeerId],
        params: &ConfigGenParams,
    ) -> BTreeMap<PeerId, ServerModuleConfig> {
        info!("trusted dealer gen");
        let params = params
            .get::<SmolFSConfigGenParams>()
            .expect("Invalid mint params");
        let mint_cfg: BTreeMap<_, SmolFSConfig> = peers
            .iter()
            .map(|&peer| {
                let config = SmolFSConfig {
                    local: SmolFSConfigLocal {
                        pubkey: String::from("trusted dealer gen"),
                        backup: String::new(),
                    },
                    consensus: SmolFSConfigConsensus {
                        merkle_root: vec![4 as u8, 2 as u8],
                    },
                };
                (peer, config)
            })
            .collect();

        mint_cfg
            .into_iter()
            .map(|(k, v)| (k, v.to_erased()))
            .collect()
    }

    async fn distributed_gen(
        &self,
        _connections: &MuxPeerConnections<ModuleInstanceId, DkgPeerMsg>,
        _our_id: &PeerId,
        _instance_id: ModuleInstanceId,
        _peers: &[PeerId],
        params: &ConfigGenParams,
        _task_group: &mut TaskGroup,
    ) -> anyhow::Result<Cancellable<ServerModuleConfig>> {
        let _params = params
            .get::<SmolFSConfigGenParams>()
            .expect("Invalid mint params");

        let server = SmolFSConfig {
            local: SmolFSConfigLocal {
                pubkey: String::from("distributed gen"),
                backup: String::new(),
            },
            consensus: SmolFSConfigConsensus {
                merkle_root: vec![],
            },
        };

        Ok(Ok(server.to_erased()))
    }

    fn to_config_response(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<ModuleConfigResponse> {
        let config = serde_json::from_value::<SmolFSConfigConsensus>(config)?;

        Ok(ModuleConfigResponse {
            client: config.to_client_config(),
            consensus_hash: config.consensus_hash()?,
        })
    }

    fn validate_config(&self, identity: &PeerId, config: ServerModuleConfig) -> anyhow::Result<()> {
        config.to_typed::<SmolFSConfig>()?.validate_config(identity)
    }
    fn hash_client_module(&self, config: serde_json::Value) -> anyhow::Result<sha256::Hash> {
        serde_json::from_value::<SmolFSClientConfig>(config)?.consensus_hash()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmolFSConfigGenParams {
    //TODO:Change to max size of buffer
    pub important_param: u64,
}

impl ModuleGenParams for SmolFSConfigGenParams {
    const MODULE_NAME: &'static str = "smolfs";
}

#[autoimpl(Deref, DerefMut using self.0)]
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Encodable, Decodable, Default,
)]
pub struct SmolFSInput(pub Box<SmolFSEntry>);

impl fmt::Display for SmolFSInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SmolFSInput {:?}", self.0)
    }
}

#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Encodable, Decodable, Default,
)]
pub struct SmolFSOutput();

impl fmt::Display for SmolFSOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DummyOutput")
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Encodable, Decodable)]
pub struct SmolFSOutputOutcome;

impl fmt::Display for SmolFSOutputOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DummyOutputOutcome")
    }
}

impl fmt::Display for SmolFSOutputConfirmation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DummyOutputConfirmation")
    }
}

#[async_trait]
impl ServerModule for SmolFS {
    const KIND: ModuleKind = KIND;

    type Decoder = SmolFSDecoder;
    type Input = SmolFSInput;
    type Output = SmolFSOutput;
    type OutputOutcome = SmolFSOutputOutcome;
    type ConsensusItem = SmolFSOutputConfirmation;
    type VerificationCache = SmolFSVerificationCache;

    fn decoder(&self) -> Self::Decoder {
        SmolFSDecoder
    }

    async fn await_consensus_proposal(&self, dbtx: &mut DatabaseTransaction<'_>) {
        info!("await consensus proposal");
        if self.consensus_proposal(dbtx).await.is_empty() {
            std::future::pending().await
        }
    }

    async fn consensus_proposal(
        &self,
        dbtx: &mut DatabaseTransaction<'_>,
    ) -> Vec<Self::ConsensusItem> {
        info!("consensus proposal");
        let pubkey = self.cfg.local.pubkey.clone();
        let backup = self.cfg.local.backup.clone();
        info!("pubkey {pubkey} backup {backup}");
        let dbvec: Vec<Self::ConsensusItem> = dbtx
            .find_by_prefix(&ExampleKeyPrefix)
            .await
            .map(|res| {
                let res = res.expect("DB Error");
                SmolFSOutputConfirmation(SmolFSEntry {
                    // pubkey: res.0 .0,
                    // backup: res.1,
                    pubkey: String::from("npubTEST"),
                    backup: String::from("backupTEST"),
                })
            })
            // .chain(std::iter::once(round_ci))
            .collect();
        println!("dbvec {dbvec:?}");
        // dbvec
        vec![SmolFSOutputConfirmation(SmolFSEntry {
            pubkey: String::from("npubTEST"),
            backup: String::from("backupTEST"),
        })]
    }

    async fn begin_consensus_epoch<'a, 'b>(
        &'a self,
        dbtx: &mut DatabaseTransaction<'b>,
        _consensus_items: Vec<(PeerId, Self::ConsensusItem)>,
    ) {
        info!("begin consensus epoch");
        let pubkey = self.cfg.local.pubkey.clone();
        let backup = self.cfg.local.backup.clone();
        let mut b = String::new();
        dbtx.insert_entry(
            &ExampleKey("inserting entry".to_string()),
            &"inserting backup".to_string(),
        )
        .await
        .expect("DB Error")
        .map(|a| b = a);
        // .unwrap();
        println!("printing self {self:?}");
        println!("{:?}", b);
    }

    fn build_verification_cache<'a>(
        &'a self,
        inputs: impl Iterator<Item = &'a Self::Input> + Send,
    ) -> Self::VerificationCache {
        info!("build verification");

        // why not
        let valid_users = inputs
            .flat_map(|inputs| {
                let mut h = HashMap::new();
                h.entry(inputs.pubkey.clone())
                    .or_insert(inputs.backup.clone());
                h
            })
            .collect();

        SmolFSVerificationCache { valid_users }
    }

    async fn validate_input<'a, 'b>(
        &self,
        _interconnect: &dyn ModuleInterconect,
        _dbtx: &mut DatabaseTransaction<'b>,
        _verification_cache: &Self::VerificationCache,
        _input: &'a Self::Input,
    ) -> Result<InputMeta, ModuleError> {
        info!("validate input");
        // TODO attach a payment to the backup, include details here
        // fill the pubkey vectors with payments destined to the guardians
        // make ecash wallet for fed module then use interconnect to pay to it
        Ok(InputMeta {
            amount: TransactionItemAmount {
                amount: fedimint_api::Amount::from_sats(0),
                fee: fedimint_api::Amount::from_sats(0),
            },
            puk_keys: vec![],
        })
    }

    async fn apply_input<'a, 'b, 'c>(
        &'a self,
        interconnect: &'a dyn ModuleInterconect,
        dbtx: &mut DatabaseTransaction<'c>,
        input: &'b Self::Input,
        cache: &Self::VerificationCache,
    ) -> Result<InputMeta, ModuleError> {
        info!("Applying input");
        let meta = self
            .validate_input(interconnect, dbtx, cache, input)
            .await?;
        let input = input.to_owned().0;
        let pubkey = input.pubkey;
        let backup = input.backup;
        let key = ExampleKey(pubkey);
        let value = backup;
        dbtx.insert_new_entry(&key, &value).await.expect("DB Error");

        Ok(meta)
    }

    async fn validate_output(
        &self,
        _dbtx: &mut DatabaseTransaction,
        _output: &Self::Output,
    ) -> Result<TransactionItemAmount, ModuleError> {
        unimplemented!()
    }

    async fn apply_output<'a, 'b>(
        &'a self,
        _dbtx: &mut DatabaseTransaction<'b>,
        _output: &'a Self::Output,
        _out_point: OutPoint,
    ) -> Result<TransactionItemAmount, ModuleError> {
        unimplemented!()
    }

    async fn end_consensus_epoch<'a, 'b>(
        &'a self,
        _consensus_peers: &HashSet<PeerId>,
        _dbtx: &mut DatabaseTransaction<'b>,
    ) -> Vec<PeerId> {
        vec![]
    }

    async fn output_status(
        &self,
        _dbtx: &mut DatabaseTransaction<'_>,
        _out_point: OutPoint,
    ) -> Option<Self::OutputOutcome> {
        None
    }

    async fn audit(&self, _dbtx: &mut DatabaseTransaction<'_>, _audit: &mut Audit) {}

    fn api_endpoints(&self) -> Vec<ApiEndpoint<Self>> {
        vec![api_endpoint! {
            "/smolfs",
            async |_module: &SmolFS, _dbtx, _request: String| -> () {
                Ok(())
            }
        }]
    }
}

impl SmolFS {
    /// Create new module instance
    pub fn new(cfg: SmolFSConfig) -> SmolFS {
        SmolFS { cfg }
    }
}

// Must be unique.
// TODO: we need to provide guidence for allocating these
pub const MODULE_KEY_SMOLFS: u16 = 128;
plugin_types_trait_impl!(
    MODULE_KEY_SMOLFS,
    SmolFSInput,
    SmolFSOutput,
    SmolFSOutputOutcome,
    SmolFSOutputConfirmation,
    SmolFSVerificationCache
);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Error)]
pub enum SmolFSError {
    #[error("Something went wrong")]
    SomethingDummyWentWrong,
}

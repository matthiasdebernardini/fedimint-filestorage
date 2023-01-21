use std::collections::{BTreeMap, HashSet};
use std::ffi::OsString;
use std::fmt::{self};

use async_trait::async_trait;
use common::SmolFSDecoder;
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

use crate::config::{SmolFSConfig, SmolFSConfigConsensus, SmolFSConfigLocal};

pub mod common;
pub mod config;
pub mod db;

const KIND: ModuleKind = ModuleKind::from_static_str("SmolFS");

/// SmolFS module
#[derive(Debug)]
pub struct SmolFS {
    pub cfg: SmolFSConfig,
}
#[autoimpl(Deref, DerefMut using self.0)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Encodable, Decodable)]
pub struct SmolFSOutputConfirmation(pub SmolFSEntry);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Encodable, Decodable)]
pub struct SmolFSEntry {
    pubkey: String,
    backup: String,
}

#[derive(Debug, Clone)]
pub struct SmolFSVerificationCache;

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
        Ok(SmolFS::new(cfg.to_typed()?).into())
    }

    fn trusted_dealer_gen(
        &self,
        peers: &[PeerId],
        params: &ConfigGenParams,
    ) -> BTreeMap<PeerId, ServerModuleConfig> {
        let params = params
            .get::<SmolFSConfigGenParams>()
            .expect("Invalid mint params");
        let mint_cfg: BTreeMap<_, SmolFSConfig> = peers
            .iter()
            .map(|&peer| {
                let config = SmolFSConfig {
                    local: SmolFSConfigLocal {
                        pubkey: String::new(),
                        backup: String::new(),
                    },
                    consensus: SmolFSConfigConsensus {
                        merkle_root: vec![],
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
                pubkey: String::new(),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmolFSConfigGenParams {
    //TODO:Change to max size of buffer
    pub important_param: u64,
}

impl ModuleGenParams for SmolFSConfigGenParams {
    const MODULE_NAME: &'static str = "SmolFS";
}

#[autoimpl(Deref, DerefMut using self.0)]
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Encodable, Decodable, Default,
)]
pub struct SmolFSInput(pub Vec<String>);

impl fmt::Display for SmolFSInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DummyInput {:?}", self.0)
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

    async fn await_consensus_proposal(&self, _dbtx: &mut DatabaseTransaction<'_>) {}

    async fn consensus_proposal(
        &self,
        dbtx: &mut DatabaseTransaction<'_>,
    ) -> Vec<Self::ConsensusItem> {
        dbtx.find_by_prefix(&ExampleKeyPrefix)
            .await
            .map(|res| {
                let res = res.expect("DB Error");
                SmolFSOutputConfirmation(SmolFSEntry {
                    pubkey: format!("{:?}", res.0),
                    backup: res.1,
                })
            })
            // .chain(std::iter::once(round_ci))
            .collect()
    }

    async fn begin_consensus_epoch<'a, 'b>(
        &'a self,
        dbtx: &mut DatabaseTransaction<'b>,
        _consensus_items: Vec<(PeerId, Self::ConsensusItem)>,
    ) {
        let pubkey = self.cfg.local.pubkey.clone();
        let backup = self.cfg.local.backup.clone();
        let a = dbtx
            .insert_entry(&ExampleKey(pubkey), &backup)
            .await
            .expect("DB Error")
            .unwrap();
        println!("{self:?}");
        println!("{:?}", a);
    }

    fn build_verification_cache<'a>(
        &'a self,
        _inputs: impl Iterator<Item = &'a Self::Input> + Send,
    ) -> Self::VerificationCache {
        SmolFSVerificationCache
    }

    async fn validate_input<'a, 'b>(
        &self,
        _interconnect: &dyn ModuleInterconect,
        _dbtx: &mut DatabaseTransaction<'b>,
        _verification_cache: &Self::VerificationCache,
        _input: &'a Self::Input,
    ) -> Result<InputMeta, ModuleError> {
        todo!("Get input from config here");
    }

    async fn apply_input<'a, 'b, 'c>(
        &'a self,
        _interconnect: &'a dyn ModuleInterconect,
        _dbtx: &mut DatabaseTransaction<'c>,
        _input: &'b Self::Input,
        _cache: &Self::VerificationCache,
    ) -> Result<InputMeta, ModuleError> {
        unimplemented!()
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
            "/SmolFS",
            async |_module: &SmolFS, _dbtx, _request: ()| -> () {
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
pub const MODULE_KEY_DUMMY: u16 = 128;
plugin_types_trait_impl!(
    MODULE_KEY_DUMMY,
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

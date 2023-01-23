use fedimint_api::config::{
    ClientModuleConfig, TypedClientModuleConfig, TypedServerModuleConfig,
    TypedServerModuleConsensusConfig,
};
use fedimint_api::core::ModuleKind;
use fedimint_api::encoding::Encodable;
use fedimint_api::module::__reexports::serde_json;
use fedimint_api::PeerId;
use serde::{Deserialize, Serialize};

use crate::KIND;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Encodable)]
pub struct SmolFSConfig {
    pub local: SmolFSConfigLocal,
    /// Contains all configuration that needs to be the same for every federation member
    pub consensus: SmolFSConfigConsensus,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Encodable)]
pub struct SmolFSConfigConsensus {
    pub merkle_root: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Encodable)]
pub struct SmolFSConfigLocal {
    pub pubkey: String,
    pub backup: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encodable)]
pub struct SmolFSClientConfig {
    pub merkle_root: Vec<u8>,
}

impl TypedClientModuleConfig for SmolFSClientConfig {
    fn kind(&self) -> fedimint_api::core::ModuleKind {
        KIND
    }
}

impl TypedServerModuleConsensusConfig for SmolFSConfigConsensus {
    fn to_client_config(&self) -> ClientModuleConfig {
        ClientModuleConfig::new(
            KIND,
            serde_json::to_value(&SmolFSClientConfig {
                merkle_root: self.merkle_root.clone(),
            })
            .expect("Serialization can't fail"),
        )
    }
}

impl TypedServerModuleConfig for SmolFSConfig {
    type Local = SmolFSConfigLocal;
    type Private = ();
    type Consensus = SmolFSConfigConsensus;

    fn from_parts(local: Self::Local, _private: Self::Private, consensus: Self::Consensus) -> Self {
        Self { local, consensus }
    }

    fn to_parts(self) -> (ModuleKind, Self::Local, Self::Private, Self::Consensus) {
        (KIND, self.local, (), self.consensus)
    }

    fn validate_config(&self, _identity: &PeerId) -> anyhow::Result<()> {
        Ok(())
    }
}

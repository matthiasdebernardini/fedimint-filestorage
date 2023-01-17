use fedimint_api::config::{
    ClientModuleConfig, TypedClientModuleConfig, TypedServerModuleConfig,
    TypedServerModuleConsensusConfig,
};
use fedimint_api::core::ModuleKind;
use fedimint_api::module::__reexports::serde_json;
use fedimint_api::PeerId;
use serde::{Deserialize, Serialize};

use crate::KIND;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DummyConfig {
    pub local: DummyConfigLocal,
    /// Contains all configuration that needs to be the same for every federation member
    pub consensus: DummyConfigConsensus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DummyConfigConsensus {
    pub merkle_root: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DummyConfigLocal {
    pub max_size: u64,
    pub new_user_backup: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DummyClientConfig {
    pub merkle_root: Vec<u8>,
}

impl TypedClientModuleConfig for DummyClientConfig {
    fn kind(&self) -> fedimint_api::core::ModuleKind {
        KIND
    }
}

impl TypedServerModuleConsensusConfig for DummyConfigConsensus {
    fn to_client_config(&self) -> ClientModuleConfig {
        ClientModuleConfig::new(
            KIND,
            serde_json::to_value(&DummyClientConfig {
                merkle_root: self.merkle_root.clone(),
            })
            .expect("Serialization can't fail"),
        )
    }
}

impl TypedServerModuleConfig for DummyConfig {
    type Local = DummyConfigLocal;
    type Private = ();
    type Consensus = DummyConfigConsensus;

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

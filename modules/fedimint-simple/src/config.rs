use anyhow::bail;
use fedimint_api::config::{
    ClientModuleConfig, TypedClientModuleConfig, TypedServerModuleConfig,
    TypedServerModuleConsensusConfig,
};
use fedimint_api::core::ModuleKind;
use fedimint_api::encoding::Encodable;
use fedimint_api::PeerId;
use serde::{Deserialize, Serialize};
use threshold_crypto::serde_impl::SerdeSecret;

use crate::KIND;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleConfig {
    /// Contains all configuration that will be encrypted such as private key material
    pub private: SimpleConfigPrivate,
    /// Contains all configuration that needs to be the same for every server
    pub consensus: SimpleConfigConsensus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encodable)]
pub struct SimpleConfigConsensus {
    /// The threshold public keys for encrypting the LN preimage
    pub threshold_pub_keys: threshold_crypto::PublicKeySet,
    /// Fees charged for LN transactions
    pub fee_consensus: FeeConsensus,
}

impl SimpleConfigConsensus {
    /// The number of decryption shares required
    pub fn threshold(&self) -> usize {
        self.threshold_pub_keys.threshold() + 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleConfigPrivate {
    // TODO: propose serde(with = "…") based protection upstream instead
    /// Our secret key for decrypting preimages
    pub threshold_sec_key: SerdeSecret<threshold_crypto::SecretKeyShare>,
}

impl TypedClientModuleConfig for SimpleClientConfig {
    fn kind(&self) -> ModuleKind {
        KIND
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Encodable)]
pub struct SimpleClientConfig {
    pub threshold_pub_key: threshold_crypto::PublicKey,
    pub fee_consensus: FeeConsensus,
}

impl TypedServerModuleConsensusConfig for SimpleConfigConsensus {
    fn to_client_config(&self) -> ClientModuleConfig {
        ClientModuleConfig::new(
            KIND,
            serde_json::to_value(&SimpleClientConfig {
                threshold_pub_key: self.threshold_pub_keys.public_key(),
                fee_consensus: self.fee_consensus.clone(),
            })
            .expect("Serialization can't fail"),
        )
    }
}

impl TypedServerModuleConfig for SimpleConfig {
    type Local = ();
    type Private = SimpleConfigPrivate;
    type Consensus = SimpleConfigConsensus;

    fn from_parts(_local: Self::Local, private: Self::Private, consensus: Self::Consensus) -> Self {
        Self { private, consensus }
    }

    fn to_parts(self) -> (ModuleKind, Self::Local, Self::Private, Self::Consensus) {
        (KIND, (), self.private, self.consensus)
    }

    fn validate_config(&self, identity: &PeerId) -> anyhow::Result<()> {
        if self.private.threshold_sec_key.public_key_share()
            != self
                .consensus
                .threshold_pub_keys
                .public_key_share(identity.to_usize())
        {
            bail!("Simple private key doesn't match pubkey share");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Encodable)]
pub struct FeeConsensus {
    pub contract_input: fedimint_api::Amount,
    pub contract_output: fedimint_api::Amount,
}

impl Default for FeeConsensus {
    fn default() -> Self {
        Self {
            contract_input: fedimint_api::Amount::ZERO,
            contract_output: fedimint_api::Amount::ZERO,
        }
    }
}

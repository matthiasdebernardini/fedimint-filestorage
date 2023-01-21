use std::io;

use fedimint_api::core::Decoder;
use fedimint_api::encoding::{Decodable, DecodeError};
use fedimint_api::module::registry::ModuleDecoderRegistry;

use crate::{SmolFSInput, SmolFSOutput, SmolFSOutputConfirmation, SmolFSOutputOutcome};

#[derive(Debug, Default, Clone)]
pub struct SmolFSDecoder;

impl Decoder for SmolFSDecoder {
    type Input = SmolFSInput;
    type Output = SmolFSOutput;
    type OutputOutcome = SmolFSOutputOutcome;
    type ConsensusItem = SmolFSOutputConfirmation;

    fn decode_input(&self, mut d: &mut dyn io::Read) -> Result<SmolFSInput, DecodeError> {
        SmolFSInput::consensus_decode(&mut d, &ModuleDecoderRegistry::default())
    }

    fn decode_output(&self, mut d: &mut dyn io::Read) -> Result<SmolFSOutput, DecodeError> {
        SmolFSOutput::consensus_decode(&mut d, &ModuleDecoderRegistry::default())
    }

    fn decode_output_outcome(
        &self,
        mut d: &mut dyn io::Read,
    ) -> Result<SmolFSOutputOutcome, DecodeError> {
        SmolFSOutputOutcome::consensus_decode(&mut d, &ModuleDecoderRegistry::default())
    }

    fn decode_consensus_item(
        &self,
        mut r: &mut dyn io::Read,
    ) -> Result<SmolFSOutputConfirmation, DecodeError> {
        SmolFSOutputConfirmation::consensus_decode(&mut r, &ModuleDecoderRegistry::default())
    }
}

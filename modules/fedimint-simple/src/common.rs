use std::io;

use fedimint_api::core::Decoder;
use fedimint_api::encoding::Decodable;
use fedimint_api::encoding::DecodeError;
use fedimint_api::module::registry::ModuleDecoderRegistry;

use crate::{SimpleConsensusItem, SimpleInput, SimpleOutput, SimpleOutputOutcome};

#[derive(Debug, Default, Clone)]
pub struct SimpleDecoder;

impl Decoder for SimpleDecoder {
    type Input = SimpleInput;
    type Output = SimpleOutput;
    type OutputOutcome = SimpleOutputOutcome;
    type ConsensusItem = SimpleConsensusItem;

    fn decode_input(&self, mut d: &mut dyn io::Read) -> Result<SimpleInput, DecodeError> {
        SimpleInput::consensus_decode(&mut d, &ModuleDecoderRegistry::default())
    }

    fn decode_output(&self, mut d: &mut dyn io::Read) -> Result<SimpleOutput, DecodeError> {
        SimpleOutput::consensus_decode(&mut d, &ModuleDecoderRegistry::default())
    }

    fn decode_output_outcome(
        &self,
        mut d: &mut dyn io::Read,
    ) -> Result<SimpleOutputOutcome, DecodeError> {
        SimpleOutputOutcome::consensus_decode(&mut d, &ModuleDecoderRegistry::default())
    }

    fn decode_consensus_item(
        &self,
        mut r: &mut dyn io::Read,
    ) -> Result<SimpleConsensusItem, DecodeError> {
        SimpleConsensusItem::consensus_decode(&mut r, &ModuleDecoderRegistry::default())
    }
}

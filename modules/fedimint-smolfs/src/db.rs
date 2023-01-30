use fedimint_api::db::DatabaseKeyPrefixConst;
use fedimint_api::encoding::{Decodable, Encodable};
use serde::Serialize;
use strum_macros::EnumIter;

use crate::SmolFSOutputOutcome;

#[repr(u8)]
#[derive(Clone, EnumIter, Debug)]
pub enum DbKeyPrefix {
    // TODO: Make sure this does not collide with other modules
    Example = 0x80,
    SmolFSOutPoint = 0x81,
}

impl std::fmt::Display for DbKeyPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Encodable, Decodable, Eq, PartialEq, Hash, Serialize)]
pub struct ExampleKey(pub String);

impl DatabaseKeyPrefixConst for ExampleKey {
    const DB_PREFIX: u8 = DbKeyPrefix::Example as u8;
    type Key = Self;
    type Value = String;
}

#[derive(Debug, Encodable, Decodable)]
pub struct ExampleKeyPrefix;

impl DatabaseKeyPrefixConst for ExampleKeyPrefix {
    const DB_PREFIX: u8 = DbKeyPrefix::Example as u8;
    type Key = ExampleKey;
    type Value = String;
}

#[derive(Clone, Debug, Encodable, Decodable, Serialize)]
pub struct FinishedSmolFSEntry(pub fedimint_api::OutPoint);

impl DatabaseKeyPrefixConst for FinishedSmolFSEntry {
    const DB_PREFIX: u8 = DbKeyPrefix::SmolFSOutPoint as u8;
    type Key = Self;
    type Value = SmolFSOutputOutcome;
}

#[derive(Clone, Debug, Encodable, Decodable)]
pub struct FinishedSmolFSEntryPrefix;

impl DatabaseKeyPrefixConst for FinishedSmolFSEntryPrefix {
    const DB_PREFIX: u8 = DbKeyPrefix::SmolFSOutPoint as u8;
    type Key = FinishedSmolFSEntry;
    type Value = SmolFSOutputOutcome;
}

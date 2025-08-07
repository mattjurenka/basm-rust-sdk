use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{io::HostResult, memory::{leak_to_shared_memory, FatPointer}};

#[link(wasm_import_module = "env")]
extern "C" {
    pub fn verifyAttestation(offset: u32, size: u32) -> FatPointer;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttestationOutput {
    pub raw_claims: Option<String>,
}

#[derive(Error, Debug)]
pub enum AttestationError {
    #[error("Bad serialization of input data: {0}")]
    BadSerialization(serde_json::Error),
    #[error("Bad deserialization of output data: {0}")]
    BadDeserialization(serde_json::Error),
    #[error("Attestation Failed: {0}")]
    AttestationFailed(String)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnclaveMeasurement {
    pub platform: String,
    pub code: String
}

pub fn verify_attestation(
    enclave_attested_pubkey: String,
    transitive_attestation: String,
    acceptable_measurements: Vec<EnclaveMeasurement>,
) -> Result<AttestationOutput, AttestationError> {
    let input = json!({
        "enclave_attested_app_public_key": enclave_attested_pubkey,
        "transitive_attestation": transitive_attestation,
        "acceptable_measurements": acceptable_measurements,
    });

    let serialized_input = serde_json::to_string(&input)
        .map_err(|e| AttestationError::BadSerialization(e))?;

    let input_ptr = leak_to_shared_memory(serialized_input.as_bytes());
    let output_ptr = unsafe { verifyAttestation(input_ptr.offset(), input_ptr.size()) };
    let data = output_ptr.copy_data();

    let host_result = serde_json::from_slice::<HostResult<AttestationOutput>>(&data)
        .map_err(|e| AttestationError::BadDeserialization(e))?;

    if host_result.ok {
        Ok(host_result.value)
    } else {
        Err(AttestationError::AttestationFailed(host_result.error))
    }
}
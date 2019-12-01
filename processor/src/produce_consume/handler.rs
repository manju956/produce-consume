// Copyright 2019 Walmart Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::super::proto::action::Action_Command;
use super::payload::ProduceConsumePayload;
use crypto::digest::Digest;
use crypto::sha2::Sha512;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
        use sabre_sdk::TransactionContext;
        use sabre_sdk::TransactionHandler;
        use sabre_sdk::TpProcessRequest;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
        use sawtooth_sdk::processor::handler::TransactionHandler;
        use sawtooth_sdk::messages::processor::TpProcessRequest;
    }
}

const PRODUCE_CONSUME: &str = "produce-consume";
const VERSION: &str = "1.0";

pub struct ProduceConsumeHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl ProduceConsumeHandler {
    pub fn new() -> ProduceConsumeHandler {
        ProduceConsumeHandler {
            family_name: PRODUCE_CONSUME.to_string(),
            family_versions: vec![VERSION.to_string()],
            namespaces: vec![get_produce_consume_prefix().to_string()],
        }
    }
}

impl TransactionHandler for ProduceConsumeHandler {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        self.namespaces.clone()
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut dyn TransactionContext,
    ) -> Result<(), ApplyError> {
        warn!("Received the payload {:?}", &request.get_payload());
        let payload = match ProduceConsumePayload::new(request.get_payload()) {
            Ok(decoded) => decoded,
            Err(err) => return Err(err),
        };

        // Compute address for the item
        let address = compute_address(&payload.get_identifier());

        // Get the quantity in the store
        let raw_value: Option<Vec<u8>> = match context.get_state_entry(&address) {
            Ok(present) => present,
            Err(err) => return Err(ApplyError::InternalError(err.to_string())),
        };
        // Deserialize the value
        let value = match raw_value {
            Some(present) => {
                let mut array: [u8; 4] = [0; 4];
                array.copy_from_slice(&present[..4]);
                i32::from_ne_bytes(array)
            }
            None => 0,
        };
        info!("Read the value {}: {}", &payload.get_identifier(), value);

        // Check for overflow scenarios
        let new_value = match payload.get_command() {
            Action_Command::PRODUCE => value.checked_add(payload.get_quantity()),
            Action_Command::CONSUME => value.checked_sub(payload.get_quantity()),
        };
        // unwrapping is safe after none condition check
        if new_value.is_none() || new_value.unwrap() < 0 {
            return Err(ApplyError::InvalidTransaction(
                "Invalid resultant quantity".to_string(),
            ));
        }
        info!(
            "Computed new value {}: {}",
            &payload.get_identifier(),
            new_value.unwrap()
        );

        // Either produce or consume successful, store the new state back, serialize the value
        let new_value_bytes = new_value.unwrap().to_ne_bytes();

        context.set_state_entries(vec![(address, new_value_bytes.to_vec())])?;

        Ok(())
    }
}

fn compute_address(identifier: &str) -> String {
    let prefix = get_produce_consume_prefix();
    let mut sha = Sha512::new();
    sha.input_str(identifier);
    let remaining = sha.result_str()[..64].to_string();
    prefix + &remaining
}

fn get_produce_consume_prefix() -> String {
    let mut sha = Sha512::new();
    sha.input_str(PRODUCE_CONSUME);
    sha.result_str()[..6].to_string()
}

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

use super::super::proto::action::Action;
use super::super::proto::action::Action_Command;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
    }
}

#[derive(Debug)]
pub(crate) struct ProduceConsumePayload {
    command: Action_Command,
    identifier: String,
    quantity: i32,
}

impl ProduceConsumePayload {
    pub(crate) fn new(raw_bytes: &[u8]) -> Result<ProduceConsumePayload, ApplyError> {
        warn!("Payload in raw is {:?}", &raw_bytes);
        let parsed_payload: Action = match parse_from(raw_bytes) {
            Ok(result) => result,
            Err(e) => return Err(e),
        };
        Ok(ProduceConsumePayload {
            command: parsed_payload.get_command(),
            identifier: parsed_payload.get_identifier().to_string(),
            quantity: parsed_payload.get_quantity(),
        })
    }

    pub(crate) fn get_command(&self) -> Action_Command {
        return self.command;
    }

    pub(crate) fn get_identifier(&self) -> String {
        return self.identifier.clone();
    }

    pub(crate) fn get_quantity(&self) -> i32 {
        return self.quantity;
    }
}

fn parse_from<T>(data: &[u8]) -> Result<T, ApplyError>
where
    T: protobuf::Message,
{
    protobuf::parse_from_bytes(data).map_err(|err| {
        warn!("Invalid error: Failed to parse the payload: {:?}", err);
        ApplyError::InvalidTransaction(format!("Failed to unmarshal payload: {:?}", err))
    })
}

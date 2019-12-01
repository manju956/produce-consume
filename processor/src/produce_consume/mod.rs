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

pub mod handler;
pub(crate) mod payload;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::{WasmPtr, execute_entrypoint};
        use sabre_sdk::ApplyError;
        use sabre_sdk::TransactionContext;
        use sabre_sdk::TransactionHandler;
        use sabre_sdk::TpProcessRequest;
        use handler::ProduceConsumeHandler;
    }
}

/// The business logic part of the produce-consume application.
/// Inputs:
/// ```TpProcessRequest``` client's request.
/// ```TrnsactionContext``` given by the underlying blockchain.
///
/// Outputs:
/// ```bool``` Result of the execution.
/// ```ApplyError``` The result InternalError or InvalidTransaction
/// given to the underlying blockchain.
///
/// This method is public only within the crate.
#[cfg(target_arch = "wasm32")]
pub fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = ProduceConsumeHandler::new();
    match handler.apply(request, context) {
        Ok(_) => Ok(true),
        Err(err) => {
            info!("{}", err);
            Err(err)
        }
    }
}

// No mangle is for the no compiler optimization
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe fn entrypoint(payload: WasmPtr, signer: WasmPtr, signature: WasmPtr) -> i32 {
    // Implement the apply method, this is the core business logic part of the Sabre
    execute_entrypoint(payload, signer, signature, apply)
}

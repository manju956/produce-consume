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

use crate::produce_consume::PRODUCE_CONSUME;
use crate::produce_consume::VERSION;
use std::iter::Iterator;
use sawtooth_sdk::{
    messages::{
        batch::{Batch, BatchHeader, BatchList},
        transaction::{Transaction, TransactionHeader},
    },
    signing::{PublicKey, Signer},
};
use crypto::sha2::Sha512;
use crypto::digest::Digest;
use protobuf::{Message, RepeatedField};

/// Function to create the ```BatchList``` object, which later is serialized and sent to REST API
/// Accepts ```Batch``` as a input parameter.
pub(crate) fn create_batch_list(batches: Vec<Batch>) -> BatchList {
    // Construct batch list
    let batches = RepeatedField::from_vec(batches);
    let mut batch_list = BatchList::new();
    batch_list.set_batches(batches);
    batch_list
}

/// Function to create the ```Batch``` object, this is then added to ```BatchList```. Accepts
/// signer object and ```Transaction``` as input parameters. Constructs ```BatchHeader``` , adds
/// signature of it to ```Batch```.
pub(crate) fn create_batch(signer: &Signer, transaction: Transaction) -> Batch {
    // Construct BatchHeader
    let mut batch_header = BatchHeader::new();
    // set signer public key
    let public_key = signer
        .get_public_key()
        .expect("Unable to get public key")
        .as_hex();
    let transaction_ids = vec![transaction.clone()]
        .iter()
        .map(|trans| String::from(trans.get_header_signature()))
        .collect();
    batch_header.set_transaction_ids(RepeatedField::from_vec(transaction_ids));
    batch_header.set_signer_public_key(public_key);

    // Construct Batch
    let batch_header_bytes = batch_header
        .write_to_bytes()
        .expect("Error converting batch header to bytes");
    let signature = signer
        .sign(&batch_header_bytes)
        .expect("Error signing the batch header");
    let mut batch = Batch::new();
    batch.set_header_signature(signature);
    batch.set_header(batch_header_bytes);
    batch.set_transactions(RepeatedField::from_vec(vec![transaction]));
    batch
}

/// Function to create ```Transaction``` object, accepts payload, ```TransactionHeader``` and
/// ```Signer```.
pub(crate) fn create_transaction(
    signer: &Signer,
    transaction_header: &TransactionHeader,
    payload: Vec<u8>,
) -> Transaction {
    // Construct a transaction, it has transaction header, signature and payload
    let transaction_header_bytes = transaction_header
        .write_to_bytes()
        .expect("Error converting transaction header to bytes");
    let transaction_header_signature = signer
        .sign(&transaction_header_bytes.to_vec())
        .expect("Error signing the transaction header");
    let mut transaction = Transaction::new();
    transaction.set_header(transaction_header_bytes.to_vec());
    transaction.set_header_signature(transaction_header_signature);
    transaction.set_payload(payload);
    transaction
}

/// Function to construct ```TransactionHeader``` object, accepts parameters required such as
/// input and output addresses, payload, public key of transactor, nonce to be used.
pub(crate) fn create_transaction_header(
    input_addresses: &[String],
    output_addresses: &[String],
    payload: &[u8],
    public_key: &Box<dyn PublicKey>,
    nonce: String,
) -> TransactionHeader {
    // Construct transaction header
    let mut transaction_header = TransactionHeader::new();
    transaction_header.set_family_name(PRODUCE_CONSUME.to_string());
    transaction_header.set_family_version(VERSION.to_string());
    transaction_header.set_nonce(nonce);
    transaction_header.set_payload_sha512(sha512_of_bytes(payload));
    transaction_header.set_signer_public_key(public_key.as_hex());
    transaction_header.set_batcher_public_key(public_key.as_hex());
    transaction_header.set_inputs(RepeatedField::from_vec(input_addresses.to_vec()));
    transaction_header.set_outputs(RepeatedField::from_vec(output_addresses.to_vec()));
    transaction_header.clear_dependencies();
    transaction_header
}

fn sha512_of_bytes(bytes: &[u8]) -> String {
    let mut sha = Sha512::new();
    sha.input(bytes);
    sha.result_str()
}

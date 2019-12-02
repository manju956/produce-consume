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

use crate::cli_error::CliError;
use crate::network_helper;
use crate::proto::action::Action;
use crate::proto::action::Action_Command;
use crate::sawtooth_helper;
use crypto::digest::Digest;
use crypto::sha2::Sha512;
use hex;
use protobuf::Message;
use rand::Rng;
use sawtooth_sdk::signing::{create_context, secp256k1::Secp256k1PrivateKey, PrivateKey, Signer};
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;

pub(crate) const PRODUCE_CONSUME: &str = "produce-consume";
pub(crate) const VERSION: &str = "1.0";

pub(crate) fn submit_payload(
    command: &str,
    identifier: &str,
    quantity: &str,
    url: Option<&str>,
    key: &str,
) -> Result<(), CliError> {
    let cmd: Action_Command = if command == "PRODUCE" {
        Action_Command::PRODUCE
    } else if command == "CONSUME" {
        Action_Command::CONSUME
    } else {
        panic!("Unexpected scenario");
    };

    let address = compute_address(identifier);

    let qty: i32 = match quantity.parse() {
        Ok(value) => value,
        Err(err) => return Err(CliError::from(err.to_string())),
    };

    let mut action: Action = Action::new();
    println!("Command is {:?}", cmd.clone());
    action.set_command(cmd);
    action.set_identifier(identifier.to_string());
    action.set_quantity(qty);
    let payload = action
        .write_to_bytes()
        .expect("Couldn't create a command to send to the validator");

    println!("Payload in raw is {:?}", payload.to_vec());
    let parsed_payload: Action = parse_from(&payload).expect("Cannot");
    println!("Payload is {:?} {:?} {:?}", parsed_payload.get_command(), parsed_payload.get_identifier(), parsed_payload.get_quantity());

    if url.is_none() {
        save_to_file(&payload);
        return Ok(());
    }

    let read_key = read_file(key);
    let private_key: Box<dyn PrivateKey> =
        Box::new(Secp256k1PrivateKey::from_hex(&read_key).expect("Unable to load context"));
    let context = create_context("secp256k1").expect("Unable to create a secp256k1 context");
    let signer = Signer::new(context.as_ref(), private_key.as_ref());
    // get signer and public key from signer in hex
    let public_key = signer.get_public_key().expect("Unable to get public key");

    let output_addresses = [address.clone()];
    let input_addresses = [address.clone()];

    let nonce_bytes = rand::thread_rng()
        .gen_iter::<u8>()
        .take(64)
        .collect::<Vec<u8>>();
    let nonce = to_hex_string(&nonce_bytes);

    // Create transaction header
    let transaction_header = sawtooth_helper::create_transaction_header(
        &input_addresses,
        &output_addresses,
        &payload,
        &public_key,
        nonce.to_string(),
    );
    // Create transaction
    let transaction =
        sawtooth_helper::create_transaction(&signer, &transaction_header, payload.to_vec());
    // Create batch header, batch
    let batch = sawtooth_helper::create_batch(&signer, transaction);
    let batches = vec![batch];
    let batch_list = sawtooth_helper::create_batch_list(batches);

    let raw_bytes = batch_list
        .write_to_bytes()
        .expect("Unable to write batch list as bytes");

    if url.is_some() {
        return network_helper::submit_to_rest_api(url.unwrap(), "batches", &raw_bytes);
    }
    Ok(())
}

fn parse_from<T>(data: &[u8]) -> Result<T, ()>
    where
        T: protobuf::Message,
{
    protobuf::parse_from_bytes(&data).map_err(|err| {
        println!("Invalid error: Failed to parse the payload: {:?}", err);
    })
}

/// Saves the byte stream to a file
pub fn save_to_file(bytes: &[u8]) {
    let mut current_working_directory =
        env::current_dir().expect("Error reading current working directory");
    current_working_directory.push("default.batch");
    let file_path = current_working_directory.as_path();
    write_binary_file(&bytes, file_path.to_str().expect("Unexpected filename"))
}

/// Write binary data to a file
pub fn write_binary_file(data: &[u8], filename: &str) {
    let mut file = File::create(filename).expect("File not found");
    file.write_all(data).expect("Write binary file failed");
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

/// Reads the given file as string
///
/// Note: This method will panic if file is not found or error occurs when reading file as string.
pub fn read_file_as_string(filename: &str) -> String {
    let mut file_handler = match File::open(filename) {
        Ok(file_open_successful) => file_open_successful,
        Err(error) => panic!("Error opening file! {} : {}", error, filename),
    };
    let mut read_contents = String::new();
    file_handler
        .read_to_string(&mut read_contents)
        .expect("Read operation failed");
    read_contents
}

/// Reads the given file as string, ignore the new line character at end
///
/// Note: This method will panic if file is not found or error occurs when reading file as string.
pub fn read_file(filename: &str) -> String {
    let mut read_contents = read_file_as_string(filename);
    read_contents.pop();
    read_contents
}

pub fn to_hex_string(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

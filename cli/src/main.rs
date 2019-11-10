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

extern crate clap;
extern crate futures;
extern crate hex;
extern crate hyper;
extern crate rand;
extern crate sawtooth_sdk;
extern crate tokio;

mod cli_error;
mod network_helper;
mod produce_consume;
mod proto;
mod sawtooth_helper;

use clap::App;
use clap::Arg;

fn main() {
    let matches = App::new("pc-cli")
        .author("Walmart Inc.")
        .version("1.0")
        .about("Sample sawtooth-sabre smart contract produce-consume cli")
        .arg(
            Arg::with_name("command")
                .short("C")
                .long("command")
                .help("Command either PRODUCE or CONSUME")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("identifier")
                .short("I")
                .long("identifier")
                .help("Identifier of the produced item")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("quantity")
                .short("Q")
                .long("quantity")
                .help("Quantity of the produced item")
                .takes_value(true)
                .required(true),
        )
        // Optional arguments, for which the default values are used
        .arg(
            Arg::with_name("url")
                .short("U")
                .long("url")
                .help("URL of the validator to send the request")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("key")
                .short("K")
                .long("key")
                .help("Key used for signing the transaction")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    // This is a CLI application, an irrecoverable error occurs if the input is not good
    let command = matches.value_of("command").unwrap();
    let identifier = matches.value_of("identifier").unwrap();
    let quantity = matches.value_of("quantity").unwrap();
    let url = matches.value_of("url");
    let key = matches
        .value_of("key")
        .unwrap_or("/etc/sawtooth/keys/validator.priv");

    match produce_consume::submit_payload(command, identifier, quantity, url, key) {
        Ok(_) => println!("Successfully submitted the transaction"),
        Err(err) => {
            println!("Unable to submit the transaction {}", err);
            std::process::exit(1);
        }
    }
}

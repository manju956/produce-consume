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
use futures::{future, future::Future, stream::Stream};
use hyper::{client::ResponseFuture, header::HeaderMap, Error, StatusCode};
use hyper::{header, header::HeaderValue, Body, Client, Method, Request, Uri};
use std::{error, fmt};
use tokio::runtime::current_thread::Runtime;

/// Custom error for client utils
#[derive(Debug, Clone)]
struct ClientError;

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid value found")
    }
}

impl error::Error for ClientError {
    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

/// read_response_future() would return the ClientResponse which has ```hyper::Body``` and
/// ```hyper::header::HeaderMap```
///
/// Methods to read Body does consume, ideally we should access members through getters
#[derive(Debug)]
struct ClientResponse {
    pub body: Body,
    pub header_map: HeaderMap,
}

/// Sends the raw_bytes to the REST API
pub(crate) fn submit_to_rest_api(url: &str, api: &str, raw_bytes: &[u8]) -> Result<(), CliError> {
    let body_length = raw_bytes.len();
    let bytes = Body::from(raw_bytes.to_vec());

    // API to call
    let mut rest_api = String::new();
    rest_api.push_str(url);
    rest_api.push_str("/");
    rest_api.push_str(api);
    let uri = rest_api.parse::<Uri>().expect("Error constructing URI");

    // Construct client to send request
    let client = Client::new();

    // Compose POST request, to register
    let mut request = Request::new(bytes);
    *request.method_mut() = Method::POST;
    *request.uri_mut() = uri;
    request.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    request
        .headers_mut()
        .insert(header::CONTENT_LENGTH, HeaderValue::from(body_length));

    // Call read_response_future to block on reading the response
    let response_future = client.request(request);
    match read_response_future(response_future) {
        Ok(response) => {
            let body = read_body_as_string(response.body).expect("Unable to read body as string");
            println!("Received Response from the REST API {}", body);
        }
        Err(err) => return Err(CliError::from(err.to_string())),
    };
    Ok(())
}

/// Function to read ```hyper::client::ResponseFuture``` (return values of .request(), .get(), .post()
/// etc functions from hyper library).
///
/// Returns result ClientResponse and ClientError.
/// This is a blocking call. A ```tokio_core``` runner instance is created to block until
/// ```ResponseFuture``` is complete.
fn read_response_future(response_fut: ResponseFuture) -> Result<ClientResponse, ClientError> {
    let future_response = response_fut
        // 'then' waits for future_response to be ready and calls the closure supplied here on
        // Result of evaluated future. Response object is ready when closure is called.
        .then(move |response_obj| {
            match response_obj {
                Ok(response) => {
                    println!("Received response result code: {}", response.status());
                    if response.status() >= StatusCode::BAD_REQUEST {
                        println!("Response status is not successful: {}", response.status());
                        return Err(ClientError);
                    }
                    // Borrow response headers, to be passed in ClientResponse
                    let header_map = response.headers().to_owned();
                    let body = response.into_body();
                    let client_response = ClientResponse { body, header_map };
                    Ok(client_response)
                }
                Err(error) => {
                    println!(
                        "Error occurred while waiting for the ResponseFuture {}",
                        error
                    );
                    Err(ClientError)
                }
            }
        });

    // Create a runner instance for evaluating ResponseFuture
    let mut runner = Runtime::new().expect("Error creating runtime");
    // blocks until future is evaluated, otherwise error out
    match runner.block_on(future_response) {
        Ok(successful) => Ok(successful),
        Err(_) => Err(ClientError),
    }
}

/// Function to read ```hyper::Body``` (body) as string.
///
/// Returns result of ```String``` and ```ClientError```.
/// This is a blocking call. Body is streamed and collected as vector, which later is converted to
/// string representation.
fn read_body_as_string(body: Body) -> Result<String, ClientError> {
    body.fold(Vec::new(), |mut vector, chunk| {
        vector.extend_from_slice(&chunk[..]);
        future::ok::<_, Error>(vector)
    })
    // 'then' evaluates Future to Result. Note that body should be available already.
    // Construct a Result of string to be returned when body is available.
    .then(move |body_as_byte_vector| match body_as_byte_vector {
        Ok(byte_vector) => {
            let body =
                String::from_utf8(byte_vector).expect("Error reading body byte stream as string");
            Ok(body)
        }
        Err(error) => {
            println!("Error reading body as string {}", error);
            Err(ClientError)
        }
    })
    // Wait for completion of task assigned to then
    .wait()
}

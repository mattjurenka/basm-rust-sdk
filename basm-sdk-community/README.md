# BASM Sdk Community

This crate is a community maintained sdk for the Blocky Attestation Service developed by [https://www.blocky.rocks/](https://www.blocky.rocks/)

## Example
```
use basm_sdk_community::{
    http::send_http_request,
    prelude::*
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputJson {
    error: String,
    data: u64
}

#[bky_entrypoint]
pub fn entrypoint(ctx: Context<String, String>) -> OutputJson {
    let result = send_http_request(
        "GET".into(),
        "https://example.com/api/v2/test".into(),
        &BTreeMap::from([
            ("Content-Type".into(), vec!["application/json".into()]),
        ]),
        &[]
    );

    // Printing http result to log outputted by blocky service
    log!(
        "HTTP Request Result: {:?}",
        result
    );

    let rand_number = rand::random::<u64>();
    // Printing random number to 
    host_log!(
        "Printing a random number: {}",
        rand_number
    );

    OutputJson {
        error: String::new(),
        data: rand_number
    }
}
```

An example crate that consumes this sdk can be found [here](https://github.com/mattjurenka/basm-sdk-community/tree/master/integration-test).

## Build Environment
This crate is meant to be used with the `wasm32-wasip1` build target on release profile (`--release`). When using this library, it is ok to use std, however things that the Blocky service doesn't provide like File IO won't work.

Please look in the [integration test example repo](https://github.com/mattjurenka/basm-sdk-community/tree/master/integration-test) for more information on how to setup a crate to work properly. In addition the `.cargo/config.toml` is important to set additional linker settings that are necessary for the wasm executable to run on the Blocky service.

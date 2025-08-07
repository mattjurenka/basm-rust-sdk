use basm_rust_sdk::{
    host_log, io::{output_data, HostWriter, LogWriter}, log, memory::FatPointer
};

use serde::{Serialize, Deserialize};
use std::io::Write;

pub struct Context<I=(), S=()> {
    input: I,
    secrets: S
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputJson {
    x: String,
    y: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SecretJson {
    privkey: String,
    api_key: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputJson {
    error: String,
    data: u64
}

const ENCLAVE_ATTESTED_PUBKEY: &str = "eyJwbGF0Zm9ybSI6InBsYWluIiwicGxhdGZvcm1fYXR0ZXN0YXRpb25zIjpbImV5SmtZWFJoSWpvaVpYbEthbVJZU2pKYVZqa3daVmhDYkVscWIybGpSRWt4VG0xemVFbHBkMmxhUjBZd1dWTkpOa2xyU1QwaUxDSnRaV0Z6ZFhKbGJXVnVkQ0k2ZXlKd2JHRjBabTl5YlNJNkluQnNZV2x1SWl3aVkyOWtaU0k2SW5Cc1lXbHVJbjE5IiwiZXlKa1lYUmhJam9pVkZoYVdsVldaRVpYYldSRVZHNXJNVkpxYkUxTldFSm9UbXhLUzJKNlNtMWtia1p1WVdwc2FtRllaejBpTENKdFpXRnpkWEpsYldWdWRDSTZleUp3YkdGMFptOXliU0k2SW5Cc1lXbHVJaXdpWTI5a1pTSTZJbkJzWVdsdUluMTkiLCJleUprWVhSaElqb2lXbTVhWVdWdVdrWlJibkJ0VmpKS1RHSnFRbXBWUTNSd1UxUk9ORk5GZEZsVlZrNHhVV3QwVW1KSGF6MGlMQ0p0WldGemRYSmxiV1Z1ZENJNmV5SndiR0YwWm05eWJTSTZJbkJzWVdsdUlpd2lZMjlrWlNJNkluQnNZV2x1SW4xOSIsImV5SmtZWFJoSWpvaVZrVXhhMVV5VGxoYU0wcDZTM3BXTVdNeGJ6Qk1lbFpYVTFaS1VWUlVNR2xtVVQwOUlpd2liV1ZoYzNWeVpXMWxiblFpT25zaWNHeGhkR1p2Y20waU9pSndiR0ZwYmlJc0ltTnZaR1VpT2lKd2JHRnBiaUo5ZlE9PSIsImV5SmtZWFJoSWpvaVNsRTVXUzlQWVM5Uk4zSjFTVkYzWXpsalJHUnBjMDVLZDJWTE4wbG1WV0pHUjNCU09DdFNXV3BTYXowaUxDSnRaV0Z6ZFhKbGJXVnVkQ0k2ZXlKd2JHRjBabTl5YlNJNkluQnNZV2x1SWl3aVkyOWtaU0k2SW5Cc1lXbHVJbjE5Il19";
const TRANSITIVE_CLAIMS: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA6AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAUAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAoAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAzZjQzMjBiNzY4ZDAzMmQ4MTkzMDMxYTBkYWE3NTBkMzEwNzUzNjhhMDY3YTJkNTlkZGQ3ZjY5MWIxMzVlMjhiNGUxYzVhZTRmZTlmMDRkMGZmMDE1NTgyOTkyMjljMmNjZGE3MzI0OGYzZWU0Y2FjYTAwYzZkNzNiNjJmZDlkYwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAALaGVsbG9fd29ybGQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgGYzNDg1ZDUzZDk5NDYyMDVlMTJiNzAxNzVmNmU5ODE2NmRmYjM0MjJhNmQ4ZTg0NGEwYTJjMzM5Njk2NjJkZDVjNjAyYzEzY2U5Y2Y4M2ViYzViOTk0YTdiNDkyNjg2MzNhZTk1MmNhMjEwYjhjMGI4ZTRmZjUwMDkzZWE5YTEzAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABFPdXRwdXQgb2YgdGhlIGZuIQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAYzY0NjAyMWQ1YTA0YTExZDIwZmU5YzE0NDI2ZjUwNDQ2NDQyZDg5ZjFiNzY1MThiYWQyYjdjMDc2YWQ2Y2Y0MTUyY2I5ZWYwOWZhMDhkZWU3OGY4MjNlMWFlMGNmMTM5ZWNlZGFiYzg0ZTVkOGRhMTYxY2ZmOGNmM2E0ZjFiYTUAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQa7SRriHXqlUwXl5B+Lft3O33gT644gq5dxD+vf4pJSxD6aq9nqASk0KTiDKs4UmRFHp/BXYSD2JpIxeomRb0dIBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";

// TODO:
// add testing for everything
// write docs
// write readme
// change name to community
// record video
// write bky_function macro
// add workspace commands

#[no_mangle]
pub extern "C" fn hello_world(input_ptr: FatPointer, secret_ptr: FatPointer) -> FatPointer {
//#[bky_function]
//pub fn hello_world(ctx: Context<InputJson, SecretJson>) -> OutputJson {
    log!(
        "Formatted Log Output, {}",
        32
    );
    log!(
        "Logging input {}",
        str::from_utf8(&input_ptr.copy_data()).unwrap()
        //serde_json::to_string(&ctx.input).unwrap()
    );

    //let result = send_http_request(
    //    "GET".into(),
    //    "https://dogapi.dog/api/v2/breeds".into(),
    //    &HashMap::from([
    //        ("Content-Type".into(), vec!["application/json".into()]),
    //        ("User-Agent".into(), vec!["Basm Rust SDK Test Client".into()])
    //    ]),
    //    &[]
    //);
    //host_log!(
    //    "HTTP Request Result: {:?}",
    //    result
    //);

    //let attestation_result = verify_attestation(
    //    ENCLAVE_ATTESTED_PUBKEY.into(),
    //    TRANSITIVE_CLAIMS.into(),
    //    Vec::from([
    //        EnclaveMeasurement {
    //            platform: "plain".into(),
    //            code: "plain".into()
    //        },
    //    ])
    //);
    //host_log!(
    //    "Attestation Result {:?}",
    //    attestation_result
    //);

    host_log!(
        "Printing secrets for debug: {}",
        str::from_utf8(&secret_ptr.copy_data()).unwrap()
        //serde_json::to_string(&ctx.secrets).unwrap()
    );
    
    host_log!(
        "Printing a random number: {}",
        rand::random::<u64>()
    );
    host_log!(
        "Printing a timestamp: {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    return output_data("Output of the fn!".as_bytes());
    //OutputJson {
        //error: String::from(""), data: 256
    //}
}

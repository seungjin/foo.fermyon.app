mod bindings;

use anyhow::Result;
use bindings::exports::seungjin::foo::fortune;

use wasi::http::outgoing_handler::handle;
use wasi::http::outgoing_handler::OutgoingRequest;
use wasi::http::types::ErrorCode;
use wasi::http::types::{Fields, Method, Scheme};
use wasi::io::poll;

struct Component;

impl fortune::Guest for Component {
    fn say() -> Result<String, u32> {
        let fields = Fields::new();
        let req = OutgoingRequest::new(fields);
        req.set_method(&Method::Get).unwrap();
        req.set_scheme(Some(&Scheme::Http)).unwrap();
        req.set_authority(Some("www.yerkee.com")).unwrap();
        req.set_path_with_query(Some("/api/fortune")).unwrap();
        let res = handle(req, None).unwrap();
        let pollable = res.subscribe();
        poll::poll(&[&pollable]);

        let incoming_response = res.get().unwrap().unwrap().unwrap();
        match incoming_response.status() {
            200 => {
                let a = incoming_response.consume().unwrap();
                let b = a.stream().unwrap();

                let a = incoming_response
                    .headers()
                    .get(&"content-length".to_string());

                let joined: Vec<u8> =
                    a.iter().flat_map(|v| v.iter().cloned()).collect();

                let content_length =
                    String::from_utf8(joined).unwrap().parse::<u64>().unwrap();

                let content =
                    String::from_utf8(b.read(content_length).unwrap())
                        .unwrap();

                Ok(content)
            }
            404 => return Err(404),
            500 => return Err(500),
            _ => return Err(0),
        }
    }
}

bindings::export!(Component with_types_in bindings);

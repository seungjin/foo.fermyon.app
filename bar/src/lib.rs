use anyhow::Result;
use anyhow::bail;
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;
use multipart::server::Multipart;
use std::str;
use substring::Substring;
use std::io::BufRead;
use std::fs::File;
use std::io::prelude::*;

/// A simple Spin HTTP component.
#[http_component]
async fn handle_bar(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));

    let headers = req
        .headers()
        .map(|(k, v)| (k.to_string(), v.as_str()))
        .collect::<Vec<_>>();

    //println!("{headers:?}");

    let a = req.header("content-type").unwrap().as_str().unwrap();
    //println!("{a}");

    let boundary = get_multipart_boundary(&req).unwrap(); // TODO: Match its error
    println!("{boundary}");
    
    let body = req.body();

    let mut mp = Multipart::with_body(
        body,
        boundary,
    );
    
    while let Some(mut field) = mp.read_entry().unwrap() {
        let data = field.data.fill_buf().unwrap();
        let s = String::from_utf8_lossy(data);
        
        //println!("headers: {:?}, data: {}", field.headers, s);
        match field.headers.filename {
            Some(x) => { write_to_file(x, data)? },
            None => {  },
        }

    }


    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Hello, Fermyon")
        .build())
}


pub fn get_multipart_boundary(req: &Request) -> Result<String> {
    let boundary = req.header("content-type").unwrap();
    let a = boundary.as_str().unwrap();
    if a.substring(0,30) == "multipart/form-data; boundary=" {
        let mut split = a.split("boundary=").collect::<Vec<&str>>();
        let a = split[1];
        return Ok(a.to_string());
    } 
    bail!("Can't find boundary from header")
    
}

// spin watch --direct-mounts --allow-transient-write
// Write to file to check it is getting it right ...  
pub fn write_to_file(file_name: String, data: &[u8]) -> Result<()>{
    let mut file = File::create(file_name)?;
    file.write_all(data)?;
    Ok(())

}

pub fn send_to_s3() {

}
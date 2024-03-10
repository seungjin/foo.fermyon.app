use anyhow::bail;
use anyhow::Result;
use chrono::{DateTime, Utc};
use hmac::{Mac, SimpleHmac};
use multipart_2021::server::Multipart;
use sha2::{Digest, Sha256};
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;
use std::fs::File;
use std::io::prelude::*;
use std::str;
use substring::Substring;

/// A simple Spin HTTP component.
#[http_component]
async fn handle_bar(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));

    let headers = req
        .headers()
        .map(|(k, v)| (k.to_string(), v.as_str()))
        .collect::<Vec<_>>();

    //println!("{headers:?}");

    //let a = req.header("content-type").unwrap().as_str().unwrap();
    //println!("{a}");

    let boundary = get_multipart_boundary(&req).unwrap(); // TODO: Match its error

    let body = req.body();

    let l = body.len();
    println!("body size: {}", l);

    let mut mp: Multipart<&[u8]> = Multipart::with_body(body, boundary);

    while let Some(field) = mp.read_entry().unwrap() {
        //let s = String::from_utf8_lossy(data);
        let data: Result<Vec<u8>, std::io::Error> = field.data.bytes().collect();
        //let size_of_data = data.unwrap().len();
        let a = data.unwrap();
        let b: &[u8] = &a;

        //println!("headers: {:?}, data: {}", field.headers, size_of_data);
        match field.headers.filename {
            Some(x) => send_to_s3(x, b).await?,
            None => {}
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
    if a.substring(0, 30) == "multipart/form-data; boundary=" {
        let split = a.split("boundary=").collect::<Vec<&str>>();
        let a = split[1];
        return Ok(a.to_string());
    }
    bail!("Can't find boundary from header")
}

// spin watch --direct-mounts --allow-transient-write
// Write to file to check it is getting it right ...
pub fn write_to_file(file_name: String, data: &[u8]) -> Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(data)?;
    Ok(())
}

/*

curl --location --request PUT 'https://cnbbb4fp6bwv.compat.objectstorage.ap-seoul-1.oraclecloud.com/tmp/1x1black.png' \
--header 'Content-Type: image/png' \
--header 'X-Amz-Content-Sha256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855' \
--header 'X-Amz-Date: 20240304T065016Z' \
--header 'Authorization: AWS4-HMAC-SHA256 Credential=433f5f8a64597b8aaf3c3289963e08b45631503d/20240304/ap-seoul-1/s3/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-content-sha256;x-amz-date, Signature=97f2b4fb213899602ac9d5f898d76675d2c717ce92ecd8b0e36afabd6dbc2683' \
--data '@/var/home/seungjin/Downloads/1x1black.png'

*/

// https://docs.aws.amazon.com/IAM/latest/UserGuide/create-signed-request.html#create-string-to-sign
// https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html

pub async fn send_to_s3(file_name: String, file: &[u8]) -> Result<()> {
    //let host = "s3.amazonaws.com";
    //let bucket = "seungjin";
    //let target = format!("https://{bucket}.{host}/{file_name}");
    //let region = "us-east-1".to_string();

    let host = "cnbbb4fp6bwv.compat.objectstorage.ap-seoul-1.oraclecloud.com";
    let bucket = "tmp";
    let target = format!("https://{host}/{bucket}/{file_name}");
    let region = "ap-seoul-1".to_string();

    let access_key = std::env::var("S3_ACCESS_KEY").unwrap();
    let secret_key = std::env::var("S3_SECRET_KEY").unwrap();

    let content_type = "image/png";
    let content_length = file.len().to_string();
    println!("--- content length : {}", content_length);

    let service = "s3".to_string();

    let (x_amz_date, ddate) = get_x_amz_date().await;
    let yyyymmdd = x_amz_date.substring(0, 8).to_string();

    /*example from AWS doc
    let host = "s3.amazonaws.com";
    let bucket = "examplebucket";
    let target = format!("https://{bucket}.{host}/{file_name}");
    let access_key = "AKIAIOSFODNN7EXAMPLE";
    let secret_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
    let x_amz_date = "20130524T000000Z";
    let yyyymmdd = x_amz_date.substring(0, 8).to_string();
    let ddate = "Fri, 24 May 2013 00:00:00 GMT";
    */

    let x_amz_content_sha256 = sha256hash_hex_encoded(file).await.unwrap();
    let xxx = x_amz_content_sha256.clone();

    let http_method = "PUT";
    let url_encoded_file_name = urlencoding::encode(file_name.as_str());
    //let canonical_uri = format!("/{url_encoded_file_name}");
    let canonical_uri = format!("/{bucket}/{url_encoded_file_name}");
    let canonical_query_string = "";
    //let canonical_headers = format!("date:{ddate}\nhost:{bucket}.{host}:443\nx-amz-content-sha256:{xxx}\nx-amz-date:{x_amz_date}\n");
    let canonical_headers = format!(
        "date:{ddate}\nhost:{host}:443\nx-amz-content-sha256:{xxx}\nx-amz-date:{x_amz_date}\n"
    );
    let signed_headers = "date;host;x-amz-content-sha256;x-amz-date";
    let hashed_paylod = x_amz_content_sha256.clone();
    let canonical_request = format!(
        "{http_method}\n{canonical_uri}\n\n{canonical_headers}\n{signed_headers}\n{hashed_paylod}"
    );

    println!("\n\ncanonical_request:\n{}\n\n", canonical_request);

    let sha256_of_canonical_request = sha256hash_hex_encoded(canonical_request.as_bytes())
        .await
        .unwrap();
    println!(
        "sha256_of_canonical_request: {}",
        sha256_of_canonical_request,
    );

    // Calc the signature
    /*
    DateKey = HMAC-SHA256("AWS4"+"<SecretAccessKey>", "<YYYYMMDD>")
    DateRegionKey = HMAC-SHA256(<DateKey>, "<aws-region>")
    DateRegionServiceKey = HMAC-SHA256(<DateRegionKey>, "<aws-service>")
    SigningKey = HMAC-SHA256(<DateRegionServiceKey>, "aws4_request")
    */

    let date_key = hmac_sha256(
        format!("AWS4{secret_key}").into_bytes(),
        yyyymmdd.clone().into_bytes(),
    )
    .await
    .unwrap();
    let date_region_key = hmac_sha256(date_key, region.clone().into_bytes())
        .await
        .unwrap();
    let date_region_service_key = hmac_sha256(date_region_key, service.clone().into_bytes())
        .await
        .unwrap();
    let signing_key = hmac_sha256(
        date_region_service_key,
        "aws4_request".to_string().into_bytes(),
    )
    .await
    .unwrap();

    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{x_amz_date}\n{yyyymmdd}/{region}/{service}/aws4_request\n{sha256_of_canonical_request}"
    );

    let signature = hmac_sha256(signing_key, string_to_sign.clone().into_bytes())
        .await
        .unwrap();

    let signature_string = hex_encoded(signature).await.unwrap();

    println!("string_to_sign: {string_to_sign}");
    println!("signature: {}", signature_string);
    println!("-------------------\n\n\n\n");

    let authorization = format!("AWS4-HMAC-SHA256 Credential={access_key}/{yyyymmdd}/{region}/s3/aws4_request,SignedHeaders=date;host;x-amz-content-sha256;x-amz-date,Signature={signature_string}");

    // AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request,SignedHeaders=host;range;x-amz-content-sha256;x-amz-date,Signature=f0e8bdb87c964420e857bd35b5d6ed310bd44f0170aba48dd91039c6036bdb41
    // AWS4-HMAC-SHA256 Credential=433f5f8a64597b8aaf3c3289963e08b45631503d/20240304/ap-seoul-1/s3/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-content-sha256;x-amz-date, Signature=97f2b4fb213899602ac9d5f898d76675d2c717ce92ecd8b0e36afabd6dbc2683"

    println!("{authorization}");

    let request = Request::builder()
        .method(Method::Put)
        .uri(target)
        .header("Authorization", authorization)
        .header("Content-Length", content_length)
        //.header("Content-Type", "text/plain")
        .header("X-Amz-Content-Sha256", x_amz_content_sha256)
        .header("X-Amz-Date", x_amz_date)
        .header("Date", ddate)
        .body(file.to_vec())
        .build();

    let response: Response = spin_sdk::http::send(request).await?;
    let r = response.status();
    let r1 = response.body();
    let r2 = std::str::from_utf8(r1).unwrap();

    println!("{r}");
    println!("{r2}");

    Ok(())
}

pub async fn get_x_amz_date() -> (String, String) {
    let current_time: DateTime<Utc> = Utc::now();
    (
        current_time.format("%Y%m%dT%H%M%SZ").to_string(),
        // Fri, 24 May 2013 00:00:00 GMT
        current_time.format("%a, %d %b %Y %T GMT").to_string(),
    )
}

// https://www.devglan.com/online-tools/hmac-sha256-online
// HMAC() returns a byte[] and not a hex string.
// HMAC(key, data) represents an HMAC-SHA256 function that returns output in binary format.
// The result of each hash function becomes input for the next one.
// https://stackoverflow.com/questions/67656612/how-to-compute-hmac-sha-256-for-aws-authentication
pub async fn hmac_sha256(key: Vec<u8>, text: Vec<u8>) -> Result<Vec<u8>> {
    type HmacSha256 = SimpleHmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(&key).expect("Error from hmac_sha256");
    mac.update(&text);
    let f = mac.finalize();
    let f2 = f.into_bytes().to_vec();
    Ok(f2)
}

pub async fn sha256hash_hex_encoded(data: &[u8]) -> Result<String> {
    let mut hasher = Sha256::new();
    //hasher.update(data);
    hasher.update(data);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

pub async fn hex_encoded(a: Vec<u8>) -> Result<String> {
    Ok(hex::encode(a))
}

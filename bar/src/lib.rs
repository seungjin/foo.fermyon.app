use anyhow::bail;
use anyhow::Result;
use chrono::{DateTime, Utc};
use hex::encode;
use hmac::{Mac, SimpleHmac};
use multipart::server::Multipart;
use sha2::{Digest, Sha256};
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufRead;
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
    println!("{boundary}");

    let body = req.body();

    let l = body.len();
    println!("{l}");

    let mut mp = Multipart::with_body(body, boundary);

    while let Some(mut field) = mp.read_entry().unwrap() {
        let data = field.data.fill_buf().unwrap();
        let s = String::from_utf8_lossy(data);

        //println!("headers: {:?}, data: {}", field.headers, s);
        match field.headers.filename {
            Some(x) => send_to_s3(x, data).await?,
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
        let mut split = a.split("boundary=").collect::<Vec<&str>>();
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
    let host = "s3.amazonaws.com";
    let bucket = "seungjin";
    let target = format!("https://{bucket}.{host}/{file_name}");

    let access_key = std::env::var("S3_ACCESS_KEY").unwrap();
    let secret_key = std::env::var("S3_SECRET_KEY").unwrap();

    let content_type = "image/png";
    let content_length = file.len().to_string();

    let region = "us-east-1".to_string();
    let service = "s3".to_string();

    let x_amz_date = get_x_amz_date();
    println!("{x_amz_date}");

    let yyyymmdd = x_amz_date.substring(0, 8).to_string();
    println!("{yyyymmdd}");

    let x_amz_content_sha256 = sha256hash(file).unwrap();

    // Calc the signature
    /*
    DateKey = HMAC-SHA256("AWS4"+"<SecretAccessKey>", "<YYYYMMDD>")
    DateRegionKey = HMAC-SHA256(<DateKey>, "<aws-region>")
    DateRegionServiceKey = HMAC-SHA256(<DateRegionKey>, "<aws-service>")
    SigningKey = HMAC-SHA256(<DateRegionServiceKey>, "aws4_request")
    */

    let date_key = hmac_sha256(format!("AWS4{secret_key}"), yyyymmdd.clone()).unwrap();
    let date_region_key = hmac_sha256(date_key, region.clone()).unwrap();
    let date_region_service_key = hmac_sha256(date_region_key, service.clone()).unwrap();
    let signing_key = hmac_sha256(date_region_service_key, "aws4_request".to_string()).unwrap();

    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{x_amz_date}\n{yyyymmdd}/{region}/{service}/aws4_request\n{signing_key}"
    );

    let signature = hmac_sha256(signing_key, string_to_sign.clone()).unwrap();

    println!("string_to_sign: {string_to_sign}");
    println!("signature: {signature}");
    println!("-------------------");

    let authorization = format!("AWS4-HMAC-SHA256 Credential={access_key}/{yyyymmdd}/{region}/s3/aws4_request,SignedHeaders=host;range;x-amz-content-sha256;x-amz-date,Signature={signature}");

    // AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request,SignedHeaders=host;range;x-amz-content-sha256;x-amz-date,Signature=f0e8bdb87c964420e857bd35b5d6ed310bd44f0170aba48dd91039c6036bdb41
    // AWS4-HMAC-SHA256 Credential=433f5f8a64597b8aaf3c3289963e08b45631503d/20240304/ap-seoul-1/s3/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-content-sha256;x-amz-date, Signature=97f2b4fb213899602ac9d5f898d76675d2c717ce92ecd8b0e36afabd6dbc2683"

    println!("{authorization}");

    let request = Request::builder()
        .method(Method::Put)
        .uri(target)
        .header("Authorization", authorization)
        .header("Content-Length", content_length)
        .header("Content-Type", content_type)
        .header("X-Amz-Content-Sha256", x_amz_content_sha256)
        .header("X-Amz-Date", x_amz_date)
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

pub fn get_x_amz_date() -> String {
    let current_time: DateTime<Utc> = Utc::now();
    current_time.format("%Y%m%dT%H%M%SZ").to_string()
}

// https://www.devglan.com/online-tools/hmac-sha256-online
pub fn hmac_sha256(text: String, key: String) -> Result<String> {
    type HmacSha256 = SimpleHmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).expect("Error from hmac_sha256");
    mac.update(text.as_bytes());
    let f = mac.finalize();
    let f1 = f.into_bytes();
    let result = hex::encode(&f1);
    Ok(result)
}

pub fn sha256hash(data: &[u8]) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

// AWS4-HMAC-SHA256 Credential={{ACCESS_KEY}}/20240305/{{AWS_REGION}}/s3/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-content-sha256;x-amz-date, Signature=42a691834fe12627a7a97bb768446781f7333712814ae44796e92a48f3086cce

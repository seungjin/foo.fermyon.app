wit_bindgen::generate!({
    world: "spin-timer",
    path: "../..",
    exports: {
        world: MySpinTimer
    }
});

use anyhow::Result;
use multipart_2021::client::lazy::Multipart;
use rand::seq::SliceRandom;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use spin_sdk::http::{self, IntoResponse, Method, Request, RequestBuilder, Response};
use spin_sdk::{http_component, variables};
use std::fs;
use std::io::Read;
use tracing_subscriber::{filter::EnvFilter, FmtSubscriber};

//use fermyon::spin::variables;

#[derive(Debug, Default)]
struct Restaurant {
    name: String,
    lat: f64,
    lng: f64,
    place_id: String,
    address: String,
    rating: f64,
    pics_url: Vec<String>,
    pics_binary: Vec<Vec<u8>>,
    mstd_media_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Geopoint {
    lat: f64,
    lng: f64,
    iso2: String,
    population: Option<i64>,
}

struct MySpinTimer;

impl Guest for MySpinTimer {
    async fn handle_timer_request() {
        //let text = variables::get("message").unwrap();
        //  println!("{text}");
        let a = handle_timer().await;
    }
}

/// A simple Spin HTTP component.
//#[http_component]
//async fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
async fn handle_timer() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_env("APP_LOG_LEVEL"))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let geopoints = read_geopoints().await.unwrap();
    let mut rr: Restaurant = Restaurant::default();

    get_random_city(&mut rr, geopoints).await;
    let _ = search_nearby(&mut rr).await;
    let _ = get_place_details(&mut rr).await;
    let _ = verify_nearby(&mut rr).await;
    let _ = get_images(&mut rr).await.unwrap();
    let attached_medias = upload_mstd_images(&mut rr).await.unwrap();
    rr.mstd_media_ids = attached_medias;
    let _ = post_message(&rr).await;

    Ok(())
}

async fn read_geopoints() -> Result<Vec<Geopoint>> {
    tracing::debug!("read_geopoints");

    let pointscsv = fs::read("./geopoints.csv").expect("Should have been able to read the file");

    let pointscsv_bytes = pointscsv.as_slice();

    let mut geopoints: Vec<Geopoint> = Vec::new();
    let mut rdr = csv::Reader::from_reader(pointscsv_bytes);
    for result in rdr.deserialize() {
        let record: Geopoint = result?;
        geopoints.push(record);
    }

    Ok(geopoints)
}

async fn get_random_city(r: &mut Restaurant, g: Vec<Geopoint>) {
    let mut weighted_points: Vec<Geopoint> = Vec::new();
    let weighted_countries = vec![
        "DE", "FR", "ES", "IT", "TW", "TH", "VN", "PT", "KR", "SG", "HK",
    ];

    let mg = g
        .iter()
        .filter(|&g| g.population.unwrap_or(0) > 25000_i64)
        .cloned()
        .collect::<Vec<Geopoint>>();

    for gp in mg {
        weighted_points.push(gp.clone());

        if weighted_countries.contains(&gp.clone().iso2.as_str()) {
            weighted_points.push(gp.clone());
        }
    }

    match weighted_points.choose(&mut rand::thread_rng()) {
        Some(c) => {
            r.lat = c.lat;
            r.lng = c.lng;
        }
        None => panic!("No city picked up"),
    }
}

async fn search_nearby(r: &mut Restaurant) -> Result<()> {
    // Search restaurant nearby the city and pick one
    let api_key = spin_sdk::variables::get("google_api_key")
        .expect("You must set the GOOGLE_API_KEY environment var!");
    let url: String = format!(
        "https://maps.googleapis.com/maps/api/place/nearbysearch/json?location={},{}&radius=50000&type=cafe&keyword=coffee&key={}",
        r.lat, r.lng, api_key
    );

    let resp: Response = spin_sdk::http::send(Request::get(url)).await?;
    let body = resp.body();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    let mut filtered_places: Vec<Value> = Vec::new();
    for i in body_json["results"].as_array().unwrap() {
        if i["types"]
            .as_array()
            .unwrap()
            .contains(&Value::String("hotel".to_string()))
            || i["types"]
                .as_array()
                .unwrap()
                .contains(&Value::String("lodge".to_string()))
            || i["types"]
                .as_array()
                .unwrap()
                .contains(&Value::String("lodging".to_string()))
            || i["types"]
                .as_array()
                .unwrap()
                .contains(&Value::String("gas_station".to_string()))
            || i["types"]
                .as_array()
                .unwrap()
                .contains(&Value::String("convenience_store".to_string()))
            || i["types"]
                .as_array()
                .unwrap()
                .contains(&Value::String("grocery_or_supermarket".to_string()))
            || i["types"]
                .as_array()
                .unwrap()
                .contains(&Value::String("night_club".to_string()))
            || i["types"]
                .as_array()
                .unwrap()
                .contains(&Value::String("restaurant".to_string()))
            || i["types"]
                .as_array()
                .unwrap()
                .contains(&Value::String("bar".to_string()))
        {
            continue;
        }
        if i["rating"].as_f64().unwrap_or(0_f64) >= 2.5_f64
            && i["user_ratings_total"].as_f64().unwrap_or(0_f64) > 9_f64
        {
            filtered_places.push(i.clone());
        }
    }

    let p = filtered_places.choose(&mut rand::thread_rng()).unwrap();
    r.place_id = p.clone()["place_id"].as_str().unwrap().to_string();
    r.name = p.clone()["name"].as_str().unwrap().to_string();
    if p.get("rating").is_some() {
        r.rating = p["rating"].as_f64().unwrap();
    } else {
        r.rating = 0.0;
    };

    Ok(())
}

async fn get_place_details(r: &mut Restaurant) -> Result<()> {
    // Get restaurnat's detailed photos and formatted_address

    let api_key = spin_sdk::variables::get("google_api_key")
        .expect("You must set the GOOGLE_API_KEY environment var!");
    let url: String = format!("https://maps.googleapis.com/maps/api/place/details/json?place_id={}&fields=photos,formatted_address&key={}",
        r.place_id, api_key
    );

    let resp: Response = spin_sdk::http::send(Request::get(url)).await?;
    let body = resp.body();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    r.address = body_json["result"]["formatted_address"]
        .as_str()
        .unwrap()
        .to_string();

    let mut n: usize = 0;
    if body_json["result"]["photos"].as_array().unwrap().len() == 0 {
        tracing::info!("No picture from google");
        // return certain error and take care that error at main
    } else if body_json["result"]["photos"].as_array().unwrap().len() < 4 {
        n = body_json["result"]["photos"].as_array().unwrap().len();
    } else {
        n = 4
    }
    for i in 0..n {
        r.pics_url.push(format!(
            "https://maps.googleapis.com/maps/api/place/photo?maxwidth=640&photoreference={}&key=",
            body_json["result"]["photos"][i]["photo_reference"]
                .clone()
                .as_str()
                .unwrap()
                .to_string(),
        ));
    }

    Ok(())
}

async fn verify_nearby(r: &mut Restaurant) -> Result<()> {
    // Check distance not too far?
    // Nothing to verify now
    Ok(())
}

async fn search_street_image(r: &mut Restaurant) -> Result<()> {
    // First picture is from street image
    let api_key =
        variables::get("google_api_key").expect("You must set the GOOGLE_API_KEY environment var!");
    let url = format!("https://maps.googleapis.com/maps/api/streetview/metadata?size=640x640&location={},{}&key={}",
        r.lat,
        r.lng,
        api_key,
    );

    let hr: Response = spin_sdk::http::send(Request::get(url)).await?;
    let resp: Value = serde_json::from_slice(hr.body()).unwrap();
    if resp["status"] != "ZERO_RESULT" {
        let pic_url = format!("https://maps.googleapis.com/maps/api/streetview?size=640x640&return_error_codes=true&location={},{}&key=",
                              r.lat,
                              r.lng,
        );
        r.pics_url.push(pic_url);
    }
    Ok(())
}

async fn simple_http_get_request(url: &str) -> Result<Response> {
    Ok(spin_sdk::http::send(Request::get(url)).await?)
}

async fn get_images(r: &mut Restaurant) -> Result<()> {
    let mut images: Vec<Vec<u8>> = Vec::new();
    for (_, url) in r.pics_url.iter().enumerate() {
        let api_key = variables::get("google_api_key")
            .expect("You must set the GOOGLE_API_KEY environment var!");
        let url: String = format!("{url}{api_key}");
        let mut http_get_retry = 0;
        let mut hr = simple_http_get_request(url.as_str()).await.unwrap();

        // http stauts 302 is redirection.
        while hr.status() != &200u16 && http_get_retry < 5 {
            http_get_retry += 1;
            let location = hr.header("Location").unwrap().as_str().unwrap();
            hr = simple_http_get_request(location).await.unwrap();
        }

        let a = hr.body().to_owned();
        images.push(a);
    }
    r.pics_binary = images;

    Ok(())
}

async fn upload_mstd_images(r: &mut Restaurant) -> Result<Vec<String>> {
    let access_token = variables::get("mstdn_access_token")
        .expect("You must set the MSTDN_ACCESS_TOKEN environment var!");
    let mstdn_uri: String =
        variables::get("mstdn_url").expect("You must set the MSTDN environment var!");

    let mut media_attachements: Vec<String> = Vec::new();
    for (i, image) in r.pics_binary.iter().enumerate() {
        let url = format!("https://{mstdn_uri}/api/v2/media");
        //let my_file = imgs.get(i).unwrap();

        let mut mp = Multipart::new();
        mp.add_text("foo", "bar");
        mp.add_stream(
            "file",
            image.as_slice(),
            Some("file"),
            Some(mime::IMAGE_JPEG),
        );

        let mut b = mp.prepare().unwrap();

        let mut foo = Vec::new();
        let d = b.read_to_end(&mut foo);

        let boundary = b.boundary();

        let request = RequestBuilder::new(Method::Post, url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(foo)
            .build();
        let response: Response = http::send(request).await?;
        let status = response.status();
        let bbody = response.body();
        let bbbody: Value = serde_json::from_str(std::str::from_utf8(bbody).unwrap()).unwrap();
        let media_id = bbbody.get("id").unwrap().as_str().unwrap().to_string();
        media_attachements.push(media_id);
    }
    Ok(media_attachements)
}

async fn post_message(r: &Restaurant) -> Result<()> {
    let mstdn_uri: String =
        variables::get("mstdn_url").expect("You must set the MSTDN environment var!");

    let access_token = variables::get("mstdn_access_token")
        .expect("You must set the MSTDN_ACCESS_TOKEN environment var!");

    let msg: String = format!(
        "{}\n{}\n{}\nhttps://www.google.com/maps/search/?api=1&query={},{}&query_place_id={}\n#coffee #cafe",
        r.name,
        r.address,
        rating_stars(r.rating).await,
        r.lat,
        r.lng,
        r.place_id,
    );

    let b = json!({
        "status": msg,
        "visibility": "public",
        "language": "eng",
        "media_ids": r.mstd_media_ids,
    });

    let req = RequestBuilder::new(Method::Post, format!("https://{mstdn_uri}/api/v1/statuses"))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", format!("application/json"))
        .body(serde_json::to_string(&b).unwrap())
        .build();
    let _: Response = http::send(req).await?;

    Ok(())
}

async fn rating_stars(rating: f64) -> String {
    let major: usize = (rating - (rating % 1.0)) as usize;
    let minor: f64 = rating % 1.0;
    let mut star: String = "★".repeat(major);
    if minor > 0.0 {
        star = format!("{star}☆");
    }
    star
}

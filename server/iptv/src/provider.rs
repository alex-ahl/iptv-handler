use std::{ env};



pub async fn  verbs() {
    let m3u = match env::var("M3U") {
        Ok(val) => val,
        Err(e) => e.to_string(),
    };

    let _res = match get_m3u(&m3u).await {
        Ok(res) => res,
        Err(e) => e.to_string(),
    };
}

async fn get_m3u(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let res = surf::get(url).await?.body_string().await?;
    Ok(res)
}
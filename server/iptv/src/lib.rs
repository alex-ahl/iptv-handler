use surf::Url;

pub async fn parser(url: Url) {
    let _res = match get_m3u(&url).await {
        Ok(res) => res,
        Err(e) => e.to_string(),
    };
}

async fn get_m3u(url: &Url) -> Result<String, Box<dyn std::error::Error>> {
    let res = surf::get(url).await?.body_string().await?;
    Ok(res)
}

use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

pub const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0";

pub fn build_client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?)
}

pub async fn fetch_html(client: &Client, url: &str) -> Result<String> {
    let response = client.get(url).send().await?.error_for_status()?;
    Ok(response.text().await?)
}

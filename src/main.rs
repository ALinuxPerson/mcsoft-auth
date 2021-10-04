use std::env;
use anyhow::Context;
use warp::Filter;
use std::sync::mpsc;
use serde::Deserialize;
use std::borrow::Cow;
use rand::Rng;
use rand::distributions::Alphanumeric;
use reqwest::Url;

#[derive(Deserialize)]
pub struct Query {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize)]
pub struct AccessToken {
    pub access_token: String,
}

#[derive(Deserialize)]
pub struct Xui {
    #[serde(rename = "uhs")]
    pub user_hash: String,
}

#[derive(Deserialize)]
pub struct DisplayClaims {
    pub xui: Vec<Xui>,
}

#[derive(Deserialize)]
pub struct AuthenticateWithXboxLiveOrXsts {
    #[serde(rename = "Token")]
    pub token: String,

    #[serde(rename = "DisplayClaims")]
    pub display_claims: DisplayClaims,
}

#[derive(Deserialize, PartialEq)]
pub struct Item {
    pub name: Cow<'static, str>,
    // pub signature: String, // todo: signature verification
}

impl Item {
    pub const PRODUCT_MINECRAFT: Self = Self {
        name: Cow::Borrowed("product_minecraft")
    };
    pub const GAME_MINECRAFT: Self = Self {
        name: Cow::Borrowed("game_minecraft")
    };
}

#[derive(Deserialize)]
pub struct Store {
    pub items: Vec<Item>,

    // pub signature: String, // todo: signature verification

    #[serde(rename = "keyId")]
    pub key_id: String,
}

impl AuthenticateWithXboxLiveOrXsts {
    pub fn extract_essential_information(self) -> anyhow::Result<(String, String)> {
        let token = self.token;
        let user_hash = self.display_claims.xui
            .into_iter()
            .next()
            .context("no xui found")?
            .user_hash;

        Ok((token, user_hash))
    }
}

#[derive(Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

async fn receive_query(port: u16) -> Query {
    let (sender, receiver) = mpsc::sync_channel(1);
    let route = warp::get()
        .and(warp::filters::query::query())
        .map(move |query: Query| {
            sender.send(query).expect("failed to send query");
            "Successfully received query"
        });

    tokio::task::spawn(warp::serve(route).run(([127, 0, 0, 1], port)));

    receiver.recv().expect("channel has hung up")
}

fn random_string() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let client_id = env::var("CLIENT_ID").context("CLIENT_ID is needed")?;
    let client_secret = env::var("CLIENT_SECRET").context("CLIENT_SECRET is needed")?;
    let redirect_uri: Url = env::var("REDIRECT_URI")
        .context("REDIRECT_URI is needed")?
        .parse()
        .context("redirect uri is not a valid url")?;

    match redirect_uri.domain() {
        Some(domain) => anyhow::ensure!(domain == "localhost" || domain == "127.0.0.1", "domain '{}' isn't valid, it must be '127.0.0.1' or 'localhost'", domain),
        None => anyhow::bail!("the redirect uri must have a domain")
    }

    let port = env::var("PORT")
        .ok()
        .and_then(|port| match port.parse::<u16>() {
            Ok(port) => Some(port),
            Err(_) => {
                eprintln!("'{}' is not a valid port, using the given redirect uri's port", port);
                None
            }
        })
        .unwrap_or_else(|| match redirect_uri.port() {
            Some(port) => port,
            None => {
                eprintln!("The redirect uri '{}' doesn't have a port given, assuming port is 80", redirect_uri);
                80
            }
        });
    let state = random_string();
    let url = format!("https://login.live.com/oauth20_authorize.srf\
?client_id={}\
&response_type=code\
&redirect_uri={}\
&scope=XboxLive.signin%20offline_access\
&state={}", client_id, redirect_uri, state);

    if let Err(error) = webbrowser::open(&url) {
        println!("error opening browser: {}", error);
        println!("use this link instead:\n{}", url)
    }

    println!("Now awaiting code.");
    let query = receive_query(port).await;

    anyhow::ensure!(query.state == state, "state mismatch: got state '{}' from query, but expected state was '{}'", query.state, state);

    let client = reqwest::Client::new();

    println!("Now getting the access token.");
    let access_token: AccessToken = client
        .post("https://login.live.com/oauth20_token.srf")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", query.code),
            ("grant_type", "authorization_code".to_string()),
            ("redirect_uri", redirect_uri.to_string())
        ])
        .send()
        .await?
        .json()
        .await?;
    let access_token = access_token.access_token;
    let json = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", access_token),
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });
    println!("Now authenticating with Xbox Live.");
    let auth_with_xbl: AuthenticateWithXboxLiveOrXsts = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&json)
        .send()
        .await?
        .json()
        .await?;
    let (token, user_hash) = auth_with_xbl.extract_essential_information()?;
    println!("Now getting an Xbox Live Security Token (XSTS).");
    let json = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });
    let auth_with_xsts: AuthenticateWithXboxLiveOrXsts = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&json)
        .send()
        .await?
        .json()
        .await?;
    let (token, _) = auth_with_xsts.extract_essential_information()?;
    println!("Now authenticating with Minecraft.");
    let access_token: AccessToken = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", user_hash, token)
        }))
        .send()
        .await?
        .json()
        .await?;
    let access_token = access_token.access_token;

    println!("Checking for game ownership.");
    // i don't know how to do signature verification, so we just have to assume the signatures are
    // valid :)
    let store: Store = client
        .get("https://api.minecraftservices.com/entitlements/mcstore")
        .bearer_auth(&access_token)
        .send()
        .await?
        .json()
        .await?;

    anyhow::ensure!(
        store.items.contains(&Item::PRODUCT_MINECRAFT),
        "product_minecraft item doesn't exist. do you really own the game?"
    );

    anyhow::ensure!(
        store.items.contains(&Item::GAME_MINECRAFT),
        "game_minecraft item doesn't exist. do you really own the game?"
    );

    println!("Getting game profile.");

    let profile: Profile = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(&access_token)
        .send()
        .await?
        .json()
        .await?;

    println!("Congratulations, you authenticated to minecraft from Rust!");
    println!("access_token={} username={} uuid={}", access_token, profile.name, profile.id);

    Ok(())
}

use oauth2::TokenResponse;
use std::{io, process};

use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, CurlHttpClient,
    PkceCodeChallenge, RedirectUrl, Scope, TokenUrl, basic::BasicClient, url::Url,
};

use crate::utils::load_config;

pub struct TokenManager {}

impl TokenManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_token() -> String {
        let mut cfg = load_config();
        let current_account = cfg.current_account();
        let client = BasicClient::new(ClientId::new(current_account.client_id.clone()))
            .set_client_secret(ClientSecret::new(current_account.client_secret.clone()))
            .set_auth_uri(AuthUrl::new("https://x.com/i/oauth2/authorize".to_string()).unwrap())
            .set_token_uri(TokenUrl::new("https://api.x.com/2/oauth2/token".to_string()).unwrap())
            .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:3000".to_string()).unwrap());

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("bookmark.read".to_string()))
            .add_scope(Scope::new("tweet.read".to_string()))
            .add_scope(Scope::new("users.read".to_string()))
            .add_scope(Scope::new("offline.access".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        println!("Open this URL in a browser:");
        println!("{auth_url}");
        println!("Paste the full callback URL:");
        let mut callback_url = String::new();
        io::stdin().read_line(&mut callback_url).unwrap();

        let parsed = Url::parse(callback_url.trim()).unwrap();

        let returned_state = parsed
            .query_pairs()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.into_owned())
            .ok_or("missing state")
            .unwrap();

        if returned_state != *csrf_token.secret() {
            println!("CSRF token mismatch");
            process::exit(1)
        }

        let code = parsed
            .query_pairs()
            .find(|(k, _)| k == "code")
            .map(|(_, v)| v.into_owned())
            .ok_or("missing code")
            .unwrap();

        let client = BasicClient::new(ClientId::new(current_account.client_id.clone()))
            .set_client_secret(ClientSecret::new(current_account.client_secret.clone()))
            .set_auth_uri(AuthUrl::new("https://x.com/i/oauth2/authorize".to_string()).unwrap())
            .set_token_uri(TokenUrl::new("https://api.x.com/2/oauth2/token".to_string()).unwrap())
            .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:3000".to_string()).unwrap());

        let token = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request(&CurlHttpClient)
            .unwrap();

        let access_token = token.access_token().secret().to_owned();
        access_token
    }
}

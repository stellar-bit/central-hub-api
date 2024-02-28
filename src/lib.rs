use std::str::FromStr;

use reqwest::{ClientBuilder, Request, RequestBuilder, Response, StatusCode, Url};
use serde::Deserialize;

const HUB_WEB_ADDR: &str = "http://localhost:3000/";

#[derive(Deserialize, Debug)]
pub struct ServerDetails {
    pub name: String,
    pub id: i64,
    pub addr: Option<String>, //None if offline
    pub owner_id: i64,
}

/// # API for the Stellar Bit Hub
/// Ensures that each request is appropriately authorized with the supplied credentials.
pub struct HubAPI {
    client: reqwest::Client,
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
pub struct UserData {
    pub username: String,
    pub id: i64,
}

impl HubAPI {
    pub async fn connect(username: String, password: String) -> Self {
        let client = ClientBuilder::new().cookie_store(true).build().unwrap();

        Self {
            client,
            username,
            password
        }
    }
    pub fn get(&self, rel_path: &str) -> RequestBuilder {
        self.client.get(Url::from_str(HUB_WEB_ADDR).unwrap().join(rel_path).unwrap())
    }
    pub fn post(&self, rel_path: &str) -> RequestBuilder {
        self.client.post(Url::from_str(HUB_WEB_ADDR).unwrap().join(rel_path).unwrap())
    }
    pub async fn login(&self) {
        let params = [("username", &self.username), ("password", &self.password)];

        self.post("/api/login").form(&params).send().await.unwrap().error_for_status().unwrap();
    }
    pub async fn send(&self, req: RequestBuilder) -> Response {
        let resp = req.try_clone().unwrap().send().await.unwrap();
        if resp.status() == StatusCode::UNAUTHORIZED {
            self.login().await; 
            let resp = req.send().await.unwrap();
            resp
        }
        else {
            resp
        }

    }
    pub async fn servers(&self) -> Vec<ServerDetails> {
        let req = self.get("/api/servers"); 
        self.send(req).await.error_for_status().unwrap().json::<Vec<ServerDetails>>().await.unwrap()
    }
    pub async fn user_data(&self, id: i64) -> UserData {
        let resp = self.send(self.get(&format!("/api/users/{id}"))).await;
        resp.error_for_status().unwrap().json::<UserData>().await.unwrap()
    }
    pub async fn user_data_username(&self, username: &str) -> UserData {
        let resp = self.send(self.get(&format!("/api/users/by_username/{username}"))).await;
        resp.error_for_status().unwrap().json::<UserData>().await.unwrap()
    }
    pub async fn server_keep_alive(&self, server_id: i64, server_addr: &str) {
        let req = self.post(&format!("/api/servers/keep_alive/{server_id}/{server_addr}"));
        self.send(req).await.error_for_status().unwrap();
    }
}
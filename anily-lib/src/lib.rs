use soup::prelude::*;
use reqwest::header;
use aes::{Aes256, NewBlockCipher};
use aes::cipher::{
    BlockDecrypt, generic_array::GenericArray
};
use url::Url;

static BASE_URL: &'static str = "https://www1.gogoanime.cm";
static USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36";

pub struct Anime {
    pub name: String,
    pub ep_start: u32,
    pub ep_end: u32,

    watched_ep: Vec<u32>,
    base_link: String,
    ep_link: String,
}

fn get_ajax_params(resp: &str, id: &str) {
    let soup = Soup::new(&resp);
    let mut ts: String = String::new();
    let mut sc_crypto: String = String::new();
    for script in soup.tag("script").find_all() {
        let data = match script.get("data-name") {
            Some (x) => x,
            None => continue,
        };
        if data == "ts" {
            ts = script.get("data-value").unwrap();
        }
        if data == "crypto" {
            sc_crypto = script.get("data-value").unwrap();
        }
    }
    let mut crypto: String = String::new();
    for meta in soup.tag("meta").find_all() {
        let name = meta.get("name").unwrap();
        if name == "crypto" {
            crypto = meta.get("content").unwrap();
            break
        }
    }
    let key = format!("{}{}", ts, ts);
    let cipher = Aes256::new(GenericArray::from_slice(key.as_bytes()));
    let mut cryp = *GenericArray::from_slice(sc_crypto.as_bytes());
    let sm = cipher.decrypt_block(&mut cryp);
}

impl Anime {
    fn new(name: &str) -> Anime {
        let name = name
            .replace(" & ", "-")
            .replace(" ", "-")
            .replace("(", "")
            .replace(")", "")
            .replace(":", "")
            .replace("!", "");
        let base_link = format!("{}/category/{}", BASE_URL, name);
        let ep_link = format!("{}/{}-episode-", BASE_URL, name);

        let anime = Anime{
            name: name,
            ep_end: 1,
            ep_start: 0,
            watched_ep: vec!(),
            base_link: base_link,
            ep_link: ep_link,
        };
        anime
    }

    fn get_embbed_url(&self, ep_link: &str) -> String {
        let resp = reqwest::blocking::get(ep_link)
            .expect("can't Connect")
            .text()
            .unwrap();
        let soup = Soup::new(&resp);
        let vidcdn = soup.class("vidcdn").find().expect("Couldn't find vidcdn");
        let embbed = vidcdn.tag("a").find().expect("Couldn't find 'a' tag");
        let embbed_link = embbed.get("data-video").expect("No DATA");
        let url = format!("https:{}", embbed_link);
        url
    }

    fn get_available_videos(&self, ep_number: u32) {
        let ep_link = format!("{}{}", self.ep_link, ep_number);
        let url = self.get_embbed_url(&ep_link);
        let embbed = Url::parse(&url).unwrap();
        let mut headers = header::HeaderMap::new();
        headers.insert("User-Agent",
            header::HeaderValue::from_static(USER_AGENT));
        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        let resp = client.get(url)
            .send()
            .unwrap()
            .text()
            .unwrap();
        let mut query_pairs = embbed.query_pairs();
        let (id_name, id) =  query_pairs.next().unwrap();
        let params = get_ajax_params(&resp, &id.into_owned());
    }

    pub fn get_ep_list(&mut self) {
        let resp = reqwest::blocking::get(&self.base_link)
            .expect("Can't Connect")
            .text()
            .unwrap();
        let soup = Soup::new(&resp);
        let active = soup.class("active").find().unwrap();

        self.ep_start = active.get("ep_start")
            .unwrap()
            .parse()
            .unwrap();

        self.ep_end = active.get("ep_end")
            .unwrap()
            .parse()
            .unwrap();
    }

    pub fn get_episode(&self,ep_number: u32) {
        self.get_available_videos(ep_number);
    }
}

pub fn search_anime(search_term: &str) -> Vec<Anime> {
    let search_url = format!("{}//search.html?keyword={}",
        BASE_URL, search_term);
    let mut anime_vec: Vec<Anime> = vec!();
    let resp = reqwest::blocking::get(search_url)
        .expect("Can't connect")
        .text()
        .unwrap();
    let soup = Soup::new(&resp);
    for p in soup.class("name").find_all() {
        let a = p.tag("a").find().expect("SMTH");
        let name = a.get("title").unwrap();
        let anime = Anime::new(&name);
        anime_vec.push(anime);
    }
    anime_vec
}

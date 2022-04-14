use derive_more::From;
use std::fs;
use json::{JsonError, JsonValue};
use crate::ApiError::OtherError;

#[derive(Debug)]
struct ApiClient {
    login: String,
    password: String,
    domain: String,
}

#[derive(Debug, From)]
enum ApiError {
    UReqError(ureq::Error),
    JsonError(JsonError),
    StdError(std::io::Error),
    OtherError(String),
}

impl ApiClient {
    fn request(&self, method: &str) -> Result<JsonValue, ApiError> {
        self.request_url(&self.make_url(method))
    }

    fn make_url(&self, method: &str) -> String {
        format!("https://{}.zendesk.com/api/v2/{}", self.domain, method)
    }

    fn request_url(&self, url: &String) -> Result<JsonValue, ApiError> {
        let auth_str = base64::encode(format!("{}:{}", &self.login, &self.password));
        let r = ureq::get(&url)
            .set("Authorization", &format!("Basic {}", &auth_str))
            .call()?;

        eprintln!("Requested {url}");

        Ok(
            json::parse(
                &(r.into_string()?)
            )?
        )
    }
}

struct TicketIterator<'a> {
    client: &'a ApiClient,
    page: u32,
    next_page_link: Option<String>,
    finished: bool,
    known_page_size: Option<usize>,
    total_tickets: Option<u32>,
}

impl<'a> TicketIterator<'a> {
    fn new(client: &'a ApiClient) -> Self {
        TicketIterator {
            client,
            page: 0,
            next_page_link: None,
            finished: false,
            known_page_size: None,
            total_tickets: None,
        }
    }
}

impl<'a> Iterator for TicketIterator<'a> {
    type Item = Result<JsonValue, ApiError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let mut req_url = &self.client.make_url("tickets");
        if let Some(url) = &self.next_page_link {
            req_url = &url;
        }

        let resp = self.client.request_url(req_url);

        match &resp {
            Ok(val) => {
                if !val.has_key("tickets") {
                    Some(Err(OtherError("Tickets elt not found".to_string())))
                } else if !val["tickets"].is_array() {
                    Some(Err(OtherError("Tickets are not array".to_string())))
                } else if val["tickets"].len() == 0 {
                    Some(Err(OtherError("Tickets len is 0".to_string())))
                } else {
                    if let None = self.known_page_size {
                        self.known_page_size = Some(val["tickets"].len());
                        self.total_tickets = val["count"].as_u32()
                    }

                    self.page += 1;
                    self.next_page_link = val["next_page"].as_str().map(|s| s.to_string());
                    if self.next_page_link.is_none() {
                        self.finished = true;
                    }
                    Some(resp)
                }
            }
            Err(_) => {
                Some(resp)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let (Some(ps), Some(tt)) = (self.known_page_size, self.total_tickets) {
            let size = (tt as usize) / ps;
            (size, Some(size + 1))
        } else {
            (0, None)
        }
    }
}

fn main() {
    let config_str = fs::read_to_string("config.toml").unwrap();
    let config: toml::Value = toml::from_str(&config_str).unwrap();
    let client: ApiClient = ApiClient {
        login: config.get("login").expect("Expect login in config.toml").as_str().unwrap().to_string(),
        password: config.get("password").expect("Expect password in config.toml").as_str().unwrap().to_string(),
        domain: config.get("domain").expect("Expect domain in config.toml").as_str().unwrap().to_string(),
    };

    let mut tickets: Vec<JsonValue> = vec![];
    let tickets_iter = TicketIterator::new(&client);
    for v in tickets_iter {
        match v {
            Ok(val) => {
                eprint!("Got {} tickets", val["tickets"].len());
                tickets.extend(val["tickets"].members().map(|v| v.clone()));

                eprintln!(", total {} tickets", tickets.len());
            }
            Err(ref err) => {
                eprintln!("Some error: {:?}", err);
            }
        }
    }


}

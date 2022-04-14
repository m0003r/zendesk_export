use derive_more::From;
use json::{Error as JsonError, JsonValue};
use crate::ApiError::OtherError;

#[derive(Debug)]
pub struct ApiClient {
    login: String,
    password: String,
    domain: String,
}

#[derive(Debug, From)]
pub enum ApiError {
    UReqError(ureq::Error),
    JsonError(JsonError),
    StdError(std::io::Error),
    OtherError(String),
}

impl ApiClient {
    pub fn new(login: &str, password: &str, domain: &str) -> Self {
        Self {
            login: login.to_string(),
            password: password.to_string(),
            domain: domain.to_string(),
        }
    }

    pub fn request(&self, method: &str) -> Result<JsonValue, ApiError> {
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

        eprintln!("-- requested {url}");

        Ok(
            json::parse(
                &(r.into_string()?)
            )?
        )
    }
}

pub struct TicketIterator<'a> {
    client: &'a ApiClient,
    page: u32,
    next_page_link: Option<String>,
    finished: bool,
    known_page_size: Option<usize>,
    total_tickets: Option<u32>,
}

impl<'a> TicketIterator<'a> {
    pub fn new(client: &'a ApiClient) -> Self {
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

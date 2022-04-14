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

    pub fn get_comments(&self, id: &str) -> Result<JsonValue, ApiError> {
        self.request_url(&self.make_url(&format!("tickets/{}/comments", id)))
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

    pub fn get_all_pages(&self, object: &str) -> Vec<JsonValue> {
        let mut objects: Vec<JsonValue> = vec![];
        let tickets_iter = ZendeskPageIterator::new(&self, object);
        for v in tickets_iter {
            match v {
                Ok(val) => {
                    eprint!("Got {} {}", val[object].len(), object);
                    objects.extend(val[object].members().map(|v| v.clone()));

                    eprintln!(", total {} {}", objects.len(), object);
                }
                Err(ref err) => {
                    eprintln!("Some error: {:?}", err);
                }
            }
        }
        objects
    }

}

pub struct ZendeskPageIterator<'a> {
    client: &'a ApiClient,
    method: String,
    page: u32,
    next_page_link: Option<String>,
    finished: bool,
    known_page_size: Option<usize>,
    total_size: Option<u32>,
}

impl<'a> ZendeskPageIterator<'a> {
    pub fn new(client: &'a ApiClient, method: &str) -> Self {
        ZendeskPageIterator {
            client,
            method: String::from(method),
            page: 0,
            next_page_link: None,
            finished: false,
            known_page_size: None,
            total_size: None,
        }
    }
}

impl<'a> Iterator for ZendeskPageIterator<'a> {
    type Item = Result<JsonValue, ApiError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let mut req_url = &self.client.make_url(&self.method);
        if let Some(url) = &self.next_page_link {
            req_url = &url;
        }

        let resp = self.client.request_url(req_url);

        match &resp {
            Ok(val) => {
                if !val.has_key(&self.method) {
                    Some(Err(OtherError(format!("{} not found in response", &self.method))))
                } else if !val[&self.method].is_array() {
                    Some(Err(OtherError(format!("{} is not an array", &self.method))))
                } else if val[&self.method].len() == 0 {
                    Some(Err(OtherError(format!("{} is empty array", &self.method))))
                } else {
                    if let None = self.known_page_size {
                        self.known_page_size = Some(val[&self.method].len());
                        self.total_size = val["count"].as_u32()
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
        if let (Some(ps), Some(tt)) = (self.known_page_size, self.total_size) {
            let size = (tt as usize) / ps;
            (size, Some(size + 1))
        } else {
            (0, None)
        }
    }
}

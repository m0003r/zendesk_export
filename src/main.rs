use std::fs;
use json::JsonValue;
use zendesk_export::{ApiClient, TicketIterator};

fn main() {
    let config_str = fs::read_to_string("config.toml").unwrap();
    let config: toml::Value = toml::from_str(&config_str).unwrap();
    let client: ApiClient = ApiClient::new(
        config.get("login").expect("Expect login in config.toml").as_str().unwrap(),
        config.get("password").expect("Expect password in config.toml").as_str().unwrap(),
        config.get("domain").expect("Expect domain in config.toml").as_str().unwrap(),
    );

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

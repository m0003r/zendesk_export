use std::{fs, thread};
use std::time::Duration;
use clap::{Arg, Command};

use rayon::prelude::*;

use zendesk_export::ApiClient;
use zendesk_export::ApiError::UReqError;

fn main() {
    let mut cmd = Command::new("Zendesk Export")
        .version("0.1")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true)
            .default_value("config.toml"))
        .arg(Arg::new("tickets")
            .short('t')
            .long("tickets")
            .help("Do fetch tickets")
            .takes_value(false))
        .arg(Arg::new("users")
            .short('u')
            .long("users")
            .help("Do fetch users")
            .takes_value(false));
    let matches = cmd.get_matches();


    let config_str = fs::read_to_string(matches.value_of("config").unwrap()).unwrap();
    let config: toml::Value = toml::from_str(&config_str).unwrap();
    let client: ApiClient = ApiClient::new(
        config.get("login").expect("Expect login in config.toml").as_str().unwrap(),
        config.get("password").expect("Expect password in config.toml").as_str().unwrap(),
        config.get("domain").expect("Expect domain in config.toml").as_str().unwrap(),
    );

    rayon::ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();

    if matches.is_present("tickets") {
        fetch_tickets(&client);
    }
    if matches.is_present("users") {
        fetch_users(&client);
    }
}

fn fetch_tickets(client: &ApiClient) {
    let mut tickets = client.get_all_pages("tickets");
    tickets
        .par_iter_mut()
        .for_each(|ticket| {
            let ticket_id = ticket["id"].as_u64();
            match ticket_id {
                Some(id) => {
                    let id_str = id.to_string();
                    loop {
                        let comments = client.get_comments(&id_str);
                        match comments {
                            Ok(val) => {
                                eprintln!("Got {} comments for {}", val["comments"].len(), &id_str);
                                ticket["comments"] = val.clone();
                                break;
                            }
                            Err(ref err) => {
                                eprintln!("Some error: {:?}", err);
                                if let UReqError(ureq::Error::Status(status, _)) = err {
                                    if *status == 429 {
                                        thread::sleep(Duration::from_secs(10));
                                    }
                                }
                            }
                        }
                    }
                }
                None => {
                    eprintln!("No ticket id");
                }
            }
        });

    fs::write("tickets.json", json::stringify_pretty(tickets, 2)).unwrap();
}

fn fetch_users(client: &ApiClient) {
    let users = client.get_all_pages("users");
    fs::write("users.json", json::stringify_pretty(users, 2)).unwrap();
}


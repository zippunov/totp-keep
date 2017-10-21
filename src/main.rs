extern crate crypto;
extern crate clap;
extern crate base32;
extern crate oath;
extern crate rand;

mod crpt;
mod errors;
mod totpkeep;
mod utils;

use clap::{App, Arg, SubCommand};
use errors::{Error};

fn main() {
    let matches = App::new("totpkeep")
        .arg(Arg::with_name("password")
            .help("password for the service records file")
            .short("p")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("file")
            .help("service records file. Default is ~/.config/totpkeep.tkp")
            .takes_value(true)
            .short("f")
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add service record")
                .arg(Arg::with_name("name")
                    .help("Service name. For example \"site1 MyUserName 2FA\"")
                    .index(1)
                    .takes_value(true)
                    .required(true)
                )
                .arg(Arg::with_name("secret")
                    .help("TOTP secret")
                    .index(2)
                    .takes_value(true)
                    .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Remove service record")
                // or "myapp help"
                .arg(Arg::with_name("index")
                    .help("index of the record")
                    .index(1)
                    .takes_value(true)
                    .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List TOTP codes for all service records")
        )
        .subcommand(
            SubCommand::with_name("recrypt")
                .about("Remove service record")
                // or "myapp help"
                .arg(Arg::with_name("newpass")
                    .help("new password")
                    .index(1)
                    .takes_value(true)
                    .required(true)
                )
        )
        .get_matches();

    let password = matches.value_of("password").unwrap();
    let file = matches.value_of("file");
    let rslt = match matches.subcommand() {
        ("add", Some(m)) => {
            let name = m.value_of("name").unwrap();
            let code = m.value_of("secret").unwrap();
            println!("Got command add with {} {}", name, code);
            totpkeep::add_service(name, code, password, file)
        },
        ("remove", Some(m)) => {
            let index = m.value_of("index").unwrap().parse::<u16>().unwrap();
            totpkeep::remove_service(index, password, file)
        },
        ("list", Some(m)) => totpkeep::list_services(password, file),
        ("recrypt", Some(m)) => Ok(()),
        (&_, _) => Err(Error::UnknownCommand)
    };
    match rslt {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}

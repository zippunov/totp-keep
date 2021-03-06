extern crate base32;
extern crate byteorder;
extern crate clap;
extern crate crypto;
extern crate oath;
extern crate rand;

mod crpt;
mod errors;
mod totpkeep;
mod utils;
mod table;

use clap::{App, Arg, SubCommand};
use errors::{Error};

fn main() {
    let matches = App::new("totpkeep")
        .arg(Arg::with_name("password")
            .help("password for the TOTP records file")
            .short("p")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("file")
            .help("TOTP records file. Default is ~/.config/totpkeep.tkp")
            .takes_value(true)
            .short("f")
        )
        .arg(Arg::with_name("ascii")
            .help("display table with ASCII symbols instead of Unicode")
            .takes_value(false)
            .short("a")
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add record")
                .arg(Arg::with_name("name")
                    .help("Name. For example \"site1 MyUserName 2FA\"")
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
                .about("Remove record")
                // or "myapp help"
                .arg(Arg::with_name("index")
                    .help("index of the record. See index in the \"totpkeep list\" output")
                    .index(1)
                    .takes_value(true)
                    .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List codes for all records")
        )
        .subcommand(
            SubCommand::with_name("recrypt")
                .about("Re-encrypt file")
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
    let symbols: &table::TableSymbols = match matches.is_present("ascii") {
        true => &table::AsciiTableSymbols{},
        false => &table::UnicodeTableSymbols{}
    };
    let rslt = match matches.subcommand() {
        ("add", Some(m)) => {
            let name = m.value_of("name").unwrap();
            let code = m.value_of("secret").unwrap();
            totpkeep::add_service(name, code, password, file, symbols)
        },
        ("remove", Some(m)) => {
            let index = m.value_of("index").unwrap().parse::<u16>().unwrap();
            totpkeep::remove_service(index, password, file, symbols)
        },
        ("list", Some(m)) => totpkeep::list_services(password, file, symbols),
        ("recrypt", Some(m)) => Ok(()),
        (&_, _) => Err(Error::UnknownCommand)
    };
    match rslt {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}

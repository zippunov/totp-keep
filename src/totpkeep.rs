use std::fs::File;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use oath::{totp_raw_custom_time as totpfn, HashType};
use crpt::{encrypt, decrypt};
use errors::{Error};

#[derive(Debug)]
struct ServiceRecord {
    name: String,
    secret: Vec<u8>
}

impl ServiceRecord {
    fn new(name: &str, code: &str) -> Result<ServiceRecord, Error> {
        use base32:: {decode, Alphabet};
        let secret = match decode(
            Alphabet::RFC4648 { padding: false },
            &code.trim().to_uppercase().replace(" ", "").replace("\t", "")
        ) {
            Some(bytes) => bytes,
            None => return Err(Error::WrongServiceRecordData)
        };
        Ok(ServiceRecord{ name:  String::from(name), secret })
    }

    fn from(s: &str) -> Result<ServiceRecord, Error> {
        let mut parts = s.split("\0");
        ServiceRecord::new(parts.next().unwrap(), parts.next().unwrap())
    }

    fn marshall_secret(&self) -> String {
        use base32:: {encode, Alphabet};
        encode(Alphabet::RFC4648 { padding: false }, &self.secret)
    }
}

fn default_registry_path() -> Result<PathBuf, Error> {
    use std::env;
    match env::home_dir() {
        Some(home_dir) => {
            let path: PathBuf = [
                home_dir,
                PathBuf::from(".config"),
                PathBuf::from("totpkeep.tkp")
            ].iter().collect();
            Ok(path)
        }
        _ => Err(Error::NoHomeDirectory)
    }
}

#[inline]
fn get_path(file: Option<&str>) -> Result<PathBuf, Error> {
    match file {
        Some(path) => Ok(PathBuf::from(path)),
        None => default_registry_path()
    }
}

fn totp(secret: &[u8], time: u64) -> u64 {
    totpfn(secret, 6, 0, 30, time, &HashType::SHA1)
}

fn display_registry(registry: &[ServiceRecord]) {
    use std::iter::repeat;
    use std::io::{stdout, Write};

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let secs_pass = match now % 30 {
        0 => 30,
        n => n
    };

    let mut header = String::with_capacity(412);
    header.push_str("╔═");
    repeat("═").take(20).for_each(|ch| header.push_str(ch));
    header.push_str("═╤═");
    repeat("═").take(8).for_each(|ch| header.push_str(ch));
    header.push_str("═╤═");
    repeat("═").take(8).for_each(|ch| header.push_str(ch));
    header.push_str("═╤═");
    repeat("═").take(8).for_each(|ch| header.push_str(ch));
    header.push_str("═╗\n");
    header.push_str("║ Name                ");
    header.push_str(" │ Previous");
    header.push_str(" │ Current ");
    header.push_str(" │   Next   ║\n");
    header.push_str("╠═");
    repeat("═").take(20).for_each(|ch| header.push_str(ch));
    header.push_str("═╪═");
    repeat("═").take(8).for_each(|ch| header.push_str(ch));
    header.push_str("═╪═");
    repeat("═").take(8).for_each(|ch| header.push_str(ch));
    header.push_str("═╪═");
    repeat("═").take(8).for_each(|ch| header.push_str(ch));
    header.push_str("═╣\n");

    let mut footer = String::with_capacity(205);
    footer.push_str("╚═");
    repeat("═").take(20).for_each(|ch| footer.push_str(ch));
    footer.push_str("═╧═");
    repeat("═").take(8).for_each(|ch| footer.push_str(ch));
    footer.push_str("═╧═");
    repeat("═").take(8).for_each(|ch| footer.push_str(ch));
    footer.push_str("═╧═");
    repeat("═").take(8).for_each(|ch| footer.push_str(ch));
    footer.push_str("═╝\n");
    footer.push_str("║");
    repeat("░").take(secs_pass as usize).for_each(|ch| footer.push_str(ch));
    repeat(" ").take((30 - secs_pass) as usize).for_each(|ch| footer.push_str(ch));
    footer.push_str("║\n");

    let mut div = String::with_capacity(20 + 8*3 + 5 + 2 * 4);
    div.push_str("╟─");
    repeat("─").take(20).for_each(|ch| div.push_str(ch));
    div.push_str("─┼─");
    repeat("─").take(8).for_each(|ch| div.push_str(ch));
    div.push_str("─┼─");
    repeat("─").take(8).for_each(|ch| div.push_str(ch));
    div.push_str("─┼─");
    repeat("─").take(8).for_each(|ch| div.push_str(ch));
    div.push_str("─╢\n");

    let mut body = Vec::new();
    let mut index: u16 = 0;
    for record in registry {
        let res = [now - 30, now, now + 30].iter()
            .map(|t| totp(&record.secret, *t).to_string())
            .collect::<Vec<String>>();
        index = index + 1;
        body.push(format!("║ {0:}. {1: <17.20} │  {2:0>6}  │  {3:0>6}  │  {4:0>6}  ║\n", index, record.name, res[0], res[1], res[2]));
    }
    stdout().write(header.as_bytes());
    stdout().write(body.join(&div[..]).as_bytes());
    stdout().write(footer.as_bytes());
}

fn load_registry(file: Option<&str>, password: &str, ignore_not_exist: bool) -> Result<Vec<ServiceRecord>, Error> {
    use std::io::{BufReader, BufRead, Read, Cursor};
    let path = get_path(file)?;
    if !path.exists() {
        if ignore_not_exist {
            return Ok(Vec::new());
        }
        return Err(Error::FileNotFound)
    }

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut encrypted: Vec<u8> = Vec::new();
    reader.read_to_end(&mut encrypted)?;
    let decrypted: Vec<u8> = decrypt(&encrypted, password)?;
    let cursor = Cursor::new(decrypted);
    let registry = cursor
        .lines()
        .map(|el| el.unwrap())
        .map(|l| ServiceRecord::from(&l).unwrap()).collect::<Vec<ServiceRecord>>();
    return Ok(registry);
}

fn save_registry(file: Option<&str>, password: &str, registry: &Vec<ServiceRecord>) -> Result<(), Error> {
    use std::io::{Cursor, Write};
    let mut buff = Cursor::new(Vec::new());
    for record in registry {
        buff.write(record.name.as_bytes());
        buff.write(b"\0");
        buff.write(record.marshall_secret().as_bytes());
        buff.write(b"\n")?;
    };
    let encrypted = encrypt(&buff.into_inner(), password);
    let path = get_path(file)?;
    let mut file = File::create(path)?;
    file.write_all(&encrypted[..])?;
    Ok(())
}

pub fn add_service(name: &str, code: &str, password: &str, file: Option<&str>) -> Result<(), Error> {
    let mut registry = load_registry(file, password, true)?;
    let new_record = ServiceRecord::new(name, code)?;
    registry.push(new_record);
    save_registry(file, password, &registry)?;
    Ok(())
}

pub fn remove_service(index: u16, password: &str, file: Option<&str>) -> Result<(), Error> {
    let mut registry = load_registry(file, password, true)?;
    // TODO range check
    registry.remove((index + 1) as usize);
    save_registry(file, password, &registry)?;
    display_registry(&registry);
    Ok(())
}

pub fn list_services(password: &str, file: Option<&str>) -> Result<(), Error> {
    let registry = load_registry(file, password, true)?;
    display_registry(&registry);
    Ok(())
}

pub fn change_password(old_pass: &str, new_pass: &str, file: Option<&str>) -> Result<(), Error> {
    Ok(())
}

# TOTP keeper

Small CLI utility to store bunch of TOTP registration records and display current
TOTP token for each record. 
```bash
$ totpkeep -p mypass list
╔══════════════════════╤══════════╤══════════╤══════════╗
║ Name                 │ Previous │ Current  │   Next   ║
╠══════════════════════╪══════════╪══════════╪══════════╣
║ 1. peerio-zipp36     │  542832  │  649267  │  482709  ║
╟──────────────────────┼──────────┼──────────┼──────────╢
║ 2. peerio-zipp12     │  870701  │  388800  │  841458  ║
╟──────────────────────┼──────────┼──────────┼──────────╢
║ 3. AWS               │  864362  │  324277  │  227500  ║
╚══════════════════════╧══════════╧══════════╧══════════╝
║░░░░░░░░░░░░░░░               ║
```
- TOTP records are stored in form of `Name` + `TOTP Secret` pairs in encrypted file.
- As encryption is mandatory `totpkeep` requires password in form of `-p <my password>`
  parameter for each call.
- You can choose to run programm with default storage file `~/.config/totpkeep.tkp` or specify
  custom file with parameter `-f <path to custom file>`

## Installation
- Install Rust programming language. This project was built with Rust version `1.21.0`
- Run `cargo build --release`
- Resulting executable `totpkeep` will be created in `target/relese` directory

## Usage

### getting help
Run `totpkeep --help` for general usage help.
```bash
$ ./target/release/totpkeep --help
totpkeep 

USAGE:
    totpkeep [OPTIONS] -p <password> [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f <file>            TOTP records file. Default is ~/.config/totpkeep.tkp
    -p <password>        password for the TOTP records file

SUBCOMMANDS:
    add        Add record
    help       Prints this message or the help of the given subcommand(s)
    list       List codes for all records
    recrypt    Re-encrypt file
    remove     Remove record
```

You can get further help for each of sub-commands with `totp <subcommand> -h`

```bash
$totpkeep add -h
totpkeep-add 
Add record

USAGE:
    totpkeep -p <password> add <name> <secret>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <name>      Name. For example "site1 MyUserName 2FA"
    <secret>    TOTP secret
```

### Add TOTP record

```bash
$ totpkeep -p mypass add 'Name of my record' K5QXKNRDGEZTCZ2AFRLFW3JZGU
$
```

### List current TOTP tokens

```bash
$ totpkeep -p mypass list
╔══════════════════════╤══════════╤══════════╤══════════╗
║ Name                 │ Previous │ Current  │   Next   ║
╠══════════════════════╪══════════╪══════════╪══════════╣
║ 1. peerio-zipp36     │  542832  │  649267  │  482709  ║
╟──────────────────────┼──────────┼──────────┼──────────╢
║ 2. peerio-zipp12     │  870701  │  388800  │  841458  ║
╟──────────────────────┼──────────┼──────────┼──────────╢
║ 3. AWS               │  864362  │  324277  │  227500  ║
╚══════════════════════╧══════════╧══════════╧══════════╝
║░░░░░░░░░░░░░░░               ║
$
```
Progress bar at the bottom of the table shows number of seconds passed before next TOTP token change

### Remove TOTP record
Removes TOTP record by its index in `totpkeep list` table.

```bash
$ totpkeep -p mypass remove 1
$
```
### Change password
```bash
$ totpkeep -p mypassword recrypt mynewpassword
$
```

## Storage file

- File is encrypted with combination of Chacha20 and Poly1305 algorithms.
- Encryption key is derived from password with bcrypt_pbkdf.
- First 16 bytes of the file are crypto salt bytes
- Last 16 bytes of the file are Poly1305 MAC
- Bytes in between are serialised and encrypted TOTP records


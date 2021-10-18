
// Rust Imports
use std::io::{self, Read, Error, ErrorKind};
use sqlite::*;
use std::env;
use clap::{Arg, App, SubCommand};

// Static Definitions
static DATABASE: &str = "/.rustychef/rustychef.db";

// Trim newline characters for buffer input
fn trim_newline(s: &mut String) -> String {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
    s.to_string()
}

// Initialise database, will not modify if exists
fn init_db(s: &str) -> Result<sqlite::Connection> {

    let conn = sqlite::open(s).unwrap();
    
    // Create tables if not exist
    conn.execute(
        "create table if not exists recipes (
            id integer primary key,
            name text not null,
            recipe text not null,
            return text)",
        ).unwrap();

    conn.execute(
        "create table if not exists servers (
            id integer primary key,
            name text not null,
            uri text not null)",
        ).unwrap();

    conn.execute(
        r#"INSERT OR IGNORE INTO recipes VALUES (1,
        'Base64',
        '[{"op":"To Base64","args":["A-Za-z0-9+/="]}]',
        'string')"#
        ).unwrap();

    conn.execute(
        r#"INSERT OR IGNORE INTO servers VALUES (1,
        'local',
        'http://localhost:3000/bake')"#
            ).unwrap();

    Ok(conn)
}

// Collect recipe by ID
fn get_recipe(conn: &sqlite::Connection, id: i64) -> Result<String> {

    let mut cursor = conn.prepare("SELECT recipe,return FROM recipes WHERE ID=? LIMIT 1").unwrap().into_cursor();
    cursor.bind(&[Value::Integer(id)]).unwrap();

    let row = cursor.next().unwrap().unwrap();
    let return_val = format!(r#""recipe":{},"outputType":"{}""#,
                             row[0].as_string().unwrap(),
                             row[1].as_string().unwrap()
                             );
    Ok(return_val) 
}


// Get Server by ID
fn get_server(conn: &sqlite::Connection, id: i64) -> Result<String> {

    let mut cursor = conn.prepare("SELECT uri FROM servers WHERE ID=? LIMIT 1").unwrap().into_cursor();
    cursor.bind(&[Value::Integer(id)]).unwrap();

    let row = cursor.next().unwrap().unwrap();
    let return_val = format!("{}",row[0].as_string().unwrap());
    Ok(return_val) 
}


fn main() -> io::Result<()> {

    // Initialise Argument Parser
    let matches = App::new("RustyChef")
        .version("0.1")
        .author("Blitztide")
        .about("CLI for CyberChef-server")
        .arg(Arg::new("recipe")
             .short('r')
             .long("recipe")
             .takes_value(true)
             .help("Selects recipe by ID"))
        .arg(Arg::new("server")
             .short('s')
             .long("server")
             .takes_value(true)
             .help("Selects server by ID"))
        .get_matches();

    // Gets recipe id from arguments defaults to 1 as i64
    let recipe_id = matches.value_of("recipe").unwrap_or("1").parse::<i64>().unwrap();

    // Gets server id from arguments defaults to 1 as i64
    let server_id = matches.value_of("server").unwrap_or("1").parse::<i64>().unwrap();

    // Get home directory from ENV
    let home_dir = env::var("HOME").unwrap();
    // Set database location to ~/.rustychef/rustychef.db
    let db  = format!("{}{}",home_dir,DATABASE);

    // Open Connection to database and initialise
    let conn = init_db(db.as_str()).unwrap();

    // Read StdIn to buffer as a String
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    // Collect recipe from database
    let recipe = get_recipe(&conn, recipe_id).unwrap();

    // Collect server from database
    let server = get_server(&conn, server_id).unwrap();

    // Read clean stdin buffer and format for cyberchef
    let request = format!(r#"{{"input":"{}",{}}}"#,trim_newline(&mut buffer),recipe);
   
    // Creating POST request
    let client = reqwest::blocking::Client::new();
    let response = client.post(server)
        .header("User-Agent", "RustyChef")
        .header("Content-Type", "application/json")
        .body(request)
        .send()
        .unwrap();

    if response.status().is_success() {
        let result = json::parse(&response.text().unwrap()).unwrap();
        println!("{}",result["value"]);
        Ok(())
    } else {
        let error = std::io::Error::new(ErrorKind::Other, "Failed Request");
        Err(error)
    }
}

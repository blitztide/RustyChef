
// Rust Imports
use std::io::{self, Read};
use sqlite::*;
use std::env;
use clap::{Arg, App, SubCommand};

// Static Definitions
static URI: &str = "http://192.168.100.152:3000/bake";
static DATABASE: &str = "/.rustychef/rustychef.db";

// Example requests


/* Example POST request:
Connection from 127.0.0.1:37292
POST /bake HTTP/1.1
Host: localhost:3333
User-Agent: curl/7.79.1
Accept: 
Content-Type:application/json
Content-Length: 196

{"input":"... ---:.-.. --- -. --. --..--:.- -. -..:- .... .- -. -.- ...:..-. --- .-.:.- .-.. .-..:- .... .:..-. .. ... ....", "recipe":{"op":"from morse code", "args": {"wordDelimiter": "Colon"}}}

*/

fn trim_newline(s: &mut String) -> String {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
    s.to_string()
}

fn init_db(s: &str) -> Result<sqlite::Connection> {

    let conn = sqlite::open(s).unwrap();
    
    // Create tables if not exist
    conn.execute(
        "create table if not exists recipes (
            id integer primary key,
            recipe text not null,
            return text)",
        ).unwrap();

    conn.execute(
        "create table if not exists servers (
            id integer primary key,
            protocol text not null,
            url text not null)",
        ).unwrap();

    conn.execute(
        r#"INSERT OR IGNORE INTO recipes VALUES (1,
        '[{"op":"To Base64","args":["A-Za-z0-9+/="]}]',
        'string')"#
        ).unwrap();

    conn.execute(
        r#"INSERT OR IGNORE INTO servers VALUES (1,
        'HTTP',
        'localhost:3000')"#
            ).unwrap();

    Ok(conn)
}

fn get_recipe(conn: sqlite::Connection, id: i64) -> Result<String> {

    let mut cursor = conn.prepare("SELECT recipe,return FROM recipes WHERE ID=? LIMIT 1").unwrap().into_cursor();
    cursor.bind(&[Value::Integer(id)]).unwrap();

    let row = cursor.next().unwrap().unwrap();
    let return_val = format!(r#""recipe":{},"outputType":"{}""#,
                             row[0].as_string().unwrap(),
                             row[1].as_string().unwrap()
                             );
    Ok(return_val) 
}

fn get_server(conn: sqlite::Connection, id: i64) -> Result<String> {

    let mut cursor = conn.prepare("SELECT protocol,uri FROM recipes WHERE ID=? LIMIT 1").unwrap().into_cursor();
    cursor.bind(&[Value::Integer(id)]).unwrap();

    let row = cursor.next().unwrap().unwrap();
    let return_val = format!(r#""recipe":{},"outputType":"{}""#,
                             row[0].as_string().unwrap(),
                             row[1].as_string().unwrap()
                             );
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
        .get_matches();

    // Gets recipe id from arguments
    let recipe_id = matches.value_of("recipe").unwrap_or("1");

    // Get home directory from ENV
    let home_dir = env::var("HOME").unwrap();
    // Set database location to ~/.rustychef/rustychef.db
    let db  = format!("{}{}",home_dir,DATABASE);

    // Open Connection to database and initialise
    let conn = init_db(db.as_str()).unwrap();
    // Read StdIn to buffer as a String
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let recipe = get_recipe(conn, recipe_id.parse::<i64>().unwrap()).unwrap();
    let request = format!(r#"{{"input":"{}",{}}}"#,trim_newline(&mut buffer),recipe);
    // Printing request to stdout
    
    let client = reqwest::blocking::Client::new();
    let response = client.post(URI)
        .header("User-Agent", "RustyChef")
        .header("Content-Type", "application/json")
        .body(request)
        .send()
        .unwrap();
    println!("{}",response.text().unwrap());
    Ok(())
}

pub mod packets;
use clap::Parser;
use packets::ip_resover;
use reqwest::Client;
use tokio::fs::{self, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
//use tokio::net::TcpStream; // Async TCP stream
use std::path::PathBuf;
use std::ops::RangeInclusive;
use colored::Colorize;


#[derive(Parser)]
#[command(name = "BirdEye")]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'u',long)]
    url: String,

    #[arg(short = 'w',long, required = false)]
    wordlist: Option<String>,

    #[arg(short= 'p', long, value_parser = port_in_range)]
    ports: Option<u16>,

    //#[arg(short = 'o', long)]
    //output: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let cli = Cli::parse();
    let client = Client::new();

    let wordlist_path = PathBuf::from(&cli.wordlist.unwrap());
    let wordlist_bytes = fs::read(wordlist_path).await.expect("Failed to read wordlist");
    let wordlist = String::from_utf8_lossy(&wordlist_bytes);

    
    let output_file = File::create("output.txt").await.expect("Failed to create output file");
    let mut writer = BufWriter::new(output_file);

            if let Some(max_port) = cli.ports {
                let mut handles = vec![]; 
                for port in 1..=max_port {
                    let host = format!("{}:{}", cli.url, port);  // Create the "host:port" strin
                    let handle = tokio::spawn(async move {
                        match timeout(Duration::from_secs(3), TcpStream::connect(host)).await {
                            Ok(Ok(_)) => {
                                println!("Port {} is {}", port, "OPEN".green());
                            }
                            Ok(Err(_)) => {
                                println!("Port {} is {}", port, "CLOSED".red());
                            }
                            Err(_) => {
                                println!("Port {} timed out", port);
                            }
                        }
                    });
                handles.push(handle);
            }
            for handle in handles {
                handle.await.unwrap();
            }
        }
    
    println!("Starting directory brute-forcing...");
    for line in wordlist.lines() {
        
        let full_url = format!("{}{}/{}", "https://", cli.url.trim_end_matches("/"), line.trim());
        
        match client.get(&full_url)
            .header("User-Agent", "Mozilla/5.0")
            .timeout(Duration::from_secs(10))
            .send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("{} {}: {}", "FOUND!".green(), response.status() ,full_url);
                    writer.write_all(format!("{:?} Found: {}\n", response.status(), full_url).as_bytes()).await?;
                }else if response.status().is_redirection() {
                    println!("{} {} {}", "MEHHH".yellow() ,response.status() ,full_url);
                    writer.write_all(format!("{:?}---- {}\n", response.status(), full_url).as_bytes()).await?;
                }else if response.status().is_server_error() {
                    println!("{} {} {}", "TAKE A LOOK :3".blue(), response.status(),full_url);
                    writer.write_all(format!("{:?}---- {}\n", response.status(), full_url).as_bytes()).await?;
                }else if response.status().is_client_error(){
                    if response.status() == 404{
                        println!("{} {} {}", "BORING".red() , response.status(),full_url);
                    } else{
                        println!("{} {} {}", "BORING??".on_magenta() , response.status(),full_url);
                    }
                }
                writer.flush().await?;
            },
            Err(e) => {
                println!("Error making request to {}: {}", full_url, e);
            }
        }
    }


        
        println!("Completed!");
        Ok(())
}


const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a port number"))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}
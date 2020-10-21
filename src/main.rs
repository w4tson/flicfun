use std::sync::{Arc, Mutex};

use flicbtn::*;
use std::{time, thread};
use structopt::StructOpt;
use tokio::task;
use anyhow::Result;
use crate::hue::HueApi;

pub mod hue;

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::from_args();
    let flic_server = options.server;
    let button = options.button;
    println!("flic server = {}", &flic_server);
    println!("flic button = {}", &button);
    let username : String = options.user;
    
    let hue = HueApi::with_user(&username).await;
    let hue_mutex = Arc::new(Mutex::new(hue));
    
    let on_event = event_handler(move |event| {
        let hue_mutex = Arc::clone(&hue_mutex);
        
        match event {
            Event:: ConnectionStatusChanged{ conn_id: _, connection_status: ConnectionStatus::Ready, disconnect_reason: _ } => println!("READY..."),
            Event::ButtonClickOrHold {click_type: ClickType::ButtonClick, conn_id:_ ,time_diff: _, was_queued: _} =>  {
                println!("CLICKED");
    
                task::spawn_blocking(move || {
                    let hue_mutex = Arc::clone(&hue_mutex);
                    let guard = hue_mutex.lock().unwrap();
                    let result = guard.toggle_light(12).expect("failed to toggle");
                    eprintln!("result = {:#?}", result);
                    guard.list_lights();
                });
            },
            _ => { }
        }
    });

    let client = FlicClient::new(&format!("{}:5551", flic_server))
        .await?
        .register_event_handler(on_event)
        .await;
    let client1 = Arc::new(client);
    let client2 = client1.clone();

    let program_loop = tokio::spawn(async move {
        println!("===============================================");
        println!("*** Started listening to events             ***");
        println!("===============================================");
        client1.submit(Command::GetInfo).await;

        thread::sleep(time::Duration::from_millis(200));
        
        client1
            .submit(Command::CreateConnectionChannel {
                conn_id: 0,
                bd_addr: button.to_string(),
                latency_mode: LatencyMode::NormalLatency,
                auto_disconnect_time: 11111_i16,
            })
            .await;
        
        let splash = include_str!("splash.txt");
        println!("{}",splash);
    });
    
    
    let lst = tokio::spawn(async move {
        client2.listen().await;
        println!("Finished");
    });

    lst.await?;
    program_loop.await?;

    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "flicfun", about = "Hacking on the flic button")]
pub struct Options {
    #[structopt(short = "s", long = "server", env = "FLIC_SERVER")]
    /// the hostname of the flicd server
    pub server: String,

    #[structopt(short = "b", long = "button", env = "FLIC_BUTTON")]
    /// the mac address of the flic button to connect to 
    pub button: String,

    #[structopt(short = "u", long = "hue-user", env = "HUE_USERNAME")]
    /// the hostname of the hue bridge
    pub user: String,
}


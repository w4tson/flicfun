use std::io::*;
use std::sync::{Arc, Mutex};

use flicbtn::*;
use std::{time, thread};
use structopt::StructOpt;
use hueclient::bridge::Bridge;
use tokio::task;
use std::sync::MutexGuard;

fn list_lights(bridge: &MutexGuard<Bridge>) {
    match bridge.get_all_lights() {
        Ok(lights) => {
            println!("id name                 on    bri   hue sat temp  x      y");
            for ref l in lights.iter() {
                println!(
                    "{:2} {:20} {:5} {:3} {:5} {:3} {:4}K {:4} {:4}",
                    l.id,
                    l.light.name,
                    if l.light.state.on { "on" } else { "off" },
                    if l.light.state.bri.is_some() {l.light.state.bri.unwrap()} else {0},
                    if l.light.state.hue.is_some() {l.light.state.hue.unwrap()} else {0},
                    if l.light.state.sat.is_some() {l.light.state.sat.unwrap()} else {0},
                    if l.light.state.ct.is_some() {l.light.state.ct.map(|k| if k != 0 { 1000000u32 / (k as u32) } else { 0 }).unwrap()} else {0},
                    if l.light.state.xy.is_some() {l.light.state.xy.unwrap().0} else {0.0},
                    if l.light.state.xy.is_some() {l.light.state.xy.unwrap().1} else {0.0},
                );
            }
        }
        Err(err) => {
            println!("Error: {}", err);
            ::std::process::exit(2)
        }
    }
}

fn toggle_light(bridge : &MutexGuard<Bridge>, id: usize) {
    let lights = bridge.get_all_lights().unwrap();
    let light = lights.iter().find(|&light| light.id == id).expect(&format!("No light with id {}", id));
    if light.light.state.on {
        bridge.set_light_state(id, &hueclient::bridge::CommandLight::default().off());
    } else {
        bridge.set_light_state(id, &hueclient::bridge::CommandLight::default().on());
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::from_args();
    let flic_server = options.server;
    let button = options.button;
    println!("flic server = {}", &flic_server);
    println!("flic button = {}", &button);
    
    let username : String = options.user;
    let bridge = task::spawn_blocking(move || {
        Bridge::discover().unwrap().with_user(username)
    }).await?;
    
    let bridge_mutex = Arc::new(Mutex::new(bridge));

    let event = event_handler(move |event| {
        let bridge_mutex = Arc::clone(&bridge_mutex);
        
        match event {
            Event:: ConnectionStatusChanged{ conn_id: _, connection_status: ConnectionStatus::Ready, disconnect_reason: _ } => println!("READY..."),
            Event::ButtonClickOrHold {click_type: ClickType::ButtonClick, conn_id:_ ,time_diff: _, was_queued: _} =>  {
                println!("CLICKED");
    
                task::spawn_blocking(move || {
                    let bridge_mutex = Arc::clone(&bridge_mutex);
                    let guard = bridge_mutex.lock().unwrap();

                    // list_lights(&guard);
                    toggle_light(&guard, 12);
                });
            },
            _ => { }
        }
    });

    let client = FlicClient::new(&format!("{}:5551", flic_server))
        .await?
        .register_event_handler(event)
        .await;
    let client1 = Arc::new(client);
    let client2 = client1.clone();

    let program_loop = tokio::spawn(async move {
        println!("===============================================");
        println!("*** Started listening to events             ***");
        println!("===============================================");
        client1.submit(Command::GetInfo).await;

        thread::sleep(time::Duration::from_millis(1000));
        
        client1
            .submit(Command::CreateConnectionChannel {
                conn_id: 0,
                bd_addr: button.to_string(),
                latency_mode: LatencyMode::NormalLatency,
                auto_disconnect_time: 11111_i16,
            })
            .await;

        let delay = time::Duration::from_millis(500);

        loop {
            thread::sleep(delay);
        }
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
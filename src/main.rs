use std::io::*;
use std::sync::Arc;

use flicbtn::*;
use std::{time, thread};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::from_args();
    let flic_server = options.server;
    let button = options.button;
    println!("flic server = {}", &flic_server);
    println!("flic button = {}", &button);
    
    let event = event_handler(|event| {
        match event {
            Event:: ConnectionStatusChanged{ conn_id, connection_status: ConnectionStatus::Ready, disconnect_reason} => println!("READY..."),
            Event::ButtonClickOrHold {click_type: ClickType::ButtonClick, conn_id,time_diff, was_queued} => println!("BUTTON CLICK: {:?}", event),
            _ =>  println!("ping response: {:?}", event)
        }
    });

    let client = FlicClient::new(&format!("{}:5551", flic_server))
        .await?
        .register_event_handler(event)
        .await;
    let client1 = Arc::new(client);
    let client2 = client1.clone();


    let mut conn_id = 0;

    let programLoop = tokio::spawn(async move {
        println!("===============================================");
        println!("*** Started listening to events             ***");
        println!("===============================================");
        client1.submit(Command::GetInfo).await;

        thread::sleep(time::Duration::from_millis(1000));
        
        client1
            .submit(Command::CreateConnectionChannel {
                conn_id,
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
    programLoop.await?;

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
    pub button: String
}
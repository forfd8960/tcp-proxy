use anyhow::bail;
use std::env;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() < 3 {
        bail!("not enough arguments")
    }

    let addr = &args[1];
    let remote_addr = &args[2];
    let listener = TcpListener::bind(&addr).await?;

    loop {
        let remote_clone = remote_addr.clone();
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            match proxy(socket, remote_clone).await {
                Ok(_) => {}
                Err(e) => {
                    println!("proxy err: {}", e);
                }
            }
        });
    }
}

async fn proxy(mut socket: TcpStream, remote: String) -> anyhow::Result<()> {
    let mut remote_socket = TcpStream::connect(&remote).await?;

    match tokio::io::copy(&mut socket, &mut remote_socket).await {
        Ok(n) => {
            println!("copy {} from {:?}", n, socket.local_addr())
        }
        Err(e) => bail!("{}", e),
    }

    match tokio::io::copy(&mut remote_socket, &mut socket).await {
        Ok(n) => {
            println!(
                "copy {} from {:?} to {:?}",
                n,
                remote_socket.local_addr(),
                socket.local_addr()
            );
            Ok(())
        }
        Err(e) => bail!("{}", e),
    }
}

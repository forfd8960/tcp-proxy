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

    println!("listening on: {}", addr);

    loop {
        let remote_clone = remote_addr.clone();
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            match proxy(socket, remote_clone).await {
                Ok(_) => {
                    println!("done proxy");
                    return;
                }
                Err(e) => {
                    println!("proxy err: {}", e);
                }
            }
        });
    }
}

async fn proxy(mut socket: TcpStream, remote: String) -> anyhow::Result<()> {
    println!("proxy tcp to: {}", remote);

    let mut remote_socket = TcpStream::connect(&remote).await?;
    let (mut r_read, mut r_write) = remote_socket.split();

    let (mut s_read, mut s_write) = socket.split();
    let (r1, r2) = tokio::join!(
        tokio::io::copy(&mut s_read, &mut r_write),
        tokio::io::copy(&mut r_read, &mut s_write)
    );

    match r1 {
        Ok(n) => {
            println!("copy {} from {:?}", n, socket.local_addr())
        }
        Err(e) => bail!("{}", e),
    }

    match r2 {
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

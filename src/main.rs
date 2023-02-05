use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    //  broadcast a message to all clients
    let (tx, _rx) = broadcast::channel::<String>(10);

    loop {
        let (mut socket, _addr) = listener.accept().await.unwrap();

        // clone ts to bring into the scope of the loop
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // wrap all handling for an individual client in a call to tokio spawn
        // moves all client handling into an independent task
        tokio::spawn(async move {
            let (socket_reader, mut socket_writer) = socket.split();
            let mut reader = BufReader::new(socket_reader);
            let mut line = String::new();

            loop {
                // select takes an identifier and then a future. Similar to python's async.gather or from_futures
                tokio::select! {
                // read from the client
                    result = reader.read_line(& mut line)=>{
                    if result.unwrap() == 0 {
                    break;
                }
                // after reading the line from client, put something on the cue
                    tx.send(line.clone()).unwrap();
                    line.clear();
                }
                // add another future
                // read from the cue
                    result = rx.recv()=>{
                        let msg = result.unwrap();
                         // send to the client
                        socket_writer.write_all(msg.as_bytes()).await.unwrap();
                    }
                };
            }
        });
    }
}

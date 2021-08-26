use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::mpsc::Receiver;
use async_trait::async_trait;

#[async_trait]
trait Actor {
    fn new(receiver: mpsc::Receiver<ActorMessage>) -> Self;

    fn handle_message(&mut self, msg: ActorMessage);

    // while loop consuming messages from receiver
    async fn run(&mut self);
}

// message pump
// it has the receiver
// it pulls messages from the receiver
// and it calls the message handler of its actor

enum ActorMessage {
    GetNextUID {
        reply_to: oneshot::Sender<u64>,
    }
}

//
// Handle
//
struct Handle {
    sender: mpsc::Sender<ActorMessage>,
}

impl Handle {
    async fn get_next_uid(&mut self) -> u64 {
        let (reply_to, response) = oneshot::channel();

        let msg = ActorMessage::GetNextUID {
            reply_to,
        };

        let _ = self.sender.send(msg).await;
        response.await.expect("response channel failed")
    }

    fn new() -> Self {
        let (sender, receiver) = mpsc::channel(10);
        let mut actor = MyActor::new(receiver);
        tokio::spawn(async move { actor.run().await });
        Handle {
            sender,
        }
    }
}

//
// MyActor
//
struct MyActor {
    receiver: mpsc::Receiver<ActorMessage>,
    counter: u64,
}

#[async_trait]
impl Actor for MyActor {
    fn new(receiver: Receiver<ActorMessage>) -> Self {
        MyActor {
            receiver,
            counter: 0,
        }
    }
    fn handle_message(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::GetNextUID { reply_to } => {
                self.counter += 1;
                let _ = reply_to.send(self.counter);
            }
        }
    }

    // i dont want to have to copy this each time
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }
}

#[tokio::main]
async fn main() {
    let mut handle = Handle::new();
    for _ in 0..10 {
        let uid = handle.get_next_uid().await;
        println!("uid={}", uid);
    }
}

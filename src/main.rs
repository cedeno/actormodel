use tokio::sync::mpsc;
use tokio::sync::oneshot;
use async_trait::async_trait;

trait Actor {
    fn new() -> Self;
    fn handle_message(&mut self, msg: ActorMessage);
}

// message pump
// feeds messages to the Actor
struct ActorMessagePump<A>
    where A: Actor
{
    actor: A,
    receiver: mpsc::Receiver<ActorMessage>,
}

impl<A> ActorMessagePump<A>
    where A: Actor
{
    fn new(actor: A, receiver: mpsc::Receiver<ActorMessage>) -> Self {
        ActorMessagePump {
            actor,
            receiver,
        }
    }

    // while loop consuming messages from receiver
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.actor.handle_message(msg);
        }
    }
}


// don't like that you have to add all messages here
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
        let actor = MyActor::new();
        let mut pump = ActorMessagePump::new(actor, receiver);

        tokio::spawn(async move { pump.run().await });
        Handle {
            sender,
        }
    }
}

//
// MyActor
//
struct MyActor {
    counter: u64,
}

#[async_trait]
impl Actor for MyActor {
    fn new() -> Self {
        MyActor {
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
}

#[tokio::main]
async fn main() {
    let mut handle = Handle::new();
    for _ in 0..10 {
        let uid = handle.get_next_uid().await;
        println!("uid={}", uid);
    }
}

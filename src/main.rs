use tokio::sync::mpsc;
use tokio::sync::oneshot;

//trait ActorMessageHandler {
//  fn handle_message(&mut self, msg: ActorMessage);
//}

// actor
// receives messages
// handles them
// has internal state
struct Actor<T> {
    receiver: mpsc::Receiver<ActorMessage>,
    state: T,
}

impl<T> Actor<T> {
    fn new(receiver: mpsc::Receiver<ActorMessage>, initial_state: T) -> Self {
        Self {
            receiver,
            state: initial_state,
        }
    }
}

enum ActorMessage {
    GetNextUID {
        reply_to: oneshot::Sender<u64>,
    },
}

// runs the actor in a loop
async fn run_actor(mut actor: Actor<u64>) {
    while let Some(msg) = actor.receiver.recv().await {
        let _ = actor.handle_message(msg);
    }
}

impl Actor<u64>
{
    fn handle_message(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::GetNextUID { reply_to } => {
                self.state = self.state + 1;
                let _ = reply_to.send(self.state);
            }
        }
    }
}

// Handle
// creates actor and spawns it
// has sender to send message to actor
// has API to facilitate sending messages and getting results
struct ActorHandle {
    sender: mpsc::Sender<ActorMessage>,
}
impl ActorHandle {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel(10);
        let counter : u64 = 0;
        let actor = Actor::new(receiver, counter);
        tokio::spawn(run_actor(actor));
        ActorHandle {
            sender,
        }
    }

    async fn get_next_uid(&self) -> u64 {
        let (sender, receiver) = oneshot::channel();
        let msg = ActorMessage::GetNextUID {
            reply_to: sender,
        };
        let _ = self.sender.send(msg).await;
        receiver.await.expect("oneshot receiver failed")
    }
}

#[tokio::main]
async fn main() {
    let actor = ActorHandle::new();
    for _ in 0..10 {
        let uid = actor.get_next_uid().await;
        println!("next uid={}", uid);
    }
}

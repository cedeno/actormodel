use tokio::sync::mpsc;
use tokio::sync::oneshot;

struct MyActor {
    receiver: mpsc::Receiver<ActorMessage>,
    counter: u32,
}

impl MyActor {
    fn new(receiver: mpsc::Receiver<ActorMessage>) -> MyActor{
        MyActor {
            receiver,
            counter: 0,
        }
    }

    fn handle_message(&mut self, message: ActorMessage) {
        match message {
            ActorMessage::GetUniqueUID { respond_to} => {
                self.counter += 1;
                let _ = respond_to.send(self.counter);
            }
        }
    }
}
enum ActorMessage {
    GetUniqueUID {
        respond_to: oneshot::Sender<u32>,
    },
}

// runs the actor in a loop
async fn run_actor(mut actor: MyActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg);
    }
}

// handle to the actor,
// creates and starts the actor
// allows you to communicate with the actor,
#[derive(Clone)]
struct Handle {
    //actor: MyActor,
    sender: mpsc::Sender<ActorMessage>,
}
impl Handle {
    fn new() -> Handle {
        let (sender, receiver) = mpsc::channel(8);
        let actor = MyActor::new(receiver);
        tokio::spawn(run_actor(actor));
        Handle {
            sender,
        }
    }

    async fn get_unique_id(self) -> u32 {
        let (sender, receiver) = oneshot::channel();
        let msg = ActorMessage::GetUniqueUID {
            respond_to: sender,

        };
        let _ = self.sender.send(msg).await;
        let res = receiver.await.expect("could not read");
        res
    }
}

#[tokio::main]
async fn main() {
    let handle = Handle::new();
    for _ in 0..10 {
        let val = handle.clone().get_unique_id().await;
        println!("got val {}", val);
    }
}

use tokio::sync::mpsc;
use tokio::sync::oneshot;

trait Actor {
    type Message;
    fn new() -> Self;
    fn handle_message(&mut self, msg: Self::Message);
}

// feeds messages to the Actor
struct MessageReceiver<A>
    where A: Actor,
{
    actor: A,
    receiver: mpsc::Receiver<A::Message>,
}

impl<A> MessageReceiver<A>
    where A: Actor,
{
    fn new(actor: A, receiver: mpsc::Receiver<A::Message>) -> Self
    {
        MessageReceiver {
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

struct MessageDispatcher<A>
    where A: Actor
{
    sender: mpsc::Sender<A::Message>,
}
impl<A> MessageDispatcher<A>
    where A: Actor + Send + 'static
{
    fn new(actor: A) -> Self
        where <A as Actor>::Message: Send
    {
        let (sender, receiver) = mpsc::channel(10);
        let mut message_receiver = MessageReceiver::new(actor, receiver);
        tokio::spawn(async move { message_receiver.run().await });
        MessageDispatcher {
            sender,
        }
    }

    async fn send(&mut self, msg: A::Message) {
        let _ = self.sender.send(msg).await;
    }
}

//
// MyActor
//
struct MyActor {
    counter: u64,
}

impl Actor for MyActor {
    type Message = MyActorMessage;

    fn new() -> Self {
        MyActor {
            counter: 0,
        }
    }
    fn handle_message(&mut self, msg: Self::Message) {
        match msg {
            MyActorMessage::GetNextUID { reply_to } => {
                self.counter += 1;
                let _ = reply_to.send(self.counter);
            }
        }
    }
}

enum MyActorMessage {
    GetNextUID {
        reply_to: oneshot::Sender<u64>,
    },
}

//
// Handle
//
struct MyActorHandle {
    dispatcher: MessageDispatcher<MyActor>,
}

impl MyActorHandle {
    async fn get_next_uid(&mut self) -> u64 {
        let (reply_to, response) = oneshot::channel();

        let msg = MyActorMessage::GetNextUID {
            reply_to,
        };

        let _ = self.dispatcher.send(msg).await;
        response.await.expect("response channel failed")
    }

    fn new() -> Self {
        MyActorHandle {
            dispatcher: MessageDispatcher::new(MyActor::new())
        }
    }
}

#[tokio::main]
async fn main() {
    let mut handle = MyActorHandle::new();
    for _ in 0..10 {
        let uid = handle.get_next_uid().await;
        println!("uid={}", uid);
    }
}

use actormodel::{Actor,MessageDispatcher};
use tokio::sync::{oneshot};

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

#[derive(Clone)]
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

    let mut handle2 = handle.clone();
    for _ in 0..10 {
        let uid = handle2.get_next_uid().await;
        println!("uid={}", uid);
    }
}

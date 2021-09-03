use actormodel::{Actor,MessageDispatcher};
use tokio::sync::{oneshot};

struct MyActor {
    index: usize,
    db: Vec<u64>, // fake database
}

impl MyActor {
    fn new(db: &Vec<u64>) -> Self {
        MyActor {
            index: 0,
            db: db.clone(),
        }
    }
}

impl Actor for MyActor {
    type Message = MyActorMessage;

    fn handle_message(&mut self, msg: Self::Message) {
        match msg {
            MyActorMessage::GetNextUID { reply_to } => {
                if self.index >= self.db.len() {
                    self.index = 0;
                }
                let x = self.db.get(self.index).unwrap();
                self.index += 1;
                let _ = reply_to.send(*x);
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

    fn new(db: &Vec<u64>) -> Self {
        MyActorHandle {
            dispatcher: MessageDispatcher::new(MyActor::new(db))
        }
    }
}

#[tokio::main]
async fn main() {
    let v = vec![10,20,30,40,50];
    let mut handle = MyActorHandle::new(&v);
    for _ in 0..10 {
        let uid = handle.get_next_uid().await;
        println!("uid={}", uid);
    }
}

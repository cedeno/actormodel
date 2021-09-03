use tokio::sync::mpsc;
use async_trait::async_trait;

#[async_trait]
pub trait Actor {
    type Message;
    async fn handle_message(&mut self, msg: Self::Message);
}

// feeds messages to the Actor
pub struct MessageReceiver<A>
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
            self.actor.handle_message(msg).await;
        }
    }
}

pub struct MessageDispatcher<A>
    where A: Actor,
{
    sender: mpsc::Sender<A::Message>,
}
impl<A> MessageDispatcher<A>
    where A: Actor + Send + 'static
{
    pub fn new(actor: A) -> Self
        where <A as Actor>::Message: Send
    {
        let (sender, receiver) = mpsc::channel(10);
        let mut message_receiver = MessageReceiver::new(actor, receiver);
        tokio::spawn(async move { message_receiver.run().await });
        MessageDispatcher {
            sender,
        }
    }

    pub async fn send(&mut self, msg: A::Message) {
        let _ = self.sender.send(msg).await;
    }
}
impl<T> Clone for MessageDispatcher<T>
    where T: Actor + Send + 'static
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}
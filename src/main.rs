use crossbeam::{
    channel::{IntoIter, Receiver, Sender},
    unbounded,
};

use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex, MutexGuard},
};

pub struct PubSub {
    topics: Arc<Mutex<HashMap<String, Topic>>>,
}

#[derive(Clone)]
pub struct Topic {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl Topic {
    fn new(sender: &Sender<Message>, receiver: &Receiver<Message>) -> Topic {
        Topic {
            sender: sender.to_owned(),
            receiver: receiver.to_owned(),
        }
    }
    fn send_message(&mut self, body: &str) {
        self.sender
            .send(Message::new(body))
            .expect("error sending message to topic");
    }

    fn listen(self) -> IntoIter<Message> {
        self.receiver.into_iter()
    }
}

//Hmm bad to comment this as we can't drop topic anymore
// impl Drop for Topic {
//     fn drop(&mut self) {
//         drop(self.sender.to_owned());
//     }
// }

#[derive(Clone)]
pub struct Message {
    body: String,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.body)
    }
}

impl Message {
    fn new(body: &str) -> Message {
        Message {
            body: String::from(body),
        }
    }
}

type Topics<'a> = MutexGuard<'a, HashMap<String, Topic>>;

trait TopicsMethods {
    fn get(&mut self, channel: &str) -> &mut Topic;
}

impl<'a> TopicsMethods for Topics<'a> {
    fn get(&mut self, channel: &str) -> &mut Topic {
        return self.get_mut(channel).unwrap();
    }
}
pub trait PubSubTrait {
    fn new() -> PubSub;
    fn subscribe(&mut self, channel: &str);
    fn unsubscribe(&mut self, channel: &str);
    fn publish(&mut self, channel: &str, body: &str);
    fn get_topics(&mut self) -> Vec<String>;
    fn topics(&mut self) -> Topics;
    fn listen(&mut self, channel: &str) -> IntoIter<Message>;
}

//https://www.reddit.com/r/rust/comments/ay1t2i/cant_get_shared_hashmap_across_threads_to_work/
impl PubSubTrait for PubSub {
    // vec::IntoIter,
    fn get_topics(&mut self) -> Vec<String> {
        self.topics()
            .keys()
            .map(|topic| topic.clone())
            .collect::<Vec<String>>()
    }

    fn topics(&mut self) -> Topics {
        return self.topics.lock().unwrap();
    }

    fn new() -> PubSub {
        let topics = Arc::new(Mutex::new(HashMap::new()));
        PubSub { topics: topics }
    }

    fn subscribe(&mut self, channel: &str) {
        //TODO: launch a new thread
        let (s, r) = unbounded();
        //loop {
        //https://stackoverflow.com/questions/39045636/how-do-i-have-one-thread-that-deletes-from-a-hashmap-and-another-that-inserts
        self.topics()
            .insert(String::from(channel), Topic::new(&s, &r)); // lock the mutex, insert a value, unlock
                                                                // }
    }
    fn unsubscribe(&mut self, channel: &str) {
        //TODO: stop the thread we launched with subscribe method with join handle. Hmm but the drop sender should drop the thread but we'll see
        //loop {
        drop(self.topics().get(channel)); //drop sender
        self.topics().remove(channel);
        //}
    }

    fn publish(&mut self, channel: &str, body: &str) {
        // loop {
        self.topics().get(channel).send_message(body);
        //}
    }

    fn listen(&mut self, channel: &str) -> IntoIter<Message> {
        self.topics().get(channel).clone().listen()
    }
}

fn main() {
    let mut pubsub = PubSub::new();
    pubsub.subscribe("firsttopic");
    let topics = pubsub.get_topics();
    println!("topics : {:?}", topics);
    pubsub.publish("firsttopic", "hello from firsttopic");
    for message in pubsub.listen("firsttopic") {
        println!("{}", message);
    }
}

use tokio::io::{AsyncWrite, AsyncWriteExt, AsyncRead, AsyncReadExt};
use tokio::stream::{Stream, StreamExt, StreamMap};
use std::collections::HashMap;
use std::{io, sync::{Arc, Mutex, RwLock}};
use tokio::sync::mpsc;
use std::future::Future;
use std::pin::Pin;

use crate::{message, parser};

type Read = dyn AsyncRead + Send + Sync + Unpin;
type Write = dyn AsyncWrite + Send + Sync + Unpin;

type OwnedRead = Box<Read>;
type OwnedWrite = Box<Write>;
type SharedWrite = Arc<Mutex<OwnedWrite>>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerId(u64);

pub struct ConnectionManager<M: Message> {
    peer_writers: HashMap<PeerId, SharedWrite>,
    peer_readers: RwLock<StreamMap<PeerId, tokio::sync::mpsc::Receiver<M>>>,
    _phantom: std::marker::PhantomData<M>,
}

pub trait Message : Clone + Send + Sync + std::fmt::Debug {
    fn read(read: &mut (dyn AsyncRead + Send + Sync + Unpin))
        -> Box<dyn std::future::Future<Output=Result<Self, io::Error>> + Send + Sync + Unpin>;

    fn write(&self, write: &mut dyn AsyncWrite) -> Result<(), io::Error>;
}

impl<M> ConnectionManager<M>
    where M: Message {
    pub fn new() -> Self {
        ConnectionManager {
            peer_writers: HashMap::new(),
            peer_readers: RwLock::new(StreamMap::new()),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_connection(&mut self, id: PeerId, stream: impl AsyncRead + AsyncWrite + Send + Sync + 'static)
        where M: Message + 'static {
        let (incoming_tx, incoming_rx) = mpsc::channel(1024);
        let (read_half, write_half) = tokio::io::split(stream);

        let write: SharedWrite = Arc::new(Mutex::new(Box::new(write_half)));
        tokio::spawn(peer_connection_background::<M>(Box::new(read_half), write.clone(), incoming_tx));

        self.peer_writers.insert(id, write);
        self.peer_readers.write().unwrap().insert(id, incoming_rx);
    }


    /// Gets the next message from any peer.
    pub async fn next_message(&self) -> Result<(PeerId, M), io::Error> {
        match self.peer_readers.write().unwrap().next().await {
            Some(message) => Ok(message),
            None => unimplemented!("all peers hung up"),
        }
    }

    // pub fn incoming_messages(&self) -> Stream<Item=Result<(PeerId, M), io::Error>> {
    // }
}

struct IncomingMessages<'conn, M: Message> {
    connection_manager: &'conn ConnectionManager<M>,
    next_message_fut: Option<Pin<Box<dyn Future<Output=Result<(PeerId, M), io::Error>> + 'conn + Unpin>>>,
}

impl<'conn, M: Message> IncomingMessages<'conn, M> {
    pub fn new(connection_manager: &'conn ConnectionManager<M>) -> Self {
        IncomingMessages { connection_manager, next_message_fut: None }
    }
}

impl<'conn, M: Message> Stream for IncomingMessages<'conn, M> {
    type Item = Result<M, io::Error>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context)
        -> std::task::Poll<Option<Result<M, std::io::Error>>> {
        if let None = self.next_message_fut {
            self.next_message_fut = Some(Pin::new(Box::new(self.connection_manager.next_message())));
        }

        let f = (self.next_message_fut.as_mut().unwrap().deref_mut());
        match Future::poll(f, cx) {
        }

        unimplemented!();
    }
}

async fn peer_connection_background<M>(mut read_half: OwnedRead,
                                       write: SharedWrite,
                                       mut incoming_tx: mpsc::Sender<M>)
    -> Result<(), String>
    where M: Message {
    loop {
        println!("loop");
        let message = M::read(&mut read_half).await.map_err(|e| e.to_string())?;
        println!("[info] received peer message: {:?}", message);

        incoming_tx.send(message).await.unwrap();
    }
}


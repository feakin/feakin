use std::collections::{HashMap, HashSet};
use std::io;
use std::ops::Range;
use std::sync::{Arc, Mutex};

use actix::Actor;
use actix_web_actors::ws;
use diamond_types::{LocalVersion, Time};
use log::error;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::UnboundedSender;

use crate::living::live_coding::LiveCoding;
use crate::living::random_name;
use crate::model::{Command, ConnId, FkResponse, id_generator, RemoteVersion, RoomId};

#[derive(Debug)]
pub struct LivingEditServer {
  sessions: HashMap<ConnId, UnboundedSender<FkResponse>>,

  rooms: HashMap<RoomId, HashSet<ConnId>>,

  codings: HashMap<RoomId, Arc<Mutex<LiveCoding>>>,

  versions: HashMap<RoomId, Time>,

  agents: HashMap<ConnId, String>,

  cmd_rx: mpsc::UnboundedReceiver<Command>,
}

#[derive(Debug, Clone)]
pub struct LiveEditServerHandle {
  cmd_tx: UnboundedSender<Command>,
}

impl LiveEditServerHandle {
  pub async fn connect(&self) -> ConnId {
    let (res_tx, res_rx) = oneshot::channel();

    // unwrap: chat server should not have been dropped
    self.cmd_tx
      .send(Command::Connect { res_tx })
      .unwrap();

    // unwrap: chat server does not drop out response channel
    res_rx.await.unwrap()
  }

  pub fn disconnect(&self, conn: ConnId) {
    self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
  }

  pub(crate) async fn create(&self, conn: ConnId, room_name: impl Into<String>, agent_name: impl Into<String>, content: impl Into<String>, conn_tx: &UnboundedSender<FkResponse>) -> FkResponse {
    let (res_tx, res_rx) = oneshot::channel();

    self.cmd_tx
      .send(Command::Create {
        conn,
        room_id: room_name.into(),
        content: content.into(),
        agent_name: agent_name.into(),
        conn_tx: conn_tx.clone(),
        res_tx,
      })
      .unwrap();

    res_rx.await.unwrap()
  }

  pub(crate) async fn insert(&self, conn: ConnId, content: impl Into<String>, pos: usize, room_id: RoomId) -> FkResponse {
    let (res_tx, res_rx) = oneshot::channel();

    self.cmd_tx
      .send(Command::Insert {
        conn,
        content: content.into(),
        pos,
        room_id: room_id.to_string(),
        res_tx,
      })
      .unwrap();

    res_rx.await.unwrap()
  }

  pub(crate) async fn join(&self, conn: ConnId, room_id: impl Into<String>, agent_name: impl Into<String>) -> FkResponse {
    let (res_tx, res_rx) = oneshot::channel();

    self.cmd_tx
      .send(Command::Join {
        conn,
        room_id: room_id.into(),
        agent_name: agent_name.into(),
        res_tx,
      })
      .unwrap();

    res_rx.await.unwrap()
  }

  pub(crate) async fn delete(&self, conn: ConnId, room_id: RoomId, range: Range<usize>) -> FkResponse {
    let (res_tx, res_rx) = oneshot::channel();

    self.cmd_tx
      .send(Command::Delete {
        conn,
        room_id: room_id.to_string(),
        range,
        res_tx,
      })
      .unwrap();

    res_rx.await.unwrap()
  }

  pub async fn list_rooms(&self) -> Vec<String> {
    let (res_tx, res_rx) = oneshot::channel();
    self.cmd_tx.send(Command::List { res_tx }).unwrap();
    res_rx.await.unwrap()
  }

  pub async fn list_agent(&self) -> Vec<String> {
    let (res_tx, res_rx) = oneshot::channel();
    self.cmd_tx.send(Command::ListAgents { res_tx }).unwrap();
    res_rx.await.unwrap()
  }

  pub async fn content(&self, room_id: RoomId) -> String {
    let (res_tx, res_rx) = oneshot::channel();
    self.cmd_tx.send(Command::Content { room_id, res_tx }).unwrap();
    res_rx.await.unwrap()
  }
}

impl Actor for LivingEditServer {
  type Context = ws::WebsocketContext<Self>;
}

impl LivingEditServer {
  pub fn new() -> (Self, LiveEditServerHandle) {
    let rooms = HashMap::with_capacity(4);
    let codings = HashMap::with_capacity(100);

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

    (
      Self {
        sessions: HashMap::new(),
        rooms,
        versions: Default::default(),
        codings,
        agents: Default::default(),
        cmd_rx,
      },
      LiveEditServerHandle {
        cmd_tx
      },
    )
  }

  pub async fn run(mut self) -> io::Result<()> {
    while let Some(cmd) = self.cmd_rx.recv().await {
      match cmd {
        Command::Connect { res_tx } => {
          let conn_id = self.connect().await;
          let _ = res_tx.send(conn_id);
        }
        Command::Disconnect { conn } => {
          self.disconnect(conn).await;
        }
        Command::Create { conn, room_id, content, agent_name, conn_tx, res_tx } => {
          let room_id = self.create(room_id, conn, content, conn_tx, agent_name).await;
          let _ = res_tx.send(FkResponse::create(room_id));
        }
        Command::Join { conn, room_id, agent_name, res_tx } => {
          let output = self.join(conn, room_id, agent_name).await;
          let _ = res_tx.send(output);
        }
        Command::Insert { conn, content, pos, room_id, res_tx } => {
          let opt_version = self.insert(conn, room_id.clone(), content, pos).await;
          self.broadcast_patch(room_id, conn).await;
          let _ = res_tx.send(FkResponse::insert(opt_version));
        }
        Command::Delete { conn, room_id, range, res_tx } => {
          let opt_version = self.delete(conn, room_id.clone(), range).await;
          self.broadcast_patch(room_id, conn).await;
          let _ = res_tx.send(FkResponse::delete(opt_version));
        }
        Command::List { res_tx } => {
          let _ = res_tx.send(self.list_rooms());
        }
        Command::ListAgents { res_tx } => {
          let _ = res_tx.send(self.list_agents());
        }
        Command::Content { room_id, res_tx } => {
          let content = self.content(room_id).await;
          let _ = res_tx.send(content.unwrap_or("".to_string()));
        }
      }
    }

    Ok(())
  }

  async fn broadcast_patch(&self, room: RoomId, skip: ConnId) {
    if let Some(sessions) = self.rooms.get(&room) {
      let remote_version: RemoteVersion;
      let patch: Vec<u8>;

      if let Some(coding) = self.codings.get(&room) {
        let coding = coding.lock().unwrap();
        remote_version = coding.remote_version();
        patch = coding.patch_from_version();
      } else {
        return;
      }

      for conn_id in sessions {
        if *conn_id != skip {
          if let Some(tx) = self.sessions.get(conn_id) {
            let _ = tx.send(FkResponse::upstream(remote_version.clone(), patch.clone()));
          }
        }
      }
    }
  }

  async fn send_system_message(&self, room: &str, skip: ConnId, msg: impl Into<String>) {
    if let Some(sessions) = self.rooms.get(room) {
      let msg = msg.into();

      for conn_id in sessions {
        if *conn_id != skip {
          if let Some(tx) = self.sessions.get(conn_id) {
            let _ = tx.send(FkResponse::system_message(msg.clone()));
          }
        }
      }
    }
  }

  fn list_rooms(&mut self) -> Vec<String> {
    self.rooms.keys().cloned().collect()
  }

  fn list_agents(&mut self) -> Vec<String> {
    self.agents.values().cloned().collect()
  }

  async fn connect(&mut self) -> ConnId {
    id_generator()
  }

  async fn create(&mut self, room_id: RoomId, conn: ConnId, content: String, conn_tx: UnboundedSender<FkResponse>, agent_name: String) -> String {
    self.sessions.insert(conn, conn_tx);

    self.agents.insert(conn, agent_name);

    self.rooms
      .entry(room_id.clone())
      .or_insert_with(HashSet::new)
      .insert(conn);

    let agent_name = &random_name();

    let mut coding = LiveCoding::new(&agent_name);
    let version = coding.create(agent_name, &content);

    self.versions.insert(room_id.clone(), version);
    self.codings.insert(room_id.clone(), Arc::new(Mutex::new(coding)));

    room_id.clone()
  }

  async fn insert(&mut self, conn: ConnId, room_id: RoomId, content: String, pos: usize) -> Option<String> {
    match self.codings.get(&*room_id) {
      Some(coding) => {
        let mut mutex_coding = coding.lock().unwrap();
        let agent = conn.to_string();

        let version = mutex_coding.insert(&*agent, pos, &content);
        self.versions.insert(room_id, version);

        let content = mutex_coding.content().clone();
        Some(content)
      }
      None => {
        error!("room {:?} not found", room_id);
        None
      }
    }
  }

  async fn delete(&mut self, conn: ConnId, room_id: RoomId, range: Range<usize>) -> Option<String> {
    match self.codings.get(&*room_id) {
      Some(coding) => {
        let mut mutex_coding = coding.lock().unwrap();
        let agent = conn.to_string();

        let version = mutex_coding.delete(&*agent, range);
        self.versions.insert(room_id, version);

        Some("".to_string())
      }
      None => {
        error!("room {:?} not found", room_id);
        None
      }
    }
  }

  async fn content(&self, room_id: RoomId) -> Option<String> {
    match self.codings.get(&*room_id) {
      Some(coding) => {
        let mutex_coding = coding.lock().unwrap();
        let content = mutex_coding.content().clone();
        Some(content)
      }
      None => {
        error!("content {:?} not found", room_id);
        None
      }
    }
  }

  async fn join(&mut self, conn: ConnId, room_id: RoomId, agent_name: String) -> FkResponse {
    self.rooms
      .entry(room_id.clone())
      .or_insert_with(HashSet::new)
      .insert(conn);

    self.agents.insert(conn, agent_name.clone());

    return match self.codings.get_mut(&*room_id.clone()) {
      Some(coding) => {
        let mut mutex_coding = coding.lock().unwrap();

        let agent_id = mutex_coding.join(&*agent_name);
        let data = mutex_coding.base_version();

        FkResponse::join(room_id.clone(), data, agent_id.to_string(), None, agent_name)
      }
      None => {
        let error_msg = format!("room {:?} not found", room_id);
        FkResponse::join("".to_string(), vec![], "".to_string(), Some(error_msg), agent_name)
      }
    };
  }

  async fn disconnect(&mut self, conn_id: ConnId) {
    let mut rooms: Vec<String> = Vec::new();

    if self.sessions.remove(&conn_id).is_some() {
      for (name, sessions) in &mut self.rooms {
        if sessions.remove(&conn_id) {
          rooms.push(name.to_owned());
        }
      }
    }

    self.agents.remove(&conn_id);

    // send message to other users
    for room in rooms {
      self.send_system_message(&room, 0, "Someone disconnected")
        .await;
    }
  }
}

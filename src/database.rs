use worker::wasm_bindgen_futures::spawn_local;
use worker::Env;

use crate::{
    ticket::{DefId, TicketDef, TicketId, UserTicket},
    user::{User, UserId},
};

enum Task {
    GetUser {
        username: String,
        result: oneshot::Sender<Option<User>>,
    },
    InsertUser {
        username: String,
        password: String,
        result: oneshot::Sender<()>,
    },
    GetUserTickets {
        id: UserId,
        result: oneshot::Sender<Vec<UserTicket>>,
    },
    GetTicket {
        id: TicketId,
        result: oneshot::Sender<Option<UserTicket>>,
    },
    UpdateTicketUsages {
        id: TicketId,
        usages: u64,
        result: oneshot::Sender<()>,
    },
    InsertTicket {
        def: DefId,
        id: UserId,
        qr: String,
        result: oneshot::Sender<()>,
    },
    GetTicketDefs {
        result: oneshot::Sender<Vec<TicketDef>>,
    },
    Close,
}

#[derive(Clone)]
pub struct DatabaseConn(async_channel::Sender<Task>);

impl DatabaseConn {
    pub async fn get_user(&self, username: String) -> Option<User> {
        let (result, user) = oneshot::channel();
        let task = Task::GetUser { username, result };
        self.send(task).await;
        user.await.unwrap()
    }

    pub async fn insert_user(&self, username: String, password: String) {
        let (result, finished) = oneshot::channel();
        let task = Task::InsertUser {
            username,
            password,
            result,
        };
        self.send(task).await;
        finished.await.unwrap();
    }

    pub async fn get_user_tickets(&self, id: UserId) -> Vec<UserTicket> {
        let (result, user) = oneshot::channel();
        let task = Task::GetUserTickets { id, result };
        self.send(task).await;
        user.await.unwrap()
    }

    pub async fn get_ticket(&self, id: TicketId) -> Option<UserTicket> {
        let (result, user) = oneshot::channel();
        let task = Task::GetTicket { id, result };
        self.send(task).await;
        user.await.unwrap()
    }

    pub async fn update_ticket_usages(&self, id: TicketId, usages: u64) {
        let (result, user) = oneshot::channel();
        let task = Task::UpdateTicketUsages { id, usages, result };
        self.send(task).await;
        user.await.unwrap();
    }

    pub async fn insert_tickets_for(&self, id: UserId, def: DefId, qr: String) {
        let (result, finished) = oneshot::channel();
        let task = Task::InsertTicket {
            def,
            id,
            qr,
            result,
        };
        self.send(task).await;
        finished.await.unwrap();
    }

    pub async fn get_ticket_defs(&self) -> Vec<TicketDef> {
        let (result, finished) = oneshot::channel();
        let task = Task::GetTicketDefs { result };
        self.send(task).await;
        finished.await.unwrap()
    }

    pub async fn close(self) {
        self.send(Task::Close).await;
    }

    async fn send(&self, task: Task) {
        self.0.send(task).await.unwrap();
    }
}

pub(crate) fn database(env: Env) -> DatabaseConn {
    let (tx, rx) = async_channel::bounded::<Task>(16);

    spawn_local(async move {
        let db = env.d1("database").unwrap();

        while let Ok(task) = rx.recv().await {
            if let Task::Close = &task {
                return;
            }

            let sql = match &task {
                Task::GetUser { .. } => "SELECT * FROM users WHERE username = ?1",
                Task::InsertUser { .. } => {
                    "INSERT INTO users (username, password_hash) VALUES (?1, ?2)"
                }
                Task::GetUserTickets { .. } => "SELECT * FROM user_tickets WHERE user = ?1",
                Task::GetTicket { .. } => "SELECT * from user_tickets WHERE id = ?1",
                Task::UpdateTicketUsages { .. } => {
                    "UPDATE user_tickets SET usages = ?1 WHERE id = ?2"
                }
                Task::InsertTicket { .. } => {
                    "INSERT INTO user_tickets (user, def, qr) VALUES (?1, ?2, ?3)"
                }
                Task::GetTicketDefs { .. } => "SELECT * FROM ticket_defs",
                Task::Close => unreachable!(),
            };

            let mut query = db.prepare(sql);

            query = match &task {
                Task::GetUser { username, .. } => query.bind(&[username.into()]).unwrap(),
                Task::InsertUser {
                    username, password, ..
                } => query
                    .bind(&[username.as_str().into(), password.as_str().into()])
                    .unwrap(),
                Task::GetUserTickets { id: UserId(id), .. } => {
                    query.bind(&[(*id as f64).into()]).unwrap()
                }
                Task::GetTicket {
                    id: TicketId(id), ..
                } => query.bind(&[(*id as f64).into()]).unwrap(),
                Task::UpdateTicketUsages {
                    id: TicketId(id),
                    usages,
                    ..
                } => query
                    .bind(&[(*usages as f64).into(), (*id as f64).into()])
                    .unwrap(),
                Task::InsertTicket {
                    id: UserId(id),
                    def: DefId(def),
                    qr,
                    ..
                } => query
                    .bind(&[(*id as f64).into(), (*def as f64).into(), qr.into()])
                    .unwrap(),
                Task::GetTicketDefs { .. } => query,
                Task::Close => unreachable!(),
            };

            match task {
                Task::GetUser { result, .. } => {
                    let res = query.first(None).await.unwrap();
                    result.send(res).unwrap();
                }
                Task::InsertUser { result, .. } => {
                    query.run().await.unwrap();
                    result.send(()).unwrap();
                }
                Task::GetUserTickets { result, .. } => {
                    let res = query.all().await.unwrap();
                    result.send(res.results().unwrap()).unwrap();
                }
                Task::GetTicket { result, .. } => {
                    let res = query.first(None).await.unwrap();
                    result.send(res).unwrap();
                }
                Task::UpdateTicketUsages { result, .. } => {
                    query.run().await.unwrap();
                    result.send(()).unwrap();
                }
                Task::InsertTicket { result, .. } => {
                    query.run().await.unwrap();
                    result.send(()).unwrap();
                }
                Task::GetTicketDefs { result } => {
                    let res = query.all().await.unwrap();
                    result.send(res.results().unwrap()).unwrap();
                }
                Task::Close => unreachable!(),
            }
        }
    });

    DatabaseConn(tx)
}

use crate::models::user::User;

use worker::{wasm_bindgen_futures, Env};

enum Task {
    GetUser {
        key: String,
        result: oneshot::Sender<Option<User>>,
    },
    PutUser {
        key: String,
        user: User,
        result: oneshot::Sender<()>,
    },
    RemoveUser {
        key: String,
        result: oneshot::Sender<()>,
    },
    Close,
}

#[derive(Clone)]
pub struct Sessions(async_channel::Sender<Task>);

impl Sessions {
    pub async fn get(&self, key: String) -> Option<User> {
        let (result, finished) = oneshot::channel();
        self.send(Task::GetUser { key, result }).await;
        finished.await.unwrap()
    }

    pub async fn put(&self, key: String, user: User) {
        let (result, finished) = oneshot::channel();
        self.send(Task::PutUser { key, user, result }).await;
        finished.await.unwrap();
    }

    pub async fn remove(&self, key: String) {
        let (result, finished) = oneshot::channel();
        self.send(Task::RemoveUser { key, result }).await;
        finished.await.unwrap();
    }

    pub async fn close(self) {
        self.send(Task::Close).await;
    }

    async fn send(&self, task: Task) {
        self.0.send(task).await.unwrap();
    }
}

pub(crate) fn sessions(env: Env) -> Sessions {
    let (tx, rx) = async_channel::bounded::<Task>(16);

    wasm_bindgen_futures::spawn_local(async move {
        let sessions = env.kv("sessions").unwrap();

        while let Ok(task) = rx.recv().await {
            match task {
                Task::Close => return,
                Task::GetUser { key, result } => {
                    let get = sessions.get(&key);
                    let user = get.json().await.unwrap();
                    result.send(user).unwrap();
                }
                Task::PutUser { key, user, result } => {
                    let put = sessions.put(&key, user);
                    put.unwrap().execute().await.unwrap();
                    result.send(()).unwrap();
                }
                Task::RemoveUser { key, result } => {
                    let rm = sessions.delete(&key);
                    rm.await.unwrap();
                    result.send(()).unwrap();
                }
            }
        }
    });

    Sessions(tx)
}

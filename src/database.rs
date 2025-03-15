use js_sys::wasm_bindgen::JsValue;
use worker::{wasm_bindgen_futures::spawn_local, D1PreparedStatement};
use worker::{D1Database, Env};

#[repr(transparent)]
pub struct Binding(JsValue);

impl From<&str> for Binding {
    fn from(value: &str) -> Self {
        Binding(value.into())
    }
}

impl From<u32> for Binding {
    fn from(value: u32) -> Self {
        Binding(value.into())
    }
}

pub trait Query {
    type Result: for<'de> serde::Deserialize<'de>;

    fn query(&self) -> &'static str;
    fn bindings(&self) -> Vec<Binding>;
}

#[derive(Clone)]
pub struct DatabaseConn(async_channel::Sender<Command>);

impl DatabaseConn {
    pub async fn query<T: Query + Send + Sync + 'static>(&self, query: T) -> Vec<T::Result> {
        let (tx, json) = oneshot::channel();

        self.0
            .send(Command::Query(Box::from(query), tx))
            .await
            .unwrap();

        json.await
            .unwrap()
            .into_iter()
            .map(|obj| serde_json::from_str(&obj).unwrap())
            .collect()
    }

    pub async fn query_one<T: Query + Send + Sync + 'static>(&self, query: T) -> Option<T::Result> {
        let results = self.query(query).await;
        match results.len() {
            0 => None,
            1 => results.into_iter().next(),
            _ => panic!("Multiple results"),
        }
    }

    pub async fn run<T: Query<Result = ()> + Send + Sync + 'static>(&self, query: T) {
        let _ = self.query(query).await;
    }

    pub async fn close(self) {
        self.0.send(Command::Close).await.unwrap();
    }
}

trait Statement {
    fn statement(&self, db: &D1Database) -> worker::Result<D1PreparedStatement>;
}

impl<T> Statement for T
where
    T: Query,
{
    #[inline(always)]
    fn statement(&self, db: &D1Database) -> worker::Result<D1PreparedStatement> {
        db.prepare(self.query()).bind(
            &self
                .bindings()
                .into_iter()
                .map(|Binding(value)| value)
                .collect::<Vec<_>>(),
        )
    }
}

enum Command {
    Query(
        Box<dyn Statement + Send + Sync>,
        oneshot::Sender<Vec<String>>,
    ),
    Close,
}

pub fn database(env: Env) -> DatabaseConn {
    let (tx, rx) = async_channel::bounded::<Command>(16);

    /// Convert a [`JsValue`] to a Json String.
    ///
    /// From: `gloo_utils`
    fn to_json(value: JsValue) -> String {
        // Turns out `JSON.stringify(undefined) === undefined`, so if
        // we're passed `undefined` reinterpret it as `null` for JSON
        // purposes.
        if value.is_undefined() {
            String::from("null")
        } else {
            js_sys::JSON::stringify(&value).map(String::from).unwrap()
        }
    }

    spawn_local(async move {
        let db = env.d1("database").unwrap();

        while let Ok(cmd) = rx.recv().await {
            match cmd {
                Command::Close => return,
                Command::Query(query, res) => {
                    let query = query.statement(&db).expect("failed to bind");
                    let results = query.raw_js_value().await.unwrap();

                    res.send(results.into_iter().map(to_json).collect())
                        .unwrap();
                }
            }
        }
    });

    DatabaseConn(tx)
}

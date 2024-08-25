use crate::errors::AppResult;
use fe2o3_amqp::connection::ConnectionHandle;
use fe2o3_amqp::sasl_profile::SaslProfile;
use fe2o3_amqp::session::SessionHandle;
use fe2o3_amqp::{Connection, Sender, Session};
use tokio::sync::Mutex;

pub struct State {
    pub connection: ConnectionHandle<()>,
    pub session: SessionHandle<()>,
    pub broadcast_sender: Mutex<Sender>,
}

impl State {
    pub async fn init() -> AppResult<Self> {
        // TODO: get credentials from env
        let profile = SaslProfile::Plain {
            username: "artemis".to_string(),
            password: "artemis".to_string(),
        };

        let mut connection = Connection::builder()
            .container_id("connection-1")
            .sasl_profile(profile)
            .open("amqp://localhost:5672")
            .await?;

        let mut session = Session::begin(&mut connection).await?;

        let sender = Sender::attach(
            &mut session,
            "broadcast",
            "muuzika.broadcast",
        ).await?;

        Ok(Self {
            connection,
            session,
            broadcast_sender: Mutex::new(sender),
        })
    }
}



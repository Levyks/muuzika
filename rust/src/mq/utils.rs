use std::fmt::Debug;
use std::future::Future;

use fe2o3_amqp::{Connection, Receiver, Session};
use fe2o3_amqp::link::delivery::DeliveryInfo;
use fe2o3_amqp::link::receiver::TerminalDeliveryState;
use fe2o3_amqp::link::RecvError;
use fe2o3_amqp::sasl_profile::SaslProfile;
use fe2o3_amqp::types::messaging::{FromBody, Message, Modified};
use log::{error, info, trace};
use tokio::select;
use tokio::sync::mpsc;
use url::Url;

use crate::errors::{MuuzikaError, MuuzikaResult};
use crate::state::{read_env_var_or_else, read_required_env_var_parse};

pub struct MqParameters {
    pub mq_url: Url,
    pub mq_username: String,
    pub mq_password: String,
    pub mq_container_id: String,
}

impl MqParameters {
    pub fn from_env() -> Self {
        Self {
            mq_url: read_env_var_or_else("MQ_URL", || Url::parse("amqp://localhost").unwrap()),
            mq_username: read_env_var_or_else("MQ_USERNAME", || "artemis".to_string()),
            mq_password: read_env_var_or_else("MQ_PASSWORD", || "artemis".to_string()),
            mq_container_id: read_required_env_var_parse("MQ_CONTAINER_ID"),
        }
    }
}

fn spawn_handler_thread<Fut>(
    tx: &mpsc::UnboundedSender<(DeliveryInfo, TerminalDeliveryState)>,
    info: DeliveryInfo,
    future: Fut,
) where
    Fut: Future<Output = MuuzikaResult<TerminalDeliveryState>> + Send + 'static,
{
    let tx = tx.clone();
    tokio::spawn(async move {
        let state = future.await.unwrap_or_else(|err| {
            error!("Error processing message: {:?}", err);
            (Modified {
                delivery_failed: Some(true),
                undeliverable_here: None,
                message_annotations: None,
            })
            .into()
        });

        tx.send((info, state)).unwrap_or_else(|err| {
            error!("Error sending delivery state: {:?}", err);
        });
    });
}

pub async fn listen<T, Cb, Fut>(
    mq_parameters: MqParameters,
    address: String,
    callback: Cb,
) -> MuuzikaResult<()>
where
    for<'de> T: FromBody<'de> + Send + Debug,
    Cb: Fn(Message<T>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = MuuzikaResult<TerminalDeliveryState>> + Send + 'static,
{
    let profile = SaslProfile::Plain {
        username: mq_parameters.mq_username,
        password: mq_parameters.mq_password,
    };

    let mut connection = Connection::builder()
        .container_id(mq_parameters.mq_container_id)
        .sasl_profile(profile)
        .open(mq_parameters.mq_url)
        .await?;

    info!("Connected to the message broker");

    let mut session = Session::begin(&mut connection).await?;
    info!("Session started");

    let mut receiver = Receiver::attach(&mut session, "", address).await?;
    info!("Receiver attached");

    let (tx, mut rx) = mpsc::unbounded_channel::<(DeliveryInfo, TerminalDeliveryState)>();

    loop {
        select! {
            received = receiver.recv::<T>() => {
                match received {
                    Ok(delivery) => {
                        let (info, message) = delivery.into_parts();
                        trace!("Received message, delivery id: {}, body: {:?}", info.delivery_id(), message.body);
                        spawn_handler_thread(&tx, info, callback(message));
                    }
                    Err(RecvError::LinkStateError(err)) => {
                        return Err(err.into());
                    }
                    Err(RecvError::MessageDecode(err)) => {
                        trace!("Error decoding message: {:?}", err);
                        receiver.reject(err.info, None).await?;
                    },
                    Err(err) => {
                        error!("Error receiving message: {:?}", err);
                    }
                }
            }
            Some((info, state)) = rx.recv() => {
                trace!("Disposing of delivery with id: {}, state: {:?}", info.delivery_id(), state);
                receiver.dispose(info, state).await?;
            }
        }
    }
}

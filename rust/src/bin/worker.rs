use std::env;

use fe2o3_amqp::{Connection, Receiver, Sender, Session};
use fe2o3_amqp::link::delivery::Delivery;
use fe2o3_amqp::link::delivery::DeliveryInfo;
use fe2o3_amqp::link::receiver::TerminalDeliveryState;
use fe2o3_amqp::link::RecvError;
use fe2o3_amqp::sasl_profile::SaslProfile;
use fe2o3_amqp::types::definitions::ErrorCondition;
use fe2o3_amqp::types::messaging::{Body, DeliveryState, Message, Modified};
use fe2o3_amqp::types::primitives::Value;
use log::{error, info, trace};
use serde_amqp::primitives::OrderedMap;
use tokio::select;
use tokio::sync::mpsc;

use muuzika::mq::handler::{handle_mq_message, Job, process_job};
use muuzika::state::EnvParameters;

#[tokio::main]
pub async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init_timed();

    let env = EnvParameters::from_env();

    let profile = SaslProfile::Plain {
        username: env.mq_username,
        password: env.mq_password,
    };

    let mut connection = Connection::builder()
        .container_id(format!(
            "worker-{}-{}",
            env.snowflake_machine_id, env.snowflake_node_id
        ))
        .sasl_profile(profile)
        .open(env.mq_url)
        .await
        .expect("Failed to connect to the message broker");

    info!("Connected to the message broker");

    let mut session = Session::begin(&mut connection)
        .await
        .expect("Failed to start a session with the message broker");
    info!("Session started");

    let mut receiver = Receiver::attach(&mut session, "", env.mq_jobs_queue)
        .await
        .expect("Failed to attach a receiver to the message broker");
    info!("Receiver attached");

    let (tx, mut rx) = mpsc::unbounded_channel::<(DeliveryInfo, TerminalDeliveryState)>();

    loop {
        select! {
            received = receiver.recv::<Body<Value>>() => {
                match received {
                    Ok(delivery) => {
                        let (info, message) = delivery.into_parts();
                        trace!("Received message, delivery id: {}, body: {:?}", info.delivery_id(), message.body);
                        let mut fields = OrderedMap::new();
                        println!("fields: {:?}", fields);
                        let modified = Modified {
                            delivery_failed: Some(true),
                            undeliverable_here: None,
                            message_annotations: Some(fields),
                        };
                        receiver.modify(info, modified).await.unwrap();
                        /*
                        if let Err((info, error)) = handle_mq_message(delivery, &tx) {
                            if let Err(e) = receiver.reject(info, error).await {
                                error!("Error rejecting message: {:?}", e);
                                break;
                            }
                        }

                         */
                    }
                    Err(e) => {
                        error!("Error receiving message: {:?}", e);
                        if let RecvError::LinkStateError(_) = e {
                            break;
                        }
                    }
                }
            }
            Some((info, state)) = rx.recv() => {
                trace!("Disposing of delivery with id: {}, state: {:?}", info.delivery_id(), state);
                if let Err(e)  = receiver.dispose(info, state).await {
                    error!("Error disposing message: {:?}", e);
                    break;
                }
            }
            else => break,
        }
    }
}

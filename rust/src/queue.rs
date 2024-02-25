use fe2o3_amqp::{Connection, Receiver, Session};
use fe2o3_amqp::link::delivery::DeliveryInfo;
use fe2o3_amqp::link::receiver::TerminalDeliveryState;
use fe2o3_amqp::sasl_profile::SaslProfile;
use fe2o3_amqp::types::messaging::Accepted;
use fe2o3_amqp::types::primitives::Value;
use tokio::select;
use tokio::sync::mpsc;

pub async fn sla_mano() {
    let profile = SaslProfile::Plain {
        username: "artemis".to_string(),
        password: "artemis".to_string(),
    };

    let mut connection = Connection::builder()
        .container_id("container-1")
        .sasl_profile(profile)
        .open("amqp://localhost:5672")
        .await
        .unwrap();

    println!("Connected");

    let mut session = Session::begin(&mut connection).await.unwrap();

    println!("Session started");

    let mut receiver = Receiver::attach(&mut session, "", "jobs").await.unwrap();

    println!("Receiver attached");
    let (sender, _) = mpsc::channel::<String>(100);

    let (tx, mut rx) = mpsc::unbounded_channel::<(DeliveryInfo, TerminalDeliveryState)>();

    loop {
        select! {
            Ok(message) = receiver.recv::<Value>() => {
                println!("Received message: {:?}", message);
                let tx = tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    println!("Message processed");
                    tx.send((message.into(), (Accepted {}).into())).unwrap();
                });
            }
            Some((info, state)) = rx.recv() => {
                receiver.dispose(info, state).await.unwrap();
                println!("Message disposed");
            }
            else => break,
        }

        /*
        let message: Delivery<Value> = receiver.recv().await.unwrap();
        println!("Received message: {:?}", message);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            println!("Message processed");
            // somehow accept?
            // this doesn't work as receiver is used as mutable in the .recv() call
            receiver.accept(&message).await.unwrap();
            println!("Message accepted");
        });

         */
    }
}

pub fn start_sla_mano() {
    tokio::spawn(sla_mano());
}

use amqprs::callbacks::{DefaultChannelCallback, DefaultConnectionCallback};
use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, Channel, ExchangeDeclareArguments,
    QueueBindArguments, QueueDeclareArguments,
};
use amqprs::{
    connection::{Connection, OpenConnectionArguments},
    BasicProperties, DELIVERY_MODE_PERSISTENT,
};
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Notify;
use tokio::{self, task};

async fn add_new_queue(
    channel: &Channel,
    queue_name: String,
    exchange_name: String,
    routing_key: String,
) {
    let queue_args = QueueDeclareArguments::default()
        .queue(queue_name.clone())
        .durable(true)
        .finish();
    channel.queue_declare(queue_args).await.unwrap().unwrap();
    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            &exchange_name,
            &routing_key,
        ))
        .await
        .unwrap();
}

async fn consume_messages(channel: &Channel, queue_name: &String) {
    let consumer_args = BasicConsumeArguments::new(queue_name, queue_name)
        .manual_ack(false)
        .finish();
    let (_ctag, mut rx) = channel.basic_consume_rx(consumer_args).await.unwrap();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Some(payload) = msg.content {
                println!("\r{:?}", str::from_utf8(&payload).unwrap());
                print!("Enter service name: ");
                std::io::stdout().flush().unwrap();
            }
        }
    });
}

#[tokio::main]
async fn main() {
    let cli_args = env::args().collect::<Vec<_>>();
    if cli_args.len() != 2 {
        println!("Usage: agency <agency_name>");
        return;
    }
    let agency_name = cli_args[1].clone();

    let args = OpenConnectionArguments::new("localhost", 5672, "guest", "guest");
    let connection = Connection::open(&args).await.unwrap();

    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();

    let channel = connection.open_channel(None).await.unwrap();
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    let exchange_name = "exchange";
    let exchange_args = ExchangeDeclareArguments::new(exchange_name, "topic")
        .durable(true)
        .finish();
    channel.exchange_declare(exchange_args).await.unwrap();

    let reply_queue_name = format!("{}_reply_queue", agency_name);
    let reply_queue_args = QueueDeclareArguments::default()
        .queue(reply_queue_name.clone())
        .durable(true)
        .finish();
    channel
        .queue_declare(reply_queue_args)
        .await
        .unwrap()
        .unwrap();
    channel
        .queue_bind(QueueBindArguments::new(
            &reply_queue_name,
            &exchange_name,
            &format!("{}_reply.service", agency_name),
        ))
        .await
        .unwrap();

    consume_messages(&channel, &reply_queue_name).await;

    add_new_queue(
        &channel,
        String::from("people_transport"),
        exchange_name.to_string(),
        String::from("people_transport.service"),
    )
    .await;

    add_new_queue(
        &channel,
        String::from("payload_transport"),
        exchange_name.to_string(),
        String::from("payload_transport.service"),
    )
    .await;

    add_new_queue(
        &channel,
        String::from("satelite_placement"),
        exchange_name.to_string(),
        String::from("satelite_placement.service"),
    )
    .await;

    add_new_queue(
        &channel,
        String::from("admin"),
        exchange_name.to_string(),
        String::from("*.service"),
    )
    .await;

    let admin_incoming_msgs_queue = format!("{}-admin_incoming_messages", agency_name);
    add_new_queue(
        &channel,
        admin_incoming_msgs_queue.clone(),
        exchange_name.to_string(),
        String::from("#.admin_messages"),
    )
    .await;
    consume_messages(&channel, &admin_incoming_msgs_queue).await;

    let publish_people_transport_args =
        BasicPublishArguments::new(exchange_name, "people_transport.service");
    let publish_payload_transport_args =
        BasicPublishArguments::new(exchange_name, "payload_transport.service");
    let publish_satelite_placement_args =
        BasicPublishArguments::new(exchange_name, "satelite_placement.service");

    let mut service_mapping = HashMap::new();
    service_mapping.insert("people_transport", publish_people_transport_args);
    service_mapping.insert("payload_transport", publish_payload_transport_args);
    service_mapping.insert("satelite_placement", publish_satelite_placement_args);

    let channel_clone = connection.open_channel(None).await.unwrap();
    channel_clone
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    task::spawn_blocking(move || {
        let mut input = String::new();
        let mut service_name;
        let stdin = io::stdin();
        let rt = tokio::runtime::Runtime::new().unwrap();
        while input != "exit\n" {
            input.clear();
            print!("Enter service name: ");
            io::stdout().flush().unwrap();

            stdin.read_line(&mut input).unwrap();

            service_name = input.trim();
            if service_mapping.contains_key(&service_name) {
                let correlation_id = format!(
                    "{}-{}",
                    agency_name,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                );
                let props = BasicProperties::default()
                    .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
                    .with_reply_to(&format!("{}_reply.service", agency_name))
                    .with_correlation_id(&correlation_id)
                    .finish();
                let publish_args = service_mapping.get(service_name).unwrap();
                rt.block_on(channel_clone.basic_publish(
                    props.clone(),
                    format!("[ID: {}] - {} scheduled!", correlation_id, service_name).into_bytes(),
                    publish_args.clone(),
                ))
                .unwrap();
            }
        }
    });

    let guard = Notify::new();
    guard.notified().await;

    channel.close().await.unwrap();
    connection.close().await.unwrap();
}

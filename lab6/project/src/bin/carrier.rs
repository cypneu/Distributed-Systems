use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicAckArguments, BasicConsumeArguments, BasicPublishArguments, BasicQosArguments,
        Channel, ExchangeDeclareArguments, QueueBindArguments, QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use std::str;
use std::{collections::HashSet, env, io::Write};
use tokio::{self, sync::Notify};

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
        .manual_ack(true)
        .finish();

    let (_ctag, mut rx) = channel.basic_consume_rx(consumer_args).await.unwrap();

    let channel = channel.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Some(payload) = msg.content {
                println!(" [x] Received {:?}", str::from_utf8(&payload).unwrap());
                let recv_props = msg.basic_properties.unwrap();
                let correlation_id = recv_props.correlation_id().unwrap();
                let publisher_props = BasicProperties::default()
                    .with_correlation_id(&correlation_id)
                    .finish();
                let publish_args = BasicPublishArguments::default()
                    .exchange(String::from("exchange"))
                    .routing_key(recv_props.reply_to().unwrap().into())
                    .finish();
                channel
                    .basic_publish(
                        publisher_props.clone(),
                        format!(
                            "Scheduled service with ID: {} has been fulfilled",
                            String::from(correlation_id)
                        )
                        .into_bytes(),
                        publish_args,
                    )
                    .await
                    .unwrap();
                channel
                    .basic_ack(BasicAckArguments::new(
                        msg.deliver.unwrap().delivery_tag(),
                        false,
                    ))
                    .await
                    .unwrap();
            }
        }
    });
}

async fn consume_admin_messages(channel: &Channel, queue_name: &String) {
    let consumer_args = BasicConsumeArguments::new(queue_name, queue_name)
        .manual_ack(false)
        .finish();
    let (_ctag, mut rx) = channel.basic_consume_rx(consumer_args).await.unwrap();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Some(payload) = msg.content {
                println!("\r{:?}", str::from_utf8(&payload).unwrap());
                std::io::stdout().flush().unwrap();
            }
        }
    });
}

#[tokio::main]
async fn main() {
    let cli_args = env::args().collect::<Vec<_>>();
    if cli_args.len() != 4 {
        println!("Usage: carrier <carrier_name> <service1> <service2>");
        return;
    }

    let mut available_services: HashSet<_> = vec![
        String::from("people_transport"),
        String::from("payload_transport"),
        String::from("satelite_placement"),
    ]
    .into_iter()
    .collect();

    if !available_services.remove(&cli_args[2]) || !available_services.remove(&cli_args[3]) {
        println!(
            "Services must be two of: [people_transport, payload_transport, satelite_placement]"
        );
        return;
    }
    let carrier_name = cli_args[1].clone();

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

    add_new_queue(
        &channel,
        cli_args[2].clone(),
        exchange_name.to_string(),
        format!("{}.service", cli_args[2]),
    )
    .await;

    add_new_queue(
        &channel,
        cli_args[3].clone(),
        exchange_name.to_string(),
        format!("{}.service", cli_args[3]),
    )
    .await;

    let admin_incoming_msgs_queue = format!("{}-admin_incoming_messages", carrier_name);
    add_new_queue(
        &channel,
        admin_incoming_msgs_queue.clone(),
        exchange_name.to_string(),
        String::from("admin_messages.#"),
    )
    .await;
    consume_admin_messages(&channel, &admin_incoming_msgs_queue).await;

    let qos_args = BasicQosArguments::new(0, 1, false);
    channel.basic_qos(qos_args).await.unwrap();

    consume_messages(&channel, &cli_args[2]).await;
    consume_messages(&channel, &cli_args[3]).await;

    println!(" [*] Waiting for messages. To exit press CTRL+C");

    let guard = Notify::new();
    guard.notified().await;
}

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicAckArguments, BasicConsumeArguments, BasicPublishArguments, BasicQosArguments,
        Channel, ExchangeDeclareArguments, QueueBindArguments, QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    BasicProperties, DELIVERY_MODE_PERSISTENT,
};
use std::io::{self, Write};
use std::str;
use tokio::{self, sync::Notify, task};

async fn add_new_queue(
    channel: &Channel,
    queue_name: &String,
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
            queue_name,
            &exchange_name,
            &routing_key,
        ))
        .await
        .unwrap();
}

async fn consume_messages(channel: &Channel, queue_name: &String) {
    let consumer_args = BasicConsumeArguments::new(queue_name, queue_name);
    let (_ctag, mut rx) = channel.basic_consume_rx(consumer_args).await.unwrap();

    let channel = channel.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Some(payload) = msg.content {
                println!("{:?}", str::from_utf8(&payload).unwrap());
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

#[tokio::main]
async fn main() {
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

    let queue_name = String::from("admin");

    add_new_queue(
        &channel,
        &queue_name,
        exchange_name.to_string(),
        String::from("*.service"),
    )
    .await;

    let qos_args = BasicQosArguments::new(0, 1, false);
    channel.basic_qos(qos_args).await.unwrap();

    consume_messages(&channel, &queue_name).await;

    let props = BasicProperties::default()
        .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
        .finish();

    task::spawn_blocking(move || {
        let mut input = String::new();
        let mut receiver_group;
        let stdin = io::stdin();
        let rt = tokio::runtime::Runtime::new().unwrap();
        while input != "exit\n" {
            input.clear();
            print!("Enter message multicast receiver [A, C, AC]: ");
            io::stdout().flush().unwrap();

            stdin.read_line(&mut input).unwrap();

            receiver_group = input.trim();

            let routing_key = match receiver_group {
                "A" => "agencies.admin_messages",
                "C" => "admin_messages.carriers",
                "AC" => "admin_messages",
                _ => {
                    println!("Invalid receiver group");
                    continue;
                }
            };

            print!("Enter the message: ");
            io::stdout().flush().unwrap();
            input.clear();
            stdin.read_line(&mut input).unwrap();
            let message = input.trim();

            let publish_args = BasicPublishArguments::new(exchange_name, routing_key);

            rt.block_on(channel.basic_publish(
                props.clone(),
                format!("Message from admin: {}", message).into_bytes(),
                publish_args,
            ))
            .unwrap();
        }
    });

    let guard = Notify::new();
    guard.notified().await;
}

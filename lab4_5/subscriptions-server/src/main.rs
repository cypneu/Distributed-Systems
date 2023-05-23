use std::time::Duration;
use subscription::notification::{Content, NotificationType};
use subscription::subscription_server::{Subscription, SubscriptionServer};
use subscription::{Notification, SubscriptionRequest};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod subscription {
    tonic::include_proto!("subscription");
}

pub struct SubscriptionService;

#[tonic::async_trait]
impl Subscription for SubscriptionService {
    type SubscribeStream = ReceiverStream<Result<Notification, Status>>;

    async fn subscribe(
        &self,
        request: Request<tonic::Streaming<SubscriptionRequest>>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let addr = request.remote_addr().unwrap();
        println!("RPC subscribe() called from address: {}", addr);

        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(10);

        while let Some(req_mess) = stream.next().await {
            let tx = tx.clone();

            let _task_handle = tokio::spawn(async move {
                let req_mess = req_mess.unwrap();
                let mut buffer: Vec<Notification> = Vec::new();

                let mut interval =
                    interval(Duration::from_secs(req_mess.notification_frequency as u64));

                let response = Notification {
                    frequency: req_mess.notification_frequency,
                    notification_type: NotificationType::Advertisement.into(),
                    content: Content {
                        body: vec![
                            req_mess.name.to_string(),
                            NotificationType::Advertisement.as_str_name().to_string(),
                            "Please buy our product!".to_string(),
                        ],
                    }
                    .into(),
                };

                loop {
                    interval.tick().await;

                    if let Err(err) = tx.send(Ok(response.clone())).await {
                        buffer.push(response.clone());
                        println!("Attempt of sending to {} failed", addr);
                        if buffer.len() > 10 {
                            println!("{} from address {}", err, addr);
                            break;
                        }
                    } else {
                        let mut temp_buffer: Vec<Notification> = Vec::new();
                        while let Some(overdue_response) = buffer.pop() {
                            if let Err(_) = tx.send(Ok(overdue_response.clone())).await {
                                temp_buffer.push(overdue_response.clone());
                            }
                        }
                        buffer = temp_buffer;
                    }
                }
            });
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let subscription_service = SubscriptionService {};

    Server::builder()
        .tcp_keepalive(Some(Duration::from_secs(30)))
        .add_service(SubscriptionServer::new(subscription_service))
        .serve(addr)
        .await?;

    Ok(())
}

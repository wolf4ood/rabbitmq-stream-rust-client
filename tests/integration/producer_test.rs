use fake::{Fake, Faker};
use futures::StreamExt;
use rabbitmq_stream_client::types::{Message, OffsetSpecification};
use tokio::sync::mpsc::channel;

use crate::common::TestEnvironment;

#[tokio::test(flavor = "multi_thread")]
async fn producer_send_ok() {
    let env = TestEnvironment::create().await;
    let reference: String = Faker.fake();

    let producer = env
        .env
        .producer()
        .name(&reference)
        .build(&env.stream)
        .await
        .unwrap();

    let mut consumer = env
        .env
        .consumer()
        .offset(OffsetSpecification::Next)
        .build(&env.stream)
        .await
        .unwrap();

    let _ = producer
        .send(Message::builder().body(b"message".to_vec()).build())
        .await
        .unwrap();

    producer.close().await.unwrap();

    let delivery = consumer.next().await.unwrap().unwrap();
    assert_eq!(1, delivery.subscription_id);
    assert_eq!(Some(b"message".as_ref()), delivery.message.data());

    consumer.handle().close().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn producer_send_with_callback() {
    let env = TestEnvironment::create().await;
    let reference: String = Faker.fake();

    let (tx, mut rx) = channel(1);
    let producer = env
        .env
        .producer()
        .name(&reference)
        .build(&env.stream)
        .await
        .unwrap();

    let _ = producer
        .send_with_callback(
            Message::builder().body(b"message".to_vec()).build(),
            move |confirm_result| {
                let inner_tx = tx.clone();
                async move {
                    let _ = inner_tx.send(confirm_result).await;
                }
            },
        )
        .await
        .unwrap();

    let result = rx.recv().await.unwrap();

    assert_eq!(1, result.unwrap());

    producer.close().await.unwrap();
}
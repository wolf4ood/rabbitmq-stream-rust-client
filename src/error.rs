use rabbitmq_stream_protocol::{
    error::{DecodeError, EncodeError},
    ResponseCode,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error("Cast Error: {0}")]
    CastError(String),
    #[error(transparent)]
    GenericError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Client already closed")]
    AlreadyClosed,
    #[error("Connection closed")]
    ConnectionClosed,
    #[error(transparent)]
    Tls(#[from] tokio_rustls::rustls::Error),
    #[error("Request error: {0:?}")]
    RequestError(ResponseCode),
}

#[derive(Error, Debug)]
pub enum ConsumerStoreOffsetError {
    #[error("Failed to store offset, missing consumer name")]
    NameMissing,
    #[error(transparent)]
    Client(#[from] ClientError),
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Encode Error {0:?}")]
    Encode(EncodeError),
    #[error("Decode Error {0:?}")]
    Decode(DecodeError),
}

impl From<EncodeError> for ClientError {
    fn from(err: EncodeError) -> Self {
        ClientError::Protocol(ProtocolError::Encode(err))
    }
}

impl From<DecodeError> for ClientError {
    fn from(err: DecodeError) -> Self {
        ClientError::Protocol(ProtocolError::Decode(err))
    }
}

#[derive(Error, Debug)]
pub enum StreamCreateError {
    #[error("Failed to create stream {stream} status: {status:?}")]
    Create {
        stream: String,
        status: ResponseCode,
    },
    #[error(transparent)]
    Client(#[from] ClientError),
}

#[derive(Error, Debug)]
pub enum StreamDeleteError {
    #[error("Failed to delete stream {stream} status: {status:?}")]
    Delete {
        stream: String,
        status: ResponseCode,
    },
    #[error(transparent)]
    Client(#[from] ClientError),
}

#[derive(Error, Debug)]
pub enum ProducerCreateError {
    #[error("Failed to create producer for stream {stream} status {status:?}")]
    Create {
        stream: String,
        status: ResponseCode,
    },

    #[error("Stream {stream} does not exist")]
    StreamDoesNotExist { stream: String },

    #[error(transparent)]
    Client(#[from] ClientError),

    #[error("Filtering is not supported by the broker (requires RabbitMQ 3.13+ and stream_filtering feature flag activated)")]
    FilteringNotSupport,
}

#[derive(Error, Debug)]
pub enum ProducerPublishError {
    #[error("Failed to publish message for stream {stream} status {status:?}")]
    Create {
        stream: String,
        publisher_id: u8,
        status: ResponseCode,
    },
    #[error("Failed to send batch messages for stream {stream}")]
    Batch { stream: String },
    #[error("Failed to publish message, the producer is closed")]
    Closed,
    #[error("Failed to publish message, confirmation channel returned None for stream {stream}")]
    Confirmation { stream: String },
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error("Failed to publish message, timeout")]
    Timeout,
}

#[derive(Error, Debug)]
pub enum SuperStreamProducerPublishError {
    #[error("Failed to send message to stream")]
    ProducerPublishError(),
    #[error("Failed to create a producer")]
    ProducerCreateError(),
}

impl From<ProducerPublishError> for SuperStreamProducerPublishError {
    fn from(_err: ProducerPublishError) -> Self {
        SuperStreamProducerPublishError::ProducerPublishError()
    }
}
impl From<ProducerCreateError> for SuperStreamProducerPublishError {
    fn from(_err: ProducerCreateError) -> Self {
        SuperStreamProducerPublishError::ProducerCreateError()
    }
}

#[derive(Error, Debug)]
pub enum ProducerCloseError {
    #[error("Failed to close producer for stream {stream} status {status:?}")]
    Close {
        stream: String,
        status: ResponseCode,
    },
    #[error("Producer already closed")]
    AlreadyClosed,
    #[error(transparent)]
    Client(#[from] ClientError),
}

#[derive(Error, Debug)]
pub enum ConsumerCreateError {
    #[error("Failed to create consumer for stream {stream} status {status:?}")]
    Create {
        stream: String,
        status: ResponseCode,
    },

    #[error("Stream {stream} does not exist")]
    StreamDoesNotExist { stream: String },

    #[error(transparent)]
    Client(#[from] ClientError),

    #[error("Filtering is not supported by the broker (requires RabbitMQ 3.13+ and stream_filtering feature flag activated)")]
    FilteringNotSupport,

    #[error("if you set single active consumer a consumer and super_stream consumer name need to be setup")]
    SingleActiveConsumerNotSupported,
}

#[derive(Error, Debug)]
pub enum ConsumerDeliveryError {
    #[error("Failed to create consumer for stream {stream} status {status:?}")]
    Credit {
        stream: String,
        status: ResponseCode,
    },
    #[error(transparent)]
    Client(#[from] ClientError),
}
#[derive(Error, Debug)]
pub enum ConsumerCloseError {
    #[error("Failed to close consumer for stream {stream} status {status:?}")]
    Close {
        stream: String,
        status: ResponseCode,
    },
    #[error("Consumer already closed")]
    AlreadyClosed,
    #[error(transparent)]
    Client(#[from] ClientError),
}

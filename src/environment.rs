use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use crate::producer::NoDedup;
use crate::types::OffsetSpecification;
use std::collections::HashMap;

use crate::{
    client::{Client, ClientOptions, MetricsCollector},
    consumer::ConsumerBuilder,
    error::StreamDeleteError,
    producer::ProducerBuilder,
    stream_creator::StreamCreator,
    superstream::RoutingStrategy,
    superstream_consumer::SuperStreamConsumerBuilder,
    superstream_producer::SuperStreamProducerBuilder,
    RabbitMQStreamResult,
};

/// Main access point to a node
#[derive(Clone)]
pub struct Environment {
    pub(crate) options: EnvironmentOptions,
}

impl Environment {
    pub fn builder() -> EnvironmentBuilder {
        EnvironmentBuilder(EnvironmentOptions::default())
    }

    async fn boostrap(options: EnvironmentOptions) -> RabbitMQStreamResult<Self> {
        // check connection
        let client = Client::connect(options.client_options.clone()).await?;
        client.close().await?;
        Ok(Environment { options })
    }

    /// Returns a builder for creating a stream with a specific configuration
    pub fn stream_creator(&self) -> StreamCreator {
        StreamCreator::new(self.clone())
    }

    /// Returns a builder for creating a producer
    pub fn producer(&self) -> ProducerBuilder<NoDedup> {
        ProducerBuilder {
            environment: self.clone(),
            name: None,
            batch_size: 100,
            batch_publishing_delay: Duration::from_millis(100),
            data: PhantomData,
            filter_value_extractor: None,
            client_provided_name: String::from("rust-stream-producer"),
        }
    }

    pub fn super_stream_producer(
        &self,
        routing_strategy: RoutingStrategy,
    ) -> SuperStreamProducerBuilder<NoDedup> {
        SuperStreamProducerBuilder {
            environment: self.clone(),
            data: PhantomData,
            filter_value_extractor: None,
            route_strategy: routing_strategy,
            client_provided_name: String::from("rust-super-stream-producer"),
        }
    }

    /// Returns a builder for creating a consumer
    pub fn consumer(&self) -> ConsumerBuilder {
        ConsumerBuilder {
            consumer_name: None,
            environment: self.clone(),
            offset_specification: OffsetSpecification::Next,
            filter_configuration: None,
            client_provided_name: String::from("rust-stream-consumer"),
            properties: HashMap::new(),
        }
    }

    pub fn super_stream_consumer(&self) -> SuperStreamConsumerBuilder {
        SuperStreamConsumerBuilder {
            environment: self.clone(),
            offset_specification: OffsetSpecification::Next,
            filter_configuration: None,
            client_provided_name: String::from("rust-super-stream-consumer"),
            properties: HashMap::new(),
        }
    }

    pub(crate) async fn create_client(&self) -> RabbitMQStreamResult<Client> {
        Client::connect(self.options.client_options.clone()).await
    }
    pub(crate) async fn create_client_with_options(
        &self,
        opts: impl Into<ClientOptions>,
    ) -> RabbitMQStreamResult<Client> {
        Client::connect(opts).await
    }

    /// Delete a stream
    pub async fn delete_stream(&self, stream: &str) -> Result<(), StreamDeleteError> {
        let client = self.create_client().await?;
        let response = client.delete_stream(stream).await?;
        client.close().await?;

        if response.is_ok() {
            Ok(())
        } else {
            Err(StreamDeleteError::Delete {
                stream: stream.to_owned(),
                status: response.code().clone(),
            })
        }
    }

    pub async fn delete_super_stream(&self, super_stream: &str) -> Result<(), StreamDeleteError> {
        let client = self.create_client().await?;
        let response = client.delete_super_stream(super_stream).await?;
        client.close().await?;

        if response.is_ok() {
            Ok(())
        } else {
            Err(StreamDeleteError::Delete {
                stream: super_stream.to_owned(),
                status: response.code().clone(),
            })
        }
    }
}

/// Builder for [`Environment`]
pub struct EnvironmentBuilder(EnvironmentOptions);

impl EnvironmentBuilder {
    pub async fn build(self) -> RabbitMQStreamResult<Environment> {
        Environment::boostrap(self.0).await
    }

    pub fn host(mut self, host: &str) -> EnvironmentBuilder {
        self.0.client_options.host = host.to_owned();
        self
    }

    pub fn username(mut self, username: &str) -> EnvironmentBuilder {
        self.0.client_options.user = username.to_owned();
        self
    }

    pub fn password(mut self, password: &str) -> EnvironmentBuilder {
        self.0.client_options.password = password.to_owned();
        self
    }

    pub fn virtual_host(mut self, virtual_host: &str) -> EnvironmentBuilder {
        self.0.client_options.v_host = virtual_host.to_owned();
        self
    }
    pub fn port(mut self, port: u16) -> EnvironmentBuilder {
        self.0.client_options.port = port;
        self
    }

    pub fn tls(mut self, tls_configuration: TlsConfiguration) -> EnvironmentBuilder {
        self.0.client_options.tls = tls_configuration;

        self
    }

    pub fn heartbeat(mut self, heartbeat: u32) -> EnvironmentBuilder {
        self.0.client_options.heartbeat = heartbeat;
        self
    }
    pub fn metrics_collector(
        mut self,
        collector: impl MetricsCollector + 'static,
    ) -> EnvironmentBuilder {
        self.0.client_options.collector = Arc::new(collector);
        self
    }

    pub fn load_balancer_mode(mut self, load_balancer_mode: bool) -> EnvironmentBuilder {
        self.0.client_options.load_balancer_mode = load_balancer_mode;
        self
    }

    pub fn client_provided_name(mut self, name: &str) -> EnvironmentBuilder {
        self.0.client_options.client_provided_name = name.to_owned();
        self
    }
}
#[derive(Clone, Default)]
pub struct EnvironmentOptions {
    pub(crate) client_options: ClientOptions,
}

/** Helper for tls configuration */
#[derive(Clone)]
pub struct TlsConfiguration {
    pub(crate) enabled: bool,
    pub(crate) trust_certificates: bool,
    pub(crate) root_certificates_path: String,
    pub(crate) client_certificates_path: String,
    pub(crate) client_keys_path: String,
}

impl Default for TlsConfiguration {
    fn default() -> TlsConfiguration {
        TlsConfiguration {
            enabled: true,
            trust_certificates: false,
            root_certificates_path: String::from(""),
            client_certificates_path: String::from(""),
            client_keys_path: String::from(""),
        }
    }
}

impl TlsConfiguration {
    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn trust_certificates(&mut self, trust_certificates: bool) {
        self.trust_certificates = trust_certificates
    }

    pub fn trust_certificates_enabled(&self) -> bool {
        self.trust_certificates
    }

    pub fn get_root_certificates_path(&self) -> String {
        self.root_certificates_path.clone()
    }
    //
    pub fn add_root_certificates_path(&mut self, certificate_path: String) {
        self.root_certificates_path = certificate_path
    }

    pub fn get_client_certificates_path(&self) -> String {
        self.client_certificates_path.clone()
    }

    pub fn get_client_keys_path(&self) -> String {
        self.client_keys_path.clone()
    }
    //
    pub fn add_client_certificates_keys(
        &mut self,
        certificate_path: String,
        client_private_key_path: String,
    ) {
        self.client_certificates_path = certificate_path;
        self.client_keys_path = client_private_key_path;
    }
}

pub struct TlsConfigurationBuilder(TlsConfiguration);

impl TlsConfigurationBuilder {
    pub fn enable(mut self, enable: bool) -> TlsConfigurationBuilder {
        self.0.enabled = enable;
        self
    }

    pub fn trust_certificates(mut self, trust_certificates: bool) -> TlsConfigurationBuilder {
        self.0.trust_certificates = trust_certificates;
        self
    }

    pub fn add_root_certificates(mut self, certificate_path: String) -> TlsConfigurationBuilder {
        self.0.root_certificates_path = certificate_path;
        self
    }

    pub fn add_client_certificates_keys(
        mut self,
        certificate_path: String,
        client_private_key_path: String,
    ) -> TlsConfigurationBuilder {
        self.0.client_certificates_path = certificate_path;
        self.0.client_keys_path = client_private_key_path;
        self
    }

    pub fn build(self) -> TlsConfiguration {
        self.0
    }
}

impl TlsConfiguration {
    pub fn builder() -> TlsConfigurationBuilder {
        TlsConfigurationBuilder(TlsConfiguration::default())
    }
}

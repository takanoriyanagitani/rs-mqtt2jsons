use std::io;
use std::pin::Pin;

use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;

use rumqttc::AsyncClient;
use rumqttc::Event;

use rumqttc::EventLoop;
use rumqttc::MqttOptions;
use rumqttc::QoS;

pub use rumqttc;

pub async fn get_event(el: &mut EventLoop) -> Result<Event, io::Error> {
    el.poll().await.map_err(io::Error::other)
}

pub struct EventSource {
    cl: AsyncClient,
    el: EventLoop,
}

impl EventSource {
    pub async fn get_payload(el: &mut EventLoop, retries: usize) -> Result<Vec<u8>, io::Error> {
        for _ in 0..retries {
            let evt = get_event(el).await?;
            if let Event::Incoming(rumqttc::Packet::Publish(p)) = evt {
                return Ok(p.payload.into());
            }
        }
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "get_payload timed out",
        ))
    }

    pub async fn subscribe<S>(&self, topic: S, qos: QoS) -> Result<(), io::Error>
    where
        S: Into<String>,
    {
        self.cl
            .subscribe(topic, qos)
            .await
            .map_err(io::Error::other)
    }

    pub fn into_stream(
        self,
        retries: usize,
    ) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, io::Error>> + Send>> {
        Box::pin(futures::stream::unfold(self.el, move |mut el| async move {
            loop {
                match Self::get_payload(&mut el, retries).await {
                    Ok(p) => return Some((Ok(p), el)),
                    Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
                    Err(e) => return Some((Err(e), el)),
                }
            }
        }))
    }
}

/// Converts the payloads assuming they are utf8 strings.
pub fn payloads2strings<S>(original: S) -> impl Stream<Item = Result<String, io::Error>> + Unpin
where
    S: Stream<Item = Result<Vec<u8>, io::Error>> + Unpin,
{
    original.map(|r| r.map(|v: Vec<u8>| String::from_utf8(v).unwrap_or_default()))
}

pub async fn print_strings<S>(stream: S) -> Result<(), io::Error>
where
    S: Stream<Item = Result<String, io::Error>> + Unpin,
{
    stream
        .try_fold((), |_, s| async move {
            println!("{s}");
            Ok(())
        })
        .await
}

pub const RETRIES_DEFAULT: usize = 10;
pub const QOS_DEFAULT: QoS = QoS::AtMostOnce;
pub const CAP_DEFAULT: usize = 10;

pub struct EvtSourceCfg {
    pub opts: MqttOptions,
    pub topic: String,
    pub retries: usize,
    pub qos: QoS,
    pub cap: usize,
}

impl EvtSourceCfg {
    pub fn into_source(self) -> EventSource {
        let (cl, el) = AsyncClient::new(self.opts, self.cap);
        EventSource { cl, el }
    }
}

impl EvtSourceCfg {
    pub async fn into_strings(
        self,
    ) -> Result<impl Stream<Item = Result<String, io::Error>>, io::Error> {
        let topic: String = self.topic.clone();
        let qos: QoS = self.qos;
        let retries: usize = self.retries;
        let src: EventSource = self.into_source();
        src.subscribe(topic, qos).await?;
        let payloads = src.into_stream(retries);
        Ok(payloads2strings(payloads))
    }
}

impl EvtSourceCfg {
    pub fn new(opts: MqttOptions, topic: String, retries: usize, qos: QoS, cap: usize) -> Self {
        Self {
            opts,
            topic,
            retries,
            qos,
            cap,
        }
    }

    pub fn new_with_default(opts: MqttOptions, topic: String) -> Self {
        Self::new(opts, topic, RETRIES_DEFAULT, QOS_DEFAULT, CAP_DEFAULT)
    }
}

/// Note: Default impl use "zero" uuid for client_id.
pub struct OptCfg {
    pub client_id: String,
    pub host_ip: String,
    pub host_port: u16,
}

impl OptCfg {
    pub fn into_opts(self) -> MqttOptions {
        MqttOptions::new(self.client_id, self.host_ip, self.host_port)
    }
}

impl OptCfg {
    pub fn with_client_id(mut self, id: String) -> Self {
        self.client_id = id;
        self
    }

    pub fn with_host_ip(mut self, ip: String) -> Self {
        self.host_ip = ip;
        self
    }

    pub fn with_host_port(mut self, port: u16) -> Self {
        self.host_port = port;
        self
    }
}

impl Default for OptCfg {
    fn default() -> Self {
        Self {
            client_id: "00000000-0000-0000-0000-000000000000".into(),
            host_ip: "127.0.0.1".into(),
            host_port: 1883,
        }
    }
}

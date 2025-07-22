use std::env;
use std::process::ExitCode;

use std::io;

use rs_mqtt2jsons::rumqttc;

use rumqttc::MqttOptions;
use rumqttc::QoS;

use rs_mqtt2jsons::EvtSourceCfg;
use rs_mqtt2jsons::OptCfg;
use rs_mqtt2jsons::print_strings;

async fn sub() -> Result<(), io::Error> {
    let client_id: String = env::var("MQTT_CLIENT_ID").map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("MQTT_CLIENT_ID not set or invalid: {e}"),
        )
    })?;

    let topic: String = env::var("MQTT_TOPIC").map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("MQTT_TOPIC not set or invalid: {e}"),
        )
    })?;

    let host_ip: String = env::var("MQTT_HOST_IP").unwrap_or_else(|_| OptCfg::default().host_ip);
    let host_port: u16 = match env::var("MQTT_HOST_PORT") {
        Ok(s) => s.parse::<u16>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("MQTT_HOST_PORT invalid: {e}"),
            )
        })?,
        Err(_) => OptCfg::default().host_port,
    };

    let ocfg: OptCfg = OptCfg::default()
        .with_client_id(client_id)
        .with_host_ip(host_ip)
        .with_host_port(host_port);

    let opts: MqttOptions = ocfg.into_opts();

    let mut cfg: EvtSourceCfg = EvtSourceCfg::new_with_default(opts, topic);

    cfg.qos = QoS::AtLeastOnce;

    let strings = cfg.into_strings().await?;
    print_strings(strings).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    match sub().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

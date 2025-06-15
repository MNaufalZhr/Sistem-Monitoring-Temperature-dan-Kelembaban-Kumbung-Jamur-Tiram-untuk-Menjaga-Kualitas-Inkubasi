use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};
use serde::Deserialize;
use reqwest::Client;

use ethers::prelude::*;
use ethers::abi::Abi;
use std::{fs, sync::Arc};
use chrono::DateTime;

#[derive(Deserialize, Debug)]
struct SensorData {
    timestamp: String,
    sensor_id: String,
    location: String,
    process_stage: String,
    temperature_celsius: f32,
    humidity_percent: f32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- InfluxDB setup ---
    let influx_url = "http://localhost:8086/api/v2/write?org=gamtenk&bucket=kelompokgacor&precision=s";
    let influx_token = "yBoxgtA5n1iXvYpHPEiYnUS8ZIkEtZ5QfiTvo3FKgckP2GE4MYyspnH9xTRaKGx-2WMJ7Y6WCk2cOqzfo7R25g==";
    let http_client = Client::new();

    // --- Ethereum setup ---
    let provider = Provider::<Http>::try_from("http://localhost:8545")?;
    let wallet: LocalWallet = "0x83aafe12d1b9d322450b49df1972adfec1665c922365be59b6e4383cc85baccc"
        .parse::<LocalWallet>()?
        .with_chain_id(1337u64);
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    // Baca dan parse ABI dan bytecode dengan benar
    let abi_str = fs::read_to_string("build/SensorStorage.abi")?;
    let bytecode_str = fs::read_to_string("build/SensorStorage.bin")?;

    let abi: Abi = serde_json::from_str(&abi_str)?;
    let bytecode = bytecode_str.trim().parse::<Bytes>()?;

    let factory = ContractFactory::new(abi, bytecode, client.clone());

    let contract = factory.deploy(())?.send().await?;
    println!("✅ Smart contract deployed at: {:?}", contract.address());

    // --- TCP Server ---
    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    println!("🚪 TCP Server listening on port 9000...");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("🔌 New connection from {}", addr);

        let influx_url = influx_url.to_string();
        let influx_token = influx_token.to_string();
        let http_client = http_client.clone();
        let contract = contract.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(socket);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                match serde_json::from_str::<SensorData>(&line) {
                    Ok(data) => {
                        println!("📥 Received sensor data: {:?}", data);

                        // --- InfluxDB Write ---
                        let timestamp = DateTime::parse_from_rfc3339(&data.timestamp)
                            .unwrap()
                            .timestamp();

                        let line_protocol = format!(
                            "monitoring,sensor_id={},location={},stage={} temperature={},humidity={} {}",
                            data.sensor_id.replace(" ", "\\ "),
                            data.location.replace(" ", "\\ "),
                            data.process_stage.replace(" ", "\\ "),
                            data.temperature_celsius,
                            data.humidity_percent,
                            timestamp
                        );

                        match http_client
                            .post(&influx_url)
                            .header("Authorization", format!("Token {}", influx_token))
                            .header("Content-Type", "text/plain")
                            .body(line_protocol)
                            .send()
                            .await
                        {
                            Ok(resp) if resp.status().is_success() => {
                                println!("✅ InfluxDB: data written");
                            }
                            Ok(resp) => {
                                println!("⚠️ InfluxDB error: {}", resp.status());
                            }
                            Err(e) => {
                                println!("❌ InfluxDB HTTP error: {}", e);
                            }
                        }

                        // --- Ethereum Contract Write ---
                        let method_call = contract
    .method::<_, H256>("storeData", (
        timestamp as u64,
        data.sensor_id.clone(),
        data.location.clone(),
        data.process_stage.clone(),
        (data.temperature_celsius * 100.0) as i64,
        (data.humidity_percent * 100.0) as i64,
    ))
    .unwrap();

let tx = method_call.send().await;

                        match tx {
                            Ok(pending_tx) => {
                                println!("📡 Ethereum: tx sent: {:?}", pending_tx);
                            }
                            Err(e) => {
                                println!("❌ Ethereum tx error: {:?}", e);
                            }
                        }
                    }
                    Err(e) => println!("❌ Invalid JSON received: {}", e),
                }
            }
        });
    }
}


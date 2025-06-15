# Sistem-Monitoring-Temperature-dan-Kelembaban-Kumbung-Jamur-Tiram-untuk-Menjaga-Kualitas-Inkubasi
Jamur tiram (Pleurotus ostreatus) membutuhkan kondisi lingkungan optimal, yaitu suhu 22–28°C dan kelembapan 80–90%. Sistem monitoring berbasis IoT telah banyak diterapkan, seperti oleh (Hudhoifah & Mulyana 2024) yang menggunakan NodeMCU dan DHT22, serta (Ntihung et al. 2024) dengan Arduino dan DHT11. Meski efektif, keduanya masih memiliki keterbatasan akurasi sensor, reliabilitas data, dan ketergantungan mikrokontroler .
Untuk itu, dikembangkan sistem monitoring menggunakan sensor SHT20 Modbus RTU tanpa mikrokontroler, terhubung langsung ke TCP server berbasis Rust, dengan penyimpanan data di InfluxDB dan visualisasi real-time via Grafana. Sistem ini juga dilengkapi integrasi blockchain dan Web3 untuk keamanan dan transparansi data, menjadikannya lebih akurat, stabil, dan aman untuk budidaya jamur tiram. 
# NAMA KELOMPOK :
1. Muhammad Naufal Zuhair (2042231005)
2. Pandu Galuh Satrio (2042231019)
3. Yanuar Rahmansyah (2042231057)
4. Ahmad Radhy, S.Si., M.Si (Supervisor)
# Pengertian Setiap Tools
## Sensor SHT 20
Sensor modbus SHT 20 merupakan sensor temperature dan kelembapan dengan memiliki presisi yang tinggi. Sensor ini menggunakan protocol komunikasi Modbus RTU berbasis RS485. SHT 20 memiliki karakteristik resistif terhadap perubahan kadar air di udara serta terdapat chip yang bisa mengkonversi analog ke digital dengan menggunakan bidirectional (kabel tunggal dua arah).
   
## Modbus Client 
Modbus merupakan protokol komunikasi yang banyak digunakan dalam sistem otomasi industri untuk pertukaran data antara perangkat seperti PLC, sensor, dan aktuator. Dalam versi Modbus TCP, istilah Modbus Client mengacu pada perangkat atau program yang berperan aktif dalam memulai komunikasi, seperti membaca data dari atau menulis data ke Modbus Server. Dalam hal ini, bahasa pemrograman Rust menjadi pilihan yang menarik untuk mengembangkan Modbus Client karena memiliki keunggulan dalam hal keamanan memori, kecepatan eksekusi, dan efisiensi dalam pengelolaan proses secara bersamaan (asinkron).
   
## TCP Server 
TCP Server dalam pemrograman Rust adalah program yang bertugas menerima dan merespons koneksi dari client menggunakan protokol TCP, yang menjamin data dikirim secara utuh dan berurutan. Rust sangat cocok untuk membuat TCP server karena punya sistem manajemen memori yang aman tanpa garbage collector, sehingga server lebih stabil dan bebas dari bug seperti crash atau data race. Selain itu, Rust terkenal efisien dan cepat, membuatnya mampu menangani banyak koneksi sekaligus tanpa membuat sistem lambat. Dengan bantuan pustaka seperti std::net untuk versi sederhana (sinkron) atau tokio untuk versi asinkron (non-blocking), kita bisa membangun server yang bisa melayani ribuan client secara bersamaan.
   
## InfluxDB
InfluxDB merupakan salah satu implementasi dari database time-series yang dikembangkan secara khusus untuk memenuhi kebutuhan penyimpanan dan pengolahan data berdasarkan waktu. InfluxDB didesain dengan arsitektur yang mampu menangani data dalam jumlah besar secara efisien dan memberikan performa tinggi, baik dalam hal pencatatan data (write performance) maupun pengambilan data (query performance), terutama pada skala waktu yang sangat detail (misalnya per detik atau bahkan milidetik).
   
## Grafana 
Grafana merupakan sebuah platform open-source yang digunakan untuk visualisasi data, pemantauan sistem, dan analisis data secara real-time. Platform ini memungkinkan pengguna untuk membuat dashboard interaktif yang menampilkan berbagai jenis visualisasi seperti grafik, tabel, dan gauge, sehingga memudahkan pemantauan performa sistem atau aplikasi secara langsung. Grafana mendukung koneksi ke berbagai sumber data (data source) seperti InfluxDB, Prometheus, MySQL, PostgreSQL, dan Elasticsearch, sehingga fleksibel digunakan pada berbagai jenis aplikasi dan bidang.
   
## Blockchain 
sebuah teknologi penyimpanan data digital yang tersusun dalam bentuk rantai blok, di mana setiap blok berisi sejumlah transaksi atau data, dan saling terhubung serta diamankan menggunakan teknik kriptografi.
   
## Web3 
istilah untuk generasi ketiga dari perkembangan internet yang dibangun di atas teknologi blockchain. Web3 bertujuan membuat internet yang lebih terbuka, terdesentralisasi, dan dimiliki oleh penggunanya.
# STEP BY STEP PERANCANGAN SISTEM
## 1. Konfigurasi Sensor
   ### Main.rs Pada Sensor Sht20
   ```Rust
    use tokio_modbus::{client::rtu, prelude::*};
    use tokio_serial::{SerialPortBuilderExt, Parity, StopBits, DataBits};
    use tokio::net::TcpStream;
    use tokio::io::AsyncWriteExt;
    use serde::Serialize;
    use chrono::Utc;
    use std::error::Error;
    use tokio::time::{sleep, Duration};
    
    #[derive(Serialize)]
    struct SensorData {
        timestamp: String,
        sensor_id: String,
        location: String,
        process_stage: String,
        temperature_celsius: f32,
        humidity_percent: f32,
    }
    
    async fn read_sensor(slave: u8) -> Result<Vec<u16>, Box<dyn Error>> {
        let builder = tokio_serial::new("/dev/ttyUSB0", 9600)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .data_bits(DataBits::Eight)
            .timeout(std::time::Duration::from_secs(1));

    let port = builder.open_native_async()?;
    let mut ctx = rtu::connect_slave(port, Slave(slave)).await?;
    let response = ctx.read_input_registers(1, 2).await?;

    Ok(response)
    }

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn Error>> {
        loop {
            match read_sensor(1).await {
                Ok(response) if response.len() == 2 => {
                    let temp = response[0] as f32 / 10.0;
                    let rh = response[1] as f32 / 10.0;

                println!("📡 Temp: {:.1} °C | RH: {:.1} %", temp, rh);

                let data = SensorData {
                    timestamp: Utc::now().to_rfc3339(),
                    sensor_id: "SHT20-Kelembapan-001".into(),
                    location: "Kumbung Inkubasi 1".into(),
                    process_stage: "Inkubasi".into(),
                    temperature_celsius: temp,
                    humidity_percent: rh,
                };

                let json = serde_json::to_string(&data)?;
                
                match TcpStream::connect("127.0.0.1:9000").await {
                    Ok(mut stream) => {
                        stream.write_all(json.as_bytes()).await?;
                        stream.write_all(b"\n").await?;
                        println!("✅ Data dikirim ke TCP server");
                    },
                    Err(e) => {
                        println!("❌ Gagal konek ke TCP server: {}", e);
                    }
                }
            },
            Ok(other) => {
                println!("⚠️ Data tidak lengkap: {:?}", other);
            },
            Err(e) => {
                println!("❌ Gagal baca sensor: {}", e);
            }
        }

        sleep(Duration::from_secs(5)).await;
      }
    }

```
**Program ini bertugas untuk:**
1. Membaca data sensor suhu dan kelembapan via Modbus RTU di port serial /dev/ttyUSB0.
2. Mengemas data sensor (dengan informasi timestamp, sensor ID, lokasi, dan tahap proses) dalam format JSON.
3. Mengirim data JSON tersebut ke server TCP di 127.0.0.1:9000.
4. Melakukan proses ini secara periodik setiap 5 detik.

### sensor.toml
```
[package]
name = "sensor_client"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.32", features = ["full"] }
tokio-serial = "5.4.0"
tokio-modbus = "0.5.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }

```
## 2. Konfigurasi Server
  ### Main.rs 
```
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
    let wallet: LocalWallet = "0xbab91d012fe3c2256e401be4f924da9602a2ac836c20412b90e12f79147b18f2"
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
```
**Program ini berfungsi sebagai TCP server yang:**
1. Menerima data sensor dari client via koneksi TCP (port 9000) dalam format JSON.
2. Menyimpan data ke InfluxDB (database time-series) untuk pemantauan suhu & kelembapan.
3. Mencatat data ke blockchain Ethereum lewat smart contract yang dideploy di node lokal.

### Server.toml
```
[package]
name = "sensor_server_blockchain"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
chrono = "0.4"
ethers = { version = "2.0", features = ["abigen"] }
anyhow = "1.0"
```
### Contractssensor.sol
```
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SensorStorage {
    event DataStored(
        uint256 timestamp,
        string sensorId,
        string location,
        string stage,
        int256 temperature,
        int256 humidity
    );

    function storeData(
        uint256 timestamp,
        string memory sensorId,
        string memory location,
        string memory stage,
        int256 temperature,
        int256 humidity
    ) public {
        emit DataStored(timestamp, sensorId, location, stage, temperature, humidity);
    }
}
```

## 3. Web 3
### Script.js
```
const contractAddress = "0x43dd0a7c03dac44e63681edaea59946618d2a8b5"; // Ganti dengan alamat kontrak dari Ganache
const abiPath = "abi/SensorStorage.abi";

let chart;

async function loadSensorData() {
  const abiRes = await fetch(abiPath);
  const abi = await abiRes.json();

  const provider = new ethers.BrowserProvider(window.ethereum || "http://localhost:8545");
  await provider.send("eth_requestAccounts", []);
  const signer = await provider.getSigner();
  const contract = new ethers.Contract(contractAddress, abi, signer);

  const filter = contract.filters.DataStored();
  const events = await contract.queryFilter(filter, 0, "latest");

  const tableBody = document.querySelector("#sensorTable tbody");
  tableBody.innerHTML = "";

  const labels = [];
  const temps = [];
  const hums = [];

  events.forEach((e) => {
    const data = e.args;
    const timeStr = new Date(Number(data.timestamp) * 1000).toLocaleString();
    const temp = Number(data.temperature) / 100;
    const hum = Number(data.humidity) / 100;

    tableBody.innerHTML += `
      <tr>
        <td>${timeStr}</td>
        <td>${data.sensorId}</td>
        <td>${data.location}</td>
        <td>${data.stage}</td>
        <td>${temp.toFixed(2)}</td>
        <td>${hum.toFixed(2)}</td>
      </tr>
    `;

    labels.push(timeStr);
    temps.push(temp);
    hums.push(hum);
  });

  renderChart(labels, temps, hums);
}

function renderChart(labels, temps, hums) {
  const ctx = document.getElementById('chart').getContext('2d');

  if (chart) chart.destroy();

  chart = new Chart(ctx, {
    type: 'line',
    data: {
      labels,
      datasets: [
        {
          label: "Temperature (°C)",
          data: temps,
          borderColor: 'rgba(255, 99, 132, 1)',
          fill: false
        },
        {
          label: "Humidity (%)",
          data: hums,
          borderColor: 'rgba(54, 162, 235, 1)',
          fill: false
        }
      ]
    },
    options: {
      responsive: true,
      scales: {
        y: { beginAtZero: true }
      }
    }
  });
}
```
### style.css
```
body {
  font-family: sans-serif;
  margin: 20px;
  background: #f9f9f9;
}

h1 {
  color: #333;
}

table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 20px;
  background: white;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

th, td {
  padding: 10px;
  border: 1px solid #ddd;
  text-align: center;
}

button {
  padding: 10px 15px;
  font-size: 16px;
  margin-bottom: 10px;
  cursor: pointer;
}
```
## 4. InfluxDb
1. Install & setup InfluxDB
2. Buat org, bucket, token
3. Siapkan data dalam line protocol
4. Kirim via HTTP POST
5. Cek via InfluxDB UI atau query CLI
   Setelah semua selesai, salin org, bucket dan token yang nantinya dipanggil semua fungsi yang ada di main.rs server.

## 5. GUI pyqt
``` Python
import tkinter as tk
from tkinter import ttk
import requests
import threading
import time
import csv
from io import StringIO
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from matplotlib.figure import Figure
from collections import deque

# === Konfigurasi InfluxDB ===
INFLUX_QUERY_URL = "http://localhost:8086/api/v2/query"
ORG = "gamtenk"
BUCKET = "kelompokgacor"
TOKEN = "yBoxgtA5n1iXvYpHPEiYnUS8ZIkEtZ5QfiTvo3FKgckP2GE4MYyspnH9xTRaKGx-2WMJ7Y6WCk2cOqzfo7R25g=="

# === Riwayat Data Realtime ===
history_length = 50
temp_history = deque(maxlen=history_length)
rh_history = deque(maxlen=history_length)
time_history = deque(maxlen=history_length)

def get_latest_data():
    flux_query = f'''
    from(bucket: "{BUCKET}")
      |> range(start: -1m)
      |> filter(fn: (r) => r._measurement == "monitoring")
      |> filter(fn: (r) => r._field == "temperature" or r._field == "humidity")
      |> last()
    '''
    headers = {
        "Authorization": f"Token {TOKEN}",
        "Content-Type": "application/vnd.flux",
        "Accept": "application/csv"
    }

    try:
        response = requests.post(INFLUX_QUERY_URL, params={"org": ORG}, headers=headers, data=flux_query)
        reader = csv.DictReader(StringIO(response.text))
        data = {}
        for row in reader:
            try:
                field = row["_field"]
                value = float(row["_value"])
                data[field] = value
            except:
                continue

        if "temperature" in data and "humidity" in data:
            return data["temperature"], data["humidity"]
        return None
    except Exception as e:
        print("❌ Exception query Influx:", e)
        return None

def get_data_range(start_time, end_time):
    flux_query = f'''
    from(bucket: "{BUCKET}")
      |> range(start: {start_time}, stop: {end_time})
      |> filter(fn: (r) => r._measurement == "monitoring")
      |> filter(fn: (r) => r._field == "temperature" or r._field == "humidity")
    '''

    headers = {
        "Authorization": f"Token {TOKEN}",
        "Content-Type": "application/vnd.flux",
        "Accept": "application/csv"
    }

    try:
        response = requests.post(INFLUX_QUERY_URL, params={"org": ORG}, headers=headers, data=flux_query)
        reader = csv.DictReader(StringIO(response.text))
        rows = []
        for row in reader:
            try:
                time_str = row["_time"]
                field = row["_field"]
                value = float(row["_value"])
                rows.append((time_str, field, value))
            except:
                continue
        return rows
    except Exception as e:
        print("❌ Exception query Influx:", e)
        return []

def update_data():
    while True:
        result = get_latest_data()
        current_time = time.strftime('%H:%M:%S')

        if result:
            temp, rh = result
            label_temp.config(text=f"Suhu: {temp:.1f} °C")
            label_rh.config(text=f"Kelembaban: {rh:.1f} %")
            status_label.config(text="✅ Data dari Influx")

            temp_history.append(temp)
            rh_history.append(rh)
            time_history.append(current_time)

            plot_graph()
        else:
            label_temp.config(text="Suhu: ---")
            label_rh.config(text="Kelembaban: ---")
            status_label.config(text="❌ Gagal ambil data")

        time.sleep(2)

def plot_graph():
    if not time_history or not temp_history or not rh_history:
        return  # hindari plot kosong

    ax1.clear()
    ax2.clear()

    fig.patch.set_facecolor('black')
    ax1.set_facecolor('black')
    ax2.set_facecolor('black')

    x = list(range(len(time_history)))
    times = list(time_history)
    temps = list(temp_history)
    rhs = list(rh_history)

    ax1.plot(x, temps, label='Suhu (°C)', color='red', marker='o', linestyle='-')
    ax2.plot(x, rhs, label='Kelembaban (%)', color='cyan', marker='x', linestyle='-')

    ax1.set_title("Grafik Suhu", color='white')
    ax2.set_title("Grafik Kelembaban", color='white')
    ax1.set_ylabel("°C", color='white')
    ax2.set_ylabel("%", color='white')

    interval = max(1, len(times) // 5)
    tick_positions = x[::interval]
    tick_labels = times[::interval]

    ax1.set_xticks(tick_positions)
    ax1.set_xticklabels(tick_labels, rotation=45, ha="right", color='white')
    ax2.set_xticks(tick_positions)
    ax2.set_xticklabels(tick_labels, rotation=45, ha="right", color='white')

    for ax in [ax1, ax2]:
        ax.tick_params(axis='y', colors='white')
        ax.tick_params(axis='x', colors='white')
        ax.grid(True, linestyle='--', alpha=0.3, color='gray')
        for spine in ax.spines.values():
            spine.set_color('white')

    fig.tight_layout()
    canvas.draw()

# Untuk mencatat waktu unik yang sudah ditambahkan ke tabel
existing_times = set()

def auto_refresh_table():
    rows = get_data_range("-5m", "now()")
    if rows:
        grouped = {}
        for t, f, v in rows:
            if t not in grouped:
                grouped[t] = {}
            grouped[t][f] = v

        for t in sorted(grouped.keys()):
            if t in existing_times:
                continue  # skip jika sudah ditampilkan
            row = grouped[t]
            temp = row.get("temperature", "--")
            rh = row.get("humidity", "--")
            table.insert("", "end", values=(t[11:19], f"{temp:.1f}", f"{rh:.1f}"))
            existing_times.add(t)

    root.after(5000, auto_refresh_table)

# === GUI ===
root = tk.Tk()
root.title("Monitor SHT20 dari InfluxDB")
root.geometry("900x700")

notebook = ttk.Notebook(root)
tab_realtime = ttk.Frame(notebook)
tab_table = ttk.Frame(notebook)
notebook.add(tab_realtime, text="📡 Realtime")
notebook.add(tab_table, text="📋 Tabel Historis")
notebook.pack(fill="both", expand=True)

# Realtime Widgets
label_temp = tk.Label(tab_realtime, text="Suhu: -- °C", font=("Helvetica", 16))
label_temp.pack(pady=5)
label_rh = tk.Label(tab_realtime, text="Kelembaban: -- %", font=("Helvetica", 16))
label_rh.pack(pady=5)
status_label = tk.Label(tab_realtime, text="Status: ---", fg="blue")
status_label.pack(pady=5)

fig = Figure(figsize=(7, 4.5), dpi=100)
ax1 = fig.add_subplot(211)
ax2 = fig.add_subplot(212)
canvas = FigureCanvasTkAgg(fig, master=tab_realtime)
canvas.get_tk_widget().pack(pady=10)

# Tabel Historis
cols = ("Waktu", "Suhu (°C)", "Kelembaban (%)")
table = ttk.Treeview(tab_table, columns=cols, show="headings", height=20)
for col in cols:
    table.heading(col, text=col)
    table.column(col, width=150, anchor="center")
table.pack(padx=10, pady=10, fill="both", expand=True)

# Start Background Thread dan Refresh
threading.Thread(target=update_data, daemon=True).start()
auto_refresh_table()

# Start App
root.mainloop()
```
**Membuat dashboard realtime berbasis desktop yang:**
1. Mengambil data sensor suhu & kelembaban dari database InfluxDB
2. Menampilkan nilai realtime ke GUI (angka + grafik)
3. Menampilkan histori data dalam bentuk tabel
4. Auto-refresh data secara periodik

## 6. Grafana
1. Pastikan Service berjalan
   ```
   sudo systemctl status influxdb
   sudo systemctl status grafana-server
   ```
3. Akses grafana http://localhost:3000
4. Tambah Data Source
     Configuration (⚙️) → Data Sources → Add data source
     Pilih InfluxDB
     Isi: URL: http://localhost:8086
     Token: (token kamu)
     Org: (org kamu)
     Bucket: (bucket yang sudah kamu buat)
     Save & Test
5. Buat Dashboard
    + Create → Dashboard
    + Add new panel
6. Tulis Flux Query
7. Appky & Save Dashboard
8. Referesh
   
# TAMPILAN DASHBOARD DAN HASIL DATA
## 1. Grafana
!![image](https://github.com/user-attachments/assets/fc79a895-9148-4f00-ae7a-b400bfe0ca95)
## 2. InfluxDb
!![image](https://github.com/user-attachments/assets/4cb78480-ac2e-4f36-bc3a-5ec1641792e0)
## 3. PYQT
!![image](https://github.com/user-attachments/assets/2a4a94ce-efc0-4b55-8929-9657d4ba6f81)
!![image](https://github.com/user-attachments/assets/3a9bf5d9-2b7e-48d9-9592-fed6852a5358)
## 4. Blockchain & Web3
!![image](https://github.com/user-attachments/assets/f2e96352-a7ec-41bc-a4b4-45a6bc5f1762)
!![image](https://github.com/user-attachments/assets/9901e02b-dccb-43f4-b9d8-905088b2fb6d)
!![image](https://github.com/user-attachments/assets/b2da119d-d871-4243-bca9-b04d82025c15)





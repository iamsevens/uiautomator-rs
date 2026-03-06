use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use tokio::runtime::Runtime;
use uiautomator::Device;

fn device_info_response_body() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "displayWidth": 1080,
            "displayHeight": 2400,
            "displayRotation": 0,
            "currentPackageName": "com.example.settings",
            "sdkInt": 34,
            "screenOn": true,
            "naturalOrientation": true
        }
    })
    .to_string()
}

fn handle_connection(stream: &mut TcpStream, body: &str) {
    let mut request_buf = [0_u8; 4096];
    let _ = stream.read(&mut request_buf);

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn spawn_device_info_server() -> (String, Arc<AtomicBool>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind benchmark listener");
    listener
        .set_nonblocking(true)
        .expect("set benchmark listener nonblocking");
    let addr = listener.local_addr().expect("read benchmark listener addr");
    let running = Arc::new(AtomicBool::new(true));
    let running_flag = running.clone();
    let body = device_info_response_body();

    let handle = thread::spawn(move || {
        while running_flag.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => handle_connection(&mut stream, &body),
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(1));
                }
                Err(_) => break,
            }
        }
    });

    (format!("http://{addr}/jsonrpc/0"), running, handle)
}

fn bench_device_info_cache(c: &mut Criterion) {
    let runtime = Runtime::new().expect("create tokio runtime");
    let (rpc_url, running, handle) = spawn_device_info_server();

    let uncached = Device::connect_with_rpc_url(Some("bench-uncached"), &rpc_url)
        .expect("create uncached device");
    let cached =
        Device::connect_with_rpc_url(Some("bench-cached"), &rpc_url).expect("create cached device");
    cached.set_cache_ttl(Duration::from_secs(60));
    runtime
        .block_on(async { cached.info().await })
        .expect("warm cached device");

    let mut group = c.benchmark_group("device_info");
    group.bench_function("uncached", |b| {
        b.iter(|| {
            black_box(
                runtime
                    .block_on(async { uncached.info().await })
                    .expect("uncached info"),
            )
        })
    });
    group.bench_function("cached", |b| {
        b.iter(|| {
            black_box(
                runtime
                    .block_on(async { cached.info().await })
                    .expect("cached info"),
            )
        })
    });
    group.finish();

    running.store(false, Ordering::SeqCst);
    handle.join().expect("join benchmark listener");
}

criterion_group!(benches, bench_device_info_cache);
criterion_main!(benches);

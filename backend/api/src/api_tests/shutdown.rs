//! 停机：收到信号以后最多再等 `shutdown_grace_secs`，还没结束的连接（进度流、发了一半的上传）到点断开。

use std::time::{Duration, Instant};

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::app::spawn_servers;
use crate::config::AppConfig;
use crate::test_support::config::TestDirs;

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map(|addr| addr.port())
        .expect("free port")
}

async fn connect_when_listening(port: u16) -> TcpStream {
    for _ in 0..200 {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)).await {
            return stream;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("server never started listening on {port}");
}

/// 一条一直不结束的连接：上传的请求头说有 1 MB，只发开头几个字节，处理函数就一直等请求体。
/// 以前优雅停机会陪它等到客户端自己断开（`docker stop` 等满 10 秒 SIGKILL）。
///
/// 反证方式：把 `serve_with_shutdown` 里 `drain_deadline` 那一支去掉，这个测试必须变红。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_stops_waiting_for_hung_connections_after_the_grace_period() {
    let dirs = TestDirs::create(&format!("rust-api-shutdown-drain-{}", fastrand::u64(..)));
    let port = free_port();
    let config = AppConfig {
        port,
        simple_port: free_port(),
        shutdown_grace_secs: 1,
        ..dirs.config()
    };
    let servers = spawn_servers(config);

    let mut hung = connect_when_listening(port).await;
    hung.write_all(
        format!(
            "POST /api/v1/uploads HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-API-Key: test-key\r\n\
             Content-Type: multipart/form-data; boundary=x\r\nContent-Length: 1048576\r\n\r\n--x\r\n"
        )
        .as_bytes(),
    )
    .await
    .expect("write partial request");
    // 让服务器把请求读进处理函数。
    tokio::time::sleep(Duration::from_millis(300)).await;

    let started = Instant::now();
    tokio::time::timeout(Duration::from_secs(5), servers.shutdown())
        .await
        .expect("停机必须在等满宽限期之后结束，而不是陪着挂住的连接一直等")
        .expect("servers stopped cleanly");
    assert!(started.elapsed() >= Duration::from_millis(900), "宽限期内应该先等着: {:?}", started.elapsed());
    // 那条连接本身由 axum 的连接任务持有，正式运行时停机返回后进程退出，它随之断开；
    // 这里测试运行时还活着，所以只看停机按时返回。
    drop(hung);
    let _ = std::fs::remove_dir_all(&dirs.root);
}

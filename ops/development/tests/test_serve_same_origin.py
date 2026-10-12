"""`serve_same_origin.py` 的转发行为。

它把前端和 `/api` 挂在同一个端口上（`dev.sh up --same-origin`），所以它转发路径上
的任何缓冲都会直接变成用户可见的卡顿，或者干脆「没有反应」。后端有两处是长连接、
边生成边推送的 SSE：

- 实时翻译事件流 `backend/api/src/routes/jobs/query/live_translation.rs`
- 阅读助手的流式回答 `backend/api/src/services/ai/api.rs`

上游没有 `Content-Length` 时如果先 `read()` 完再发，这两处会一直空着，直到代理的
3600 秒超时；阅读助手也不再逐字显示。下面用「先说一句、停住、再说一句」的桩上游
把这条钉住：**第一段能不能在上游结束之前到达客户端**，是缓冲式和流式实现的分水岭。

放在 `ops/development/tests` 而不是 `frontend/web/tests`：后者是 `node:test`，CI 的
frontend job 也没有配 Python；这个脚本是 `dev.sh` 拉起来的本地栈服务，跟 dev.sh
自己的门禁放一起，归 CI 的 ops tooling job 跑。
"""

from __future__ import annotations

import http.client
import http.server
import socket
import subprocess
import sys
import threading
import time
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[3]
SCRIPT = REPO / "frontend" / "web" / "scripts" / "serve_same_origin.py"

FIRST_EVENT = b"data: first\n\n"
SECOND_EVENT = b"data: second\n\n"
PLAIN_BODY = b'{"ok":true}'


def _start_upstream(released: threading.Event) -> tuple[http.server.ThreadingHTTPServer, int]:
    """桩上游：`/stream` 是 SSE 形状（无 Content-Length，自己分 chunk）。"""

    class Handler(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def do_GET(self) -> None:
            if self.path.endswith("/not-modified"):
                # 后端发文件时会带 ETag，浏览器重新校验就会走到这里。
                self.send_response(304)
                self.send_header("ETag", '"v1"')
                self.end_headers()
                return
            if self.path.endswith("/plain"):
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(PLAIN_BODY)))
                self.end_headers()
                self.wfile.write(PLAIN_BODY)
                return
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Transfer-Encoding", "chunked")
            self.end_headers()
            self._send_chunk(FIRST_EVENT)
            # 停住：只有客户端确实收到了第一段，才放行第二段。
            released.wait(timeout=30)
            self._send_chunk(SECOND_EVENT)
            self.wfile.write(b"0\r\n\r\n")
            self.wfile.flush()

        def _send_chunk(self, data: bytes) -> None:
            self.wfile.write(b"%x\r\n" % len(data) + data + b"\r\n")
            self.wfile.flush()

        def log_message(self, *args) -> None:
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, server.server_address[1]


def _free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def _wait_until_listening(port: int, proc: subprocess.Popen, timeout: float = 20.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            output = proc.stdout.read().decode("utf-8", "replace") if proc.stdout else ""
            raise AssertionError(f"代理没起来就退出了：\n{output}")
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.5):
                return
        except OSError:
            time.sleep(0.05)
    raise AssertionError("代理没有在超时前开始监听")


@pytest.fixture
def proxy(tmp_path: Path):
    released = threading.Event()
    upstream, upstream_port = _start_upstream(released)
    port = _free_port()
    proc = subprocess.Popen(
        [
            sys.executable,
            str(SCRIPT),
            "--port",
            str(port),
            "--root",
            str(tmp_path),
            "--api-base",
            f"http://127.0.0.1:{upstream_port}",
            "--api-key",
            "test-key",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    try:
        _wait_until_listening(port, proc)
        yield port, released
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        upstream.shutdown()
        upstream.server_close()


def test_a_chunk_reaches_the_client_before_the_upstream_finishes(proxy) -> None:
    """上游只发了第一段就停住时，代理必须已经把第一段交出去。

    缓冲式实现在这里读到的是超时：它要等上游把整个响应写完才肯开始转发，
    而 SSE 永远不会写完。
    """
    port, released = proxy
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
    try:
        conn.request("GET", "/api/v1/stream")
        # getresponse 也要包进来：整段缓冲的实现连响应头都不会先发，
        # 卡住的是这一句而不是后面的 read。
        try:
            response = conn.getresponse()
            status = response.status
            first = response.read(len(FIRST_EVENT))
        except TimeoutError:
            pytest.fail(
                "代理把响应攒住了：上游还没结束，客户端就收不到第一段 "
                "（SSE 会一直空着，直到代理超时）"
            )
        assert status == 200
        assert first == FIRST_EVENT

        # 放行第二段，确认整条流仍然完整、且以 chunked 结束。
        released.set()
        assert response.read() == SECOND_EVENT
    finally:
        released.set()
        conn.close()


def test_a_content_length_response_is_forwarded_intact(proxy) -> None:
    """普通响应（带 Content-Length）照旧原样转发，长度头也要保留。"""
    port, _ = proxy
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
    try:
        conn.request("GET", "/api/v1/plain")
        response = conn.getresponse()
        assert response.status == 200
        assert response.getheader("Content-Length") == str(len(PLAIN_BODY))
        assert response.read() == PLAIN_BODY
    finally:
        conn.close()


def test_runtime_config_is_not_cacheable_and_stamped_once(proxy) -> None:
    """runtime-config 仍要 `no-store`，且只能有一个 Cache-Control。

    以前这里有两条来源（本脚本自己发一条，serve_static 也特判一条），叠成两份。
    """
    port, _ = proxy
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
    try:
        conn.request("GET", "/runtime-config.local.js")
        response = conn.getresponse()
        assert response.status == 200
        assert response.headers.get_all("Cache-Control") == ["no-store"]
        assert b"window.location.origin" in response.read()
    finally:
        conn.close()


def test_a_304_does_not_poison_the_next_response_on_the_same_connection(proxy) -> None:
    """304 没有响应体，代理不能替它写 chunked 结尾。

    多写的那几个字节浏览器不会读，它们会留在 keep-alive 连接上，被当成下一个
    响应的开头 —— 第二个请求于是读到一个坏响应。
    """
    port, _ = proxy
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
    try:
        conn.request("GET", "/api/v1/not-modified", headers={"If-None-Match": '"v1"'})
        first = conn.getresponse()
        assert first.status == 304
        assert first.getheader("Transfer-Encoding") is None, "304 不该带 Transfer-Encoding"
        assert first.read() == b""

        # 同一条连接上的下一个响应必须还能正常解析。
        conn.request("GET", "/api/v1/plain")
        second = conn.getresponse()
        assert second.status == 200, "304 之后的下一个响应被读坏了"
        assert second.read() == PLAIN_BODY
    finally:
        conn.close()


def test_api_responses_do_not_pick_up_the_static_cache_header(proxy) -> None:
    """转发出去的 API 响应不能被套上静态层那份 `public, max-age=3600`。

    上游不设 Cache-Control 时，代理也不能补一个 —— 任务列表 / 详情被浏览器缓存
    一小时，表现就是进度不动、新任务不出现。Server / Date 同理只保留一份。
    """
    port, _ = proxy
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
    try:
        conn.request("GET", "/api/v1/plain")
        response = conn.getresponse()
        assert response.status == 200
        assert response.getheader("Cache-Control") is None, "代理给 API 响应加了缓存头"
        assert len(response.headers.get_all("Server") or []) == 1, "Server 头出现了两份"
        assert len(response.headers.get_all("Date") or []) == 1, "Date 头出现了两份"
        assert response.read() == PLAIN_BODY
    finally:
        conn.close()

"""Docker 交付包的占位 key 换成随机 key：app 入口的 api-key.sh 和 web 的 15-backend-api-key.envsh。

两段脚本都用 sh 真跑一遍（不起容器），再静态核对 compose、两个 Dockerfile 把它们接上了。
"""

from __future__ import annotations

import json
import os
import re
import stat
import subprocess
from pathlib import Path

import pytest


REPO_ROOT = Path(__file__).resolve().parents[3]
APP_SCRIPT = REPO_ROOT / "ops/deployment/docker/backend/api-key.sh"
WEB_SCRIPT = REPO_ROOT / "ops/deployment/docker/backend-api-key.envsh"
PLACEHOLDER = "replace-with-your-backend-key"


def _text(relative: str) -> str:
    return (REPO_ROOT / relative).read_text(encoding="utf-8")


def _base_env() -> dict[str, str]:
    # 只留 PATH，避免本机的 RUST_API_KEYS 之类漏进来。
    return {"PATH": os.environ["PATH"]}


def _run_app(auth_file: Path, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "sh",
            "-c",
            'set -eu; . "$0"; retainpdf_resolve_api_key "$1"; printf %s "${RUST_API_KEYS-<unset>}"',
            str(APP_SCRIPT),
            str(auth_file),
        ],
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )


def _run_web(env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["sh", "-c", '. "$0"; printf %s "${RETAINPDF_PROXY_API_KEY-<unset>}"', str(WEB_SCRIPT)],
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )


def _auth_file(tmp_path: Path, keys: list[str]) -> Path:
    path = tmp_path / "auth.local.json"
    path.write_text(json.dumps({"api_keys": keys}), encoding="utf-8")
    return path


def test_placeholder_is_replaced_by_a_generated_key_that_survives_restarts(tmp_path: Path) -> None:
    key_dir = tmp_path / "key"
    key_dir.mkdir()
    env = {**_base_env(), "RETAINPDF_SHARED_KEY_DIR": str(key_dir)}
    auth = _auth_file(tmp_path, [PLACEHOLDER])

    first = _run_app(auth, env)
    assert re.fullmatch(r"[0-9a-f]{64}", first.stdout), first
    key_file = key_dir / "backend-api-key"
    assert key_file.read_text(encoding="utf-8") == first.stdout
    assert stat.S_IMODE(key_file.stat().st_mode) == 0o600
    assert first.stdout not in first.stderr, "the key must not be printed"

    second = _run_app(auth, env)
    assert second.stdout == first.stdout


def test_without_the_shared_directory_the_placeholder_is_kept_with_a_warning(tmp_path: Path) -> None:
    # 旧 compose：没有 RETAINPDF_SHARED_KEY_DIR，照旧用占位 key（web 那边也还是占位 key，两边对得上）。
    result = _run_app(_auth_file(tmp_path, [PLACEHOLDER]), _base_env())
    assert result.stdout == "<unset>"
    assert "WARNING" in result.stderr

    env = {**_base_env(), "RETAINPDF_SHARED_KEY_DIR": str(tmp_path / "missing")}
    result = _run_app(_auth_file(tmp_path, [PLACEHOLDER]), env)
    assert result.stdout == "<unset>"
    assert "WARNING" in result.stderr


@pytest.mark.parametrize("keys", [["my-own-backend-key-123"], [PLACEHOLDER, "my-own-backend-key-123"]])
def test_operator_keys_are_left_alone(tmp_path: Path, keys: list[str]) -> None:
    key_dir = tmp_path / "key"
    key_dir.mkdir()
    env = {**_base_env(), "RETAINPDF_SHARED_KEY_DIR": str(key_dir)}
    result = _run_app(_auth_file(tmp_path, keys), env)
    assert result.stdout == "<unset>"
    assert result.stderr == ""
    assert not (key_dir / "backend-api-key").exists()


def test_explicit_rust_api_keys_wins(tmp_path: Path) -> None:
    key_dir = tmp_path / "key"
    key_dir.mkdir()
    env = {**_base_env(), "RETAINPDF_SHARED_KEY_DIR": str(key_dir), "RUST_API_KEYS": "set-by-operator-0001"}
    result = _run_app(_auth_file(tmp_path, [PLACEHOLDER]), env)
    assert result.stdout == "set-by-operator-0001"
    assert not (key_dir / "backend-api-key").exists()


def test_web_uses_the_generated_key_only_in_place_of_the_placeholder(tmp_path: Path) -> None:
    key_dir = tmp_path / "key"
    key_dir.mkdir()
    (key_dir / "backend-api-key").write_text("f" * 64, encoding="utf-8")
    shared = {**_base_env(), "RETAINPDF_SHARED_KEY_DIR": str(key_dir)}

    assert _run_web({**shared, "RETAINPDF_PROXY_API_KEY": PLACEHOLDER}).stdout == "f" * 64
    # 多用户叠加配置清空它，不让 NGINX 补 key：必须保持空。
    assert _run_web({**shared, "RETAINPDF_PROXY_API_KEY": ""}).stdout == ""
    assert _run_web({**shared, "RETAINPDF_PROXY_API_KEY": "my-own-backend-key-123"}).stdout == "my-own-backend-key-123"
    # 旧 compose 没挂共享目录。
    assert _run_web({**_base_env(), "RETAINPDF_PROXY_API_KEY": PLACEHOLDER}).stdout == PLACEHOLDER

    (key_dir / "backend-api-key").unlink()
    missing = _run_web({**shared, "RETAINPDF_PROXY_API_KEY": PLACEHOLDER})
    assert missing.stdout == PLACEHOLDER
    assert "WARNING" in missing.stderr


def test_placeholder_literal_matches_the_bundle_and_the_backend() -> None:
    auth = json.loads(_text("ops/deployment/docker/delivery/docker/auth.local.json"))
    assert auth["api_keys"] == [PLACEHOLDER]
    assert f"RETAINPDF_PROXY_API_KEY={PLACEHOLDER}\n" in _text("ops/deployment/docker/delivery/docker/web.env")
    assert f'RETAINPDF_PLACEHOLDER_API_KEY="{PLACEHOLDER}"' in APP_SCRIPT.read_text(encoding="utf-8")
    assert f'= "{PLACEHOLDER}" ]' in WEB_SCRIPT.read_text(encoding="utf-8")
    rust = _text("backend/packages/retainpdf-core/src/config/auth.rs")
    assert f'pub const DELIVERY_PLACEHOLDER_API_KEY: &str = "{PLACEHOLDER}";' in rust


def test_compose_and_images_wire_the_shared_key_directory() -> None:
    compose = _text("ops/deployment/docker/delivery/docker-compose.yml")
    app = compose[compose.index("\n  app:\n") : compose.index("\n  web:\n")]
    web = compose[compose.index("\n  web:\n") : compose.index("\nvolumes:\n")]
    assert "RETAINPDF_SHARED_KEY_DIR: /run/retainpdf-key" in app
    assert "- api_key:/run/retainpdf-key\n" in app
    assert "RETAINPDF_SHARED_KEY_DIR: /run/retainpdf-key" in web
    assert "- api_key:/run/retainpdf-key:ro\n" in web
    assert "\n  api_key:\n" in compose[compose.index("\nvolumes:\n") :]

    app_dockerfile = _text("ops/deployment/docker/backend/Dockerfile.app")
    assert "COPY ops/deployment/docker/backend/api-key.sh /usr/local/lib/retainpdf/api-key.sh" in app_dockerfile
    assert "chown -R retainpdf:retainpdf /data /tmp/retainpdf /run/retainpdf-key" in app_dockerfile
    entrypoint = _text("ops/deployment/docker/backend/entrypoint-app.sh")
    resolve = entrypoint.index('retainpdf_resolve_api_key "$RUST_API_ROOT/auth.local.json"')
    assert entrypoint.index(". /usr/local/lib/retainpdf/api-key.sh") < resolve < entrypoint.index("exec /usr/local/bin/rust_api")

    web_dockerfile = _text("ops/deployment/docker/Dockerfile.web")
    match = re.search(r"COPY ops/deployment/docker/backend-api-key\.envsh /docker-entrypoint\.d/(\d+)-[\w-]+\.envsh", web_dockerfile)
    assert match, "web image does not install the key script"
    # 必须排在官方的 20-envsubst-on-templates.sh 前面，模板展开时才拿得到换过的 key。
    assert int(match.group(1)) < 20
    assert "/docker-entrypoint.d/15-backend-api-key.envsh" in web_dockerfile.split("RUN chmod +x", 1)[1].splitlines()[0]

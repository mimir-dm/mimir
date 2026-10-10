"""Web app tasks (COLLIERY-I-0612): the Leptos app in crates/mimir-web,
built with trunk, served by mimir-server.

Commands:
- angreal web build: build the wasm bundle into crates/mimir-web/dist
- angreal web serve: run mimir-server on the fixture data with the bundle,
  and rebuild the bundle when the code changes
"""
import os
import shutil
import subprocess
import sys
from pathlib import Path

import angreal

PROJECT_ROOT = Path(angreal.get_root()).parent
WEB_CRATE = PROJECT_ROOT / "crates" / "mimir-web"
WEB_DIST = WEB_CRATE / "dist"
# Dev data for `web serve`: fixture data only, never real campaigns.
DEV_DATA_DIR = PROJECT_ROOT / "target" / "web-dev"
WASM_TARGET = "wasm32-unknown-unknown"

web = angreal.command_group(name="web", about="Web app (mimir-web) commands")


def _ensure_wasm_target() -> int:
    """`rustup target add` does nothing when the target is there."""
    return subprocess.run(["rustup", "target", "add", WASM_TARGET], cwd=str(PROJECT_ROOT)).returncode


def _trunk():
    trunk = shutil.which("trunk")
    if trunk is None:
        print("trunk is not installed. Install it: cargo install trunk --locked", file=sys.stderr)
    return trunk


def _build(release: bool) -> int:
    if _ensure_wasm_target() != 0:
        print(f"rustup target add {WASM_TARGET} failed", file=sys.stderr)
        return 1
    trunk = _trunk()
    if trunk is None:
        return 1
    cmd = [trunk, "build"] + (["--release"] if release else [])
    print(f"Building mimir-web ({'release' if release else 'dev'})...")
    result = subprocess.run(cmd, cwd=str(WEB_CRATE))
    if result.returncode != 0:
        print("trunk build failed", file=sys.stderr)
        return result.returncode
    print(f"Bundle written to {WEB_DIST}")
    return 0


@web()
@angreal.command(
    name="build",
    about="Build the web app (trunk) into crates/mimir-web/dist",
    tool=angreal.ToolDescription(
        """
        Build the mimir-web wasm bundle with trunk. Adds the
        wasm32-unknown-unknown target if it is missing.

        To serve the bundle: run mimir-server with
        MIMIR_WEB_DIST=crates/mimir-web/dist, or build mimir-server with
        `--features embed-web` after `angreal web build --release`.

        ## Examples
        ```
        angreal web build            # dev profile
        angreal web build --release  # optimized bundle (what ships)
        ```
        """,
        risk_level="safe",
    ),
)
@angreal.argument(name="release", long="release", takes_value=False, is_flag=True, help="Optimized build (what ships)")
def build(release=False):
    return _build(release)


@web()
@angreal.command(
    name="serve",
    about="Run mimir-server on fixture data with the web app; rebuild on change",
    tool=angreal.ToolDescription(
        """
        Build the bundle, start mimir-server (built with the `fixtures`
        feature) with MIMIR_SEED=fixture and the data in target/web-dev,
        then run `trunk watch` so a code change rebuilds the bundle. Reload
        the browser after a rebuild. Ctrl-C stops both.

        Without --token the server is in open mode (no sign-in).

        ## Examples
        ```
        angreal web serve                       # http://127.0.0.1:8740
        angreal web serve --port 8750
        angreal web serve --token dev-token     # test the sign-in page
        angreal web serve --fresh               # delete target/web-dev first
        ```
        """,
        risk_level="safe",
    ),
)
@angreal.argument(name="port", long="port", short="p", default="8740", help="Port on 127.0.0.1 (default 8740)")
@angreal.argument(name="token", long="token", help="DM token (default: open mode)")
@angreal.argument(name="fresh", long="fresh", takes_value=False, is_flag=True, help="Delete the dev data first")
def serve(port="8740", token=None, fresh=False):
    if fresh and DEV_DATA_DIR.exists():
        shutil.rmtree(DEV_DATA_DIR)
        print(f"Deleted {DEV_DATA_DIR}")
    code = _build(False)
    if code != 0:
        return code

    env = dict(os.environ)
    env.update(
        MIMIR_BIND=f"127.0.0.1:{port}",
        MIMIR_DATA_DIR=str(DEV_DATA_DIR),
        MIMIR_WEB_DIST=str(WEB_DIST),
        MIMIR_SEED="fixture",
    )
    env.pop("MIMIR_API_TOKEN", None)
    if token:
        env["MIMIR_API_TOKEN"] = token

    server = subprocess.Popen(
        ["cargo", "run", "-p", "mimir-server", "--features", "fixtures"],
        cwd=str(PROJECT_ROOT),
        env=env,
    )
    print(f"mimir-server: http://127.0.0.1:{port}  (data: {DEV_DATA_DIR})")
    try:
        return subprocess.run([_trunk(), "watch"], cwd=str(WEB_CRATE)).returncode
    except KeyboardInterrupt:
        return 0
    finally:
        server.terminate()
        try:
            server.wait(timeout=10)
        except subprocess.TimeoutExpired:
            server.kill()

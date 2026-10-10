"""Web app tasks (COLLIERY-I-0612): the Leptos app in crates/mimir-web,
built with trunk, served by mimir-server.

Commands:
- angreal web build: build the wasm bundle into crates/mimir-web/dist
- angreal web serve: run mimir-server on the fixture data with the bundle,
  and rebuild the bundle when the code changes
- angreal web e2e: the Playwright smoke suite (crates/mimir-web/web-e2e)
- angreal web screenshots: capture the screens into a new run directory
- angreal web screenshots-diff: compare two capture runs
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
def serve(port=None, token=None, fresh=False):
    port = port or "8740"
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


E2E_DIR = WEB_CRATE / "web-e2e"
RUNS_DIR = E2E_DIR / "screenshots" / "runs"


def _npm_ready() -> int:
    if (E2E_DIR / "node_modules").exists():
        return 0
    return subprocess.run(["npm", "ci", "--no-audit", "--no-fund"], cwd=str(E2E_DIR)).returncode


@web()
@angreal.command(
    name="e2e",
    about="Run the web smoke suite (Playwright) against mimir-server on fixture data",
    tool=angreal.ToolDescription(
        """
        Build the bundle, then run the Playwright smoke suite in
        crates/mimir-web/web-e2e. Playwright starts mimir-server (web-e2e/
        session.sh) on a new scratch data dir seeded with the UI fixture, in
        token mode, and removes the dir after.

        ## Examples
        ```
        angreal web e2e
        ```
        """,
        risk_level="safe",
    ),
)
def e2e():
    code = _build(False) or _npm_ready()
    if code != 0:
        return code
    return subprocess.run(["npx", "playwright", "test", "specs/smoke.spec.ts", "specs/map.spec.ts"], cwd=str(E2E_DIR)).returncode


@web()
@angreal.command(
    name="screenshots",
    about="Capture the web app screens (desktop and tablet, per theme) into a new run directory",
    tool=angreal.ToolDescription(
        """
        Build the bundle and capture each screen at desktop (1280x800) and
        tablet (820x1100) width in each theme, into
        crates/mimir-web/web-e2e/screenshots/runs/<timestamp>/. Compare two
        runs with `angreal web screenshots-diff`.

        ## Examples
        ```
        angreal web screenshots                         # light and dark
        angreal web screenshots --themes light,dark,hyper
        ```
        """,
        risk_level="safe",
    ),
)
@angreal.argument(name="themes", long="themes", short="t", default="light,dark", help="Comma-separated themes: light, dark, hyper (default light,dark)")
def screenshots(themes=None):
    themes = themes or "light,dark"
    code = _build(False) or _npm_ready()
    if code != 0:
        return code
    from datetime import datetime

    run_dir = RUNS_DIR / datetime.now().strftime("%Y%m%d-%H%M%S")
    env = dict(os.environ, SCREENSHOT_DIR=str(run_dir), THEMES=themes)
    code = subprocess.run(["npx", "playwright", "test", "specs/capture.spec.ts"], cwd=str(E2E_DIR), env=env).returncode
    print(f"Screenshots: {run_dir}")
    return code


@web()
@angreal.command(
    name="screenshots-diff",
    about="Compare two screenshot runs pixel by pixel and write diff images",
    tool=angreal.ToolDescription(
        """
        Compare two runs of `angreal web screenshots`. Prints the changed
        pixels per image and writes diff images to <after>/diff.

        ## Examples
        ```
        angreal web screenshots-diff --before 20261010-120000
        angreal web screenshots-diff --before 20261010-120000 --after 20261010-130000 --fail
        ```
        """,
        risk_level="safe",
    ),
)
@angreal.argument(name="before", long="before", short="b", help="Run to compare from (a run name under web-e2e/screenshots/runs, or a path)")
@angreal.argument(name="after", long="after", short="a", help="Run to compare to (default: the latest run)")
@angreal.argument(name="fail", long="fail", takes_value=False, is_flag=True, help="Exit 1 when an image changed, is new or is missing")
def screenshots_diff(before=None, after=None, fail=False):
    def resolve(name):
        path = Path(name)
        return path if path.is_absolute() or path.exists() else RUNS_DIR / name

    if not before:
        print("--before is required", file=sys.stderr)
        return 2
    if after:
        after_dir = resolve(after)
    else:
        runs = sorted(p for p in RUNS_DIR.iterdir() if p.is_dir()) if RUNS_DIR.exists() else []
        if not runs:
            print("No runs. Run: angreal web screenshots", file=sys.stderr)
            return 1
        after_dir = runs[-1]
    if _npm_ready() != 0:
        return 1
    cmd = ["node", "diff.mjs", str(resolve(before)), str(after_dir)] + (["--fail"] if fail else [])
    return subprocess.run(cmd, cwd=str(E2E_DIR)).returncode

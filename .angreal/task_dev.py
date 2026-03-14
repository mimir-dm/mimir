"""Dev environment management tasks for Mimir project

Commands:
- angreal dev reset: Reset the dev database (delete and let app recreate)
- angreal dev launch: Launch the dev UI (Vite + Tauri)
"""
import angreal
import subprocess
from pathlib import Path
import sys
import os

PROJECT_ROOT = Path(angreal.get_root()).parent
FRONTEND_DIR = PROJECT_ROOT / "crates" / "mimir" / "frontend"

# Dev paths (matches crates/mimir/src/state.rs dev mode logic)
DEV_DIR = Path.home() / "Library" / "Application Support" / "com.mimir.app" / "dev"
DEV_DATA_DIR = DEV_DIR / "data"
DEV_DB_PATH = DEV_DATA_DIR / "mimir.db"
DEV_ASSETS_DIR = DEV_DIR / "assets"

# Production database path - NEVER touch this
PROD_DATA_DIR = Path.home() / "Library" / "Application Support" / "com.mimir.app" / "data"
PROD_DB_PATH = PROD_DATA_DIR / "mimir.db"

dev = angreal.command_group(name="dev", about="Dev environment management for Mimir")


@dev()
@angreal.command(name="reset", about="Reset the dev database (deletes dev DB only, never production)")
def reset():
    """Delete the dev database so the app creates a fresh one on next launch."""
    if DEV_DB_PATH.exists():
        size_mb = DEV_DB_PATH.stat().st_size / (1024 * 1024)
        print(f"Deleting dev database: {DEV_DB_PATH}")
        print(f"  Size: {size_mb:.1f} MB")
        DEV_DB_PATH.unlink()
        print("Dev database deleted. Restart the app to create a fresh one.")
    else:
        print(f"No dev database found at {DEV_DB_PATH}")

    # Clean extracted map images (JPEG/PNG generated from UVTT files)
    if DEV_ASSETS_DIR.exists():
        import shutil
        asset_count = sum(1 for _ in DEV_ASSETS_DIR.iterdir())
        if asset_count > 0:
            shutil.rmtree(DEV_ASSETS_DIR)
            DEV_ASSETS_DIR.mkdir(parents=True, exist_ok=True)
            print(f"Cleared {asset_count} dev assets.")

    # Safety check: confirm prod DB is untouched
    if PROD_DB_PATH.exists():
        print(f"\nProduction database is safe at {PROD_DB_PATH}")
    else:
        print(f"\nWARNING: No production database found at {PROD_DB_PATH}")


@dev()
@angreal.command(name="launch", about="Launch the dev UI (Vite dev server + Tauri app)")
def launch():
    """Start the Vite dev server and Tauri app for development."""
    # Check frontend dependencies
    if not (FRONTEND_DIR / "node_modules").exists():
        print("Installing frontend dependencies...")
        subprocess.run(["npm", "ci"], cwd=FRONTEND_DIR, check=True)

    # Start Vite dev server in background
    print("Starting Vite dev server...")
    vite = subprocess.Popen(
        ["npm", "run", "dev"],
        cwd=FRONTEND_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    try:
        # Give Vite a moment to start
        import time
        time.sleep(2)

        # Check sidecar binary exists
        sidecar_dir = PROJECT_ROOT / "crates" / "mimir" / "binaries"
        if not any(sidecar_dir.glob("mimir-mcp-*")):
            print("Building mimir-mcp sidecar...")
            subprocess.run(
                ["bash", str(PROJECT_ROOT / "scripts" / "build-sidecar.sh")],
                cwd=PROJECT_ROOT,
                check=True,
            )

        # Run the Tauri app
        print("Launching Mimir...")
        subprocess.run(
            ["cargo", "run", "-p", "mimir", "--no-default-features"],
            cwd=PROJECT_ROOT,
        )
    finally:
        print("Shutting down Vite dev server...")
        vite.terminate()
        vite.wait()

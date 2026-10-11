"""
Docs tasks: `angreal docs build`, `angreal docs serve` and `angreal docs clean`.

The docs are an Astro Starlight site in docs/, with the Colliery theme
starlight-aurora (MIMIR-T-0735). They need Node 22 or later.
"""

import shutil
import subprocess
import sys
from pathlib import Path

import angreal  # type: ignore

PROJECT_ROOT = Path(angreal.get_root()).parent
DOCS_ROOT = PROJECT_ROOT / "docs"

docs = angreal.command_group(name="docs", about="documentation site commands")


def _npm(*args):
    npm = shutil.which("npm")
    if not npm:
        print("npm not found: install Node 22 or later", file=sys.stderr)
        return 1
    if not (DOCS_ROOT / "node_modules").exists():
        result = subprocess.run([npm, "ci", "--no-audit", "--no-fund"], cwd=str(DOCS_ROOT))
        if result.returncode != 0:
            return result.returncode
    return subprocess.run([npm, *args], cwd=str(DOCS_ROOT)).returncode


@docs()
@angreal.command(
    name="build",
    about="build the docs site into docs/dist",
    tool=angreal.ToolDescription(
        "Builds the Starlight docs site in docs/ into docs/dist. Installs the "
        "npm dependencies on the first run. Fails when a page does not build.",
        risk_level="safe",
    ),
)
def docs_build():
    return _npm("run", "build")


@docs()
@angreal.command(
    name="serve",
    about="serve the docs site with live reload",
    tool=angreal.ToolDescription(
        "Serves the docs site at http://localhost:4321 and reloads on each "
        "change. In a terminal it runs until Ctrl+C. From a script or an agent, "
        "Astro runs it in the background: stop it with `npx astro dev stop` in docs/.",
        risk_level="safe",
    ),
)
def docs_serve():
    return _npm("run", "dev")


@docs()
@angreal.command(
    name="clean",
    about="remove the built docs site",
    tool=angreal.ToolDescription(
        "Removes docs/dist, the output of `angreal docs build`.",
        risk_level="safe",
    ),
)
def docs_clean():
    dist = DOCS_ROOT / "dist"
    if dist.exists():
        shutil.rmtree(dist)
        print(f"removed {dist}")
    return 0

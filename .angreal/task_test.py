"""Test-related Angreal tasks for Mimir project

Test Organization:
- Unit tests: Located in src/ files with #[cfg(test)] modules, run with --lib
- Integration tests: Located in tests/ directories, run with --test
- The web app: `angreal web e2e` (Playwright against the server)

Commands:
- angreal test unit: Run the Rust tests of the workspace
- angreal test coverage: Generate the coverage report of mimir-core
"""
import angreal
import subprocess
from pathlib import Path
import sys

# Get the project root directory
PROJECT_ROOT = Path(angreal.get_root()).parent

# Create a test command group
test = angreal.command_group(name="test", about="Testing commands for the Mimir workspace")

@test()
@angreal.command(name="unit", about="Run the Rust tests of the workspace")
def unit():
    """Run the Rust tests (unit + integration) of every crate."""
    print("\nRunning Rust tests (unit + integration)...")
    result = subprocess.run(
        ["cargo", "test", "--workspace", "--", "--test-threads=1"],
        cwd=PROJECT_ROOT,
        capture_output=False
    )
    if result.returncode != 0:
        print("\nTest failures.")
        sys.exit(1)
    print("\nAll tests passed!")

@test()
@angreal.command(name="coverage", about="Run tests with coverage reporting")
@angreal.argument(name="open", long="open", short="o", takes_value=False, is_flag=True, help="Open coverage reports in browser")
def coverage(open: bool = False):
    """Run the mimir-core tests with code coverage (tarpaulin.toml)."""
    print("Running tests with coverage...")

    # Check if cargo-tarpaulin is installed
    check_result = subprocess.run(
        ["cargo", "tarpaulin", "--version"],
        capture_output=True,
        cwd=PROJECT_ROOT
    )
    if check_result.returncode != 0:
        print("cargo-tarpaulin not found. Installing...")
        subprocess.run(["cargo", "install", "cargo-tarpaulin"], cwd=PROJECT_ROOT)

    result = subprocess.run(
        ["cargo", "tarpaulin", "--config", "tarpaulin.toml"],
        cwd=PROJECT_ROOT,
        capture_output=False
    )
    if result.returncode != 0:
        print("\nCoverage failed.")
        sys.exit(1)
    print("Coverage report: target/coverage/tarpaulin-report.html")
    if open:
        subprocess.run(["open", "target/coverage/tarpaulin-report.html"], cwd=PROJECT_ROOT)
    else:
        print("Use --open flag to view reports in browser")

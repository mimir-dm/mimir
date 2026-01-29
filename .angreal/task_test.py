"""Test-related Angreal tasks for Mimir project

Test Organization:
- Unit tests: Located in src/ files with #[cfg(test)] modules, run with --lib
- Integration tests: Located in tests/ directories, run with --test
- Frontend tests: Located in frontend/, run with npm test

Commands:
- angreal test: Run all tests (backend + frontend)
- angreal test --watch: Run tests in watch mode
- angreal test --coverage: Generate coverage reports
"""
import angreal
import subprocess
from pathlib import Path
import sys

# Get the project root directory
PROJECT_ROOT = Path(angreal.get_root()).parent
FRONTEND_DIR = PROJECT_ROOT / "crates" / "mimir" / "frontend"

# Create a test command group
test = angreal.command_group(name="test", about="Testing commands for the Mimir workspace")

@test()
@angreal.command(name="unit", about="Run tests")
@angreal.argument(name="watch", long="watch", short="w", takes_value=False, help="Run tests in watch mode")
@angreal.argument(name="core", long="core", short="c", takes_value=False, help="Run core/CLI crate tests only (default)")
@angreal.argument(name="ui", long="ui", short="u", takes_value=False, help="Run UI/frontend tests only")
@angreal.argument(name="all", long="all", short="a", takes_value=False, help="Run all tests (core + UI)")
def unit(watch: bool = False, core: bool = False, ui: bool = False, all: bool = False):
    """Run tests (core only by default, or UI only, or all)"""
    failures = []

    # Determine what to run:
    # - If --all is set, run both
    # - If --ui is set, run only UI
    # - Otherwise (default or --core), run only core
    if all:
        run_core = True
        run_ui = True
    elif ui:
        run_core = False
        run_ui = True
    else:  # default or --core
        run_core = True
        run_ui = False

    # Run core crate tests if requested
    if run_core:
        print("\nRunning Rust tests (unit + integration) for core crates...")
        result = subprocess.run(
            ["cargo", "test", "--workspace", "--exclude", "mimir", "--", "--test-threads=1"],
            cwd=PROJECT_ROOT,
            capture_output=False
        )
        if result.returncode != 0:
            failures.append("Core tests")

    # Run UI/frontend tests if requested
    if run_ui:
        print("\nRunning frontend tests...")
        if not FRONTEND_DIR.exists():
            print(f"Frontend directory not found: {FRONTEND_DIR}")
            sys.exit(1)

        if not (FRONTEND_DIR / "node_modules").exists():
            print("Installing frontend dependencies...")
            subprocess.run(["npm", "install"], cwd=FRONTEND_DIR)

        cmd = ["npm", "test"]
        if not watch:
            cmd.extend(["--", "--run"])

        result = subprocess.run(cmd, cwd=FRONTEND_DIR, capture_output=False)
        if result.returncode != 0:
            failures.append("UI tests")

    # Summary
    if failures:
        print(f"\nTest failures in: {', '.join(failures)}")
        sys.exit(1)
    else:
        print("\nAll tests passed!")

@test()
@angreal.command(name="coverage", about="Run tests with coverage reporting")
@angreal.argument(name="core", long="core", short="c", takes_value=False, help="Run core crate coverage only (default)")
@angreal.argument(name="ui", long="ui", short="u", takes_value=False, help="Run UI/frontend coverage only")
@angreal.argument(name="all", long="all", short="a", takes_value=False, help="Run all coverage (core + UI)")
@angreal.argument(name="open", long="open",short="o", takes_value=False, help="Open coverage reports in browser")
def coverage(core: bool = False, ui: bool = False, all: bool = False, open: bool = False):
    """Run tests with code coverage (core only by default, or UI only, or all)"""
    print("Running tests with coverage...")

    failures = []

    # Determine what to run (same logic as unit tests)
    if all:
        run_core = True
        run_ui = True
    elif ui:
        run_core = False
        run_ui = True
    else:  # default or --core
        run_core = True
        run_ui = False

    # Run core coverage if requested
    if run_core:
        print("\n[Core Coverage (Rust)]")

        # Check if cargo-tarpaulin is installed
        check_result = subprocess.run(
            ["cargo", "tarpaulin", "--version"],
            capture_output=True,
            cwd=PROJECT_ROOT
        )

        if check_result.returncode != 0:
            print("cargo-tarpaulin not found. Installing...")
            subprocess.run(["cargo", "install", "cargo-tarpaulin"], cwd=PROJECT_ROOT)

        # Run tarpaulin using the config file
        cmd = [
            "cargo", "tarpaulin",
            "--config", "tarpaulin.toml"
        ]

        result = subprocess.run(cmd, cwd=PROJECT_ROOT, capture_output=False)
        if result.returncode != 0:
            failures.append("Core coverage")
        else:
            print("Core coverage report: target/coverage/tarpaulin-report.html")
            if open:
                subprocess.run(["open", "target/coverage/tarpaulin-report.html"], cwd=PROJECT_ROOT)

    # Run UI coverage if requested
    if run_ui:
        print("\n[Frontend Coverage (Vue/TypeScript)]")
        
        # Check if coverage package is installed
        check_result = subprocess.run(
            ["npm", "list", "@vitest/coverage-v8"],
            capture_output=True,
            cwd=FRONTEND_DIR
        )
        
        if check_result.returncode != 0:
            print("Installing @vitest/coverage-v8...")
            subprocess.run(["npm", "install", "--save-dev", "@vitest/coverage-v8"], cwd=FRONTEND_DIR)
        
        # Run frontend coverage
        result = subprocess.run(
            ["npm", "run", "test:coverage"],
            cwd=FRONTEND_DIR,
            capture_output=False
        )
        if result.returncode != 0:
            failures.append("UI coverage")
        else:
            print("UI coverage report: crates/mimir/frontend/coverage/index.html")
            if open:
                subprocess.run(["open", "crates/mimir/frontend/coverage/index.html"], cwd=PROJECT_ROOT)
    
    # Summary
    if not failures:
        print("\nAll coverage reports generated successfully!")
        if not open:
            print("Use --open flag to view reports in browser")
    else:
        print(f"\nFailed: {', '.join(failures)}")
        sys.exit(1)
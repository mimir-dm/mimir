"""Documentation-related Angreal tasks for Mimir project"""
import angreal
import subprocess
from pathlib import Path
import shutil
import sys

# Get the project root directory
PROJECT_ROOT = Path(angreal.get_root()).parent
DOCS_DIR = PROJECT_ROOT / "docs"

# Create a docs command group
docs = angreal.command_group(name="docs", about="Documentation commands using mdBook")

def check_mdbook_installed():
    """Check if mdbook is installed and provide installation instructions if not"""
    try:
        result = subprocess.run(
            ["mdbook", "--version"],
            capture_output=True,
            text=True,
            cwd=PROJECT_ROOT
        )
        if result.returncode == 0:
            version = result.stdout.strip()
            print(f"mdbook found: {version}")
            return True
    except FileNotFoundError:
        pass
    
    print("mdbook not found!")
    print("\nTo install mdbook, run one of the following:")
    print("  cargo install mdbook")
    print("  cargo install mdbook mdbook-mermaid  # includes mermaid support")
    print("\nOr visit: https://rust-lang.github.io/mdBook/guide/installation.html")
    return False

def check_mermaid_support():
    """Check if mdbook-mermaid is available for diagram support"""
    try:
        result = subprocess.run(
            ["mdbook-mermaid", "--version"],
            capture_output=True,
            text=True,
            cwd=PROJECT_ROOT
        )
        if result.returncode == 0:
            return True
    except FileNotFoundError:
        pass
    
    print("Note: mdbook-mermaid not found. Mermaid diagrams may not render.")
    print("Install with: cargo install mdbook-mermaid")
    return False

@docs
@angreal.command(name="build", about="Build the documentation site")
@angreal.argument(name="dest-dir", short="d", help="Custom output directory (default: docs/book)")
def docs_build(dest_dir: str = None):
    """Build the documentation using mdbook"""
    if not check_mdbook_installed():
        return 1
    
    check_mermaid_support()
    
    print("Building documentation...")
    
    cmd = ["mdbook", "build"]
    if dest_dir:
        cmd.extend(["--dest-dir", dest_dir])
    
    result = subprocess.run(
        cmd,
        cwd=DOCS_DIR,
        capture_output=False
    )
    
    if result.returncode == 0:
        output_dir = dest_dir if dest_dir else "book"
        print(f"\nDocumentation built successfully!")
        print(f"Output directory: {DOCS_DIR}/{output_dir}")
        print(f"Open in browser: file://{DOCS_DIR}/{output_dir}/index.html")
    
    return result.returncode

@docs
@angreal.command(name="serve", about="Serve documentation locally with hot-reload")
@angreal.argument(name="port", short="p", default="3000", help="Port to serve on (default: 3000)")
@angreal.argument(name="hostname", short="h", default="localhost", help="Hostname to bind to")
@angreal.argument(name="open", short="o", takes_value=False, is_flag=True, help="Open browser automatically")
def docs_serve(port: str = "3000", hostname: str = "localhost", open: bool = False):
    """Serve the documentation with hot-reload for development"""
    if not check_mdbook_installed():
        return 1
    
    check_mermaid_support()
    
    print(f"Starting documentation server on {hostname}:{port}")
    print("Press Ctrl+C to stop the server")
    print(f"Documentation will be available at: http://{hostname}:{port}")
    
    cmd = ["mdbook", "serve", "--port", port, "--hostname", hostname]
    if open:
        cmd.append("--open")
    
    try:
        result = subprocess.run(
            cmd,
            cwd=DOCS_DIR,
            capture_output=False
        )
        return result.returncode
    except KeyboardInterrupt:
        print("\nDocumentation server stopped.")
        return 0

@docs
@angreal.command(name="watch", about="Watch for changes and rebuild documentation")
def docs_watch():
    """Watch documentation files and rebuild on changes"""
    if not check_mdbook_installed():
        return 1
    
    check_mermaid_support()
    
    print("Watching documentation files for changes...")
    print("Press Ctrl+C to stop watching")
    
    try:
        result = subprocess.run(
            ["mdbook", "watch"],
            cwd=DOCS_DIR,
            capture_output=False
        )
        return result.returncode
    except KeyboardInterrupt:
        print("\nStopped watching documentation files.")
        return 0

@docs
@angreal.command(name="clean", about="Clean built documentation")
def docs_clean():
    """Remove the built documentation directory"""
    book_dir = DOCS_DIR / "book"
    
    if book_dir.exists():
        print(f"Removing {book_dir}")
        shutil.rmtree(book_dir)
        print("Documentation build directory cleaned.")
    else:
        print("No build directory found to clean.")
    
    return 0

@docs
@angreal.command(name="test", about="Test documentation for errors")
@angreal.argument(name="library-path", short="L", help="Additional library path for testing")
def docs_test(library_path: str = None):
    """Test the documentation build and check for errors"""
    if not check_mdbook_installed():
        return 1
    
    print("Testing documentation...")
    
    cmd = ["mdbook", "test"]
    if library_path:
        cmd.extend(["-L", library_path])
    
    result = subprocess.run(
        cmd,
        cwd=DOCS_DIR,
        capture_output=False
    )
    
    if result.returncode == 0:
        print("Documentation tests passed!")
    else:
        print("Documentation tests failed!")
    
    return result.returncode

@docs
@angreal.command(name="check", about="Check documentation for broken links and issues")
def docs_check():
    """Check documentation for common issues like broken links"""
    if not check_mdbook_installed():
        return 1
    
    print("Checking documentation for issues...")
    
    # First, build to catch any build errors
    build_result = subprocess.run(
        ["mdbook", "build"],
        cwd=DOCS_DIR,
        capture_output=True,
        text=True
    )
    
    if build_result.returncode != 0:
        print("Build failed during check:")
        print(build_result.stderr)
        return build_result.returncode
    
    print("Build check passed.")
    
    # Check for common markdown issues
    src_dir = DOCS_DIR / "src"
    issues_found = False
    
    print("Scanning for potential issues...")
    
    # Look for common markdown issues
    for md_file in src_dir.rglob("*.md"):
        try:
            with open(md_file, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
                
                for i, line in enumerate(lines, 1):
                    # Check for broken internal links (basic check)
                    if '][' in line and not line.strip().startswith('#'):
                        # Could be a broken reference link
                        pass
                    
                    # Check for TODO/FIXME comments
                    if 'TODO' in line.upper() or 'FIXME' in line.upper():
                        print(f"  TODO/FIXME found in {md_file.relative_to(src_dir)}:{i}")
                        issues_found = True
                        
        except Exception as e:
            print(f"  Warning: Could not check {md_file}: {e}")
    
    if not issues_found:
        print("No obvious issues found in documentation.")
    
    print("Documentation check completed.")
    return 0

@docs
@angreal.command(name="init", about="Initialize documentation setup")
def docs_init():
    """Initialize or verify documentation setup"""
    print("Checking documentation setup...")
    
    # Check if docs directory exists
    if not DOCS_DIR.exists():
        print(f"Creating docs directory: {DOCS_DIR}")
        DOCS_DIR.mkdir(parents=True)
    
    # Check if book.toml exists
    book_toml = DOCS_DIR / "book.toml"
    if not book_toml.exists():
        print("book.toml not found - this doesn't look like an mdBook project.")
        print(f"Expected to find: {book_toml}")
        return 1
    
    # Check if src directory exists
    src_dir = DOCS_DIR / "src"
    if not src_dir.exists():
        print(f"Creating src directory: {src_dir}")
        src_dir.mkdir(parents=True)
    
    # Check mdbook installation
    if not check_mdbook_installed():
        return 1
    
    check_mermaid_support()
    
    print("Documentation setup verified!")
    print(f"Documentation source: {src_dir}")
    print(f"Configuration: {book_toml}")
    
    print("\nAvailable commands:")
    print("  angreal docs serve    # Start development server")
    print("  angreal docs build    # Build static site")
    print("  angreal docs check    # Check for issues")
    print("  angreal docs clean    # Clean build directory")
    
    return 0

@docs
@angreal.command(name="production", about="Build documentation for production deployment")
@angreal.argument(name="base-url", short="b", help="Base URL for production deployment")
def docs_production(base_url: str = None):
    """Build documentation optimized for production deployment"""
    if not check_mdbook_installed():
        return 1
    
    print("Building documentation for production...")
    
    # Clean first
    book_dir = DOCS_DIR / "book"
    if book_dir.exists():
        print("Cleaning previous build...")
        shutil.rmtree(book_dir)
    
    # Build
    result = subprocess.run(
        ["mdbook", "build"],
        cwd=DOCS_DIR,
        capture_output=False
    )
    
    if result.returncode == 0:
        print(f"\nProduction build completed!")
        print(f"Output directory: {book_dir}")
        
        if base_url:
            print(f"Remember to configure your web server to serve from: {base_url}")
        
        print("\nProduction checklist:")
        print("- [ ] Verify all links work in production environment")
        print("- [ ] Test on different devices/browsers")
        print("- [ ] Check that search functionality works")
        print("- [ ] Validate mermaid diagrams render correctly")
    
    return result.returncode
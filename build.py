import argparse
import subprocess
import toml
from pathlib import Path
import sys

ROOT_DIR = Path(__file__).resolve().parent
CARGO_TOML_PATH = f"{ROOT_DIR}/Cargo.toml"
INSTALLER_SCRIPT_PATH = f"{ROOT_DIR}/InstallerScripts/ComiInstallerScript.iss"


def read_crate_version():
    """Reads the crate version from Cargo.toml"""
    try:
        with open(CARGO_TOML_PATH, 'r') as f:
            cargo_toml = toml.load(f)
        return cargo_toml['package']['version']
    except Exception as e:
        print(f"Error reading Cargo.toml: {e}")
        sys.exit(1)


def build_rust_project(build_type: str):
    """Runs cargo build with the specified build type."""
    print(f"Building Rust project [{build_type}]...")
    cmd = ["cargo", "build"]
    if build_type == "release":
        cmd.append("--release")

    try:
        subprocess.run(cmd, check=True)
    except subprocess.CalledProcessError:
        print("Cargo build failed.")
        sys.exit(1)


def build_installer(crate_version: str):
    """Runs iscc to build the installer with the crate version."""
    print(f"Building installer with version: {crate_version}")
    try:
        subprocess.run([
            "iscc",
            str(INSTALLER_SCRIPT_PATH),
            f"/DMyAppVersion={crate_version}"
        ], check=True)
    except FileNotFoundError:
        print("iscc not found. Is Inno Setup installed and added to your PATH?")
        sys.exit(1)
    except subprocess.CalledProcessError:
        print("iscc failed.")
        sys.exit(1)


def parse_args():
    parser = argparse.ArgumentParser(description="Rust project build script")
    parser.add_argument(
        "build_type",
        choices=["release", "debug"],
        help="Type of build to perform"
    )
    parser.add_argument(
        "--installer",
        action="store_true",
        help="Build the installer after compiling the Rust project"
    )
    return parser.parse_args()


def main():
    args = parse_args()
    crate_version = read_crate_version()

    build_rust_project(args.build_type)

    if args.installer:
        build_installer(crate_version)


if __name__ == "__main__":
    main()

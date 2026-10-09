"""Emit Cargo JSON diagnostics for firmware and native host tools."""

from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parent.parent
FIRMWARE_TARGET = "thumbv7em-none-eabihf"


def host_target() -> str:
    result = subprocess.run(
        ["rustc", "-vV"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    for line in result.stdout.splitlines():
        if line.startswith("host: "):
            return line.removeprefix("host: ").strip()
    raise RuntimeError("rustc -vV did not report a host target")


def main() -> int:
    host = host_target()
    failed = False
    for packages, target in [(["app", "boot"], FIRMWARE_TARGET), (["xtask"], host)]:
        command = ["cargo", "check"]
        for package in packages:
            command.extend(["-p", package])
        command.extend(
            [
                "--all-targets",
                "--target", target,
                "--target-dir", "target/rust-analyzer",
                "--message-format=json",
            ]
        )
        # Inherit stdout/stderr: stdout must contain only Cargo JSON for the LSP.
        result = subprocess.run(command, cwd=ROOT)
        failed = failed or result.returncode != 0
    return int(failed)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, subprocess.CalledProcessError, RuntimeError) as error:
        print(f"Rust Analyzer check failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr, end="")
        sys.exit(1)
    except KeyboardInterrupt:
        sys.exit(130)

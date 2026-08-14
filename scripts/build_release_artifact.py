#!/usr/bin/env python3

import argparse
import gzip
import hashlib
import json
from pathlib import Path
import stat
import tarfile
import tomllib
import zipfile


PLUGIN_NAME = "recoverable-delete"
ARCHIVE_ROOT = "recoverable_delete"
PLUGIN_FILES = {
    ".codex-plugin/plugin.json",
    "hooks/dispatch_hook.ps1",
    "hooks/dispatch_hook.sh",
    "hooks/hooks.json",
    "skills/recoverable-delete/SKILL.md",
    "skills/recoverable-delete/agents/openai.yaml",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate a packaged Plugin and create release metadata."
    )
    parser.add_argument("--package-root", type=Path, required=True)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--expected-version", required=True)
    parser.add_argument("--binary-name", required=True)
    parser.add_argument(
        "--marketplace-manifest",
        type=Path,
        default=(
            Path(__file__).resolve().parent.parent
            / ".agents/plugins/marketplace.json"
        ),
    )
    parser.add_argument(
        "--cargo-manifest",
        type=Path,
        default=Path(__file__).resolve().parent.parent / "Cargo.toml",
    )
    return parser.parse_args()


def output_paths(archive: Path) -> tuple[Path, Path]:
    return (
        Path(f"{archive}.sha256"),
        Path(f"{archive}.contents.txt"),
    )


def validate_output_paths(archive: Path) -> None:
    checksum, contents = output_paths(archive)
    for path in (archive, checksum, contents):
        if path.exists():
            raise ValueError(
                f"release output already exists: {path}. Move it to Trash before retrying"
            )


def read_json(path: Path) -> dict:
    with path.open(encoding="utf-8") as source:
        value = json.load(source)
    if not isinstance(value, dict):
        raise ValueError(f"expected a JSON object: {path}")
    return value


def validate_package(
    package_root: Path, binary_name: str, expected_version: str, cargo_manifest: Path
) -> list[Path]:
    if package_root.name != PLUGIN_NAME or not package_root.is_dir():
        raise ValueError(f"package root must be a {PLUGIN_NAME} directory: {package_root}")

    expected_files = PLUGIN_FILES | {f"bin/{binary_name}"}
    actual_files: set[str] = set()
    for path in package_root.rglob("*"):
        relative = path.relative_to(package_root).as_posix()
        if path.is_symlink():
            raise ValueError(f"package symlinks are not allowed: {relative}")
        if path.is_file():
            actual_files.add(relative)

    missing = sorted(expected_files - actual_files)
    unexpected = sorted(actual_files - expected_files)
    if missing:
        raise ValueError(f"missing package file: {missing[0]}")
    if unexpected:
        raise ValueError(f"unexpected package file: {unexpected[0]}")

    plugin_manifest = read_json(package_root / ".codex-plugin/plugin.json")
    if plugin_manifest.get("name") != PLUGIN_NAME:
        raise ValueError("Plugin manifest name mismatch")
    if plugin_manifest.get("version") != expected_version:
        raise ValueError(
            "Plugin manifest version mismatch: "
            f"expected {expected_version}, got {plugin_manifest.get('version')!r}"
        )

    read_json(package_root / "hooks/hooks.json")
    with cargo_manifest.open("rb") as source:
        cargo_package = tomllib.load(source).get("package", {})
    if cargo_package.get("name") != PLUGIN_NAME:
        raise ValueError("Cargo package name mismatch")
    if cargo_package.get("version") != expected_version:
        raise ValueError(
            "Cargo package version mismatch: "
            f"expected {expected_version}, got {cargo_package.get('version')!r}"
        )

    return [package_root / relative for relative in sorted(expected_files)]


def validate_marketplace(marketplace_manifest: Path) -> str:
    if marketplace_manifest.is_symlink():
        raise ValueError("marketplace symlinks are not allowed")
    if not marketplace_manifest.is_file():
        raise ValueError(f"marketplace manifest not found: {marketplace_manifest}")
    marketplace = read_json(marketplace_manifest)
    marketplace_name = marketplace.get("name")
    if not isinstance(marketplace_name, str) or not marketplace_name:
        raise ValueError("marketplace name is missing")

    plugins = marketplace.get("plugins")
    if not isinstance(plugins, list) or len(plugins) != 1:
        raise ValueError("release marketplace must contain exactly one Plugin")
    plugin = plugins[0]
    if not isinstance(plugin, dict) or plugin.get("name") != PLUGIN_NAME:
        raise ValueError("marketplace Plugin name mismatch")
    source = plugin.get("source")
    if not isinstance(source, dict) or source.get("path") != f"./plugins/{PLUGIN_NAME}":
        raise ValueError("marketplace source mismatch")

    return marketplace_name


def archive_mode(path: Path) -> int:
    executable = path.stat().st_mode & 0o111
    return 0o755 if executable else 0o644


def create_tar_gz(archive: Path, files: list[tuple[Path, str]]) -> None:
    with archive.open("xb") as raw_archive:
        with gzip.GzipFile(fileobj=raw_archive, mode="wb", filename="", mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w") as output:
                for path, name in files:
                    info = output.gettarinfo(str(path), arcname=name)
                    # Fix release metadata so identical inputs produce byte-identical archives.
                    info.uid = 0
                    info.gid = 0
                    info.uname = ""
                    info.gname = ""
                    info.mtime = 0
                    info.mode = archive_mode(path)
                    with path.open("rb") as source:
                        output.addfile(info, source)


def create_zip(archive: Path, files: list[tuple[Path, str]]) -> None:
    with zipfile.ZipFile(
        archive, mode="x", compression=zipfile.ZIP_DEFLATED, compresslevel=9
    ) as output:
        for path, name in files:
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | archive_mode(path)) << 16
            output.writestr(info, path.read_bytes(), compress_type=zipfile.ZIP_DEFLATED)


def create_archive(archive: Path, files: list[tuple[Path, str]]) -> None:
    archive.parent.mkdir(parents=True, exist_ok=True)
    if archive.name.endswith(".tar.gz"):
        create_tar_gz(archive, files)
    elif archive.suffix == ".zip":
        create_zip(archive, files)
    else:
        raise ValueError("archive must end with .tar.gz or .zip")


def archive_entries(archive: Path) -> list[str]:
    if archive.name.endswith(".tar.gz"):
        with tarfile.open(archive, mode="r:gz") as source:
            return sorted(member.name for member in source.getmembers() if member.isfile())
    with zipfile.ZipFile(archive, mode="r") as source:
        return sorted(info.filename for info in source.infolist() if not info.is_dir())


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def write_metadata(
    archive: Path, expected_version: str, marketplace_name: str, entries: list[str]
) -> None:
    checksum_path, contents_path = output_paths(archive)
    digest = sha256(archive)
    checksum_path.write_text(f"{digest}  {archive.name}\n", encoding="utf-8")
    contents_path.write_text(
        "\n".join(
            [
                f"version: {expected_version}",
                f"marketplace: {marketplace_name}",
                f"archive: {archive.name}",
                f"sha256: {digest}",
                "files:",
                *entries,
                "",
            ]
        ),
        encoding="utf-8",
    )


def main() -> None:
    args = parse_args()
    validate_output_paths(args.archive)
    package_root = args.package_root.resolve()
    plugin_files = validate_package(
        package_root,
        args.binary_name,
        args.expected_version,
        args.cargo_manifest.resolve(),
    )
    marketplace_manifest = args.marketplace_manifest.absolute()
    marketplace_name = validate_marketplace(marketplace_manifest)
    files = [
        (
            marketplace_manifest,
            f"{ARCHIVE_ROOT}/.agents/plugins/marketplace.json",
        ),
        *[
            (
                path,
                (
                    Path(ARCHIVE_ROOT)
                    / "plugins"
                    / PLUGIN_NAME
                    / path.relative_to(package_root)
                ).as_posix(),
            )
            for path in plugin_files
        ],
    ]
    files.sort(key=lambda item: item[1])
    expected_entries = [name for _, name in files]
    create_archive(args.archive, files)
    actual_entries = archive_entries(args.archive)
    if actual_entries != expected_entries:
        raise ValueError("archive content does not match the validated package")
    write_metadata(
        args.archive, args.expected_version, marketplace_name, actual_entries
    )
    print(args.archive)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, json.JSONDecodeError, tomllib.TOMLDecodeError) as error:
        raise SystemExit(str(error)) from error

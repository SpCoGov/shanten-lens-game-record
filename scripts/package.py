import hashlib
import json
import os
import platform
import stat
from pathlib import Path
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED


def package():
    root = Path(__file__).resolve().parents[1]
    manifest = json.loads((root / "plugin.json").read_text(encoding="utf-8"))
    system = {"Windows": "windows", "Darwin": "macos", "Linux": "linux"}[platform.system()]
    arch = {"amd64": "x64", "x86_64": "x64", "arm64": "arm64", "aarch64": "arm64"}[platform.machine().lower()]
    executable = "game-record.exe" if system == "windows" else "game-record"
    manifest["backend"]["entry"] = f"backend/{executable}"
    platform_id = f"{system}-{arch}"
    version = manifest["version"]
    package_version = json.loads((root / "package.json").read_text(encoding="utf-8"))["version"]
    if version != package_version:
        raise ValueError("plugin.json and package.json versions must match")
    if os.environ.get("GITHUB_REF_TYPE") == "tag" and os.environ.get("GITHUB_REF_NAME") != f"v{version}":
        raise ValueError("release tag must match the plugin version")
    filename = f"{manifest['id']}-{version}-{platform_id}.plugin.zip"
    target = Path(os.environ.get("CARGO_TARGET_DIR", root / "target")) / "release" / executable
    output = root / "dist" / filename
    output.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(output, "w", compression=ZIP_DEFLATED) as archive:
        archive.writestr("plugin.json", json.dumps(manifest, ensure_ascii=False, indent=2))
        for folder, prefix in [(root / "dist/frontend", "frontend"), (root / "locales", "locales")]:
            for file in sorted(folder.rglob("*")):
                if file.is_file():
                    archive.write(file, f"{prefix}/{file.relative_to(folder).as_posix()}")
        entry = ZipInfo(f"backend/{executable}")
        entry.external_attr = (stat.S_IFREG | 0o755) << 16
        entry.compress_type = ZIP_DEFLATED
        archive.writestr(entry, target.read_bytes())
        archive.write(root / "LICENSE", "LICENSE")
    data = output.read_bytes()
    metadata = {"id": manifest["id"], "version": version,
                "downloadUrl": f"https://github.com/SpCoGov/shanten-lens-game-record/releases/download/v{version}/{filename}",
                "sha256": hashlib.sha256(data).hexdigest(), "size": len(data)}
    (root / "dist" / f"{platform_id}.json").write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
    print(output)


if __name__ == "__main__":
    package()

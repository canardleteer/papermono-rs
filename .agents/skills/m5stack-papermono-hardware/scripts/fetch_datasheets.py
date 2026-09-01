#!/usr/bin/env python3
"""Populate the local (gitignored) datasheet cache for this skill.

Downloads vendor PDFs into resources/datasheets/pdf/ and extracts Markdown
into resources/datasheets/md/. Records SHA-256 of every cached file in
resources/datasheets.sha256 (committed) so an IPFS CIDv1 can be derived later.

This is a machine-local cache, not a vendored documentation corpus.
Agents must not run `fetch` unless a human asked. `status` is local-only.
PaperMono / PaperMono-Lite parts only. Schematic ids also cache the
dated OSS gallery PNGs next to the PDF.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

SKILL_DIR = Path(__file__).resolve().parent.parent
RESOURCES_DIR = SKILL_DIR / "resources"
CACHE_DIR = RESOURCES_DIR / "datasheets"
PDF_DIR = CACHE_DIR / "pdf"
MD_DIR = CACHE_DIR / "md"
PNG_DIR = CACHE_DIR / "png"
SHA256_PATH = RESOURCES_DIR / "datasheets.sha256"
SHA256_JSON_PATH = RESOURCES_DIR / "datasheets.sha256.json"

USER_AGENT = (
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36"
)

# Keep ids and filenames in sync with resources/datasheets.md.
# urls are tried in order; later entries are public mirrors.
DOCUMENTS: tuple[dict[str, object], ...] = (
    {
        "id": "ssd1677",
        "title": "Solomon Systech SSD1677 datasheet",
        "urls": (
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/SSD1677.pdf",
            "https://files.waveshare.com/upload/2/2a/SSD1677_1.0.pdf",
            "https://www.solumco.com/files/SSD1677.pdf",
        ),
    },
    {
        "id": "epd-module",
        "title": "M5Stack e-paper module user manual",
        "urls": (
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/EPD_Module_User_Manual.pdf",
        ),
    },
    {
        "id": "m5pm1",
        "title": "M5Stack M5PM1 datasheet",
        "urls": (
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1207/M5PM1_Datasheet_EN.pdf",
        ),
    },
    {
        "id": "m5ioe1",
        "title": "M5Stack M5IOE1 IO expander datasheet",
        "urls": (
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1210/IO_Expander_Datasheet_EN.pdf",
        ),
    },
    {
        "id": "papermono-schematic",
        "title": "M5Stack PaperMono schematic V0.6.2",
        "urls": (
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf",
        ),
        "source_page": "https://docs.m5stack.com/en/core/PaperMono",
        "gallery": (
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_01.png",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_02.png",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_03.png",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_04.png",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_05.png",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_06.png",
        ),
    },
    {
        "id": "papermono-lite-schematic",
        "title": "M5Stack PaperMono-Lite schematic V0.6.2",
        "urls": (
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522.pdf",
        ),
        "source_page": "https://docs.m5stack.com/en/core/PaperMono-Lite",
        "gallery": (
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522_page_01.png",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522_page_02.png",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522_page_03.png",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522_page_04.png",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522_page_05.png",
        ),
    },
    {
        "id": "papermono-product",
        "title": "M5Stack PaperMono product documentation PDF",
        "urls": (
            "https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/pdf/static/en/core/PaperMono.pdf",
        ),
    },
    {
        "id": "papermono-lite-product",
        "title": "M5Stack PaperMono-Lite product documentation PDF",
        "urls": (
            "https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/pdf/static/en/core/PaperMono-Lite.pdf",
        ),
    },
    {
        "id": "ft6336g",
        "title": "FocalTech FT6336G datasheet",
        "urls": (
            "https://www.display-lcd.com/data/upload/admin/202503/67e3663dbe0d2.pdf",
        ),
    },
    {
        "id": "bmi270",
        "title": "Bosch BMI270 datasheet BST-BMI270-DS000",
        "urls": (
            "https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bmi270-ds000.pdf",
            "https://content.arduino.cc/assets/bmi270-ds000.pdf",
        ),
    },
    {
        "id": "rx8130ce",
        "title": "Epson RX8130CE application manual",
        "urls": (
            "https://download.epsondevice.com/td/pdf/app/RX8130CE_en.pdf",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1132/RX8130CE_cn-Register-Datasheet.pdf",
        ),
    },
    {
        "id": "ip2315",
        "title": "Injoinic IP2315 datasheet",
        "urls": (
            "https://www.chipsourcetek.com/DataSheet/IP2315.pdf",
        ),
    },
    {
        "id": "st25r3916",
        "title": "ST ST25R3916 datasheet",
        "urls": (
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1205/ST25R3916_EN.pdf",
            "https://www.st.com/resource/en/datasheet/st25r3916.pdf",
        ),
    },
    {
        "id": "sx1262",
        "title": "Semtech SX1261/SX1262 datasheet V2.2",
        "urls": (
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1177/DS_SX1261_2_V2-2.pdf",
        ),
    },
    {
        "id": "esp32-s3-datasheet",
        "title": "Espressif ESP32-S3 datasheet",
        "urls": (
            "https://documentation.espressif.com/esp32-s3_datasheet_en.pdf",
            "https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/472/esp32-s3_datasheet_en.pdf",
            "https://www.espressif.com/documentation/esp32-s3_datasheet_en.pdf",
        ),
    },
    {
        "id": "esp32-s3-trm",
        "title": "Espressif ESP32-S3 technical reference manual",
        "urls": (
            "https://documentation.espressif.com/esp32-s3_technical_reference_manual_en.pdf",
            "https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf",
        ),
    },
)


def pdf_path(doc_id: str) -> Path:
    return PDF_DIR / f"{doc_id}.pdf"


def md_path(doc_id: str) -> Path:
    return MD_DIR / f"{doc_id}.md"


def gallery_paths(doc: dict[str, object]) -> list[tuple[str, Path]]:
    """(url, dest) for optional schematic page PNGs."""
    urls = doc.get("gallery")
    if not urls:
        return []
    doc_id = str(doc["id"])
    out: list[tuple[str, Path]] = []
    for url in urls:  # type: ignore[union-attr]
        name = Path(urllib.parse.urlparse(str(url)).path).name
        if "_page_" in name:
            page = name.rsplit("_page_", 1)[-1]
            dest_name = f"{doc_id}-page-{page}"
        else:
            dest_name = f"{doc_id}-{name}"
        out.append((str(url), PNG_DIR / dest_name))
    return out


def cache_files(doc: dict[str, object]) -> list[Path]:
    doc_id = str(doc["id"])
    paths = [pdf_path(doc_id), md_path(doc_id)]
    paths.extend(dest for _, dest in gallery_paths(doc))
    return paths


def cache_rel(path: Path) -> str:
    return str(path.relative_to(RESOURCES_DIR))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def selected(doc_id: str | None) -> tuple[dict[str, object], ...]:
    if doc_id is None:
        return DOCUMENTS
    for doc in DOCUMENTS:
        if doc["id"] == doc_id:
            return (doc,)
    known = ", ".join(str(doc["id"]) for doc in DOCUMENTS)
    raise SystemExit(f"unknown id {doc_id!r}; expected one of: {known}")


def load_previous_hash_records() -> dict[str, dict[str, object]]:
    if not SHA256_JSON_PATH.is_file():
        return {}
    try:
        payload = json.loads(SHA256_JSON_PATH.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    previous: dict[str, dict[str, object]] = {}
    for record in payload.get("files", []):
        if isinstance(record, dict) and "path" in record:
            previous[str(record["path"])] = record
    return previous


def write_hashes() -> None:
    previous = load_previous_hash_records()
    records: list[dict[str, object]] = []
    lines: list[str] = []
    for doc in DOCUMENTS:
        doc_id = str(doc["id"])
        for path in cache_files(doc):
            rel = cache_rel(path)
            if path.is_file():
                digest = sha256_file(path)
                records.append(
                    {
                        "id": doc_id,
                        "path": rel,
                        "sha256": digest,
                        "bytes": path.stat().st_size,
                    }
                )
            elif rel in previous:
                # `fetch --id` must not drop hashes for files that are not
                # on this machine.
                records.append(previous[rel])
            else:
                continue
            lines.append(f"{records[-1]['sha256']}  {rel}\n")
    SHA256_PATH.write_text("".join(lines), encoding="utf-8")
    SHA256_JSON_PATH.write_text(
        json.dumps(
            {
                "algorithm": "sha256",
                "ipfs_note": (
                    "CIDv1 can be derived later from these SHA-256 digests "
                    "(typically raw codec 0x55 + sha2-256)."
                ),
                "files": records,
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    print(f"wrote {SHA256_PATH.relative_to(SKILL_DIR)} ({len(records)} files)")


def load_expected_hashes() -> dict[str, str]:
    if not SHA256_PATH.is_file():
        return {}
    expected: dict[str, str] = {}
    for line in SHA256_PATH.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        digest, path = line.split(None, 1)
        expected[path] = digest
    return expected


def cmd_status(doc_id: str | None) -> int:
    missing: list[str] = []
    expected = load_expected_hashes()
    print(f"cache: {CACHE_DIR}")
    for doc in selected(doc_id):
        doc_id_s = str(doc["id"])
        flags = []
        for kind, path in (("pdf", pdf_path(doc_id_s)), ("md", md_path(doc_id_s))):
            if not path.is_file():
                flags.append(f"{kind}=NO")
                missing.append(f"{doc_id_s}:{kind}")
                continue
            digest = sha256_file(path)
            rel = cache_rel(path)
            want = expected.get(rel)
            if want and want != digest:
                flags.append(f"{kind}=HASH_MISMATCH")
                missing.append(f"{doc_id_s}:{kind}:hash")
            else:
                flags.append(f"{kind}=yes")
        gallery = gallery_paths(doc)
        if gallery:
            png_ok = 0
            for url, path in gallery:
                if not path.is_file():
                    missing.append(f"{doc_id_s}:png:{path.name}")
                    continue
                digest = sha256_file(path)
                want = expected.get(cache_rel(path))
                if want and want != digest:
                    missing.append(f"{doc_id_s}:png:{path.name}:hash")
                    continue
                png_ok += 1
            flags.append(f"png={png_ok}/{len(gallery)}")
        print(f"{doc_id_s:22} {' '.join(flags)}")
    if missing:
        print()
        print("Cache incomplete or hash mismatch. Ask the user to capture files:")
        print(f"  python3 {Path(__file__).resolve()} fetch")
        return 1
    print("all listed datasheets are present (pdf + md)")
    return 0


def download_url(url: str) -> bytes:
    parsed_host = urllib.parse.urlparse(url).netloc
    request = urllib.request.Request(
        url,
        headers={
            "User-Agent": USER_AGENT,
            "Accept": "application/pdf,application/octet-stream,*/*;q=0.8",
            "Referer": f"https://{parsed_host}/",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=180) as response:
            return response.read()
    except TimeoutError as error:
        raise urllib.error.URLError(f"timeout: {error}") from error


def download_png(url: str, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    body = download_url(url)
    if not body.startswith(b"\x89PNG"):
        raise RuntimeError(f"{url}: not a PNG")
    if len(body) < 8_000:
        raise RuntimeError(f"{url}: too small ({len(body)} bytes)")
    dest.write_bytes(body)
    print(f"wrote {dest.relative_to(SKILL_DIR)} ({len(body)} bytes) from {url}")


def fetch_gallery(doc: dict[str, object], force: bool) -> None:
    for url, dest in gallery_paths(doc):
        if dest.is_file() and not force:
            print(f"{doc['id']}: {dest.name} already present")
            continue
        download_png(url, dest)


def download(doc: dict[str, object]) -> str:
    dest = pdf_path(str(doc["id"]))
    dest.parent.mkdir(parents=True, exist_ok=True)
    errors: list[str] = []
    for url in doc["urls"]:  # type: ignore[union-attr]
        try:
            body = download_url(str(url))
        except urllib.error.URLError as error:
            errors.append(f"{url}: {error}")
            continue
        if not body.startswith(b"%PDF"):
            errors.append(f"{url}: not a PDF")
            continue
        if len(body) < 8_000:
            errors.append(f"{url}: too small ({len(body)} bytes)")
            continue
        dest.write_bytes(body)
        print(f"wrote {dest.relative_to(SKILL_DIR)} ({len(body)} bytes) from {url}")
        return str(url)
    raise RuntimeError(
        f"{doc['id']}: download failed. Save the file as {dest} and run convert.\n"
        + "\n".join(errors)
    )


def convert_one(doc: dict[str, object], source_url: str | None) -> None:
    source = pdf_path(str(doc["id"]))
    dest = md_path(str(doc["id"]))
    if not source.is_file():
        raise RuntimeError(f"{doc['id']}: missing {source}")
    pdftotext = shutil.which("pdftotext")
    if pdftotext is None:
        raise RuntimeError(
            "pdftotext not found (install poppler-utils). "
            f"PDF is at {source}; markdown was not written."
        )
    result = subprocess.run(
        [pdftotext, "-layout", "-enc", "UTF-8", str(source), "-"],
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        err = result.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(f"{doc['id']}: pdftotext failed: {err or result.returncode}")
    text = result.stdout.decode("utf-8", errors="replace").strip()
    dest.parent.mkdir(parents=True, exist_ok=True)
    digest = sha256_file(source)
    used = source_url or str(doc["urls"][0])  # type: ignore[index]
    header = (
        f"# {doc['title']}\n\n"
        f"- id: `{doc['id']}`\n"
        f"- source: {used}\n"
        f"- local pdf: `pdf/{doc['id']}.pdf`\n"
        f"- pdf sha256: `{digest}`\n"
        f"- extracted with `pdftotext -layout` for agent reading; figures stay in the PDF\n\n"
        "---\n\n"
    )
    dest.write_text(header + text + "\n", encoding="utf-8")
    print(f"wrote {dest.relative_to(SKILL_DIR)}")


def cmd_fetch(doc_id: str | None, force: bool) -> int:
    failed = 0
    for doc in selected(doc_id):
        dest = pdf_path(str(doc["id"]))
        source_url: str | None = None
        if dest.is_file() and not force:
            print(f"{doc['id']}: pdf already present")
        else:
            try:
                source_url = download(doc)
            except RuntimeError as error:
                print(error, file=sys.stderr)
                failed += 1
                continue
        try:
            convert_one(doc, source_url)
        except RuntimeError as error:
            print(error, file=sys.stderr)
            failed += 1
        try:
            fetch_gallery(doc, force)
        except (RuntimeError, urllib.error.URLError) as error:
            print(error, file=sys.stderr)
            failed += 1
    write_hashes()
    return 1 if failed else 0


def cmd_convert(doc_id: str | None) -> int:
    failed = 0
    for doc in selected(doc_id):
        try:
            convert_one(doc, None)
        except RuntimeError as error:
            print(error, file=sys.stderr)
            failed += 1
    write_hashes()
    return 1 if failed else 0


def cmd_hash() -> int:
    write_hashes()
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Local datasheet cache for m5stack-papermono-hardware (gitignored)."
    )
    parser.add_argument(
        "command",
        choices=("status", "fetch", "convert", "hash"),
        help="status is local-only; fetch downloads (needs a human ask)",
    )
    parser.add_argument("--id", dest="doc_id", help="limit to one document id")
    parser.add_argument(
        "--force",
        action="store_true",
        help="re-download PDFs even when already present",
    )
    args = parser.parse_args()
    if args.command == "status":
        return cmd_status(args.doc_id)
    if args.command == "fetch":
        return cmd_fetch(args.doc_id, args.force)
    if args.command == "hash":
        return cmd_hash()
    return cmd_convert(args.doc_id)


if __name__ == "__main__":
    sys.exit(main())

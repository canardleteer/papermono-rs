# Source provenance

Dated snapshots of the official M5Stack product pages, exported
with each page’s **view as markdown** control. These are vendor
docs, not a measurement and not a pinout absorbed here.

The living HTML remains the index. Re-read those URLs when a
net, SKU row, or refresh note looks stale. A later docs
publish can disagree with this snapshot.

Do not run `rumdl` on the snapshot files. They are vendor
markdown (custom tags, long lines). This `SOURCE.md` is
authored.

| Field | PaperMono (`C153`) | PaperMono-Lite (`C153-Lite`) |
| --- | --- | --- |
| Upstream page | https://docs.m5stack.com/en/core/PaperMono | https://docs.m5stack.com/en/core/PaperMono-Lite |
| Export | M5Stack docs **view as markdown** | Same |
| Snapshot file | `PaperMono.2026-10-01.md` (prior: `PaperMono.2026-09-01.md`) | `PaperMono-Lite.2026-10-01.md` (prior: `PaperMono-Lite.2026-09-01.md`) |
| Vendored on | 2026-10-01 | 2026-10-01 |
| SHA-256 | `96990812f2e4ac81a1909ff0f41fef5d9b0866036f90a8c18abfc9d69927876c` | `0aec6aac04f0f459cdd1707d7bdf2b7a48522d95142abb0d1ad12a0b694509e9` |
| Copyright | M5Stack (product documentation). Vendored for offline agent use. | Same |

Both snapshots contain the **M5GFX LUT Refresh Speed** table and
explicitly identify the values as PaperMono laboratory results.
Treat them as reference timings for PaperMono (`C153`), not
PaperMono-Lite timings or on-unit results. Both pages also retain
the recommendation to use the panel manufacturer's OTP example
because M5GFX waveforms are currently unstable, and advise a full
refresh after about 10 partial fast refreshes. See
[display.md](../../references/display.md) for how this relates to
this repository's OTP refresh path and current implementation.

Product PDFs (`papermono-product` /
`papermono-lite-product`) are a different snapshot and can
lag these HTML exports
([datasheets.md](../datasheets.md)).

Absorbed headings: [catalog.md](../../references/catalog.md),
[display.md](../../references/display.md),
[pin-map.md](../../references/pin-map.md).

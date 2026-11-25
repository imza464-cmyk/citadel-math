# syntax=docker/dockerfile:1.7
# Citadel-Math — Hardened Multi-Stage Rust + Python MCP Server Container
# Dual-licensed under GNU AGPL-3.0-only OR Commercial License.

# ---------- Stage 1: Build Pure-Rust (#![forbid(unsafe_code)]) Core ----------
FROM rust:1-slim-bookworm AS rust-builder

WORKDIR /build
COPY citadel_math_core ./citadel_math_core
COPY database ./database
COPY data ./data

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/citadel_math_core/target \
    cargo build --release --manifest-path citadel_math_core/Cargo.toml && \
    cp citadel_math_core/target/release/citadel_math_core /build/citadel_math_core_bin && \
    strip /build/citadel_math_core_bin

# ---------- Stage 2: Minimal Non-Root Python MCP Runtime ----------
FROM python:3.12-slim-bookworm AS runtime

LABEL org.opencontainers.image.title="Citadel-Math MCP Server" \
      org.opencontainers.image.description="Deterministic Z^7 dimensional shield, checked-i64 algebraic identity checker, IEEE 1788-2015 Interval64, and 15-tool offline MCP server" \
      org.opencontainers.image.source="https://github.com/imza464-cmyk/citadel-math" \
      org.opencontainers.image.licenses="AGPL-3.0-only" \
      io.modelcontextprotocol.server.name="io.github.imza464-cmyk/citadel-math"

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    OPENBLAS_NUM_THREADS=1 \
    KMATH_RUST_BIN=/usr/local/bin/citadel_math_core

WORKDIR /app

# Install pinned Python dependencies with no cache bloat
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# Copy compiled Rust core binary and read-only SSOT assets
COPY --from=rust-builder --chmod=0555 /build/citadel_math_core_bin /usr/local/bin/citadel_math_core
COPY engine ./engine
COPY database ./database
COPY data ./data
COPY kmath.py kmath_mcp.py AGENTS.md DISCLAIMER.md LICENSE COMMERCIAL_LICENSE.md ./

# Create unprivileged user (UID 10001) — zero root execution at runtime
RUN useradd --uid 10001 --create-home --shell /usr/sbin/nologin kmath && \
    chown -R kmath:kmath /app

USER 10001:10001

# Self-contained Python healthcheck (zero curl/shell bloat)
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD ["python", "-c", "import json, kmath_mcp; r=json.loads(kmath_mcp.kmath_prove_identity('(x+1)^2','x^2+2*x+1')); assert r.get('proven_identical') is True"]

# Exec form ensures PID 1 directly receives SIGTERM for clean shutdown
ENTRYPOINT ["python", "/app/kmath_mcp.py"]

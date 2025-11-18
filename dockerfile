# -------------------------------------------------
#  Builder stage – install Python + deps
# -------------------------------------------------
FROM python:3.12-slim AS builder

# System packages needed by Playwright (optional)
# If you never need JS rendering, you can delete the whole block.
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libc6 \
    libcairo2 \
    libdbus-1-3 \
    libexpat1 \
    libfontconfig1 \
    libgcc1 \
    libglib2.0-0 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libpango-1.0-0 \
    libpangocairo-1.0-0 \
    libstdc++6 \
    libx11-6 \
    libxcomposite1 \
    libxdamage1 \
    libxext6 \
    libxfixes3 \
    libxrandr2 \
    libxshmfence1 \
    wget \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy the source code (including static assets)
COPY . .

# -------------------------------------------------
#  Runtime stage – tiny image with only runtime deps
# -------------------------------------------------
FROM python:3.12-slim

WORKDIR /app

# Copy only the compiled wheels and source from builder
COPY --from=builder /usr/local/lib/python3.12/site-packages /usr/local/lib/python3.12/site-packages
COPY --from=builder /app /app

# Expose the port Render will set (via $PORT)
ENV PORT=8080
EXPOSE 8080

# Entrypoint – start the ASGI server
CMD ["uvicorn", "app:app", "--host", "0.0.0.0", "--port", "8080"]
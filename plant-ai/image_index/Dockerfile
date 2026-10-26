# The image index, in a throwaway-style worker image (like fp-runner / fp-upscale).
# Build (needs the internet ONCE, to download the open model into the image):
#   docker build -t gfa-image-index -f plant-ai/image_index/Dockerfile plant-ai
# After that it runs offline (HF_HUB_OFFLINE=1). No port is published unless you ask for
# 127.0.0.1:PORT yourself; see README.md.
FROM python:3.12-slim

ENV PYTHONDONTWRITEBYTECODE=1 PYTHONUNBUFFERED=1 \
    HF_HOME=/models HF_HUB_DISABLE_TELEMETRY=1

WORKDIR /app
COPY image_index/requirements.txt /tmp/requirements.txt
RUN pip install --no-cache-dir --require-hashes -r /tmp/requirements.txt \
    --extra-index-url https://download.pytorch.org/whl/cpu

# Models: MODELS is a space-separated list of "name=huggingface-id@revision" (see models.py).
# Changing the default means re-running the evaluation (README, "How good is it").
COPY image_index/models.py image_index/fetch_models.py /app/image_index/
ARG MODELS="dinov2-small"
RUN python -m image_index.fetch_models $MODELS && rm -rf /root/.cache

COPY image_index /app/image_index
ENV HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1
# Runs as a normal user; the data folders are mounted read-only except /index.
RUN useradd -m idx && mkdir /index && chown idx /index
USER idx
ENTRYPOINT ["python", "-m", "image_index"]

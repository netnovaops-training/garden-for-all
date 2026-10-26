# The tag reader (OCR) for the scan queue: Tesseract + RapidOCR (PaddleOCR models on ONNX) + zbar.
# Runs on the PC with the graphics card, bound to 127.0.0.1 only. CPU only: the card stays free
# for the language models.
#
#   docker build -t garden-ocr plant-ai/ocr
#   docker run -d --name garden-ocr --restart unless-stopped -p 127.0.0.1:8788:8098 --read-only \
#       --tmpfs /tmp --cap-drop ALL --security-opt no-new-privileges garden-ocr
FROM python:3.13-slim

# Tesseract with English, zbar for barcodes/QR, and the two libraries OpenCV (used by RapidOCR) loads.
RUN apt-get update \
 && apt-get install -y --no-install-recommends tesseract-ocr tesseract-ocr-eng libzbar0 libgl1 libglib2.0-0 \
 && rm -rf /var/lib/apt/lists/*

# Exact versions with checksums (requirements.in says how to rebuild the list).
COPY requirements.txt /app/requirements.txt
# pip is removed afterwards: the reader never installs anything, and pip carries its own copies of
# libraries (msgpack, urllib3) that the image scan flags.
RUN pip install --no-cache-dir --require-hashes -r /app/requirements.txt && pip uninstall -y pip

COPY server.py /app/server.py
# RapidOCR keeps its models inside its package; warm them once so the first tag is not slow.
RUN python -c "from rapidocr_onnxruntime import RapidOCR; RapidOCR()"

RUN useradd --system --no-create-home reader
USER reader
WORKDIR /app
ENV OMP_THREAD_LIMIT=2 PYTHONUNBUFFERED=1
EXPOSE 8098
HEALTHCHECK --interval=30s --timeout=5s CMD python -c "import urllib.request,sys; sys.exit(0 if urllib.request.urlopen('http://127.0.0.1:8098/health').status==200 else 1)"
CMD ["python", "server.py"]

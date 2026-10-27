"""Download the named models into the image (build time only) and print their exact revisions.

    python -m image_index.fetch_models dinov2-small bioclip-2
"""
import sys

from huggingface_hub import snapshot_download

from .models import MODELS, snapshot_args


def main(names):
    for name in names:
        # nosec B615: the revision is the pin in models.py (an empty pin = newest, printed below)
        path = snapshot_download(**snapshot_args(MODELS[name]))  # nosec B615
        print(f"{name}: {MODELS[name].hf_id} revision {path.rsplit('/', 1)[-1]}")


if __name__ == "__main__":
    main(sys.argv[1:])

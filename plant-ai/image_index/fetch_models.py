"""Download the named models into the image (build time only) and print their exact revisions.

    python -m image_index.fetch_models dinov2-small siglip-base
"""
import sys

from huggingface_hub import snapshot_download

from .models import MODELS


def main(names):
    for name in names:
        spec = MODELS[name]
        path = snapshot_download(spec.hf_id, revision=spec.revision or None,
                                 allow_patterns=["*.json", "*.safetensors", "*.txt", "*.model"])
        print(f"{name}: {spec.hf_id} revision {path.rsplit('/', 1)[-1]}")


if __name__ == "__main__":
    main(sys.argv[1:])

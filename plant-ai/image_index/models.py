"""The open models the index can use. All run on the CPU and all work offline once downloaded.

Each entry: huggingface id, a pinned revision (a commit of the model repo), the picture size the
model wants, its colour normalisation, how to cut the picture to a square, and how to turn the
model's output into one vector.
`fetch_models.py` prints the exact revision it downloaded; pin it here ("" = newest at download).
Only dinov2-small is pinned: it is the one the image ships. The others were only tried for the
comparison in the README.
"""
from dataclasses import dataclass

IMAGENET = ((0.485, 0.456, 0.406), (0.229, 0.224, 0.225))
HALF = ((0.5, 0.5, 0.5), (0.5, 0.5, 0.5))
CLIP = ((0.4815, 0.4578, 0.4082), (0.2686, 0.2613, 0.2758))


@dataclass(frozen=True)
class Spec:
    hf_id: str
    revision: str
    size: int
    norm: tuple
    kind: str  # dino | siglip | clip
    mode: str = "crop"  # crop | squash (see decode.py)
    feature: str = "cls+mean"  # dino only: which output makes the vector


MODELS = {
    # The pick (README, "Which model"): same accuracy as the bigger ones, smallest and fastest.
    # 336 px instead of 224 gave +3 points top-1 for 2.6x the time.
    "dinov2-small": Spec("facebook/dinov2-small", "ed25f3a31f01632728cabb09d1542f84ab7b0056",
                         336, IMAGENET, "dino", feature="cls"),
    "dinov2-base": Spec("facebook/dinov2-base", "", 224, IMAGENET, "dino"),
    "siglip-base": Spec("google/siglip-base-patch16-224", "", 224, HALF, "siglip"),
    "clip-b32": Spec("openai/clip-vit-base-patch32", "", 224, CLIP, "clip"),
}
DEFAULT_MODEL = "dinov2-small"


def fingerprint(name):
    """Everything that decides what a stored vector means; if it changes, vectors are redone."""
    s = MODELS[name]
    return f"{name}|{s.hf_id}@{s.revision}|{s.size}|{s.mode}|{s.feature}"

"""The open models the index can use. All run on the CPU and all work offline once downloaded.

Each entry: huggingface id, a pinned revision (a commit of the model repo), the picture size the
model wants, its colour normalisation, how to cut the picture to a square, and how to turn the
model's output into one vector.
`fetch_models.py` prints the exact revision it downloaded; pin it here ("" = newest at download).
dinov2-small and the plant-trained ones are pinned; the others were only tried for the
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
    kind: str  # dino | siglip | clip | plantclef | bioclip
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
    # Plant-trained backends (README, "Plant-trained models"). Both can also check a name.
    # PlantCLEF-2024 DINOv2 ViT-B/14, 7,806 south-west-European species. LICENSE CC-BY-NC-4.0:
    # fine for a private garden, not for anything sold.
    "plantclef-b14": Spec("vincent-espitalier/dino-v2-reg4-with-plantclef2024-weights",
                          "24656bb424790575adcbd7954ceefb1d87ec8b24", 518, IMAGENET, "plantclef",
                          feature="cls"),
    "plantclef-b14-336": Spec("vincent-espitalier/dino-v2-reg4-with-plantclef2024-weights",
                              "24656bb424790575adcbd7954ceefb1d87ec8b24", 336, IMAGENET,
                              "plantclef", feature="cls"),
    # BioCLIP 2 (ViT-L/14) and BioCLIP 1 (ViT-B/16), trained on tree-of-life photos. MIT.
    "bioclip-2": Spec("imageomics/bioclip-2", "2957b322090f9cb17ae72c71981c7218a28d81e0",
                      224, CLIP, "bioclip", feature=""),
    "bioclip": Spec("imageomics/bioclip", "ce901ab3c6a913f9e9ef94ce6d27761069f4f01c",
                    224, CLIP, "bioclip", feature=""),
}
DEFAULT_MODEL = "dinov2-small"


def snapshot_args(spec):
    """Which files of a model repo we keep. The SAME arguments must be used to download (build
    time) and to find the folder again offline, or the cache looks incomplete."""
    return {"repo_id": spec.hf_id, "revision": spec.revision or None,
            "allow_patterns": ["*.json", "*.safetensors", "*.txt", "*.model", "*.bin"],
            # BioCLIP 2 ships the same weights twice (.bin and .safetensors): take the safe one
            "ignore_patterns": ["*.bin"] if spec.hf_id.endswith("bioclip-2") else None}


def fingerprint(name):
    """Everything that decides what a stored vector means; if it changes, vectors are redone."""
    s = MODELS[name]
    return f"{name}|{s.hf_id}@{s.revision}|{s.size}|{s.mode}|{s.feature}"

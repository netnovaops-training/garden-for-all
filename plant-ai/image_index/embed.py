"""Turn small photo arrays into unit-length vectors with an open model on the CPU.

Backends (models.py `kind`):
  dino       DINOv2 (transformers)                          vector = class token
  siglip     SigLIP image tower (transformers)
  clip       CLIP image tower (transformers)
  plantclef  PlantCLEF-2024 DINOv2 ViT-B/14 (timm)    vector = class token; also species scores
  bioclip    BioCLIP / BioCLIP 2 (open_clip)          vector = image embedding; also text side
Only `plantclef` (species scores) and `bioclip` (taxon text) can verify a name.
"""
import glob
import os

import numpy as np

from .decode import cores
from .models import MODELS, snapshot_args


def _snapshot(spec):
    """The model folder already downloaded into the image (no network)."""
    from huggingface_hub import snapshot_download
    # nosec B615: the revision is the pin in models.py (snapshot_args); nothing is fetched here
    return snapshot_download(local_files_only=True, **snapshot_args(spec))  # nosec B615


class Embedder:
    def __init__(self, name, threads=None, feature=None):
        import torch

        self.name, self.spec = name, MODELS[name]
        self.feature = feature or self.spec.feature
        self.torch = torch
        torch.set_num_threads(threads or cores())
        kind = self.spec.kind
        if kind in ("dino", "siglip", "clip"):
            from transformers import AutoModel, CLIPVisionModelWithProjection, SiglipVisionModel
            cls = {"dino": AutoModel, "siglip": SiglipVisionModel,
                   "clip": CLIPVisionModelWithProjection}[kind]
            self.model = cls.from_pretrained(self.spec.hf_id,
                                             revision=self.spec.revision or None).eval()
        elif kind == "plantclef":
            self._load_plantclef()
        elif kind == "bioclip":
            self._load_bioclip()
        mean, std = self.spec.norm
        self.mean = torch.tensor(mean).view(1, 3, 1, 1)
        self.std = torch.tensor(std).view(1, 3, 1, 1)

    # --- loading the two non-transformers backends ---
    def _load_plantclef(self):
        import timm
        from safetensors.torch import load_file
        folder = _snapshot(self.spec)
        with open(os.path.join(folder, "species_id_mapping.txt"), encoding="utf-8") as f:
            # class index -> "Genus species Author"
            self.species = [ln.strip() for ln in f if ln.strip()]
        self.model = timm.create_model("vit_base_patch14_reg4_dinov2.lvd142m", pretrained=False,
                                       num_classes=len(self.species), img_size=518,
                                       dynamic_img_size=True)  # any size: position codes resized
        self.model.load_state_dict(load_file(glob.glob(os.path.join(folder, "*.safetensors"))[0]))
        self.model.eval()

    def _load_bioclip(self):
        import open_clip
        folder = _snapshot(self.spec)
        self.model, _, _ = open_clip.create_model_and_transforms("local-dir:" + folder)
        self.model.eval()
        self.tokenizer = open_clip.get_tokenizer("local-dir:" + folder)

    @property
    def dim(self):
        return self.embed(np.zeros((1, self.spec.size, self.spec.size, 3), np.uint8)).shape[1]

    def _pixels(self, batch):
        t = self.torch
        # np.array makes a copy: the decoded array may be read-only
        x = t.from_numpy(np.array(batch)).permute(0, 3, 1, 2).float() / 255.0
        return (x - self.mean) / self.std

    def embed(self, batch):
        """batch: uint8 (n, size, size, 3) -> float32 (n, dim), each row length 1."""
        t = self.torch
        x = self._pixels(batch)
        with t.inference_mode():
            k = self.spec.kind
            if k == "dino":
                h = self.model(pixel_values=x).last_hidden_state
                a, b = h[:, 0], h[:, 1:].mean(1)
                unit = lambda z: z / z.norm(dim=1, keepdim=True)  # noqa: E731
                v = {"cls": a, "mean": b, "cls+mean": t.cat([unit(a), unit(b)], 1)}[self.feature]
            elif k == "siglip":
                v = self.model(pixel_values=x).pooler_output
            elif k == "clip":
                v = self.model(pixel_values=x).image_embeds
            elif k == "plantclef":
                v = self.model.forward_features(x)[:, 0]
            else:  # bioclip
                v = self.model.encode_image(x)
            v = v / v.norm(dim=1, keepdim=True)
        return v.numpy().astype(np.float32)

    # --- name checks ---
    def text_embed(self, prompts):
        """bioclip: unit vectors for text prompts, e.g. 'a photo of Salvia yangii'."""
        t = self.torch
        with t.inference_mode():
            v = self.model.encode_text(self.tokenizer(list(prompts)))
            v = v / v.norm(dim=1, keepdim=True)
        return v.numpy().astype(np.float32)

    def logit_scale(self):
        """bioclip: the factor (about 100) that turns image-text cosines into scores."""
        s = self.model.logit_scale
        return float((s.detach() if hasattr(s, "detach") else s).exp())

    def species_logits(self, batch):
        """plantclef: scores for each of the model's species, (n, n_species)."""
        with self.torch.inference_mode():
            f = self.model.forward_features(self._pixels(batch))
            return self.model.forward_head(f).numpy().astype(np.float32)

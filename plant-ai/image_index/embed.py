"""Turn small photo arrays into unit-length vectors with an open model on the CPU."""
import numpy as np

from .decode import cores
from .models import MODELS


class Embedder:
    def __init__(self, name, threads=None, feature=None):
        import torch
        from transformers import AutoModel, CLIPVisionModelWithProjection, SiglipVisionModel

        self.name, self.spec = name, MODELS[name]
        self.feature = feature or self.spec.feature
        self.torch = torch
        torch.set_num_threads(threads or cores())
        kw = {"revision": self.spec.revision or None}
        cls = {"dino": AutoModel, "siglip": SiglipVisionModel,
               "clip": CLIPVisionModelWithProjection}[self.spec.kind]
        self.model = cls.from_pretrained(self.spec.hf_id, **kw).eval()
        mean, std = self.spec.norm
        self.mean = torch.tensor(mean).view(1, 3, 1, 1)
        self.std = torch.tensor(std).view(1, 3, 1, 1)

    @property
    def dim(self):
        return self.embed(np.zeros((1, self.spec.size, self.spec.size, 3), np.uint8)).shape[1]

    def embed(self, batch):
        """batch: uint8 (n, size, size, 3) -> float32 (n, dim), each row length 1."""
        t = self.torch
        x = t.from_numpy(np.ascontiguousarray(batch)).permute(0, 3, 1, 2).float() / 255.0
        x = (x - self.mean) / self.std
        with t.inference_mode():
            out = self.model(pixel_values=x)
            k = self.spec.kind
            if k == "dino":
                h = out.last_hidden_state
                a, b = h[:, 0], h[:, 1:].mean(1)
                unit = lambda z: z / z.norm(dim=1, keepdim=True)  # noqa: E731
                v = {"cls": a, "mean": b, "cls+mean": t.cat([unit(a), unit(b)], 1)}[self.feature]
            elif k == "siglip":
                v = out.pooler_output
            else:
                v = out.image_embeds
            v = v / v.norm(dim=1, keepdim=True)
        return v.numpy().astype(np.float32)

"""Settings for one run, in one place: folders, the Ollama address, the two models, the photo rules.

Every other file reads `CFG` when it needs a value (never at import time), so the command line
(cli.py) can set things first. Nothing here belongs to one garden: the photo thresholds are plain
defaults, and a garden can tune them with its own calib.json in the work folder.
"""
import json
import os
from dataclasses import dataclass, field
from pathlib import Path

# The photo rules. Defaults measured on the author's test garden; calib.json may override them.
CALIB_DEFAULTS = {
    "min_frame_share": 30,          # the plant must fill at least this % of the picture
    "max_distinct_plants": 4,       # more kinds of plant in view = not a picture OF this plant
    "require_viewer_can_tell": True,
    "blur_min": 20.0,               # sharpness floor (Laplacian variance at 1000 px)
    "min_px": 500,                  # long side, smallest kept
    "max_px": 2000,                 # long side, bigger ones are shrunk once to this
}


@dataclass
class Settings:
    work: Path = Path("plant-ai-work")          # cache, state.db, logs
    out: Path = Path("plant-ai-out")            # photos (.webp) and report files
    ollama: str = "http://127.0.0.1:11434"      # only ever this computer, never a public address
    fill_model: str = "phi4"
    check_model: str = "gemma4:12b"
    min_delay: float = 3.0                      # seconds between two requests to one web site
    photos_dir: Path | None = None              # existing garden photos (for checking a current garden)
    reference_photos: list[Path] = field(default_factory=list)   # optional few-shot examples, the owner's own
    calib: dict = field(default_factory=lambda: dict(CALIB_DEFAULTS))

    def load_calib(self) -> None:
        """Merge work/calib.json (if any) over the defaults. Unknown keys are ignored."""
        p = self.work / "calib.json"
        if p.is_file():
            try:
                got = json.loads(p.read_text(encoding="utf-8"))
            except ValueError:
                return
            for k in CALIB_DEFAULTS:
                if k in got:
                    self.calib[k] = got[k]


CFG = Settings(
    ollama=os.environ.get("PLANTAI_OLLAMA", "http://127.0.0.1:11434"),
    fill_model=os.environ.get("PLANTAI_FILL_MODEL", "phi4"),
    check_model=os.environ.get("PLANTAI_CHECK_MODEL", "gemma4:12b"),
)


def use(work: Path, out: Path) -> None:
    """Point the run at its folders and make them."""
    CFG.work, CFG.out = Path(work), Path(out)
    for d in (CFG.work, CFG.work / "cache", CFG.out):
        d.mkdir(parents=True, exist_ok=True)
    CFG.load_calib()

import os
from dataclasses import dataclass
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]
TARGET_URLS = {
    "explore": "https://www.xiaohongshu.com/explore",
    "creator": "https://creator.xiaohongshu.com/login",
}
HOME_URLS = {
    "explore": TARGET_URLS["explore"],
    "creator": "https://creator.xiaohongshu.com/new/home",
}


def validate_target(target: str) -> str:
    if target not in TARGET_URLS:
        raise ValueError("target must be explore or creator")
    return target


@dataclass(frozen=True)
class BrowserSettings:
    backend: str = "nodriver"
    headless: bool = False
    executable: str | None = None
    proxy: str | None = None
    timeout: float = 25
    data_dir: Path = PROJECT_ROOT / ".local"

    @classmethod
    def from_env(cls):
        backend = os.getenv("XHS_BROWSER_BACKEND", "nodriver")
        if backend != "nodriver":
            raise ValueError("Only the nodriver backend is supported")
        timeout = float(os.getenv("XHS_BROWSER_TIMEOUT", "25"))
        if not 1 <= timeout <= 300:
            raise ValueError("XHS_BROWSER_TIMEOUT must be between 1 and 300 seconds")
        return cls(
            backend=backend,
            headless=os.getenv("XHS_BROWSER_HEADLESS", "0") == "1",
            executable=os.getenv("XHS_BROWSER_EXECUTABLE") or None,
            # Explicit browser proxy: don't accidentally inherit build/API proxies.
            proxy=os.getenv("XHS_BROWSER_PROXY") or None,
            timeout=timeout,
            data_dir=Path(os.getenv("XHS_DATA_DIR", str(PROJECT_ROOT / ".local"))).resolve(),
        )

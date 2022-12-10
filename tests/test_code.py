from pathlib import Path
from importlib.util import spec_from_file_location, module_from_spec
from types import ModuleType

from fine_coverage.tracer import Tracer


HERE = Path(__file__).parent


def load_module(path: Path) -> ModuleType:
    spec = spec_from_file_location(path.stem, path)
    mod = module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def test_trinary():
    mod = load_module(HERE / 'code.py')
    with Tracer() as tracer:
        mod.trinary()
    for locs in tracer.events:
        print(locs.file, dict.fromkeys(locs.sources()).keys())

from pathlib import Path
from importlib.util import spec_from_file_location, module_from_spec

from fine_coverage.tracer import Tracer


HERE = Path(__file__).parent


def test_trinary():
    spec = spec_from_file_location('code', HERE / 'code.py')
    mod = module_from_spec(spec)
    spec.loader.exec_module(mod)
    
    with Tracer() as tracer:
        mod.trinary()

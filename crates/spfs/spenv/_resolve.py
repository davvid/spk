from typing import Sequence, List, Optional, Dict
import os

from . import storage
from ._config import get_config


def resolve_runtime_envrionment(runtime: storage.Runtime) -> Dict[str, str]:

    packages = resolve_layers_to_packages(runtime.config.layers)
    env = resolve_packages_to_environment(packages)
    env["SPENV_RUNTIME"] = runtime.rootdir
    return env


def resolve_packages_to_environment(
    packages: Sequence[storage.Package]
) -> Dict[str, str]:

    env: Dict[str, str] = {}
    for package in packages:
        # TODO: allow extending base environment (os.expandenv)
        env.update(package.config.iter_env())
    return env


def resolve_overlayfs_options(runtime: storage.Runtime) -> str:

    config = get_config()
    repo = config.get_repository()
    lowerdirs = [runtime.lowerdir]
    packages = resolve_layers_to_packages(runtime.config.layers)
    for package in packages:
        lowerdirs.append(package.diffdir)

    return f"lowerdir={':'.join(lowerdirs)},upperdir={runtime.upperdir},workdir={runtime.workdir}"


def resolve_layers_to_packages(layers: Sequence[str]) -> List[storage.Package]:

    config = get_config()
    repo = config.get_repository()
    packages = []
    for ref in layers:

        entry = repo.read_ref(ref)
        if isinstance(entry, storage.Package):
            packages.append(entry)
        else:
            expanded = resolve_layers_to_packages(entry.layers)
            packages.extend(expanded)
    return packages


def which(name: str) -> Optional[str]:

    search_paths = os.getenv("PATH", "").split(os.pathsep)
    for path in search_paths:
        filepath = os.path.join(path, name)
        if _is_exe(filepath):
            return filepath
    else:
        return None


def _is_exe(filepath: str) -> bool:

    return os.path.isfile(filepath) and os.access(filepath, os.X_OK)

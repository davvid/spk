from typing import List, Union, Dict, Iterable, Tuple
import os
import uuid
import errno
import shutil
import tarfile
import hashlib

from ._platform import PlatformStorage, Platform, UnknownPlatformError
from ._package import PackageStorage, Package
from ._runtime import RuntimeStorage, Runtime
from ._layer import Layer


class Repository:

    _pack = "pack"
    _plat = "plat"
    _tag = "tags"
    _run = "run"
    dirs = (_pack, _plat, _tag, _run)

    def __init__(self, root: str):

        self._root = os.path.abspath(root)
        self.packages = PackageStorage(self._join_path(self._pack))
        self.platforms = PlatformStorage(self._join_path(self._plat))
        self.runtimes = RuntimeStorage(self._join_path(self._run))

    def _join_path(self, *parts: str) -> str:

        return os.path.join(self._root, *parts)

    def read_ref(self, ref: str) -> Union[Layer, Runtime]:

        tag_path = self._join_path(self._tag, ref)
        try:
            with open(tag_path, "r", encoding="ascii") as f:
                ref = f.read().strip()
        except OSError as e:
            if e.errno == errno.ENOENT:
                pass
            else:
                raise

        try:
            return self.packages.read_package(ref)
        except ValueError:
            pass

        try:
            return self.platforms.read_platform(ref)
        except UnknownPlatformError:
            pass

        try:
            return self.runtimes.read_runtime(ref)
        except ValueError:
            pass

        raise ValueError("Unknown ref: " + ref)

    def find_aliases(self, ref: str) -> List[str]:

        ref = self.read_ref(ref).ref
        aliases = set([ref])
        for tag, target in self.iter_tags():
            if target == ref:
                aliases.add(tag)
        aliases.remove(ref)
        return list(aliases)

    def iter_tags(self) -> Iterable[Tuple[str, str]]:

        tag_dir = self._join_path(self._tag)
        for root, _, files in os.walk(tag_dir):

            for filename in files:
                linkfile = os.path.join(root, filename)
                with open(linkfile, "r", encoding="ascii") as f:
                    ref = f.read().strip()
                tag = os.path.relpath(linkfile, tag_dir)
                yield (tag, ref)

    def commit_package(self, runtime: Runtime, env: Dict[str, str] = None) -> Package:
        """Commit the working file changes of a runtime to a new package."""

        return self.packages.commit_dir(runtime.upperdir, env=env)

    def commit_platform(self, runtime: Runtime, env: Dict[str, str] = None) -> Platform:
        """Commit the full layer stack and working files to a new platform."""

        top = self.packages.commit_dir(runtime.upperdir, env=env)
        runtime.append_package(top)
        return self.platforms.commit_runtime(runtime)

    def tag(self, ref: str, tag: str) -> None:

        layer = self.read_ref(ref)
        tagdir = self._join_path(self._tag)
        linkfile = os.path.join(tagdir, tag)
        os.makedirs(os.path.dirname(linkfile), exist_ok=True)
        with open(linkfile, "w+", encoding="ascii") as f:
            f.write(layer.ref)


def ensure_repository(path: str) -> Repository:

    os.makedirs(path, exist_ok=True, mode=0o777)
    for subdir in Repository.dirs:
        os.makedirs(os.path.join(path, subdir), exist_ok=True, mode=0o777)

    return Repository(path)

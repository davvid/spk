import shutil

import structlog

from . import storage, tracking
from ._config import get_config

_logger = structlog.get_logger(__name__)


def push_ref(ref: str, remote_name: str) -> storage.Object:

    config = get_config()
    local = config.get_repository()
    remote = config.get_remote(remote_name)
    return sync_ref(ref, local, remote)


def pull_ref(ref: str) -> storage.Object:
    """Pull a reference to the local repository, searching all configured remotes.

    Args:
        ref (str): The reference to localize

    Raises:
        ValueError: If the remote ref could not be found
    """

    config = get_config()
    local = config.get_repository()
    for name in config.list_remote_names():
        remote = config.get_remote(name)
        try:
            remote.read_object(ref)
        except ValueError:
            continue
        return sync_ref(ref, remote, local)
    else:
        raise ValueError("Unknown ref: " + ref)


def sync_ref(
    ref: str, src: storage.Repository, dest: storage.Repository
) -> storage.Object:

    obj = src.read_object(ref)
    sync_object(obj, src, dest)
    if obj.digest != ref:
        dest.push_tag(ref, obj.digest)
    return obj


def sync_object(
    obj: storage.Object, src: storage.Repository, dest: storage.Repository
) -> None:

    if isinstance(obj, storage.Layer):
        sync_layer(obj, src, dest)
    elif isinstance(obj, storage.Platform):
        sync_platform(obj, src, dest)
    else:
        raise NotImplementedError("Push: Unhandled object of type: " + str(type(obj)))


def sync_platform(
    platform: storage.Platform, src: storage.Repository, dest: storage.Repository
) -> None:

    if dest.has_platform(platform.digest):
        _logger.info("platform exists locally", digest=platform.digest)
        return
    _logger.info("syncing platform", digest=platform.digest)
    for ref in platform.stack:
        sync_ref(ref, src, dest)

    dest.write_platform(platform)


def sync_layer(
    layer: storage.Layer, src: storage.Repository, dest: storage.Repository
) -> None:

    if dest.has_layer(layer.digest):
        _logger.info("layer exists locally", digest=layer.digest)
        return

    _logger.info("syncing layer", digest=layer.digest)
    total_entries = len(layer.manifest.entries)
    processed_entry_count = -1
    for _, entry in layer.manifest.walk():

        processed_entry_count += 1
        if processed_entry_count % 100 == 0:
            _logger.info(
                f"syncing layer data [{processed_entry_count}/{total_entries}]"
            )

        if entry.kind is not tracking.EntryKind.BLOB:
            continue
        if dest.has_blob(entry.object):
            _logger.debug("blob exists locally", digest=entry.object)
            continue
        with src.open_blob(entry.object) as blob:
            _logger.debug("syncing blob", digest=entry.object)
            dest.write_blob(blob)

    dest.write_layer(layer)

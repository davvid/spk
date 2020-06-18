from typing import Union, Optional, Tuple
from dataclasses import dataclass, field
import unicodedata

from ruamel import yaml

from ._version import Version, parse_version
from ._build import Build, parse_build
from ._name import validate_name


@dataclass
class Ident:
    """Ident represents a package identifier.

	The identifier is either a specific package or
	range of package versions/releases depending on the
	syntax and context
	"""

    name: str
    version: Version = field(default_factory=Version)
    build: Optional[Build] = None

    def __str__(self) -> str:

        out = self.name
        if self.version:
            out += "/" + str(self.version)
        if self.build:
            out += "/" + self.build.digest
        return out

    __repr__ = __str__

    def clone(self) -> "Ident":
        """Create a copy of this identifier."""

        return parse_ident(str(self))

    def with_build(self, build: Union[Build, str, None]) -> "Ident":
        """Return a copy of this identifier with the given build replaced"""

        if build is None or build == "":
            return parse_ident(f"{self.name}/{self.version}")

        return parse_ident(f"{self.name}/{self.version}/{build}")

    def parse(self, source: str) -> None:
        """Parse the given didentifier string into this instance."""

        name, version, build, *other = str(source).split("/") + ["", ""]

        if any(other):
            raise ValueError(f"Too many tokens in identifier: {source}")

        self.name = validate_name(name)
        self.version = parse_version(version)
        self.build = parse_build(build) if build else None


def parse_ident(source: str) -> Ident:
    """Parse a package identifier string."""
    ident = Ident("")
    ident.parse(source)
    return ident


yaml.Dumper.add_representer(
    Ident,
    lambda dumper, data: yaml.representer.SafeRepresenter.represent_str(
        dumper, str(data)
    ),
)

yaml.SafeDumper.add_representer(
    Ident,
    lambda dumper, data: yaml.representer.SafeRepresenter.represent_str(
        dumper, str(data)
    ),
)

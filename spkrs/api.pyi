from typing import (
    Any,
    Dict,
    List,
    Optional,
    Set,
    Tuple,
    Union,
    overload,
)
import typing

EMBEDDED: str
SRC: str
COMPATIBLE: Compatibility
VERSION_SEP: str

def opt_from_dict(input: Dict[str, Any]) -> Option: ...
def opt_from_request(input: Request) -> Option: ...
def request_from_dict(input: Dict[str, Any]) -> Request: ...
def parse_ident(v: str) -> Ident: ...
def parse_version(v: str) -> Version: ...
def parse_compat(v: str) -> str: ...
def parse_ident_range(v: str) -> RangeIdent: ...
def parse_version_range(v: str) -> VersionRange: ...
def host_options() -> OptionMap: ...
def validate_name(name: str) -> None: ...
def render_compat(compat: str, version: Version) -> str: ...
def read_spec_file(filename: str) -> Spec: ...
def save_spec_file(filename: str, spec: Spec) -> None: ...
def version_range_is_satisfied_by(
    range: VersionRange,
    spec: Spec,
    required: Optional[str],
) -> Compatibility: ...

class Compatibility:
    def __init__(self, msg: str = "") -> None: ...

class Ident:
    version: Version
    build: Optional[str]
    @property
    def name(Self) -> str: ...
    def __init__(
        self, name: str, version: Version = None, build: str = None
    ) -> None: ...
    def is_source(self) -> bool: ...
    def set_build(self, build: Optional[str]) -> None: ...
    def with_build(self, build: Optional[str]) -> Ident: ...

class Spec:
    pkg: Ident
    compat: str
    deprecated: bool
    sources: List[SourceSpec]
    build: BuildSpec
    tests: List[TestSpec]
    install: InstallSpec
    @staticmethod
    def from_dict(input: Dict[str, Any]) -> Spec: ...
    def to_dict(self) -> Dict[str, Any]: ...
    def __init__(self) -> None: ...
    def copy(self) -> Spec: ...
    def resolve_all_options(self, given: OptionMap) -> OptionMap: ...
    def sastisfies_request(self, request: Request) -> Compatibility: ...
    def satisfies_var_request(self, request: VarRequest) -> Compatibility: ...
    def satisfies_pkg_request(self, request: PkgRequest) -> Compatibility: ...
    def update_spec_for_build(
        self, options: OptionMap, resolved: List[Spec]
    ) -> None: ...

class BuildSpec:
    script: List[str]
    options: List[Option]
    variants: List[OptionMap]
    def __init__(self, options: List[Option]) -> None: ...
    def resolve_all_options(
        self, package_name: Optional[str], given: OptionMap
    ) -> OptionMap: ...
    def validate_options(
        self,
        package_name: str,
        given_options: OptionMap,
    ) -> Compatibility: ...
    def upsert_opt(self, opt: Option) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class ComponentSpec:
    name: str
    requirements: List[Request]
    embedded: List[Spec]
    uses: List[str]

class InstallSpec:
    requirements: List[Request]
    embedded: List[Spec]
    components: List[ComponentSpec]
    environment: List[EnvOp]
    def upsert_requirement(self, request: Request) -> None: ...

class RangeIdent:
    version: VersionRange
    components: Set[str]
    build: Optional[str]
    @property
    def name(self) -> str: ...
    def is_satisfied_by(self, spec: Spec, required: str) -> Compatibility: ...

class PkgRequest:
    pkg: RangeIdent
    pin: Optional[str]
    prerelease_policy: str
    inclusion_policy: str
    required_compat: str
    @staticmethod
    def from_dict(input: Dict[str, Any]) -> PkgRequest: ...
    @staticmethod
    def from_ident(pkg: Ident) -> PkgRequest: ...
    def __init__(self, pkg: RangeIdent, prerelease_policy: str = None) -> None: ...
    def copy(self) -> PkgRequest: ...
    def is_version_applicable(self, version: Version) -> Compatibility: ...
    def is_satisfied_by(self, spec: Spec) -> Compatibility: ...
    def restrict(self, other: PkgRequest) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class VarRequest:
    var: str
    pin: bool
    @property
    def value(self) -> str: ...
    def __init__(self, var: str, value: str = "") -> None: ...
    def package(self) -> Optional[str]: ...

Request = Union[PkgRequest, VarRequest]

class PkgOpt:
    default: str
    prerelease_policy: str
    @property
    def pkg(self) -> str: ...
    @property
    def value(self) -> Optional[str]: ...
    def __init__(self, pkg: str, value: str = None) -> None: ...
    def copy(self) -> PkgOpt: ...
    def name(self) -> str: ...
    def to_request(self, given_value: str = None) -> PkgRequest: ...
    def namespaced_name(self, pkg: str) -> str: ...
    @overload
    def get_value(self, given: None = None) -> Optional[str]: ...
    @overload
    def get_value(self, given: str) -> str: ...
    def set_value(self, value: str) -> None: ...
    def validate(self, value: Optional[str]) -> Compatibility: ...

class VarOpt:
    var: str
    default: str
    inheritance: str
    choices: Set[str]
    @property
    def value(self) -> Optional[str]: ...
    def __init__(self, name: str, value: str = None) -> None: ...
    def copy(self) -> PkgOpt: ...
    def name(self) -> str: ...
    def to_request(self, given_value: str = None) -> VarRequest: ...
    def namespaced_name(self, pkg: str) -> str: ...
    @overload
    def get_value(self, given: None = None) -> Optional[str]: ...
    @overload
    def get_value(self, given: str) -> str: ...
    def set_value(self, value: str) -> None: ...
    def validate(self, value: Optional[str]) -> Compatibility: ...

Option = Union[PkgOpt, VarOpt]

class TestSpec:
    stage: str
    script: List[str]
    selectors: List[OptionMap]
    requirements: List[Request]

class TagSet:
    def __getitem__(self, k: str) -> int: ...
    def __setitem__(self, k: str, v: int) -> None: ...
    def __delitem__(self, k: str) -> None: ...
    def __len__(self) -> int: ...

class Version:
    major: int
    minor: int
    patch: int
    tail: List[int]
    pre: TagSet
    post: TagSet
    def __init__(self, major: int = 0, minor: int = 0, patch: int = 0) -> None: ...
    def __lt__(self, other: Version) -> bool: ...
    @property
    def parts(self) -> List[int]: ...
    @property
    def base(self) -> str: ...
    def is_zero(self) -> bool: ...

class LocalSource:
    exclude: List[str]
    filter: List[str]
    subdir: Optional[str]
    @staticmethod
    def from_dict(input: Dict[str, Any]) -> LocalSource: ...
    @property
    def path(self) -> str: ...

class GitSource:
    git: str
    reference: str
    depth: int
    subdir: Optional[str]
    @staticmethod
    def from_dict(input: Dict[str, Any]) -> GitSource: ...

class TarSource:
    tar: str
    subdir: Optional[str]
    @staticmethod
    def from_dict(input: Dict[str, Any]) -> TarSource: ...

class ScriptSource:
    script: List[str]
    subdir: Optional[str]
    @staticmethod
    def from_dict(input: Dict[str, Any]) -> TarSource: ...

SourceSpec = Union[LocalSource, GitSource, TarSource, ScriptSource]

class OptionMap:
    @typing.overload
    def __init__(self, data: Dict[str, str]) -> None: ...
    @typing.overload
    def __init__(self, **data: str) -> None: ...
    @typing.overload
    def get(self, default: str) -> str: ...
    @typing.overload
    def get(self, default: None = None) -> Optional[str]: ...
    def copy(self) -> OptionMap: ...
    def update(self, other: OptionMap) -> None: ...
    def to_environment(self) -> Dict[str, str]: ...
    def items(self) -> List[Tuple[str, str]]: ...
    def __getitem__(self, k: str) -> str: ...
    def __setitem__(self, k: str, v: str) -> None: ...
    def __delitem__(self, k: str) -> None: ...
    def __len__(self) -> int: ...
    def __contains__(self, item: str) -> bool: ...
    @property
    def digest(self) -> str: ...

class SemverRange: ...
class WildcardRange: ...
class LowestSpecifiedRange: ...
class GreaterThanRange: ...
class LessThanRange: ...
class GreaterThanOrEqualToRange: ...
class LessThanOrEqualToRange: ...
class ExactVersion: ...
class ExcludedVersion: ...
class CompatRange: ...
class VersionFilter: ...

VersionRange = Union[
    SemverRange,
    WildcardRange,
    LowestSpecifiedRange,
    GreaterThanRange,
    LessThanRange,
    GreaterThanOrEqualToRange,
    LessThanOrEqualToRange,
    ExactVersion,
    ExcludedVersion,
    CompatRange,
    VersionFilter,
]

class AppendEnv:
    append: str
    value: str
    separator: Optional[str]
    def sep(self) -> str: ...
    def bash_source(self) -> str: ...
    def tcsh_source(self) -> str: ...

class PrependEnv:
    prepend: str
    value: str
    separator: Optional[str]
    def sep(self) -> str: ...
    def bash_source(self) -> str: ...
    def tcsh_source(self) -> str: ...

class SetEnv:
    set: str
    value: str
    def bash_source(self) -> str: ...
    def tcsh_source(self) -> str: ...

EnvOp = Union[AppendEnv, PrependEnv, SetEnv]

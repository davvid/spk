from . import api, solve

class BinaryPackageBuilder:
    @staticmethod
    def from_spec(spec: api.Spec) -> BinaryPackageBuilder: ...
    def get_solve_graph(self) -> solve.Graph: ...
    def build(self) -> api.Spec: ...

class SourcePackageBuilder:
    @staticmethod
    def from_spec(spec: api.Spec) -> SourcePackageBuilder: ...
    def build(self) -> api.Ident: ...

def validate_build_changeset() -> None: ...
def validate_source_changeset() -> None: ...
def build_options_path(pkg: api.Ident, prefix: str = None) -> str: ...
def build_script_path(pkg: api.Ident, prefix: str = None) -> str: ...
def build_spec_path(pkg: api.Ident, prefix: str = None) -> str: ...
def source_package_path(pkg: api.Ident, prefix: str = None) -> str: ...

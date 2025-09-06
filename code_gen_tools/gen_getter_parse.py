from typing import List, Optional, Tuple


class RustStruct:
    def __init__(self) -> None:
        self.struct_name: str
        self.attrs: List[Tuple[str, str]]


struct_start = "pub struct "
struct_final = "{"
attr_start = ["pub ", "pub(crate) ", "pub(super) "]
attr_middl = ": "
attr_final = ","


def parse(data: str) -> List[RustStruct]:
    result: List[RustStruct] = []
    tmp: Optional[RustStruct] = None

    for line in data.splitlines():
        if not line:
            continue
        if line.strip().startswith("#"):
            continue
        if line.strip().startswith("//"):
            continue

        the_line = line.strip()
        if the_line.startswith(struct_start) and the_line.endswith(struct_final):
            tmp = RustStruct()
            tmp.struct_name = the_line[len(struct_start) : -len(struct_final)].strip()
            tmp.attrs = []
        elif tmp and the_line.endswith(attr_final):
            a = the_line[: -len(attr_final)]
            for ast in attr_start:
                if a.startswith(ast):
                    a = a[len(ast):]
            mid_index = a.index(attr_middl)
            attr_name = a[:mid_index]
            attr_type = a[mid_index + len(attr_middl) :]
            tmp.attrs.append((attr_name, attr_type))
        elif tmp and the_line == "}":
            result.append(tmp)
            tmp = None

    return result


test_data = """
use godot::prelude::*;

pub use auto_impl::*;

#[derive(Default)]
pub struct Qwe {
    /// xxxx
    pub aaaa: String,
    /// xxxx
    pub ssss: String,
    /// xxxxx
    pub dddd: f64,
}
"""


def test():
    from gen_getter_print import print_sss

    ll = parse(test_data)
    print_sss(ll)


if __name__ == "__main__":
    test()

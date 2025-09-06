from typing import List
from gen_getter_parse import RustStruct


mod_start = """
mod auto_impl {
    use super::*;
"""

mod_final = """
}
"""

trait_start = """
    pub trait Proxy{[(<struct_name>)]} {
        fn as_self(&self) -> &{[(<struct_name>)]};
        fn as_mut_self(&mut self) -> &mut {[(<struct_name>)]};
"""

trait_final = """
    }
"""

getter_copy_type = """
        fn get_{[(<attr_name>)]}(&self) -> {[(<attr_type>)]} {
            self.as_self().{[(<attr_name>)]}
        }
"""

setter_copy_type = """
        fn set_{[(<attr_name>)]}(&mut self, v: {[(<attr_type>)]}) {
            self.as_mut_self().{[(<attr_name>)]} = v
        }
"""

getter_normal_type = """
        fn get_{[(<attr_name>)]}(&self) -> &{[(<attr_type>)]} {
            &self.as_self().{[(<attr_name>)]}
        }
"""

setter_normal_type = """
        fn set_{[(<attr_name>)]}(&mut self, v: {[(<attr_type>)]}) {
            self.as_mut_self().{[(<attr_name>)]} = v
        }
"""


def print_mod_start():
    print(mod_start, end="")


def print_mod_final():
    print(mod_final, end="")


def print_a_trait(rust_struct: RustStruct):
    struct_name = rust_struct.struct_name
    attrs = rust_struct.attrs
    print(trait_start.replace("{[(<struct_name>)]}", struct_name), end="")

    for attr in attrs:
        attr_name = attr[0]
        attr_type = attr[1]
        if attr_type in ['i64', 'f64']:
            print(
                getter_copy_type.replace("{[(<attr_name>)]}", attr_name).replace(
                    "{[(<attr_type>)]}", attr_type
                ),
                end="",
            )
            print(
                setter_copy_type.replace("{[(<attr_name>)]}", attr_name).replace(
                    "{[(<attr_type>)]}", attr_type
                ),
                end="",
            )
        else:
            print(
                getter_normal_type.replace("{[(<attr_name>)]}", attr_name).replace(
                    "{[(<attr_type>)]}", attr_type
                ),
                end="",
            )
            print(
                setter_normal_type.replace("{[(<attr_name>)]}", attr_name).replace(
                    "{[(<attr_type>)]}", attr_type
                ),
                end="",
            )

    print(trait_final, end="")


def print_sss(ll: List[RustStruct]):
    print_mod_start()
    for rust_struct in ll:
        print_a_trait(rust_struct)
    print_mod_final()


def test():
    rust_struct = RustStruct()
    rust_struct.struct_name = "Test"
    rust_struct.attrs = [("a", "f64"), ("b", "String")]

    print_mod_start()
    print_a_trait(rust_struct)
    print_mod_final()


if __name__ == "__main__":
    test()

pub mod common;

use crate::common::common_helper;

#[test]
fn test_func() {
    assert_eq!(1, common_helper::V);
}

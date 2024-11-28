mod name {
    include!(concat!(env!("OUT_DIR"), "/rc_anim_names_test.rs"));
}
mod root {
    include!(concat!(env!("OUT_DIR"), "/rc_anim_root_names_test.rs"));
}

use super::{anim_name_fwd, anim_name_rev, anim_root_name_fwd, anim_root_name_rev};
use crate::tests::test;

test!(name, name::ALL, anim_name_fwd, anim_name_rev);
test!(root, root::ALL, anim_root_name_fwd, anim_root_name_rev);

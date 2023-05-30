use super::defs::{Stab, StabInfo};

fn stab_binsearch(
    stabs: *const Stab,
    region_left: usize,
    region_right: usize,
    _type: u8,
    addr: usize,
) -> (usize, usize) {
    let mut left: usize = region_left;
    let mut right: usize = region_right;
    let mut final_region_left: usize = region_left;
    let mut final_region_right: usize = region_right;
    let mut found: bool = false;

    while (left <= right) {
        let middle_point = (left + right) / 2;
        let mut m = middle_point;

        unsafe {
            while (m >= left && (*stabs.offset(m as isize)).n_type != _type) {
                m -= 1;
            }
        }

        if (m < left) {
            left = middle_point + 1;
            continue;
        }

        unsafe {
            found = true;
            if ((*stabs.offset(m as isize)).n_value < addr) {
                final_region_left = m;
                left = middle_point + 1;
            } else if ((*stabs.offset(m as isize)).n_value > addr) {
                final_region_right = m - 1;
                right = middle_point - 1;
            } else {
                final_region_left = middle_point;
                left = middle_point;
                addr += 1;
            }

            if (!found) {
                final_region_right = final_region_left - 1;
            } else {
                left = final_region_right;
                while (left > final_region_left && (*stabs.offset(left as isize)).n_type != _type) {
                    left -= 1;
                }
                final_region_left = left;
            }
        }
    }

    return (final_region_left, final_region_right);
}

fn stab_info(addr: usize) -> StabInfo {
    let info: StabInfo = StabInfo {
        eip_file: "<unknown>",
        eip_line: 0,
        eip_fn_name: "<unknown>",
        eip_fn_namelen: 9,
        eip_fn_addr: addr,
        eip_fn_narg: 0,
    };

    return info;
}

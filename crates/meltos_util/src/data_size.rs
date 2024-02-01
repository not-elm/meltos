#[inline]
pub const fn gib(num_mib: u64) -> u64{
    mib(num_mib) * 1024
}

#[inline]
pub const fn mib(num_mib: u64) -> u64{
    kib(num_mib) * 1024
}


#[inline]
pub const fn kib(num_kib: u64) -> u64{
    num_kib * 1024
}


#[inline]
pub const fn byte_to_gib(bytes: u64) -> u64{
    byte_to_mib(bytes) / 1024
}


#[inline]
pub const fn byte_to_mib(bytes: u64) -> u64{
    byte_to_kib(bytes) / 1024
}


#[inline]
pub const fn byte_to_kib(bytes: u64) -> u64{
    bytes / 1024
}



#[cfg(test)]
mod tests{
    use crate::data_size::gib;

    #[test]
    fn it_calc_1gib(){
        assert_eq!(gib(1), 1024 * 1024 * 1024);
    }
}
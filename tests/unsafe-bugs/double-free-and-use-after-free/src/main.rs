// Proof-of-concept of several double-free vulnerabilities (CVE-2018-20996, CVE-2019-16880, CVE-2019-16144, CVE-2019-16881)
// Not very similar to those real CVEs but should be enough for illustration purpose

fn genvec1() -> Vec<u8> {
    let mut s = String::from("a_tmp_string");
    let ptr = s.as_mut_ptr();
    let v;
    unsafe{
        v = Vec::from_raw_parts(ptr, s.len(), s.len());
    }
    // std::mem::forget(s); // do not drop s
    // otherwise, s is dropped before return
    return v;
}

fn genvec2() -> Vec<u8> {
    let mut s = String::from("a_tmp_string");
    let ptr = s.as_mut_ptr();
    let v;
    unsafe{
        v = Vec::from_raw_parts(ptr, s.len(), s.len());
    }
    std::mem::forget(s); // do not drop s
    // otherwise, s is dropped before return
    return v;
}
fn main(){
    let v=genvec1();
    let u=genvec2();
    // use v -> use after free
    // drop v before return -> double free
}